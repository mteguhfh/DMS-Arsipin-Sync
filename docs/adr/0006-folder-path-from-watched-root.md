# ADR-0006 — Pemetaan folder DMS dari nama root folder lokal

- **Status:** Accepted
- **Tanggal:** 2026-06-24 (commit `3a14f84`), dicatat 2026-11-10

## Context

DMS menyimpan dokumen dalam hierarki folder. Pengguna memilih satu atau lebih folder lokal untuk dipantau, dan struktur di dalam folder itu seharusnya tercermin di DMS. Awalnya aplikasi mendukung satu folder saja; versi 1.1.0 membuka dukungan multi-folder, sehingga aturan pemetaan harus ditegaskan.

## Decision

`process_queue()` menurunkan `folderPath` dari **nama folder root yang dipantau** ditambah path relatif file:

```
folderPath = "<nama root>" + "/" + "<subfolder relatif>"
```

Contoh: root `/Users/ana/Projects/Kepegawaian`, file `2026/SK-01.pdf` → `Kepegawaian/2026`. File langsung di root → `Kepegawaian`.

Server membuat folder yang belum ada lewat endpoint upload; klien tidak memanggil API pembuatan folder. `relative_path()` dinormalisasi lewat `Path::strip_prefix` dengan fallback perbandingan prefix manual (case-insensitive di Windows).

## Consequences

**Positif**
- Struktur folder lokal langsung terlihat di DMS tanpa konfigurasi tambahan.
- Dua root dengan nama sama akan bergabung ke folder DMS yang sama — sering kali justru diinginkan.
- Tidak ada state tambahan; semua diturunkan dari path event.

**Negatif**
- Pengguna tidak bisa menentukan nama folder tujuan DMS; nama selalu mengikuti nama folder di disk.
- Rename folder root lokal membuat isi pindah ke cabang DMS yang baru.
- `folder_cache.rs` (peta path→id) tidak dipakai jalur ini meskipun sudah ada di codebase, sehingga kode itu berpotensi menyesatkan.

## Alternatives considered

- **Pilih folder tujuan DMS per root folder** — ditolak untuk v1.1.0: menambah UI dan state, sedangkan nama root sudah cukup mewakili.
- **Petakan ke akar DMS tanpa subfolder** — ditolak: semua file menumpuk di satu tempat.
