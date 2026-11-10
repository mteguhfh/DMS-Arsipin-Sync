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
- 📁 **Pilih Folder Tujuan DMS** — Tentukan sendiri folder tujuan di DMS untuk setiap folder lokal, atau biarkan mengikuti nama folder
- 🔐 **Login Berbasis Session** — Login sekali, cookie tersimpan walau aplikasi ditutup
- 🗂️ **Pemetaan Folder Cerdas** — Struktur subfolder lokal dicerminkan otomatis di bawah folder DMS pilihan
- 🖥️ **System Tray** — Berjalan di latar belakang, bisa diakses dari menu bar
- 🚀 **Multi-Platform** — Berfungsi di Windows, macOS (Intel & Silicon), dan Linux
- ⚡ **Deduplikasi File** — Lewati file yang sudah pernah diupload berdasarkan hash konten
- 🔁 **Retry Otomatis** — Upload gagal dicoba ulang dengan jeda bertahap, tidak ada file yang hilang
- 🛡️ **Tahan File Setengah Jadi** — File yang masih disalin atau diunduh ditunggu sampai stabil sebelum diupload
- 🌗 **Mode Terang & Gelap** — Tema mengikuti preferensi Anda

---

## 🚀 Memulai

### Unduh Aplikasi

Ambil installer terbaru untuk platform Anda dari halaman **[Releases](https://github.com/mteguhfh/DMS-Arsipin-Sync/releases)**.

| Platform | Format | Catatan |
|:---:|:---:|---|
| **Windows** | `.msi` / `.exe` | Installer otomatis |
| **macOS Intel** | `.dmg` | x86_64 |
| **macOS Silicon** | `.dmg` | aarch64 (Apple Silicon) |
| **Linux (Debian/Ubuntu)** | `.deb` | `apt-get install ./DMS\ Sync_*.deb` |
| **Linux (Fedora/RHEL)** | `.rpm` | `dnf install ./*.rpm` |

> ⚠️ **Linux:** jangan pakai `dpkg -i`. Paket bergantung pada `libgtk-3-0`, yang di distro baru sudah berganti nama menjadi `libgtk-3-0t64`. Hanya `apt` yang memahami hubungan *Provides* tersebut, jadi gunakan `apt-get install ./<file>.deb`.

### Cara Kerja

1. **Masuk** dengan akun Arsipin DMS Anda
2. **Pilih folder lokal** yang ingin dipantau (mis. `~/Documents/Surat`)
3. **Pilih folder tujuan** di DMS melalui dropdown *Tujuan DMS* (opsional)
4. File baru atau yang berubah akan otomatis terupload

#### Memilih Folder Tujuan

Setiap folder lokal punya dropdown **Tujuan DMS**:

| Pilihan | Perilaku |
|---|---|
| **Otomatis** | Nama folder lokal dipakai sebagai folder di DMS, dan subfolder ikut dicerminkan |
| **Folder tertentu** | Semua file masuk ke folder DMS yang dipilih, subfolder tetap dicerminkan di bawahnya |

Contoh: folder lokal `Dokumen/SK` diarahkan ke `Kepegawaian`, maka `Dokumen/SK/2026/sk-01.pdf` tersimpan sebagai `Kepegawaian/2026/sk-01.pdf`.

> Folder yang dilindungi password ditandai **"(terkunci)"** pada dropdown. Upload ke folder tersebut bisa ditolak server.

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
│   ├── types.ts                  # Tipe bersama + helper (buildFolderPaths, errorMessage)
│   ├── components/
│   │   ├── BrandLogo.tsx         # Logo Arsipin + badge Sync
│   │   ├── Dashboard.tsx         # Dashboard sinkronisasi & pemilihan folder tujuan
│   │   ├── LoginPage.tsx         # Halaman login
│   │   └── Settings.tsx          # Pengaturan aplikasi
│   └── App.css                   # Styling & tema terang/gelap
│
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── lib.rs               # Entry point aplikasi & perintah Tauri
│   │   ├── api.rs               # Klien API DMS
│   │   ├── config.rs            # Konfigurasi aplikasi (lintas platform)
│   │   ├── sync.rs              # Mesin sinkronisasi, antrian & retry
│   │   ├── watcher.rs           # Pemantau file system
│   │   ├── tray.rs              # Menu system tray
│   │   └── folder_cache.rs      # Cache folder (belum dipakai jalur sync)
│   └── Cargo.toml               # Dependensi Rust
│
├── docs/adr/                     # Architecture Decision Records
├── AGENTS.md                     # Panduan untuk AI coding agent
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
| `npm run dev` | Jalankan Vite dev server saja (UI tanpa backend, `invoke()` akan gagal) |
| `npm run build` | Typecheck + build frontend |
| `npx tsc --noEmit` | Typecheck tanpa output (sama seperti CI) |
| `npm run lint` | Cek dengan ESLint |
| `npm run tauri dev` | Jalankan aplikasi lengkap dalam mode dev |
| `npm run tauri build` | Build aplikasi production untuk semua target |
| `npm run tauri build -- --bundles deb` | Build hanya paket `.deb` (lebih cepat) |

### Cek Rust

```bash
cd src-tauri
cargo check          # verifikasi kompilasi
cargo test --lib     # unit test (config, dsb.)
```

> CI hanya menjalankan lint, typecheck, dan build frontend. Perubahan Rust **tidak** dicek CI, jadi jalankan `cargo check` sendiri sebelum commit.

### Menjalankan di Linux (Wayland)

WebKitGTK kadang menampilkan jendela kosong di sesi Wayland. Matikan compositing:

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 dms-sync
```

Kalau masih kosong, coba `WEBKIT_DISABLE_DMABUF_RENDERER=1` atau paksa XWayland dengan `GDK_BACKEND=x11`.

### Struktur Konfigurasi

Konfigurasi disimpan di:

| Platform | Lokasi |
|---|---|
| Windows | `%APPDATA%\dms-sync\config.json` |
| macOS | `~/Library/Application Support/dms-sync/config.json` |
| Linux | `~/.local/share/dms-sync/config.json` |

File ini berisi session cookie dan memiliki permission `600` di Linux/macOS. Jangan bagikan isinya.

---

## 🔄 CI/CD & Release

### Workflow CI

[`ci.yml`](.github/workflows/ci.yml) berjalan pada setiap push dan pull request ke `master`/`main`:

1. `npm ci`
2. `npm run lint`
3. `npx tsc --noEmit`
4. `npx vite build`

> Job ini **tidak** mengompilasi Rust. Jalankan `cargo check` secara lokal.

### Trigger Release Otomatis

Naikkan versi di `package.json`, `src-tauri/Cargo.toml`, dan `src-tauri/tauri.conf.json` (ketiganya harus sama), commit, lalu push tag:

```bash
git tag v1.2.0
git push origin v1.2.0
```

### Workflow Release

[Workflow Release](.github/workflows/release.yml) membangun untuk 4 target secara paralel:

- ✅ Windows `.msi` / `.exe`
- ✅ macOS Intel `.dmg`
- ✅ macOS Silicon `.dmg`
- ✅ Linux `.deb` + `.rpm`

Artifacts diunggah ke **GitHub Release** yang langsung terbit.

> AppImage tidak dibangun. `linuxdeploy` gagal di distro Linux baru (mis. Ubuntu 26.10), jadi daftar target di `tauri.conf.json` dibatasi ke `deb` dan `rpm`.

---

## 📋 Changelog

### v1.2.0

**Fitur baru**

- Pilih folder tujuan DMS yang sudah ada untuk setiap folder lokal
- Folder terproteksi password ditandai di dropdown

**Perbaikan**

- Upload yang gagal dicoba ulang otomatis (maks. 3 kali, jeda bertahap)
- File yang masih disalin/diunduh tidak lagi terupload sebagian
- Antrian file dibersihkan saat folder berhenti dipantau
- `logout` benar-benar mengakhiri sesi dan menghentikan sinkronisasi
- Konfigurasi tersimpan dengan permission `600` (Linux/macOS)
- Session cookie tidak lagi tercatat di log
- Perbaikan pembacaan konfigurasi yang menyebabkan sesi hilang saat restart
- Perbaikan pembacaan hierarki folder dari API (`parentId`)
- Login tidak lagi menampilkan kolom Server URL
- Tampilan login dan sidebar memakai logo Arsipin

### v1.1.0

- Dukungan banyak folder yang dipantau
- Pilihan tema terang/gelap

---

## 📝 Lisensi

© **Arsipin** — Internal Tool
