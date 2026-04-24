use anyhow::Result;
use std::path::Path;

use crate::db;
use crate::output;

const CHROMIUM_DT_EXPR: &str = "datetime({}/1000000 - 11644473600, 'unixepoch', 'localtime')";

fn chromium_dt(col: &str) -> String {
    CHROMIUM_DT_EXPR.replace("{}", col)
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
        println!("Total visits:    {}", row.get(0).unwrap_or(&"0".to_string()));
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
        let domain = row.get(0).unwrap_or(&"".to_string()).clone();
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
        let typ = row.get(0).unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<20} {}", typ, count);
    }

    Ok(())
}
