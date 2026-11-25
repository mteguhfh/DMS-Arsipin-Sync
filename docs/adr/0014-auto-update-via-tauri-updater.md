# ADR-0014 — Pembaruan otomatis via Tauri updater

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0008

## Context

Rilis `v1.3.0` masih mengharuskan pengguna mengunduh installer manual dari GitHub Releases. Pengguna tidak diberi tahu saat versi baru tersedia, sehingga install lama bisa bertahan berbulan-bulan.

Tauri v2 menyediakan plugin `updater` dan `process`. Plugin updater memverifikasi artefak dengan kunci tanda tangan, lalu membaca manifest `latest.json`. Manifest itu bisa di-host di GitHub Releases tanpa server tambahan.

## Decision

- Plugin `tauri-plugin-updater` dan `tauri-plugin-process` didaftarkan di Rust dan diberi permission `updater:default` serta `process:default`.
- `tauri.conf.json` mengaktifkan `bundle.createUpdaterArtifacts`, memasang public key, dan menunjuk endpoint `https://github.com/mteguhfh/DMS-Arsipin-Sync/releases/latest/download/latest.json`.
- Kunci privat disimpan sebagai GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`; kunci publik ditanam di aplikasi.
- Workflow rilis menghasilkan artefak bertanda tangan untuk MSI, NSIS, DMG, dan AppImage. macOS juga membangun target `app` agar Tauri menghasilkan `.tar.gz` dan `.sig`. Linux tetap memakai DEB/RPM sebagai installer manual, sementara AppImage dipakai untuk update OTA.
- Tauri tidak menghasilkan `latest.json`; workflow membuat manifest multi-platform dari `.sig` dan URL aset rilis sebelum publikasi.
- UI menampilkan banner "Versi X tersedia" sekali per sesi. Tombol "Perbarui" mengunduh, memasang, lalu menjalankan ulang aplikasi.

## Consequences

**Positif**

- Pengguna mendapat pemberitahuan dan bisa memperbarui tanpa membuka browser.
- Update diverifikasi secara kriptografis, bukan sekadar diunduh dari URL.
- Backend tidak berubah; distribusi tetap memakai GitHub Releases.

**Negatif**

- Kunci privat harus disimpan aman. Kehilangan kunci berarti tidak bisa merilis update untuk pengguna yang sudah terpasang.
- Windows memakai mode instal pasif; macOS dan Linux perlu restart manual lewat `relaunch()`.
- Untuk Linux, hanya pengguna AppImage yang dapat update OTA. Pengguna DEB/RPM tetap memperbarui manual.

## Alternatives considered

- **Server manifest di backend:** butuh endpoint dan deploy baru; GitHub Releases sudah cukup.
- **Tanpa tanda tangan:** tidak didukung Tauri, dan membuka risiko artefak palsu.
- **DEB/RPM sebagai artefak update:** Tauri updater tidak memakai keduanya; AppImage adalah format yang didukung.
