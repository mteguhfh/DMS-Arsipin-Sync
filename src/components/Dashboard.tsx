import { useState, useEffect, useCallback } from 'react'

interface StatusData {
  status?: string
  queue_length?: number
  watched_folder?: string
  server_url?: string
  last_email?: string
  api_available?: boolean
  event_count?: number
}

interface Props {
  status: StatusData
}

export default function Dashboard({ status }: Props) {
  const [logs, setLogs] = useState<any[]>([])
  const [currentStatus, setCurrentStatus] = useState(status)
  const [folderInput, setFolderInput] = useState('')
  const [error, setError] = useState('')
  const [sessionOk, setSessionOk] = useState<boolean | null>(null)
  const [testingSession, setTestingSession] = useState(false)
  const [testResult, setTestResult] = useState<string | null>(null)
  const [testingSync, setTestingSync] = useState(false)
  const [watcherEvents, setWatcherEvents] = useState<string[]>([])
  const [showDebug, setShowDebug] = useState(false)

  const refresh = useCallback(async (debug?: boolean) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const s = await invoke('get_status')
      setCurrentStatus(s as any)
      const l = await invoke('get_sync_log')
      setLogs(l as any[])
      if ((s as any)?.watched_folder) setFolderInput((s as any).watched_folder)
      if (debug || showDebug) {
        const ev = await invoke('get_watcher_events')
        setWatcherEvents(ev as string[])
      }
    } catch (err: any) {
      console.error('refresh error:', err)
    }
  }, [showDebug])

  useEffect(() => {
    refresh()
    const interval = setInterval(refresh, 5000)
    return () => clearInterval(interval)
  }, [refresh])

  // Auto-start watcher on mount if saved watched folder exists
  useEffect(() => {
    const folder = currentStatus?.watched_folder
    if (folder) {
      ;(async () => {
        try {
          const { invoke } = await import('@tauri-apps/api/core')
          await invoke('set_watch_folder', { path: folder })
          setError('')
          refresh()
        } catch (err: any) {
          setError(err.message || 'Failed to auto-start watcher')
        }
      })()
    }
  }, [])

  async function testSyncUpload() {
    setTestingSync(true)
    setTestResult(null)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('test_sync')
      setTestResult(result as string)
      refresh()
    } catch (err: any) {
      setTestResult('ERROR: ' + (typeof err === 'string' ? err : err.message || 'Unknown'))
    } finally {
      setTestingSync(false)
    }
  }

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({ directory: true, multiple: false, title: 'Pilih Folder untuk Sync' })
      if (selected) {
        setFolderInput(selected)
      }
    } catch (err: any) {
      setError(err.message || 'Failed to open folder picker')
    }
  }

  async function testConnection() {
    setTestingSession(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const session = await invoke('check_session')
      setSessionOk(true)
      setError('')
      console.log('Session:', session)
    } catch (err: any) {
      setSessionOk(false)
      setError('Session error: ' + (typeof err === 'string' ? err : err.message || 'Unknown'))
    } finally {
      setTestingSession(false)
    }
  }

  async function handleSetFolder() {
    if (!folderInput.trim()) return
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('set_watch_folder', { path: folderInput.trim() })
      setError('')
      refresh()
    } catch (err: any) {
      setError(err.message || 'Failed to set folder')
    }
  }

  const isSyncing = currentStatus?.status === 'Syncing'
  const isIdle = currentStatus?.status === 'Idle'

  return (
    <div className="dashboard">
      <div className="status-card">
        <div className={`status-indicator ${isSyncing ? 'syncing' : isIdle ? 'idle' : 'error'}`}>
          {isSyncing ? '⏳' : isIdle ? '✓' : '!'}
        </div>
        <div className="status-info">
          <strong>Status: {currentStatus?.status || 'Unknown'}</strong>
          <span>Queue: {currentStatus?.queue_length || 0} files</span>
          <span>Server: {currentStatus?.server_url}</span>
          <span>Account: {currentStatus?.last_email}</span>
        </div>
      </div>

      <div className="connection-section">
        <h3>Connection</h3>
        <div className="btn-row">
          <button onClick={testConnection} disabled={testingSession}>
            {testingSession ? 'Testing...' : 'Test Session'}
          </button>
          <button onClick={testSyncUpload} disabled={testingSync}>
            {testingSync ? 'Uploading...' : 'Test Sync Upload'}
          </button>
        </div>
        {sessionOk === true && <span className="ok-msg">Session OK</span>}
        {sessionOk === false && <span className="err-msg">Session Invalid</span>}
        {testResult && <pre className="test-result">{testResult}</pre>}
        <div className="status-details">
          <span>API ready: {currentStatus?.api_available ? '✓' : '✗'}</span>
          <span>Events: {currentStatus?.event_count || 0}</span>
          <button className="small-btn" onClick={() => setShowDebug(!showDebug)}>
            {showDebug ? 'Hide Debug' : 'Debug'}
          </button>
        </div>
        {showDebug && (
          <div className="debug-panel">
            <h4>Watcher Events</h4>
            <pre className="debug-events">
              {watcherEvents.length === 0 ? '(no events)' : watcherEvents.join('\n')}
            </pre>
          </div>
        )}
      </div>

      <div className="folder-section">
        <h3>Watched Folder</h3>
        <div className="folder-input-row">
          <input
            type="text"
            value={folderInput}
            onChange={(e) => setFolderInput(e.target.value)}
            placeholder={navigator.platform?.includes('Win') ? 'C:\\Users\\...\\MySyncFolder' : '/Users/.../MySyncFolder'}
            className="folder-input"
          />
          <button onClick={handleBrowse} className="browse-btn">Browse</button>
          <button onClick={handleSetFolder}>
            {currentStatus?.watched_folder ? 'Update' : 'Start Watching'}
          </button>
        </div>
      </div>

      {error && <div className="error-msg">{error}</div>}

      <div className="log-section">
        <h3>Sync Log</h3>
        <div className="log-list">
          {logs.length === 0 && <p className="empty">No sync activity yet</p>}
          {[...logs].reverse().map((log, i) => (
            <div key={i} className={`log-item ${log.status}`}>
              <span className="log-file">{log.file_path.split(/[/\\]/).pop()}</span>
              <span className="log-status">{log.status}</span>
              <span className="log-folder">folder: {log.folder_path || '(root)'}</span>
              {log.error && <span className="log-error">{log.error}</span>}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
