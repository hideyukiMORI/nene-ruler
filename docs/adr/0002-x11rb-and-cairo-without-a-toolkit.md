# ADR 0002 — X11 は x11rb、描画は cairo の ImageSurface。UI ツールキットを使わない

- 状態: 受理
- 日付: 2026-09-27
- Issue: #1
- 影響する規則: ARC-002 / ARC-003 / ARC-005 / ARC-007 / ARC-011 / ARC-013 / RS-010

## 文脈

NeNe Ruler は枠の無い細長い窓に目盛と数字を描き、掴んで動かし、端で伸縮し、クリップボードへ書く。
必要なのは「窓を 1 つ出す」「画素を描く」「文字を描く」「選択を所有する」「WM にヒントを渡す」の 5 つだけである。
先行の NeNe Loupe は同じ理由で UI ライブラリを使わず素の Win32 で作った（Loupe ADR 0002）。

開発の主戦場はクラウドの Linux コンテナで、実ディスプレイが無い。Xvfb の上で窓を出し PNG を撮って読むのが検証手段になる。
施主の目視は WSLg（XWayland）である。

Rust で X11 に触る経路を 4 つ比べた（2026-09-27・クラウドコンテナで crate の取得と cairo のヘッダの存在を実測）。

| 経路 | 依存 | `unsafe` | 備考 |
| --- | --- | --- | --- |
| `x11rb`（既定 feature） | 純 Rust の X プロトコル実装。libX11 に結ばない | 不要 | 拡張（RandR・XFixes 等）も同じ crate |
| `x11-dl` / `x11` | libX11 を動的に読む | 全 API が `unsafe` | RS-010 に反する |
| `cairo-rs` の `XlibSurface` / `XCBSurface` | libX11 か libxcb の接続ポインタが要る | 接続の取り出しが `unsafe` | x11rb の純 Rust 接続とは結べない |
| `cairo-rs` の `ImageSurface` | libcairo（MIT 互換の LGPL/MPL 二重）。X には依存しない | 不要 | 画素の列をメモリに描き、X へは `PutImage` で転送する |

文字の描画に cairo の toy text API（`show_text`）を使えば、フォントは fontconfig から得られ、pango を足さずに済む。

## 決定

**X との会話は `x11rb`（純 Rust）、描画は `cairo-rs` の `ImageSurface` に完成させてから `PutImage` で転送する。UI ツールキットは使わない。**

1. `crates/ui-x11` が窓・イベント・`ImageSurface`・転送を持つ。`crates/adapters-x11` が RandR の物理寸法・`CLIPBOARD` の所有・設定の保存・時刻を持つ。
   両者は `x11rb` を別々の接続で使う。接続を共有したくなったら application のポートに畳んで ADR を切る（ARC-002）
2. 枠は `_MOTIF_WM_HINTS` で装飾だけを外し、窓は WM に管理させる。override-redirect は使わない（フォーカス・重なり順・Snap を WM に任せる）
3. 常に最前面は `_NET_WM_STATE_ABOVE`。ドラッグは自前の座標計算ではなく `_NET_WM_MOVERESIZE` を優先し、対応しない WM では自前に落ちる
4. 描画は `ImageSurface`（ARGB32）に完成させる。**画素の列は X 無しで生成でき、単体テストで色と位置を検証する**（QLT-013 の「単体テストは表示サーバを要しない」の実現手段）
5. `unsafe` を要する経路（`XCBSurface`・共有メモリ `MIT-SHM`）は採らない。速さが足りなければ実測を添えて別の ADR を切る
6. 依存は `x11rb` と `cairo-rs`（＋その `-sys`）だけ。`deny.toml` の `allow` は足すときに `cargo deny list` の実測で更新する

## 強制

- **active**: ARC-013（依存の許可制）。`x11rb` / `cairo-rs` 以外の UI 系 crate は CNF-013 の許可表と `deny.toml` で拒む。`unsafe` は `unsafe_code = "forbid"`
- **planned**: `ImageSurface` 経由でしか描かないこと（`XlibSurface` の禁止）。最初の縦切りで CNF の禁止語に足す

## 結果

得られるもの:

- libX11 に依存しない。配るのは 1 バイナリ ＋ システムの libcairo
- 画素の列がテスト可能になり、クラウドでの検証が「Xvfb で撮った PNG」と「単体テストの画素」の 2 段になる
- `unsafe` ゼロが保てる

失うもの:

- `PutImage` は毎フレーム画素を X へ送る。56×400 px 程度の窓では問題にならないが、実測してから確定する（QLT-013）
- toy text API は複雑な文字（結合文字・縦書き）を扱わない。数字と `px` / `mm` / `in` だけなので初版は足りる
- libcairo がシステムに要る（WSLg の Ubuntu には既定で入っている。無ければ `libcairo2`）

**正直に記録しておくこと:** 本 ADR は依存の取得と cairo のヘッダの存在までしか実測していない。窓が出ること・WSLg で動くことは最初の縦切りで実測する。

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| GTK 4 / Qt | 5 つの機能のために巨大な実行時依存とビルド依存を持ち込む。枠なし窓の制御と画素単位の描画は、ツールキットの抽象を迂回して書くことになる |
| `egui` / `iced` / `slint` | 即時モードや宣言的 UI の抽象が、物差しの「画素をそのまま描く」要求と噛み合わない。依存が数十 crate になり ARC-013 の許可制が形骸化する |
| `x11-dl` で Xlib を直接呼ぶ | 全 API が `unsafe`。RS-010 を最初の Issue で捨てることになる |
| `cairo-rs` の `XCBSurface` | x11rb の純 Rust 接続からは xcb のポインタが取れない。`allow-unsafe-code` feature が要る |
| 純 Rust の描画（`tiny-skia` ＋ `ab_glyph`） | C ライブラリへのリンク 0 にはなるが、フォントの探索（fontconfig）を自前で書くか埋め込むかの判断が増える。目盛と数字だけの初版には重い。速さや配布の要件で libcairo が問題になったら再検討する |
| Wayland ネイティブ（`smithay-client-toolkit`） | 枠なし・常に最前面・絶対座標の移動が Wayland では原理的に制限される。物差しは画面上の絶対位置が意味を持つ道具なので X11 を主とする。WSLg は XWayland を持つ |
