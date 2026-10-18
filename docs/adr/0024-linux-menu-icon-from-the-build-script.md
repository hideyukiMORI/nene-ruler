# ADR 0024 — Linux のメニュー用のアイコンも唯一の build.rs が core の絵から作り、窓に WM_CLASS を付ける

- 状態: 受理
- 日付: 2026-10-18
- Issue: #72
- 影響する規則: ARC-001 / RS-010 / CNF-008 / CNF-012。ADR 0023 の build.rs の役目を Linux へ広げる

## 文脈

v0.1.0 で Linux 版を `.deb` と zip で配る（施主決定 2026-10-18）。`.deb` から入れたアプリがメニュー（WSLg なら Windows のスタートメニュー）に出るには、`.desktop` の項目と、アイコンテーマ（hicolor）の画像が要る。ADR 0020 / 0023 は、絵の正本を core の `AppIcon` の 1 か所にし、画像ファイルをリポジトリに置かないとした。

また X11 の窓に `WM_CLASS` が無かった。デスクトップは `WM_CLASS` と `.desktop` の `StartupWMClass` を突き合わせて、窓とメニューの項目（とそのアイコン）を結ぶ。

## 決定

**core に PNG の書き出し（`png_file`）を置き、`crates/app/build.rs` が Linux のターゲットのとき `OUT_DIR/icons/hicolor/<N>x<N>/apps/nene-ruler.png`（16〜256 の 7 枚）を書く。`.desktop` は `packaging/linux/` に置く。X11 の枠とモーダルの窓に `WM_CLASS`（`nene-ruler`）を付ける。**

1. PNG は RGBA を無圧縮の deflate（stored）で包む。CRC32 と Adler-32 は core で計算する（依存 0）。256 px で約 256 KiB
2. 配布物を作る手順は、cargo の `--message-format=json` が出す `build-script-executed` の `out_dir` から画像を拾う（cargo が公開している経路。ハッシュ付きのディレクトリ名を推測しない）
3. build.rs は Windows では資源、Linux では画像を作るだけで、どちらも Rust のコードを生成しない。CNF-008 / CNF-012 の例外は今まで通り `crates/app/build.rs` の 1 つ
4. `WM_CLASS` はインスタンス名・類名とも `nene-ruler`。`.desktop` の `StartupWMClass` と `Icon` も同じ名前

## 強制

- **active**: core の単体テスト（CRC32 と Adler-32 の既知の値・署名と IHDR と IEND・65535 バイトを越える stored の区切り・行の濾過の印）
- **active**: 書いた PNG を外の道具（Python の zlib と `file`）で読み、CRC と画素を確かめた（gate-proofs 第 41 節）
- **planned**: `.deb` から入れたときのメニューの見え（配布の仕組みの Issue と施主の目視）

## 結果

- Linux の配布物にも、同じ絵のアイコンが入る
- Linux のビルドのたびに build.rs が 7 枚を書く（`rerun-if-changed` で絵の変更の時だけ）
- `.desktop` が 1 つ増える。名前（`nene-ruler`）は 3 か所（`WM_CLASS`・`Icon`・`StartupWMClass`）で揃える

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| PNG をリポジトリに置く | 絵の正本が 2 つになる（ADR 0020 の却下理由と同じ） |
| アプリに `--icon-png` のような引数を足して書き出させる | 配布のためだけに製品の入口が増える |
| SVG で置く | 形は書けるが、WSLg のスタートメニューなど SVG を読まない受け手がある。PNG は確実 |
| deflate で圧縮する | 実装が増える。サイズは配布物の中で 1 MiB に満たない |
