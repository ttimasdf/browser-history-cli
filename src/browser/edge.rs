use anyhow::Result;
use std::path::{Path, PathBuf};

use super::chromium;
use crate::db;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn get_db_path() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("EDGE_HISTORY_DB") {
        return Ok(PathBuf::from(custom));
    }

    let base_dir = if cfg!(target_os = "macos") {
        home_dir().join("Library/Application Support/Microsoft Edge")
    } else if cfg!(target_os = "linux") {
        home_dir().join(".config/microsoft-edge")
    } else {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}/AppData/Local", p)))
            .unwrap_or_default();
        PathBuf::from(local_app_data).join("Microsoft/Edge/User Data")
    };

    if base_dir.join("Default/History").exists() {
        Ok(base_dir.join("Default/History"))
    } else {
        let profile = find_first_profile(&base_dir);
        if let Some(p) = profile {
            Ok(p)
        } else {
            Ok(base_dir.join("Default/History"))
        }
    }
}

fn find_first_profile(base_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(base_dir).ok()?;
    let mut profiles: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && e.file_name().to_string_lossy().starts_with("Profile ")
        })
        .map(|e| e.path().join("History"))
        .filter(|p| p.exists())
        .collect();
    profiles.sort();
    profiles.into_iter().next()
}

fn prepared_db() -> Result<PathBuf> {
    let path = get_db_path()?;
    db::prepare_db(&path).map_err(|_| {
        anyhow::anyhow!(
            "Edge history DB not found: {}. Set EDGE_HISTORY_DB env var.",
            path.display()
        )
    })
}

pub fn urls(
    _from: Option<&str>,
    _to: Option<&str>,
    _limit: i64,
    _format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    chromium::urls(&db_path, _from, _to, _limit, _format)
}

pub fn visits(
    _from: Option<&str>,
    _to: Option<&str>,
    _limit: i64,
    _format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    chromium::visits(&db_path, _from, _to, _limit, _format)
}

pub fn searches(
    _from: Option<&str>,
    _to: Option<&str>,
    _limit: i64,
    _format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    chromium::searches(&db_path, _from, _to, _limit, _format)
}

pub fn downloads(
    _from: Option<&str>,
    _to: Option<&str>,
    _limit: i64,
    _format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    chromium::downloads(&db_path, _from, _to, _limit, _format)
}

pub fn summary(
    _from: Option<&str>,
    _to: Option<&str>,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    chromium::summary(&db_path, "Edge", _from, _to)
}

struct CleanupGuard(PathBuf);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
