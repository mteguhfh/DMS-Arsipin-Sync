# 🍎 Panduan Mac — DMS Sync

Panduan lengkap install, troubleshoot, dan develop DMS Sync di macOS (Intel & Apple Silicon).

---

## 📦 Cara Install

### 1. Unduh Installer

Ambil file `.dmg` yang sesuai dari [halaman Releases](https://github.com/arsipin/dms-sync/releases):

| Mac kamu | Download |
|---|---|
| **Apple Silicon** (M1, M2, M3, M4) | `DMS-Sync_x64.dmg` atau `DMS-Sync_aarch64.dmg` |
| **Intel** (Core i5/i7/i9) | `DMS-Sync_x64.dmg` |

> 💡 **Rekomendasi:** Pilih yang `aarch64` untuk performa maksimal di Apple Silicon. Tapi kalo cuma ada `x64`, tetap jalan kok via Rosetta 2.

### 2. Buka .dmg & Install

1. Klik 2x file `.dmg` yang sudah di-download
2. Drag icon **DMS Sync** ke folder **Applications**
3. Buka **Launchpad** atau **Finder > Applications**, lalu klik **DMS Sync**

### 3. Bypass Gatekeeper (Pertama Kali)

Karena app ini belum di-notarize ke Apple (kecuali kamu punya Apple Developer account), macOS akan menampilkan peringatan *"DMS Sync cannot be opened because the developer cannot be verified."*

**Cara mengatasinya:**

**Opsi A — Klik kanan > Buka**
1. Klik kanan (atau Ctrl+klik) **DMS Sync** di folder Applications
2. Pilih **Buka**
3. Klik **Buka** di dialog konfirmasi
4. Cukup sekali — selanjutnya bisa buka normal 2x klik

**Opsi B — Hapus atribut quarantine (terminal)**
```bash
sudo xattr -rd com.apple.quarantine /Applications/DMS\ Sync.app
```

**Opsi C — Whitelist di System Settings**
1. Buka **System Settings > Privacy & Security**
2. Scroll ke bawah ke **Security**
3. Klik **Open Anyway** di samping "DMS Sync was blocked from opening"

---

## 🛡️ Izin yang Diperlukan

### Akses Folder

Saat pertama kali memilih folder untuk di-sync, macOS akan meminta izin akses folder. Klik **Allow** / **Izinkan**.

Kalo terlanjur di-deny, reset di:
**System Settings > Privacy & Security > Files and Folders > DMS Sync**

### Akses System Tray

DMS Sync berjalan di menu bar (system tray) Mac. Ikon akan muncul di samping kanan atas.

Kalo gak muncul, cek:
**System Settings > Control Center > Menu Bar Only** — pastikan aplikasi diizinkan.

### Akses File System (Folder Monitoring)

Untuk fitur auto-sync, app perlu akses ke folder yang dipantau. macOS akan minta izin otomatis lewat dialog `tauri-plugin-dialog`.

---

## 🔧 Build dari Source (Developer)

Buat kamu yang mau compile sendiri dari kode sumber:

### Prasyarat

```bash
# 1. Install Xcode Command Line Tools
xcode-select --install

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install Node.js (via nvm — recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
nvm install --lts
```

### Clone & Build

```bash
git clone <url-repo-kamu>
cd dms-sync
npm install
npm run tauri build
```

Hasil build ada di:
```
src-tauri/target/release/bundle/dmg/DMS Sync_1.0.0.dmg
```

### Dual Architecture (Universal Binary)

Biar 1 file jalan di Intel **dan** Apple Silicon:

```bash
# Tambah target architectures
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Build both
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin

# Gabung pake lipo (hasil ada di folder universal/)
lipo -create \
  src-tauri/target/x86_64-apple-darwin/release/dms-sync \
  src-tauri/target/aarch64-apple-darwin/release/dms-sync \
  -output src-tauri/target/universal/dms-sync
```

---

## 🏷️ Code Signing & Notarization (Production)

Kalo kamu punya **Apple Developer Account ($99/tahun)**, kamu bisa signing app biar gak muncul peringatan Gatekeeper:

### 1. Export Certificate

```bash
# Di Mac developer, export dari Keychain Access:
# Pilih certificate > File > Export Items > .p12
# Simpan file certificate.p12 dengan password
```

### 2. Signing & Notarization

```bash
# Set environment variables
export APPLE_SIGNING_IDENTITY="Developer ID Application: Nama Kamu (TEAMID)"
export APPLE_ID="email@apple.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAMID"

# Build + sign (via tauri-action di GitHub Actions)
# Atau manual:
codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" \
  src-tauri/target/release/bundle/macos/DMS\ Sync.app

# Notarize
xcrun notarytool submit \
  src-tauri/target/release/bundle/dmg/DMS\ Sync_1.0.0.dmg \
  --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_PASSWORD"
```

### 3. GitHub Actions Secrets

Kalo pake [release workflow](../.github/workflows/release.yml), set secrets di repo:
- `APPLE_CERTIFICATE` — isi file `.p12` di-base64 (`base64 -i certificate.p12 | pbcopy`)
- `APPLE_CERTIFICATE_PASSWORD` — password file `.p12`
- `APPLE_SIGNING_IDENTITY` — nama identity (example: `Developer ID Application: John Doe (ABCD1234)`)
- `APPLE_ID` — email Apple ID
- `APPLE_PASSWORD` — app-specific password (generate di appleid.apple.com)
- `APPLE_TEAM_ID` — Team ID (10 karakter, lihat di developer.apple.com)

---

## ❓ Troubleshooting

### "DMS Sync cannot be opened because the developer cannot be verified"
→ Lihat bagian **Bypass Gatekeeper** di atas.

### Ikon gak muncul di menu bar
→ Coba restart app.
→ Kalo masih gak muncul, jalankan dari Terminal untuk lihat error:
  ```bash
  /Applications/DMS\ Sync.app/Contents/MacOS/dms-sync
  ```

### App hang atau gak responsif
```bash
# Kill process
pkill -f dms-sync
# Lalu buka lagi
```

### File gak ke-sync
1. Cek koneksi internet
2. Cek apakah session masih valid (klik **Test Session** di Dashboard)
3. Cek log di tab **Dashboard** untuk melihat error detail
4. Pastikan folder yang dipilih memiliki izin baca

### Config file di mana?
```
~/Library/Application Support/dms-sync/config.json
```

### Log ada di mana?
```
~/Library/Application Support/dms-sync/config.json
```
> (File config berisi session cookie dan history sync)

### Reset total
```bash
rm -rf ~/Library/Application Support/dms-sync
```
> Hapus folder config, lalu buka app lagi. Kamu perlu login ulang.

---

## 💡 Tips

- **Dark mode** — Ikon tray otomatis menyesuaikan light/dark mode macOS
- **Auto-start** — App bisa di-set auto-start pas login (ada di Settings > Preferences)
- **Shortcut** — Bisa pake Spotlight (Cmd+Space) terus ketik "DMS Sync" buat buka app
- **Uninstall** — Cukup drag `DMS Sync.app` ke Trash. Config file manual dihapus kalo perlu.
