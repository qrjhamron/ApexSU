# インストール

ApexSU はサポート対象の Android GKI デバイス専用です。

## 対応条件

- カーネルバージョンに `android` が含まれること  
  例: `5.10.209-android12-9-00016-g7c6bbcca33e1`
- 現在のファームウェア/ビルドに一致する `boot.img`
- ApexSU Manager

## 非 GKI の扱い

- 非 GKI は非対応です。
- インストールはブロックされます。
- Local LKM は非 GKI の回避策ではありません。

## LKM

- Repository LKM: 推奨/既定（対応 GKI 向け）
- Local LKM: 手動の上級者向け（対応 GKI のみ）

## 安全上の注意

boot イメージの変更はブートループを引き起こす可能性があります。必ず重要データをバックアップしてください。
