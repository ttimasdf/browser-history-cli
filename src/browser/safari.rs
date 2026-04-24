use anyhow::Result;
use std::path::PathBuf;

use crate::db;
use crate::output;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn get_db_path() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("SAFARI_HISTORY_DB") {
        return Ok(PathBuf::from(custom));
    }
    Ok(home_dir().join("Library/Safari/History.db"))
}

fn prepared_db() -> Result<PathBuf> {
    if !cfg!(target_os = "macos") && std::env::var("SAFARI_HISTORY_DB").is_err() {
        anyhow::bail!("Safari is only available on macOS. Set SAFARI_HISTORY_DB env var if you have a Safari History.db file.");
    }
    let path = get_db_path()?;
    db::prepare_db(&path).map_err(|_| {
        anyhow::anyhow!(
            "Safari history DB not found: {}. Set SAFARI_HISTORY_DB env var. \
             Note: macOS Mojave+ requires Full Disk Access permission.",
            path.display()
        )
    })
}

const SAFARI_DT_EXPR: &str = "datetime({} + 978307200, 'unixepoch', 'localtime')";

fn safari_dt(col: &str) -> String {
    SAFARI_DT_EXPR.replace("{}", col)
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

    let date_filter = if from.is_some() || to.is_some() {
        let visit_where = db::build_safari_date_filter("hv.visit_time", from, to)?;
        if !visit_where.is_empty() {
            format!("AND hi.id IN (SELECT hv.history_item FROM history_visits hv WHERE {})", visit_where)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let dt = safari_dt("hi.visit_count_score");
    let sql = format!(
        "SELECT hi.url, COALESCE(hv_title.title, '') as title, hi.visit_count, \
         {} as last_visit \
         FROM history_items hi \
         LEFT JOIN ( \
           SELECT history_item, title, \
             ROW_NUMBER() OVER (PARTITION BY history_item ORDER BY visit_time DESC) as rn \
           FROM history_visits WHERE title IS NOT NULL AND title != '' \
         ) hv_title ON hi.id = hv_title.history_item AND hv_title.rn = 1 \
         WHERE hi.visit_count > 0 {} \
         ORDER BY hi.visit_count_score DESC \
         LIMIT {};",
        dt, date_filter, limit
    );

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
    let where_clause = db::build_safari_date_filter("hv.visit_time", from, to)?;
    let dt = safari_dt("hv.visit_time");

    let mut sql = format!(
        "SELECT hi.url, COALESCE(hv.title, '') as title, \
         {} as visit_time, \
         COALESCE(rs.url, '') as redirect_source, \
         COALESCE(rd.url, '') as redirect_destination \
         FROM history_visits hv \
         JOIN history_items hi ON hv.history_item = hi.id \
         LEFT JOIN history_visits rsv ON hv.redirect_source = rsv.id \
         LEFT JOIN history_items rs ON rsv.history_item = rs.id \
         LEFT JOIN history_visits rdv ON hv.redirect_destination = rdv.id \
         LEFT JOIN history_items rd ON rdv.history_item = rd.id",
        dt
    );
    if !where_clause.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    sql.push_str(&format!(" ORDER BY hv.visit_time DESC LIMIT {};", limit));

    output::print_header(sep, &["url", "title", "visit_time", "redirect_source", "redirect_destination"]);
    db::query_db_formatted(&db_path, &sql, sep)
}

pub fn summary(
    from: Option<&str>,
    to: Option<&str>,
) -> Result<()> {
    let db_path = prepared_db()?;
    let _guard = CleanupGuard(db_path.clone());
    let where_clause = db::build_safari_date_filter("hv.visit_time", from, to)?;
    let wc = if !where_clause.is_empty() {
        format!(" WHERE {}", where_clause)
    } else {
        String::new()
    };

    println!("=== Safari History Summary ===");
    if let Some(f) = from {
        println!("From: {}", f);
    }
    if let Some(t) = to {
        println!("To: {}", t);
    }
    println!();

    println!("--- Basic Stats ---");
    let stats_sql = format!(
        "SELECT COUNT(*), COUNT(DISTINCT hv.history_item) FROM history_visits hv {};",
        wc
    );
    let stats = db::query_db(&db_path, &stats_sql, '\t')?;
    if let Some(row) = stats.first() {
        println!("Total visits:    {}", row.get(0).unwrap_or(&"0".to_string()));
        println!("Unique URLs:     {}", row.get(1).unwrap_or(&"0".to_string()));
    }

    println!();
    println!("--- Top 10 Domains ---");
    let domains_sql = format!(
        "SELECT \
           REPLACE(REPLACE(SUBSTR(hi.url, INSTR(hi.url,'://')+3),'www.',''), \
             SUBSTR(REPLACE(SUBSTR(hi.url, INSTR(hi.url,'://')+3),'www.',''), \
               INSTR(REPLACE(SUBSTR(hi.url, INSTR(hi.url,'://')+3),'www.',''),'/')), '') as domain, \
           COUNT(*) as cnt \
           FROM history_visits hv \
           JOIN history_items hi ON hv.history_item = hi.id {} \
           GROUP BY domain ORDER BY cnt DESC LIMIT 10;",
        wc
    );
    let domains = db::query_db(&db_path, &domains_sql, '\t')?;
    for row in domains {
        let domain = row.get(0).unwrap_or(&"".to_string()).clone();
        let count = row.get(1).unwrap_or(&"0".to_string()).clone();
        println!("  {:<40} {} visits", domain, count);
    }

    Ok(())
}

struct CleanupGuard(PathBuf);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
