import { useState } from 'react'
import BrandLogo from './BrandLogo'
import { errorMessage } from '../types'

interface Props {
  onLogin: () => void
}

/** Fixed backend; this client only ever talks to Arsipin DMS. */
const DEFAULT_SERVER = 'https://dms.arsipin.id'

export default function LoginPage({ onLogin }: Props) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('login', { email, password, serverUrl: DEFAULT_SERVER })
      onLogin()
    } catch (err) {
      setError(errorMessage(err) || 'Login gagal')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="login-container">
      <form onSubmit={handleSubmit} className="login-form">
        <BrandLogo />

        <div className="login-header">
          <h1>Selamat datang</h1>
          <p>Masuk untuk mulai menyinkronkan dokumen Anda ke Arsipin DMS</p>
        </div>

        <label className="field">
          <span>Email</span>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="nama@perusahaan.com"
            autoComplete="username"
            required
          />
        </label>

        <label className="field">
          <span>Password</span>
          <div className="password-wrap">
            <input
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Masukkan password"
              autoComplete="current-password"
              required
            />
            <button
              type="button"
              className="password-toggle"
              onClick={() => setShowPassword(v => !v)}
              aria-label={showPassword ? 'Sembunyikan password' : 'Tampilkan password'}
            >
              {showPassword ? 'Sembunyikan' : 'Lihat'}
            </button>
          </div>
        </label>

        {error && <div className="login-error">{error}</div>}

        <button type="submit" disabled={loading} className="btn-primary btn-login">
          {loading ? 'Memproses...' : 'Masuk'}
        </button>

        <p className="login-footnote">
          Terhubung ke <strong>dms.arsipin.id</strong>
        </p>
      </form>
    </div>
  )
}
