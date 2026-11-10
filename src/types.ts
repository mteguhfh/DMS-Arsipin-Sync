export interface StatusData {
  status?: string
  queue_length?: number
  watched_folders?: string[]
  server_url?: string
  last_email?: string
  api_available?: boolean
  event_count?: number
}

export interface SyncLog {
  file_path: string
  document_id: string
  status: 'success' | 'error'
  timestamp: string
  folder_path: string | null
  error: string | null
}

export interface AppConfig {
  serverUrl: string
  watchedFolder: string | null
  watchedFolders: string[]
  folderTargets: TargetFolder[]
  autoStart: boolean
  lastEmail: string | null
  sessionCookie: string | null
}

export interface TargetFolder {
  localPath: string
  dmsFolderId: string
  dmsFolderPath: string
}

export interface DmsFolder {
  id: string
  name: string
  parentId: string | null
}

/** Flatten the folder tree into "Root/Sub/Leaf" display paths. */
export function buildFolderPaths(folders: DmsFolder[]): { id: string; path: string }[] {
  const byId = new Map(folders.map(f => [f.id, f]))
  const cache = new Map<string, string>()

  function pathOf(id: string, guard = 0): string {
    const hit = cache.get(id)
    if (hit !== undefined) return hit
    const f = byId.get(id)
    if (!f || guard > 50) return ''
    const parent = f.parentId && byId.has(f.parentId) ? pathOf(f.parentId, guard + 1) : ''
    const full = parent ? `${parent}/${f.name}` : f.name
    cache.set(id, full)
    return full
  }

  return folders
    .map(f => ({ id: f.id, path: pathOf(f.id) }))
    .filter(f => f.path !== '')
    .sort((a, b) => a.path.localeCompare(b.path))
}

/** Tauri commands reject with a plain string, but a JS Error can leak through. */
export function errorMessage(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error) return err.message
  return 'Unknown error'
}
