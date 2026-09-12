# SafeStream Browser

配信者のための安全なブラウザです。Rust + OS標準のWebView（macOSではWebKit）で作られています。

配信中に「戻る」「進む」の誤操作で意図しないサイトに飛んでしまう事故を防ぐため、戻る/進む機能そのものを持たせていません。別のサイトへ移動したいときは、専用のポップアップウィンドウにURLを手打ちします。

## 特徴

- **戻る/進むボタンなし** — UI上にボタンを置かないだけでなく、スワイプによる戻る/進むジェスチャーや、ページ自身が呼び出す `history.back()` / `forward()` / `go()`（iframe内からの呼び出しも含む）を無効化しています。
- **URLは手打ちでのみ移動** — 画面上部メニューバーの「移動」→「URLを開く...」（`Cmd+L`）からポップアップを開き、URLを入力して移動します。
- **危険なスキームを拒否** — `http` / `https` 以外（`file:`、`javascript:` など）への遷移は、リンククリック・手打ち入力のどちらでも拒否します。
- **新規ウィンドウ/タブを増やさない** — `target="_blank"` や `window.open()` によるポップアップ・別タブは作らず、同じ画面内で遷移します。
- **起動時に画面を最大化** — macOSのネイティブフルスクリーン（Spaces切り替えを伴うもの）ではなく、通常のウィンドウのまま画面いっぱいに広げて起動します。

## 動作環境

- macOS
- Rust（[rustup](https://rustup.rs/) でインストール）

## 使い方

### 開発中に実行する

```sh
cargo run
```

### テスト・静的解析

```sh
cargo test
cargo clippy
```

### アプリとしてインストールする

`cargo run` はターミナルから直接バイナリを起動するだけなので、Dockアイコンやアプリ名は反映されません。`.app` バンドルとして使うには以下を実行します。

```sh
chmod +x scripts/bundle_mac.sh   # 初回のみ
./scripts/bundle_mac.sh
```

`target/release/bundle/SafeStream Browser.app` が生成されます。`/Applications` にコピーすればLaunchpadやSpotlightからも起動できます。

```sh
cp -R "target/release/bundle/SafeStream Browser.app" "/Applications/SafeStream Browser.app"
```

## プロジェクト構成

| ファイル | 役割 |
| --- | --- |
| `src/main.rs` | イベントループ本体 |
| `src/browser.rs` | メインウィンドウ・WebViewの生成、履歴操作の無効化、ナビゲーション制御 |
| `src/menu.rs` | macOSメニューバー（「移動」→「URLを開く...」、標準の編集メニュー）の構築 |
| `src/url_popup.rs` | URL手打ち用ポップアップウィンドウ |
| `src/url_input.rs` | URL文字列の正規化・検証（スキーム制限など） |
| `src/event.rs` | イベントループ内で扱うアプリケーション独自イベント |
| `assets/` | アプリアイコン（`icon.svg` / `AppIcon.icns`） |
| `scripts/bundle_mac.sh` | `.app` バンドルを組み立てるビルドスクリプト |
