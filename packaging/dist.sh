#!/usr/bin/env bash
# make dist の本体（ADR 0025）。Linux の上で、配る物を dist/ に作る。品質の判断は持たない。
#
#   dist/nene-ruler_amd64.deb             Linux の .deb（/usr/bin・.desktop・hicolor のアイコン）
#   dist/nene-ruler-linux-x86_64.zip      Linux のポータブル
#   dist/NeNe-Ruler-windows-portable.zip  Windows のポータブル
#   dist/nene-ruler.exe                   MSI と MSIX の材料（Windows のランナーが包む。それ自体は配らない）
#   dist/msix-assets/                     MSIX のロゴ 25 枚（build.rs が core の絵から作った物・ADR 0026）
#   dist/store-logo-300.png               Partner Center に入れるアプリ タイル アイコン（同じく build.rs から）
#   dist/*.sha256                         配る 3 つの sha256（sha256sum --check の形）
#
# アイコンの画像はリポジトリに無い。crates/app/build.rs が core の絵から OUT_DIR に書いたものを、
# cargo の build-script-executed の out_dir から拾う（ADR 0024）。
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

# 版は [workspace.package] の version が唯一の出どころ（FR-011）。
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
[ -n "$version" ] || { echo "dist: Cargo.toml に version が無い" >&2; exit 1; }

dist=dist
stage=$dist/stage
rm -rf "$dist"
mkdir -p "$stage"

# ── Linux ────────────────────────────────────────────────────────────────
out_dir=$(cargo build --release --locked -p nene-ruler --message-format=json \
  | jq -r 'select(.reason == "build-script-executed" and (.package_id | test("/crates/app#"))) | .out_dir' \
  | tail -n 1)
icons=$out_dir/icons/hicolor
[ -d "$icons" ] || { echo "dist: build.rs のアイコンが無い（$icons）" >&2; exit 1; }
linux_binary=target/release/nene-ruler

deb=$stage/deb
install -D -m 0755 "$linux_binary" "$deb/usr/bin/nene-ruler"
strip "$deb/usr/bin/nene-ruler"
install -D -m 0644 packaging/linux/nene-ruler.desktop "$deb/usr/share/applications/nene-ruler.desktop"
install -D -m 0644 LICENSE "$deb/usr/share/doc/nene-ruler/copyright"
mkdir -p "$deb/usr/share/icons"
cp -r "$icons" "$deb/usr/share/icons/"
# 共有ライブラリの依存は dpkg-shlibdeps が実物から決める（libc6・libcairo2 など）。
shlibs=$stage/shlibs
mkdir -p "$shlibs/debian"
printf 'Source: nene-ruler\n' > "$shlibs/debian/control"
depends=$(cd "$shlibs" && dpkg-shlibdeps -O "$root/$deb/usr/bin/nene-ruler" | sed -n 's/^shlibs:Depends=//p')
installed_size=$(du -sk "$deb/usr" | cut -f1)
mkdir -p "$deb/DEBIAN"
cat > "$deb/DEBIAN/control" <<EOF
Package: nene-ruler
Version: $version
Section: utils
Priority: optional
Architecture: amd64
Maintainer: hideyukiMORI <info@ayane.co.jp>
Installed-Size: $installed_size
Depends: $depends
Homepage: https://github.com/hideyukiMORI/nene-ruler
Description: NeNe Ruler - a tiny transparent screen ruler
 A transparent frame with ticks on all four edges that measures in physical pixels.
 Drag a line to move it, drag a bracket to resize it, click a number to copy it.
EOF
dpkg-deb --root-owner-group --build "$deb" "$dist/nene-ruler_amd64.deb" >/dev/null

portable=$stage/nene-ruler-linux-x86_64
mkdir -p "$portable"
cp "$deb/usr/bin/nene-ruler" packaging/linux/nene-ruler.desktop LICENSE README.md "$portable/"
mkdir -p "$portable/icons"
cp -r "$icons" "$portable/icons/"
(cd "$stage" && zip -qr "../nene-ruler-linux-x86_64.zip" nene-ruler-linux-x86_64)

# ── Windows（gnu のターゲット・exe にアイコンが埋まる・ADR 0021 / 0023） ──────────────
windows_out_dir=$(cargo build --release --locked -p nene-ruler --target x86_64-pc-windows-gnu --message-format=json \
  | jq -r 'select(.reason == "build-script-executed" and (.package_id | test("/crates/app#"))) | .out_dir' \
  | tail -n 1)
msix_assets=$windows_out_dir/msix/Assets
[ "$(find "$msix_assets" -name '*.png' | wc -l)" -eq 25 ] || { echo "dist: build.rs の MSIX のロゴが 25 枚でない（$msix_assets）" >&2; exit 1; }
cp -r "$msix_assets" "$dist/msix-assets"
cp "$windows_out_dir/store/store-logo-300.png" "$dist/store-logo-300.png"
windows_binary=target/x86_64-pc-windows-gnu/release/nene-ruler.exe
cp "$windows_binary" "$dist/nene-ruler.exe"
x86_64-w64-mingw32-strip "$dist/nene-ruler.exe"
windows=$stage/NeNe-Ruler-windows-portable
mkdir -p "$windows"
cp "$dist/nene-ruler.exe" LICENSE README.md "$windows/"
(cd "$stage" && zip -qr "../NeNe-Ruler-windows-portable.zip" NeNe-Ruler-windows-portable)

# ── sha256（配る物だけ） ─────────────────────────────────────────────────
rm -rf "$stage"
(cd "$dist" && for file in nene-ruler_amd64.deb nene-ruler-linux-x86_64.zip NeNe-Ruler-windows-portable.zip; do
  sha256sum "$file" > "$file.sha256"
done)
echo "dist: NeNe Ruler $version"
ls -l "$dist"
