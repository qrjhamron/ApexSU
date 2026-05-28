# Cài đặt

ApexSU chỉ dành cho thiết bị Android GKI được hỗ trợ.

## Yêu cầu

- Phiên bản kernel phải chứa `android`  
  Ví dụ: `5.10.209-android12-9-00016-g7c6bbcca33e1`
- `boot.img` khớp chính xác firmware/build hiện tại
- ApexSU Manager

## Thiết bị non-GKI

- Không được hỗ trợ.
- Cài đặt bị chặn.
- Local LKM không phải giải pháp vòng cho non-GKI.

## LKM

- Repository LKM: mặc định/khuyến nghị cho GKI được hỗ trợ
- Local LKM: tùy chọn thủ công nâng cao, chỉ cho GKI được hỗ trợ

## Cảnh báo an toàn

Sửa đổi boot image có thể gây bootloop. Hãy sao lưu dữ liệu quan trọng trước khi cài đặt.
