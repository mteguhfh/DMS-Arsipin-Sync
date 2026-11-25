import { useEffect, useState } from 'react'
import { findUpdate, installUpdate, type AvailableUpdate } from '../lib/updater'

export default function UpdateNotice() {
  const [available, setAvailable] = useState<AvailableUpdate | null>(null)
  const [busy, setBusy] = useState(false)
  const [progress, setProgress] = useState<number | null>(null)
  const [error, setError] = useState('')
  const [dismissed, setDismissed] = useState(false)

  useEffect(() => {
    let cancelled = false

    void (async () => {
      try {
        const found = await findUpdate()
        if (cancelled) {
          found?.update.close()
          return
        }
        setAvailable(found)
      } catch {
        // Updater tidak tersedia di browser/dev; jangan ganggu pengguna.
      }
    })()

    return () => {
      cancelled = true
    }
  }, [])

  function handleDismiss() {
    available?.update.close()
    setDismissed(true)
  }

  async function handleInstall() {
    if (!available) return
    setBusy(true)
    setError('')
    setProgress(0)
    try {
      await installUpdate(available.update, setProgress)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      setBusy(false)
      setProgress(null)
    }
  }

  if (!available || dismissed) return null

  return (
    <div className="update-banner">
      <div className="update-banner-body">
        <div className="update-banner-title">
          Versi {available.version} tersedia
        </div>
        <div className="update-banner-meta">
          Berjalan di {available.currentVersion}. Pembaruan diunduh dari GitHub Releases.
        </div>
        {busy && (
          <div className="update-banner-progress">
            <div
              className="update-banner-progress-fill"
              style={{ width: progress === null ? '100%' : `${progress}%` }}
            />
          </div>
        )}
        {error && <div className="update-banner-error">{error}</div>}
      </div>
      <div className="update-banner-actions">
        <button className="btn-small" onClick={handleDismiss} disabled={busy}>
          Nanti
        </button>
        <button className="btn-primary" onClick={handleInstall} disabled={busy}>
          {busy ? 'Mengunduh...' : 'Perbarui'}
        </button>
      </div>
    </div>
  )
}
