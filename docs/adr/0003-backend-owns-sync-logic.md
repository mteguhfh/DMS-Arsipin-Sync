# ADR-0003 — Seluruh logika sync di backend Rust

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

Sinkronisasi folder butuh state yang hidup terus: daftar folder yang dipantau, antrian file, hash yang sudah terupload, dan status terkini. State itu harus tetap berjalan meskipun jendela aplikasi ditutup dan aplikasi hanya tersisa di tray.

## Decision

Frontend tidak menyentuh filesystem atau jaringan. Semua operasi lewat Tauri command yang terdaftar di `generate_handler!` pada `src-tauri/src/lib.rs`:

`login`, `check_session`, `test_sync`, `get_status`, `get_sync_log`, `add_watch_folder`, `remove_watch_folder`, `get_config`, `get_folder_cache`, `clear_sync_log`, `get_watcher_events`.

State disimpan di `AppState` dan `SyncEngine`, semuanya dibungkus `Arc<tokio::sync::Mutex<..>>`. Frontend memanggil `invoke()` dan hanya menampilkan hasilnya.

## Consequences

**Positif**
- Sync tetap jalan saat UI ditutup.
- Satu sumber kebenaran untuk queue, log, dan status.
- Batas keamanan jelas: webview tidak punya akses langsung ke filesystem atau jaringan.

**Negatif**
- Setiap fitur baru butuh perubahan di dua tempat: command Rust dan pemanggilan `invoke()`.
- Semua state memakai satu mutex besar per komponen, jadi operasi panjang (misalnya upload) berisiko menahan lock.
- `SyncLogEntry` hanya hidup di memori; restart menghapus log yang `get_sync_log` kembalikan (config punya field `sync_log`, tetapi jalur command tidak mengisinya).

## Alternatives considered

- **Logika sync di frontend (JS)** — ditolak: tidak berjalan saat jendela ditutup dan tidak punya akses watcher native.
- **Proses sidecar terpisah** — ditolak untuk versi ini karena menambah kompleksitas packaging dan IPC.
