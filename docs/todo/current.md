# いまのタスク — NeNe Ruler

> GitHub Issue と PR が進行状態の正本。この文書は 2026-10-16 の設定モーダル（Issue #4）の要約。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust 1.98.0 / x11rb 0.14 ＋ cairo-rs 0.22（ADR 0002）。施主決定 2026-09-27 |
| 規範文書・ADR 0001〜0005 | ✅ Issue #1 / #2 |
| ゲート（clippy 二段構え・xtask・cargo-deny・rustfmt・rustdoc）と `make` の個別 target | ✅ Issue #1 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（69 件） | ✅ Issue #1・[gate-proofs](../quality/gate-proofs.md) |
| 仕様の改訂: 透明な 2 次元の枠・四辺に目盛・鉤括弧で伸縮・単色（施主の口述 2026-10-15） | ✅ Issue #2・[ADR 0005](../adr/0005-transparent-frame-with-input-shape.md) |
| 5 クレート（core / application / adapters-x11 / ui-x11 / app） | ✅ Issue #2 |
| 透明の窓・目盛・幅と高さの数字・鉤括弧・ギアの描画。線で移動、鉤括弧で伸縮、Esc で閉じる | ✅ Issue #2・Xvfb で実測（gate-proofs 第 8 節） |
| `make coverage`（region 90%）・`make screenshot` | ✅ Issue #2。分岐は stable で測れず QLT-009 は planned |
| ギアで設定モーダル。色見本 8 色と `#RRGGBB` の直接入力、単位、見え（ダーク・ライト）、最前面、版 | ✅ Issue #4・Xvfb で実測（gate-proofs 第 9 節）。PR #2 の枝の上 |
| CI の必須 check（`make ci`）と commit-msg フック | 配線済み。最初の PR で実行証拠を得るまで planned |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |
| WSLg（施主の実機）での目視 | 未。透明に出ること・掴めること・`_NET_WM_MOVERESIZE` が効くことを見る |

## 次に行うこと

PR #3（Issue #2）と Issue #4 の PR を施主が WSLg で目視する。その後の焦点 Issue（それぞれ 1 つずつ）:

- 設定の保存（FR-012 / ARC-009）。`RulerSettings` を版 1 の形式で adapters-x11 が読み書きする
- 数字のクリックでクリップボードへコピー（FR-007）。選択の所有に待ち受けが要るので先に ADR
- モーダルの見え「システムに従う」。検出の方法（XSETTINGS か GTK の settings.ini か）を施主と決める
- 矢印で 1 px / 10 px の移動（FR-015）、合成器が無いときの表示（FR-014）、HiDPI の倍率（FR-016）

## 残る穴

- QLT-009 の単位（分岐か region か）を ADR で決める
- CNF-012 はマクロ・別名・間接呼び出しを見ない。ui-x11 は `std` を使うので字句検査だけが残る
- CNF-005（不変条件を持つ型の単独モジュール）は判定方法が未定。core の型は 1 ファイル 1 型になっている
- `_NET_WM_MOVERESIZE` の経路は WM の無い Xvfb では通らない。WSLg で確かめる
- 当たり判定の矩形と描いたものの一致はレビュー事項（ADR 0005）
