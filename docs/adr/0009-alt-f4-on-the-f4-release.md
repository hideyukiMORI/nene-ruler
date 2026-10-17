# ADR 0009 — Alt+F4 は、この窓が Alt の押下を受けていれば F4 の離しでも閉じる

- 状態: 受理
- 日付: 2026-10-17
- Issue: #20
- 影響する規則: RS-002 / QLT-013。仕様 FR-015。ADR 0008 の文脈 3 と決定 4 を置き換える

## 文脈

ADR 0008 は、WSLg の実測（[wslg-2026-10-17](../quality/wslg-2026-10-17.md)）から「WSLg は Alt+F4 を `Alt_L` ＋ `F4` の素の `KeyPress` として窓に届ける」とし、`KeyPress` の F4 ＋ `Mod1` で閉じることにした。
**この前提は誤りだった。** 初回の実測で、`xev` の `KeyPress` と `KeyRelease` を分けずに読んでいた。

再目視（`feat/15-label-overlap`・`0edce90`）で `xev -id <枠> -event keyboard` を取り直した。施主の本物のキーボードで Alt+F4 を 3 回押した並びは次のとおりで、PowerShell の `keybd_event` でも、枠でもモーダルでも同じだった。

```text
KeyPress   Alt_L  state 0x0
KeyRelease F4     state 0x8   （1〜3 回）
KeyRelease Alt_L  state 0x8
```

**F4 の `KeyPress` は 1 度も届かない。** ADR 0008 の決定 4 は WSLg で一度も発火しなかった。

Alt+F4 の押下は Windows（msrdc）が握り、RAIL の `SC_CLOSE` として Weston に送る（weston-mirror `libweston/backend-rdp/rdprail.c` の `rail_client_Syscommand_callback` → `request_window_close`）。分かったことは次のとおり。

1. その `SC_CLOSE` は**すぐには届かない**。施主の操作では、Alt+F4 では消えず、しばらくして線を触ったら消えた（終了コード 0）。それより前の「触っていないのに消えた」2 件も、先に送った Alt+F4 や `SC_CLOSE` が遅れて届いたものと読める
2. `SC_CLOSE` を Windows の窓へ直接 `PostMessage` すると、枠は 1 秒で終了した。モーダルは閉じなかった
3. `WM_DELETE_WINDOW` を X から直接送ると、モーダルは即座に閉じる。アプリの受け口は生きている
4. `xmodmap -pm` では `mod1` に `Alt_L` が付いている

つまり WSLg では、Alt+F4 は「効かないか、遅れて効く」。アプリが即座に閉じるには、届く唯一の印である F4 の `KeyRelease`（`Mod1` 付き）を読むしかない。

## 決定

**枠とモーダルは、F4 の `KeyPress`（`Mod1` 付き）に加えて、この窓が Alt の `KeyPress` を受けた後、Alt を離す前に届いた F4 の `KeyRelease`（`Mod1` 付き）でも閉じる。**

1. `KeyInput::Alt`（`XK_Alt_L` / `XK_Alt_R`）を足す
2. `AltF4Gesture`（`crates/ui-x11`）が、この窓で Alt の押下を見たかを覚える。Alt の押下で立て、Alt の離しで下ろす。F4 の押下は `Mod1` があれば閉じる。F4 の離しは、Alt の押下を見ていて、かつ `Mod1` があれば閉じる
3. 枠とモーダルは `EventMask::KEY_RELEASE` を選び、`KeyRelease` もそれぞれの窓へ振り分ける（`event_loop::window_of`）
4. `WM_DELETE_WINDOW` は今まで通り。WSLg の `SC_CLOSE` が遅れて届いたときは、すでに閉じた後なので何も起きない

## 強制

- **active**: `AltF4Gesture` の単体テスト（押下で閉じる・Alt を見た後の離しで閉じる・Alt を見ていない離しでは閉じない・Alt を離した後の離しでは閉じない・`Mod1` の無い離しでは閉じない・他の鍵では閉じない）
- **active**: `KeyInput` の単体テスト（Alt）
- **planned**: WSLg での本物のキーボードの Alt+F4 は施主の目視（QLT-013）。記録は gate-proofs 第 17 節

## 結果

得られるもの:

- WSLg で Alt+F4 が即座に効く。押下が届く Xorg と Xvfb でも今まで通り効く
- 別の窓を Alt+F4 で閉じた直後に焦点が移ってきても、その窓の F4 の離しだけでは閉じない

失うもの・残る穴:

- Alt を押してから焦点が移り、移った先で F4 を離すと閉じる。Alt を押したまま窓をまたぐ操作は稀なので許す
- Xvfb では X サーバが押されていない鍵の離しを捨てるので、`xdotool keyup F4` では WSLg の並びを作れない。F4 の離しは `XSendEvent` で窓へ直接送って確かめた（`send_event` の印が付くが、アプリはその印を見ない）
- WSLg の `SC_CLOSE` が遅れる理由、モーダルが `SC_CLOSE` で閉じない理由は追っていない

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| F4 の離し（`Mod1` 付き）だけで閉じる | 別の窓を Alt+F4 で閉じた直後に焦点が移ってくると、F4 の離しだけを受けて巻き添えで閉じる |
| `KeyPress` の F4 だけを読む（ADR 0008 の決定 4 のまま） | WSLg では F4 の押下が届かない（本物のキーボードで実測） |
| WSLg の `SC_CLOSE` → `WM_DELETE_WINDOW` に任せる | 遅れて届き、モーダルには届かない。人には「効かない」に見える |
| `FocusIn` / `FocusOut` で Alt の記憶を消す | 焦点の出入りの順序は WM ごとに違い、WSLg での並びも測っていない。Alt の離しで消せば足りる |
