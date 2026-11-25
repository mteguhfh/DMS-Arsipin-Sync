# ADR-0013 — Folder berpassword dan payload pohon folder

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0009

## Context

Klien mengambil seluruh folder dari `GET /api/folders`. Di tenant produksi dengan 6.662 folder, respons mencapai ~760 kB. Klien hanya butuh `id`, `name`, `parentId`, dan status kunci.

Backend `GET /api/folders?tree=1` sudah mengirim kolom minimal, tetapi belum mengirim `hasPassword`. Tanpa flag itu, peringatan folder `(terkunci)` di UI tidak bisa diisi dari payload ringan.

`POST /api/sync/upload` menerima `folderPath` dan membuat folder bila belum ada. Jika folder tujuan berpassword, upload tetap diterima. Klien tidak punya cara mengirim kode buka kunci lewat endpoint ini.

## Decision

- Backend `getFolderTree` menghapus `passwordHash` dari hasil, lalu menambahkan `hasPassword: !!passwordHash`.
- Klien memanggil `GET /api/folders?tree=1` dan memetakan `hasPassword` ke penanda folder terkunci.
- `POST /api/sync/upload` menolak folder berpassword dengan HTTP 403 dan `code: FOLDER_PASSWORD_REQUIRED` sebelum file dipakai.
- Perilaku `isHidden` tidak diubah: sync masih mengabaikan folder tersembunyi saat mencari folder root.

## Consequences

**Positif**

- Payload folder jauh lebih kecil.
- Status kunci tampil di UI tanpa membuka hash password.
- Upload ke folder berpassword gagal cepat dan jelas, bukan diam-diam masuk.

**Negatif**

- Folder berpassword harus dibuka di aplikasi web sebelum sinkronisasi bisa dipakai.
- Perubahan backend harus aktif sebelum klien memakai `tree=1`; klien lama tetap kompatibel karena `GET /api/folders` biasa juga mengirim `hasPassword`.
