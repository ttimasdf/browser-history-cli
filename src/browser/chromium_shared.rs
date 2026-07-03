use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::db;
use crate::output;

const CHROMIUM_DT_EXPR: &str = "datetime({}/1000000 - 11644473600, 'unixepoch', 'localtime')";

fn chromium_dt(col: &str) -> String {
    CHROMIUM_DT_EXPR.replace("{}", col)
}

pub fn find_chromium_db_path(base_dir: &Path, profile: Option<&str>) -> Result<PathBuf> {
    if let Some(p) = profile {
        let path = base_dir.join(p).join("History");
        if path.exists() {
            return Ok(path);
        }
        anyhow::bail!(
            "Profile '{}' not found or has no History DB at {}",
            p, path.display()
        );
    }

    if base_dir.join("Default/History").exists() {
        return Ok(base_dir.join("Default/History"));
    }

    let (_names, _order, last_used) = read_profile_names(base_dir);

    // Collect available profile dirs with History files
    let entries = std::fs::read_dir(base_dir)?;
    let available: HashSet<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && e.file_name().to_string_lossy().starts_with("Profile ")
        })
        .filter(|e| e.path().join("History").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Try last_used first (most recently used profile)
    if let Some(ref dir) = last_used {
        if dir != "Default" && available.contains(dir) {
            return Ok(base_dir.join(dir).join("History"));
        }
    }

    // Fallback: alphabetical
    let mut sorted: Vec<&String> = available.iter().collect();
    sorted.sort();
    if let Some(dir) = sorted.into_iter().next() {
        return Ok(base_dir.join(dir).join("History"));
    }

    anyhow::bail!(
        "No History DB found in {}. Set env var or use --profile.",
        base_dir.display()
    )
}

/// Build a human-readable profile description from directory name and Local State.
/// e.g. "Profile 3 (Rabbit)" or just "Profile 2" if no display name set.
pub fn profile_desc(base_dir: &Path, dir_name: &str) -> String {
    let (names, _, _) = read_profile_names(base_dir);
    if let Some(display) = names.get(dir_name) {
        format!("{} ({})", dir_name, display)
    } else {
        dir_name.to_string()
    }
}

pub fn list_profiles(base_dir: &Path) -> Result<()> {
    if !base_dir.exists() {
        anyhow::bail!("Browser data directory not found: {}", base_dir.display());
    }

    let (display_names, order, _last_used) = read_profile_names(base_dir);

    // Collect all profile dirs with History files
    let has_default = base_dir.join("Default/History").exists();

    let entries = std::fs::read_dir(base_dir)?;
    let profile_set: std::collections::HashSet<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
        })
        .filter(|e| e.path().join("History").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if !has_default && profile_set.is_empty() {
        println!("No profiles found in {}", base_dir.display());
        return Ok(());
    }

    // Build ordered list from profiles_order, then any leftovers sorted alphabetically
    let mut found: Vec<(String, String)> = Vec::new();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dir in &order {
        if dir == "Default" && has_default {
            let display = display_names
                .get(dir)
                .cloned()
                .unwrap_or_else(|| "Default".to_string());
            found.push(("Default".to_string(), display));
            used.insert("Default".to_string());
        } else if profile_set.contains(dir) {
            let display = display_names
                .get(dir)
                .cloned()
                .unwrap_or_else(|| dir.clone());
            found.push((dir.clone(), display));
            used.insert(dir.clone());
        }
    }

    // Default not in profiles_order — add it at the start
    if has_default && !used.contains("Default") {
        let display = display_names
            .get("Default")
            .cloned()
            .unwrap_or_else(|| "Default".to_string());
        found.push(("Default".to_string(), display));
        used.insert("Default".to_string());
    }

    // Remaining profiles not in profiles_order — sort alphabetically
    let mut remaining: Vec<String> = profile_set
        .iter()
        .filter(|d| d != &"Default" && !used.contains(d.as_str()))
        .cloned()
        .collect();
    remaining.sort();
    for p in &remaining {
        let display = display_names
            .get(p)
            .cloned()
            .unwrap_or_else(|| p.clone());
        found.push((p.clone(), display));
    }

    for (i, (dir, display)) in found.iter().enumerate() {
        if dir == "Default" || *display == *dir {
            println!("[{}] {}", i + 1, dir);
        } else {
            println!("[{}] {}  ({})", i + 1, dir, display);
        }
    }

    Ok(())
}

/// Read profile display names, ordering, and last-used profile from `Local State` JSON.
/// Returns: (name_map, profiles_order, last_used)
fn read_profile_names(base_dir: &Path) -> (HashMap<String, String>, Vec<String>, Option<String>) {
    let empty = (HashMap::new(), Vec::new(), None);
    let local_state = base_dir.join("Local State");
    let Ok(data) = std::fs::read_to_string(&local_state) else {
        return empty;
    };
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&data) else {
        return empty;
    };
    let profile = match root.get("profile") {
        Some(p) => p,
        None => return empty,
    };

    // Parse display names from info_cache
    let mut names = HashMap::new();
    if let Some(obj) = profile.get("info_cache").and_then(|v| v.as_object()) {
        for (dir, info) in obj {
            if let Some(name) = info.get("name").and_then(|n| n.as_str()) {
                if !name.is_empty() {
                    names.insert(dir.clone(), name.to_string());
                }
            }
        }
    }

    // Parse profiles_order (for listing order)
    let order: Vec<String> = profile
        .get("profiles_order")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse last_used (single most-recent profile, for default selection)
    let last_used = profile
        .get("last_used")
        .and_then(|v| v.as_str())
        .map(String::from);

    (names, order, last_used)
}

pub fn urls(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("last_visit_time", from, to)?;
    let dt = chromium_dt("last_visit_time");

    let mut sql = format!(
        "SELECT url, title, visit_count, typed_count, {} as last_visit FROM urls",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY last_visit_time DESC LIMIT {};", limit));

    output::print_header(sep, &["url", "title", "visit_count", "typed_count", "last_visit_time"]);
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn visits(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("v.visit_time", from, to)?;
    let dt = chromium_dt("v.visit_time");

    let mut sql = format!(
        "SELECT u.url, u.title, {} as visit_time, \
         ROUND(v.visit_duration/1000000.0, 1) as duration_sec, \
         CASE (v.transition & 0xFF) \
           WHEN 0 THEN 'LINK' WHEN 1 THEN 'TYPED' WHEN 2 THEN 'BOOKMARK' \
           WHEN 3 THEN 'AUTO_SUBFRAME' WHEN 4 THEN 'MANUAL_SUBFRAME' \
           WHEN 5 THEN 'GENERATED' WHEN 6 THEN 'AUTO_TOPLEVEL' \
           WHEN 7 THEN 'FORM_SUBMIT' WHEN 8 THEN 'RELOAD' \
           WHEN 9 THEN 'KEYWORD' WHEN 10 THEN 'KEYWORD_GENERATED' \
           ELSE 'OTHER(' || (v.transition & 0xFF) || ')' \
         END as transition_type, \
         COALESCE(fu.url, '') as from_url \
         FROM visits v \
         JOIN urls u ON v.url = u.id \
         LEFT JOIN visits fv ON v.from_visit = fv.id \
         LEFT JOIN urls fu ON fv.url = fu.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY v.visit_time DESC LIMIT {};", limit));

    output::print_header(
        sep,
        &["url", "title", "visit_time", "duration_sec", "transition", "from_url"],
    );
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn searches(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("u.last_visit_time", from, to)?;
    let dt = chromium_dt("u.last_visit_time");

    let mut sql = format!(
        "SELECT k.term, u.url, u.title, {} as last_visit \
         FROM keyword_search_terms k JOIN urls u ON k.url_id = u.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY u.last_visit_time DESC LIMIT {};", limit));

    output::print_header(sep, &["term", "url", "title", "last_visit_time"]);
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn downloads(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("start_time", from, to)?;
    let dt = chromium_dt("start_time");

    let mut sql = format!(
        "SELECT target_path, total_bytes, mime_type, \
         CASE state \
           WHEN 0 THEN 'IN_PROGRESS' WHEN 1 THEN 'COMPLETE' \
           WHEN 2 THEN 'CANCELLED' WHEN 3 THEN 'INTERRUPTED' \
           ELSE 'UNKNOWN(' || state || ')' END as state, \
         {} as start_time, COALESCE(tab_url, '') as tab_url, COALESCE(referrer, '') as referrer \
         FROM downloads",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY start_time DESC LIMIT {};", limit));

    output::print_header(
        sep,
        &[
            "target_path",
            "total_bytes",
            "mime_type",
            "state",
            "start_time",
            "tab_url",
            "referrer",
        ],
    );
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn annotations(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("v.visit_time", from, to)?;
    let dt = chromium_dt("v.visit_time");

    let mut sql = format!(
        "SELECT u.url, u.title, {} as visit_time, \
         COALESCE(ca.categories, '') as categories, \
         COALESCE(ca.page_language, '') as page_language, \
         COALESCE(ca.search_terms, '') as search_terms \
         FROM content_annotations ca \
         JOIN visits v ON ca.visit_id = v.id JOIN urls u ON v.url = u.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY v.visit_time DESC LIMIT {};", limit));

    output::print_header(
        sep,
        &["url", "title", "visit_time", "categories", "page_language", "search_terms"],
    );
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn contexts(
    db_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    format: &str,
) -> Result<()> {
    let sep = output::sep_for_format(format);
    let where_clause = db::build_chromium_date_filter("v.visit_time", from, to)?;
    let dt = chromium_dt("v.visit_time");

    let mut sql = format!(
        "SELECT u.url, u.title, {} as visit_time, \
         ROUND(ctx.total_foreground_duration/1000000.0, 1) as foreground_sec, \
         ctx.response_code, ctx.tab_id, ctx.window_id \
         FROM context_annotations ctx \
         JOIN visits v ON ctx.visit_id = v.id JOIN urls u ON v.url = u.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY v.visit_time DESC LIMIT {};", limit));

    output::print_header(
        sep,
        &["url", "title", "visit_time", "foreground_sec", "response_code", "tab_id", "window_id"],
    );
    db::query_db_formatted(db_path, &sql, sep)
}

pub fn summary(
    db_path: &Path,
    browser_name: &str,
    profile_desc: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<()> {
    let where_clause = db::build_chromium_date_filter("v.visit_time", from, to)?;
    let wc = if !where_clause.is_empty() {
        format!(" WHERE {}", where_clause)
    } else {
        String::new()
    };

    println!("=== {} History Summary ===", browser_name);
    println!("Profile: {}", profile_desc);
    if let Some(f) = from {
        println!("From: {}", f);
    }
    if let Some(t) = to {
        println!("To: {}", t);
    }
    println!();

    println!("--- Basic Stats ---");
    let stats_sql = format!(
        "SELECT COUNT(*), COUNT(DISTINCT v.url), \
         ROUND(SUM(v.visit_duration)/1000000.0/3600, 2) \
         FROM visits v {};",
        wc
    );
    let stats = db::query_db(db_path, &stats_sql, '\t')?;
    if let Some(row) = stats.first() {
        println!("Total visits:    {}", row.first().unwrap_or(&"0".to_string()));
        println!("Unique URLs:     {}", row.get(1).unwrap_or(&"0".to_string()));
        println!(
            "Total duration:  {} hours",
            row.get(2).unwrap_or(&"0".to_string())
        );
    }

    println!();
    println!("--- Top 10 Domains ---");
    let domains_sql = format!(
        "SELECT \
           REPLACE(REPLACE(SUBSTR(u.url, INSTR(u.url,'://')+3),'www.',''), \
             SUBSTR(REPLACE(SUBSTR(u.url, INSTR(u.url,'://')+3),'www.',''), \
               INSTR(REPLACE(SUBSTR(u.url, INSTR(u.url,'://')+3),'www.',''),'/')), '') as domain, \
           COUNT(*) as cnt FROM visits v JOIN urls u ON v.url=u.id {} \
           GROUP BY domain ORDER BY cnt DESC LIMIT 10;",
        wc
    );
    let domains = db::query_db(db_path, &domains_sql, '\t')?;
    for row in domains {
        let domain = row.first().unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<40} {} visits", domain, count);
    }

    println!();
    println!("--- Transition Types ---");
    let transitions_sql = format!(
        "SELECT \
           CASE (v.transition & 0xFF) \
             WHEN 0 THEN 'LINK' WHEN 1 THEN 'TYPED' \
             WHEN 2 THEN 'BOOKMARK' WHEN 7 THEN 'FORM_SUBMIT' \
             WHEN 8 THEN 'RELOAD' WHEN 9 THEN 'KEYWORD' ELSE 'OTHER' \
           END as type, COUNT(*) as cnt FROM visits v {} \
           GROUP BY type ORDER BY cnt DESC;",
        wc
    );
    let transitions = db::query_db(db_path, &transitions_sql, '\t')?;
    for row in transitions {
        let typ = row.first().unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<20} {}", typ, count);
    }

    Ok(())
}
