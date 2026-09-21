//! Locations of the central database and its backups (spec §5).

use std::ffi::OsString;
use std::path::PathBuf;

/// Environment variable that overrides the data directory (tests, experiments).
pub const DATA_DIR_ENV: &str = "BIB_DATA_DIR";

/// `$BIB_DATA_DIR`, else `$XDG_DATA_HOME/bib`, else `~/.local/share/bib`.
pub fn data_dir() -> PathBuf {
    data_dir_from(|name| std::env::var_os(name))
}

pub fn db_path() -> PathBuf {
    data_dir().join("bib.db")
}

pub fn backup_dir() -> PathBuf {
    data_dir().join("backups")
}

fn data_dir_from(var: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    if let Some(dir) = var(DATA_DIR_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = var("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(xdg).join("bib");
    }
    let home = var("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share/bib")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = pairs.iter().map(|(k, v)| (k.to_string(), OsString::from(v))).collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn override_wins() {
        let dir = data_dir_from(env(&[
            ("BIB_DATA_DIR", "/tmp/x"),
            ("XDG_DATA_HOME", "/xdg"),
            ("HOME", "/home/a"),
        ]));
        assert_eq!(dir, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn xdg_then_home() {
        assert_eq!(
            data_dir_from(env(&[("XDG_DATA_HOME", "/xdg"), ("HOME", "/home/a")])),
            PathBuf::from("/xdg/bib")
        );
        assert_eq!(
            data_dir_from(env(&[("HOME", "/home/a")])),
            PathBuf::from("/home/a/.local/share/bib")
        );
    }

    #[test]
    fn empty_values_are_ignored() {
        let dir = data_dir_from(env(&[("BIB_DATA_DIR", ""), ("XDG_DATA_HOME", ""), ("HOME", "/h")]));
        assert_eq!(dir, PathBuf::from("/h/.local/share/bib"));
    }
}
