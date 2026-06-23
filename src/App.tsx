import { useState, useEffect } from 'react'
import LoginPage from './components/LoginPage'
import Dashboard from './components/Dashboard'
import Settings from './components/Settings'
import './App.css'

type Page = 'login' | 'dashboard' | 'settings'
type Theme = 'light' | 'dark'

function App() {
  const [page, setPage] = useState<Page>('login')
  const [status, setStatus] = useState<any>(null)
  const [theme, setTheme] = useState<Theme>(() => {
    return (localStorage.getItem('dms-sync-theme') as Theme) || 'light'
  })

  useEffect(() => {
    checkLogin()
  }, [])

  useEffect(() => {
    localStorage.setItem('dms-sync-theme', theme)
  }, [theme])

  function toggleTheme() {
    setTheme(t => t === 'light' ? 'dark' : 'light')
  }

  async function checkLogin() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const s = await invoke('get_status')
      setStatus(s as any)
      if (s) {
        setPage('dashboard')
      }
    } catch {
      setPage('login')
    }
  }

  async function handleLogout() {
    setPage('login')
    setStatus(null)
  }

  if (page === 'login') {
    return (
      <div className={`app ${theme}`}>
        <LoginPage onLogin={() => { checkLogin(); setPage('dashboard') }} />
      </div>
    )
  }

  const isSyncing = status?.status === 'Syncing'

  return (
    <div className={`app ${theme}`}>
      <div className="titlebar">
        <div className="titlebar-dots">
          <span className="dot dot-red"></span>
          <span className="dot dot-yellow"></span>
          <span className="dot dot-green"></span>
        </div>
        <span className="titlebar-text">DMS Sync — Arsipin</span>
      </div>
      <div className="app-layout">
        <aside className="sidebar">
          <div className="sidebar-header">
            <h3>DMS Sync</h3>
            <p>v1.0.0 · {status?.last_email ? 'Terhubung' : 'Offline'}</p>
          </div>
          <nav className="sidebar-nav">
            <button
              className={`nav-item ${page === 'dashboard' ? 'active' : ''}`}
              onClick={() => setPage('dashboard')}
            >
              <span className="nav-icon">📋</span> Aktivitas
            </button>
            <button
              className={`nav-item ${page === 'settings' ? 'active' : ''}`}
              onClick={() => setPage('settings')}
            >
              <span className="nav-icon">⚙️</span> Pengaturan
            </button>
          </nav>
          <div className="sidebar-footer">
            <div className="tray-item">
              <span className="tray-dot" style={{ background: isSyncing ? '#f59e0b' : '#22c55e' }}></span>
              DMS Sync {isSyncing ? 'Menyinkronkan' : 'Aktif'}
            </div>
            <button className="theme-btn" onClick={toggleTheme}>
              {theme === 'light' ? '🌙' : '☀️'} Mode {theme === 'light' ? 'Gelap' : 'Terang'}
            </button>
            <button className="nav-item logout-item" onClick={handleLogout}>
              <span className="nav-icon">🚪</span> Keluar
            </button>
          </div>
        </aside>
        <main className="main-content">
          {page === 'dashboard' && <Dashboard status={status} />}
          {page === 'settings' && <Settings onLogout={handleLogout} />}
        </main>
      </div>
    </div>
  )
}

export default App
