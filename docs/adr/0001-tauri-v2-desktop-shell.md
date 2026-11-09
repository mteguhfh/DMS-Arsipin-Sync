# ADR-0001 — Tauri v2 sebagai shell desktop

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

DMS Sync harus berjalan di Windows, macOS (Intel dan Apple Silicon), dan Linux. Aplikasi butuh akses langsung ke filesystem untuk memantau folder, serta berjalan di latar belakang lewat system tray.

Kandidat utama saat itu: Electron dan Tauri.

## Decision

Memakai Tauri v2 dengan frontend React 19 + TypeScript (Vite) dan backend Rust. Semua akses sistem berat (filesystem watcher, HTTP client, tray) ada di Rust; frontend hanya untuk tampilan.

Bundel target: `.msi`/NSIS (Windows), `.dmg` (macOS x86_64 dan aarch64), `.deb` (Linux) — sesuai `src-tauri/tauri.conf.json` dan `release.yml`.

## Consequences

**Positif**
- Ukuran bundle dan pemakaian memori jauh lebih kecil dari Electron.
- Rust memberi watcher filesystem native (`notify`) dan HTTP performa tinggi tanpa lapisan Node.
- `tauri-plugin-autostart` dan `tauri-plugin-dialog` menangani autostart dan pemilihan folder secara cross-platform.

**Negatif**
- Perubahan backend menuntut compile Rust penuh; iterasi lebih lambat dari Electron.
- Tim harus menguasai Rust dan TypeScript sekaligus.
- Signing/notarization macOS butuh setup terpisah (lihat `README-MAC.md`).

## Alternatives considered

- **Electron** — ekosistem lebih besar, tapi bundle besar dan tidak butuh watcher native yang ditawarkan Rust.
- **CLI/daemon murni tanpa GUI** — ditolak: pengguna butuh login, pemilihan folder, dan log visual.
