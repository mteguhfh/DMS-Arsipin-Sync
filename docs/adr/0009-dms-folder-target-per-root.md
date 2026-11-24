# ADR-0009 — Pilih folder tujuan DMS per root lokal

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menggantikan:** ADR-0006 untuk pemetaan utama; aturan lama tetap sebagai fallback

## Context

ADR-0006 menurunkan `folderPath` dari nama folder root lokal ditambah subfolder relatif. Aturan itu otomatis, tetapi pengguna tidak bisa mengarahkan folder lokal ke folder DMS yang sudah ada.

Rilis 1.2.0 menambah dropdown **Tujuan DMS** di Dashboard. Server menyediakan `GET /api/folders`; klien perlu menampilkan pohon folder sebagai path dan menyimpan pilihan pengguna lintas restart.

## Decision

Setiap root lokal bisa punya satu target DMS:

- `list_dms_folders` memanggil `DmsApi::get_folders()`; frontend merangkai `parentId` menjadi path lewat `buildFolderPaths()`.
- `set_folder_target(path, dms_folder_id, dms_folder_path)` menyimpan `folder_targets` di `config.json` dan `targets` di `SyncEngine`.
- `enqueue_file()` menyalin target root ke `QueuedFile.dms_target`.
- `process_queue()` menentukan `folderPath`:
  - target ada: `target + "/" + subfolder relatif`, atau `target` jika file langsung di root;
  - target kosong: fallback ADR-0006, yaitu `nama root + "/" + subfolder relatif`.
- `dms_folder_id` ikut disimpan, tetapi upload tetap mengirim `folderPath` berupa string. Server yang membuat atau me-resolve folder.

## Consequences

**Positif**

- Pengguna bisa mengirim file ke struktur DMS yang sudah ada tanpa rename folder lokal.
- Pilihan target persisten lewat `config.json`.
- Perilaku lama tetap tersedia lewat opsi **Otomatis**.
- Folder berpassword diberi penanda di UI sebelum upload ditolak server.

**Negatif**

- Satu root hanya bisa punya satu target DMS; tidak ada aturan per subfolder.
- Mengubah target tidak memindahkan dokumen yang sudah terupload.
- `folder_cache.rs` masih tidak dipakai untuk me-resolve ID saat upload.
- `dms_folder_id` tersimpan tetapi tidak dikirim ke endpoint upload.

## Alternatives considered

- **Selalu pakai nama root lokal** — dipertahankan sebagai fallback, tapi tidak memberi kontrol kepada pengguna.
- **Satu target global untuk semua root** — ditolak: tiap root bisa punya tujuan berbeda.
- **Buat folder DMS dari klien** — ditolak: endpoint upload sudah membuat folder yang belum ada.
