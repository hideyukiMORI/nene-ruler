# いまのタスク — NeNe Ruler

> GitHub Issue と PR が進行状態の正本。この文書は 2026-10-17 の WSLg の実測への対応（Issue #14 / #15）の要約。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust 1.98.0 / x11rb 0.14 ＋ cairo-rs 0.22（ADR 0002）。施主決定 2026-09-27 |
| 規範文書・ADR 0001〜0010 | ✅ Issue #1 / #2 / #6 / #8 / #14 / #20 / #26 |
| ゲート（clippy 二段構え・xtask・cargo-deny・rustfmt・rustdoc）と `make` の個別 target | ✅ Issue #1 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（69 件） | ✅ Issue #1・[gate-proofs](../quality/gate-proofs.md) |
| 仕様の改訂: 透明な 2 次元の枠・四辺に目盛・鉤括弧で伸縮・単色（施主の口述 2026-10-15） | ✅ Issue #2・[ADR 0005](../adr/0005-transparent-frame-with-input-shape.md) |
| 5 クレート（core / application / adapters-x11 / ui-x11 / app） | ✅ Issue #2 |
| 透明の窓・目盛・幅と高さの数字・鉤括弧・ギアの描画。線で移動、鉤括弧で伸縮、Esc で閉じる | ✅ Issue #2・Xvfb で実測（gate-proofs 第 8 節） |
| `make coverage`（region 90%）・`make screenshot` | ✅ Issue #2。QLT-009 は region に改めて active（Issue #26・ADR 0010） |
| ギアで設定モーダル。色見本 8 色と `#RRGGBB` の直接入力、単位、見え（ダーク・ライト）、最前面、版 | ✅ Issue #4・Xvfb で実測（gate-proofs 第 9 節）。PR #3 の枝の上 |
| 設定の保存 `~/.config/nene-ruler/settings.v1.txt`。読めない版・壊れたファイルはモーダルに通知 | ✅ Issue #6・ADR 0006・Xvfb で再起動を実測（gate-proofs 第 10 節）。PR #5 の枝の上 |
| 数字のクリックで CLIPBOARD へコピー。成功は反転、失敗は取り消し線 | ✅ Issue #8・ADR 0007・Xvfb で `xclip -o` を実測（gate-proofs 第 11 節）。PR #7 の枝の上 |
| 矢印で 1 px、Shift+矢印で 10 px の移動 | ✅ Issue #10・Xvfb で実測（gate-proofs 第 12 節）。PR #9 の枝の上 |
| 不透明への降格と物理寸法の欠如を枠に、設定の失敗と物理寸法の欠如をモーダルに示す（FR-014） | ✅ Issue #12・Xvfb で実測（gate-proofs 第 13 節）。PR #11 の枝の上 |
| WSLg（施主の実機）での目視 | ✅ 2026-10-17・[wslg-2026-10-17](../quality/wslg-2026-10-17.md)・gate-proofs 第 14 節。13 件が成り立ち、5 件が崩れた |
| WSLg への適合: 当たり判定の帯をアルファ 1 で描く・影を消す・Alt+F4 を自分で読む・最前面ができないことを示す | ✅ Issue #14・ADR 0008。施主の再目視待ち。PR #13 の枝の上 |
| 縦書きの高さの数字と通知を、左辺の主目盛の数字の右へ寄せる | ✅ Issue #15。PR #16 の枝の上 |
| CI の必須 check（`make ci`）と commit-msg フック | 配線済み。最初の PR で実行証拠を得るまで planned |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

施主が WSLg で Issue #14 の 4 件と Issue #15 の重なりを再目視し、PR #3 → #5 → #7 → #9 → #11 → #13 → #16 → Issue #15 の PR の順に squash merge する。その後の焦点 Issue（それぞれ 1 つずつ）:

- HiDPI の倍率（FR-016）。`Xft.dpi` か `RandR` の DPI から倍率を決め、1x / 2x の整数に丸めて core の初期値に掛ける（施主決定 2026-10-17）

「システムに従う」は置かないと決めた（Issue #26）。

## 残る穴

- CNF-012 はマクロ・別名・間接呼び出しを見ない。ui-x11 は `std` を使うので字句検査だけが残る
- CNF-005（不変条件を持つ型の単独モジュール）は判定方法が未定。core の型は 1 ファイル 1 型になっている
- 最前面は WSLg では実現できない（`_NET_WM_STATE_ABOVE` が無い）。通知で示すだけ
- WSLg で 1 度だけ、2 回目の起動で Windows 側に絵が出ず無言で終わった。再現待ち（gate-proofs 第 14.3 節）
- 当たり判定の矩形と描いたものの一致はレビュー事項（ADR 0005）
