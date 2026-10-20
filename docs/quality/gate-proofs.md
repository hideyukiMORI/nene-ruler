# ゲート発火の証明 — NeNe Ruler

> 記録: 2026-09-27 / Issue #1、2026-10-15 / Issue #2（第 8 節）、2026-10-16 / Issue #4（第 9 節）/ Issue #6（第 10 節）/ Issue #8（第 11 節）/ Issue #10（第 12 節）/ Issue #12（第 13 節）/ WSLg の実機（第 14 節・2026-10-17）/ Issue #14（第 15 節）/ Issue #15（第 16 節）/ Issue #20（第 17 節）/ Issue #23（第 18 節）/ Issue #26（第 19 節）/ Issue #28（第 20 節）/ Issue #30（第 21 節）/ Issue #32（第 22 節）/ Issue #34（第 23 節）/ Issue #36（第 24 節）/ Issue #38（第 25 節）/ Issue #40（第 26 節）/ Issue #42（第 27 節）/ Issue #44（第 28 節）/ Issue #46（第 29 節）/ Issue #48（第 30 節）/ Issue #50（第 31 節）/ Issue #52（第 32 節）/ Issue #54（第 33 節）/ Issue #56（第 34 節）/ Issue #58（第 35 節）/ Issue #60（第 36 節）/ Issue #62（第 37 節）。以下は実行した結果のみ。
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
| QLT-009 | 実測値（region 97.67%）より上の下限 98 を置く | `cargo llvm-cov … --fail-under-regions 98`（第 19 節） | 終了 1。下限 90 の `make coverage` は終了 0 |
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
| CNF-014 | Windows のクレートの lint の表の 1 項目を緩める・消す | xtask の正例・反例（第 36 節） | 反例は CNF-014 を 2 件（rust と clippy の表）。正例（unsafe_code だけ deny）と他のクレートは通る |

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
| CNF-008 | 4 | cnf008_allows_only_the_icon_build_script, cnf008_catches_a_build_script, cnf008_catches_a_manifest_pointing_at_one, cnf008_ignores_a_similar_file_name |
| CNF-010 | 3 | cnf010_accepts_marker_with_issue, cnf010_catches_marker_without_issue, cnf010_ignores_markdown |
| CNF-011 | 4 | cnf011_catches_continue_on_error, cnf011_catches_forbidden_filename, cnf011_catches_lint_set_to_allow, cnf011_ignores_clean_configuration |
| CNF-012 | 7 | cnf012_allows_adapters, cnf012_allows_arguments_in_composition_root, cnf012_allows_files_and_commands_only_in_the_icon_build_script, cnf012_catches_env_in_ui, cnf012_catches_other_env_in_composition_root, cnf012_catches_time_in_core, cnf012_ignores_xtask |
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
- 分岐カバレッジ。stable では測れない（第 8 節）。QLT-009 は region に改めた（ADR 0010・第 19 節）
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

## 10. Issue #6 — 設定の保存（2026-10-16）

同じコンテナ。Issue #4 の枝の上に積んだ。`HOME` を一時ディレクトリに向け、`XDG_CONFIG_HOME` を空にして実行した。

### 10.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 43・application 12・adapters-x11 8・ui-x11 20・xtask 69、すべて成功 |
| `make conformance` | 違反なし（110 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 98.13%・functions 95.61%・lines 97.14% |

### 10.2 Xvfb での再起動

| 手順 | 結果 | PNG |
| --- | --- | --- |
| 起動 → ギア → 青の色見本 → `mm` → Esc → Esc | `~/.config/nene-ruler/settings.v1.txt` が生まれ、中身は `version = 1` / `ink = #1E88E5` / `modal_theme = dark` / `unit = mm` / `layer = always_on_top` の 5 行。一時ファイルは残らない | |
| 再起動 | 枠が青で出る。単位は mm で保存されているが、Xvfb の `RandR` はミリ寸法が 0 なので表示は px に降格（FR-008） | [restored](screenshots/2026-10-16-xvfb-settings-restored.png) |
| ファイルを `version = 9` の 1 行にして再起動 → ギア | 既定の赤で出る。モーダルの脚注の上に `settings: unknown file version; using defaults`（FR-014） | [unknown-version](screenshots/2026-10-16-xvfb-settings-unknown-version.png) |
| 橙の色見本を押す | 保存が成功し、通知が消える。ファイルは `ink = #FB8C00` / `unit = px` で書き直される | |

### 10.3 まだ証明していないこと（Issue #6）

- 書けない失敗（権限・ディスク）の通知。単体テスト（ファイルの代わりにディレクトリ）でしか見ていない
- WSLg（`XDG_CONFIG_HOME` が設定された環境）での置き場

## 11. Issue #8 — クリップボード（2026-10-16）

同じコンテナ。Issue #6 の枝の上に積んだ。貼り付け側は `xclip`（`apt-get install xclip`）。

### 11.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 43・application 14・adapters-x11 11・ui-x11 22・xtask 69、すべて成功 |
| `make conformance` | 違反なし（118 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 98.07%・functions 95.49%・lines 97.06% |

### 11.2 Xvfb での貼り付け（合成器あり・WM 無し）

| 手順 | 結果 | PNG |
| --- | --- | --- |
| `400 px` を押す（`mousemove 204 36 click 1`） | 数字が反転（インクの箱に透明の文字） | [copied](screenshots/2026-10-16-xvfb-copied.png) |
| `xclip -o -selection clipboard` | `400 px` | |
| `xclip -o -selection clipboard -t TARGETS` | `TARGETS UTF8_STRING STRING` | |
| ポインタを中身へ（`mousemove 300 120`） | 印が消える。貼り付けは引き続き `400 px` | |
| 単位を in にして `240 px` を押す | `xclip -o` は `240 px`（Xvfb はミリ寸法が 0 で px に降格） | |

🔴 **見つかった欠陥:** 印が消えなかった。中身は入力 shape の外なので、数字から中身へ動いたときに届くのは `MotionNotify` ではなく
`LeaveNotify` である。`LEAVE_WINDOW` を選び、`LeaveNotify` でも印を消すようにして消えた。入力 shape（ADR 0005）の帰結で、Xvfb の実測で出た。

### 11.3 まだ証明していないこと（Issue #8）

- クリップボードマネージャ（WSLg の Windows 側の同期を含む）との相性。所有を取った直後に相手が `TARGETS` と `UTF8_STRING` を要求する流れは `xclip` で見ただけ
- 失敗の取り消し線は画素の単体テストでだけ見ている（Xvfb では失敗させられなかった）

## 12. Issue #10 — 矢印での移動（2026-10-16）

同じコンテナ。Issue #8 の枝の上に積んだ。

### 12.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 44・application 14・adapters-x11 11・ui-x11 22・xtask 69、すべて成功 |
| `make conformance` | 違反なし（120 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 98.10%・functions 95.56%・lines 97.11% |

### 12.2 Xvfb での鍵（WM 無し・焦点は `PointerRoot`）

窓を (100, 100) に置き、ポインタを上辺の帯（根の (300, 105)）に置いて `xdotool key` を送った。

| 鍵 | `getwindowgeometry` の Position |
| --- | --- |
| 前 | `100,100` |
| `Right` | `101,100` |
| `shift+Right` | `111,100` |
| `Down` → `shift+Up` | `111,91` |
| `Left` × 2（帯の上で） | `98,100`（別の起動で 100,100 から） |

🔴 **記録しておくこと:** 窓が動いてポインタが中身（入力 shape の外）に乗ると、WM 無しの Xvfb では鍵が根へ行き、動かなくなる。
これは焦点が `PointerRoot` の環境の性質で、WM があればクリックで焦点が枠の窓に付く。WSLg での焦点の挙動は施主の目視（Issue #10 のやらないこと）。

## 13. Issue #12 — 通知（2026-10-16）

同じコンテナ。Issue #10 の枝の上に積んだ。

### 13.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 45・application 16・adapters-x11 11・ui-x11 24・xtask 69、すべて成功 |
| `make conformance` | 違反なし（123 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 97.92%・functions 95.08%・lines 96.80% |

### 13.2 Xvfb での表示

| 条件 | 結果 | PNG |
| --- | --- | --- |
| 合成器なしで起動 | 白い背景の枠の左下、下辺の目盛の上に `no compositor: drawn opaque` | [notice-opaque](screenshots/2026-10-16-xvfb-notice-opaque.png) |
| 同じ状態でギア → `mm` | 単位のボタンの右に `no screen size`（Xvfb の `RandR` はミリ寸法 0） | [notice-modal](screenshots/2026-10-16-xvfb-notice-modal.png) |
| 合成器ありで再起動（保存された `mm` を読み戻す） | 透明な枠の左下に `no screen size: px only` の 1 行。不透明の行は出ない | [notice-px-only](screenshots/2026-10-16-xvfb-notice-px-only.png) |

見つかった不備: モーダルの通知の初版 `px only: no screen size` は窓の右端で切れた。`no screen size` に短くした。

## 14. WSLg の実機（2026-10-17・施主の目視と計測）

枝 `feat/12-frame-notices`（`5630008`）。Win11 ＋ WSLg（Ubuntu 22.04）。入力は Windows 側の本物の入力、見た目は Windows 側の画面キャプチャ、当たり判定は Windows の `WindowFromPoint`。
全文は [wslg-2026-10-17.md](wslg-2026-10-17.md)。

### 14.1 Xvfb の記録が実機で成り立ったもの

透明・素通し・線での移動（WM 経由の `_NET_WM_MOVERESIZE`）・両方の鉤括弧での伸縮・ギアのモーダル（`WM_TRANSIENT_FOR`・WSLg がタイトルバーを付ける）・色見本と直接入力と拒否・保存と読み戻し・数字のコピー（X の CLIPBOARD と **Windows の Get-Clipboard の両方**）・矢印の 5 通り・Esc（枠とモーダル）。

### 14.2 実機で崩れたもの（Issue #14 / #15）

再目視は `feat/15-label-overlap`（`0edce90`）・2026-10-17 夕方・同じ機械と手順（[wslg-2026-10-17.md](wslg-2026-10-17.md) の「再目視」）。

| 項目 | 実測 | 原因 | 対応 | 再目視 |
| --- | --- | --- | --- | --- |
| 当たり判定の帯 | Windows 側は X の入力 shape に従わず、**描いた画素（アルファ > 0）だけが当たる**。線は 1 px、ギアの穴は素通し、数字は字画の上だけ | WSLg の RDP の窓が `WS_EX_LAYERED` で、アルファ 0 はクリックが抜ける | 当たり判定の矩形をアルファ 1/255 で描く（ADR 0008） | ✅ 線は 9 px の帯、ギアは 16×16 全体、数字は箱全体 |
| 影 | 飾りなしなのに外周に影（右下 20 px）。外周 5 px の当たり帯も影 | Weston の XWM が飾りなしの窓に影を描き、RDP で送る。`_NET_WM_WINDOW_TYPE` に `_KDE_NET_WM_WINDOW_TYPE_OVERRIDE` があれば描かない（施主が 300×200 の窓で実験） | map の前に付ける（ADR 0008） | ✅ 影なし、外周 5 px の当たり帯なし、下塗りは見えない |
| Alt+F4 | 閉じない。~~`Alt_L` ＋ `F4` の素の `KeyPress` として届く~~ **訂正: F4 は `KeyRelease` だけが届く**（押下と離しを分けずに読んだ誤り） | Windows が押下を握り、`SC_CLOSE` を遅れて送る | ~~F4 の押下を読む（ADR 0008）~~ Alt を見た後の F4 の離しでも閉じる（ADR 0009・Issue #20） | ❌ ADR 0008 の版は閉じない（押下が届かない）。Issue #20 の版は第 17 節 |
| 最前面 | `_NET_WM_STATE` が空、Windows 側 topmost=False | WSLg の `_NET_SUPPORTED` に `_NET_WM_STATE_ABOVE` が無い | 環境の制約。できないことを通知する（Issue #14） | ✅ 枠の通知は出る。⚠️ モーダルの通知は右端で `…man` と切れる |
| 縦書きの `270 px` | 左辺の主目盛の数字 `150` と重なる | 数字の置き場が目盛の数字の幅を見込んでいない | Issue #15 | ✅ `150` の右 |
| 通知 `no screen size: px only` | 左辺の主目盛の数字 `250` と重なる | 同上 | Issue #15 | ✅ `250` の右・幅 400 に収まる |

### 14.3 未解明

- ~~Alt+F4 を送った後の 2 回目の起動で、X 側では描けているのに Windows 側に絵が出ず、プロセスが無言で終わった。3 回目は正常。再現していない~~
  → 再目視で 2 つに分かれた。「無言で終わった」は遅れて届いた `SC_CLOSE`（終了コード 0）。「絵が出ない」は起動直後に再現し、窓を動かすと出た（RDP のアイドルからの復帰と重なった。原因は未検証）
- `_KDE_NET_WM_WINDOW_TYPE_OVERRIDE` を付けた枠で、MOVERESIZE・焦点・Alt+Tab・タスクバーがどう変わるかは未確認（Weston のソースでは影と飾りだけのはず）。再目視では右下の鉤括弧の伸縮（WM 経由）と、Windows の `SetForegroundWindow` での前面化は効いた
- 線を掴んでから 200 ms で動かし始めると動かず、600 ms 置くと動いた（WM 経由の MOVERESIZE の開始待ち）。人の操作では問題にならない速さだが、記録しておく

## 15. Issue #14 — WSLg への適合（2026-10-17）

同じコンテナ。Issue #12 の枝の上に積んだ。施主の WSLg での再目視は未（第 14.2 節の 4 件）。

### 15.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 46・application 16・adapters-x11 11・ui-x11 26・xtask 69、すべて成功 |
| `make conformance` | 違反なし（127 ファイル） |
| `make deny` / `make fmt-check` / `make doc-check` | ok・差分なし・警告なし |
| `make coverage` | core ＋ application: regions 97.93%・functions 95.20%・lines 96.83% |

画素の単体テストで、当たり判定の矩形の角と中央はアルファ ≥ 1、中身（100, 60）はアルファ 0、歯車の抜きはアルファ 1、反転した数字の箱の中はアルファ ≥ 1 を確かめた。
窓の角 (0, 0) は上辺の帯・左辺の帯・左上の鉤括弧の当たり判定が重なるのでアルファ 3 になる（1/255 が 3 回。目には見えない）。

### 15.2 Xvfb

| 手順 | 結果 |
| --- | --- |
| `xprop _NET_WM_WINDOW_TYPE` | `_KDE_NET_WM_WINDOW_TYPE_OVERRIDE, _NET_WM_WINDOW_TYPE_NORMAL` |
| 中身の素通し（`getmouselocation`） | 根の窓（`543`）。アルファ 1 の帯は X の入力 shape の外なので、X では素通しのまま |
| 帯の上 | 枠の窓 |
| 枠で `alt+F4` | プロセスが終了（`/proc` から消え、終了コード 0） |
| モーダルで `alt+F4` | モーダルだけ消え、枠は残る（`State: S`）。続けて Esc で終了（終了コード 0） |

🔴 **記録しておくこと（検証の手順の欠陥）:** `( … & )` で起動したプロセスは親の shell が先に消えると zombie になり、`pgrep` / `kill -0` が「生きている」と答える。
本節から `$!` を取って `/proc/<pid>/status` の `State` と `wait` の終了コードで判定するようにした。第 9〜12 節の「終了した」の判定は `xdotool search` で窓が消えたことで見ており、影響は無い。

## 16. Issue #15 — 縦書きの数字と通知の置き場（2026-10-17）

同じコンテナ。Issue #14 の枝の上に積んだ。core の `FrameLayout` だけを変えた。

### 16.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 47・application 16・adapters-x11 11・ui-x11 26・xtask 69、すべて成功 |
| `make conformance` | 違反なし（127 ファイル） |
| `make fmt-check` | 差分なし |
| `make coverage` | core ＋ application: regions 97.67%・functions 95.31%・lines 96.91% |

新しいテスト `vertical_label_and_notices_clear_the_left_tick_numbers` が、高さ 240 / 270 / 1200 で、縦軸の数字の矩形と通知の矩形が左辺の主目盛の数字の矩形と交わらないことを見る。

### 16.2 Xvfb

`unit = mm` で起動し（Xvfb はミリ寸法 0 なので通知が出る）、右下の鉤括弧で高さ 270 に伸ばして撮った。
縦書きの `270 px` は `150` の右、通知 `no screen size: px only` は `250` の右に出て、重ならない。[height-270](screenshots/2026-10-17-xvfb-height-270.png)

## 17. Issue #20 — WSLg の Alt+F4（2026-10-17）

施主の WSL（Ubuntu 22.04・rustc 1.98.0・cairo 1.16.0）。Issue #15 の枝の上に積んだ。ADR 0009。

### 17.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 47・application 16・adapters-x11 11・xtask は成功。ui-x11 は 32 件中 31 成功、**`canvas::tests::a_successful_copy_inverts_the_number` が失敗**。本件の変更を退避した `feat/15-label-overlap`（`0edce90`）でも同じく落ちるので既存の失敗（フォント依存の推定・Issue #21） |
| `make conformance` | 違反なし（129 ファイル） |
| `make fmt-check` / `make doc-check` | 差分なし・警告なし |

新しいテスト: `alt_f4_gesture` の 6 件と、`key_input` の Alt の 2 行。

### 17.2 Xvfb（`-listen tcp`・WM 無し）

WSLg の機械では `/tmp/.X11-unix` が WSLg のもので Xvfb が unix ソケットを作れないので、`Xvfb :97 -listen tcp -nolisten unix` と `DISPLAY=127.0.0.1:97` で起動した。
X サーバは押されていない鍵の離しを捨てるので、F4 の離しは `XSendEvent`（`KeyRelease`・keycode は `XK_F4`）で窓へ直接送った。Alt の押下と離しは `xdotool keydown` / `keyup`。

| 手順 | 期待 | 結果 |
| --- | --- | --- |
| 枠で `xdotool key alt+F4`（押下の経路） | 終了 | 終了コード 0 |
| 枠で Alt 押下 → F4 の離し（`state` 0x8）→ Alt 離し（WSLg の並び） | 終了 | 終了コード 0 |
| 枠で F4 の離し（`state` 0x8）だけ。Alt の押下なし | 残る | 残る |
| 枠で Alt 押下 → Alt 離し → F4 の離し（`state` 0x8） | 残る | 残る |
| 枠で Alt 押下 → F4 の離し（`state` 0x0）→ Alt 離し | 残る | 残る |
| 枠で Esc | 終了 | 終了コード 0 |
| モーダルで Alt 押下 → F4 の離し（`state` 0x8） | モーダルだけ閉じる | 閉じる・枠は残る |
| モーダルで F4 の離しだけ | 残る | 残る |
| モーダルで `xdotool key alt+F4` | モーダルだけ閉じる | 閉じる・枠は残る |

### 17.3 WSLg（施主の機械・`e535bad` の release ビルド）

| 手順 | 結果 |
| --- | --- |
| 施主の本物のキーボードで、枠に Alt+F4 を 3〜4 回 | 枠に届いたのは最後の 1 回だけ（`xev`: `KeyPress Alt_L` → `KeyRelease F4` state 0x108）で、その回で終了コード 0。それ以前の回は枠に鍵が届いていない（施主が同時に別の操作をしていた。焦点が枠に無かった） |
| 定規が見えていて別の窓（ASHEN）が前面の状態で、線を 1 回クリック | Windows の前面が `NeNe Ruler (Ubuntu-22.04)` になり、X の焦点も枠の窓に来る（`xdotool getwindowfocus`） |
| 続けて Alt+F4（`keybd_event`・前面の題名を確かめてから送る） | 1 秒以内に終了コード 0 |
| ギアを押してモーダルを開き、Alt+F4 | モーダルだけ閉じる。枠は残る |
| その後 26 秒待って線を 2 回クリック | 枠は残る（モーダル宛の `SC_CLOSE` が遅れて届いても巻き添えにならない） |

`WSLg` では定規が最前面にならないので、他の窓の陰に入るとクリックはその窓に行き、焦点も来ない（ADR 0008 の 5・最前面の通知のとおり）。

## 18. Issue #23 — モーダルの最前面の通知が右端で切れる（2026-10-17）

施主の WSL。Issue #20 の枝の上に積んだ。モーダルの通知の文言を `not supported by this window manager` から `not supported here` に縮めた（枠の通知は幅に余裕があるので変えない）。

### 18.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 47・application 16・adapters-x11 11・xtask は成功。ui-x11 は 33 件中 32 成功。失敗は第 17.1 節と同じ既存の `a_successful_copy_inverts_the_number`（Issue #21） |
| `make conformance` | 違反なし |
| `make fmt-check` | 差分なし |

新しいテスト `the_layer_notice_ends_inside_the_right_margin` は、最前面に対応しない環境でモーダルを描き、通知の行の赤いインクの右端が「窓の幅 − 余白」より左にあることを見る。
旧い文言に戻すと落ち、新しい文言で通ることを確かめた（テストが不備を捕まえる証明）。

### 18.2 WSLg

モーダルを開き、Windows 側の画面キャプチャで `not supported here` が「Always on top」の右に、窓の右端より内側で出ることを確かめた。

## 19. Issue #26 — 引き継ぎの施主判断を記録し、QLT-009 を region で active にする（2026-10-17）

施主の WSL。Issue #23 の枝の上に積んだ。rustc 1.98.0 / cargo-llvm-cov 0.9.0。

### 19.1 QLT-009 の発火

| 実行 | 結果 |
| --- | --- |
| `make coverage`（`--fail-under-regions 90`） | 終了 0。core ＋ application: regions 97.67%・functions 95.31%・lines 96.91% |
| `cargo llvm-cov -p nene-ruler-core -p nene-ruler-application --locked --summary-only --fail-under-regions 98` | **終了 1。** 表は同じ値で出し、下限を割った理由の文言は出さない |

下限を実測値より上に置いたときだけ落ちるので、`--fail-under-regions` が効いている。

### 19.2 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make conformance` | 違反なし |
| `make doc-check` | 警告なし |

製品コードの変更は `ModalTheme` の doc コメント 1 行だけなので、`make test` は回していない（第 18.1 節の結果を再利用・ADR 0004）。

## 20. Issue #28 — HiDPI の倍率 1x / 2x（2026-10-17）

施主の WSL。Issue #26 の枝の上に積んだ。倍率の決め方と、掛けるもの・掛けないものは [ADR 0011](../adr/0011-integer-ui-scale.md)。

### 20.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 55・application 18・adapters-x11 11・xtask 69 は成功。ui-x11 は 36 件中 35 成功。失敗は既存の `a_successful_copy_inverts_the_number`（Issue #21） |
| `make conformance` | 違反なし（131 ファイル） |
| `make doc-check` | 警告なし |
| `make coverage` | core ＋ application: regions 98.03%・functions 96.12%・lines 97.41% |

新しい画素テスト `at_double_the_layer_notice_ends_before_the_right_ticks` は、2x の枠で最前面の通知が右辺の中目盛より手前で終わることを見る。
旧い文言 `always on top: not supported by this window manager` に戻すと落ち（右端 392・目盛 393）、新しい文言で通ることを確かめた。

### 20.2 Xvfb（`-noreset` で起動）

Xvfb は最後のクライアントが切れるとサーバを初期化し、ルート窓の `RESOURCE_MANAGER` を消す。`-noreset` を付けないと `xrdb` で置いた `Xft.dpi` が定規の起動前に消える。

| `Xft.dpi` | 枠の窓 | 見え |
| --- | --- | --- |
| 96 | 424 × 264 | 今まで通り |
| 192 | 448 × 288 | 帯・鉤括弧・ギア・目盛の長さ・数字・通知が 2 倍。線は 1 px のまま。最初は通知が枠の右からはみ出したので、文言を縮めた（ADR 0011 の 5） |
| 192（モーダル） | 640 × 672 | 部品と字が 2 倍。2x の座標 (180, 340) のクリックで単位が mm になった（一時の `HOME` で保存を確認） |
| 192（伸縮） | 左上の鉤括弧 (8, 8) を (108, 58) へ | 窓は (100, 50)・幅 348（枠 300 ＋ 2x の余白 48） |

### 20.3 WSLg

施主の WSLg には `Xft.dpi` が無く（`xrdb -query` が空）、主出力 rdp-0 は RandR で 0 mm なので、**そのままでは 1x になる**。
`xrdb -merge` で `Xft.dpi: 192` を置いて起動すると、Windows 側の画面キャプチャで 2x の枠が透明のまま出て、通知 `always on top: not supported here` は枠の中に収まった。確認の後に `xrdb -remove` で元の空に戻した。

## 21. Issue #30 — 最前面の通知と mm / in を外す（2026-10-17）

施主の WSL。Issue #28 の枝の上に積んだ。決定は [ADR 0012](../adr/0012-px-only-and-no-layer-notice.md)。

### 21.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 52・application 18・adapters-x11 12・xtask 69 は成功。ui-x11 は 33 件中 32 成功。失敗は既存の `a_successful_copy_inverts_the_number`（Issue #21） |
| `make conformance` | 違反なし（130 ファイル） |
| `make doc-check` / `make fmt-check` | 警告なし・差分なし |
| `make coverage` | core ＋ application: regions 97.92%・functions 95.83%・lines 97.27% |

### 21.2 実測（WSLg・`Xft.dpi: 192`）

- 枠: Windows 側の画面キャプチャで、`always on top: not supported here` が出ない（`_NET_WM_STATE_ABOVE` の無い WSLg）
- モーダル: 640 × 560（2x）。単位の段・`no screen size`・`not supported here` が無い
- 施主の設定ファイル（`unit = px` を含む v1）を読んで起動できた

### 21.3 WSLg の最前面（作らなかった回避策の記録）

msrdc の窓 `NeNe Ruler (Ubuntu-22.04)` に Windows 側から `SetWindowPos(HWND_TOPMOST)` を掛けると、拡張スタイルが `0x80100` → `0x80108`（`WS_EX_TOPMOST`）になり、5 秒後も残った。
別の窓（NeNeNib）を前面にしても、定規の描いた画素の上では `WindowFromPoint` が定規を返した。Windows 版で不要になるので実装はしない（ADR 0012 の却下した選択肢）。

## 22. Issue #32 — 2x でも主目盛の数字は 9 px（2026-10-17）

施主の WSL。Issue #30 の枝の上に積んだ。決定は [ADR 0013](../adr/0013-tick-numbers-keep-their-size.md)。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 52 は成功（`double_grows_the_window_by_twice_the_margins` に 2x の主目盛の数字のベースライン (62, 46) と左辺の数字の幅が 1x と同じことを足した）。ui-x11 は 33 件中 32（既存の #21） |
| `make conformance` | 違反なし |

Xvfb（`-noreset`・`Xft.dpi: 192`・一時の `HOME`）で撮り、主目盛の数字が 1x と同じ大きさ、`400 px`・鉤括弧・ギア・通知は 2x のままであることを確かめた。

## 23. Issue #34 — モーダルを 1x・飾りなし・自前の × と移動に（2026-10-17）

施主の WSL。Issue #32 の枝の上に積んだ。決定は [ADR 0014](../adr/0014-frameless-single-scale-modal.md)。

### 23.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 52・application 18 は成功。ui-x11 は 34 件中 33（既存の #21）。縁線と × の画素テストを足した |
| `make conformance` | 違反なし（132 ファイル） |

### 23.2 Xvfb の実操作（`Xft.dpi: 192`・WM なし・一時の `HOME`）

| 操作 | 結果 |
| --- | --- |
| 2x の枠のギア (432, 272) をクリック | モーダル `NeNe Ruler Settings` が 320 × 280（1x）で開く。黒い背景・灰色の縁線・右上の × |
| 題の右 (150, 20) を押して (250, 120) へ動かして離す | WM が無いので自前のドラッグ。モーダルが (100, 100) へ動いた |
| 動いた先の × (+300, +20) をクリック | モーダルが閉じた（`xdotool search` で見つからない） |

## 24. Issue #36 — 枠の窓のポインタの形（2026-10-17）

施主の WSL。Issue #34 の枝の上に積んだ。X の標準カーソルフォントの字形を使い、依存は増やしていない。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | ui-x11 は 35 件中 34（既存の #21）。当たり判定の種類 → 字形の対応のテストを足した |
| `make conformance` | 違反なし |

Xvfb（`Xft.dpi: 192`）でポインタを置き、XFixes の `GetCursorImage` で、そのとき出ているカーソルを読んだ。

| ポインタの位置 | カーソル（16 × 16 のホットスポット・serial） |
| --- | --- |
| 窓の外 (600, 500) | (7, 7)・1（根の窓の既定） |
| 上辺の線 (200, 8) | (8, 8)・2（`XC_fleur`） |
| 左上の鉤括弧 (3, 3) | (1, 1)・3（`XC_top_left_corner`） |
| 右下の鉤括弧 (405, 245) | (14, 14)・4（`XC_bottom_right_corner`） |
| ギア (432, 272) | (0, 1)・5（`XC_hand2`） |
| 幅の数字 (204, 64) | (0, 1)・5（`XC_hand2`） |
| 枠の中身 (200, 150) | (7, 7)・1（入力 shape の外なので下の窓の形） |

## 25. Issue #38 — 2x でも幅と高さの数字は 11 px（2026-10-17）

施主の WSL。Issue #36 の枝の上に積んだ。決定は [ADR 0015](../adr/0015-length-numbers-keep-their-size.md)。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 52 は成功（2x の横の数字のベースライン (208, 61) と当たり判定 (187, 50, 42, 15) を足した）。ui-x11 は 35 件中 34（既存の #21） |
| `make conformance` | 違反なし |

Xvfb（`Xft.dpi: 192`）で撮り、`400 px` / `240 px` が 1x と同じ大きさで出ることを確かめた。

## 26. Issue #40 — 入力欄を押すとモーダルが焦点を取る（2026-10-17）

施主の WSL。Issue #38 の枝の上に積んだ。

### 26.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件（`window_of` が認知的複雑度 12/10 になったので、入力と窓のイベントの 2 つに分けた） |
| `make test` | ui-x11 は 36 件中 35（既存の #21）。焦点ありで選択色の縁と棒、なしで無いことの画素テストを足した |
| `make conformance` | 違反なし |

### 26.2 Xvfb（WM なし・一時の `HOME`）

| 操作 | 結果 |
| --- | --- |
| ギアでモーダルを開き、`xdotool windowfocus` で焦点を枠の窓へ移す | 焦点は枠の窓 |
| 入力欄 (60, 120) をクリック | 焦点がモーダルへ移った（`xdotool getwindowfocus`）。欄の縁が白になり、文字の後ろに棒 |
| Backspace × 6・`00aa55`・Return | インク色が `#00AA55` になり、設定ファイルに保存された |
| 焦点を枠の窓へ戻す | 縁が灰色に戻り、棒が消えた |

最初の実装は `SetInputFocus` の後に flush しておらず、要求がバッファに残ったまま焦点が移らなかった（押した部品が入力欄なので、その後に何も送らない）。flush を足して直した。
WSLg の施主の報告（押しても焦点が来ない）も、これと同じ理由の可能性がある。施主の目視で確かめる。

### 26.3 WSLg の焦点の実験（試験窓）

実際のモーダルと同じ印（飾りなし・`_KDE_NET_WM_WINDOW_TYPE_OVERRIDE`・`WM_TRANSIENT_FOR`）を持つ試験窓を python（ctypes の Xlib）で出し、Windows 側で `SetForegroundWindow` して `XGetInputFocus` を読んだ。
単独の transient では 5 回とも焦点がその窓に来た（親を前面にしても子へ移る）。transient を 2 つ同時に出した試行では、飾りなしの窓を前面にしても焦点が `None` のことがあった。

## 27. Issue #42 — モーダルの Quit ボタン（2026-10-17）

施主の WSL。Issue #40 の枝の上に積んだ。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 52 は成功（`Quit` の当たり判定を足した）。ui-x11 は 37 件中 36（既存の #21・`Quit` の描画のテストを足した） |
| `make conformance` | 違反なし |

Xvfb: ギアでモーダルを開き、「Always on top」の段の右端の `Quit` (270, 228) を押すと、プロセスが終了コード 0 で終わった。

## 28. Issue #44 — カラーピッカー（2026-10-17）

施主の WSL。Issue #42 の枝の上に積んだ。決定は [ADR 0016](../adr/0016-hsv-picker-in-core.md)。

### 28.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 56・application 19・adapters-x11 12 は成功。ui-x11 は 38 件中 37（既存の #21） |
| `make conformance` | 違反なし（137 ファイル） |
| `make coverage` | core ＋ application: regions 98.17% |

色相を最初 1 度刻みにしたら、プリセット `#FB8C00` の往復が `#FB8A00` になってテストが落ちた。0.1 度刻みに直した。

### 28.2 Xvfb の実操作（一時の `HOME`・インク色は既定の `#E53935` から）

| 操作 | 保存されたインク色 |
| --- | --- |
| 帯の (234, 208) を押す（色相 約 242 度・彩度と明度はそのまま） | `#3B35E5` |
| 四角の右上 (215, 144) を押す（彩度 1000・明度 1000） | `#0900FF` |
| 四角の (100, 200) を押したまま (60, 180) → 窓の外 (300, 400) へ引いて離す | `#000000`（外は縁へ寄せ、彩度 1000・明度 0） |

撮った画面で、四角・帯・丸の印・帯の横線が出て、入力欄が `#000000` を示すことを確かめた。

## 29. Issue #46 — 枠からはみ出す目盛の数字を置かない（2026-10-17）

施主の WSL。Issue #44 の枝の上に積んだ。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 57 は成功（幅 360 で `350` を置かず 400 で置く・高さ 252 で `250` を置かず 260 で置く、を足した）。ui-x11 は 38 件中 37（既存の #21） |
| `make conformance` | 違反なし |

## 30. Issue #48 — 小さな枠の詰めた表示（2026-10-17）

施主の WSL。Issue #46 の枝の上に積んだ。決定は [ADR 0017](../adr/0017-compact-frame-detail.md)。

### 30.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件（`impl Trait` の引数を型引数に直した） |
| `make test` | core 60・application 20・adapters-x11 12 は成功。ui-x11 は 39 件中 38（既存の #21） |
| `make conformance` | 違反なし（140 ファイル） |
| `make coverage` | core ＋ application: regions 98.31% |

### 30.2 Xvfb（`Xft.dpi: 192`・合成器なしで不透明）

`xdotool windowsize` で窓を縮めて撮った。

| 窓 | 枠 | 見え |
| --- | --- | --- |
| 360 × 288 | 312 × 240 | 全部描く。上辺の目盛の数字は `250` まで（`300` は右の線を越えるので置かない・Issue #46） |
| 200 × 288 | 152 × 240 | 全部描く。不透明の通知が右の線からはみ出したので、枠の内側で切るようにした |
| 150 × 100 | 102 × 52 | 詰めた表示。目盛と通知が消え、中央に `102 × 52 px` |
| 112 × 84 | 64 × 40 | 詰めた表示。`64 × 40 px` は枠の幅ぎりぎりで、右の線に触れる（ADR 0017 の残る穴） |

## 31. Issue #50 — 線を消した表示（2026-10-17）

施主の WSL。Issue #48 の枝の上に積んだ。決定は [ADR 0018](../adr/0018-bare-frame-detail.md)。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 61・application 20 は成功。ui-x11 は 39 件中 38（既存の #21） |
| `make conformance` | 違反なし |

Xvfb（合成器なしで不透明）で、1x（`Xft.dpi: 96`）と 2x（192）の枠 102 × 52・60 × 40・40 × 40 を撮った。102 × 52 は詰めた表示（線あり）、60 × 40 と 40 × 40 は線が消えて細い鉤括弧になり、`W × H px` が窓の中央に出た。1x の 40 × 40 では文字が窓（64 px）の幅いっぱい。

## 32. Issue #52 — 窓を最小の枠より小さく縮めさせない（2026-10-17）

施主の WSL。Issue #50 の枝の上に積んだ。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件（`step` が 63 行になったので、窓の大きさの変化を `resized` に分けた） |
| `make test` | core 62 は成功（`smallest_window` が 1x で 64・2x で 88 を足した）。ui-x11 は 39 件中 38（既存の #21） |
| `make conformance` | 違反なし |

Xvfb（`Xft.dpi: 192`）:

| 操作 | 修正前 | 修正後 |
| --- | --- | --- |
| `xprop WM_NORMAL_HINTS` | 最小の大きさなし | `program specified minimum size: 88 by 88` |
| `xdotool windowsize` で 60 × 60 | 窓は 60 × 60 のまま。`px` とギアが窓の外で切れた | 窓が 88 × 88 に戻り、`40 × 40 px` とギアが窓の中に出た |
| 最小の窓で、左上の鉤括弧を右下へ引く（WM なし・自前のドラッグ） | — | 窓は 88 × 88 のまま（下限が `smallest_window`） |

## 33. Issue #54 — 最小の枠 16 × 16 と短い形（2026-10-17）

施主の WSL。Issue #52 の枝の上に積んだ。決定は [ADR 0019](../adr/0019-smaller-minimum-and-short-size.md)。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 64・application 21・adapters-x11 12 は成功。ui-x11 は 39 件中 38（既存の #21） |
| `make conformance` | 違反なし |

Xvfb で `xdotool windowsize` を 10 × 10 にすると、1x（`Xft.dpi: 96`）では窓 40 × 40、2x（192）では 64 × 64 に戻り、どちらも枠 16 × 16 の線を消した表示で `16×16` が窓の中に出た。文字は細い鉤括弧に重なる。

## 34. Issue #56 — 線を消した表示では鉤括弧の縦の腕を描かない（2026-10-17）

施主の WSL。Issue #54 の枝の上に積んだ。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | core 64 は成功（Bare のインクが横の腕 2 本だけ、を確かめるように直した）。ui-x11 は 39 件中 38（既存の #21） |
| `make conformance` | 違反なし |

Xvfb（2x）で 16 × 16 まで縮めて撮り、鉤括弧が横の腕だけになって `16×16` に重ならないことを確かめた。

## 35. Issue #58 — アプリのアイコン（2026-10-17）

施主の WSL。Issue #56 の枝の上に積んだ。決定は [ADR 0020](../adr/0020-app-icon-drawn-in-app.md)。

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make test` | ui-x11 は 41 件中 40（既存の #21）。アイコンの画素と `_NET_WM_ICON` の並びのテストを足した |
| `make conformance` | 違反なし（144 ファイル） |

Xvfb: 枠の窓とモーダルの窓の `xprop _NET_WM_ICON` に 16 / 24 / 32 / 48 / 64 / 128 の 6 つが出た。`xprop -f _NET_WM_ICON 32c` の値を画像に戻し、明るい地と暗い地に並べて、デザインのカンバスの案 A と同じ絵であることを確かめた。

## 36. Issue #60 — Windows 版の土台と枠の窓（2026-10-18）

施主の WSL。main（`baa859a`）から切った。決定は [ADR 0021](../adr/0021-windows-port-direct2d-and-confined-unsafe.md)。
WSL に `rustup target add x86_64-pc-windows-gnu`（1.98.0）と `gcc-mingw-w64-x86-64` を入れ、Windows の exe を WSL で作った。

### 36.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` | 0 件 |
| `make lint-windows` | 0 件（Windows のターゲットの clippy。最初は `semicolon_inside_block`・`undocumented_unsafe_blocks`・`too_many_arguments`・`excessive_nesting` が出たので直した） |
| `make test` | core 64・application 21・settings-file 9・adapters-x11 3・ui-win32 1・xtask 75 は成功。ui-x11 は 41 件中 40（既存の #21） |
| `make conformance` | 違反なし（163 ファイル）。CNF-014 の証明行が無い状態では CNF-009 が止めた |
| `make deny` | 最初は `unicode-ident` の `Unicode-3.0`（windows の手続きマクロの下）で licenses が落ちた。許可に足して ok |
| `make build-windows` | `nene-ruler.exe` 1.6 MB。mingw のリンカが `corrupt .drectve at end of def file` の警告を 1 つ出す（windows-link の取り込み表・動作に関係しない） |

xtask の新しいテスト: `cnf012_allows_the_windows_adapters_and_the_settings_file`・`cnf013_reads_target_specific_dependencies`・`cnf013_keeps_the_settings_file_free_of_external_crates`・`cnf014_accepts_a_faithful_copy_with_unsafe_denied`・`cnf014_catches_a_loosened_or_missing_lint`・`cnf014_ignores_other_crates`。

### 36.2 施主の Windows での起動（画面のキャプチャ）

exe を `C:\Users\Public` に置き、PowerShell の `Start-Process` で起動して 2 秒後に主モニタの中央を撮り、`Stop-Process` で閉じた。

- 透明な枠が出て、下のターミナルが透けた。線・目盛・目盛の数字・`400 px`・縦書きの `240 px`・鉤括弧・ギアが Direct2D で描けた
- 最初は幅が `384 px` だった。作成直後の `WM_NCCALCSIZE`（wparam = 0）に答えておらず、`WS_THICKFRAME` の左右 8 px ずつが差し引かれていた。常に 0 を返すように直して `400 px` になった
- 施主が手で確かめた（2026-10-18）: 線で移動（十字の矢印）・鉤括弧で伸縮（斜めの両矢印）・数字のクリックでコピー・矢印で 1 px と Shift で 10 px・Esc と Alt+F4 で終了・ほかの窓より上に残る（最前面）の 6 項目すべて成り立った

## 37. Issue #62 — Windows の設定モーダル（2026-10-18）

施主の WSL。Issue #60 の枝の上に積んだ。

### 37.1 ゲート

| target | 結果 |
| --- | --- |
| `make lint` / `make lint-windows` | 0 件（`manual_midpoint`・`doc_markdown`・`default_numeric_fallback`・`impl_trait_in_params` を直した） |
| `make test` | ui-x11 の既存の #21 以外すべて成功 |
| `make conformance` | 違反なし（171 ファイル） |

### 37.2 共有の状態の作り

Issue #60 の窓の手続きは、窓に結んだ生のポインタから毎回 `&mut` を作り直していた。処理の中で同じスレッドの窓へ同期的にメッセージが届く（`UpdateLayeredWindow`・窓を作る・見せる）と、借用が二重になる。
枠とモーダルの 2 つの窓が 1 つの `WindowsApp` を `RefCell` で共有し、窓の手続きは `try_borrow_mut` で借りるように直した。借りている最中に入れ子で届いたメッセージは既定の手続きへ流れる。
窓を閉じるのは `WM_CLOSE` を投げて後で行う（借りている最中に壊すと、入れ子の `WM_DESTROY` が状態に届かず `WM_QUIT` が積まれない）。

### 37.3 施主の Windows（画面に触らない確かめ方）

PowerShell の `PostMessage` で、枠の窓へギアの位置の `WM_LBUTTONDOWN` / `WM_LBUTTONUP` を送り、画面を撮った。

- 最初はモーダルが出なかった。一時の記録（ファイルへ書く・コミットしない）で見ると、送った (416, 256) が (520, 320) で届いていた。PowerShell は dpi 非対応で、Windows が座標を 125%（施主の画面の拡大率）に直す。割り戻した (333, 205) で送ると (417, 257) で届き、ギアに当たった
- モーダルが枠の右に開いた。黒い背景・縁線・×・色見本と選択の輪・入力欄と棒（焦点あり）・ピッカー・見え・最前面・Quit・脚注が、X11 版と同じ置き場で描けた
- 部品の操作は施主の手で確かめた（下の表）

| 項目 | 結果（施主・2026-10-18） |
| --- | --- |
| ギアでモーダルが枠の右に開く | ✅ |
| 色見本・ピッカーで枠の色がすぐ変わる | ✅ |
| `#RRGGBB` の欄に焦点が来て、Enter で色が変わる | ✅ |
| Dark / Light の切り替え | ✅ |
| 最前面を外すと他の窓の後ろに回れる | ✅ |
| × と Esc でモーダルだけ閉じ、Quit で全部終わる | ✅ |
| 何も無い所を掴んでモーダルを動かせる | ✅ |

あわせて施主が「目盛が実際に描画されているピクセルの所しか hover が効かない」とした。Issue #66 で扱う（第 38 節）。

## 38. Issue #66 — 目盛の帯を移動の当たり判定にする（2026-10-18）

施主の WSL。PR #65 の枝の上に積んだ。ADR 0022。

### 38.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `make lint` / `make lint-windows` | 0 件 |
| `make test`（`--no-fail-fast` で全クレート） | ui-x11 の既存の #21 以外すべて成功。core に 2 件足し、application の当たり判定の数を 9 → 13 に直した |
| `make conformance` | 違反なし（173 ファイル） |

### 38.2 施主の Windows（`WindowFromPoint` の地図・入力は送らない）

dpi 対応にした PowerShell で枠の窓を起動し、窓の中を横 2 px・縦 3 px おきに `WindowFromPoint` で引いて、枠の窓が返る所を `#` にした（窓 424 × 264・1x）。

- 直す前: 線の帯（9 px）と数字の矩形は全面が当たり、目盛と目盛の数字は描いた画素だけが当たって、隙間は下の窓へ抜けた（施主の言う穴あき）
- 直した後: 上辺は窓の上端から 28 px、左辺は 37 px、下辺と右辺は線の帯と合わせて約 16 px が全面当たる。幅と高さの数字は矩形で当たり、その手前の 1〜2 px と枠の中央は抜ける

### 38.3 Xvfb（`-listen tcp`・合成器 `xcompmgr`・WM 無し）

`xdotool mousemove` → `getmouselocation` の窓（枠 = `8388609`、根 = `1293`）。

| 点 | 何の上 | 結果 |
| --- | --- | --- |
| (61, 12) / (61, 24) | 上辺の目盛の隙間 / 目盛の数字の隙間 | 枠 |
| (20, 130) / (395, 130) / (200, 236) | 左辺 / 右辺 / 下辺の目盛の帯 | 枠 |
| (10, 10) | 左上の鉤括弧 | 枠 |
| (200, 29) / (38, 130) | 上辺の帯と幅の数字の間 / 左辺の帯と高さの数字の間 | 根（素通し） |
| (200, 120) | 枠の中央 | 根（素通し） |

### 38.4 施主の目視

施主が Windows で、目盛の隙間でも移動のポインタが出て掴めることを確かめた（2026-10-18「うん、いいと思う」）。

## 39. Issue #69 — `make prove` が回らなくなっていた（2026-10-18）

施主の WSL。Issue #68（アイコンの埋め込み）でゲートを触るので `make prove` を回したら、main でも落ちた。

| 原因 | いつから | 直し方 |
| --- | --- | --- |
| 仮の workspace の `members` を 1 行だけ置き換えていた。根の `Cargo.toml` の `members` が複数行になり、2 行目以降が残って TOML が壊れた | PR #61（97a6776） | `members = [` から閉じ括弧の行までを 1 行に替える（`probe_members`） |
| P-24 / P-25 が `deny.toml` の `allow = ["MIT"]` という文字列を置き換えていた。許可が 4 つの並びになって置き換えが空振りし、使っていないライセンスで落ちた | PR #2（cc84442） | `allow = [` で始まる行を替える（`license_allow`）。戻すときは probe の MIT だけにする |

| target | 結果 |
| --- | --- |
| `cargo test -p xtask` | 78 件成功（置き換えの単体テスト 3 件を足した） |
| `make prove` | 26 件すべて成功（`target/proofs/results.json`） |

## 40. Issue #68 — アイコンを core に移し、Windows の exe に埋める（2026-10-18）

施主の WSL。PR #70（Issue #69）の枝の上に積んだ。ADR 0023。

### 40.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `make lint` / `make lint-windows` / `make doc-check` | 0 件 |
| `make test`（`--no-fail-fast` で全クレート） | ui-x11 の既存の #21 以外すべて成功。core に 6 件（`app_icon` 4・`icon_file` 2）、xtask に 2 件（CNF-008 / CNF-012 の例外）を足した |
| `make conformance` | 違反なし（177 ファイル）。例外を足す前は `crates/app/build.rs` に CNF-008 が 1 件、CNF-012 が 3 件出た |
| `make deny` | 通る（依存は増えていない。build-dependency は内部の core だけ） |
| `make prove` | 26 件すべて成功 |

### 40.2 exe の資源

`make build-windows` の exe の `.rsrc` を読んだ。`RT_ICON`（3）に 8 枚（1128〜270376 バイト、16〜256 px）、`RT_GROUP_ICON`（14）の番号 1 に 1 つ。exe は約 2.0 MB（埋める前は約 1.67 MB）。

### 40.3 Xvfb（`-listen tcp`・WM 無し）

`xprop -f _NET_WM_ICON 32c` で 16 / 24 / 32 / 48 / 64 / 128 の 6 枚（24652 語）。128 px の角 `00000000`・中央 `ff1a1917`（地）・(50, 34) `fff5f2eb`（生成り）・(78, 93) `ffe53935`（赤）。cairo で描いていた時と同じ色。

### 40.4 施主の目視

施主が Windows で、エクスプローラーの exe・タスクバー・Alt+Tab に 2 つの鉤括弧のアイコンが出ることを確かめた（2026-10-18「OK」）。

## 41. Issue #72 — Linux のメニュー用のアイコンと WM_CLASS（2026-10-18）

施主の WSL。ADR 0024。

### 41.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `make lint` / `make lint-windows` / `make doc-check` | 0 件 |
| `make test`（`--no-fail-fast` で全クレート） | ui-x11 の既存の #21 以外すべて成功。core に `png_file` の 4 件を足した |
| `make conformance` | 違反なし（179 ファイル） |
| `make deny` | 通る |

### 41.2 書いた PNG を外の道具で読む

`cargo build --release --message-format=json` の `build-script-executed`（`nene-ruler` の `crates/app`）の `out_dir` の下に、`icons/hicolor/{16,24,32,48,64,128,256}x…/apps/nene-ruler.png` の 7 枚ができた。

- `file`: `PNG image data, 256 x 256, 8-bit/color RGBA, non-interlaced`
- Python の `zlib`（`-I`）で全塊の CRC を照合し、IDAT を展開した。7 枚とも角 `00000000`・中央 `1a1917ff`（地）。256 px は 262488 バイト
- 256 px を目で見た。暗い角丸の地に、左上の生成りと右下の赤の鉤括弧

### 41.3 Xvfb の WM_CLASS

`xprop WM_CLASS` → `WM_CLASS(STRING) = "nene-ruler", "nene-ruler"`。`xdotool search --class nene-ruler` が枠の窓を返す（前は何も返さなかった）。

## 42. Issue #74 — 配布の仕組み（2026-10-18）

施主の WSL と GitHub Actions。PR #73 の枝の上に積んだ。ADR 0025。

### 42.1 手元の `make dist`

`dist/` に `nene-ruler_amd64.deb`（309 KB）・`nene-ruler-linux-x86_64.zip`・`NeNe-Ruler-windows-portable.zip`・`nene-ruler.exe`（MSI の材料・strip 後 1.4 MB）と 3 つの `.sha256` ができ、`sha256sum -c` がすべて OK。

| 確かめたこと | 結果 |
| --- | --- |
| `dpkg-deb -I` | `Version: 0.1.0`・`Depends: libc6 (>= 2.34), libcairo2 (>= 1.4.10), libgcc-s1 (>= 4.2)`（`dpkg-shlibdeps` が決めた）・`Installed-Size: 1224` |
| `dpkg-deb -c` | `/usr/bin/nene-ruler`・`/usr/share/applications/nene-ruler.desktop`・`/usr/share/doc/nene-ruler/copyright`・`/usr/share/icons/hicolor/{16..256}/apps/nene-ruler.png` の 7 枚。持ち主は root |
| zip の中身 | Linux: 本体・`.desktop`・`icons/hicolor/…`・LICENSE・README。Windows: `nene-ruler.exe`・LICENSE・README |
| exe | strip の後も `.rsrc` が残る（アイコン）。依存する DLL は Windows の標準だけ（mingw の実行時ライブラリを同梱しなくてよい） |

最初の版は Linux の zip で `icons/` の下の `hicolor` が抜けていたので直した。

### 42.2 PR の上の release.yml

| 実行 | build（ubuntu-24.04） | msi（windows-2025・WiX 5.0.2） | release |
| --- | --- | --- | --- |
| 39100486456 | 成功 | 成功（`msi: NeNe Ruler 0.1.0`） | 飛ばした（PR） |
| 39100643145（checkout を v7.0.1 に上げた後） | 成功 | 成功 | 飛ばした（PR） |

- 1 回目は checkout（check.yml と同じ commit）に Node.js 20 の非推奨の警告が出た。v7.0.1 に上げた 2 回目は 0 件。check.yml は Issue #76
- 2 回目の成果物を `gh run download` で取り、4 つの `.sha256` を Linux の `sha256sum --check --strict` で照合して OK（Windows で書いた MSI の `.sha256` も LF で照合できた）

### 42.3 施主の Windows で MSI を入れる（実行 39100643145 の成果物）

- 14:35:02 にインストールが成功（MsiInstaller 11707 / 1033）。スタートメニューに鉤括弧のアイコンで出た
- 続けて「Please wait while Windows configures NeNe Ruler」のまま止まって見えた。`consent.exe` が「NeNe-Ruler-Setup.msi がアクセス許可を要求しています」で待っていた（UAC の確認がほかの窓の後ろにあった）。msi を開き直した分の再構成（1035）が 3 回、どれも成功
- 再構成がスタートメニューからの起動（宣伝されたショートカットの自動修復）でないことを確かめるため、施主がスタートメニューから起動し直した。問題なく起動し、14:36:12 より後の MsiInstaller の事象は 0 件

### 42.4 施主の機械でポータブル zip と `.deb` を試す

施主が、Windows のポータブル zip を展開して `nene-ruler.exe` が起動すること、WSL に `.deb` を `apt install` で入れてスタートメニューから起動できることを確かめた（2026-10-18「3 つとも大丈夫」）。MSI（第 42.3 節）と合わせて、配る 4 つのうち 3 つを施主の機械で試した。Linux の zip は試していない。

## 43. v0.1.0 を出す（2026-10-18）

ADR 0025 の手順で、施主に版とリリースノートを見せて OK をもらってから出した。

| 実行 | 何を | 結果 |
| --- | --- | --- |
| 39101655811 | main（`6e391cd`）で release.yml を手動で実行し、配る物を作る | build・msi が成功 |
| 39101864683 | `publish_from_run=39101655811`・`dry_run=true` | 成功。Release の実行・成功・main の commit・4 つの sha256 の一致・同じタグが無いことを確かめ、`dry run: nothing was tagged or published` |
| 39101887481 | 同じ実行を `dry_run=false` で出す | 成功。Release `v0.1.0`（タグは `6e391cd`）に 4 つと 4 つの `.sha256` |

出した後、`gh release edit` でリリースノートの先頭に施主に見せた要約を足した（自動の PR の一覧はその下に残した）。
これで ADR 0025 の `publish_from_run` の確かめ（planned）は、実際の 1 回で通った。

## 44. Issue #80 — Store に出す MSIX（2026-10-18）

施主の WSL と GitHub Actions。PR #79 の枝の上に積んだ。ADR 0026。身元は施主が Partner Center で予約した値。

| 確かめたこと | 結果 |
| --- | --- |
| `make lint` / `make lint-windows` / `make conformance` | 0 件・違反なし |
| 手元の `make dist` | `dist/msix-assets` に 25 枚。`Square150x150Logo` は 150 / 188 / 225 / 300 / 600、`Square44x44Logo` は 44 / 55 / 66 / 88 / 176、`StoreLogo` は 50〜200、`targetsize` は 16〜256（Loupe と同じ大きさ） |
| PR の上の release.yml（実行 39105299324） | build・msi が成功。msi のジョブが MSI に続けて MSIX を作った。版が 0.1.0 なので `NeNe-Ruler-store-NOT-SUBMITTABLE.msix` と警告（`MSIX の版 0.1.0.0 は先頭が 0 なので Store に出せない`） |
| 成果物の MSIX を手元で開く | `.sha256` が一致。中身は `AppxManifest.xml`・`AppxBlockMap.xml`・`[Content_Types].xml`・`nene-ruler.exe`・`resources.pri`・`Assets/` の 25 枚 |
| マニフェストの身元 | `Name="HideyukiMori.NeNeRuler"`・`Publisher="CN=C37230AA-B52D-403B-9BFD-E7980F088422"`・`PublisherDisplayName` は `Hideyuki Mori`。`store-identity.json`（Partner Center の値）と同じ。`EntryPoint="Windows.FullTrustApplication"`・`runFullTrust`・`MinVersion="10.0.19041.0"` |
| exe | MSIX の中の exe の SHA-256 は `50417DAF…`。スクリプトがポータブル zip の中の exe と同じことを確かめて通った |
| `resources.pri` | 25 枚の名前がすべて索引に載っている |

入れた状態の見え方と WACK はまだ（版を 1.0.0 に上げた後、施主の機械で）。

## 45. Issue #88 — Store の画像（2026-10-18）

| 確かめたこと | 結果 |
| --- | --- |
| 300×300 のタイル | `crates/app/build.rs` が Windows のターゲットで `OUT_DIR/store/store-logo-300.png` を書き、手元の `make dist` で `dist/store-logo-300.png`（300×300）ができた |
| スクリーンショット | `packaging/windows/capture-store-images.ps1` を施主の Windows（主画面・125%）で、施主の合図の後に 2 回動かした。撮る範囲は背景の窓（1920×1080）だけ |

撮影で分かったこと:

- Windows PowerShell 5.1 は BOM の無い UTF-8 を Shift_JIS として読み、日本語のコメントで構文エラーになった（実行の前に止まり、画面には何も出なかった）。スクリプトを BOM 付きにした
- 1 回目は施主の設定のインク（`#FDD835`）で描かれ、白の上で数字が読めなかった。ボタンを囲んだ 1 枚は `240 × 56 px` が `Sign up` の文字に重なった。2 回目は撮影で起動したルーラーにだけ空の一時フォルダを `APPDATA` として渡し、既定の設定（赤）で描かせた。施主の設定ファイルは読んでも書いてもいない。囲む対象を文字の無い入力欄に変えた
- 枠の位置は `SetWindowPos`、設定画面はギアの位置への `PostMessage` で開いた。マウスとキーボードは動かしていない。撮影で起動したルーラーは PID で止めた

- 施主の希望で、背景の架空の画面をダークモードの配色にして 3 回目を撮った（同じ手順・既定の設定）

3 回目の 3 枚（`docs/images/store/`）を目で見た。カード全体（1120 × 640 px・目盛と数字）・入力欄（中央に `992 × 56 px`）・設定画面（版 `1.0.0` の表示）。施主のデスクトップと他のアプリは写っていない。

## 46. Issue #90 — Microsoft Store への申請（2026-10-18）

施主が NeNe Ruler 1.0.0 を申請した（Submission 1）。入力の記録は `docs/release/store-listing.md` の「2026-10-18 の入力で分かったこと」。

| 物 | SHA-256 |
| --- | --- |
| 提出した `NeNe-Ruler-store.msix`（実行 39109585664・main `6f402f9`） | `1b3d6acc81a3bf19d3c765073690c8497900f7de099a7e4f50ebb820ad3a6343`（成果物の `.sha256` と一致） |
| その中の `nene-ruler.exe` | `d03781b05467a25c47dd9cc9b72368d6fd424544babe9cd20398c2072135485b` |
| 同じ実行のポータブル zip の中の `nene-ruler.exe` | `d03781b05467a25c47dd9cc9b72368d6fd424544babe9cd20398c2072135485b`（MSIX の中と同じ。`build-msix.ps1` も作るときに確かめた） |

WACK は回していない（施主の判断 2026-10-18・推し: 同じ形の Loupe は WACK の高 DPI の警告 1 件があっても認定された。Store の認定で同じ種類の検査が走る）。

認定の後、GitHub の `v1.0.0` は **同じ実行 39109585664** の成果物を `publish_from_run` で出す（ADR 0025 / 0026）。

## 47. v1.0.0 を出す（2026-10-19）

1.0.0（Submission 1・Issue #90）が 2026-10-19 に Microsoft Store の認定を通り、公開された（施主申告・<https://apps.microsoft.com/detail/9N6JVHKL9LM3>）。
ADR 0025 / 0026 の手順で、Store に出したのと同じ実行の成果物を作り直さずに出した。タグは手で打っていない。

| 実行 | 何を | 結果 |
| --- | --- | --- |
| 39245956568 | `publish_from_run=39109585664`・`dry_run=true` | 成功。5 つの sha256（MSIX を含む）が `OK`、`would publish v1.0.0 at 6f402f9446cffab0e5fda13eeba2c9b6a4dc92a2`、`dry run: nothing was tagged or published` |
| 39245998175 | 同じ実行を `dry_run=false` で出す | 成功。Release `v1.0.0`（Latest・タグは `6f402f9`）に 4 つと 4 つの `.sha256`。MSIX は添付しない |

リリースノートは自動の PR の一覧のまま。要約は施主に見せてから足す。

## 48. Issue #96 — Store 版の設定の置き場（2026-10-19）

施主の Windows。hub が `explorer.exe 'shell:AppsFolder\HideyukiMori.NeNeRuler_3sft4ch5kzywy!NeNeRuler'` で Store 版を起動し、
`Get-Process` でパスが `C:\Program Files\WindowsApps\HideyukiMori.NeNeRuler_1.0.0.0_x64__3sft4ch5kzywy\nene-ruler.exe` であることを確かめてから、施主がインクの色を変えて Esc で閉じた。
施主の機械には MSI 版も入っており、スタートメニューからの起動はどちらの版か区別できないので数えていない。

| 実の `%APPDATA%\nene-ruler` | 書かれた場所 | 他方 |
| --- | --- | --- |
| ある（MSI 版の残り） | 実の `%APPDATA%\nene-ruler\settings.v1.txt`（20:57:22・`ink=#E53935`） | パッケージの LocalCache には何も作られない |
| 無い（`nene-ruler.bak` へ退避） | `%LOCALAPPDATA%\Packages\HideyukiMori.NeNeRuler_3sft4ch5kzywy\LocalCache\Roaming\nene-ruler\settings.v1.txt`（20:58:29・`ink=#1E88E5`） | 実の `%APPDATA%\nene-ruler` は作られない |

起動しただけではフォルダは作られない。Loupe と同じく、フォルダが既にあるかどうかで置き場が分かれる。
後始末: 2 つ目でできたファイルは消し、`.bak` は元の名前に戻した。

## 49. Issue #100 — 単位を戻し、論理画素に切り替える（2026-10-19）

同じコンテナ。`make lint` / `make fmt-check` / `make test` / `make conformance` / `make doc-check` / `make deny` が 0 件で通り、`make coverage` は core ＋ application で regions 98.7% 台。
ui-win32 と adapters-windows は `cargo clippy --target x86_64-pc-windows-gnu -- -D warnings` が 0 件（`make lint-windows` そのものは windres が無いので app の build.rs で止まる・ADR 0023）。

語は施主が 2026-10-19 に決めた: モーダルと設定ファイルは `Physical px` / `Logical px`（`physical_px` / `logical_px`）で対にし、枠の数字は `400 px` / `266.7 logical px`。
枠の数字まで `physical px` にすると、目盛を保てる枠の下限が幅 112・高さ 120 から幅 174・高さ 182 に上がるので採らなかった（ADR 0027）。

### 49.1 Xvfb（`Xft.dpi: 144`・合成器あり・WM 無し・`XDG_CONFIG_HOME` は空のディレクトリ）

`xprop -root -format RESOURCE_MANAGER 8s -set RESOURCE_MANAGER 'Xft.dpi:<TAB>144'` で `xrdb` の代わりをし、`./target/debug/nene-ruler` を起動した。UI の倍率は 2x（ADR 0011）。

| 操作 | 命令 | 結果 | PNG |
| --- | --- | --- | --- |
| 起動 | | `400 px` / `240 px` | |
| ギアを押す | `mousemove 432 272 click 1` | モーダル 320 × 448。Ink の下に `Unit` の段（`Physical px` が選ばれ、`Logical px`）、その下の行に `1 logical px = 1.5 physical px`、さらに Settings window・Always on top | |
| `Logical px` を押す | `mousemove 688 276 click 1`（モーダルを (520, 0) へ動かしてから） | ボタンの選択が移る。枠の数字が `266.7 logical px` / `160 logical px` になる。ファイルは `version = 1` / `ink = #E53935` / `modal_theme = dark` / `unit = logical_px` / `layer = always_on_top` の 5 行 | [modal](screenshots/2026-10-19-xvfb-logical-px-modal-144dpi.png) |
| `Physical px` → `Logical px` | `mousemove 584 276 click 1` / `688 276` | `400 px` に戻り、また `266.7 logical px` に | |
| Esc | `key Escape` | モーダルが消え、枠は `266.7 logical px` / `160 logical px` のまま | |
| 右下の鉤括弧で引く | `mousedown 1` (405, 245) → `mouseup 1` (505, 285) | 枠 500 × 280 で `333.3 logical px` / `186.7 logical px`。目盛の数字は物理画素のまま（50・100・…・450） | [frame](screenshots/2026-10-19-xvfb-logical-px-frame-500x280-144dpi.png) |
| Esc → 再起動 | | `unit = logical_px` を読んで `266.7 logical px` / `160 logical px` で始まる（FR-012）。終了コード 0 | |

### 49.2 Xvfb（`Xft.dpi` 無し・同じ設定ファイル）

`xprop -root -remove RESOURCE_MANAGER` のあと起動。UI の倍率は 1x。

| 操作 | 結果 | PNG |
| --- | --- | --- |
| 起動 | `Logical px` が選ばれたまま `400 logical px` / `240 logical px`。dpi が無いので物理画素と同じ数 | |
| ギアを押す（`mousemove 417 257 click 1`） | 下の説明が `1 logical px = 1 physical px` | [modal-1x](screenshots/2026-10-19-xvfb-logical-px-modal-1x.png) |

**planned のまま**: 施主の Windows（150%）でブラウザの開発者ツールと数字を突き合わせること（ADR 0027）。

## 50. Issue #102 — 字の大きさは論理画素で、目盛は選んだ単位で（2026-10-20）

施主の WSL（Ubuntu・`/tmp/.X11-unix` が読み取り専用なので Xvfb は `-listen tcp -nolisten unix -ac` で立て、`DISPLAY=127.0.0.1:98`）。PR #101 の枝の上。
`make lint` / `make fmt-check` / `make test`（core 90・application 22・settings-file 10 ほか。ui-x11 の `a_successful_copy_inverts_the_number` はこの機械では main でも落ちる・フォント依存）/ `make conformance`（189 ファイル・違反なし）/ `make doc-check` / `make deny` が 0 件。
`make coverage` は core ＋ application で regions 98.99%・functions 97.35%・lines 98.36%。
ui-win32 と adapters-windows は `cargo clippy --target x86_64-pc-windows-gnu -- -D warnings` が 0 件。

### 50.1 Xvfb（`Xft.dpi: 144`・合成器あり・WM 無し・`XDG_CONFIG_HOME` は空のディレクトリ）

`xprop -root -format RESOURCE_MANAGER 8s -set RESOURCE_MANAGER 'Xft.dpi:<TAB>144'` のあと `./target/debug/nene-ruler` を起動。UI の倍率は 2x、比は 1.5。

| 操作 | 結果 | PNG |
| --- | --- | --- |
| 起動（物理画素） | `400 px` / `240 px`。字は 1.5 倍（主目盛の数字 14 px・幅と高さの数字 17 px）。目盛は 5 物理 px ごとで `50` `100` … `350` が 50 px ごと。第 49 節の同じ場面より字が大きく、目盛と数字の置き場はそのまま | [physical](screenshots/2026-10-20-xvfb-text-ratio-physical-144dpi.png) |
| ギア → `Logical px` → Esc | `266.7 logical px` / `160 logical px` で**全部描く**（第 49 節では字が小さかったので全部描けていた。字が大きくなっても、縦の数字と横の数字は横に離れているので交差判定で全部描ける）。目盛は 7.5 物理 px ごと（8 / 7 の交互）、主目盛は 75 px ごとに `50` `100` `150` `200`。字の大きさは物理画素のときと同じ | [logical](screenshots/2026-10-20-xvfb-text-ratio-logical-144dpi.png) |
| 右下の鉤括弧で 500 × 280 に引く | `333.3 logical px` / `186.7 logical px`。上辺の数字は `50` … `300`、左辺は `50` … `150`。ファイルは `unit = logical_px` | [logical-500x280](screenshots/2026-10-20-xvfb-text-ratio-logical-500x280-144dpi.png) |

見えた穴（ADR 0028 の「失うもの」に記録）: 論理画素の高さ 240 では左辺の `150`（原点から物理 225 px）が下辺の主目盛（長さ 24 px）に重なる。判定が枠の底しか見ない Issue #46 のままで、物理画素でも高さ 260 の `250` で起きる既存の穴。

**planned のまま**: 施主の Windows（150%）で字が読めることと、ブラウザの開発者ツールの要素の幅と目盛が合うこと（ADR 0028）。

## 51. Issue #104 — 比は定規の窓が乗っているモニタの dpi に従う（2026-10-20）

同じ WSL。PR #103 の枝の上。施主の実測（主 125%・他 150% / 175%・主で起動すると他のモニタで外れる）から、比を `WM_DPICHANGED` で取り直すようにした。
`make lint` / `make fmt-check` / `make test`（application 23 件。`the_ratio_follows_the_monitor_under_the_window` を追加）/ `make conformance` / `make doc-check` / `make deny` が 0 件。
ui-win32 と adapters-windows は `cargo clippy --target x86_64-pc-windows-gnu -- -D warnings` が 0 件。`make build-windows` で exe を作り、施主の Windows に置いた。

Xvfb では確かめられない（X11 には `DpiChanged` を送る所が無い）。

施主の 2 回目の実測（同日）: モニタをまたぐと `Physical px` の数字が数 px 変わった（Windows が非クライアント領域の再計算で窓を伸縮させる）。`WM_GETDPISCALEDSIZE` で大きさを覚えて「このまま」と答え、`WM_DPICHANGED` で `SetWindowPos` で戻す形にした（ADR 0029 の決定 3）。Windows clippy 0 件、exe を作り直して置いた。

施主の 3 回目の実測（同日）: 物理画素を保ったので `Logical px` の数字が 320 → 228.6 → 266.7 と変わった。枠は選んだ単位の大きさを保つ形に改めた（論理画素なら `rescale` で物理画素を伸縮・ADR 0029 の決定 3）。core 92 件・application 23 件、Windows clippy 0 件、exe を作り直して置いた。

**planned のまま**: 施主の 3 モニタで、125% の主モニタで起動 → 150% / 175% へ動かすと説明文が 1.5 / 1.75 に変わり、`Logical px` の数字は変わらず枠が物理的に伸縮して開発者ツールと合うこと、`Physical px` の数字もまたいでも変わらないこと（ADR 0029）。

## 52. 1.1.0 の配る物（2026-10-20）

#101 / #103 / #105 を順に squash merge し（施主の 3 モニタでの実測が合った後）、#107 で版を 1.1.0 に上げた。main（`33137c5`）で release.yml を `workflow_dispatch`（`publish_from_run` 空・`dry_run` true）で 1 回だけ実行した。

| 実行 | ジョブ | 成果物 |
| --- | --- | --- |
| [39437069290](https://github.com/hideyukiMORI/nene-ruler/actions/runs/39437069290) | build ✅ / msi ✅ / release（skipped・出すときに使う） | `nene-ruler-linux`（669 KB）・`nene-ruler-windows`（499 KB）・`nene-ruler-exe`（550 KB）・`nene-ruler-msi`（935 KB）・`nene-ruler-store-msix`（546 KB） |

`nene-ruler-store-msix` を手元に落として `AppxManifest.xml` を読んだ: `Identity Name="HideyukiMori.NeNeRuler" Version="1.1.0.0"`。`NeNe-Ruler-store.msix`（3.75 MB）と `.sha256` を施主の Windows の `Downloads\nene-ruler-1.1.0\` に置いた。

**次**: 施主が Partner Center に出す → 認定の後、同じ実行を `publish_from_run=39437069290` で dry run → 本番（ADR 0025 / 0026）。成果物の保存期間は 90 日（2027-01-18 ごろまで）。
