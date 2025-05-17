**| :us: [English](./en.md) | :jp: 日本語 |**
# twintail コマンド一覧
このガイドには、すべてのコマンドの使用方法の詳細が含まれていない場合があります。

twintailが持つすべてのコマンドを確認するには、``help``フラグを付けて実行してください。
```
twintail --help
```

## ``fetch ab``
ゲームのアセットをダウンロードします。

### 例
- 日本サーバーからすべてのアセットをダウンロードし、``bundles``というフォルダに保存します。
  ```
  twintail fetch ab bundles
  ```
- グローバルサーバーから、名前に``scenario``を含むアセットのみをダウンロードします。
  ```
  twintail fetch ab --filter "scenario" --server global assets
  ```
- [アセットバンドル情報ファイル](#fetch-ab-info)を使用して日本サーバーからアセットをダウンロードします。
  ```
  twintail fetch ab --info 4.0.5.10.json --no-update bundles
  ```
- 最新のアセットバージョンと[アセットバンドル情報ファイル](#fetch-ab-info)の差分のみをダウンロードします。
  ```
  twintail fetch ab --info 4.0.5.10.json bundles
  ```

## ``fetch ab-info``
ゲームの全アセットのリストを``json``ファイルとして保存します。

### 例
- ゲームの全アセットのリストをダウンロードし、``asset_version.json``として保存します（asset_versionは最新のアセットバージョン）。
  ```
  fetch ab-info
  ```

## ``fetch suite``
Suitemasterファイルをダウンロードします。

### 例
- 日本サーバーからSuitemasterファイルをダウンロードし、``suite``というフォルダに保存します。
  ```
  twintail fetch suite suite
  ```
- 日本サーバーから暗号化されたSuitemasterファイルをダウンロードし、``suite_encrypted``というフォルダに保存します。
  ```
  twintail fetch suite --encrypt suite_encrypted
  ```

## ``fetch save``
公式サーバーからプレイヤーのセーブデータをダウンロードします。

### 例
- 日本サーバーからプレイヤーのセーブデータをダウンロードします。
  ```
  twintail fetch save --id <transfer_id> --password <transfer_password>
  ```
  - ``<transfer_id>``と``<transfer_password>``は、引継ぎ開始時にゲームから提供された値です。
- グローバルサーバーからプレイヤーのセーブデータをダウンロードし、``saves``というフォルダに保存します。
  ```
  twintail fetch save --id <transfer_id> --password <transfer_password> --server global
  ```

## ``encrypt ab``
Project SEKAIで使用するUnityアセットバンドルファイルを暗号化します。

### 例
- 単一のファイルをその場で暗号化
  ```
  twintail encrypt ab files/assetbundle0
  ```
- ディレクトリ全体を暗号化し、結果を新しいディレクトリに出力
  ```
  twintail encrypt ab ./decrypted_files ./encrypted_files
  ```

## ``encrypt suite``
Suitemasterファイルを暗号化します。

### 例
- 単一のファイルをその場で暗号化
  ```
  twintail encrypt suite suite/cards.json
  ```
- ディレクトリ全体を暗号化し、結果を新しいディレクトリに出力
  ```
  twintail encrypt suite ./suite ./encrypted_suite
  ```

## ``decrypt ab``
他のツールで使用するためにゲーム形式のアセットバンドルを復号化します。

### 例
- 単一のファイルを復号化し、結果を新しいファイルに出力
  ```
  twintail decrypt ab encrypted_files/assetbundle0 decrypted_files/assetbundle0
  ```
- ディレクトリ全体をその場で再帰的に復号化
  ```
  twintail decrypt ab --recursive ./encrypted_files
  ```

## ``decrypt suite``
Suitemasterファイルを復号化します。

### 例
- 単一のファイルを復号化し、結果を新しいディレクトリに出力
  ```
  twintail decrypt suite suite/00_0m12kmj3k21mvnmx12 decrypted_suite
  ```
- ディレクトリ全体を新しいディレクトリに復号化
  ```
  twintail decrypt suite ./encrypted ./decrypted
  ```