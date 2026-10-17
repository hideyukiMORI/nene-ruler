# ゲート発火の証明 — NeNe Ruler

> 記録: 2026-09-27 / Issue #1、2026-10-15 / Issue #2（第 8 節）、2026-10-16 / Issue #4（第 9 節）。以下は実行した結果のみ。
> 関連: QLT-007 / QLT-011 / QLT-013 / ADR 0003

## 1. 環境と範囲

クラウドの Linux コンテナ（Ubuntu 24.04・x86_64）。rustc 1.98.0 / clippy 0.1.98 / rustfmt 1.9.0 / cargo-deny 0.20.2 / cargo-llvm-cov 0.9.1。
これらは実測時の版であり、今後使う版の正本は `rust-toolchain.toml`（道具は `make doctor`）。
Issue #1 の時点では製品コードは 0。表示を伴う確認は無い。

## 2. active の規則と、その証明

CNF-009 は active の規則ごとに `| ID |` で始まる行がここにあることを要求する。

| 規則 | 最小の違反 | 検証経路 | 実測 |
| --- | --- | --- | --- |
| RS-001 | 非公開 newtype をモジュール外で直接構築 | `make prove` P-15 | E0603 |
| RS-002 | `match` に `_`・variant の漏れ | P-04 / P-05 | `wildcard_enum_match_arm` / E0004 |
| RS-003 | 未初期化・部分構築・`Default` | P-16 / P-17 ＋ CNF-001 | E0381 / E0063 / xtask |
| RS-004 | `unwrap` | P-13 | `unwrap_used`（forbid） |
| RS-005 | `unwrap`・slice の添字 | P-13 / P-14 | `unwrap_used` / `indexing_slicing` |
| RS-006 | `dyn Any` | CNF-002 の反例テスト | xtask |
| RS-007 | `as` で数値変換 | P-06 | `as_conversions` |
| RS-008 | 未文書の公開関数 | P-09 | `missing_docs` |
| RS-009 | `OnceLock` | P-18 ＋ CNF-002 | `unsafe_code` / xtask |
| RS-010 | `unsafe` ブロック・`build.rs` | P-18 ＋ CNF-002 / CNF-008 | `unsafe_code` / xtask |
| RS-011 | 認知的複雑度 13・62 行の関数 | P-11 / P-12 | `cognitive_complexity` / `too_many_lines` |
| RS-012 | `mod.rs`・2 つ目の主要宣言 | P-10 ＋ CNF-003 | `mod_module_files` / xtask |
| RS-013 | 束縛名 `data`・型名 `RulerManager` | P-26 ＋ CNF-004 | `disallowed-names` / xtask |
| RS-014 | `println!` | P-19 | `print_stdout` |
| RS-015 | core の `std::time`・ui の `std::env` | CNF-012 の反例テスト | xtask |
| RS-016 | `HashMap` | P-07 | `disallowed-types` |
| RS-017 | release で `255_u8 + 1_u8` | P-21 | `attempt to add with overflow`・終了 101 |
| RS-018 | 型注釈の無いリテラル | P-08 | `default_numeric_fallback` |
| RS-019 | forbid を `#[expect]`・腐った `#[expect]`・`#[allow]` | P-01 / P-02 / P-03 ＋ CNF-006 | E0453 / unfulfilled / `allow_attributes` |
| ARC-002 | クレートの循環・ui → adapters | P-23 ＋ CNF-013 | `cyclic package dependency` / xtask（実ファイル: 第 8 節） |
| ARC-003 | core の `#![no_std]` を外す・ui-x11 が adapters-x11 に依存 | CNF-007 ＋ CNF-013（実ファイル・第 8 節） | xtask が CNF-007 / CNF-013 を報告 |
| ARC-007 | adapters の外で `std::time` | CNF-012 の反例テスト | xtask |
| ARC-013 | 許可外のライセンス・先回りの許可・lock 無し | P-24 / P-25 / P-22 ＋ CNF-013 | cargo-deny / `--locked` / xtask |
| QLT-002 | lint・整形・言語の違反 | P-01〜P-20 | すべて非 0。復帰は 0 |
| QLT-003 | `#[allow]`・`= "allow"`・`continue-on-error` | P-03 ＋ CNF-011 | `allow_attributes`（forbid） / xtask |
| QLT-004 | 一行に詰めた `main` | P-20 ＋ CNF-011 | `Diff in`・終了 1 |
| QLT-011 | `Cargo.lock` を消して `--locked` | P-22 | 終了 101 |
| CNF-001 | `Default` の derive・impl・穴埋め | xtask の正例・反例 | 反例は CNF-001、正例は 0 |
| CNF-002 | `OnceLock`・proc-macro・`XlibSurface` / `XCBSurface` | 同上 | 反例は CNF-002 |
| CNF-003 | 2 つ目の主要宣言 | 同上 | 反例は CNF-003。ローカル型は通る |
| CNF-004 | 禁止語尾・禁止モジュール名 | 同上 | 反例は CNF-004。`TickProcessor` は通る |
| CNF-006 | 無い規則 ID・ID の無い reason・複数行 | 同上 | 反例は CNF-006 |
| CNF-007 | `-core` / `-application` に `#![no_std]` が無い | 同上 | 反例は CNF-007。bin は対象外 |
| CNF-008 | `build.rs`・マニフェストの `build = ` | 同上 | 反例は CNF-008。`rebuild.rs` は通る |
| CNF-009 | 状態の不一致・行の欠落・証明の欠落・未定義 ID・雛形値・切れたリンク | 同上 ＋ 実ファイル | 反例は CNF-009。本文書が無い状態で 45 件（active の行）を報告した |
| CNF-010 | Issue 番号の無い `FIXME` | 同上 ＋ 実ファイル | 反例は CNF-010。検査器自身の doc コメントの `TODO` を 1 件検出し、文言を直した |
| CNF-011 | `rustfmt.toml`・`= "allow"`・`continue-on-error` | 同上 | 反例は CNF-011 |
| CNF-012 | core の `std::time`・app の `std::env::var` | 同上 | 反例は CNF-012。adapters と `std::env::args` は通る |
| CNF-013 | ui → adapters・core の外部 crate・xtask の依存 | 同上 ＋ 実ファイル | 反例は CNF-013。`Range` / `Case` の 2 つ目の宣言を CNF-003 が実ファイルで検出し、モジュールを分けた |

## 3. `make prove` の実出力（26 件）

`cargo run -p xtask -- prove` が `target/proofs/` に最小 workspace を作り、本物の道具に違反を食わせた。
各件は「落ちること」の後に元へ戻して「通ること」も確かめている（復帰の終了コードは 0）。

| # | 規則 | 命令 | 終了 | 実出力 |
| --- | --- | --- | --- | --- |
| P-01 | RS-019 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0453]: expect(clippy::unwrap_used) incompatible with previous forbid` |
| P-02 | RS-019 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: this lint expectation is unfulfilled` |
| P-03 | QLT-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: #[allow] attribute found` |
| P-04 | RS-002 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: wildcard matches only a single variant and will also match any future added variants` |
| P-05 | RS-002 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0004]: non-exhaustive patterns: 'Mode::Mm' not covered` |
| P-06 | RS-007 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: using a potentially dangerous silent 'as' conversion` |
| P-07 | RS-016 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of a disallowed type 'std::collections::HashMap'` |
| P-08 | RS-018 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: default numeric fallback might occur` |
| P-09 | RS-008 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: missing documentation for a function` |
| P-10 | RS-012 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: 'mod.rs' files are not allowed, found 'probe/src/probe10/mod.rs'` |
| P-11 | RS-011 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: the function has a cognitive complexity of (13/10)` |
| P-12 | RS-011 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: this function has too many lines (62/60)` |
| P-13 | RS-005 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: used 'unwrap()' on 'Some' value` |
| P-14 | RS-005 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: indexing may panic` |
| P-15 | RS-001 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0603]: tuple struct constructor 'Id' is private` |
| P-16 | RS-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0381]: used binding 'z' isn't initialized` |
| P-17 | RS-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0063]: missing field 'b' in initializer of 'Config'` |
| P-18 | RS-010 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: usage of an 'unsafe' block` |
| P-19 | RS-014 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of 'println!'` |
| P-26 | RS-013 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of a disallowed/placeholder name 'data'` |
| P-20 | QLT-004 | `cargo fmt --all --check` | 1 | `Diff in target/proofs/workspace/probe/src/main.rs:1:` |
| P-21 | RS-017 | `cargo run --release -p probe` | 101 | `attempt to add with overflow` |
| P-22 | QLT-011 | `cargo check --locked（Cargo.lock を消した状態）` | 101 | `error: cannot create the lock file target/proofs/workspace/Cargo.lock because --locked was passed to prevent this` |
| P-23 | ARC-002 | `cargo check（a ⇄ b の循環）` | 101 | `error: cyclic package dependency: package 'a v0.0.0 (target/proofs/cycle/a)' depends on itself. Cycle:` |
| P-24 | ARC-013 | `cargo deny --locked check licenses --deny license-not-encountered（allow = []）` | 4 | `│            rejected: license is not explicitly allowed` |
| P-25 | ARC-013 | `cargo deny --locked check licenses --deny license-not-encountered（allow = ["MIT", "Apache-2.0"]）` | 4 | `error[license-not-encountered]: license was not encountered` |

## 4. `xtask` の正例・反例（`cargo test -p xtask`・69 件）

| 検査 | 件数 | テスト名 |
| --- | --- | --- |
| CNF-001 | 6 | cnf001_catches_derive_default, cnf001_catches_impl_default, cnf001_catches_struct_update_syntax, cnf001_ignores_clean_source, cnf001_ignores_markdown, cnf001_ignores_test_module |
| CNF-002 | 4 | cnf002_catches_lazy_global, cnf002_catches_proc_macro_manifest, cnf002_catches_x_surfaces, cnf002_ignores_clean_source |
| CNF-003 | 3 | cnf003_allows_single_declaration, cnf003_catches_second_declaration, cnf003_ignores_nested_declaration |
| CNF-004 | 3 | cnf004_allows_context_dependent_name, cnf004_catches_forbidden_module_name, cnf004_catches_forbidden_suffix |
| CNF-006 | 4 | cnf006_accepts_expect_citing_known_rule, cnf006_catches_reason_without_rule_id, cnf006_catches_unknown_rule_id, cnf006_reads_multiline_expect |
| CNF-007 | 5 | cnf007_allows_core_crate_with_no_std, cnf007_catches_core_crate_without_no_std, cnf007_catches_no_std_after_real_code, cnf007_ignores_a_binary_crate, cnf007_requires_no_std_of_application_too |
| CNF-008 | 3 | cnf008_catches_a_build_script, cnf008_catches_a_manifest_pointing_at_one, cnf008_ignores_a_similar_file_name |
| CNF-010 | 3 | cnf010_accepts_marker_with_issue, cnf010_catches_marker_without_issue, cnf010_ignores_markdown |
| CNF-011 | 4 | cnf011_catches_continue_on_error, cnf011_catches_forbidden_filename, cnf011_catches_lint_set_to_allow, cnf011_ignores_clean_configuration |
| CNF-012 | 6 | cnf012_allows_adapters, cnf012_allows_arguments_in_composition_root, cnf012_catches_env_in_ui, cnf012_catches_other_env_in_composition_root, cnf012_catches_time_in_core, cnf012_ignores_xtask |
| CNF-013 | 5 | cnf013_allows_application_depending_on_core_only, cnf013_allows_the_documented_direction, cnf013_catches_a_dependency_in_xtask, cnf013_catches_external_crate_in_core, cnf013_catches_ui_depending_on_adapters |
| GIT-003 / GIT-002 / GIT-004 | 23 | accepts_the_documented_form, accepts_each_prefix, branch_names, breaking_commit_needs_footer_but_title_does_not, catches_active_without_proof, catches_broken_relative_link, catches_duplicate_definition, catches_missing_matrix_row, catches_missing_state, catches_state_mismatch, catches_template_placeholder, catches_undefined_rule_reference, consistent_documents_pass, extracts_ids_from_prose, first_state_line_wins_over_later_ones, ignores_external_and_anchor_links, pull_request_passes_with_all_fields, pull_request_rejects_draft_missing_close_and_bad_title, rejects_english_description, rejects_missing_issue, rejects_unknown_type_and_bad_scope, rejects_zero_issue_and_long_subject, rejects_wrong_shapes |

## 5. この手順で見つかった実際の欠陥

- 🔴 **`indexing_slicing` は固定長配列の定数添字を「安全」と判断して発火しない。** P-14 の初版 `[1_u8][0]` は clippy が通した。
  slice（`&[u8]`）に直して初めて `indexing may panic` が出た。設定が在ることは検査が効いていることの証明ではない
- 検査器が自分自身を検出した: CNF-010 が `check.rs` の doc コメントの `TODO` を、CNF-003 が `git_conventions.rs` と `prove.rs` の 2 つ目の型を報告した。
  除外で逃げず、文言を直し、型を `git_range.rs` / `proof_case.rs` に分けた
- 証明の実出力に絶対パスが混ざった。実行環境ごとに記録が変わるので、リポジトリからの相対に直して記録するよう `prove` を変えた

## 6. まだ証明していないこと

- CI の `make ci` と commit-msg フックの実行証拠（最初の PR で得る）
- ruleset（`main` の保護・squash-only）。未設定
- QLT-009 の「分岐」カバレッジ。stable では測れない（第 8 節）
- WSLg（施主の実機）での表示・透明・掴む手応え（第 8 節は Xvfb のみ）
- WM が `_NET_WM_MOVERESIZE` に対応する環境での移動と伸縮（Xvfb には WM が無く、自前の経路しか通っていない）

## 7. 再現

```bash
make prove
cargo test -p xtask
make conformance
make coverage
make screenshot
```

## 8. Issue #2 — 最初の縦切り（2026-10-15）

同じコンテナ。rustc 1.98.0 / x11rb 0.14.0 / cairo-rs 0.22.9 / Xvfb / xcompmgr / ImageMagick / xdotool。

### 8.1 実ファイルでの規約検査の発火

| 規則 | やったこと | `make conformance` の実出力 |
| --- | --- | --- |
| CNF-007（ARC-003） | `crates/core/src/lib.rs` の `#![no_std]` を注釈行に置き換えた | `CNF-007: crates/core/src/lib.rs — nene-ruler-core は中核クレートだが、先頭の属性に #![no_std] が無い（ARC-003）` |
| CNF-013（ARC-002 / ARC-003） | `crates/ui-x11/Cargo.toml` に `nene-ruler-adapters-x11` を足した（`--locked` が先に落ちるので `cargo run --offline -p xtask -- conformance` で実行） | `CNF-013: crates/ui-x11/Cargo.toml:20 — nene-ruler-adapters-x11 への依存は crates/ui-x11/ に許されていない（ARC-002 / ARC-013）` |
| CNF-012（ARC-007） | `crates/ui-x11/src/flow.rs` に `std::env::var("HOME")` を足した | `CNF-012: crates/ui-x11/src/flow.rs:12 — std::env は adapters-x11 の外で読まない（ARC-007）` |
| CNF-003 | 書いている途中で `ruler_window.rs` に 3 つの型が並んだ | `CNF-003: … Transfer は 2 つ目の主要宣言（1 つ目は Flow）`。`flow.rs` / `transfer.rs` に分けた |

元に戻した後の `make conformance`: `規約違反なし（検査 87 ファイル）`。

### 8.2 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 31・application 4・ui-x11 12・xtask 69、すべて成功 |
| `make conformance` | 違反なし（87 ファイル） |
| `make deny` | advisories / bans / licenses / sources すべて ok。`allow` は `cargo deny list` の実測で MIT・Apache-2.0・Apache-2.0 WITH LLVM-exception |
| `make fmt-check` / `make doc-check` | 差分なし・警告なし |
| `make coverage` | core ＋ application: regions 98.88%・functions 99.23%・lines 99.08%（下限 90% は region） |

**分岐カバレッジは測れなかった。** `cargo llvm-cov --branch` は stable で `-Z coverage-options=branch` を要求して落ちる（空のクレートで実測）。QLT-009 は planned のまま。

### 8.3 Xvfb での表示（環境検証・単体テストではない）

`make screenshot` 相当。Xvfb `:97`（640×400×24）、根の窓を灰色にし、`xcompmgr -n` を起動して `cargo run`、`import -window root` で撮った。

| 条件 | PNG | 見えたもの |
| --- | --- | --- |
| 合成器あり | [2026-10-15-xvfb-composited.png](screenshots/2026-10-15-xvfb-composited.png) | 中身が根の灰色のまま（透明）。四辺に目盛、上辺と左辺に主目盛の数字、上辺の中央に `400 px`、左辺の中央に回転した `240 px`、両角に鉤括弧、右下の外にギア。すべて `#E53935` |
| 合成器なし | [2026-10-15-xvfb-opaque.png](screenshots/2026-10-15-xvfb-opaque.png) | 同じ絵が白い背景の上（インクの補色への降格。ADR 0005 の 5） |
| 操作の後 | [2026-10-15-xvfb-after-drag.png](screenshots/2026-10-15-xvfb-after-drag.png) | 窓が (100, 50) に動き、枠が 440×260 に伸びている |

### 8.4 xdotool での操作（Xvfb には WM が無いので、自前のドラッグの経路）

| 操作 | 命令 | 結果 |
| --- | --- | --- |
| 中身の素通し | `xdotool mousemove 200 120; xdotool getmouselocation --shell` | `WINDOW=543`（根の窓）。枠の窓ではない |
| 線の帯 | `xdotool mousemove 200 5; …` | `WINDOW=6291457`（枠の窓） |
| 線を掴んで移動 | `mousemove 200 5; mousedown 1; mousemove 300 55; mouseup 1` | `Position: 0,0` → `Position: 100,50`、大きさは `424x264` のまま |
| 右下の鉤括弧で伸縮 | `mousemove 500 290; mousedown 1; mousemove 540 310; mouseup 1` | `Geometry: 424x264` → `464x284`（枠は 440×260） |
| Esc | `mousemove 200 5; xdotool key Escape` | `KeyPress` の keycode 9 が届き、プロセスが終了 |

🔴 **見つかった欠陥:** 入力 shape は、`XFixesQueryVersion` を先に送らないと効かなかった（中身の上でも `WINDOW=6291457` が返った）。
`Display::connect` で版 2.0 を交渉するようにして、素通しになった。単体テストでは見えない種類の欠陥で、Xvfb の実測で初めて出た。

## 9. Issue #4 — 設定モーダル（2026-10-16）

同じコンテナ。Issue #2 の枝の上に積んだ。

### 9.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 43・application 9・ui-x11 19・xtask 69、すべて成功 |
| `make conformance` | 違反なし（104 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 98.55%・functions 96.24%・lines 97.75% |

### 9.2 Xvfb での操作（`xdotool`・合成器あり・WM 無し）

| 操作 | 命令 | 結果 | PNG |
| --- | --- | --- | --- |
| ギアを押す | `mousemove 416 256 click 1` | 2 枚目の窓 `NeNe Ruler Settings`（320×336）が出る。題・Ink の色見本 8 つ・`#E53935` の入力欄・Unit・Settings window・Always on top・脚注 | [settings-open](screenshots/2026-10-16-xvfb-settings-open.png) |
| 色見本（紫） | `mousemove 188 84 click 1` | 枠の線・目盛・ギアが紫になり、入力欄が `#8E24AA` に変わる | |
| 単位 mm → in | `mousemove 108 180 click 1` / `172 180` | 物理寸法が無い Xvfb では px のまま（FR-008 の降格）。ボタンの選択は移る | |
| 見え Light | `mousemove 132 236 click 1` | 背景が `#F5F5F5` に変わる。枠の窓は変わらない（FR-018） | [light-in](screenshots/2026-10-16-xvfb-settings-light-in.png) |
| 直接入力 | `key BackSpace`×7、`type "#00aa55"`、`key Return` | 枠が `#00AA55` の緑になり、入力欄は `#00AA55` | [hex](screenshots/2026-10-16-xvfb-settings-hex.png) |
| 無効な入力 | `key BackSpace`、`key Return` | 入力欄の右に `not #RRGGBB`。枠の色は変わらない（FR-014） | [rejected](screenshots/2026-10-16-xvfb-settings-rejected.png) |
| 最前面の切替 | `mousemove 28 284 click 1` | チェックが消える。WM が無いので重なり順の変化は見えない | |
| Esc | `key Escape` | モーダルが消え、枠は残る。再びギアで開き直せる（新しい窓 ID） | |
| 閉じる要求 | `xdotool windowclose`（`WM_DELETE_WINDOW`） | 同上 | |

🔴 **見つかった欠陥:** Esc でモーダルを閉じても窓が残った。`WindowWrapper` の `Drop` が送る `DestroyWindow` が送信バッファに残り、
次の `wait_for_event` では流れない。閉じた直後に `flush` するようにして消えた。単体テストでは見えない種類で、Xvfb の実測で出た。

### 9.3 まだ証明していないこと（Issue #4）

- WSLg（WM あり）での transient の扱い・最前面の切替（`_NET_WM_STATE` のメッセージ）・モーダルの位置
- mm / in の表示。Xvfb の `RandR` はミリ寸法を 0 で返すので px に降格した。単位の換算は `RulerController` の単体テストで見ている
