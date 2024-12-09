**| :us: English | :jp: [日本語](./jp.md) |**
# twintail Usage Guide
The following is a quick guide on how to use twintail.

## App Version and App Hash
Some commands will require you to provide the game's app version and hash.
Any time the game's app receives an update, these values will change.

As of the last time this guide was updated, these were the latest app hashes.
| Server | Version | Hash |
| ------ | ------- | ---- |
| Japan  | ``4.1.1`` | ``41fd71f2-f715-bc10-5852-0a9d8542f760``
| Global | ``3.1.5`` | ``fb23ab54-5371-45d4-abf3-c5531bc58b01``

### Getting the most recent app version and hash
If the values in the table above are outdated, you can get the latest ones using twintail's ``app-info`` command.
1. Acquire a copy of the Android APK for the server of your choice.
2. Use the app-info command to extract the app's version and hash from the APK file.
   ```
   twintail app-info latest-japan.apk
   ```
3. Use the values that twintail outputs.

## Server Selection
Some commands will require you to specify a game server.

This includes **all** download commands and some encryption/decryption commands

If not specified, these commands will download from the Japan server by default.

If you want to download from the Global server, you will have to specify this by adding ``--server global`` to your command.

## Downloading Assets

### Assetbundles
Assetbundles are the game's main type of assets, containing everything from music to 3D models.

You can use twintail's ``fetch ab`` command to download these assets.
```
twintail fetch ab --version <app_version> --hash <app_hash> assetbundles
```
- Replace ``<app_version>`` and ``<app_hash>`` with the values you aquired [above](#app-version-and-app-hash).
- This will download all assets for the most recent asset version to a folder named ``assetbundles``
- If you want the assets to go to a different location, replace ``assetbundles`` with the path to your desired location.

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-ab) for this command.

### Suitemaster Files
Suitemaster files are used by the game to know what events are active and what stats character cards have among many other things.

You can use twintail's ``fetch suite`` command to download these files.
```
twintail fetch suite --version <app_version> --hash <app_hash> suitemaster_files
```
- Replace ``<app_version>`` and ``<app_hash>`` with the values you aquired [above](#app-version-and-app-hash).
- This will download all suitemaster files for the most recent asset version to a folder named ``suitemaster_files``
- If you want the files to go to a different location, replace ``suitemaster_files`` with the path to your desired location.

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-suite) for this command.

### Assetbundle Info
The assetbundle info file is what tells the game what assetbundles are currently available to download.

You can use twintail's ``fetch ab-info`` command to download this file.
```
twintail fetch ab-info --version <app_version> --hash <app_hash>
```
- Replace ``<app_version>`` and ``<app_hash>`` with the values you aquired [above](#app-version-and-app-hash).
- This will create a new ``.json`` file in the location you ran the command with the name of the latest asset version (i.e. ``4.1.0.10.json``)

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-ab-info) for this command.

## Encrypting and Decrypting Assets
By default, all files that twintail downloads will be decrypted.

If you wanted to encrypt these assets again, you can use twintail's encryption and decryption commands.

### Assetbundles
The game expects the assetbundles it downloads to be encrypted in a particular way.

You can encrypt and decrypt them by using twintail's ``encrypt ab`` and ``decrypt ab`` commands.
```
twintail encrypt ab <directory_of_decrypted_assetbundles>
```
- Replace ``<directory_of_decrypted_assetbundles>`` with the location of decrypted assetbundles you want to encrypt.
- If you want to recursively encrypt (encrypt every file inside of a folder, including ones inside of other folders), you can use the ``--recursive`` flag.

```
twintail decrypt ab <directory_of_encrypted_assetbundles>
```
- Replace ``<directory_of_decrypted_assetbundles>`` with the location of encrypted assetbundles you want to decrypt.


For more in-depth usage details, you can go to the commands reference for the [encrypt](../commands/en.md#encrypt-ab) and [decrypt](../commands/en.md#decrypt-ab) commands.


### Suitemaster Files
When the game downloads the suitemaster files from the server, they are encrypted.

**Due to the way suitemaster file encryption/decryption is handled, you will have to [specify the server](#server-selection) that the suitemaster files were downloaded from**

You can encrypt and decrypt them by using twintail's ``encrypt suite`` and ``decrypt suite`` commands.
```
twintail encrypt suite <directory_of_suitemaster_files> encrypted_suite
```
- Replace ``<directory_of_suitemaster_files>`` with the location of the suitemaster files you want to encrypt.
- This will encrypt the files in ``<directory_of_suitemaster_files>`` into a directory named ``encrypted_suite``

```
twintail decrypt suite <directory_of_encrypted_suitemaster_files> decrypted_suite
```
- Replace ``<directory_of_encrypted_suitemaster_files>`` with the location of encrypted suitemaster files you want to decrypt.
- This will decrypt the files in ``<directory_of_encrypted_suitemaster_files>`` into a directory named ``decrypted_suite``

For more in-depth usage details, you can go to the commands reference for the [encrypt](../commands/en.md#encrypt-suite) and [decrypt](../commands/en.md#decrypt-suite) commands.