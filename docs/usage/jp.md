**| :us: [English](./en.md) | :jp: 日本語 |**
# twintail 使用ガイド
以下はtwintailの使用方法についての簡単なガイドです。

## アプリバージョンとアプリハッシュ
一部のコマンドではゲームのアプリバージョンとハッシュの指定が必要です。
ゲームアプリが更新されるたびに、これらの値は変更されます。

このガイドの最終更新時点での最新のアプリハッシュは以下の通りです。
| サーバー | バージョン | ハッシュ |
| ------ | ------- | ---- |
| 日本  | ``5.0.0`` | ``746b8607-0e65-489d-b060-a8986ba11b47``
| グローバル | ``3.2.0`` | ``1380836d-de9e-49a8-afe5-c52ba589c8c9``

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

### セーブデータ
セーブデータには、プレイヤーが所持しているキャラクター、衣装、楽曲など、多くの情報が含まれています。

このコマンドを使用するには、まず引き継ぎ手続きを開始する必要があります。
1. ゲームにログインします。ログイン後、画面右上の三本線のアイコンをタップします。
2. 2台の携帯電話のアイコンが表示されているボタン「引き継ぎ」をタップします。
3. ポップアップウィンドウが表示されます。右側の大きなボタン「異なるOS間での引き継ぎ」をタップします。
4. パスワード欄に、安全なパスワードを入力します。
   - 以降、このパスワードを``transfer_password``と呼びます
5. パスワードが8～16文字の間であれば、ポップアップ右下の緑色のボタンがタップ可能になります。
6. この緑色のボタン「パスワード設定」をタップします。
7. 新しいウィンドウが表示されます。表示される引き継ぎIDとパスワードの情報をメモしてください。
   - 以降、この引き継ぎIDを``transfer_id``と呼びます

twintailの``fetch save``コマンドを使用してこのファイルをダウンロードできます。
```
twintail fetch save --version <app_version> --hash <app_hash> --id <transfer_id> --password <transfer_password>
```
- ``<transfer_id>``と``<transfer_password>``を、上記で取得した値に置き換えてください。
- ``<app_version>``と``<app_hash>``を、[前述の手順](#アプリバージョンとアプリハッシュ)で取得した値に置き換えてください。

セーブデータのダウンロードが完了すると、一時的に元のデバイスからアカウントにログインできなくなります。

これを解決するには、上記で使用した同じ``transfer_id``と``transfer_password``を使用して、アカウントを元のデバイスに戻す必要があります。
1. ゲームを開き、タイトル画面で待機します。
2. 画面右上の三本線のアイコンをタップします。
3. 開いたウィンドウで、2台の携帯電話のアイコンが表示されている左上のボタンをタップします。
4. 新しいウィンドウが開きます。右側の大きなボタン「異なるOS間での引き継ぎ」をタップします。
5. 最初のテキストフィールドに``transfer_id``を入力します。
6. 2番目のテキストフィールドに``transfer_password``を入力します。
7. 右下の緑色のボタンをタップします。
8. 新しいポップアップが開きます。右下の緑色のボタンを再度タップします。
9. これでゲームにログインできるようになるはずです。

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