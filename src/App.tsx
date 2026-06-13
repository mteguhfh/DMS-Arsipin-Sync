import { useState, useEffect } from 'react'
import LoginPage from './components/LoginPage'
import Dashboard from './components/Dashboard'
import Settings from './components/Settings'
import './App.css'

type Page = 'login' | 'dashboard' | 'settings'

function App() {
  const [page, setPage] = useState<Page>('login')
  const [status, setStatus] = useState<any>(null)

  useEffect(() => {
    checkLogin()
  }, [])

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

  return (
    <div className="app">
      <header>
        <h1>DMS Sync</h1>
        {page !== 'login' && (
          <nav>
            <button
              className={page === 'dashboard' ? 'active' : ''}
              onClick={() => setPage('dashboard')}
            >
              Dashboard
            </button>
            <button
              className={page === 'settings' ? 'active' : ''}
              onClick={() => setPage('settings')}
            >
              Settings
            </button>
            <button className="logout-btn" onClick={handleLogout}>
              Logout
            </button>
          </nav>
        )}
      </header>

      <main>
        {page === 'login' && <LoginPage onLogin={() => { checkLogin(); setPage('dashboard') }} />}
        {page === 'dashboard' && <Dashboard status={status} />}
        {page === 'settings' && <Settings onLogout={handleLogout} />}
      </main>
    </div>
  )
}

export default App
