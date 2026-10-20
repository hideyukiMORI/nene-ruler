# いまのタスク — NeNe Ruler

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-19 時点）。再開の手順は [handover-2026-10-19](handover-2026-10-19.md)。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust 1.98.0 / x11rb 0.14 ＋ cairo-rs 0.22（ADR 0002）。施主決定 2026-09-27 |
| 規範文書・ADR 0001〜0011 | ✅ Issue #1 / #2 / #6 / #8 / #14 / #20 / #26 / #28 |
| ゲート（clippy 二段構え・xtask・cargo-deny・rustfmt・rustdoc）と `make` の個別 target | ✅ Issue #1 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（69 件） | ✅ Issue #1・[gate-proofs](../quality/gate-proofs.md) |
| 仕様の改訂: 透明な 2 次元の枠・四辺に目盛・鉤括弧で伸縮・単色（施主の口述 2026-10-15） | ✅ Issue #2・[ADR 0005](../adr/0005-transparent-frame-with-input-shape.md) |
| 5 クレート（core / application / adapters-x11 / ui-x11 / app） | ✅ Issue #2 |
| 透明の窓・目盛・幅と高さの数字・鉤括弧・ギアの描画。線で移動、鉤括弧で伸縮、Esc で閉じる | ✅ Issue #2・Xvfb で実測（gate-proofs 第 8 節） |
| `make coverage`（region 90%）・`make screenshot` | ✅ Issue #2。QLT-009 は region に改めて active（Issue #26・ADR 0010） |
| ギアで設定モーダル。色見本 8 色と `#RRGGBB` の直接入力、見え（ダーク・ライト）、最前面、版（単位は Issue #30 で外した） | ✅ Issue #4・Xvfb で実測（gate-proofs 第 9 節）。PR #3 の枝の上 |
| 設定の保存 `~/.config/nene-ruler/settings.v1.txt`。読めない版・壊れたファイルはモーダルに通知 | ✅ Issue #6・ADR 0006・Xvfb で再起動を実測（gate-proofs 第 10 節）。PR #5 の枝の上 |
| 数字のクリックで CLIPBOARD へコピー。成功は反転、失敗は取り消し線 | ✅ Issue #8・ADR 0007・Xvfb で `xclip -o` を実測（gate-proofs 第 11 節）。PR #7 の枝の上 |
| 矢印で 1 px、Shift+矢印で 10 px の移動 | ✅ Issue #10・Xvfb で実測（gate-proofs 第 12 節）。PR #9 の枝の上 |
| 不透明への降格と物理寸法の欠如を枠に、設定の失敗と物理寸法の欠如をモーダルに示す（FR-014） | ✅ Issue #12・Xvfb で実測（gate-proofs 第 13 節）。PR #11 の枝の上 |
| WSLg（施主の実機）での目視 | ✅ 2026-10-17・[wslg-2026-10-17](../quality/wslg-2026-10-17.md)・gate-proofs 第 14 節。13 件が成り立ち、5 件が崩れた |
| WSLg への適合: 当たり判定の帯をアルファ 1 で描く・影を消す・Alt+F4 を自分で読む・最前面ができないことを示す | ✅ Issue #14・ADR 0008。施主の再目視待ち。PR #13 の枝の上 |
| 縦書きの高さの数字と通知を、左辺の主目盛の数字の右へ寄せる | ✅ Issue #15。PR #16 の枝の上 |
| HiDPI の倍率 1x / 2x（`Xft.dpi` → RandR の dpi）を枠の飾りとモーダルに掛ける | ✅ Issue #28・ADR 0011・Xvfb で実測（gate-proofs 第 20 節）。施主の WSLg は `Xft.dpi` が無く主画面が 0 mm なので 1x |
| mm / in と最前面の通知を外し、長さは常に px（施主決定 2026-10-17） | ✅ Issue #30・ADR 0012 |
| 単位を戻し、物理画素と論理画素（÷ dpi / 96・ブラウザの開発者ツールの CSS px / Windows の DIP）を切り替える。モーダルに `Unit` の段（`Physical px` / `Logical px`）と換算の説明、設定ファイルに `unit = physical_px` / `logical_px`。枠の数字は `400 px` / `266.7 logical px` | ✅ Issue #100・ADR 0027・Xvfb の 144 dpi と 1x で実測（gate-proofs 第 49 節）。施主の Windows（150%）での突き合わせは planned |
| 字の大きさを論理画素で定め（単位に依らず比で物理画素に・150% で 14 / 17 px）、目盛を選んだ単位で刻む（`Logical px` なら 150% の主目盛は物理 75 px ごとに `50` `100` …）。全部描ける判定を交差判定に改める | ✅ Issue #102・ADR 0028・Xvfb の 144 dpi で両単位を実測（gate-proofs 第 50 節）。施主の Windows（150%）での目視は planned。PR #101 の枝の上 |
| 論理画素の比を主モニタ固定でなく、定規の窓が乗っているモニタの dpi から取り、`WM_DPICHANGED` で取り直す（施主の実測: 主 125%・他 150% / 175% で外れた） | ✅ Issue #104・ADR 0029・application のテストと Windows clippy。施主の 3 モニタでの実測は planned。PR #103 の枝の上 |
| 施主の 3 モニタ（主 125%・150%・175%）で実測: `Logical px` の 320 が 400 / 480 / 560 物理画素で保たれ、ブラウザの開発者ツールと合う。`Physical px` の数字もまたいで変わらない | ✅ 2026-10-20・#101 / #103 / #105 を順に squash merge |
| 版を 1.1.0 に上げ、release.yml を main で 1 回実行して配る物を作った（zip / MSI / deb / MSIX 1.1.0.0） | ✅ Issue #106・PR #107・実行 [39437069290](https://github.com/hideyukiMORI/nene-ruler/actions/runs/39437069290)（gate-proofs 第 52 節）。Store への提出と GitHub Release はこれから |
| Windows 版の土台: 設定ファイルの切り出し・adapters-windows・ui-win32（Direct2D の枠の窓）・unsafe の例外（CNF-014） | ✅ Issue #60・ADR 0021。施主の Windows で枠が描けた。枠の操作 6 項目は施主が確かめた（2026-10-18）。PR #61 |
| Windows の設定モーダル（Direct2D）・枠とモーダルで状態を `RefCell` で共有 | ✅ Issue #62。部品の操作 7 項目は施主が確かめた（2026-10-18）。PR #63 |
| 目盛の帯を移動の当たり判定にする（目盛の上ならどこでも掴める・両 OS） | ✅ Issue #66・ADR 0022・Windows の地図と Xvfb で実測、施主の目視 OK（gate-proofs 第 38 節）。PR #67 |
| アイコンの絵を core に移し、Windows の exe に埋める（唯一の `build.rs`）・X11 も同じ画素 | ✅ Issue #68・ADR 0023・exe の資源と Xvfb で実測、施主の目視 OK（gate-proofs 第 40 節）。PR #71 |
| `make prove` を直した（複数行の members・許可ライセンスの置き換え） | ✅ Issue #69・26 件成功（gate-proofs 第 39 節）。PR #70 |
| Linux のメニュー用のアイコン（hicolor の PNG・build.rs が core の絵から）・`.desktop`・X11 の `WM_CLASS` | ✅ Issue #72・ADR 0024（gate-proofs 第 41 節）。`.deb` のメニューの見えは施主が確かめた。PR #73 |
| 配布の仕組み（`make dist`・WiX の MSI・作った成果物をそのまま出す release.yml） | ✅ Issue #74・ADR 0025・手元と PR の上の Actions で実測、施主が MSI・ポータブル zip・`.deb` を試して OK（gate-proofs 第 42 節）。PR #75 |
| Microsoft Store（MSIX・身元・ロゴ・PRIVACY・掲載文・画像・版 1.0.0） | ✅ Issue #80 / #82 / #84 / #86 / #88・ADR 0026。2026-10-18 に申請（Issue #90）、2026-10-19 に認定・公開（[Store](https://apps.microsoft.com/detail/9N6JVHKL9LM3)） |
| v0.1.0 | ✅ 2026-10-18・[Release](https://github.com/hideyukiMORI/nene-ruler/releases/tag/v0.1.0)・タグは `6e391cd`。main の実行 39101655811 の成果物をそのまま出した（gate-proofs 第 43 節） |
| v1.0.0 | ✅ 2026-10-19・[Release](https://github.com/hideyukiMORI/nene-ruler/releases/tag/v1.0.0)・タグは `6f402f9`。Store に出したのと同じ実行 39109585664 の成果物をそのまま出した（gate-proofs 第 47 節） |
| CI の必須 check（`make ci`）と commit-msg フック | 配線済み。最初の PR で実行証拠を得るまで planned |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

1.0.0 が 2026-10-19 に Microsoft Store で公開され、同じ日に同じ実行 39109585664 の成果物で GitHub の `v1.0.0` を出した。README の Download に Store のリンクを足し、Store 版の設定の置き場を実測して PRIVACY.md と README に書いた（Issue #96・gate-proofs 第 48 節）。

1. **1.1.0 を出す**（実行 39437069290・作り直さない）: (a) 施主が `NeNe-Ruler-store.msix`（`C:\Users\info\Downloads\nene-ruler-1.1.0\`）を Partner Center に出す。掲載文は `docs/release/store-listing.md`（`color` に直した）を貼り直す。(b) 認定の後、release.yml を `publish_from_run=39437069290` で dry run → `dry_run=false` で本番（タグ `v1.1.0` と GitHub Release が同じ実行の成果物で出る・ADR 0025 / 0026）。(c) README の Status と CLAUDE.md 第 6 節を 1.1.0 に揃える
2. v1.0.0 のリリースノートの先頭に要約を足す（施主に案を見せてから `gh release edit`・v0.1.0 と同じ）。v1.1.0 も同じ形で
3. Issue #76（check.yml の checkout）・#21・#25。Linux の zip は施主の機械でまだ試していない

## 残る穴

- CNF-012 はマクロ・別名・間接呼び出しを見ない。ui-x11 は `std` を使うので字句検査だけが残る
- CNF-005（不変条件を持つ型の単独モジュール）は判定方法が未定。core の型は 1 ファイル 1 型になっている
- 最前面は WSLg では効かない（`_NET_WM_STATE_ABOVE` が無い）。示しもしない。Windows 版で効かせる（ADR 0012）
- WSLg で 1 度だけ、2 回目の起動で Windows 側に絵が出ず無言で終わった。再現待ち（gate-proofs 第 14.3 節）
- 当たり判定の矩形と描いたものの一致はレビュー事項（ADR 0005）
