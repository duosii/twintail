**| :us: English | :jp: [日本語](./jp.md) |**
# twintail Usage Guide
This guide may not contain usage details for every command.

To view all commands that twintail has, run twintail with the ``help`` flag.
```
twintail --help
```

## App Version and App Hash
Some commands will require you to provide the game's app version and hash.
Any time the game's app receives an update, these values will change.

As of the last time this guide was updated, these were the latest app hashes.
| Server | Version | Hash |
| ------ | ------- | ---- |
| Japan  | ``4.0.5`` | ``2179da72-9de5-23a6-f388-9e5835098ce1``
| Global | ``3.1.0`` | ``a892dc93-798e-4007-8d07-54cb13c9500a``

## ``fetch ab``
Downloads the game's assets.

### Examples
- Download all assets from the Japan server and save them in a folder called ``bundles``.
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 bundles
  ```
- Download only assets that contain ``scenario`` in their name from the global server.
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 --filter "scenario" --server global assets
  ```
- Download assets from the Japan server using an [assetbundle info file](#fetch-ab-info).
  ```
  twintail fetch ab --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 --info 4.0.5.10.json bundles
  ```

## ``fetch ab-info``
Saves a list of all of the game's assets as a ``json`` file for later use.

### Examples
- Downloads and saves a list of all of the games assets to ``asset_version.json`` where ``asset_version`` is the latest asset version.
  ```
  fetch ab-info --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1
  ```

## ``encrypt``
Encrypts Unity assetbundle files for use with Project SEKAI.

### Examples
- Encrypt a single file in-place
  ```
  twintail encrypt files/assetbundle0
  ```
- Encrypt an entire directory, and put the results into a new directory.
  ```
  twintail encrypt ./decrypted_files ./encrypted_files
  ```

## ``decrypt``
Decrypts assetbundles in the game's format for use with other tools.

### Examples
- Decrypt a single file, and put the result into a new file.
  ```
  twintail decrypt encrypted_files/assetbundle0 decrypted_files/assetbundle0
  ```
- Decrypt an entire directory in-place recursively.
  ```
  twintail decrypt --recursive ./encrypted_files
  ```