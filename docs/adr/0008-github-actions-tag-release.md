# ADR-0008 — Rilis lewat GitHub Actions dengan trigger tag `v*`

- **Status:** Accepted
- **Tanggal:** 2026-06-15 (commit `dc8ce66`), dicatat 2026-11-10

## Context

Aplikasi harus didistribusikan sebagai installer untuk empat target: Windows x64, macOS Intel, macOS Apple Silicon, dan Linux x64. Kompilasi Rust lintas platform tidak bisa dilakukan dari satu mesin developer.

## Decision

Dua workflow terpisah:

- **`ci.yml`** — pada push/PR ke `master` atau `main`: `npm ci`, `npm run lint`, `npx tsc --noEmit`, `npx vite build`. Tidak ada job Rust.
- **`release.yml`** — pada pushed tag `v*` atau `workflow_dispatch`: matrix build di `macos-latest` (dua target), `windows-latest`, dan `ubuntu-22.04`, memakai `swatinem/rust-cache` dan `npm run tauri build -- --target <target>`. Artifact diunggah eksplisit per platform, lalu job `publish` membuat GitHub Release dengan `softprops/action-gh-release`.

Signing macOS bersifat opsional dan di-guard `if [ -z "$APPLE_CERTIFICATE" ]` di dalam shell, karena GitHub Actions tidak mengizinkan `secrets` di ekspresi `if`.

## Consequences

**Positif**
- Satu `git push origin v1.2.0` menghasilkan rilis lengkap.
- Cache Rust memangkas waktu build berulang.
- Rilis tanpa sertifikat Apple tetap berjalan (pengguna memakai bypass Gatekeeper per `README-MAC.md`).

**Negatif**
- Versi tidak diturunkan otomatis dari tag: `Cargo.toml` dan `tauri.conf.json` harus dibump manual, sementara `package.json` tetap `0.0.0`.
- Kode Rust tidak diverifikasi CI; error compile hanya ketahuan saat tag dirilis atau saat developer menjalankan `cargo check` lokal.
- Build rilis memakai runner macOS terbaru tanpa pin versi, sehingga karakteristik build bisa berubah sewaktu-waktu.

## Alternatives considered

- **`tauri-apps/tauri-action`** — pernah dipakai (commit `6c6189d`, `18ff5e2`), lalu diganti upload artifact eksplisit.
- **Build lokal manual per platform** — ditolak: tidak bisa diotomatiskan dan rawan lupa satu target.
