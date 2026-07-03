mod browser;
mod db;
mod output;

use anyhow::Context;
use clap::{CommandFactory, Parser};

const VERSION: &str = "v0.2.0";

#[derive(Parser)]
#[command(
    name = "browser-history",
    version = VERSION,
    about = "Extract browsing history from Chrome, Chromium, Edge, Firefox, and Safari via their local SQLite databases",
    long_about = "A CLI tool to query browsing history from multiple browsers. \
                  Supports URLs, visits, searches, downloads, bookmarks, and summary statistics. \
                  Outputs in TSV or CSV format with optional date filtering.",
    subcommand_required = true
)]
struct Cli {
    #[command(subcommand)]
    browser: BrowserCmd,
}

#[derive(clap::Subcommand)]
enum BrowserCmd {
    /// Query Google Chrome history
    Chrome {
        /// Profile directory name (e.g. "Default", "Profile 2")
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<ChromeCommand>,
    },
    /// Query Chromium history
    Chromium {
        /// Profile directory name (e.g. "Default", "Profile 2")
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<ChromiumCommand>,
    },
    /// Query Microsoft Edge history
    Edge {
        /// Profile directory name (e.g. "Default", "Profile 2")
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<EdgeCommand>,
    },
    /// Query Mozilla Firefox history
    Firefox {
        #[command(subcommand)]
        command: Option<FirefoxCommand>,
    },
    /// Query Apple Safari history (macOS only)
    Safari {
        #[command(subcommand)]
        command: Option<SafariCommand>,
    },
}

/// Print the full --help for a specific browser subcommand and exit.
fn print_browser_help(name: &str) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    let mut sub = cmd
        .find_subcommand_mut(name)
        .with_context(|| format!("no such browser: {}", name))?
        .clone()
        .override_usage(format!("browser-history {} [OPTIONS] <COMMAND>", name));
    sub.print_help()?;
    std::process::exit(2);
}

#[derive(clap::Subcommand)]
enum ChromeCommand {
    /// List available profiles with display names
    Profiles,
    /// Show visited URLs with title and visit counts
    Urls(CommonOpts),
    /// Show individual page visits with duration and transition type
    Visits(CommonOpts),
    /// Show search terms from the omnibox / address bar
    Searches(CommonOpts),
    /// Show downloaded files
    Downloads(CommonOpts),
    /// Show page content annotations (language, categories)
    Annotations(CommonOpts),
    /// Show tab/window context annotations
    Contexts(CommonOpts),
    /// Show aggregate history statistics
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum ChromiumCommand {
    /// List available profiles with display names
    Profiles,
    /// Show visited URLs with title and visit counts
    Urls(CommonOpts),
    /// Show individual page visits with duration and transition type
    Visits(CommonOpts),
    /// Show search terms from the omnibox / address bar
    Searches(CommonOpts),
    /// Show downloaded files
    Downloads(CommonOpts),
    /// Show page content annotations (language, categories)
    Annotations(CommonOpts),
    /// Show tab/window context annotations
    Contexts(CommonOpts),
    /// Show aggregate history statistics
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum EdgeCommand {
    /// List available profiles with display names
    Profiles,
    /// Show visited URLs with title and visit counts
    Urls(CommonOpts),
    /// Show individual page visits with duration and transition type
    Visits(CommonOpts),
    /// Show search terms from the omnibox / address bar
    Searches(CommonOpts),
    /// Show downloaded files
    Downloads(CommonOpts),
    /// Show aggregate history statistics
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum FirefoxCommand {
    /// Show visited URLs with title and visit counts
    Urls(CommonOpts),
    /// Show individual page visits with duration and transition type
    Visits(CommonOpts),
    /// Show search terms from the search bar
    Searches(CommonOpts),
    /// Show bookmarked pages
    Bookmarks(CommonOpts),
    /// Show aggregate history statistics
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum SafariCommand {
    /// Show visited URLs with title and visit counts
    Urls(CommonOpts),
    /// Show individual page visits
    Visits(CommonOpts),
    /// Show aggregate history statistics
    Summary(SummaryOpts),
}

#[derive(Parser, Clone)]
pub struct CommonOpts {
    /// Start date (YYYY-MM-DD, inclusive)
    #[arg(short, long)]
    pub from: Option<String>,
    /// End date (YYYY-MM-DD, inclusive)
    #[arg(short, long)]
    pub to: Option<String>,
    /// Maximum number of results
    #[arg(short = 'n', long, default_value = "100")]
    pub limit: i64,
    /// Output format: tsv or csv
    #[arg(long, default_value = "tsv", value_parser = parse_format)]
    pub format: String,
}

#[derive(Parser, Clone)]
pub struct SummaryOpts {
    /// Start date (YYYY-MM-DD, inclusive)
    #[arg(short, long)]
    pub from: Option<String>,
    /// End date (YYYY-MM-DD, inclusive)
    #[arg(short, long)]
    pub to: Option<String>,
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
        BrowserCmd::Chrome { profile, command } => {
            let p = profile.as_deref();
            match command {
                Some(ChromeCommand::Profiles) => browser::chrome::list_profiles(),
                Some(ChromeCommand::Urls(o)) => browser::chrome::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Visits(o)) => browser::chrome::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Searches(o)) => browser::chrome::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Downloads(o)) => browser::chrome::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Annotations(o)) => browser::chrome::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Contexts(o)) => browser::chrome::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromeCommand::Summary(o)) => browser::chrome::summary(o.from.as_deref(), o.to.as_deref(), p),
                None => print_browser_help("chrome"),
            }
        }
        BrowserCmd::Chromium { profile, command } => {
            let p = profile.as_deref();
            match command {
                Some(ChromiumCommand::Profiles) => browser::chromium::list_profiles(),
                Some(ChromiumCommand::Urls(o)) => browser::chromium::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Visits(o)) => browser::chromium::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Searches(o)) => browser::chromium::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Downloads(o)) => browser::chromium::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Annotations(o)) => browser::chromium::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Contexts(o)) => browser::chromium::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(ChromiumCommand::Summary(o)) => browser::chromium::summary(o.from.as_deref(), o.to.as_deref(), p),
                None => print_browser_help("chromium"),
            }
        }
        BrowserCmd::Edge { profile, command } => {
            let p = profile.as_deref();
            match command {
                Some(EdgeCommand::Profiles) => browser::edge::list_profiles(),
                Some(EdgeCommand::Urls(o)) => browser::edge::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(EdgeCommand::Visits(o)) => browser::edge::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(EdgeCommand::Searches(o)) => browser::edge::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(EdgeCommand::Downloads(o)) => browser::edge::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                Some(EdgeCommand::Summary(o)) => browser::edge::summary(o.from.as_deref(), o.to.as_deref(), p),
                None => print_browser_help("edge"),
            }
        }
        BrowserCmd::Firefox { command } => {
            match command {
                Some(FirefoxCommand::Urls(o)) => browser::firefox::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(FirefoxCommand::Visits(o)) => browser::firefox::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(FirefoxCommand::Searches(o)) => browser::firefox::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(FirefoxCommand::Bookmarks(o)) => browser::firefox::bookmarks(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(FirefoxCommand::Summary(o)) => browser::firefox::summary(o.from.as_deref(), o.to.as_deref()),
                None => print_browser_help("firefox"),
            }
        }
        BrowserCmd::Safari { command } => {
            match command {
                Some(SafariCommand::Urls(o)) => browser::safari::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(SafariCommand::Visits(o)) => browser::safari::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                Some(SafariCommand::Summary(o)) => browser::safari::summary(o.from.as_deref(), o.to.as_deref()),
                None => print_browser_help("safari"),
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
