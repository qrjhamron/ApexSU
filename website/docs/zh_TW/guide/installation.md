# 安裝

ApexSU 僅用於受支援的 Android GKI 裝置。

## 要求

- 核心版本必須包含 `android`  
  例如：`5.10.209-android12-9-00016-g7c6bbcca33e1`
- 與目前韌體/版本完全匹配的 `boot.img`
- ApexSU Manager

## non-GKI 裝置

- 不受支援。
- 安裝會被阻擋。
- Local LKM 不是 non-GKI 相容性繞過方式。

## LKM

- Repository LKM：建議/預設，僅適用受支援 GKI
- Local LKM：進階手動選項，僅適用受支援 GKI

## 安全警告

修改 boot image 可能造成 bootloop。安裝前請先備份重要資料。
