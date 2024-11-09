**| :us: [English](./en.md) | :jp: 日本語 |**
# twintail 使用ガイド
このガイドには、すべてのコマンドの使用方法が含まれていない場合があります。

twintailが持つすべてのコマンドを確認するには、``help``フラグを付けて実行してください。
```
twintail --help
```

## アプリバージョンとアプリハッシュ
一部のコマンドでは、ゲームのアプリバージョンとハッシュの指定が必要です。
ゲームのアプリがアップデートされるたびに、これらの値は変更されます。

このガイドの最終更新時点での最新のアプリハッシュは以下の通りです。
| サーバー | バージョン | ハッシュ |
| ------ | ------- | ---- |
| 日本サーバー | ``4.0.5`` | ``2179da72-9de5-23a6-f388-9e5835098ce1``
| グローバルサーバー | ``3.1.0`` | ``a892dc93-798e-4007-8d07-54cb13c9500a``

## ``fetch ab``
ゲームのアセットをダウンロードします。

### 使用例
- 日本サーバーからすべてのアセットをダウンロードし、``bundles``フォルダに保存する。
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 bundles
  ```
- グローバルサーバーから名前に``scenario``を含むアセットのみをダウンロードする。
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 --filter "scenario" --server global assets
  ```
- [アセットバンドル情報ファイル](#fetch-ab-info)を使用して日本サーバーからアセットをダウンロードする。
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 --info 4.0.5.10.json bundles
  ```

## ``fetch ab-info``
ゲームの全アセットのリストを``json``ファイルとして保存し、後で使用できるようにします。

### 使用例
- ゲームの全アセットのリストをダウンロードし、最新のアセットバージョンに基づいた``asset_version.json``として保存する。
  ```
  fetch ab-info --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1
  ```

## ``encrypt``
UnityのアセットバンドルファイルをProject SEKAIで使用できるように暗号化します。

### 使用例
- 単一のファイルをその場で暗号化する
  ```
  twintail encrypt files/assetbundle0
  ```
- ディレクトリ全体を暗号化し、結果を新しいディレクトリに出力する
  ```
  twintail encrypt ./decrypted_files ./encrypted_files
  ```

## ``decrypt``
ゲーム形式のアセットバンドルを他のツールで使用できるように復号化します。

### 使用例
- 単一のファイルを復号化し、結果を新しいファイルとして出力する
  ```
  twintail decrypt encrypted_files/assetbundle0 decrypted_files/assetbundle0
  ```
- ディレクトリ全体をその場で再帰的に復号化する
  ```
  twintail decrypt --recursive ./encrypted_files
  ```