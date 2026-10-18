# ADR 0025 — 配る物は 1 回だけ作り、出すときはその成果物をそのまま添付する

- 状態: 受理
- 日付: 2026-10-18
- Issue: #74
- 影響する規則: QLT-005 / QLT-011 / ARC-013。ADR 0021（Windows 版は gnu）/ 0023 / 0024（アイコン）

## 文脈

v0.1.0 で Windows 版と Linux 版を GitHub Release で配る（施主決定 2026-10-17 / 10-18）。配る物は Windows の MSI とポータブル zip、Linux の `.deb` と zip、それぞれの sha256。

施主は推しの「exe は Linux のランナーで今と同じ gnu で作り、MSI だけ Windows のランナーで WiX で包む」を選んだ（2026-10-18）。手元で試している exe と同じ作り方の物を配れ、アイコンを埋める build.rs もそのまま使える。

NeNe Clock は、タグで作り直す形から「1 回だけ作り、出すときはその実行の成果物を作り直さずに添付する」形へ移った（Clock の ADR 0022）。試した物と配る物が同じになる。

## 決定

**`make dist`（`packaging/dist.sh`）が Linux の上で `.deb`・Linux の zip・Windows のポータブル zip・MSI の材料の exe・sha256 を作る。`packaging/windows/build-msi.ps1` が Windows の上で、同じ実行の exe を WiX 5 で MSI に包む。`.github/workflows/release.yml` は手順を呼んで成果物を残すだけで、出すときは指定した実行の成果物を確かめてそのまま Release に添付し、タグもそのとき打つ。**

1. 配る物の名前は版を含まない（`NeNe-Ruler-Setup.msi`・`NeNe-Ruler-windows-portable.zip`・`nene-ruler_amd64.deb`・`nene-ruler-linux-x86_64.zip`）。版は Release とタグが持つ（Clock と同じ）
2. 版は `Cargo.toml` の `[workspace.package] version` だけから読む（FR-011）。タグは `v<版>`
3. `.deb` の `Depends` は `dpkg-shlibdeps` が実物から決める。Maintainer は Clock と同じ
4. MSI は per-machine（Program Files・UAC の確認あり）・スタートメニューのショートカット・メジャーアップグレード。`UpgradeCode` は製品の間ずっと変えない
5. WiX は 5.0.2 に固定する。6 以降は利用条件（Open Source Maintenance Fee）が変わった可能性があり、確かめていないので上げる前に確かめる
6. release.yml は、配布の結線（release.yml・`packaging/`・Makefile・build.rs・Cargo.toml）に触る PR と、手動の実行で配る物を作る。出す（`publish_from_run`）ときは、その実行が成功した Release の実行で、commit が main に載っていて、sha256 が一致し、同じタグの Release が無いことを確かめる。既定は dry run
7. Actions は commit で固定する（checkout と upload-artifact は Clock と同じ v7.0.1。check.yml の checkout は Node.js 20 の版で非推奨の警告が出るので別の Issue で上げる）。同じ実行の中の受け渡しと出すときの取り出しは `gh run download` で行い、download-artifact は使わない

## 強制

- **active**: 手元の `make dist` で 4 つと sha256 ができ、`dpkg-deb -I / -c` と zip の中身が正しい（gate-proofs 第 42 節）
- **active**: PR の上で release.yml の build と msi が通る（同節）
- **planned**: 施主の機械で MSI と `.deb` を入れて動く（QLT-013）
- **planned**: `publish_from_run` の確かめ（最初の v0.1.0 で dry run から実行する）

## 結果

- 試した実行の成果物がそのまま配られる。タグを打ってから作り直す形の「試した物と違う物が出る」が起きない
- 配る手順は手元の `make dist` と同じなので、CI の外でも同じ物が作れる（MSI を除く）
- MSI だけ Windows のランナーが要る。コード署名はしない（SmartScreen の警告が出る）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| タグで起動して作る | 試した物と配る物が別の実行になる。Clock が移った理由と同じ |
| exe を Windows のランナーで MSVC で作る | 手元と作り方が変わり、build.rs の資源の経路（windres）も別に要る。施主は gnu を選んだ |
| MSI も Linux の上で作る（`wixl`） | WiX の定義の書き方が別になり、機能も限られる |
| per-user の MSI | Windows Installer の検査（ICE）に合わせる登録が増える。Store の MSIX を後で出すので、MSI は素直な per-machine にする |
| download-artifact | 最新の版が出たばかり（2026-10-07）。`gh run download` で足りる |
