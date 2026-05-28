# 安装

ApexSU 仅用于受支持的 Android GKI 设备。

## 要求

- 内核版本必须包含 `android`  
  例如：`5.10.209-android12-9-00016-g7c6bbcca33e1`
- 与当前固件/构建完全匹配的 `boot.img`
- ApexSU Manager

## non-GKI 设备

- 不受支持。
- 安装会被阻止。
- Local LKM 不是 non-GKI 兼容绕过方案。

## LKM

- Repository LKM：推荐/默认，仅用于受支持的 GKI
- Local LKM：高级手动选项，仅用于受支持的 GKI

## 安全警告

修改 boot image 可能导致 bootloop。安装前请先备份重要数据。
