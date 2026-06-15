# DMS Sync

> Klien sinkronisasi desktop untuk Arsipin DMS — upload dokumen dari folder lokal secara otomatis.

[![CI](https://github.com/arsipin/dms-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/arsipin/dms-sync/actions/workflows/ci.yml)
[![Release](https://github.com/arsipin/dms-sync/actions/workflows/release.yml/badge.svg)](https://github.com/arsipin/dms-sync/actions/workflows/release.yml)
[![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)](https://github.com/arsipin/dms-sync/releases)
[![macOS Intel](https://img.shields.io/badge/macOS%20Intel-000000?logo=apple&logoColor=white)](https://github.com/arsipin/dms-sync/releases)
[![macOS Silicon](https://img.shields.io/badge/macOS%20Silicon-000000?logo=apple&logoColor=white)](https://github.com/arsipin/dms-sync/releases)
[![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black)](https://github.com/arsipin/dms-sync/releases)
[![Panduan Mac](https://img.shields.io/badge/Panduan-Mac-999?logo=apple)](README-MAC.md)

---

## Fitur

- 🔄 **Sinkronisasi otomatis** — Pantau folder dan upload file baru/berubah ke DMS
- 🔐 **Login berbasis session** — Login sekali, cookie tersimpan walau aplikasi ditutup
- 🗂️ **Pemetaan folder** — Struktur folder lokal dipetakan ke folder DMS
- 🖥️ **System tray** — Berjalan di latar belakang, bisa diakses dari menu bar
- 🚀 **Multi-platform** — Berfungsi di Windows, macOS (Intel & Silicon), dan Linux
- ⚡ **Deduplikasi file** — Lewati file yang sudah pernah diupload berdasarkan hash konten

## Unduh

Ambil installer terbaru untuk platform kamu dari halaman [Releases](https://github.com/arsipin/dms-sync/releases).

| Platform | Format |
|---|---|
| Windows | `.msi` / `.exe` |
| macOS Intel | `.dmg` (x86_64) |
| macOS Silicon | `.dmg` (aarch64) |
| Linux (Debian/Ubuntu) | `.deb` |

### 🍎 Pengguna Mac?

Lihat [panduan lengkap untuk Mac](README-MAC.md) — termasuk cara install, izin akses folder, code signing, dan dual-architecture.

## Pengembangan

### Prasyarat

- [Node.js](https://nodejs.org) LTS
- [Rust](https://rustup.rs) (via `rustup`)
- [Dependensi sistem Tauri](https://v2.tauri.app/start/prerequisites/) sesuai platform kamu

### Setup

```bash
# Install dependensi JS
npm install

# Jalankan mode dev (hot-reload)
npm run tauri dev

# Build untuk production
npm run tauri build
```

### Struktur Proyek

```
├── src/                  # Frontend React (TypeScript)
│   ├── App.tsx
│   ├── components/
│   │   ├── Dashboard.tsx
│   │   ├── LoginPage.tsx
│   │   └── Settings.tsx
│   └── index.css
├── src-tauri/            # Backend Rust
│   ├── src/
│   │   ├── lib.rs        # Entry point aplikasi, perintah Tauri
│   │   ├── api.rs        # Klien API DMS
│   │   ├── config.rs     # Konfigurasi aplikasi (path lintas platform)
│   │   ├── sync.rs       # Mesin sinkronisasi, antrian, hash file
│   │   ├── watcher.rs    # Pemantau file system
│   │   ├── tray.rs       # Menu system tray
│   │   └── folder_cache.rs
│   └── Cargo.toml
├── .github/workflows/    # Pipeline CI/CD
│   ├── ci.yml            # Lint, typecheck, build di setiap PR
│   └── release.yml       # Build multi-platform saat tag di-push
└── package.json
```

### Perintah

| Perintah | Deskripsi |
|---|---|
| `npm run dev` | Jalankan Vite dev server |
| `npm run build` | Build frontend saja |
| `npm run lint` | ESLint check |
| `npm run tauri dev` | Jalankan aplikasi mode dev |
| `npm run tauri build` | Build aplikasi production |

## CI/CD

Push tag untuk memicu build rilis multi-platform:

```bash
git tag v1.0.0
git push origin v1.0.0
```

[Workflow Release](.github/workflows/release.yml) akan build untuk 4 target secara paralel:
- Windows `.msi`
- macOS Intel `.dmg`
- macOS Silicon `.dmg`
- Linux `.deb`

Artifacts akan otomatis diupload ke draft GitHub Release.

## Lisensi

© Arsipin — Internal tool
