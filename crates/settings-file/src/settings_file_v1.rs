//! 設定ファイルの版 1 の直列化の型。core の型と分ける（ARC-009 / ADR 0006）。

use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
use nene_ruler_core::modal_theme::ModalTheme;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::ruler_settings::RulerSettings;
use nene_ruler_core::unit::Unit;
use nene_ruler_core::window_layer::WindowLayer;

/// この型が読み書きする版。
const VERSION: &str = "1";

/// `version = 1` のファイル 1 つ分。鍵は 4 つで、`unit` だけは無くてもよい（1.0.0 のファイルに無い）。
///
/// `unit` は `physical_px` / `logical_px`。以前の `px` / `mm` / `in` は物理画素として読む（ADR 0012 → 0027）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsFileV1 {
    ink: Rgb,
    unit: Unit,
    modal_theme: ModalTheme,
    layer: WindowLayer,
}

impl SettingsFileV1 {
    /// core の設定から。
    #[must_use]
    pub const fn from_settings(settings: RulerSettings) -> Self {
        Self {
            ink: settings.ink(),
            unit: settings.unit(),
            modal_theme: settings.modal_theme(),
            layer: settings.layer(),
        }
    }

    /// core の設定へ。境界で一度だけ検証した値なので失敗しない（ARC-008）。
    #[must_use]
    pub const fn into_settings(self) -> RulerSettings {
        RulerSettings::INITIAL
            .with_ink(self.ink)
            .with_unit(self.unit)
            .with_modal_theme(self.modal_theme)
            .with_layer(self.layer)
    }

    /// ファイルの文字列に書く。鍵の順は固定（RS-016）。
    #[must_use]
    pub fn render(self) -> String {
        format!(
            "version = {VERSION}\nink = {}\nmodal_theme = {}\nunit = {}\nlayer = {}\n",
            self.ink.hex(),
            theme_word(self.modal_theme),
            unit_word(self.unit),
            layer_word(self.layer),
        )
    }

    /// ファイルの文字列を読む。空行と `#` の行は飛ばす。
    ///
    /// # Errors
    ///
    /// 版の行が無い・鍵が足りない・値が読めない → `Malformed`。版が 1 でない → `UnknownVersion`。
    pub fn parse(text: &str) -> Result<Self, SettingsStoreFailure> {
        let mut version = None;
        let mut ink = None;
        let mut unit = None;
        let mut modal_theme = None;
        let mut layer = None;
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .map(|(key, value)| (key.trim(), value.trim()))
                .ok_or(SettingsStoreFailure::Malformed)?;
            match key {
                "version" => version = Some(value),
                "ink" => {
                    ink =
                        Some(Rgb::parse(value).map_err(|_error| SettingsStoreFailure::Malformed)?);
                }
                "modal_theme" => modal_theme = Some(parse_theme(value)?),
                "unit" => unit = Some(parse_unit(value)?),
                "layer" => layer = Some(parse_layer(value)?),
                _unknown => return Err(SettingsStoreFailure::Malformed),
            }
        }
        match version {
            Some(VERSION) => {}
            Some(_other) => return Err(SettingsStoreFailure::UnknownVersion),
            None => return Err(SettingsStoreFailure::Malformed),
        }
        Ok(Self {
            ink: ink.ok_or(SettingsStoreFailure::Malformed)?,
            unit: unit.unwrap_or(Unit::Physical),
            modal_theme: modal_theme.ok_or(SettingsStoreFailure::Malformed)?,
            layer: layer.ok_or(SettingsStoreFailure::Malformed)?,
        })
    }
}

/// 見えの語。
const fn theme_word(theme: ModalTheme) -> &'static str {
    match theme {
        ModalTheme::Dark => "dark",
        ModalTheme::Light => "light",
    }
}

/// 単位の語。
const fn unit_word(unit: Unit) -> &'static str {
    match unit {
        Unit::Physical => "physical_px",
        Unit::Logical => "logical_px",
    }
}

/// 重なり順の語。
const fn layer_word(layer: WindowLayer) -> &'static str {
    match layer {
        WindowLayer::AlwaysOnTop => "always_on_top",
        WindowLayer::Normal => "normal",
    }
}

fn parse_theme(value: &str) -> Result<ModalTheme, SettingsStoreFailure> {
    match value {
        "dark" => Ok(ModalTheme::Dark),
        "light" => Ok(ModalTheme::Light),
        _other => Err(SettingsStoreFailure::Malformed),
    }
}

/// 単位。以前の `px` / `mm` / `in`（ADR 0012 以前の形）は物理画素として読む。それ以外は壊れたファイル。
fn parse_unit(value: &str) -> Result<Unit, SettingsStoreFailure> {
    match value {
        "physical_px" | "px" | "mm" | "in" => Ok(Unit::Physical),
        "logical_px" => Ok(Unit::Logical),
        _other => Err(SettingsStoreFailure::Malformed),
    }
}

fn parse_layer(value: &str) -> Result<WindowLayer, SettingsStoreFailure> {
    match value {
        "always_on_top" => Ok(WindowLayer::AlwaysOnTop),
        "normal" => Ok(WindowLayer::Normal),
        _other => Err(SettingsStoreFailure::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RulerSettings {
        RulerSettings::INITIAL
            .with_ink(Rgb::new(0x00, 0xAA, 0x55))
            .with_unit(Unit::Logical)
            .with_modal_theme(ModalTheme::Light)
            .with_layer(WindowLayer::Normal)
    }

    #[test]
    fn renders_the_documented_shape() {
        let text = SettingsFileV1::from_settings(sample()).render();
        assert_eq!(
            text,
            "version = 1\nink = #00AA55\nmodal_theme = light\nunit = logical_px\nlayer = normal\n"
        );
    }

    #[test]
    fn round_trips_through_the_text() {
        let text = SettingsFileV1::from_settings(sample()).render();
        assert_eq!(
            SettingsFileV1::parse(&text).unwrap().into_settings(),
            sample()
        );
        let initial = SettingsFileV1::from_settings(RulerSettings::INITIAL).render();
        assert_eq!(
            SettingsFileV1::parse(&initial).unwrap().into_settings(),
            RulerSettings::INITIAL
        );
    }

    #[test]
    fn tolerates_blank_lines_comments_spacing_and_order() {
        let text = "\n# hand edited\nlayer=normal\n  unit =  logical_px\nmodal_theme = dark\nink = #112233\nversion = 1\n";
        let parsed = SettingsFileV1::parse(text).unwrap().into_settings();
        assert_eq!(parsed.ink(), Rgb::new(0x11, 0x22, 0x33));
        assert_eq!(parsed.unit(), Unit::Logical);
    }

    #[test]
    fn a_file_without_a_unit_line_is_physical_pixels() {
        // 1.0.0 が書いたファイルの形。unit が無ければ物理画素で、書き戻すと unit = physical_px が足される。
        let old = "version = 1\nink = #00AA55\nmodal_theme = light\nlayer = normal\n";
        let parsed = SettingsFileV1::parse(old).unwrap();
        assert_eq!(parsed.into_settings(), sample().with_unit(Unit::Physical));
        assert!(parsed.render().contains("unit = physical_px\n"));
    }

    #[test]
    fn retired_px_millimetres_and_inches_read_as_physical_pixels() {
        // ADR 0012 より前のファイル（施主の手元にあった形）。px / mm / in は物理画素として読む。
        for word in ["mm", "in", "px"] {
            let old = format!(
                "version = 1\nink = #00AA55\nmodal_theme = light\nunit = {word}\nlayer = normal\n"
            );
            let parsed = SettingsFileV1::parse(&old).unwrap();
            assert_eq!(parsed.into_settings().unit(), Unit::Physical, "{word}");
            assert!(parsed.render().contains("unit = physical_px\n"));
        }
    }

    #[test]
    fn rejects_unknown_versions_and_malformed_files() {
        let good = SettingsFileV1::from_settings(sample()).render();
        assert_eq!(
            SettingsFileV1::parse(&good.replace("version = 1", "version = 2")),
            Err(SettingsStoreFailure::UnknownVersion)
        );
        assert_eq!(
            SettingsFileV1::parse(&good.replace("version = 1\n", "")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&good.replace("layer = normal\n", "")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&good.replace("#00AA55", "#00AA5")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&good.replace("light", "system")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&good.replace("normal", "top")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&format!("{good}unit = cm\n")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse(&format!("{good}colour = red\n")),
            Err(SettingsStoreFailure::Malformed)
        );
        assert_eq!(
            SettingsFileV1::parse("version 1\n"),
            Err(SettingsStoreFailure::Malformed)
        );
    }
}
