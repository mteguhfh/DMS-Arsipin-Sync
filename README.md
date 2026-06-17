# 📄 DMS Sync

> Klien sinkronisasi desktop untuk Arsipin DMS — upload dokumen dari folder lokal secara otomatis.

<div align="center">

[![CI](https://github.com/mteguhfh/DMS-Arsipin-Sync/actions/workflows/ci.yml/badge.svg)](https://github.com/mteguhfh/DMS-Arsipin-Sync/actions/workflows/ci.yml)
[![Release](https://github.com/mteguhfh/DMS-Arsipin-Sync/actions/workflows/release.yml/badge.svg)](https://github.com/mteguhfh/DMS-Arsipin-Sync/actions/workflows/release.yml)
[![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)
[![macOS Intel](https://img.shields.io/badge/macOS%20Intel-000000?logo=apple&logoColor=white)](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)
[![macOS Silicon](https://img.shields.io/badge/macOS%20Silicon-000000?logo=apple&logoColor=white)](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)
[![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black)](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)
[![Panduan Mac](https://img.shields.io/badge/Panduan-Mac-999?logo=apple)](README-MAC.md)

</div>

---

## ✨ Fitur Utama

- 🔄 **Sinkronisasi Otomatis** — Pantau folder dan upload file baru/berubah ke DMS secara real-time
- 🔐 **Login Berbasis Session** — Login sekali, cookie tersimpan walau aplikasi ditutup
- 🗂️ **Pemetaan Folder Cerdas** — Struktur folder lokal dipetakan otomatis ke folder DMS
- 🖥️ **System Tray** — Berjalan di latar belakang, bisa diakses dari menu bar
- 🚀 **Multi-Platform** — Berfungsi di Windows, macOS (Intel & Silicon), dan Linux
- ⚡ **Deduplikasi File** — Lewati file yang sudah pernah diupload berdasarkan hash konten

---

## 🚀 Memulai

### Unduh Aplikasi

Ambil installer terbaru untuk platform Anda dari halaman **[Releases](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)**.

| Platform | Format | Catatan |
|:---:|:---:|---|
| **Windows** | `.msi` / `.exe` | Installer otomatis |
| **macOS Intel** | `.dmg` | x86_64 |
| **macOS Silicon** | `.dmg` | aarch64 (Apple Silicon) |
| **Linux** | `.deb` | Debian/Ubuntu |

#### 🍎 Pengguna Mac?

Lihat **[Panduan Lengkap untuk Mac](README-MAC.md)** — termasuk:
- Cara install dan setup
- Izin akses folder (Full Disk Access)
- Code signing dan notarization
- Support dual-architecture

---

## 💻 Pengembangan

### Prasyarat

Sebelum memulai, pastikan Anda sudah install:

- **[Node.js](https://nodejs.org)** — LTS atau terbaru
- **[Rust](https://rustup.rs)** — via `rustup`
- **[Dependensi Sistem Tauri](https://v2.tauri.app/start/prerequisites/)** — sesuai platform Anda

### Setup Proyek

```bash
# Clone repository
git clone https://github.com/mteguhfh/DMS-Arsipin-Sync.git
cd DMS-Arsipin-Sync

# Install dependensi Node.js
npm install

# Jalankan mode development (hot-reload)
npm run tauri dev

# Build untuk production
npm run tauri build
```

### Struktur Proyek

```
DMS-Arsipin-Sync/
├── src/                          # Frontend React (TypeScript)
│   ├── App.tsx                   # Komponen utama aplikasi
│   ├── components/
│   │   ├── Dashboard.tsx         # Dashboard sinkronisasi
│   │   ├── LoginPage.tsx         # Halaman login
│   │   └── Settings.tsx          # Pengaturan aplikasi
│   └── index.css                 # Styling global
│
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── lib.rs               # Entry point aplikasi & perintah Tauri
│   │   ├── api.rs               # Klien API DMS
│   │   ├── config.rs            # Konfigurasi aplikasi (lintas platform)
│   │   ├── sync.rs              # Mesin sinkronisasi & antrian
│   │   ├── watcher.rs           # Pemantau file system
│   │   ├── tray.rs              # Menu system tray
│   │   └── folder_cache.rs      # Cache folder
│   └── Cargo.toml               # Dependensi Rust
│
├── .github/workflows/            # Pipeline CI/CD
│   ├── ci.yml                   # Lint, typecheck, build
│   └── release.yml              # Build multi-platform
│
└── package.json                  # Konfigurasi Node.js
```

### Perintah NPM

| Perintah | Deskripsi |
|---|---|
| `npm run dev` | Jalankan Vite dev server |
| `npm run build` | Build frontend saja |
| `npm run lint` | Cek dengan ESLint |
| `npm run tauri dev` | Jalankan aplikasi dalam mode dev |
| `npm run tauri build` | Build aplikasi production |

---

## 🔄 CI/CD & Release

### Trigger Release Otomatis

Push git tag untuk memicu build rilis multi-platform:

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Workflow Release

[Workflow Release](.github/workflows/release.yml) akan otomatis membangun untuk 4 target secara paralel:

- ✅ Windows `.msi`
- ✅ macOS Intel `.dmg`
- ✅ macOS Silicon `.dmg`
- ✅ Linux `.deb`

Artifacts akan otomatis diupload ke draft **GitHub Release** dan siap didistribusikan.

---

## 📝 Lisensi

© **Arsipin** — Internal Tool
