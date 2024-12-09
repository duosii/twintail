**| :us: [English](./en.md) | :jp: 日本語 |**
# twintail 使用ガイド
以下はtwintailの使用方法についての簡単なガイドです。

## アプリバージョンとアプリハッシュ
一部のコマンドではゲームのアプリバージョンとハッシュの指定が必要です。
ゲームアプリが更新されるたびに、これらの値は変更されます。

このガイドの最終更新時点での最新のアプリハッシュは以下の通りです。
| サーバー | バージョン | ハッシュ |
| ------ | ------- | ---- |
| 日本  | ``4.1.1`` | ``41fd71f2-f715-bc10-5852-0a9d8542f760``
| グローバル | ``3.1.5`` | ``fb23ab54-5371-45d4-abf3-c5531bc58b01``

### 最新のアプリバージョンとハッシュの取得方法
上記の表の値が古くなっている場合、twintailの``app-info``コマンドを使用して最新の値を取得できます。
1. 希望するサーバーのAndroid APKファイルを入手します。
2. app-infoコマンドを使用してAPKファイルからアプリのバージョンとハッシュを抽出します。
   ```
   twintail app-info latest-japan.apk
   ```
3. twintailが出力した値を使用します。

## サーバーの選択
一部のコマンドではゲームサーバーの指定が必要です。

これには**すべての**ダウンロードコマンドと一部の暗号化/復号化コマンドが含まれます。

指定がない場合、これらのコマンドはデフォルトで日本サーバーからダウンロードします。

グローバルサーバーからダウンロードする場合は、コマンドに``--server global``を追加する必要があります。

## アセットのダウンロード

### アセットバンドル
アセットバンドルはゲームの主要なアセットタイプで、音楽から3Dモデルまですべてが含まれています。

twintailの``fetch ab``コマンドを使用してこれらのアセットをダウンロードできます。
```
twintail fetch ab --version <app_version> --hash <app_hash> assetbundles
```
- ``<app_version>``と``<app_hash>``を[上記](#アプリバージョンとアプリハッシュ)で取得した値に置き換えてください。
- これにより、最新のアセットバージョンのすべてのアセットが``assetbundles``という名前のフォルダにダウンロードされます。
- アセットを別の場所に保存したい場合は、``assetbundles``を希望の保存先パスに置き換えてください。

より詳細な使用方法については、このコマンドの[コマンドリファレンス](../commands/jp.md#fetch-ab)を参照してください。

### Suitemasterファイル
Suitemasterファイルは、アクティブなイベントやキャラクターカードのステータスなど、多くの情報をゲームに提供するために使用されます。

twintailの``fetch suite``コマンドを使用してこれらのファイルをダウンロードできます。
```
twintail fetch suite --version <app_version> --hash <app_hash> suitemaster_files
```
- ``<app_version>``と``<app_hash>``を[上記](#アプリバージョンとアプリハッシュ)で取得した値に置き換えてください。
- これにより、最新のアセットバージョンのすべてのsuitemasterファイルが``suitemaster_files``という名前のフォルダにダウンロードされます。
- ファイルを別の場所に保存したい場合は、``suitemaster_files``を希望の保存先パスに置き換えてください。

より詳細な使用方法については、このコマンドの[コマンドリファレンス](../commands/jp.md#fetch-suite)を参照してください。

### アセットバンドル情報
アセットバンドル情報ファイルは、現在ダウンロード可能なアセットバンドルをゲームに伝えるものです。

twintailの``fetch ab-info``コマンドを使用してこのファイルをダウンロードできます。
```
twintail fetch ab-info --version <app_version> --hash <app_hash>
```
- ``<app_version>``と``<app_hash>``を[上記](#アプリバージョンとアプリハッシュ)で取得した値に置き換えてください。
- これにより、コマンドを実行した場所に最新のアセットバージョンの名前（例：``4.1.0.10.json``）を持つ新しい``.json``ファイルが作成されます。

より詳細な使用方法については、このコマンドの[コマンドリファレンス](../commands/jp.md#fetch-ab-info)を参照してください。

## アセットの暗号化と復号化
デフォルトでは、twintailがダウンロードするすべてのファイルは復号化されています。

これらのアセットを再度暗号化したい場合は、twintailの暗号化および復号化コマンドを使用できます。

### アセットバンドル
ゲームはダウンロードしたアセットバンドルが特定の方法で暗号化されていることを期待します。

twintailの``encrypt ab``および``decrypt ab``コマンドを使用して暗号化と復号化ができます。
```
twintail encrypt ab <directory_of_decrypted_assetbundles>
```
- ``<directory_of_decrypted_assetbundles>``を暗号化したい復号化済みアセットバンドルの場所に置き換えてください。
- フォルダ内のすべてのファイル（他のフォルダ内のファイルを含む）を再帰的に暗号化したい場合は、``--recursive``フラグを使用できます。

```
twintail decrypt ab <directory_of_encrypted_assetbundles>
```
- ``<directory_of_encrypted_assetbundles>``を復号化したい暗号化済みアセットバンドルの場所に置き換えてください。

より詳細な使用方法については、[暗号化](../commands/jp.md#encrypt-ab)と[復号化](../commands/jp.md#decrypt-ab)コマンドのコマンドリファレンスを参照してください。

### Suitemasterファイル
ゲームがサーバーからsuitemasterファイルをダウンロードする際、それらは暗号化されています。

**suitemasterファイルの暗号化/復号化の仕組み上、ファイルがダウンロードされた[サーバーを指定](#サーバーの選択)する必要があります**

twintailの``encrypt suite``および``decrypt suite``コマンドを使用して暗号化と復号化ができます。
```
twintail encrypt suite <directory_of_suitemaster_files> encrypted_suite
```
- ``<directory_of_suitemaster_files>``を暗号化したいsuitemasterファイルの場所に置き換えてください。
- これにより、``<directory_of_suitemaster_files>``内のファイルが``encrypted_suite``という名前のディレクトリに暗号化されます。

```
twintail decrypt suite <directory_of_encrypted_suitemaster_files> decrypted_suite
```
- ``<directory_of_encrypted_suitemaster_files>``を復号化したい暗号化済みsuitemasterファイルの場所に置き換えてください。
- これにより、``<directory_of_encrypted_suitemaster_files>``内のファイルが``decrypted_suite``という名前のディレクトリに復号化されます。

より詳細な使用方法については、[暗号化](../commands/jp.md#encrypt-suite)と[復号化](../commands/jp.md#decrypt-suite)コマンドのコマンドリファレンスを参照してください。