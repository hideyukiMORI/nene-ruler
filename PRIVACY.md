# Privacy Policy — NeNe Ruler

Last updated: 2026-10-18 · Publisher: Hideyuki Mori

**NeNe Ruler does not collect, store or transmit any personal information. It never connects to
a network.**

日本語は[このページの後半](#プライバシーポリシー日本語)にあります。

## What the application does with data

| Data | What happens | Where it goes |
| --- | --- | --- |
| Screen size and scale | The application reads the size of your screen and its scale (DPI) to limit how large the frame can grow and to choose 1x or 2x drawing. It does not read what is on the screen. | Kept in memory only. Never written to disk. Never sent anywhere. |
| Clipboard | When you click a number on the frame, the application places that text (for example `400 × 240 px`) on the clipboard. It does not read the clipboard. | Your clipboard, on your device. |
| Settings | Three choices are saved: the ink colour, the look of the settings window (dark or light) and always-on-top. | One small text file on your device (see below). |

The application has no accounts, no telemetry, no analytics, no advertising, no crash reporting,
no automatic update check of its own, and no third-party services.

## The settings file

The settings file contains only a schema version and the three choices above. It contains
nothing about you or about what you measured.

- Windows (installer and portable ZIP): `%APPDATA%\nene-ruler\settings.v1.txt`.
- Linux: `$XDG_CONFIG_HOME/nene-ruler/settings.v1.txt`, or `~/.config/nene-ruler/settings.v1.txt`.
- Microsoft Store version: Windows may keep the file inside the application's own package
  storage instead of the location above, and removes it when you uninstall the application.

The file stays on your device until you delete it or uninstall the application.

## Network

The application makes no network connections. Its source code contains no networking calls.
On Windows, the executable lists the Windows networking library (`WS2_32`) among its imports
because the Rust standard library it is built with includes networking support; the application
never calls it. On Linux, the application talks to your display server (X11) to draw its
windows, as every graphical application does.

If you install the application from the Microsoft Store, the Store itself — not this
application — handles download, installation and updates. That is covered by the
[Microsoft Privacy Statement](https://privacy.microsoft.com/privacystatement).

## Changes and contact

Changes to this policy are made in this file, and its history is public in the repository.
Questions: open an issue at <https://github.com/hideyukiMORI/nene-ruler/issues>.

---

# プライバシーポリシー（日本語）

最終更新: 2026-10-18 · 発行者: Hideyuki Mori

**NeNe Ruler は個人情報を収集・保存・送信しません。ネットワークには一切接続しません。**

## アプリが扱うデータ

| データ | 何をするか | どこへ行くか |
| --- | --- | --- |
| 画面の大きさと拡大率 | 枠の大きさの上限と、1x / 2x の描き分けのために、画面の大きさと拡大率（DPI）を読みます。画面に映っているものは読みません。 | メモリの中だけ。ディスクに書きません。どこにも送りません。 |
| クリップボード | 枠の数字をクリックすると、その文字列（例: `400 × 240 px`）をクリップボードに置きます。クリップボードの中身は読みません。 | お使いの端末のクリップボード。 |
| 設定 | インクの色・設定画面の見え（ダーク／ライト）・常に最前面の 3 つを保存します。 | お使いの端末の小さなテキストファイル 1 つ（下記）。 |

アカウント、利用状況の送信、解析、広告、クラッシュ報告、アプリ独自の更新確認、第三者のサービスはありません。

## 設定ファイル

設定ファイルに入るのは、形式の版と上の 3 つの選択だけです。利用者や測ったものについての情報は入りません。

- Windows（インストーラー版と ZIP 版）: `%APPDATA%\nene-ruler\settings.v1.txt`
- Linux: `$XDG_CONFIG_HOME/nene-ruler/settings.v1.txt`、無ければ `~/.config/nene-ruler/settings.v1.txt`
- Microsoft Store 版: Windows が上の場所の代わりにアプリ専用の保存場所に置くことがあり、その場合はアンインストールすると消えます。

削除するかアンインストールするまで、ファイルは端末に残ります。

## ネットワーク

アプリはネットワークに接続しません。ソースコードに通信の処理はありません。
Windows では、実行ファイルが読み込むライブラリの一覧に Windows の通信ライブラリ（`WS2_32`）が入っています。
これはアプリを作るのに使っている Rust の標準ライブラリが通信の機能を含むためで、アプリがそれを呼ぶことはありません。
Linux では、窓を描くために画面のサーバー（X11）と通信します。これはどの画面のアプリも行うことです。

Microsoft Store から入れた場合、ダウンロード・インストール・更新を行うのは Store であり、このアプリではありません。
その部分は [Microsoft のプライバシーに関する声明](https://privacy.microsoft.com/privacystatement)の対象です。

## 変更と連絡先

このポリシーの変更はこのファイルに対して行い、履歴はリポジトリで公開されます。
質問は <https://github.com/hideyukiMORI/nene-ruler/issues> へ Issue としてお寄せください。
