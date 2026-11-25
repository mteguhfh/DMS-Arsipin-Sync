# ADR-0012 — Login 2FA lewat TOTP di klien

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0002

## Context

Backend memakai plugin `twoFactor` Better Auth. Untuk akun dengan 2FA aktif, `POST /api/auth/sign-in/email` menjawab `twoFactorRedirect: true`, menghapus session cookie, lalu hanya menyetel cookie `two_factor` sementara. Klien mengabaikan flag itu, menyimpan cookie 2FA sebagai `session_cookie`, lalu mengembalikan "Login successful". Semua upload berikutnya gagal 401.

Klien juga mencatat `check_session()` gagal hanya sebagai warning. Respons `GET /api/auth/get-session` bisa berstatus 200 dengan body `null`, sehingga status HTTP saja tidak membuktikan sesi aktif.

## Decision

- `LoginResponse` membaca `twoFactorRedirect`. Bila bernilai `true`, command `login` menyimpan klien API di `AppState.pending_login` dan mengembalikan `LoginResult { two_factor_required: true }`. Config belum disimpan dan watcher belum dijalankan.
- Frontend menampilkan input kode 6 angka lalu memanggil command `verify_two_factor(code)`.
- `verify_two_factor` memanggil `POST /api/auth/two-factor/verify-totp` dengan body `{ "code": "123456" }` memakai cookie jar yang sama, lalu melanjutkan alur login normal.
- Kode salah atau validasi gagal mengembalikan klien ke `pending_login` agar bisa dicoba lagi. Klien tidak menyimpan cookie sebelum sesi terverifikasi.
- `finish_login()` menjalankan `check_session()` sebagai syarat keras; cookie kosong atau respons `null` ditolak.
- `logout` mengosongkan `pending_login` bersama kredensial dan antrean.

## Consequences

**Positif**

- Akun 2FA bisa login tanpa mengubah backend.
- Cookie 2FA tidak lagi tersimpan sebagai sesi.
- Kegagalan verifikasi sesi tidak menghasilkan status "Login successful" palsu.

**Negatif**

- `pending_login` hanya hidup di memori; restart aplikasi saat menunggu kode mengharuskan login ulang.
- Cookie sesi masih berbasis cookie plaintext di `config.json` dengan izin `0600` di Unix (lihat ADR-0011).
- Sesi bisa kedaluwarsa atau dicabut server; pengguna harus login ulang.
- Alur ini hanya mendukung TOTP. OTP email tidak dikonfigurasi di backend.

## Alternatives considered

- **API key / sync token** — ditolak untuk sekarang. Pembuatan API key backend hanya untuk admin, guard API key hanya terpasang di `/api/v1`, dan upload v1 memakai `folderId` bukan `folderPath`.
- **Simpan cookie 2FA lalu tunggu upload gagal** — ditolak: pengguna melihat login sukses padahal tidak ada sesi.
