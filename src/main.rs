mod browser;
mod db;
mod output;

use clap::Parser;

const VERSION: &str = "v0.2.0";

#[derive(Parser)]
#[command(name = "browser-history", version = VERSION, about = "CLI tool to extract browsing history from multiple browsers via SQLite")]
struct Cli {
    #[command(subcommand)]
    browser: BrowserCmd,
}

#[derive(clap::Subcommand)]
enum BrowserCmd {
    /// Google Chrome / Chromium
    Chrome {
        #[command(subcommand)]
        command: ChromeCommand,
    },
    /// Chromium (generic)
    Chromium {
        #[command(subcommand)]
        command: ChromiumCommand,
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

#[derive(clap::Subcommand)]
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

#[derive(clap::Subcommand)]
enum ChromiumCommand {
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

#[derive(clap::Subcommand)]
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

#[derive(clap::Subcommand)]
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

#[derive(clap::Subcommand)]
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
    /// Browser profile name (e.g., "Default", "Profile 1")
    #[arg(short, long)]
    pub profile: Option<String>,
}

#[derive(Parser, Clone)]
pub struct SummaryOpts {
    /// Start date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub from: Option<String>,
    /// End date (inclusive, YYYY-MM-DD)
    #[arg(short, long)]
    pub to: Option<String>,
    /// Browser profile name (e.g., "Default", "Profile 1")
    #[arg(short, long)]
    pub profile: Option<String>,
}

fn parse_format(s: &str) -> Result<String, String> {
    match s {
        "tsv" | "csv" => Ok(s.to_string()),
        _ => Err(format!("invalid format '{}': expected tsv or csv", s)),
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.browser {
        BrowserCmd::Chrome { command } => match command {
            ChromeCommand::Urls(o) => browser::chrome::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Visits(o) => browser::chrome::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Searches(o) => browser::chrome::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Downloads(o) => browser::chrome::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Annotations(o) => browser::chrome::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Contexts(o) => browser::chrome::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromeCommand::Summary(o) => browser::chrome::summary(o.from.as_deref(), o.to.as_deref(), o.profile.as_deref()),
        },
        BrowserCmd::Chromium { command } => match command {
            ChromiumCommand::Urls(o) => browser::chromium_browser::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Visits(o) => browser::chromium_browser::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Searches(o) => browser::chromium_browser::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Downloads(o) => browser::chromium_browser::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Annotations(o) => browser::chromium_browser::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Contexts(o) => browser::chromium_browser::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            ChromiumCommand::Summary(o) => browser::chromium_browser::summary(o.from.as_deref(), o.to.as_deref(), o.profile.as_deref()),
        },
        BrowserCmd::Edge { command } => match command {
            EdgeCommand::Urls(o) => browser::edge::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            EdgeCommand::Visits(o) => browser::edge::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            EdgeCommand::Searches(o) => browser::edge::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            EdgeCommand::Downloads(o) => browser::edge::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, o.profile.as_deref()),
            EdgeCommand::Summary(o) => browser::edge::summary(o.from.as_deref(), o.to.as_deref(), o.profile.as_deref()),
        },
        BrowserCmd::Firefox { command } => match command {
            FirefoxCommand::Urls(o) => browser::firefox::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            FirefoxCommand::Visits(o) => browser::firefox::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            FirefoxCommand::Searches(o) => browser::firefox::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            FirefoxCommand::Bookmarks(o) => browser::firefox::bookmarks(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            FirefoxCommand::Summary(o) => browser::firefox::summary(o.from.as_deref(), o.to.as_deref()),
        },
        BrowserCmd::Safari { command } => match command {
            SafariCommand::Urls(o) => browser::safari::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            SafariCommand::Visits(o) => browser::safari::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
            SafariCommand::Summary(o) => browser::safari::summary(o.from.as_deref(), o.to.as_deref()),
        },
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
