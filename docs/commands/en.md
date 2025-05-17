**| :us: English | :jp: [日本語](./jp.md) |**
# twintail Command List
This guide may not contain usage details for every command.

To view all commands that twintail has, run twintail with the ``help`` flag.
```
twintail --help
```

## ``fetch ab``
Downloads the game's assets.

### Examples
- Download all assets from the Japan server and save them in a folder called ``bundles``.
  ```
  twintail fetch ab bundles
  ```
- Download only assets that contain ``scenario`` in their name from the global server.
  ```
  twintail fetch ab --filter "scenario" --server global assets
  ```
- Download assets from the Japan server using an [assetbundle info file](#fetch-ab-info).
  ```
  twintail fetch ab --info 4.0.5.10.json --no-update bundles
  ```
- Download only the differences between the latest asset version and an [assetbundle info file](#fetch-ab-info).
  ```
  twintail fetch ab --info 4.0.5.10.json bundles
  ```

## ``fetch ab-info``
Saves a list of all of the game's assets as a ``json`` file for later use.

### Examples
- Downloads and saves a list of all of the games assets to ``asset_version.json`` where ``asset_version`` is the latest asset version.
  ```
  fetch ab-info
  ```

## ``fetch suite``
Downloads suitemaster files.

### Examples
- Download the suitemaster files from the Japan server and save them in a folder called ``suite``.
  ```
  twintail fetch suite suite
  ```
- Download encrypted suitemaster files from the Japan server and save them in a folder called ``suite_encrypted``.
  ```
  twintail fetch suite --encrypt suite_encrypted
  ```

## ``fetch save``
Downloads a player's save data from the official servers.

### Examples
- Download a player's save data from the Japan server
  ```
  twintail fetch save --id <transfer_id> --password <transfer_password>
  ```
  - Where ``<transfer_id>`` and ``<transfer_password>`` are the values the game gave you when you began the OS transfer.
- Download a player's save data from the Global server and save it in a folder called ``saves``.
  ```
  twintail fetch save --id <transfer_id> --password <transfer_password> --server global
  ```

## ``encrypt ab``
Encrypts Unity assetbundle files for use with Project SEKAI.

### Examples
- Encrypt a single file in-place
  ```
  twintail encrypt ab files/assetbundle0
  ```
- Encrypt an entire directory, and put the results into a new directory.
  ```
  twintail encrypt ab ./decrypted_files ./encrypted_files
  ```

## ``encrypt suite``
Encrypts a suitemaster file.

### Examples
- Encrypt a single file in-place
  ```
  twintail encrypt suite suite/cards.json
  ```
- Encrypt an entire directory, and put the results into a new directory.
  ```
  twintail encrypt suite ./suite ./encrypted_suite
  ```

## ``decrypt ab``
Decrypts assetbundles in the game's format for use with other tools.

### Examples
- Decrypt a single file, and put the result into a new file.
  ```
  twintail decrypt ab encrypted_files/assetbundle0 decrypted_files/assetbundle0
  ```
- Decrypt an entire directory in-place recursively.
  ```
  twintail decrypt ab --recursive ./encrypted_files
  ```

## ``decrypt suite``
Decrypts suitemaster files.

### Examples
- Decrypt a single file, and put the result into a new directory.
  ```
  twintail decrypt suite suite/00_0m12kmj3k21mvnmx12 decrypted_suite
  ```
- Decrypt an entire directory into a new directory.
  ```
  twintail decrypt suite ./encrypted ./decrypted
  ```