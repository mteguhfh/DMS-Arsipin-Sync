# ADR-0004 — `notify` crate untuk pemantauan folder

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

Pengguna menaruh file baru ke folder yang dipantau dan aplikasi harus mengunggahnya secepatnya. Pendekatan polling memakan I/O dan lambat; aplikasi butuh event filesystem yang asli dan lintas platform.

## Decision

Memakai `notify` v7 dengan fitur `macos_fsevent`, `RecursiveMode::Recursive`, dan `Config::default()`. Event `Create` dan `Modify` diteruskan ke channel `mpsc::UnboundedSender<String>`, lalu task tokio memanggil `enqueue_file()` dan `process_queue()`.

Setelah folder ditambahkan, watcher langsung melakukan **initial recursive scan** manual untuk mengunggah file yang sudah ada sebelum aplikasi dijalankan. Ekstensi sementara (`tmp`, `temp`, `swp`, `~`, `part`, `crdownload`) diabaikan.

`FileWatcher` dibuat di dalam command, bukan di `run()`, karena `SyncEngine` memakai `tokio::sync::Mutex` dan `blocking_lock()` hanya aman sebelum runtime tokio aktif (lihat commit `00b6623`).

## Consequences

**Positif**
- Deteksi perubahan hampir instan tanpa polling.
- Satu implementasi untuk Windows, macOS, dan Linux.
- Initial scan menangani kasus folder yang sudah berisi file.

**Negatif**
- Editor bisa memicu beberapa event untuk satu penyimpanan; deduplikasi hash yang menahan duplikat, bukan watcher.
- File yang masih ditulis bisa terupload sebagian; belum ada debounce atau pemeriksaan ukuran stabil.
- `remove_watch()` hanya memanggil `unwatch()`; scan yang sedang berjalan tidak dibatalkan.

## Alternatives considered

- **Polling berkala** — ditolak: mahal untuk folder besar dan tidak real-time.
- **`notify-debouncer-full`** — belum dipakai; bisa menjadi ADR lanjutan kalau masalah upload parsial muncul di lapangan.
