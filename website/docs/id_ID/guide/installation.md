# Instalasi

## Persyaratan

ApexSU dirancang secara eksklusif untuk perangkat GKI Android yang didukung.

- **Perangkat GKI Android yang didukung**: Versi kernel Anda harus mengandung kata kunci "android". Jika tidak, perangkat Anda dianggap sebagai perangkat non-GKI dan tidak didukung.
- **Boot image**: Anda harus memiliki boot image yang cocok dengan firmware/build Anda saat ini.
- **ApexSU Manager**: Untuk mengelola instalasi dan modul Anda.
- **Koneksi internet**: Diperlukan jika menggunakan Repository LKM.
- **File .ko lokal**: Hanya diperlukan jika menggunakan LKM lokal pada perangkat GKI yang didukung.

## Perangkat yang tidak didukung

Perangkat non-GKI **tidak didukung**.
- Instalasi diblokir dan tidak boleh dilanjutkan pada perangkat non-GKI.
- LKM lokal bukan jalan pintas untuk membuat perangkat non-GKI menjadi kompatibel.

::: warning
Perangkat non-GKI tidak didukung. ApexSU tidak dapat dipasang pada perangkat non-GKI, dan LKM lokal bukan jalan pintas untuk membuat perangkat non-GKI menjadi kompatibel.
:::

## Pemeriksaan GKI

Untuk memverifikasi apakah perangkat Anda didukung:
1. Buka ApexSU Manager.
2. Periksa versi kernel Anda di layar beranda.
3. Jika kernel mengandung kata kunci "android" (misalnya, `5.10.209-android12-9-00016-g7c6bbcca33e1`), perangkat Anda dianggap sebagai perangkat GKI dan didukung.
4. Jika tidak mengandung "android", perangkat tersebut tidak didukung dan instalasi akan diblokir.

## Opsi LKM

ApexSU mendukung dua metode instalasi untuk loadable kernel module (LKM) pada perangkat GKI yang didukung:

- **Repository LKM**: Opsi yang direkomendasikan dan default untuk perangkat GKI. Ini mengunduh modul yang diperlukan secara otomatis berdasarkan versi kernel Anda.
- **LKM lokal**: Opsi manual lanjutan yang hanya untuk perangkat GKI yang didukung. Ini mengharuskan Anda menyediakan file `.ko` lokal yang kompatibel.

*Catatan: Pada perangkat non-GKI, instalasi diblokir dan tidak ada opsi LKM yang dapat digunakan.*

## Perilaku tombol instalasi

Saat Anda mencoba menginstal ApexSU melalui Manager:
- **Pada perangkat GKI**: Anda harus memberikan boot.img yang sesuai dengan firmware Anda dan memilih Repository LKM atau LKM lokal yang valid. Tombol instalasi akan melanjutkan proses patching.
- **Pada perangkat non-GKI**: Tombol instalasi dinonaktifkan dan instalasi diblokir.

## Cadangkan boot.img bawaan (stock)

Sebelum patching, sangat penting untuk mencadangkan boot.img bawaan Anda. Melakukan root atau memodifikasi boot image dapat menyebabkan bootloop pada perangkat. Jika Anda mengalami bootloop, Anda dapat memulihkan sistem dengan mem-flash boot image bawaan melalui fastboot. Harap cadangkan semua data penting di perangkat Anda.

## Tingkat patch keamanan

Perangkat Android yang lebih baru mungkin memiliki mekanisme anti-rollback yang mencegah flashing boot image dengan tingkat patch keamanan lama. Selalu gunakan boot image yang persis sama dengan build firmware Anda untuk menghindari bootloop.

## Patching manual via Manager

1. Buka ApexSU Manager.
2. Pastikan manajer melaporkan perangkat Anda sebagai perangkat GKI yang didukung.
3. Klik tombol instal dan pilih untuk patch boot.img bawaan Anda.
4. Flash boot image yang telah dipatch ke perangkat Anda menggunakan fastboot:
   ```sh
   fastboot flash boot patched_boot.img
   fastboot reboot
   ```

::: warning
Jangan pernah mencoba memaksa instalasi atau mem-flash modul sembarangan pada perangkat yang tidak didukung.
:::
