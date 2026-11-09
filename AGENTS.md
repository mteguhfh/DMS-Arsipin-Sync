# AGENTS.md — dms-sync

Tauri v2 desktop sync client (React 19 + TS frontend, Rust backend). Watches local folders, uploads changed files to Arsipin DMS over HTTP.

## Commands

- Full dev app (compiles Rust + Vite): `npm run tauri dev` — plain `npm run dev` only runs Vite; the UI calls `invoke()` and will fail in a browser.
- CI-equivalent checks, in this order: `npm run lint` → `npx tsc --noEmit` → `npx vite build`. Note CI does **not** run `npm run build` (`tsc -b && vite build`) or `cargo check`.
- Rust must be verified manually, there is no Rust CI job: `cd src-tauri && cargo check` (or `cargo clippy`).
- No test suite exists anywhere. Do not claim tests pass; verify by build + manual run.

## Architecture

- Single Tauri window, no router: `src/App.tsx` switches pages by local state (`login` / `dashboard` / `settings`).
- All backend access goes through Tauri commands registered in `src-tauri/src/lib.rs` `generate_handler!`. Adding a command means registering it there **and** in the frontend `invoke()` call.
- Frontend uses dynamic import `const { invoke } = await import('@tauri-apps/api/core')` per call — match this style rather than a top-level import.
- Rust module roles: `lib.rs` commands + app state wiring, `sync.rs` queue/state engine (`SyncEngine`, all `Arc<Mutex<..>>`), `watcher.rs` notify-based folder watcher + initial recursive scan, `api.rs` reqwest client, `config.rs` persisted config, `tray.rs` tray menu.
- Runtime flow: watcher event → `mpsc` channel → `enqueue_file()` (hash + dedupe + queue) → `process_queue()` upload loop → `SyncLogEntry` in memory.
- `watcher_events` is a debug ring buffer (capped at 500) surfaced in the Dashboard "Log" tab. Push an event there when adding non-obvious sync logic; it is the main debugging aid.

## Gotchas

- `SyncEngine` uses `tokio::sync::Mutex` everywhere; `.blocking_lock()` appears only in `run()` before the Tauri builder starts. Do not call `blocking_lock()` inside commands or async code — it panics on the tokio runtime. This is why the watcher is created inside commands, not in `run()`.
- `folder_cache.rs` is currently unused by the sync path (`folderPath` is derived from the watched root's folder name + relative path in `process_queue`). Don't assume it drives folder mapping.
- Upload contract: `POST /api/sync/upload` multipart with `file`, `name`, `originalPath`, optional `folderPath`; response is camelCase (`documentId`, `folderId`). Session cookie is persisted as a raw cookie string in `config.json` and replayed via `load_cookies`.
- Config lives in `dirs::data_dir()/dms-sync/config.json` (not Tauri's store plugin). It contains the session cookie; migration from the old single `watched_folder` field happens in `AppConfig::load`.
- Tray menu "Pause Sync" is a stub that only logs.
- Version is duplicated: `package.json` (`0.0.0`), `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Release version comes from the git tag `v*`; bump both Rust-side files when cutting a release.
- Release: pushing a `v*` tag triggers `.github/workflows/release.yml` (4 targets, publishes the GitHub Release directly). macOS signing is skipped when `APPLE_CERTIFICATE` secret is absent.
- Watcher skips extensions `tmp`, `temp`, `swp`, `~`, `part`, `crdownload`. MIME type is guessed purely from extension in `sync.rs::mime_from_ext`; unknown types become `application/octet-stream`.

## Git workflow

- Commit after every completed piece of work, before starting the next one. Do not leave verified changes uncommitted: each commit is a rollback point.
- Commit only the files the task actually changed. Never `git add -A`; the working tree routinely carries CRLF-only diffs (see below) that would bloat the commit.
- `.gitattributes` enforces `eol=lf`. If `git status` shows a file as fully rewritten, check `git diff --stat --ignore-all-space`: an empty result means line endings only, so discard it (`git checkout -- <path>`) instead of committing.
- Message style follows history: `type: short description` (`fix:`, `docs:`, `feat:`).

## Design decisions

- `docs/adr/` holds the rationale for non-obvious choices (Tauri, cookie auth, hash dedupe, folder mapping, config file, release flow). Read the relevant ADR before changing one of those areas.
- A change that reverses an ADR needs a new ADR + `Superseded by` status on the old one, committed with the code change.

## Conventions

- Rust error type is plain `Result<T, String>` with human-readable messages; keep it.
- Path strings are normalized with `trim_end_matches(&['/', '\\', ' '][..])` and compared case-insensitively for watched-root matching on Windows. Reuse those helpers instead of ad-hoc trimming.
- New Tauri permissions go in `src-tauri/capabilities/default.json`.
