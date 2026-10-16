# ADR 0007 — CLIPBOARD は adapters-x11 が専用の接続と 1 本のスレッドで所有する

- 状態: 受理
- 日付: 2026-10-16
- Issue: #8
- 影響する規則: ARC-002 / ARC-003 / ARC-005 / ARC-007 / ARC-010 / RS-005 / RS-015。仕様 FR-007 / FR-014

## 文脈

FR-007 は、幅・高さの数字をクリックしたら表示しているその文字列を CLIPBOARD へコピーし、成功も失敗も短く示すことを求める。
X11 の CLIPBOARD は「所有」であって「書き込み」ではない。所有者は、他のアプリが貼り付けるたびに届く `SelectionRequest` に
`TARGETS` / `UTF8_STRING` / `STRING` で答え続けなければならず、所有者が消えるとクリップボードも消える。
つまり誰かが**待ち受け**を持たなければならない。

ADR 0002 は「adapters-x11 がクリップボードの所有を持ち、ui-x11 とは別の接続を使う」と決めている。
ARC-007 は、スレッドを持ってよいのは adapters-x11 だけと決めている（CNF-012 の除外）。
DEVELOPMENT_WORKFLOW 第 4 節は、並行性・スレッドモデルの判断に ADR を求めている。

## 決定

**adapters-x11 の `X11Clipboard` が専用の接続と不可視の窓で CLIPBOARD を所有し、1 本のスレッドが `SelectionRequest` に答え続ける。
コピーの文字列は `Mutex` で共有し、コピーの要求は呼び出し側のスレッドで `SetSelectionOwner` → `GetSelectionOwner` で確かめる。**

1. **接続**: `X11Clipboard::connect()` が `x11rb::connect` で専用の接続を開き、1×1 の `InputOnly` の窓を作る。ui-x11 の接続とは独立（ADR 0002 の 1）
2. **スレッド**: 接続は `Arc` で共有し、1 本のスレッドが `wait_for_event` で待ち受ける。`SelectionRequest` には `SelectionAnswer`（純関数）が決めた答えを書く。`SelectionClear` は何もしない（他のアプリが所有を取った）。プロセスが終わればスレッドも終わる
3. **答え**: `TARGETS` → `ATOM` の列 `[TARGETS, UTF8_STRING, STRING]`。`UTF8_STRING` / `STRING` / `TEXT` → その文字列。それ以外 → `property = NONE` の `SelectionNotify`（拒否）。`property` が `NONE` の古い要求には `target` を `property` として使う（ICCCM）
4. **コピー**: `ClipboardPort::copy(text)` は文字列を `Mutex` に置き、`SetSelectionOwner(CLIPBOARD, CURRENT_TIME)` を送り、`GetSelectionOwner` で自分になったことを確かめる。ならなければ `ClipboardFailure::Rejected`。接続やスレッドが死んでいれば `Unavailable`
5. **示し方**（FR-007 の「短く」）: application が `CopyMark`（軸と成否）を持ち、ui-x11 は成功なら数字を反転（インクの箱に透明の文字）、失敗なら取り消し線で描く。印は**ポインタが数字から離れたら消す**。時間で消す経路は作らない（タイマーは値の正本になれない・ARC-007）
6. **PRIMARY** は所有しない。貼り付け側（読む側）も作らない

## 強制

- **active**: スレッドと `std::thread` の名前が adapters-x11 の外に出ないこと（CNF-012）。中核が到達できないこと（CNF-007）
- **active**: `SelectionAnswer` の単体テスト（要求ごとに答えを決める純関数）。X 無しで走る
- **planned**: 実際の貼り付け（`xclip -o -selection clipboard`）は Xvfb の環境検証。gate-proofs に記録する（QLT-013）

## 結果

得られるもの:

- 層が守られる。ui-x11 は「数字が押された」を意図にするだけで、X の選択の手順を知らない（ARC-011）
- 待ち受けが ui-x11 のイベントループから独立するので、モーダルやドラッグの処理とクリップボードの応答が互いを待たない
- 可変性はスレッドと `Mutex` の 2 つだけで、どちらも adapters-x11 の区画に閉じる（ARC-005）

失うもの・残る穴:

- 製品が終了するとクリップボードの中身も消える（X11 の所有の性質。クリップボードマネージャがあれば残る）
- スレッドが panic で死ぬと、その後のコピーは `Unavailable` になる。panic は RS-005 で塞いでいるが、X の接続の切断は検出して `Unavailable` に畳む
- `Mutex` の毒（poison）は `Unavailable` に畳む。`unwrap` は書かない（RS-005）
- 大きな文字列の `INCR` 転送は実装しない。`400 px` のような短い文字列しか置かない

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| ui-x11 が自分の接続で所有し、イベントループで `SelectionRequest` に答える | ADR 0002 と ARC-003（クリップボードはアダプタ）に反する。UI がプラットフォームの手順を持つことになり、ARC-011 にも反する |
| 接続を 1 本にして application のポートに畳む | ui-x11 と adapters-x11 の両方が同じ接続のイベントを読む設計になり、イベントの振り分けが層をまたぐ。ADR 0002 の「共有したくなったら ADR」の判断としても、今は共有する理由が無い |
| `xclip` / `xsel` を子プロセスで呼ぶ | 実行時に外部のバイナリへ依存する。無い環境で黙って失敗する |
| クリップボードのクレート（`arboard` 等） | 依存が増え、Wayland・Windows の経路も抱える（ARC-013）。必要なのは X11 の所有だけ |
| 時間で印を消す | タイマーの値が UI の正本になる（ARC-007）。ポインタの移動で消すほうが決定的で、実装も短い |
