# ADR 0001 — 厳格さは機械で強制する

- 状態: 受理
- 日付: 2026-09-27
- Issue: #1
- 影響する規則: すべて

## 文脈

本リポジトリの規約は、AYANE 厳格規約ポリシーの 4 回目の適用である。先行は NeNe Loupe（C++23 / MSVC）、
NeNe Folio（C23 / clang-cl）、NeNe Nib（C++23 / clang-cl）、そして Rust では xi-tools が 1 回目。共通する中核は 1 つに要約できる。

> **一つのことを実現する方法を 1 つに固定し、そのことを人の記憶ではなく機械に守らせる。**

規約は、破ったときに何も起きなければ、時間とともに必ず腐る。

### 施主の 3 条件

言語の選定で施主が示した条件は「安定して、速度も出て、規約がかたい」。C++ の先行事例が自らの憲章に記録した比較表では、
網羅性・不変性・`null`・抑制の封じ込めがいずれも「不十分」または「言語では無い」であり、Rust の列だけが「ある」だった。
それを本リポジトリの道具（rustc 1.98.0）で測り直した。実出力は [gate-proofs](../quality/gate-proofs.md)。

```
P-05  match の variant 漏れ        → error[E0004]: non-exhaustive patterns
P-15  非公開 newtype の直接構築     → error[E0603]: tuple struct constructor `Id` is private
P-16  未初期化の束縛（Go のゼロ値）  → error[E0381]: used binding `z` isn't initialized
P-17  構造体の部分構築              → error[E0063]: missing field `b`
P-01  forbid した lint を #[expect] → error[E0453]: incompatible with previous forbid
P-02  発火しない #[expect]          → error: this lint expectation is unfulfilled
P-23  クレートの循環                → error: cyclic package dependency
```

C++ の先行事例が字句検査と waiver 台帳（人手）で補っていたものの大半を、**Rust ではコンパイラが執行している。**
`forbid` 水準の lint は、抑制しようとする行為そのものがコンパイルエラーになる。これに相当するものは C++ にも Kotlin にも Go にも無い。

**そして、いま実装が空である。** 規則を後から被せる場合に必要になる baseline が、今は要らない。違反ゼロから始められる時点は今しかない。

## 決定

**すべての規範規則に「機械強制の状態」を持たせ、その状態を文書の側で機械検査する。**

1. 規則の正本を `docs/ARCHITECTURE_CONSTITUTION.md` / `docs/CODING_RULES.md` / `docs/QUALITY_GATES.md` / `docs/COMMIT_CONVENTIONS.md` に置く。
   すべての規則に ID（ARC / RS / QLT / CNF / GIT）を与え、状態を **active / planned / 不能 / 不採用** で明示する。**planned を active と書かない**
2. 🔴 **抑制を二段構えにする。** `forbid` 層は例外が取れない（E0453）。`deny` 層は `#[expect(lint, reason = "<規則 ID>: …")]` だけで抑制でき、
   発火しなくなれば `unfulfilled_lint_expectations` が落とす。`reason` の規則 ID が規範文書に実在することは CNF-006 が見る
3. `make` の個別 target と `xtask` を判定の唯一の置き場とする。CI は `make ci` を呼ぶだけにし、CI 側にしか無い検査を作らない
4. baseline を持たない。クレート単位の `#![allow]` は `clippy::allow_attributes` が forbid なので書けない。設定側の無力化は CNF-011 が拒む
5. ゲートを弱める変更は ADR を要する（QLT-010）
6. 道具の版は `rust-toolchain.toml` だけが決める。2 か所に書かない
7. **ゲートは意図的な違反で発火することを証明してから active と書く。** `make prove` が本物の道具に違反を食わせ、記録を `docs/quality/gate-proofs.md` に残す。
   active の規則に証明行が無ければ CNF-009 が落とす
8. 規約検査（`xtask`）は依存ゼロで書く。Python を持ち込まない（Rust 1 言語）

## 強制

- **active**: CNF-009（文書整合）・CNF-011（baseline 禁止）・CNF-006（抑制の規則 ID）。実測は `cargo test -p xtask` と `make conformance`
- **不能**: 「`planned` を勝手に `active` と書き換えないこと」そのもの。マトリクスの行と実装の対応は、証明行の有無までしか機械は見ない

## 結果

得られるもの:

- 規約が自分について嘘をつく経路が塞がる
- 新しい貢献者（人でも AI でも）が「何がいま守られているか」を 1 か所で読める
- waiver 台帳という人手の制度を持たずに済む（Rust の forbid / expect が代わりに執行する）

失うもの:

- 文書のメンテナンスコスト。規則を足すたびに 3 か所（本文・マトリクス・実装）が同期を要求される
- 書き味の制約: `as` 禁止・数値リテラルの型注釈・`v[i]` 禁止は、目盛の位置計算が主な仕事のアプリで最初に摩擦になる
- `forbid` は後から緩めにくい。降格には ADR が要る（それが狙いだが、コストである）

**正直に記録しておくこと:**

- 🔴 「不正な状態を表現不能にする」は Rust でも完全ではない。`#[derive(Default)]` でゼロ値を作れる（CNF-001 で塞ぐ）。
  同じモジュールに同居する型はファクトリを迂回できる（CNF-005 は planned）
- 🔴 ARC-007 の字句検査（CNF-012）はマクロ・別名・間接呼び出しを見ない。中核は `#![no_std]` で構造的に塞ぐが、UI は `std` を使うので字句検査だけが残る
- **本 ADR の厳格さは、実装がまだ空である今だからこそ無傷で導入できた。** 緑であることは、規則が良いことの証明ではない。
  実装で規則が邪魔になったとき、緩めるのではなく **ADR で判断を残すこと**が本 ADR の眼目である

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 散文の規約だけを置く | 守られているかを確かめる手段が無く、時間とともに必ず腐る。**本 ADR の出発点そのもの** |
| 既製 lint の既定セットだけで済ませる | 既製ルールは「Rust として危ういこと」を見るが、「NeNe Ruler として守るべきこと」は見られない。規則と lint の対応が付かない |
| `clippy::restriction` / `nursery` の一括有効化 | 互いに矛盾する規則の在庫。群で入れると規則ではなく道具の機嫌に従うことになる |
| baseline を作って既存違反を後回しにする | 新規リポジトリなので既存違反が無い。ここで baseline を許すと、以後の違反も同じ入口から入ってくる |
| 実装が入ってから導入する | 既存違反を凍結する baseline が必要になる。今なら違反ゼロで始められる |
| waiver 台帳（先行事例の `WVR-NNNN`）を移植する | Rust では forbid 層に例外の申請窓口が無く、deny 層の `#[expect]` は理由が必須で不要になれば失敗する。台帳は「機械が見ていない抑制」を人手で追う仕組みであり、その対象がここには無い。forbid を deny に降ろす判断をしたら ADR で導入する |
| Python で検査器を書く（C++ の先行事例と同じ） | Rust のリポジトリに第 2 の言語を持ち込むと道具の版が 2 系統になる。`xtask` に統一する |
| 規約を `CLAUDE.md` にだけ書く | 人間と AI の遵守に依存する。先行事例で実証済みの失敗 |

## 関連

- 先行: NeNe Loupe ADR 0001、NeNe Nib ADR 0001、xi-tools ADR 0001
- ポリシー: 施主の private ワークスペース（本リポジトリには置かない）
