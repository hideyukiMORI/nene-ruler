# ADR 0021 — Windows 版は Win32 の窓と Direct2D で描き、unsafe は Windows の 2 クレートにだけ許す

- 状態: 受理
- 日付: 2026-10-18
- Issue: #60
- 影響する規則: ARC-002 / ARC-003 / ARC-007 / ARC-013 / RS-010 / RS-019 / QLT-003 / QLT-010 / CNF-012 / CNF-013。ADR 0002 を Windows について補う

## 文脈

施主は、最終的に Windows 版と Linux 版を GitHub で配り、Microsoft Store に出す（2026-10-17）。いまの定規は X11 専用（ADR 0002: x11rb ＋ cairo）で、Windows では WSLg か X サーバが要る。

2026-10-18、施主は次を決めた。

- Windows の描画は **Direct2D / DirectWrite**（cairo を Windows に持ち込まない・純 Rust の描画に揃えない）
- `unsafe` は **Windows の 2 クレートだけ**に許す（RS-010 の例外）

実測（2026-10-18）:

- WSL に `x86_64-pc-windows-gnu` と mingw-w64 を入れ、`windows` 0.62.2 で「`WS_EX_LAYERED` の窓に Direct2D の DC レンダーターゲットで描き、`UpdateLayeredWindow` で出す」試作を作った。1.4 MB の単体の exe になり、施主の Windows で透明な最前面の窓に 1 px の線と字が描けた
- Windows の layered 窓は、アルファ 0 の画素をクリックが抜ける所として扱う。WSLg と同じで、ADR 0008 のアルファ 1 の下塗りがそのまま当たり判定になる
- `windows` クレートの Win32 の呼び出しは、ほぼすべて `unsafe`

## 決定

**Windows 版は、中核（core / application）をそのまま使い、新しい 2 クレートで実装する。**

1. `crates/settings-file`（`nene-ruler-settings-file`）: 設定ファイルの形（v1）と置き場を `adapters-x11` から切り出す。標準ライブラリだけで、Linux は XDG、Windows は `%APPDATA%\nene-ruler\settings.v1.txt`。外部クレートを持たない（CNF-013）
2. `crates/adapters-windows`（`nene-ruler-adapters-windows`）: 主モニタの画素数と dpi（`MonitorFromPoint`・`GetMonitorInfoW`・`GetDpiForMonitor`）、クリップボード（`CF_UNICODETEXT`）。dpi は `Environment::with_desktop_dpi` で渡す（X11 の `Xft.dpi` と同じ口。`xft_dpi` から改名した）
3. `crates/ui-win32`（`nene-ruler-ui-win32`）: `WS_EX_LAYERED` の枠の窓とモーダル。Direct2D の DC レンダーターゲットに描き、`UpdateLayeredWindow` で出す。字は DirectWrite。最前面は `HWND_TOPMOST`、移動と伸縮は `WM_NCLBUTTONDOWN`（`HTCAPTION` / `HTTOPLEFT` / `HTBOTTOMRIGHT`）、ポインタは `IDC_SIZEALL` / `IDC_SIZENWSE` / `IDC_HAND`、アイコンは ADR 0020 の絵を Direct2D で描く。per-monitor v2 の dpi 対応にして、測る値は物理画素のまま
4. 置き場と当たり判定は core の `FrameLayout` / `SettingsLayout` から出す（ARC-001）。描く手続きだけが X11（cairo）と Windows（Direct2D）で 2 本になる
5. `crates/app` は `cfg(target_os)` で合成ルートを選ぶ。Linux は今まで通り、Windows は新しい 2 クレート
6. 依存: windows-rs の `windows` と `windows-numerics`（Direct2D の行列と点の型）だけ（Microsoft・MIT OR Apache-2.0）。Windows のターゲットでだけ引く（`[target.'cfg(windows)'.dependencies]`）。`windows` の手続きマクロが引く `unicode-ident` の `Unicode-3.0` を `deny.toml` の許可に足す
7. **unsafe**: この 2 クレートだけ、`unsafe_code` を `deny` にする。`unsafe` は Win32 を呼ぶ小さな関数に閉じ込め、関数ごとに `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:` の注釈を付ける。`clippy::undocumented_unsafe_blocks` を `deny`。他のクレートは `forbid` のまま
8. この 2 クレートは workspace の lint を継承できない（Cargo は継承した表の 1 項目だけを変えられない）。規約検査に「2 クレートの lint の表は workspace と同じで、違うのは `unsafe_code` だけ」の検査（CNF-014）を足す
9. 検査: `make lint-windows` が `--target x86_64-pc-windows-gnu` で Windows のクレートに clippy を回し、`make build-windows` が exe を作る（`rust-toolchain.toml` にターゲットを固定・リンクは mingw-w64）。Linux の `make lint` では Windows のコードは cfg で外れて見えない
10. CNF-013 は `[target.….dependencies]` の節も読む（今までは `[dependencies]` だけで、OS ごとの依存を素通りさせていた）。CNF-012 の除外に `adapters-windows` と `settings-file` を足す
11. 窓の手続きは失敗を返せない。描き直しの失敗は握り、次の描き直しに任せる（ARC-009 の例外。起動時の失敗は今まで通り合成ルートが示す）

## 強制

- **active**: 2 クレート以外の `unsafe` は workspace の `forbid`（E0453）。2 クレートの `unsafe` は `#[expect]` が無いと `deny` で落ち、注釈が無いと `undocumented_unsafe_blocks` で落ちる
- **active**: CNF-013 の許可表に 2 クレートを足す。CNF-014（lint の表の一致）
- **active**: `make lint-windows`（Windows のターゲットの clippy）
- **planned**: Windows の CI（windows-2025）でのテストの実行。いま Windows でだけ組み立てるコードの単体テストは無く、OS に依らない対応（`NonClientHit`）は Linux でテストする

## 結果

得られるもの:

- Windows で単体の exe として動く。cairo も X サーバも要らない
- 置き場・当たり判定・状態・設定の形は Linux 版と同じ 1 か所から出る

失うもの・残る穴:

- 描く手続きが 2 本になる（cairo と Direct2D）。見た目のずれは画素テストでは揃えきれない（字も Segoe UI になる）
- コピーの成功の「反転」と歯車の穴は、Direct2D 1.1 の `D2D1_PRIMITIVE_BLEND_COPY` でアルファ 1 を置く。古い Direct2D では普通に重なり、字が見えなくなる
- 窓の作成直後の `WM_NCCALCSIZE`（wparam = 0）に答えないと、`WS_THICKFRAME` の太さが差し引かれて枠が 16 px 小さく始まった（実測: 400 が 384）。いまは常に 0 を返す
- `unsafe` を書くクレートができる。境界は 2 クレートと `#[expect]` の痕跡で追える
- lint の表を 2 か所に書く（CNF-014 で一致を守る）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| Windows でも cairo（vcpkg で静的に組み込む） | 施主は Direct2D を選んだ。ビルドの手順が重く、字は Windows の GDI 由来になる |
| 純 Rust の描画（tiny-skia ＋ ab_glyph ＋同梱フォント）に揃える | 施主は Direct2D を選んだ。Windows の字（ClearType・Segoe UI）に馴染まない |
| Win32 を包む安全な関数だけの 1 クレートに unsafe を閉じ込める | 境界は一番きれいだが、包む関数を全部書く分の作業が増える。施主は 2 クレートに許す方を選んだ |
| `winsafe` などの安全なラッパー crate | Direct2D / DirectWrite を覆っていない |
