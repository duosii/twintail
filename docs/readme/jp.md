**| :us: [English](/README.md) | :jp: 日本語 |**
# twintail
プロジェクトセカイ カラフルステージ！のゲームアセットをダウンロード、または暗号化・復号化することができる高速なコマンドラインツールです。

twintailは現在、グローバル版と日本版のサーバーに対応しています。

最新バージョンは[リリースページ](/releases/latest)からダウンロードするか、[ビルド](#building)することができます。

## 使用方法
[使用方法ガイド](/docs/usage/jp.md)をご覧ください。

## ビルド方法
### 依存関係
- お使いのプラットフォーム向けの[Rust](https://www.rust-lang.org/tools/install)をインストールし、最新の状態であることを確認してください。
  ```
  rustup update
  ```

デバッグ用にビルドする場合：
```
cargo run
```

リリース用にビルドする場合：
```
cargo run --release
```

テストを実行する場合：
```
cargo test
```