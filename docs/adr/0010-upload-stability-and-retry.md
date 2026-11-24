# ADR-0010 — Tunggu file stabil dan retry upload gagal

- **Status:** Accepted
- **Tanggal:** 2026-11-25
- **Menyempurnakan:** ADR-0004 dan ADR-0005

## Context

Event `notify` bisa muncul saat file masih ditulis: download, copy dari USB, atau save dari editor. Jika klien langsung membaca file, DMS bisa menerima body yang terpotong.

Jaringan juga bisa gagal sementara. Rilis awal mencoba sekali lalu mencatat error, sehingga satu gangguan singkat membuat file tidak tersinkron sampai event berikutnya.

## Decision

- `wait_until_stable()` mem-poll ukuran dan mtime tiap 500 ms. File dianggap stabil setelah satu interval tanpa perubahan.
- Batas tunggu 5 detik. File yang masih berubah dilewati dan dicatat di `watcher_events`, bukan masuk antrean.
- `enqueue_file()` menunggu stabil sebelum `fs::read()` dan sebelum menghitung SHA-256.
- `process_queue()` retry maksimal 3 percobaan per file. Jeda 2 detik lalu 4 detik.
- Tiap retry menyalin `QueuedFile` dengan `attempts + 1`. Kegagalan akhir dicatat sebagai `SyncLogEntry` berstatus `error`.

## Consequences

**Positif**

- Upload tidak membaca file saat ukuran atau mtime masih berubah.
- Gangguan jaringan singkat tidak langsung menggagalkan sinkronisasi.
- File yang terus ditulis tidak menyumbat antrean tanpa batas.

**Negatif**

- Setiap file minimal menunggu 500 ms sebelum diproses.
- Byte file disalin untuk tiap retry; file besar menambah tekanan RAM.
- Antrean dan retry hanya hidup di memori. Restart membuang retry yang belum selesai.
- File yang dilewati karena tidak stabil tidak dijadwalkan ulang otomatis.

## Alternatives considered

- **`notify-debouncer-full`** — menunda event, tapi tidak menjamin file selesai ditulis.
- **Upload lalu koreksi di server** — ditolak: dokumen rusak sudah terlanjur tersimpan.
- **Retry tanpa batas** — ditolak: server mati bisa memutar antrean terus-menerus.
