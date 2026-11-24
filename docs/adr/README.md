# Architecture Decision Records

Daftar keputusan arsitektur DMS Sync. Setiap ADR mencatat satu keputusan, konteksnya, dan konsekuensinya.

Format: [MADR](https://adr.github.io/madr/) ringkas. Status yang dipakai: `Proposed`, `Accepted`, `Deprecated`, `Superseded by ADR-XXXX`.

## Index

| ADR | Judul | Status |
|---|---|---|
| [0001](0001-tauri-v2-desktop-shell.md) | Tauri v2 sebagai shell desktop | Accepted |
| [0002](0002-session-cookie-auth.md) | Autentikasi berbasis session cookie | Accepted |
| [0003](0003-backend-owns-sync-logic.md) | Seluruh logika sync di backend Rust | Accepted |
| [0004](0004-filesystem-watcher-notify.md) | `notify` crate untuk pemantauan folder | Accepted |
| [0005](0005-hash-based-deduplication.md) | Deduplikasi upload berbasis SHA-256 konten file | Accepted |
| [0006](0006-folder-path-from-watched-root.md) | Pemetaan folder DMS dari nama root folder lokal | Superseded by ADR-0009 |
| [0007](0007-config-file-over-store-plugin.md) | Konfigurasi di file JSON, bukan Tauri store plugin | Accepted |
| [0008](0008-github-actions-tag-release.md) | Rilis lewat GitHub Actions dengan trigger tag `v*` | Accepted |
| [0009](0009-dms-folder-target-per-root.md) | Pilih folder tujuan DMS per root lokal | Accepted |
| [0010](0010-upload-stability-and-retry.md) | Tunggu file stabil dan retry upload gagal | Accepted |
| [0011](0011-config-compatibility-and-session-hardening.md) | Format config snake_case, izin `0600`, dan logout | Accepted |

## Menambah ADR

1. Buat file `NNNN-judul-singkat.md` (nomor berikutnya, empat digit).
2. Pakai struktur: Context, Decision, Consequences (positif / negatif), Alternatives considered.
3. Tambahkan baris di tabel Index di atas.
4. Commit ADR bersama perubahan kode yang memunculkannya.

Jangan ubah ADR yang sudah `Accepted`. Kalau keputusan berubah, tulis ADR baru dan tandai yang lama `Superseded by ADR-XXXX`.
