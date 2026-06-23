import { useState } from 'react'

interface Props {
  onLogin: () => void
}

export default function LoginPage({ onLogin }: Props) {
  const [serverUrl, setServerUrl] = useState('https://dms.arsipin.id')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('login', { email, password, serverUrl })
      onLogin()
    } catch (err: any) {
      setError(typeof err === 'string' ? err : err.message || 'Login gagal')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="login-container">
      <form onSubmit={handleSubmit} className="login-form">
        <div className="login-header">
          <h2>DMS Sync</h2>
          <p>Masuk ke akun Arsipin DMS Anda</p>
        </div>

        <label>
          Server URL
          <input
            type="url"
            value={serverUrl}
            onChange={(e) => setServerUrl(e.target.value)}
            placeholder="https://dms.arsipin.id"
            required
          />
        </label>

        <label>
          Email
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="email@example.com"
            required
          />
        </label>

        <label>
          Password
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="••••••••"
            required
          />
        </label>

        {error && <div className="error-msg">{error}</div>}

        <button type="submit" disabled={loading} className="btn-primary">
          {loading ? 'Memproses...' : 'Masuk'}
        </button>
      </form>
    </div>
  )
}
