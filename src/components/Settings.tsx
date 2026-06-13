import { useState, useEffect } from 'react'

interface Props {
  onLogout: () => void
}

export default function Settings({ onLogout }: Props) {
  const [config, setConfig] = useState<any>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    loadConfig()
  }, [])

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

  if (!config) return <div className="settings">Loading...</div>

  return (
    <div className="settings">
      <div className="setting-group">
        <h3>Account</h3>
        <p>Logged in as: <strong>{config.last_email || 'Not logged in'}</strong></p>
        <button className="danger-btn" onClick={onLogout}>Logout</button>
      </div>

      <div className="setting-group">
        <h3>Sync</h3>
        <p>Watched folder: <strong>{config.watched_folder || 'Not set'}</strong></p>
        <p>Server: {config.server_url}</p>
      </div>

      <div className="setting-group">
        <h3>Maintenance</h3>
        <button onClick={handleClearLog}>Clear Sync Log</button>
        {saved && <span className="saved-msg">Cleared!</span>}
      </div>

      <div className="setting-group about">
        <h3>About</h3>
        <p>DMS Sync v1.0.0</p>
        <p>Desktop sync client for Arsipin DMS</p>
      </div>
    </div>
  )
}
