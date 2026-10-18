//! アプリのアイコンを配布物の形にする（ADR 0023 / 0024）。
//!
//! 絵の正本は core の `AppIcon`。画像ファイルはリポジトリに置かない。
//! - Windows: `.ico` と `.rc` を `OUT_DIR` に書き、`windres` で資源のオブジェクトにして exe にリンクする。
//!   Microsoft Store の MSIX のロゴ 25 枚も `OUT_DIR/msix/Assets/` に書く（`make dist` が拾う・ADR 0026）
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

/// MSIX のロゴの基準の大きさ（100% のとき）。`NeNe Loupe` と同じ 3 つ。
const MSIX_LOGOS: [(&str, u16); 3] = [
    ("Square44x44Logo", 44),
    ("Square150x150Logo", 150),
    ("StoreLogo", 50),
];
/// MSIX のロゴの倍率（%）。
const MSIX_SCALES: [u16; 5] = [100, 125, 150, 200, 400];
/// タスクバーと一覧が使う、倍率に依らない大きさ（`Square44x44Logo` の `targetsize`）。
const MSIX_TARGET_SIZES: [u16; 5] = [16, 24, 32, 48, 256];
/// `targetsize` の地なしの 2 つの形（暗い背景用と明るい背景用。同じ絵を入れる）。
const MSIX_UNPLATED: [&str; 2] = ["unplated", "lightunplated"];

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
    write_msix_logos(&out)?;
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

/// `out/msix/Assets/` に MSIX のロゴ 25 枚（3 つのロゴ × 倍率 5 つ ＋ targetsize 5 つ × 地なし 2 つ）を書く。
/// Partner Center のアプリ タイル アイコン（300×300）は `out/store/` に書く。
/// 名前と大きさは `NeNe Loupe` の提出で通ったものと同じ。倍率を掛けた大きさは四捨五入（150 の 125% は 188）。
fn write_msix_logos(out: &Path) -> Result<(), String> {
    let assets = out.join("msix/Assets");
    fs::create_dir_all(&assets).map_err(|error| format!("ロゴの置き場を作れない: {error}"))?;
    let mut logos = Vec::new();
    for (name, base) in MSIX_LOGOS {
        for scale in MSIX_SCALES {
            logos.push((
                format!("{name}.scale-{scale}.png"),
                (base * scale + 50) / 100,
            ));
        }
    }
    for size in MSIX_TARGET_SIZES {
        for form in MSIX_UNPLATED {
            logos.push((
                format!("Square44x44Logo.targetsize-{size}_altform-{form}.png"),
                size,
            ));
        }
    }
    // Partner Center に入れるアプリ タイル アイコン（パッケージには入れない）。
    let store = out.join("store");
    fs::create_dir_all(&store).map_err(|error| format!("タイルの置き場を作れない: {error}"))?;
    fs::write(
        store.join("store-logo-300.png"),
        png_file(&AppIcon::render(300)),
    )
    .map_err(|error| format!("タイルを書けない: {error}"))?;
    for (file, size) in logos {
        fs::write(assets.join(file), png_file(&AppIcon::render(size)))
            .map_err(|error| format!("ロゴを書けない: {error}"))?;
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
