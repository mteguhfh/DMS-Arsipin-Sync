import { useState } from 'react'
import BrandLogo from './BrandLogo'
import { errorMessage, type LoginResult } from '../types'

interface Props {
  onLogin: () => void
}

/** Fixed backend; this client only ever talks to Arsipin DMS. */
const DEFAULT_SERVER = 'https://dms.arsipin.id'

export default function LoginPage({ onLogin }: Props) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [code, setCode] = useState('')
  const [twoFactorRequired, setTwoFactorRequired] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  function resetTwoFactor() {
    setTwoFactorRequired(false)
    setCode('')
    setError('')
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      const { invoke } = await import('@tauri-apps/api/core')

      if (twoFactorRequired) {
        const result = await invoke<LoginResult>('verify_two_factor', { code })
        if (result.two_factor_required) {
          setError('Verifikasi 2FA belum selesai')
          return
        }
      } else {
        const result = await invoke<LoginResult>('login', {
          email,
          password,
          serverUrl: DEFAULT_SERVER,
        })
        if (result.two_factor_required) {
          setCode('')
          setTwoFactorRequired(true)
          return
        }
      }

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
          <h1>{twoFactorRequired ? 'Verifikasi 2FA' : 'Selamat datang'}</h1>
          <p>
            {twoFactorRequired
              ? 'Masukkan kode 6 angka dari aplikasi autentikasi Anda.'
              : 'Masuk untuk mulai menyinkronkan dokumen Anda ke Arsipin DMS'}
          </p>
        </div>

        {twoFactorRequired ? (
          <label className="field">
            <span>Kode 2FA</span>
            <input
              type="text"
              inputMode="numeric"
              autoComplete="one-time-code"
              pattern="[0-9]*"
              maxLength={6}
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder="123456"
              required
            />
          </label>
        ) : (
          <>
            <label className="field">
              <span>Email</span>
              <input
                type="email"
                value={email}
                onChange={(e) => {
                  setEmail(e.target.value)
                  resetTwoFactor()
                }}
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
                  onChange={(e) => {
                    setPassword(e.target.value)
                    resetTwoFactor()
                  }}
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
          </>
        )}

        {error && <div className="login-error">{error}</div>}

        <div className="login-actions">
          {twoFactorRequired && (
            <button type="button" className="btn-secondary" onClick={resetTwoFactor}>
              Kembali
            </button>
          )}
          <button type="submit" disabled={loading} className="btn-primary btn-login">
            {loading ? 'Memproses...' : twoFactorRequired ? 'Verifikasi' : 'Masuk'}
          </button>
        </div>

        <p className="login-footnote">
          Terhubung ke <strong>dms.arsipin.id</strong>
        </p>
      </form>
    </div>
  )
}
