# ADR-0005 — Deduplikasi upload berbasis SHA-256 konten file

- **Status:** Accepted
- **Tanggal:** 2026-06-14 (initial commit), dicatat 2026-11-10

## Context

Event filesystem sering terpicu berkali-kali untuk file yang sama (sinkron cloud, editor, initial scan). Tanpa penjagaan, satu file bisa terupload berulang ke DMS dan membuat duplikat dokumen.

## Decision

Sebelum masuk antrian, `enqueue_file()` membaca seluruh isi file dan menghitung SHA-256 (`SyncEngine::compute_hash`). Hash disimpan di `processed: HashMap<file_path, hash>`. Kalau path yang sama sudah punya hash yang sama, event dilewati. File yang berubah isinya (hash berbeda) tetap diunggah ulang.

Begitu upload sukses, hash dicatat ke `processed`.

## Consequences

**Positif**
- Duplikat dari multiple event hilang tanpa timer atau heuristik waktu.
- File yang memang diedit tetap tersinkron karena hash berubah.
- Tidak butuh state persisten di server untuk deduplikasi sisi klien.

**Negatif**
- Seluruh isi file dibaca ke memori dua kali per event (hash lalu upload). File sangat besar membebani RAM.
- `processed` hanya di memori. Setelah restart, setiap file di-compute hash dan di-bandingkan ulang; karena map kosong, file bisa terupload lagi satu kali.
- Hash hanya dibandingkan per path, bukan per konten global: file identik di dua folder berbeda tetap diupload dua kali.

## Alternatives considered

- **Bandingkan `mtime` + ukuran** — ditolak: tidak akurat dan mudah memicu upload palsu.
- **Cache hash ke disk** — belum dilakukan; kandidat perbaikan untuk mengurangi upload ulang setelah restart.
