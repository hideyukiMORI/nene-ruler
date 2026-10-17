# ADR 0010 — カバレッジの下限は region で測る

- 状態: 受理
- 日付: 2026-10-17
- Issue: #26
- 影響する規則: QLT-009 / QLT-010。ADR 0003 の結果「カバレッジ（QLT-009）は中核が生まれてから `make coverage` に下限を置く。今は planned」を完了させる

## 文脈

QLT-009 は「中核（core / application）の**分岐**カバレッジ下限を 90%」と書いている。

- 分岐は stable の rustc では測れない。`cargo llvm-cov --branch` は `-Z coverage-options=branch` を要求して落ちる（2026-10-15 実測・gate-proofs 第 8 節）
- `make coverage` は Issue #2 から **region** の 90% 未満で落としている。規則の文言と単位が違うので、QLT-009 は planned のまま置かれていた
- 実測（2026-10-17・`fix/23-modal-layer-notice-fits` の上・rustc 1.98.0 / cargo-llvm-cov 0.9.0）: regions 97.67%・functions 95.31%・lines 96.91%
- 引き継ぎ書（2026-10-17）第 3 節で、単位を region に改める案 (a) を推奨し、施主が推奨どおりで決めた

## 決定

**QLT-009 の下限は、中核（core / application）の region カバレッジ 90% とし、`make coverage` の `--fail-under-regions 90` で active にする。**

## 強制

- **active**: `make coverage`。下限を割ると `cargo llvm-cov` が終了 1 で落ちる（実測値より上の 98 を置いて確かめた・gate-proofs 第 19 節）
- `make ci` には入れない。他の active の規則（lint・conformance）と同じく、差分に応じて個別の target で回す（QLT-012・ADR 0004）

## 結果

得られるもの:

- 規則の文言と、落とす検査の単位が一致する。stable だけで測れる
- region は 1 つの式の中の短絡（`&&` / `||`）や `?` の両側を別に数えるので、行より細かい

失うもの・残る穴:

- 分岐（`if` の真偽の両方を通ったか）は見ない。region は「通ったか」を数え、「両側を通ったか」は数えない
- `cargo llvm-cov` は下限を割っても理由を出さず、終了コードだけが印になる
- 中核の外（adapters-x11 / ui-x11 / app）は下限の外。X の接続と画面を要するので、今まで通り Xvfb と WSLg の実測で見る

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 分岐のまま nightly を要求する | 道具の版は `rust-toolchain.toml` の 1 つだけ（QLT-011）。カバレッジのためだけに nightly を足すと、その版の分だけ再現性が割れる |
| planned のまま置く | stable で測れない規則は永久に active にならない。`make coverage` が既に region で落としているのに、規則だけが別のことを言い続ける |
| 行（lines）で測る | region より粗い。1 行に詰めた短絡の片側が通らなくても 100% になる |
