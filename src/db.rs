use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

const CHROMIUM_EPOCH_OFFSET: i64 = 11644473600;
const SAFARI_EPOCH_OFFSET: i64 = 978307200;

#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    #[cfg(target_os = "linux")]
    tm_gmtoff: i64,
    #[cfg(target_os = "linux")]
    tm_zone: *const i8,
}

extern "C" {
    fn mktime(tp: *mut Tm) -> i64;
}

pub fn prepare_db(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        anyhow::bail!("History DB not found: {}", path.display());
    }
    let tmp_dir = std::env::temp_dir();
    let tmp_name = format!("browser_history_{}.db", std::process::id());
    let tmp_path = tmp_dir.join(tmp_name);
    fs::copy(path, &tmp_path)
        .with_context(|| format!("Failed to copy DB to temp: {}", path.display()))?;
    Ok(tmp_path)
}

pub fn date_to_epoch_secs(date: &str) -> Result<i64> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid date format: {} (use YYYY-MM-DD)", date);
    }
    let year: i32 = parts[0].parse().context("Invalid year")?;
    let month: i32 = parts[1].parse().context("Invalid month")?;
    let day: i32 = parts[2].parse().context("Invalid day")?;

    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: day,
        tm_mon: month - 1,
        tm_year: year - 1900,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: -1,
        #[cfg(target_os = "linux")]
        tm_gmtoff: 0,
        #[cfg(target_os = "linux")]
        tm_zone: std::ptr::null(),
    };
    Ok(unsafe { mktime(&mut tm) })
}

pub fn date_to_chromium_ts(date: &str) -> Result<i64> {
    let epoch = date_to_epoch_secs(date)?;
    Ok((epoch + CHROMIUM_EPOCH_OFFSET) * 1_000_000)
}

pub fn date_to_firefox_ts(date: &str) -> Result<i64> {
    let epoch = date_to_epoch_secs(date)?;
    Ok(epoch * 1_000_000)
}

pub fn date_to_safari_ts(date: &str) -> Result<f64> {
    let epoch = date_to_epoch_secs(date)?;
    Ok((epoch - SAFARI_EPOCH_OFFSET) as f64)
}

pub fn build_chromium_date_filter(
    col: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<String> {
    let mut clauses = Vec::new();
    if let Some(f) = from {
        let ts = date_to_chromium_ts(f)?;
        clauses.push(format!("{} >= {}", col, ts));
    }
    if let Some(t) = to {
        let epoch = date_to_epoch_secs(t)?;
        let ts = (epoch + CHROMIUM_EPOCH_OFFSET + 86400) * 1_000_000;
        clauses.push(format!("{} < {}", col, ts));
    }
    Ok(clauses.join(" AND "))
}

pub fn build_firefox_date_filter(
    col: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<String> {
    let mut clauses = Vec::new();
    if let Some(f) = from {
        let ts = date_to_firefox_ts(f)?;
        clauses.push(format!("{} >= {}", col, ts));
    }
    if let Some(t) = to {
        let epoch = date_to_epoch_secs(t)?;
        let ts = (epoch + 86400) * 1_000_000;
        clauses.push(format!("{} < {}", col, ts));
    }
    Ok(clauses.join(" AND "))
}

pub fn build_safari_date_filter(
    col: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<String> {
    let mut clauses = Vec::new();
    if let Some(f) = from {
        let ts = date_to_safari_ts(f)?;
        clauses.push(format!("{} >= {}", col, ts as i64));
    }
    if let Some(t) = to {
        let epoch = date_to_epoch_secs(t)?;
        let ts = epoch + 86400 - SAFARI_EPOCH_OFFSET;
        clauses.push(format!("{} < {}", col, ts));
    }
    Ok(clauses.join(" AND "))
}

pub fn query_db(db_path: &Path, sql: &str, _sep: char) -> Result<Vec<Vec<String>>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(sql)?;
    let col_count = stmt.column_count();
    let mut rows = stmt.query([])?;

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let mut cols = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let val: String = row.get::<_, Option<String>>(i)?.unwrap_or_default();
            cols.push(val);
        }
        results.push(cols);
    }
    Ok(results)
}

pub fn query_db_formatted(db_path: &Path, sql: &str, sep: char) -> Result<()> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(sql)?;
    let col_count = stmt.column_count();
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let mut line = String::new();
        for i in 0..col_count {
            if i > 0 {
                line.push(sep);
            }
            let val: String = row.get::<_, Option<String>>(i)?.unwrap_or_default();
            line.push_str(&val);
        }
        println!("{}", line);
    }
    Ok(())
}