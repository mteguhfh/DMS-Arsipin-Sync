import { useState, useEffect, useCallback } from 'react'
import type { StatusData, SyncLog, DmsFolder } from '../types'
import { errorMessage, buildFolderPaths } from '../types'

interface Props {
  status: StatusData | null
}

const fileIcons: Record<string, string> = {
  pdf: '📄',
  doc: '📝', docx: '📝',
  xls: '📊', xlsx: '📊',
  ppt: '📽️', pptx: '📽️',
  jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', webp: '🖼️',
  txt: '📄', csv: '📊',
  mp4: '🎬', mp3: '🎵',
  zip: '📦', rar: '📦',
}

function getFileIcon(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || ''
  return fileIcons[ext] || '📄'
}

function timeAgo(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime()
  const sec = Math.floor(diff / 1000)
  if (sec < 10) return 'baru saja'
  if (sec < 60) return `${sec} detik lalu`
  const min = Math.floor(sec / 60)
  if (min < 60) return `${min} menit lalu`
  const hr = Math.floor(min / 60)
  return `${hr} jam lalu`
}

export default function Dashboard({ status }: Props) {
  const [logs, setLogs] = useState<SyncLog[]>([])
  const [currentStatus, setCurrentStatus] = useState(status)
  const [folders, setFolders] = useState<string[]>(status?.watched_folders || [])
  const [folderInput, setFolderInput] = useState('')
  const [error, setError] = useState('')
  const [sessionOk, setSessionOk] = useState<boolean | null>(null)
  const [testingSession, setTestingSession] = useState(false)
  const [testResult, setTestResult] = useState<string | null>(null)
  const [testingSync, setTestingSync] = useState(false)
  const [watcherEvents, setWatcherEvents] = useState<string[]>([])
  const [showDebug, setShowDebug] = useState(false)
  const [removing, setRemoving] = useState<string | null>(null)
  const [dmsFolders, setDmsFolders] = useState<{ id: string; path: string }[]>([])
  const [targets, setTargets] = useState<Record<string, string>>({})
  const [folderError, setFolderError] = useState('')

  const refresh = useCallback(async (debug?: boolean) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const s = await invoke<StatusData>('get_status')
      setCurrentStatus(s)
      setFolders(s.watched_folders || [])
      const l = await invoke<SyncLog[]>('get_sync_log')
      setLogs(l)
      if (debug || showDebug) {
        const ev = await invoke<string[]>('get_watcher_events')
        setWatcherEvents(ev)
      }
    } catch (err) {
      console.error('refresh error:', err)
    }
  }, [showDebug])

  useEffect(() => {
    // Poll the backend; the first run is deferred so it is not a synchronous
    // setState during the effect body.
    const interval = setInterval(() => { void refresh() }, 5000)
    const first = setTimeout(() => { void refresh() }, 0)
    return () => {
      clearTimeout(first)
      clearInterval(interval)
    }
  }, [refresh])

  // Resume watchers for folders restored from config. The backend re-adds each
  // root, and duplicate registration is rejected, so a failure here is expected
  // when the folder is already watched and must not surface to the user.
  useEffect(() => {
    const saved = status?.watched_folders || []
    if (saved.length === 0) return
    void (async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      for (const folder of saved) {
        try {
          await invoke('add_watch_folder', { path: folder })
        } catch {
          // already watching
        }
      }
      await refresh()
    })()
  }, [status?.watched_folders, refresh])

  async function testSyncUpload() {
    setTestingSync(true)
    setTestResult(null)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<string>('test_sync')
      setTestResult(result)
      await refresh()
    } catch (err) {
      setTestResult('ERROR: ' + errorMessage(err))
    } finally {
      setTestingSync(false)
    }
  }

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({ directory: true, multiple: false, title: 'Pilih Folder untuk Sync' })
      if (selected) setFolderInput(selected)
    } catch (err) {
      setError(errorMessage(err) || 'Failed to open folder picker')
    }
  }

  async function handleAddFolder() {
    if (!folderInput.trim()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('add_watch_folder', { path: folderInput.trim() })
      setFolderInput('')
      setError('')
      await refresh()
    } catch (err) {
      setError(errorMessage(err) || 'Failed to add folder')
    }
  }

  async function handleRemoveFolder(path: string) {
    setRemoving(path)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('remove_watch_folder', { path })
      setError('')
      await refresh()
    } catch (err) {
      setError(errorMessage(err) || 'Failed to remove folder')
    } finally {
      setRemoving(null)
    }
  }

  // Load the DMS folder tree and the saved per-folder targets once.
  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const list = await invoke<DmsFolder[]>('list_dms_folders')
        if (!cancelled) setDmsFolders(buildFolderPaths(list))
      } catch (err) {
        if (!cancelled) setFolderError('Gagal memuat folder DMS: ' + errorMessage(err))
      }
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const c = await invoke<{ folderTargets: { localPath: string; dmsFolderPath: string }[] }>('get_config')
        if (cancelled) return
        const map: Record<string, string> = {}
        for (const t of c.folderTargets || []) map[t.localPath] = t.dmsFolderPath
        setTargets(map)
      } catch {
        // targets stay empty; default naming applies
      }
    })()
    return () => { cancelled = true }
  }, [])

  async function handleSetTarget(localPath: string, dmsPath: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const id = dmsFolders.find(f => f.path === dmsPath)?.id || ''
      await invoke('set_folder_target', {
        path: localPath,
        dmsFolderId: id,
        dmsFolderPath: dmsPath,
      })
      setTargets(prev => {
        const next = { ...prev }
        if (dmsPath) next[localPath] = dmsPath
        else delete next[localPath]
        return next
      })
      setFolderError('')
    } catch (err) {
      setFolderError(errorMessage(err))
    }
  }

  async function handleTestSession() {
    setTestingSession(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('check_session')
      setSessionOk(true)
      setError('')
    } catch (err) {
      setSessionOk(false)
      setError('Session error: ' + errorMessage(err))
    } finally {
      setTestingSession(false)
    }
  }

  const isSyncing = currentStatus?.status === 'Syncing'
  const isIdle = currentStatus?.status === 'Idle'
  const queueLen = currentStatus?.queue_length || 0
  const errCount = logs.filter(l => l.status === 'error').length

  return (
    <div className="dashboard">
      <div className="status-bar">
        <span className={`status-dot ${isSyncing ? 'syncing' : 'idle'}`}></span>
        <span className="status-text">
          {isSyncing ? 'Menyinkronkan...' : isIdle ? 'Tersinkronisasi' : 'Error'}
        </span>
        <span className="status-detail">
          {queueLen > 0 && `${queueLen} file antrean`}
          {queueLen > 0 && errCount > 0 && ' · '}
          {errCount > 0 && `${errCount} error`}
          {queueLen === 0 && errCount === 0 && 'Tidak ada aktivitas'}
        </span>
      </div>

      <div className="section">
        <div className="section-header">
          <h2>Aktivitas Sinkronisasi</h2>
          {folders.length > 0 && <span className="badge">{folders.length} folder</span>}
        </div>
        <div className="sync-list">
          {logs.length === 0 && <p className="empty">Belum ada aktivitas sinkronisasi</p>}
          {[...logs].reverse().slice(0, 20).map((log, i) => (
            <div key={i} className={`sync-item ${log.status}`}>
              <span className="sync-icon">{getFileIcon(log.file_path)}</span>
              <div className="sync-info">
                <div className="sync-name">{log.file_path.split(/[/\\]/).pop()}</div>
                <div className="sync-meta">
                  {log.folder_path || '(root)'} · {timeAgo(log.timestamp)}
                </div>
              </div>
              <span className={`sync-status status-${log.status}`}>
                {log.status === 'success' ? '✓ Tersimpan' : '✗ Gagal'}
              </span>
            </div>
          ))}
        </div>
      </div>

      <div className="section">
        <div className="section-header">
          <h2>Folder yang Dipantau</h2>
        </div>
        {folders.length > 0 ? (
          <div className="folder-list">
            {folders.map((f, i) => (
              <div key={i} className="folder-item folder-item-target">
                <div className="folder-row">
                  <span className="folder-icon">📁</span>
                  <span className="folder-name">{f}</span>
                  <button
                    className="remove-btn"
                    onClick={() => handleRemoveFolder(f)}
                    disabled={removing === f}
                  >
                    {removing === f ? '...' : '✕'}
                  </button>
                </div>
                <label className="target-row">
                  <span>Tujuan DMS</span>
                  <select
                    value={targets[f] || ''}
                    onChange={(e) => handleSetTarget(f, e.target.value)}
                    className="target-select"
                  >
                    <option value="">
                      Otomatis (pakai nama folder: {f.split(/[/\\]/).filter(Boolean).pop()})
                    </option>
                    {dmsFolders.map(d => (
                      <option key={d.id} value={d.path}>{d.path}</option>
                    ))}
                  </select>
                </label>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty">Belum ada folder yang ditambahkan</p>
        )}
        {folderError && <p className="err-msg">{folderError}</p>}
        <div className="folder-form">
          <input
            type="text"
            value={folderInput}
            onChange={(e) => setFolderInput(e.target.value)}
            placeholder={
              navigator.userAgent.includes('Windows')
                ? 'C:\\Users\\...\\MySyncFolder'
                : '/home/.../MySyncFolder'
            }
            className="folder-input"
          />
          <button onClick={handleBrowse} className="btn-secondary">Browse</button>
          <button onClick={handleAddFolder} disabled={!folderInput.trim()} className="btn-primary">
            + Tambah
          </button>
        </div>
      </div>

      <div className="section">
        <div className="section-header">
          <h2>Koneksi</h2>
        </div>
        <div className="connection-bar">
          <span>API: {currentStatus?.api_available ? '✓' : '✗'}</span>
          <span>Events: {currentStatus?.event_count || 0}</span>
          <button onClick={() => setShowDebug(!showDebug)} className="btn-small">
            {showDebug ? 'Sembunyikan' : 'Debug'}
          </button>
        </div>
        <div className="btn-row">
          <button onClick={handleTestSession} disabled={testingSession} className="btn-secondary">
            {testingSession ? '...' : 'Test Session'}
          </button>
          <button onClick={testSyncUpload} disabled={testingSync} className="btn-secondary">
            {testingSync ? '...' : 'Test Upload'}
          </button>
        </div>
        {sessionOk === true && <span className="ok-msg">Session OK</span>}
        {sessionOk === false && <span className="err-msg">Session Invalid</span>}
        {testResult && <pre className="test-result">{testResult}</pre>}
        {showDebug && (
          <div className="debug-panel">
            <h4>Watcher Events</h4>
            <pre className="debug-events">
              {watcherEvents.length === 0 ? '(no events)' : watcherEvents.join('\n')}
            </pre>
          </div>
        )}
      </div>

      {error && <div className="error-banner">{error}</div>}
    </div>
  )
}
