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
| [0006](0006-settings-file-v1.md) | 設定は版付きの `key = value` テキストで保存し、読めない版は既定値へ落とさない | 受理 |
| [0007](0007-clipboard-owner-thread-in-adapters.md) | CLIPBOARD は adapters-x11 が専用の接続と 1 本のスレッドで所有する | 受理 |
| [0008](0008-wslg-layered-window-hit-and-shadow.md) | WSLg では描いた画素が当たり判定になる。帯はアルファ 1 で描き、影は窓の種類で消し、Alt+F4 は自分で読む | 受理 |
