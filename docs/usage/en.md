**| :us: English | :jp: [日本語](./jp.md) |**
# twintail Usage Guide
The following is a quick guide on how to use twintail.

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
twintail fetch ab assetbundles
```
- This will download all assets for the most recent asset version to a folder named ``assetbundles``
- If you want the assets to go to a different location, replace ``assetbundles`` with the path to your desired location.

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-ab) for this command.

### Suitemaster Files
Suitemaster files are used by the game to know what events are active and what stats character cards have among many other things.

You can use twintail's ``fetch suite`` command to download these files.
```
twintail fetch suite suitemaster_files
```
- This will download all suitemaster files for the most recent asset version to a folder named ``suitemaster_files``
- If you want the files to go to a different location, replace ``suitemaster_files`` with the path to your desired location.

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-suite) for this command.

### Assetbundle Info
The assetbundle info file is what tells the game what assetbundles are currently available to download.

You can use twintail's ``fetch ab-info`` command to download this file.
```
twintail fetch ab-info
```
- This will create a new ``.json`` file in the location you ran the command with the name of the latest asset version (i.e. ``4.1.0.10.json``)

For more in-depth usage details, you can go to the [commands reference](../commands/en.md#fetch-ab-info) for this command.

### Save Data
Save data contains information like what characters, costumes, and songs a player owns among many other things.

In order to use this command, you will first have to start an account transfer process.
1. Login to the game. Once logged in, tap the icon with the three lines in the top right corner of the screen.
2. Locate and tap the button that has an icon that features two phones. "ACCOUNT TRANSFER".
3. A window will pop-up. Tap the big button to the right. "Transfer to different OS".
4. In the password field, enter a secure password.
   - From here on, this password will be referred to as ``transfer_password``
5. If your password is between 8-16 characters, the green button in the bottom right of the pop-up will be tappable.
6. Tap this green button. "Password Settings".
7. A new window will pop-up. Note the information that it gives you, the Transfer ID and Password.
   - From here on, this Transfer ID will be referred to as ``transfer_id``

You can use twintail's ``fetch save`` command to download this file.
```
twintail fetch save --id <transfer_id> --password <transfer_password>
```
- Replace ``<transfer_id>`` and ``<transfer_password>`` with the values the game gave you directly above.

After completing the save download, you will temporarily not be able to login to your account from the device you had it on originally.

To fix this, you need to transfer your account back to the original device by using the same ``transfer_id`` and ``transfer_password`` values you used above.
1. Open the game and stay on the title screen.
2. Tap the icon with the three lines in the top right corner of the screen
3. In the window that opened, tap the button in the top left which has an icon with two phones.
4. A new window will open. Tap the big button on the right. "Transfer to different OS".
5. In the first text field, enter your ``transfer_id``.
6. In the second text field, enter your ``transfer_password``.
7. Tap the green button in the bottom right.
8. A new pop-up will open. Tap the green button in the bottom right again.
9. You should be able to login to the game now.

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
