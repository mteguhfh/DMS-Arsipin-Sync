# ADR-0007 — Konfigurasi di file JSON, bukan Tauri store plugin

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

Aplikasi perlu menyimpan server URL, daftar folder yang dipantau, session cookie, dan preferensi autostart. Tauri menyediakan `tauri-plugin-store` untuk keperluan ini. Namun session cookie dibaca di `run()` **sebelum** builder Tauri dijalankan, karena `engine.api` harus siap sebelum UI tampil.

## Decision

Konfigurasi disimpan sendiri di `dirs::data_dir()/dms-sync/config.json` lewat `AppConfig::load()` / `AppConfig::save()` di `src-tauri/src/config.rs`. Strukturnya `#[derive(Serialize, Deserialize)]`, ditulis sebagai JSON pretty.

Mekanisme khusus:
- Field lama `watched_folder: Option<String>` dimigrasi ke `watched_folders: Vec<String>` saat load; `save()` selalu menulis `watched_folder: None`.
- Field baru diberi `#[serde(default)]` supaya config lama tetap terbaca.
- `load()` menelan semua error dan jatuh ke `Default` (`server_url` = `https://dms.arsipin.id`, `auto_start` = `true`).

## Consequences

**Positif**
- Config bisa dibaca sebelum runtime Tauri aktif.
- Skema bisa berevolusi dengan `serde` default tanpa migrasi manual berat.
- Lokasi file mudah dijelaskan ke pengguna untuk troubleshooting dan reset manual.

**Negatif**
- `save()` mengabaikan error penulisan (`let _ = fs::write`); kegagalan simpan tidak terlihat oleh pengguna.
- Config rusak akan senyap di-reset ke default, menghapus daftar folder yang dipantau.
- Tidak ada validasi skema atau backup otomatis.

## Alternatives considered

- **`tauri-plugin-store`** — ditolak: butuh `AppHandle`, jadi tidak bisa dipakai di `run()` sebelum builder.
- **Registry Windows / keyring per OS** — ditolak: tiga implementasi berbeda untuk masalah yang sama.
