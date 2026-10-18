# ADR 0026 — Microsoft Store を配布経路に足し、配る exe をそのまま MSIX に包む

- 状態: 受理
- 日付: 2026-10-18
- Issue: #80
- 影響する規則: QLT-005 / ARC-001 / ARC-013。ADR 0023 / 0024（アイコン）・ADR 0025（配る物は 1 回だけ作る）に Store の経路を足す

## 文脈

施主が NeNe Ruler を Microsoft Store に出すと決めた（2026-10-18・最終の目標は 2026-10-17 の申告）。Store に MSIX で出すと、認定のあと Store が Microsoft の証明書で署名し直すので、自前の証明書は要らない。Store から入れた場合は SmartScreen の警告も出ない。一次資料での裏取りは NeNe Loupe（ADR 0008・2026-10-03）が行い、Loupe 1.0.0 は 2026-10-04 にこの経路で差し戻しなく公開された。Clock（ADR 0020）も同じ形で続いた。Ruler では調べ直していない。

施主が Partner Center で `NeNe Ruler` を予約し、身元が決まった（2026-10-18）。発行者と表示名は Loupe・Clock と同じアカウントの値。

## 決定

**Store（MSIX）を 5 つ目の配布物として足す。MSIX は、ポータブル zip で配る exe をそのまま包む。ロゴは唯一の build.rs が core の絵から作る。手順は `packaging/windows/build-msix.ps1` の 1 つで、release.yml の Windows のジョブがそれを呼ぶ。**

1. 身元（`identityName`・`publisher`・`publisherDisplayName`・`storeId`）は `packaging/windows/store-identity.json` の 1 か所に置く。Partner Center の「製品 ID」の値と一字一句同じにする。秘密ではない（公開されるパッケージに入る値）
2. MSIX の中の exe は、ポータブル zip の中の exe と SHA-256 が同じでなければ失敗する。包んだ物を開き直して、もう一度確かめる
3. ロゴは NeNe Loupe の提出で通った 25 枚（`Square44x44Logo`・`Square150x150Logo`・`StoreLogo` の倍率 100〜400% と、`targetsize` 16〜256 の地なし 2 つ）。`crates/app/build.rs` が Windows のターゲットで `OUT_DIR/msix/Assets` に書き、`make dist` が `dist/msix-assets` に拾う。`makepri` で 1 つの `resources.pri` にする
4. 版は `Cargo.toml` の版に `.0` を足す（4 つ目は Store が使う）。先頭が 0 の版は `NeNe-Ruler-store-NOT-SUBMITTABLE.msix` という名前にして警告する
5. マニフェストは `runFullTrust`（Win32 のデスクトップアプリ）・`MinVersion` 10.0.19041.0（Windows 10 version 2004）・`MaxVersionTested` 10.0.26100.0。表示は英語（`en-us`）
6. 署名しない。鍵と証明書はリポジトリに置かない。手元の検証（WACK）用の署名は、施主の機械の証明書ストアの中だけで行う
7. MSIX は GitHub Release に添付しない（Store が署名し直した物が配られる）。`publish_from_run` は、提出できる MSIX（`NeNe-Ruler-store.msix`）の無い実行を出さない。Store と GitHub の版をずらさない
8. Windows SDK の道具（`makeappx` / `makepri`）は PATH に無いので、`10.0.26100.0` のフォルダを名指しする（Clock と同じ）。無ければ `is not where it is pinned` で落ちる

## 強制

- **active**: build-msix.ps1 が exe の一致・ロゴの枚数（25）を確かめて落ちる。PR の上の release.yml で MSIX ができる（gate-proofs 第 44 節）
- **planned**: Windows App Certification Kit（施主の機械・管理者権限）。Store の認定は申請しないと分からない（不能）

## 結果

- 署名しない方針のまま、署名された配布物を 1 つ持てる。更新は Store が配る
- 設定の保存先は、Store 版ではパッケージ専用の場所へ振り替えられることがある（Loupe の実測では、本物の場所のフォルダの有無で分かれた）。プライバシーポリシーと README に書く
- プライバシーポリシー・掲載文・スクリーンショット・年齢区分の回答という、製品の外の成果物が増える
- 版の番号に Store の制約（先頭は 0 にできない）が掛かる。施主は 1.0.0 で出すと決めた（2026-10-18）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| MSI を Store に出す | 自前の署名と版ごとの HTTPS のホスティングが要る |
| Store の外で MSIX を配る・winget | 自前の署名が要る |
| MSIX 用に exe を作り直す | 配る中身が経路ごとに別物になる |
| ロゴを画像ファイルで置く | 絵の正本が 2 つになる（ADR 0020 の却下理由と同じ） |
