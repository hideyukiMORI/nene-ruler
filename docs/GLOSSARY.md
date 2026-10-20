# 用語集 — NeNe Ruler

> Status: normative（規範）/ 2026-09-27 初版
> ここに載っている語は、コード・ドキュメント・Issue で**同じ意味**で使う。同義語を発明しない。

| 語 | 意味 | 型／場所 |
| --- | --- | --- |
| 結果（outcome） | 期待される成功・失敗を表す閉じた型 | `Result<T, *Failure>`（core / application） |
| 拒否理由（failure） | 結果に添える閉じた理由の集合 | `*Failure`（`enum`） |
| ポート（port） | 中核がプラットフォームを使うための型のある窓口 | `*Port`（application の `trait`） |
| アダプタ（adapter） | 外部入力ポートの実装。物理寸法・クリップボード・設定の保存・現在時刻を中核へ渡す | `RandrScreenMetrics` / `FileSettingsStore` / `X11Clipboard`（adapters-x11） |
| ポートの束 | 合成ルートが中核へ渡すポートの組。引数を増やさずにポートを足す | `Ports`（application） |
| コピーの印 | 数字をコピーした結果。成功は反転、失敗は取り消し線。ポインタが離れるまで | `CopyMark`（application）・ADR 0007 |
| 矢印の一歩 | 矢印で枠を動かす向きと歩幅（1 px、Shift で 10 px） | `Nudge` / `NudgeDirection`（core） |
| 通知 | 利用者に見せるべき降格。枠には不透明への降格と物理寸法の欠如、モーダルには設定の失敗と物理寸法の欠如 | `FrameNotice` / `FrameNotices` / `SettingsNotices`（application） |
| 選択の答え | `SelectionRequest` の target ごとに何を書くかを決める純関数 | `SelectionAnswer`（adapters-x11） |
| 設定ファイル | 版 1 の `key = value` テキスト。直列化の型は core の型と分ける | `SettingsFileV1`（adapters-x11）・ADR 0006 |
| 合成ルート | 依存を結ぶ唯一の場所。端末と終了コードを持つ | `crates/app`（`main`） |
| 反映（render） | 決まった値を画素へ写す操作。UI 状態を変えてよい唯一の場所 | `render*`（ui-x11） |
| 隔離区画 | 可変性を許した唯一の場所 | ARC-005 の表 |
| 規約検査（conformance） | NeNe Ruler 固有の自作ゲート | `xtask`（CNF-NNN） |
| 抑制 | `#[expect(lint, reason = "<規則 ID>: …")]`。理由付きで、発火しなくなれば落ちる | RS-019 |
| 機械強制の状態 | active / planned / 不能 / 不採用 | `docs/QUALITY_GATES.md` の強制マトリクス |
| negative proof | ゲートが意図した規則で落ちることの実測 | `make prove` / `docs/quality/gate-proofs.md` |
| 長さ | 枠の一辺が測っている物理画素の数。幅と高さの両方がこれ | `Length`（core） |
| 枠 | 物差しそのもの。左上を原点とする幅と高さの組 | `FrameSize`（core） |
| 軸 | 横（幅）か縦（高さ）かの閉じた集合。目盛と数字がどちらの辺のものかを言う | `Axis`（core） |
| 画面の大きさ | RandR が返す画面の画素数と横のミリ寸法。伸縮の上限と UI の倍率（dpi）に使う。長さの換算には使わない（ADR 0012） | `ScreenMetrics`（core） |
| 単位 | 長さを綴る単位。物理画素（`px`）と論理画素（`logical px`）の閉じた集合。設定の一部（FR-008） | `Unit`（core）・ADR 0027 |
| 論理画素 | 物理画素 ÷（デスクトップの dpi / 96）。ブラウザの開発者ツールの CSS px、Windows の DIP、Android の dp と同じ数。枠の字の大きさはこれで定める（ADR 0028） | `DevicePixelRatio`（core）・ADR 0027 / 0028 |
| 換算 | 選んだ単位と、いまの環境の比の組。長さを綴るときに 10 分の 1 の単位の整数にする | `Measure`（core） |
| 表示文字列 | 長さを `400 px` / `266.7 logical px` の形に綴った文字列。表示とクリップボードで同じものを使う | `LengthText`（core） |
| 数字の種類 | 枠に出す長さの数字。幅・高さ・両方（`W × H px`）の閉じた集合。表示・当たり判定・コピーの単位 | `LengthLabel`（core）・ADR 0017 |
| 詰めた表示 | 数字が目盛を避けた内側に収まらない小さな枠の描き方。目盛と通知を消し、中央に `W × H px` | `FrameDetail::Compact`（core）・ADR 0017 |
| 線を消した表示 | 詰めても `W × H px` が枠に収まらない大きさの描き方。線を描かず鉤括弧を細くし、文字を窓の中央に枠の外へはみ出して描く | `FrameDetail::Bare`（core）・ADR 0018 |
| 目盛 | 選んだ単位での値と、物理画素に丸めた位置と、小・中・主の種類（ADR 0028） | `TickMark` / `TickKind`（core） |
| 色 | 不透明な RGB の値型。インク色にもモーダルの配色にもこれ 1 つ。`#RRGGBB` で読み書きする | `Rgb`（core） |
| インク色 | 枠の窓に描くものすべてに使う 1 色。設定の一部 | `RulerSettings::ink`（core） |
| 直接入力 | `#RRGGBB` を打ち込む欄の編集状態。16 進の文字だけを受ける | `HexEntry`（core） |
| アンカー | 左上と右下の鉤括弧。掴んで伸縮する | `Anchor`（core） |
| 当たり判定 | クリックを受ける物理画素の矩形と、そこを掴んだときの意味（移動・伸縮・ギア・コピー） | `HitRegion` / `HitTarget`（core） |
| 透明 | 描いたもの以外のアルファが 0 であること。合成器が無ければ不透明に落ちる | `Transparency`（application） |
| 環境 | 透明と、デスクトップの dpi（X11 は `Xft.dpi`、Windows は主モニタの dpi）の組。合成ルートが起動時に渡す。UI の倍率と、論理画素の比の最初の元（比はその後、窓が乗っているモニタに従う・ADR 0029） | `Environment`（application） |
| 見えないが触れる | アルファ 1/255 の画素。WSLg は描いた画素だけを当たりにするので、当たり判定の帯をこれで下塗りする | `canvas` の `TOUCHABLE_ALPHA`（ui-x11）・ADR 0008 |
| 合成器 | `_NET_WM_CM_S0` を所有し、ARGB の窓を画面に合成するプログラム（Weston・xcompmgr 等） | adapters-x11 / ui-x11 |
| モーダルの見え | 設定モーダルの背景と文字の配色。ダーク・ライトの閉じた集合（「システムに従う」は置かない・FR-010）。枠の窓には効かない | `ModalTheme`（core） |
| ピッカーの色 | カラーピッカーの色相（0.1 度）・彩度・明度（千分率）。モーダルの編集状態が持ち、灰や黒を選んでも色相を保つ | `Hsv`（core）・`ModalSession::picker`（application）・ADR 0016 |
| 配色 | 見えごとの背景・文字・入力欄・枠線・選択の印 | `ModalPalette`（core） |
| 部品 | 設定モーダルで押せるもの。色見本・入力欄・ピッカー（四角と帯）・見え・最前面・閉じる・終了 | `SettingsControl` / `SettingsLayout`（core） |
| 重なり順 | 常に最前面か、通常の重なり順かの閉じた集合 | `WindowLayer`（core） |
| 設定 | インク色・モーダルの見え・重なり順の組。保存される値の正本 | `RulerSettings`（core） |
| 物差しの状態の所有者 | 枠・設定を 1 つ持ち、意図で次の状態へ置き換える | `RulerController`（application） |
| 表示フレーム | 最新の状態から生成する、描くための値の組 | `RulerFrame` / `SettingsFrame`（application） |
| モーダルの編集状態 | モーダルが開いている間の入力欄と、直前の拒否 | `ModalSession`（application） |
| 版 | `Cargo.toml` の版を合成ルートが 1 か所で包んだもの | `ProductVersion`（application） |
| 画素の列 | cairo の `ImageSurface` に描いた ARGB の並び。X へは転送するだけ | ui-x11 |

## 使ってはいけない語

`Manager` / `Helper` / `Util` / `Utils` / `Common` を型名の語尾に使わない（RS-013）。
役割を語る名前が思いつかないときは、その型が 2 つの責務を持っている可能性が高い。
