mod browser;
mod db;
mod output;

use clap::{Parser, Subcommand};

const VERSION: &str = "v0.2.0";

#[derive(Parser)]
#[command(name = "browser-history", version = VERSION, about = "CLI tool to extract browsing history from multiple browsers via SQLite")]
struct Cli {
    #[command(subcommand)]
    browser: BrowserCmd,
}

#[derive(Subcommand)]
enum BrowserCmd {
    /// Google Chrome / Chromium
    Chrome {
        #[command(subcommand)]
        command: ChromeCommand,
    },
    /// Microsoft Edge
    Edge {
        #[command(subcommand)]
        command: EdgeCommand,
    },
    /// Mozilla Firefox
    Firefox {
        #[command(subcommand)]
        command: FirefoxCommand,
    },
    /// Apple Safari (macOS only)
    Safari {
        #[command(subcommand)]
        command: SafariCommand,
    },
}

#[derive(Subcommand)]
enum ChromeCommand {
    /// Extract visited URLs
    Urls(CommonOpts),
    /// Extract individual visit records
    Visits(CommonOpts),
    /// Extract search keywords
    Searches(CommonOpts),
    /// Extract download history
    Downloads(CommonOpts),
    /// Extract content annotations
    Annotations(CommonOpts),
    /// Extract context annotations
    Contexts(CommonOpts),
    /// Show summary statistics
    Summary(SummaryOpts),
}

#[derive(Subcommand)]
enum EdgeCommand {
    /// Extract visited URLs
    Urls(CommonOpts),
    /// Extract individual visit records
    Visits(CommonOpts),
    /// Extract search keywords
    Searches(CommonOpts),
    /// Extract download history
    Downloads(CommonOpts),
    /// Show summary statistics
    Summary(SummaryOpts),
}

#[derive(Subcommand)]
enum FirefoxCommand {
    /// Extract visited URLs
    Urls(CommonOpts),
    /// Extract individual visit records
    Visits(CommonOpts),
    /// Extract search keywords
    Searches(CommonOpts),
    /// Extract bookmarks
    Bookmarks(CommonOpts),
    /// Show summary statistics
    Summary(SummaryOpts),
}

#[derive(Subcommand)]
enum SafariCommand {
    /// Extract visited URLs
    Urls(CommonOpts),
    /// Extract individual visit records
    Visits(CommonOpts),
    /// Show summary statistics
    Summary(SummaryOpts),
}

#[derive(Parser, Clone)]
pub struct CommonOpts {
    /// Start date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub from: Option<String>,
    /// End date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub to: Option<String>,
    /// Max rows
    #[arg(short = 'n', long, default_value = "100")]
    pub limit: i64,
    /// Output format
    #[arg(long, default_value = "tsv", value_parser = parse_format)]
    pub format: String,
}

#[derive(Parser, Clone)]
pub struct SummaryOpts {
    /// Start date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub from: Option<String>,
    /// End date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub to: Option<String>,
}

fn parse_format(s: &str) -> Result<String, String> {
    match s {
        "tsv" | "csv" => Ok(s.to_string()),
        _ => Err(format!("invalid format '{}': expected tsv or csv", s)),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.browser {
        BrowserCmd::Chrome { command } => match command {
            ChromeCommand::Urls(_opts) => todo!("chrome urls"),
            ChromeCommand::Visits(_opts) => todo!("chrome visits"),
            ChromeCommand::Searches(_opts) => todo!("chrome searches"),
            ChromeCommand::Downloads(_opts) => todo!("chrome downloads"),
            ChromeCommand::Annotations(_opts) => todo!("chrome annotations"),
            ChromeCommand::Contexts(_opts) => todo!("chrome contexts"),
            ChromeCommand::Summary(_opts) => todo!("chrome summary"),
        },
        BrowserCmd::Edge { command } => match command {
            EdgeCommand::Urls(_opts) => todo!("edge urls"),
            EdgeCommand::Visits(_opts) => todo!("edge visits"),
            EdgeCommand::Searches(_opts) => todo!("edge searches"),
            EdgeCommand::Downloads(_opts) => todo!("edge downloads"),
            EdgeCommand::Summary(_opts) => todo!("edge summary"),
        },
        BrowserCmd::Firefox { command } => match command {
            FirefoxCommand::Urls(_opts) => todo!("firefox urls"),
            FirefoxCommand::Visits(_opts) => todo!("firefox visits"),
            FirefoxCommand::Searches(_opts) => todo!("firefox searches"),
            FirefoxCommand::Bookmarks(_opts) => todo!("firefox bookmarks"),
            FirefoxCommand::Summary(_opts) => todo!("firefox summary"),
        },
        BrowserCmd::Safari { command } => match command {
            SafariCommand::Urls(_opts) => todo!("safari urls"),
            SafariCommand::Visits(_opts) => todo!("safari visits"),
            SafariCommand::Summary(_opts) => todo!("safari summary"),
        },
    }
}
