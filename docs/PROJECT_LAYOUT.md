# モジュール構成と依存規則 — NeNe Ruler

> Status: normative（規範）/ 2026-09-27 初版
> workspace: `nene-ruler`（crate 名の接頭辞は `nene-ruler-`）

モジュールグラフはアーキテクチャの一部である。**モジュールの命名規約だけでは依存の境界にならない。**
Rust では層を**クレート**で表し、各クレートの `[dependencies]` が import 可能なものの全リストになる（ARC-002）。
承認された向きの表は `xtask` の CNF-013 が持ち、マニフェストと照合する。

---

## 1. 承認されたモジュール

```text
crates/app（nene-ruler・bin）
    合成ルート。ポートに実装を結び、UI を起動する。端末出力・引数・終了コードを持つ唯一の場所

crates/ui-x11（nene-ruler-ui-x11）
    画面。x11rb で窓を持ち、cairo の ImageSurface に描いて転送する。状態を描き、意図を発行する

crates/adapters-x11（nene-ruler-adapters-x11）
    物理寸法（RandR）・クリップボード（選択）・設定の保存・現在時刻を実装する。
    現在時刻・乱数・環境・ファイルを読める唯一の製品モジュール

crates/application（nene-ruler-application）
    ポートの宣言、表示値の生成、状態の所有者、結果型。#![no_std]

crates/core（nene-ruler-core）
    値・不変条件・閉じた選択肢・整形・目盛の計算。依存 0・#![no_std]

xtask
    規約検査・Git 規約・ゲート発火の証明。成果物ではない（publish = false）。依存 0
```

🔴 **空の将来用モジュールは作らない。** クレートは具体的な責務とソースがある時だけ作る。
Issue #1 の時点で存在するのは `crates/app`（起動できない理由を返す stub）と `xtask` だけである。

---

## 2. 許可された依存グラフ

```text
core
    -> なし（alloc のみ）

application
    -> core

adapters-x11
    -> application, core（外部: x11rb・XDG のパス解決に要る最小の crate は ADR で）

ui-x11
    -> application, core（外部: x11rb, cairo-rs）

app
    -> すべて（明示的な合成のためだけに）
```

ここに無い依存はすべて禁止である。とくに:

- core -> 外部 crate: **禁止**（CNF-013 が拒む）
- core -> application: 禁止
- application -> アダプタ / UI: 禁止
- ui-x11 -> adapters-x11: 禁止（X の接続を共有したくなったら、接続を application のポートに畳んで ADR）
- 何か -> 合成ルート: 禁止

---

## 3. 各モジュールの責務

### `core`

意味の正本を持つ。値型・閉じた選択肢・不変の設定値・拒否理由と結果型・目盛の位置と表示文字列の計算。
UI 状態・直列化・永続化・現在時刻を持たない。`#![no_std]`。

### `application`

振る舞いの調整を持つ。ポート（`trait`）・表示フレームの生成・状態の所有者・結果型。X11・永続化・ファイル・時刻を知らない。`#![no_std]`。

### `adapters-x11`

RandR の物理寸法、クリップボード（`CLIPBOARD` 選択の所有）、設定の永続化、現在時刻のポートを実装する。
保存形式の版と移行（ARC-009）、直列化の型をここに閉じる。
**決定性の禁止を適用しない唯一の製品モジュール**（ARC-007）。CNF-012 の除外に明示する。

### `ui-x11`

application が作った値を描き、操作を意図として渡す（ARC-011）。可変性の隔離区画（ARC-005）。

### `app`

依存を結ぶ。端末出力・`std::env::args`・終了コードを扱ってよい唯一の場所（ARC-006）。

---

## 4. ソースの置き場

| 種類 | 置き場 |
| --- | --- |
| production | `crates/core` / `crates/application` / `crates/adapters-x11` / `crates/ui-x11` / `crates/app` |
| 単体テスト | 各クレートの `#[cfg(test)]`（ファイル末尾）と `tests/`。OS 資源と表示は使わない |
| 検査器と証明 | `xtask/`（`cargo test -p xtask` が正例・反例） |
| 検査設定 | `Cargo.toml` の `[workspace.lints]` / `clippy.toml` / `deny.toml` / `rust-toolchain.toml` / `Makefile` |
| 生成物 | `target/`（cargo・`make prove` の fixture と `results.json`）/ `out/`（画面の実測。未使用） |

表示を伴う確認（Xvfb の上で窓を出し PNG を撮る）は環境検証であって単体テストではない（QLT-013）。
その道具は最初の縦切りの Issue で置く。
