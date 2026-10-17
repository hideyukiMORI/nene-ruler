//! 設定ファイルの読み書き。置き場は XDG（FR-012 / ADR 0006）。

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
use nene_ruler_application::settings_store_port::SettingsStorePort;
use nene_ruler_core::ruler_settings::RulerSettings;

use crate::settings_file_v1::SettingsFileV1;

/// 置き場のディレクトリ名とファイル名。版はファイル名にも出す。
const DIRECTORY: &str = "nene-ruler";
const FILE_NAME: &str = "settings.v1.txt";
/// 置き換えの前に書く一時ファイルの名。
const TEMPORARY_NAME: &str = "settings.v1.txt.tmp";

/// `$XDG_CONFIG_HOME/nene-ruler/settings.v1.txt` を読み書きする保存先。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSettingsStore {
    path: PathBuf,
}

impl FileSettingsStore {
    /// XDG の規則で置き場を決める。`XDG_CONFIG_HOME` が空でなければそれ、無ければ `HOME/.config`。
    ///
    /// # Errors
    ///
    /// どちらの環境変数も無い。
    pub fn in_xdg_config() -> Result<Self, SettingsStoreFailure> {
        let base = env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or(SettingsStoreFailure::NoConfigDirectory)?;
        Ok(Self::at(base.join(DIRECTORY).join(FILE_NAME)))
    }

    /// 置き場を直接指す（テストと、将来の `--config` のため）。
    #[must_use]
    pub const fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// 設定ファイルの場所。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SettingsStorePort for FileSettingsStore {
    fn load(&self) -> Result<Option<RulerSettings>, SettingsStoreFailure> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_other) => return Err(SettingsStoreFailure::Unreadable),
        };
        SettingsFileV1::parse(&text).map(|file| Some(file.into_settings()))
    }

    fn save(&self, settings: RulerSettings) -> Result<(), SettingsStoreFailure> {
        let directory = self.path.parent().ok_or(SettingsStoreFailure::Unwritable)?;
        fs::create_dir_all(directory).map_err(|_error| SettingsStoreFailure::Unwritable)?;
        let temporary = directory.join(TEMPORARY_NAME);
        let text = SettingsFileV1::from_settings(settings).render();
        fs::write(&temporary, text).map_err(|_error| SettingsStoreFailure::Unwritable)?;
        fs::rename(&temporary, &self.path).map_err(|_error| SettingsStoreFailure::Unwritable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ruler_core::rgb::Rgb;

    /// テストごとに別の一時ディレクトリ。プロセス ID とテスト名で分ける。
    fn scratch(name: &str) -> PathBuf {
        let directory =
            env::temp_dir().join(format!("nene-ruler-test-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn missing_file_is_none_not_a_failure() {
        let store = FileSettingsStore::at(scratch("missing").join("settings.v1.txt"));
        assert_eq!(store.load(), Ok(None));
    }

    #[test]
    fn saves_then_loads_the_same_settings_and_leaves_no_temporary() {
        let directory = scratch("roundtrip");
        let store = FileSettingsStore::at(directory.join("deep").join("settings.v1.txt"));
        let settings = RulerSettings::INITIAL.with_ink(Rgb::new(1, 2, 3));
        store.save(settings).unwrap();
        assert_eq!(store.load(), Ok(Some(settings)));
        assert!(!directory.join("deep").join(TEMPORARY_NAME).exists());
        let text = fs::read_to_string(store.path()).unwrap();
        assert!(text.starts_with("version = 1\n"));
    }

    #[test]
    fn a_broken_file_is_reported_not_defaulted() {
        let store = FileSettingsStore::at(scratch("broken").join("settings.v1.txt"));
        store.save(RulerSettings::INITIAL).unwrap();
        fs::write(store.path(), "version = 7\n").unwrap();
        assert_eq!(store.load(), Err(SettingsStoreFailure::UnknownVersion));
        fs::write(store.path(), "ink = #123456\n").unwrap();
        assert_eq!(store.load(), Err(SettingsStoreFailure::Malformed));
    }

    #[test]
    fn a_directory_in_place_of_the_file_is_unreadable() {
        let path = scratch("directory").join("settings.v1.txt");
        fs::create_dir_all(&path).unwrap();
        let store = FileSettingsStore::at(path);
        assert_eq!(store.load(), Err(SettingsStoreFailure::Unreadable));
        assert_eq!(
            store.save(RulerSettings::INITIAL),
            Err(SettingsStoreFailure::Unwritable)
        );
    }
}
