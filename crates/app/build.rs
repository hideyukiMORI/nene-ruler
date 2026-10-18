//! アプリのアイコンを配布物の形にする（ADR 0023 / 0024）。
//!
//! 絵の正本は core の `AppIcon`。画像ファイルはリポジトリに置かない。
//! - Windows: `.ico` と `.rc` を `OUT_DIR` に書き、`windres` で資源のオブジェクトにして exe にリンクする
//! - Linux: メニュー用の hicolor の PNG を `OUT_DIR/icons/` に書く。`make dist` が cargo の
//!   `build-script-executed` の `out_dir` から拾って `.deb` と zip に入れる
//!
//! 🔴 CNF-012 の例外はこのファイルだけ（ビルドの機械で動き、製品の実行には乗らない）。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use nene_ruler_core::app_icon::AppIcon;
use nene_ruler_core::icon_file::icon_file;
use nene_ruler_core::png_file::png_file;

/// 資源の定義。アイコンの資源番号 1 は `ui-win32` が窓の類に読む番号と同じ。
const RESOURCE_SCRIPT: &str = "1 ICON \"nene-ruler.ico\"\n";

/// cargo への指示は標準出力で渡す（clippy は build.rs の `print_stdout` を見ない）。
fn main() -> Result<(), String> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../core/src/app_icon.rs");
    println!("cargo::rerun-if-changed=../core/src/icon_file.rs");
    println!("cargo::rerun-if-changed=../core/src/png_file.rs");
    println!("cargo::rerun-if-env-changed=WINDRES");
    let out = PathBuf::from(read("OUT_DIR")?);
    match read("CARGO_CFG_TARGET_OS")?.as_str() {
        "windows" => {}
        "linux" => return write_hicolor(&out),
        _other => return Ok(()),
    }
    if read("CARGO_CFG_TARGET_ENV")? != "gnu" {
        return Err(
            "Windows は gnu のターゲットだけを作る（MSVC の rc.exe は未対応・ADR 0023）".to_owned(),
        );
    }
    let object = embed_icon(&out, &windres()?)?;
    println!("cargo::rustc-link-arg-bins={}", object.display());
    Ok(())
}

/// `.ico` と `.rc` を書き、`windres` で COFF のオブジェクトにする。オブジェクトの置き場を返す。
fn embed_icon(out: &Path, windres: &str) -> Result<PathBuf, String> {
    let icons: Vec<AppIcon> = AppIcon::WINDOWS_SIZES
        .iter()
        .map(|size| AppIcon::render(*size))
        .collect();
    fs::write(out.join("nene-ruler.ico"), icon_file(&icons))
        .map_err(|error| format!("アイコンを書けない: {error}"))?;
    fs::write(out.join("icon.rc"), RESOURCE_SCRIPT)
        .map_err(|error| format!("資源の定義を書けない: {error}"))?;
    let object = out.join("icon.o");
    let status = Command::new(windres)
        .current_dir(out)
        .args(["--input", "icon.rc", "--output-format", "coff", "--output"])
        .arg(&object)
        .status()
        .map_err(|error| format!("{windres} を起動できない（mingw-w64 が要る）: {error}"))?;
    if status.success() {
        Ok(object)
    } else {
        Err(format!("{windres} が失敗した: {status}"))
    }
}

/// `out/icons/hicolor/<N>x<N>/apps/nene-ruler.png` を大きさごとに書く（freedesktop のアイコンテーマの形）。
fn write_hicolor(out: &Path) -> Result<(), String> {
    for size in AppIcon::LINUX_SIZES {
        let directory = out
            .join("icons/hicolor")
            .join(format!("{size}x{size}"))
            .join("apps");
        fs::create_dir_all(&directory)
            .map_err(|error| format!("アイコンの置き場を作れない: {error}"))?;
        fs::write(
            directory.join("nene-ruler.png"),
            png_file(&AppIcon::render(size)),
        )
        .map_err(|error| format!("アイコンを書けない: {error}"))?;
    }
    Ok(())
}

/// 使う `windres`。`WINDRES` があればそれ、無ければターゲットの mingw-w64 の名前。
fn windres() -> Result<String, String> {
    match env::var("WINDRES") {
        Ok(path) => Ok(path),
        Err(_unset) => Ok(format!(
            "{}-w64-mingw32-windres",
            read("CARGO_CFG_TARGET_ARCH")?
        )),
    }
}

/// cargo が渡す環境変数。
fn read(name: &str) -> Result<String, String> {
    env::var(name).map_err(|error| format!("{name} が読めない: {error}"))
}
