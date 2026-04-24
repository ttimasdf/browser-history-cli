use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::db;
use crate::output;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn get_db_path() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("FIREFOX_HISTORY_DB") {
        return Ok(PathBuf::from(custom));
    }

    let base_dir = if cfg!(target_os = "macos") {
        home_dir().join("Library/Application Support/Firefox/Profiles")
    } else if cfg!(target_os = "linux") {
        home_dir().join(".mozilla/firefox")
    } else {
        let app_data = std::env::var("APPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}/AppData/Roaming", p)))
            .unwrap_or_default();
        PathBuf::from(app_data).join("Mozilla/Firefox/Profiles")
    };

    let profile_dir = find_profile(&base_dir);
    if let Some(p) = profile_dir {
        Ok(p.join("places.sqlite"))
    } else {
        Ok(base_dir.join("places.sqlite"))
    }
}

fn find_profile(base_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(base_dir).ok()?;

    let mut default_release: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && e.file_name().to_string_lossy().ends_with(".default-release")
        })
        .map(|e| e.path())
        .collect();
    default_release.sort();
    if let Some(p) = default_release.into_iter().next() {
        return Some(p);
    }

    let entries = std::fs::read_dir(base_dir).ok()?;
    let mut default_profiles: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && e.file_name().to_string_lossy().contains(".default")
        })
        .map(|e| e.path())
        .collect();
    default_profiles.sort();
    default_profiles.into_iter().next()
}

fn prepared_db() -> Result<PathBuf> {
    let path = get_db_path()?;
    db::prepare_db(&path).map_err(|_| {
        anyhow::anyhow!(
            "Firefox history DB not found: {}. Set FIREFOX_HISTORY_DB env var.",
            path.display()
        )
    })
}

const FIREFOX_DT_EXPR: &str = "datetime({}/1000000, 'unixepoch', 'localtime')";

fn firefox_dt(col: &str) -> String {
    FIREFOX_DT_EXPR.replace("{}", col)
}

pub fn urls(
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let sep = output::sep_for_format(format);
    let where_clause = db::build_firefox_date_filter("last_visit_date", from, to)?;
    let dt = firefox_dt("last_visit_date");

    let mut sql = format!(
        "SELECT url, COALESCE(title, '') as title, visit_count, {} as last_visit FROM moz_places WHERE visit_count > 0",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" AND {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY last_visit_date DESC LIMIT {};", limit));

    output::print_header(sep, &["url", "title", "visit_count", "last_visit_time"]);
    db::query_db_formatted(&db_path, &sql, sep)
}

pub fn visits(
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let sep = output::sep_for_format(format);
    let where_clause = db::build_firefox_date_filter("v.visit_date", from, to)?;
    let dt = firefox_dt("v.visit_date");

    let mut sql = format!(
        "SELECT p.url, COALESCE(p.title, '') as title, \
         {} as visit_time, \
         CASE v.visit_type \
           WHEN 1 THEN 'LINK' WHEN 2 THEN 'TYPED' WHEN 3 THEN 'BOOKMARK' \
           WHEN 4 THEN 'EMBED' WHEN 5 THEN 'REDIRECT_PERM' \
           WHEN 6 THEN 'REDIRECT_TEMP' WHEN 7 THEN 'DOWNLOAD' \
           WHEN 8 THEN 'FRAMED_LINK' WHEN 9 THEN 'RELOAD' \
           ELSE 'OTHER(' || v.visit_type || ')' \
         END as transition_type, \
         COALESCE(fp.url, '') as from_url \
         FROM moz_historyvisits v \
         JOIN moz_places p ON v.place_id = p.id \
         LEFT JOIN moz_historyvisits fv ON v.from_visit = fv.id \
         LEFT JOIN moz_places fp ON fv.place_id = fp.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY v.visit_date DESC LIMIT {};", limit));

    output::print_header(sep, &["url", "title", "visit_time", "transition", "from_url"]);
    db::query_db_formatted(&db_path, &sql, sep)
}

pub fn searches(
    _from: Option<&str>,
    _to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let sep = output::sep_for_format(format);

    let sql = format!(
        "SELECT i.input, p.url, COALESCE(p.title, '') as title, i.use_count \
         FROM moz_inputhistory i \
         JOIN moz_places p ON i.place_id = p.id \
         ORDER BY i.use_count DESC \
         LIMIT {};",
        limit
    );

    output::print_header(sep, &["input", "url", "title", "use_count"]);
    db::query_db_formatted(&db_path, &sql, sep)
}

pub fn bookmarks(
    _from: Option<&str>,
    _to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let sep = output::sep_for_format(format);

    let sql = format!(
        "SELECT b.title, COALESCE(p.url, '') as url, \
         datetime(b.dateAdded/1000000, 'unixepoch', 'localtime') as dateAdded, \
         COALESCE(pb.title, '') as parent_title \
         FROM moz_bookmarks b \
         LEFT JOIN moz_places p ON b.fk = p.id \
         LEFT JOIN moz_bookmarks pb ON b.parent = pb.id \
         WHERE b.type = 1 \
         ORDER BY b.dateAdded DESC \
         LIMIT {};",
        limit
    );

    output::print_header(sep, &["title", "url", "dateAdded", "parent_title"]);
    db::query_db_formatted(&db_path, &sql, sep)
}

pub fn summary(
    from: Option<&str>,
    to: Option<&str>,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let where_clause = db::build_firefox_date_filter("v.visit_date", from, to)?;
    let wc = if !where_clause.is_empty() {
        format!(" WHERE {}", where_clause)
    } else {
        String::new()
    };

    println!("=== Firefox History Summary ===");
    if let Some(f) = from {
        println!("From: {}", f);
    }
    if let Some(t) = to {
        println!("To: {}", t);
    }
    println!();

    println!("--- Basic Stats ---");
    let stats_sql = format!(
        "SELECT COUNT(*), COUNT(DISTINCT v.place_id) FROM moz_historyvisits v {};",
        wc
    );
    let stats = db::query_db(&db_path, &stats_sql, '\t')?;
    if let Some(row) = stats.first() {
        println!("Total visits:    {}", row.first().unwrap_or(&"0".to_string()));
        println!("Unique URLs:     {}", row.get(1).unwrap_or(&"0".to_string()));
    }

    println!();
    println!("--- Top 10 Domains ---");
    let domains_sql = format!(
        "SELECT \
           REPLACE(REPLACE(SUBSTR(p.url, INSTR(p.url,'://')+3),'www.',''), \
             SUBSTR(REPLACE(SUBSTR(p.url, INSTR(p.url,'://')+3),'www.',''), \
               INSTR(REPLACE(SUBSTR(p.url, INSTR(p.url,'://')+3),'www.',''),'/')), '') as domain, \
           COUNT(*) as cnt \
           FROM moz_historyvisits v \
           JOIN moz_places p ON v.place_id = p.id {} \
           GROUP BY domain ORDER BY cnt DESC LIMIT 10;",
        wc
    );
    let domains = db::query_db(&db_path, &domains_sql, '\t')?;
    for row in domains {
        let domain = row.first().unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<40} {} visits", domain, count);
    }

    println!();
    println!("--- Visit Types ---");
    let types_sql = format!(
        "SELECT \
           CASE v.visit_type \
             WHEN 1 THEN 'LINK' WHEN 2 THEN 'TYPED' WHEN 3 THEN 'BOOKMARK' \
             WHEN 4 THEN 'EMBED' WHEN 5 THEN 'REDIRECT_PERM' \
             WHEN 6 THEN 'REDIRECT_TEMP' WHEN 7 THEN 'DOWNLOAD' \
             WHEN 8 THEN 'FRAMED_LINK' WHEN 9 THEN 'RELOAD' ELSE 'OTHER' \
           END as type, COUNT(*) as cnt \
           FROM moz_historyvisits v {} \
           GROUP BY type ORDER BY cnt DESC;",
        wc
    );
    let types = db::query_db(&db_path, &types_sql, '\t')?;
    for row in types {
        let typ = row.first().unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<20} {}", typ, count);
    }

    Ok(())
}

struct CleanupGuard(PathBuf);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
