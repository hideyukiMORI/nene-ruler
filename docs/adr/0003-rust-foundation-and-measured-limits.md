# ADR 0003 — Rust の検査基盤と実測できた限界を固定する

- 状態: 受理
- 日付: 2026-09-27
- Issue: #1
- 影響する規則: ARC-002 / ARC-007 / ARC-013 / RS-002 / RS-003 / RS-011 / RS-017 / RS-019 / QLT-002 / QLT-004 / QLT-007 / QLT-011 / CNF-009

## 文脈

Phase 0 の実測は rustc 1.98.0 / clippy 0.1.98 / cargo-deny 0.20.2 で、`make prove` から再現する。
実出力は [gate-proofs](../quality/gate-proofs.md)。26 件の意図的な違反がすべて期待した診断で落ち、元に戻せば通った。

実測で分かったこと:

- `indexing_slicing` は固定長配列の定数添字（`[1_u8][0]`）を「安全」と判断して発火しない。slice に直して初めて落ちる（P-14 の初版）。
  **設定が在ることは検査が効いていることの証明ではない**（QLT-007）を、本リポジトリでも最初の日に踏んだ
- `cargo deny check licenses` は advisory DB を取りに行かないので、ネットワーク無しで走る。`advisories` はネットワークが要る
- `--locked` は `Cargo.lock` が無い状態でも拒む（P-22）。lock をコミットし忘れる経路が塞がる
- `overflow-checks = true` は release でも `attempt to add with overflow` で終了コード 101 になる（P-21）

クラウドのコンテナの既定の toolchain は 1.94.1 だったが、`rustup toolchain install 1.98.0` が通り、lint 名は xi-tools が 1.98.0 で生存確認した集合をそのまま使える。
現在の stable は 1.98.1。差は patch であり、`rust-toolchain.toml` の版を上げるときは lint の生存を `make prove` で確かめる。

## 決定

**rustc 1.98.0 に固定した Rust の 3 層（コンパイラ・lint・xtask）＋ cargo-deny を `make` の個別 target に結ぶ。**

- 道具の版は `rust-toolchain.toml` だけ。`Makefile` と CI には書かない
- lint の集合は `Cargo.toml` の `[workspace.lints]`（forbid 層と deny 層）と `clippy.toml`（閾値・免除・禁止名・禁止型）。規則 ID を持たない lint を足さない
- 規約検査・Git 規約・証明は `xtask` 1 つに置く。依存 0。`.rs` の `#[cfg(test)]` より後は見ない。検出語は `concat!` で割る
- 依存の許可制は `deny.toml`。`allow` は実測だけ。`--deny license-not-encountered` で先回りの許可を拒む
- 証明は `make prove` が `target/proofs/` に最小 workspace を作り、本物の道具に違反を食わせる。絶対パスは相対に直して記録する
- 中核クレートは作らない（空の将来用モジュールの禁止）。`crates/app` は起動できない理由を返す stub、`xtask` だけが実体
- カバレッジ（QLT-009）は中核が生まれてから `make coverage` に下限を置く。今は planned

## 強制

規則ごとの正本は `docs/QUALITY_GATES.md`。active の規則には `docs/quality/gate-proofs.md` に証明行があり、無ければ CNF-009 が落とす。
CI と ruleset は最初の PR で実行証拠を得るまで planned。

## 結果

- 緑は検査基盤の正常性を示し、物差しが動くことは示さない
- `xtask` は自分自身も検査対象である。除外で逃げると「検査器のコードだけ規約の外にある」穴が開く
- toolchain を上げるときは、削除・改名された lint が無いか `make prove` で確かめる

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 1.94.1（コンテナの既定）に固定する | xi-tools の lint 生存確認と揃わない。`rustup` で 1.98.0 が入ることを確かめた |
| 常に最新の stable を使う | 「手元では通る」を環境差で作る。版は固定し、上げるときに証明を再実行する |
| 証明を手順書だけにする（xi-tools の形） | C++ の先行事例は script にして再現可能にした。Rust でも `xtask prove` で同じ形にできた |
| 未実装の層を空のクレートで作る | 具体的な責務がない将来用モジュールになる（ARC-002 の禁止） |
| カバレッジ 100% と記録する | 中核の分岐も測定器の結線もまだ存在しない |
