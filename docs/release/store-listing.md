# Microsoft Store の掲載文と年齢区分の根拠 — NeNe Ruler

> Issue #84 / ADR 0026。Partner Center への入力は施主が行う。ここは入力する文面の正本である。
> 形は NeNe Loupe の `docs/release/store-listing.md` に合わせた。入力欄の上限は Loupe が 2026-10-03 に確かめた値を使う
> （製品の機能は最大 20 件、キーワードは最大 7 個・各 40 文字以内、短い説明は 270 文字以下を推奨、`runFullTrust` の理由は 500 文字まで）。
> 🔴 **Partner Center の画面の操作は、施主の画面を見ずに言い切らない**（Loupe で案内が 3 か所違った）。

掲載の言語は英語と日本語の 2 つ（施主の決定・2026-10-18）。アプリの表示は英語だけである。
各文は README・SPECIFICATION・実装と突き合わせた。できないことは書いていない。

## 英語（en-US）

### Product name

```text
NeNe Ruler
```

### Short description

```text
A tiny transparent screen ruler. Lay a frame with ticks over anything on your screen and read its width and height in physical pixels.
```

### Description

```text
NeNe Ruler is a tiny screen ruler for Windows. It is a transparent frame with tick marks on all four edges; everything inside the frame stays visible and clickable, so you can lay it over any window and read the size in physical pixels.

Drag a line or its ticks to move the frame. Drag the bracket at the top-left or bottom-right corner to resize it. The width and height are written on the frame; click a number to copy it.

• Physical pixels. Ticks every 5, 10 and 50 px, with numbers on the top and left edges.
• Click a number to copy it, for example "400 px".
• Nudge with the arrow keys: 1 px, or 10 px with Shift.
• Small frames stay readable: when the numbers no longer fit, the ticks give way to the size in the middle, down to 16 × 16 px.
• One ink color of your choice: 8 swatches, a color picker or a hex code.
• Always on top, with a switch to turn it off.
• Sharp on high-DPI displays.
• Private. No network connections, no account, no telemetry. The only thing saved is three settings in a local file.

Good to know

• The app is in English.
• Press Esc to close the frame, or use Quit in the settings window.
• Windows 10 version 2004 or later, 64-bit. Tested on Windows 11.
• Open source under the MIT License: https://github.com/hideyukiMORI/nene-ruler
```

### Product features

```text
Transparent frame with ticks on all four edges
Width and height in physical pixels
Move by dragging a line, resize by dragging a corner bracket
Click a number to copy it
Arrow keys nudge by 1 px, Shift by 10 px
Ink color from swatches, a color picker or a hex code
Always on top, switchable
No network connections, no account, no telemetry
```

### Search terms

```text
screen ruler
pixel ruler
measure
pixels
ruler
design tool
on-screen ruler
```

### Screenshot captions

画像は `packaging/windows/capture-store-images.ps1` が施主の Windows で撮った 1920×1080（2026-10-18・版 1.0.0・既定の設定）。背景はダークモードの配色の架空の画面（施主の希望）で、施主のデスクトップは写っていない。

| 画像 | Caption |
| --- | --- |
| `docs/images/store/store-1-measure-card.png` | `Lay the frame over anything on your screen and read its width and height in physical pixels.` |
| `docs/images/store/store-2-measure-field.png` | `Small frames show the size in the middle. Click a number to copy it.` |
| `docs/images/store/store-3-settings.png` | `Settings: ink color from swatches, a picker or a hex code, dark or light look, always on top.` |

## 日本語（ja-JP）

### 製品名

```text
NeNe Ruler
```

### 短い説明

```text
小さな透明の画面定規。目盛の付いた枠を画面の上のものに重ね、幅と高さを物理ピクセルで読めます。
```

### 説明

```text
NeNe Ruler は、Windows 用の小さな画面定規です。四辺に目盛の付いた透明な枠で、枠の内側は見えたまま・クリックできたままなので、どの窓の上にも重ねて大きさを物理ピクセルで読めます。

線か目盛をドラッグすると枠が動きます。左上か右下の角の鉤括弧をドラッグすると大きさが変わります。幅と高さは枠に表示され、数字をクリックするとコピーできます。

• 物理ピクセルで測ります。目盛は 5・10・50 px ごとで、上辺と左辺に数字が付きます。
• 数字をクリックするとコピーできます（例: 「400 px」）。
• 矢印キーで 1 px、Shift と一緒なら 10 px 動かせます。
• 小さな枠でも読めます。数字が収まらなくなると目盛を消して中央に大きさを出し、16 × 16 px まで縮められます。
• インクの色は 1 色を自由に。8 色の見本・カラーピッカー・16 進のコードから選べます。
• 常に最前面。スイッチで切り替えられます。
• 高 DPI の表示でもくっきり表示します。
• プライバシー。ネットワークに接続せず、アカウントも利用状況の送信もありません。保存するのは 3 つの設定だけで、端末の中のファイルに置きます。

ご注意

• アプリの表示は英語です。
• Esc で枠を閉じます。設定画面の Quit でも終了できます。
• Windows 10 version 2004 以降（64 bit）。Windows 11 で確かめています。
• MIT License のオープンソースです: https://github.com/hideyukiMORI/nene-ruler
```

### 製品の特長

```text
四辺に目盛の付いた透明な枠
幅と高さを物理ピクセルで表示
線のドラッグで移動、角の鉤括弧のドラッグで伸縮
数字をクリックしてコピー
矢印キーで 1 px、Shift と一緒に 10 px 移動
インクの色は見本・カラーピッカー・16 進のコードから
常に最前面（切り替え可）
ネットワーク接続なし、アカウントなし、利用状況の送信なし
```

### 検索語

```text
画面定規
ものさし
ルーラー
ピクセル
採寸
デザイン
定規
```

### スクリーンショットの説明

| 画像 | 説明 |
| --- | --- |
| `docs/images/store/store-1-measure-card.png` | `画面の上のものに枠を重ね、幅と高さを物理ピクセルで読めます。` |
| `docs/images/store/store-2-measure-field.png` | `小さな枠では中央に大きさを表示。数字をクリックするとコピーできます。` |
| `docs/images/store/store-3-settings.png` | `設定: インクの色（見本・ピッカー・16 進）、ダーク／ライト、常に最前面。` |

画像は 3 枚とも英語の表示で、言語ごとに作り分けていない（Loupe と同じ）。

## そのほかの入力

| 項目 | 入れる内容 | 根拠 |
| --- | --- | --- |
| カテゴリ | 「ユーティリティとツール」を提案する。施主が選ぶ | 画面を測る道具であり、開発者専用ではない（Loupe と同じ） |
| 価格 | 無料 | 施主の決定（無料配布） |
| プライバシーポリシーの URL | `https://github.com/hideyukiMORI/nene-ruler/blob/main/PRIVACY.md` | `PRIVACY.md`（Issue #82） |
| サポートの連絡先 | `https://github.com/hideyukiMORI/nene-ruler/issues` | `PRIVACY.md` の連絡先と同じ |
| Web サイト | `https://github.com/hideyukiMORI/nene-ruler` | |
| 著作権 | `© 2026 Hideyuki Mori` | LICENSE の年と発行者の表示名 |
| `runFullTrust` が必要な理由・認定の注意書き | 下の「申請オプションに貼る文面」 | 施主の確認待ち |
| 公開の保留オプション | 「認定されたらすぐに公開する」 | GitHub の `v1.0.0` は認定の後に、提出したのと同じ実行の成果物で出す（ADR 0026） |
| デバイス ファミリ | 「Windows 10/11 Desktop」だけ | 包んだ Win32 のアプリ |
| アプリ タイル アイコン | 300×300 の PNG（`store-logo-300.png`） | release.yml の成果物 `nene-ruler-store-msix` に入る（build.rs が core の絵から作る・Issue #88） |

## 申請オプションに貼る文面

どの段落も 1 行で、途中に改行を入れていない。端末の表示からではなく、このファイルをエディタで開いてコピーすること
（端末からコピーすると、折り返しの位置に改行や空白が入る・Loupe の記録）。

### 認定の注意書き（Notes for certification）

Loupe では、入力欄は「申請オプション」のページには無く、そのページの「認定の注意書き」の説明文にある「追加のテスト情報」のリンクから開く別のページにあった（2026-10-03）。

```text
NeNe Ruler is a transparent on-screen ruler. On launch it shows a frame with tick marks on all four edges; the inside of the frame is intentionally transparent and click-through, so that the user can measure what is underneath. Only the lines, ticks, numbers, the two corner brackets and the gear respond to the mouse. This is by design, not a rendering failure.
To test: launch the app. Drag a line or its ticks to move the frame; drag the bracket at the top-left or bottom-right corner to resize it; the width and height update in pixels. Click a number to copy it to the clipboard. The arrow keys move the frame by 1 px (10 px with Shift). Click the gear at the bottom-right to open the settings (ink color, dark/light look, always on top, Quit). Press Esc to close the frame. No account, sign-in or network is needed.
```

### `runFullTrust` が必要な理由

入力欄の上限は 500 文字（Loupe の実測）。下の文面は 450 文字。

```text
NeNe Ruler is an existing Win32 desktop app (Rust, no UWP components) packaged with MSIX. runFullTrust is required because the packaged executable is a classic Win32 process (EntryPoint "Windows.FullTrustApplication"). It draws a transparent layered window with Direct2D, reads the screen size and DPI, writes the measured size to the clipboard when the user clicks it, and stores three settings in a local text file. It makes no network connections.
```

## 年齢区分（IARC）のアンケートに答えるための事実

アンケートの設問そのものは Partner Center の画面で出る。ここには、答えの根拠になる事実だけを置く。

| 問われそうなこと | 事実 | 根拠 |
| --- | --- | --- |
| 暴力・性的な内容・恐怖・賭博・薬物・粗野な言葉 | 無い。アプリ自身は線と目盛と数字だけを描く | `SPECIFICATION.md`、実装 |
| 利用者同士のやり取り（チャット・投稿・共有） | 無い | 通信をしない（下） |
| インターネットへの接続 | しない。製品のコードに通信の処理は無い（exe は Rust の標準ライブラリのため `WS2_32` を読み込むが呼ばない） | `PRIVACY.md` |
| 個人情報の収集・第三者への提供 | しない | `PRIVACY.md` |
| 位置情報 | 使わない | 実装に該当の API が無い |
| アプリ内の購入・デジタル商品の販売・広告 | 無い | 実装に該当の機能が無い |
| Web の内容を制限なく表示する機能（ブラウザ） | 無い | 実装に該当の機能が無い |

Loupe は、アプリの種類を「その他のすべてのアプリの種類」にして設問をすべて「いいえ」と答え、IARC 3+ になった（2026-10-03）。

**答え直す条件:** 更新で上の表の事実が変わり、アンケートの答えが変わるときは、アンケートを答え直して区分を取り直す（Loupe が記録した IARC の通知の条項）。
通信・利用者同士のやり取り・購入や広告・Web の内容の表示のどれかを足す変更が、その合図である。

## プライバシーポリシーの設問

Loupe では「個人情報へのアクセス、収集、または送信を行いますか」に「はい」と答えて URL を入れた（画面の画素を読むことと、規約 10.5.1 のため）。
Ruler は画面の画素を読まない。答えは施主の画面の設問の文言を読んでから決める。URL は上の表のもの。

## 2026-10-18 の入力で分かったこと（Submission 1）

施主が Partner Center で申請を入力し、提出した。画面の文言は施主が貼ったものを読んだ（リナは画面を直接見ていない）。

| 項目 | 結果 |
| --- | --- |
| 提出した MSIX | `NeNe-Ruler-store.msix`（main `6f402f9` の release.yml の実行 39109585664 の成果物）。`Validated` |
| 価格と提供の状況 | 世界中のすべての市場・一般ユーザー・検索可能・できるだけ早く・$0 |
| プロパティ | カテゴリは「ユーティリティ & ツール」。個人情報の設問は「いいえ」と答えたが、**この申請ではプライバシー ポリシーが必須と表示され**、URL（`PRIVACY.md`）を入れた（`runFullTrust` の申請だからとみられる・理由は画面に出ていない）。サポートは Web サイトと Issues の URL。システム要件はマウスを最小要件に |
| 製品の公表 | 既定の 3 つ（代替ドライブへのインストール・OneDrive へのバックアップ・録画とブロードキャスト）のまま |
| 年齢区分 | 「その他のすべてのアプリの種類」・設問はすべて「いいえ」。IARC 3+ / Microsoft 3+ / ESRB 全年齢対象 / PEGI 3 / USK 0 / PCBP 0+ / DJCTQ L / CCC TE（IARC 10.3）。評価 ID は「保留中」（公開後に発行）。最後に IARC の使用条件への同意のチェックが要る |
| パッケージ | Windows 10/11 Desktop だけ。「将来のデバイス ファミリを Microsoft が決める」は外した |
| 登録情報 | 英語と日本語。日本語は「追加言語の管理」から足す。英語にスクリーンショットが無いうちは、英語の行に「完了」が付かなかった |
| 申請オプション | 認定されたらすぐに公開。`runFullTrust` の理由と、「追加のテスト情報」のページに認定の注意書き |
| 概要の表示 | 価格と年齢区分には、入力を終えても「完了」が付かない（Loupe と同じ）。「送信して認定を受ける」が押せるかで判断する |
| 提出 | 2026-10-18 18:22 ごろ。状態は「認定中」（前処理中）。所要時間は「通常数時間、最大 3 営業日」と表示 |
