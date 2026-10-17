# アーキテクチャ決定記録（ADR）

トレードオフのある判断を、**理由ごと**残す場所。正本ドキュメントが「いま何が規範か」を書くのに対し、
ADR は「なぜそう決めたか」「何を却下したか」を書く。

## 書き方

- 連番 4 桁 ＋ kebab の題名: `NNNN-short-kebab-title.md`
- `0000-template.md` を複製して書く
- 状態は `提案` / `受理` / `却下` / `置換（→ NNNN）`
- **却下した選択肢を必ず書く。** 再提案は、その理由への反論から始めること
- 受理された ADR を後から書き換えない。変えるときは新しい ADR で置き換える
- 文脈は可能な限り実データ（コード・履歴・計測・コンパイラの出力）で裏を取る

## 一覧

| ADR | 題名 | 状態 |
| --- | --- | --- |
| [0001](0001-strictness-is-mechanically-enforced.md) | 厳格さは機械で強制する | 受理 |
| [0002](0002-x11rb-and-cairo-without-a-toolkit.md) | X11 は x11rb、描画は cairo の ImageSurface。UI ツールキットを使わない | 受理 |
| [0003](0003-rust-foundation-and-measured-limits.md) | Rust の検査基盤と実測できた限界を固定する | 受理 |
| [0004](0004-diff-scoped-verification-and-result-reuse.md) | 差分に必要な検証と成功結果の再利用 | 受理 |
| [0005](0005-transparent-frame-with-input-shape.md) | 枠は透明な 2 次元の窓。描いたものだけが見え、描いたものだけが触れる | 受理 |
| [0006](0006-settings-file-v1.md) | 設定は版付きの `key = value` テキストで保存し、読めない版は既定値へ落とさない | 受理（`unit` の鍵は置換 → 0012） |
| [0007](0007-clipboard-owner-thread-in-adapters.md) | CLIPBOARD は adapters-x11 が専用の接続と 1 本のスレッドで所有する | 受理 |
| [0008](0008-wslg-layered-window-hit-and-shadow.md) | WSLg では描いた画素が当たり判定になる。帯はアルファ 1 で描き、影は窓の種類で消し、Alt+F4 は自分で読む | 受理（文脈 3 と決定 4 は置換 → 0009、決定 5 は置換 → 0012） |
| [0009](0009-alt-f4-on-the-f4-release.md) | Alt+F4 は、この窓が Alt の押下を受けていれば F4 の離しでも閉じる | 受理 |
| [0010](0010-coverage-floor-in-regions.md) | カバレッジの下限は region で測る | 受理 |
| [0011](0011-integer-ui-scale.md) | UI の倍率は 1x / 2x の整数で、飾りにだけ掛ける | 受理（主目盛の数字の大きさは置換 → 0013、モーダルの倍率は置換 → 0014） |
| [0012](0012-px-only-and-no-layer-notice.md) | 長さは px だけで綴り、最前面が効かないことは示さない | 受理 |
| [0013](0013-tick-numbers-keep-their-size.md) | 主目盛の数字は倍率に関わらず 9 px で描く | 受理 |
| [0014](0014-frameless-single-scale-modal.md) | モーダルは 1x・飾りなしで、閉じるボタンと移動を自前で持つ | 受理 |
