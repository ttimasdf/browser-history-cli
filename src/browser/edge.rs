use anyhow::Result;
use std::path::PathBuf;

use super::chromium_shared;
use crate::db;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

pub fn base_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home_dir().join("Library/Application Support/Microsoft Edge")
    } else if cfg!(target_os = "linux") {
        home_dir().join(".config/microsoft-edge")
    } else {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}/AppData/Local", p)))
            .unwrap_or_default();
        PathBuf::from(local_app_data).join("Microsoft/Edge/User Data")
    }
}

pub fn get_db_path(profile: Option<&str>) -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("EDGE_HISTORY_DB") {
        return Ok(PathBuf::from(custom));
    }
    chromium_shared::find_chromium_db_path(&base_dir(), profile)
}

pub fn list_profiles() -> Result<()> {
    if let Ok(custom) = std::env::var("EDGE_HISTORY_DB") {
        println!("Custom DB: {}", custom);
        return Ok(());
    }
    chromium_shared::list_profiles(&base_dir())
}

pub fn urls(from: Option<&str>, to: Option<&str>, limit: i64, format: &str, profile: Option<&str>) -> Result<()> {
    let db_path = get_db_path(profile)?;
    let _guard = db::TempFileGuard(db::prepare_db(&db_path)?);
    chromium_shared::urls(&_guard.0, from, to, limit, format)
}

pub fn visits(from: Option<&str>, to: Option<&str>, limit: i64, format: &str, profile: Option<&str>) -> Result<()> {
    let db_path = get_db_path(profile)?;
    let _guard = db::TempFileGuard(db::prepare_db(&db_path)?);
    chromium_shared::visits(&_guard.0, from, to, limit, format)
}

pub fn searches(from: Option<&str>, to: Option<&str>, limit: i64, format: &str, profile: Option<&str>) -> Result<()> {
    let db_path = get_db_path(profile)?;
    let _guard = db::TempFileGuard(db::prepare_db(&db_path)?);
    chromium_shared::searches(&_guard.0, from, to, limit, format)
}

pub fn downloads(from: Option<&str>, to: Option<&str>, limit: i64, format: &str, profile: Option<&str>) -> Result<()> {
    let db_path = get_db_path(profile)?;
    let _guard = db::TempFileGuard(db::prepare_db(&db_path)?);
    chromium_shared::downloads(&_guard.0, from, to, limit, format)
}

pub fn summary(from: Option<&str>, to: Option<&str>, profile: Option<&str>) -> Result<()> {
    let db_path = get_db_path(profile)?;
    let _guard = db::TempFileGuard(db::prepare_db(&db_path)?);
    chromium_shared::summary(&_guard.0, "Edge", from, to)
}
