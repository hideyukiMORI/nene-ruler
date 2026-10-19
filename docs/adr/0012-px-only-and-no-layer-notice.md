# ADR 0012 — 長さは px だけで綴り、最前面が効かないことは示さない

- 状態: 受理（決定 1・4 の単位の廃止は置換 → [0027](0027-css-pixel-unit.md)。mm / in を置かない理由と決定 2・3 はそのまま）
- 日付: 2026-10-17
- Issue: #30
- 影響する規則: ARC-001 / ARC-004 / ARC-009。仕様 FR-004 / FR-005 / FR-007 / FR-008 / FR-010 / FR-013 / FR-014。ADR 0008 の決定 5 と ADR 0006 の `unit` の鍵を置き換える

## 文脈

2026-10-17、施主が WSLg で 2x の定規を試し、次の 2 点を推しどおりに決めた。

**mm / in。** 換算の元は RandR のミリ寸法で、これはモニタの自己申告（EDID）である。

- 施主の主画面 rdp-0 は RandR で **0 mm**。主画面では mm / in を一度も出せない
- 寸法がある画面でも丸めてある。27 型 4K の rdp-4 は 600 × 340 mm と申告するが、実寸は約 597 × 336 mm（0.5% ずれ、100 mm で 0.5 mm）
- 読むのは主出力の値だけで、定規が乗っている画面の値ではない
- さらに `ScreenMetrics::new` は 0 mm を失敗にしていたので、主画面が 0 mm だと**画素数の上限（FR-005）まで捨てて**いた

**最前面の通知。** ADR 0008 の決定 5 は、WM が `_NET_WM_STATE_ABOVE` を挙げていなければ `always on top: not supported here` を枠とモーダルに出すとした。
施主は最終的に Windows 版と Linux 版を GitHub で配り、Microsoft Store に出す予定で、Windows 版では最前面が効く。WSLg は開発環境であり、その制約を利用者に示す意味が無い。
なお WSLg でも、Windows 側から msrdc の窓に `WS_EX_TOPMOST` を付けると最前面になることを実測した（別の窓を前面にしても残った）が、Windows 版で不要になるので作らない。

## 決定

**長さは常に物理画素で `400 px` と綴り、単位の選択を持たない。最前面が効かないことは枠にもモーダルにも示さない。**

1. `Unit`・`LengthText::in_millimetres` / `in_inches`・`RulerSettings::unit`・`SettingsControl::Unit`・`RulerIntent::ChooseUnit`・モーダルの単位の段を消す。モーダルは 56 px 低くなる（320 × 280）
2. `FrameNotice::NoScreenMetrics` / `NoAlwaysOnTop`・モーダルの `no screen size` / `not supported here`・`Layering`・`SettingsNotices` を消す。枠に残る通知は不透明への降格（FR-017）だけ
3. `ScreenMetrics` は画素数と横のミリ寸法だけを持つ。ミリ寸法が 0 でも作れて、画素数は上限に使い、dpi（ADR 0011）は `None` になる
4. 設定ファイル v1 は `unit` の鍵を書かない。読むときは `px` / `mm` / `in` のどれかなら受け付けて捨て、それ以外の値は今まで通り `Malformed`。未公開なので版は上げない

## 強制

- **active**: core の単体テスト（0 mm でも画素数を持つ・dpi は `None`）、application の単体テスト（0 mm の画面でも上限は画素数・通知は不透明だけ・コピーは `240 px`）、adapters-x11 の単体テスト（`unit = mm` を含む v1 が読めて、書き戻すと `unit` の行が無い・`unit = cm` は `Malformed`）、ui-x11 の画素テスト（モーダルの「Always on top」の右に赤いインクが無い・320 × 280）
- **planned**: WSLg での目視（QLT-013）。記録は gate-proofs 第 21 節

## 結果

得られるもの:

- 施主の主画面のように寸法の無い画面でも、伸縮の上限が画面の画素数で効く
- モーダルが短くなり、選ぶものが減る
- 表示とコピーの文字列が常に `N px` で、画面ごとの換算のぶれが無い

失うもの・残る穴:

- mm / in で測りたい人は自分で換算する。戻すなら、乗っている画面の寸法を読むこと、寸法が無い画面の扱いを決めてから
- WSLg（と `_NET_WM_STATE_ABOVE` を持たない WM）では、最前面の設定が黙って効かない
- 設定ファイルの `unit` の読み捨ては、施主の手元のファイルのための互換。初めて公開する前に消してよい

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| mm / in を残し、定規が乗っている画面の寸法で換算する | 施主の主画面は 0 mm で、寸法がある画面も EDID の丸めでずれる。正確でない単位を出すより出さない |
| WSLg で Windows 側から `WS_EX_TOPMOST` を付ける（`powershell.exe` を呼ぶ） | 効くことは実測したが、題名で窓を探す・1 秒ほど遅れる・WSL の interop に頼る。Windows 版では不要になる捨て実装 |
| 設定ファイルの版を 2 に上げる | 未公開で、読む人は施主の手元のファイル 1 つだけ。版を上げると v1 の読み手と移行の経路が要る |
| `unit` の行を `Malformed` にする | 施主の今のファイル（`unit = mm`）が読めなくなり、設定が既定値で始まる |
