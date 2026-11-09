# ADR-0002 — Autentikasi berbasis session cookie

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

Backend DMS memakai API `/api/auth/sign-in/email` dan `/api/auth/get-session` yang menghasilkan session cookie. Pengguna harus login sekali dan tetap tersinkron setelah aplikasi ditutup dan dibuka kembali, tanpa login ulang setiap start.

## Decision

`DmsApi` menyimpan cookie di `reqwest::cookie::Jar`. Setelah login berhasil, cookie diambil sebagai string mentah (`get_cookie_string()`) dan disimpan di `config.json` sebagai `session_cookie`. Saat aplikasi start, `run()` memuat ulang config, memanggil `load_cookies()`, lalu mengisi `engine.api` supaya sesi langsung aktif sebelum UI muncul.

Tidak ada refresh token atau OAuth. Kalau sesi kedaluwarsa, `check_session()` gagal dan pengguna harus login ulang.

## Consequences

**Positif**
- Sinkronisasi berjalan di latar belakang tanpa interaksi pengguna.
- Tidak perlu menyimpan password di klien.
- Kontrak API sederhana: satu header cookie di setiap request.

**Negatif**
- Cookie tersimpan plaintext di `config.json`. Siapa pun yang bisa membaca file itu bisa membajak sesi sampai cookie kedaluwarsa.
- Tidak ada rotasi otomatis; sesi mati berarti upload gagal dengan error "Not logged in" sampai login ulang manual.
- Perubahan nama/format cookie di server memutus `load_cookies()` yang mem-parse string mentah dengan `split(';')`.

## Alternatives considered

- **Simpan email+password dan login ulang otomatis** — ditolak: risiko penyimpanan kredensial jauh lebih besar.
- **Keychain/OS credential store** — belum dilakukan; bisa menjadi ADR lanjutan kalau kebutuhan keamanan naik.
