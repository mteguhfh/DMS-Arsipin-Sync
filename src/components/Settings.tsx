import { useState, useEffect } from 'react'
import type { AppConfig } from '../types'

interface Props {
  onLogout: () => void
}

export default function Settings({ onLogout }: Props) {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const c = await invoke<AppConfig>('get_config')
        if (!cancelled) setConfig(c)
      } catch {
        if (!cancelled) setConfig(null)
      }
    })()
    return () => { cancelled = true }
  }, [])

  async function handleClearLog() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('clear_sync_log')
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch {
      setSaved(false)
    }
  }

  if (!config) return <div className="loading">Loading...</div>

  const folders = config.watchedFolders?.length > 0
    ? config.watchedFolders
    : config.watchedFolder
      ? [config.watchedFolder]
      : []

  return (
    <div className="settings">
      <div className="setting-card">
        <h3>Akun</h3>
        <p>Masuk sebagai: <strong>{config.lastEmail || 'Belum login'}</strong></p>
        <p>Server: {config.serverUrl}</p>
        <button className="btn-danger" onClick={onLogout}>Logout</button>
      </div>

      <div className="setting-card">
        <h3>Folder yang Dipantau</h3>
        {folders.length > 0 ? (
          <div className="folder-list">
            {folders.map((f, i) => (
              <div key={i} className="folder-item" style={{ border: 'none', padding: '4px 0', cursor: 'default' }}>
                <span className="folder-icon">📁</span>
                <span className="folder-name" style={{ fontSize: '12px' }}>{f}</span>
              </div>
            ))}
          </div>
        ) : (
          <p style={{ color: '#64748b', fontStyle: 'italic', fontSize: '12px' }}>Belum ada folder</p>
        )}
      </div>

      <div className="setting-card">
        <h3>Perawatan</h3>
        <button onClick={handleClearLog} className="btn-secondary">Hapus Log Sinkronisasi</button>
        {saved && <span className="saved-msg">Dibersihkan!</span>}
      </div>

      <div className="setting-card about">
        <h3>Tentang</h3>
        <p>Versi {__APP_VERSION__}</p>
        <p>Desktop sync client untuk Arsipin DMS</p>
      </div>
    </div>
  )
}
