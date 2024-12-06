**| :us: [English](../../README.md) | :jp: 日本語 |**
# twintail
プロジェクトセカイ カラフルステージ！のゲームアセットをダウンロード、または暗号化・復号化できる高速なコマンドラインツールです。

twintailは現在、グローバル版と日本版のサーバーに対応しています。

最新バージョンは[リリースページ](https://github.com/Duosion/twintail/releases/latest)からダウンロードするか、[ビルド](#ビルド)することができます。

## 使用方法
- twintailの基本的な使い方については[使用ガイド](../usage/jp.md)をご覧ください。
- コマンドの例付きリストについては[コマンドリファレンス](../commands/jp.md)をご覧ください。

## ビルド
### 依存関係
- お使いのプラットフォーム向けの[Rust](https://www.rust-lang.org/tools/install)をインストールし、最新の状態であることを確認してください。
  ```
  rustup update
  ```

デバッグ用にビルドする場合：
```
cargo run -F cli
```

リリース用にビルドする場合：
```
cargo run -F cli --release
```

テストを実行する場合：
```
cargo test
```