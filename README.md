# twintail
A fast command-line tool for Project SEKAI (プロジェクトセカイ カラフルステージ！) that allows you to download the game's assets or encrypt and decrypt them.

You can download the most recent version from the [releases page](../releases/latest) or [build it](./#twintail).

## Commands

### ``encrypt``
Encrypts Unity assetbundle files for use with Project SEKAI.

**Examples**
- Encrypt a single file in-place
  ```
  twintail.exe encrypt files/assetbundle0
  ```
- Encrypt an entire directory, and put the results into a new directory.
  ```
  twintail.exe encrypt ./decrypted_files ./encrypted_files
  ```

### ``decrypt``
Decrypts assetbundles in the game's format for use with other tools.

**Examples**
- Decrypt a single file, and put the result into a new file.
  ```
  twintail.exe decrypt encrypted_files/assetbundle0 decrypted_files/assetbundle0
  ```
- Decrypt an entire directory in-place recursively.
  ```
  twintail.exe decrypt --recursive ./encrypted_files
  ```

## Building
### Dependencies
- Install [Rust](https://www.rust-lang.org/tools/install) for your platform and ensure that it's up-to-date.
  ```
  rustup update
  ```

To build for debugging:
```
cargo run
```

To build for release:
```
cargo run --release
```

To run tests:
```
cargo test
```
