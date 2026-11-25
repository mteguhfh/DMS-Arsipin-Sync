import type { Update } from '@tauri-apps/plugin-updater'

export interface AvailableUpdate {
  version: string
  currentVersion: string
  date?: string
  notes?: string
  update: Update
}

/** Cek endpoint `latest.json`. Gagal di browser/dev diabaikan oleh pemanggil. */
export async function findUpdate(): Promise<AvailableUpdate | null> {
  const { check } = await import('@tauri-apps/plugin-updater')
  const update = await check()
  if (!update) return null

  return {
    version: update.version,
    currentVersion: update.currentVersion,
    date: update.date,
    notes: update.body,
    update,
  }
}

/** Unduh, pasang, lalu restart. Progres 0-100 atau null bila ukuran tidak diketahui. */
export async function installUpdate(
  update: Update,
  onProgress: (percent: number | null) => void,
): Promise<void> {
  let downloaded = 0
  let total = 0

  await update.downloadAndInstall(
    (event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength
        onProgress(total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : null)
      } else {
        onProgress(100)
      }
    },
    { restartAfterInstall: false },
  )

  const { relaunch } = await import('@tauri-apps/plugin-process')
  await relaunch()
}
