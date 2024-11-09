**| :us: English | :jp: [日本語](docs/readme/jp.md) |**
# twintail
A fast command-line tool for Project SEKAI (プロジェクトセカイ カラフルステージ！) that allows you to download the game's assets or encrypt and decrypt them.

twintail currently supports the game's global and Japan servers.

You can download the most recent version from the [releases page](../releases/latest) or [build it](#building).

## Usage
Follow the [usage guide](/docs/usage/en.md)

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
