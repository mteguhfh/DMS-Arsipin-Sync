# ADR-0015 — Rustls untuk HTTP client

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0001

## Context

Paket DEB `v1.4.0` tidak bisa dijalankan di Ubuntu 26.10. `/usr/bin/dms-sync` gagal dengan `libssl.so.3: cannot open shared object file`. Runner CI `ubuntu-22.04` menghasilkan biner yang terhubung ke OpenSSL 3, sementara distro yang lebih baru memakai OpenSSL 4. AppImage tidak terkena karena membawa dependensinya sendiri.

Satu-satunya pemakai `native-tls` di aplikasi adalah `reqwest` di `src-tauri/src/api.rs`. Dependency lain, termasuk Tauri updater, sudah memakai rustls.

## Decision

- `reqwest` dinyatakan dengan `default-features = false` dan fitur `rustls-tls-native-roots`.
- TLS memakai rustls dengan root CA dari sistem, bukan OpenSSL.
- Tidak ada perubahan pada kontrak API, konfigurasi, atau endpoint update.

## Consequences

**Positif**

- Biner Linux tidak lagi bergantung pada `libssl.so.3` atau `libcrypto.so.3`.
- Paket DEB/RPM bisa dipakai pada distro dengan OpenSSL 3, OpenSSL 4, atau tanpa OpenSSL sama sekali.
- Trust store sistem tetap dipakai, sehingga CA internal perusahaan masih dikenali.

**Negatif**

- Ukuran biner sedikit bertambah karena rustls ditautkan statis.
- Perubahan TLS perlu diuji ulang terhadap endpoint backend dan GitHub Releases.

## Alternatives considered

- **Install `libssl3` di mesin pengguna:** tidak tersedia di Ubuntu 26.10 dan tidak bisa diandalkan di distro lain.
- **Build di runner distro lama:** menurunkan versi glibc dan bukannya menyelesaikan ketergantungan OpenSSL secara umum.
- **Bundel `libssl.so.3` ke paket:** rawan konflik ABI dengan pustaka sistem dan menambah kompleksitas packaging.
