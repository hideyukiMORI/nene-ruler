# ADR 0008 — WSLg では描いた画素が当たり判定になる。帯はアルファ 1 で描き、影は窓の種類で消し、Alt+F4 は自分で読む

- 状態: 受理
- 日付: 2026-10-17
- Issue: #14
- 影響する規則: ARC-004 / ARC-011 / RS-002 / QLT-013。仕様 FR-002 / FR-013 / FR-015 / FR-017

## 文脈

施主の WSLg での実測（[wslg-2026-10-17](../quality/wslg-2026-10-17.md)・gate-proofs 第 14 節）で、ADR 0005 の前提が 2 つ崩れた。

1. **Windows 側は X の入力 shape を見ない。** WSLg は X の窓を RDP で Windows の `WS_EX_LAYERED` の窓に写し、Windows はアルファ 0 の画素をクリックが抜ける所として扱う。
   Weston の XWM は X の入力 shape を Windows 側へ伝えない。結果、線は 1 px しか掴めず、ギアの抜いた所は素通しになり、数字は字画の上だけが当たる
2. **Weston の XWM は飾りなしの窓に影を描く。** `weston_wm_window_draw_decoration` は、fullscreen・最大化・`no_shadow` のどれでもない窓に `render_shadow` を描き、その影も RDP で送る。
   `no_shadow` になる条件は `_NET_WM_WINDOW_TYPE` に `_KDE_NET_WM_WINDOW_TYPE_OVERRIDE` が入っていることだけ。施主が 300×200 の窓で実験し、map の前に付けると影が消え、クリックも取れることを確かめた

加えて 2 つ、環境の性質が分かった。

3. WSLg の XWM は Alt+F4 を `WM_DELETE_WINDOW` に変えず、`Alt_L` ＋ `F4` の素の `KeyPress` として窓に届ける
4. WSLg の `_NET_SUPPORTED` に `_NET_WM_STATE_ABOVE` が無い。最前面は WM にできない

## 決定

**当たり判定は、X の入力 shape（ADR 0005）に加えて、同じ矩形をアルファ 1/255 のインクで下塗りして持つ。描いたものを透明に抜く所（ギアの歯・反転した数字の字画）もアルファ 1 にする。
枠の窓は `_NET_WM_WINDOW_TYPE = [_KDE_NET_WM_WINDOW_TYPE_OVERRIDE, _NET_WM_WINDOW_TYPE_NORMAL]` を map の前に持つ。
Alt+F4 はアプリが読んで閉じる。最前面は WM が `_NET_WM_STATE_ABOVE` を挙げていなければ、できないことを通知する。**

1. **アルファ 1 の帯**: `Canvas` は、インクを描く前に `RulerFrame::hit_regions` の矩形すべてを `rgba(ink, 1/255)` で塗る。アルファ 1 は目に見えない（premultiplied で成分は 0 か 1）が、Windows の当たり判定には乗る。X では入力 shape が今まで通り効く。当たり判定の矩形と描く矩形は同じ `FrameLayout` から出る（ARC-001）ので、帯の形は 1 か所で決まる
2. **抜かない**: ギアの歯と軸、反転した数字の字画は `Operator::Clear` ではなく、`rgba(ink, 1/255)` を `Operator::Source` で置く。見た目は透明と区別がつかず、当たり判定は残る
3. **窓の種類**: `declare_hints` で `_NET_WM_WINDOW_TYPE` を設定する。Weston 以外の WM は `_KDE_…` を知らないので次の `_NET_WM_WINDOW_TYPE_NORMAL` を読む（EWMH: 先頭から知っているものを使う）
4. **Alt+F4**: `KeyInput::F4` を足し、`state` に `Mod1`（Alt）があれば枠は `Quit`、モーダルは `Close`。`WM_DELETE_WINDOW` は今まで通り
5. **最前面の対応**: `Display` が `_NET_SUPPORTED` から `_NET_WM_STATE_ABOVE` の有無を読み、`Layering`（`Supported` / `Unsupported`）として application に渡す。`Environment`（透明と重なり順の対応）に束ねる。`AlwaysOnTop` の設定で `Unsupported` なら `FrameNotice::NoAlwaysOnTop`

## 強制

- **active**: 画素の単体テスト。帯・ギアの箱・数字の箱の中はアルファ ≥ 1、中身はアルファ 0（X 無し）
- **active**: `KeyInput` の単体テスト（F4）
- **planned**: WSLg での帯・影・Alt+F4・通知の実測は施主の目視（QLT-013）。記録は gate-proofs 第 14 節に足す

## 結果

得られるもの:

- WSLg で線・ギア・数字が設計どおりに掴める。影が消え、窓の外周の当たり帯も消える
- Alt+F4 がどの WM でも効く。最前面ができない環境で「設定したのに効かない」が無言にならない（FR-014）

失うもの・残る穴:

- 仕様 第 3 節の「アルファは 0 か 1 の 2 値」は「0 か 1/255 か 1」になる。中身（素通し）は 0 のまま
- アルファ 1/255 の画素は合成の結果として 1/255 だけ下の色を変える。目には見えないが、画素を読む道具には出る
- `_KDE_NET_WM_WINDOW_TYPE_OVERRIDE` が WSLg の焦点・Alt+Tab・タスクバーに与える影響は未確認（施主の次の目視）
- 最前面は WSLg では実現できない。Windows 側で topmost にする手段は X11 から無い

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 帯を目に見える色で描く | 仕様（線・数字・鉤括弧・ギア以外は透明）に反する。アルファ 1 で足りる |
| Windows の当たり判定だけに合わせ、X の入力 shape をやめる | Xorg ＋ 合成器の環境では入力 shape が正しく効く（Xvfb で実測）。両方持つほうが環境に依らない |
| 影を `_NET_WM_STATE` や `_MOTIF_WM_HINTS` で消す | Weston の XWM にその経路は無い（ソースの読み・施主の実験）。`_KDE_…` だけが効く |
| override-redirect で影と WM を避ける | ADR 0002 / 0005 で却下済み。移動・伸縮・焦点を失う |
| Alt+F4 を WM に任せ続ける | WSLg では届かない。自分で読むことで、WM の無い Xvfb でも閉じられる |
| 最前面を `XRaiseWindow` の繰り返しで真似る | 他の窓との取り合いになり、焦点を奪う。できないことを示すのが FR-014 の趣旨 |
