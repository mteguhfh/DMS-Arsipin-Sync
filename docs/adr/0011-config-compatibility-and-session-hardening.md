# ADR-0011 — Kompatibilitas format config, izin `0600`, dan logout

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0002 dan ADR-0007

## Context

Install lama menulis `config.json` dengan kunci snake_case seperti `server_url` dan `session_cookie`. Sebuah build antara menulis camelCase. Jika format diubah lagi, sesi tersimpan bisa hilang.

File config memuat session cookie aktif. Log session pada rilis awal mencetak nilai cookie, dan tidak ada jalur logout: menutup jendela tidak menghapus sesi atau menghentikan watcher.

## Decision

- `AppConfig` memakai `#[serde(rename_all = "snake_case")]` untuk format di disk.
- Field diberi `alias` camelCase supaya config dari build antara tetap terbaca.
- `get_config` menerjemahkan kunci snake_case ke camelCase sebelum dikirim ke frontend.
- `AppConfig::save()` mengembalikan `Result`, membuat direktori bila perlu, lalu membatasi file menjadi `0600` di Unix.
- Log check-session dan test-upload hanya mencetak ada/tidaknya cookie, bukan nilainya.
- Command `logout` menghentikan semua watcher, mengosongkan queue dan `processed`, mengembalikan status ke `Idle`, membuang klien API, lalu menghapus `session_cookie` dan `last_email` dari config.

## Consequences

**Positif**

- Install snake_case, camelCase, dan rilis saat ini bisa membaca file config yang sama.
- Cookie tidak lagi muncul di log aplikasi.
- File config tidak bisa dibaca pengguna lain pada sistem Unix multi-user.
- Logout menghentikan upload yang tertunda dan menghapus kredensial lokal.

**Negatif**

- Alias camelCase harus dipertahankan untuk migrasi; skema tidak bisa dibersihkan segera.
- Windows tidak memakai `chmod`; pembatasan akses bergantung pada ACL akun.
- Logout hanya lokal. Sesi di server tidak dicabut lewat endpoint.
- Cookie tetap plaintext di disk bagi pengguna yang memiliki akses akun.

## Alternatives considered

- **Ubah format di disk ke camelCase** — ditolak: install lama kehilangan sesi dan daftar folder.
- **Simpan cookie di OS keychain** — belum dilakukan; kandidat ADR lanjutan bila kebutuhan keamanan naik.
- **Logout hanya di frontend** — ditolak: watcher dan antrean tetap bisa mengunggah setelah keluar.
