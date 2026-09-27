# 用語集 — NeNe Ruler

> Status: normative（規範）/ 2026-09-27 初版
> ここに載っている語は、コード・ドキュメント・Issue で**同じ意味**で使う。同義語を発明しない。

| 語 | 意味 | 型／場所 |
| --- | --- | --- |
| 結果（outcome） | 期待される成功・失敗を表す閉じた型 | `Result<T, *Failure>`（core / application） |
| 拒否理由（failure） | 結果に添える閉じた理由の集合 | `*Failure`（`enum`） |
| ポート（port） | 中核がプラットフォームを使うための型のある窓口 | `*Port`（application の `trait`） |
| アダプタ（adapter） | 外部入力ポートの実装。物理寸法・クリップボード・設定の保存・現在時刻を中核へ渡す | `*Adapter`（adapters-x11） |
| 合成ルート | 依存を結ぶ唯一の場所。端末と終了コードを持つ | `crates/app`（`main`） |
| 反映（render） | 決まった値を画素へ写す操作。UI 状態を変えてよい唯一の場所 | `render*`（ui-x11） |
| 隔離区画 | 可変性を許した唯一の場所 | ARC-005 の表 |
| 規約検査（conformance） | NeNe Ruler 固有の自作ゲート | `xtask`（CNF-NNN） |
| 抑制 | `#[expect(lint, reason = "<規則 ID>: …")]`。理由付きで、発火しなくなれば落ちる | RS-019 |
| 機械強制の状態 | active / planned / 不能 / 不採用 | `docs/QUALITY_GATES.md` の強制マトリクス |
| negative proof | ゲートが意図した規則で落ちることの実測 | `make prove` / `docs/quality/gate-proofs.md` |
| 長さ | 物差しが測っている物理画素の数。単位換算の元 | `Length`（core） |
| 向き | 横か縦かの閉じた集合 | `Orientation`（core） |
| 単位 | px / mm / in の閉じた集合。巡回の順もここが正本 | `Unit`（core） |
| 物理寸法 | RandR が返す画面の画素数とミリ寸法。mm / in の換算に使う | `ScreenMetrics`（core） |
| 表示文字列 | 長さを単位に従って綴った文字列。表示とクリップボードで同じものを使う | `LengthText`（core） |
| テーマ | 利用者が選ぶ配色。ダーク・ライト・システムに従うの閉じた集合 | `Theme`（core） |
| 見えの配色 | 実際に描くほうの閉じた集合。ダーク・ライトの 2 つだけ | `ThemeAppearance`（core） |
| 配色表 | 見えの配色ごとの色の対応。X11 にも cairo にも依存しない | `ThemePalette`（core） |
| 重なり順 | 常に最前面か、通常の重なり順かの閉じた集合 | `WindowLayer`（core） |
| 設定 | テーマ・単位・向き・重なり順の組。保存される値の正本 | `RulerSettings`（core） |
| 物差しの状態の所有者 | 長さ・向き・単位・設定を 1 つ持ち、意図で次の状態へ置き換える | `RulerController`（application） |
| 表示フレーム | 最新の状態から生成する、描くための値の組 | `RulerFrame`（application） |
| 画素の列 | cairo の `ImageSurface` に描いた ARGB の並び。X へは転送するだけ | ui-x11 |

## 使ってはいけない語

`Manager` / `Helper` / `Util` / `Utils` / `Common` を型名の語尾に使わない（RS-013）。
役割を語る名前が思いつかないときは、その型が 2 つの責務を持っている可能性が高い。
