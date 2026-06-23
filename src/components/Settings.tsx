import { useState, useEffect } from 'react'

interface Props {
  onLogout: () => void
}

export default function Settings({ onLogout }: Props) {
  const [config, setConfig] = useState<any>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => { loadConfig() }, [])

  async function loadConfig() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const c = await invoke('get_config')
      setConfig(c as any)
    } catch { }
  }

  async function handleClearLog() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('clear_sync_log')
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch { }
  }

  if (!config) return <div className="loading">Loading...</div>

  const folders = config.watched_folders?.length > 0
    ? config.watched_folders
    : config.watched_folder
      ? [config.watched_folder]
      : []

  return (
    <div className="settings">
      <div className="setting-card">
        <h3>Akun</h3>
        <p>Masuk sebagai: <strong>{config.last_email || 'Belum login'}</strong></p>
        <p>Server: {config.server_url}</p>
        <button className="btn-danger" onClick={onLogout}>Logout</button>
      </div>

      <div className="setting-card">
        <h3>Folder yang Dipantau</h3>
        {folders.length > 0 ? (
          <div className="folder-list">
            {folders.map((f: string, i: number) => (
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
        <p>DMS Sync v1.1.0</p>
        <p>Desktop sync client untuk Arsipin DMS</p>
      </div>
    </div>
  )
}
