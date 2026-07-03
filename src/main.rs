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
    Chrome {
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<ChromeCommand>,
    },
    Chromium {
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<ChromiumCommand>,
    },
    Edge {
        #[arg(short, long)]
        profile: Option<String>,
        #[command(subcommand)]
        command: Option<EdgeCommand>,
    },
    Firefox {
        #[command(subcommand)]
        command: Option<FirefoxCommand>,
    },
    Safari {
        #[command(subcommand)]
        command: Option<SafariCommand>,
    },
}

#[derive(clap::Subcommand)]
enum ChromeCommand {
    Profiles,
    Urls(CommonOpts),
    Visits(CommonOpts),
    Searches(CommonOpts),
    Downloads(CommonOpts),
    Annotations(CommonOpts),
    Contexts(CommonOpts),
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum ChromiumCommand {
    Profiles,
    Urls(CommonOpts),
    Visits(CommonOpts),
    Searches(CommonOpts),
    Downloads(CommonOpts),
    Annotations(CommonOpts),
    Contexts(CommonOpts),
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum EdgeCommand {
    Profiles,
    Urls(CommonOpts),
    Visits(CommonOpts),
    Searches(CommonOpts),
    Downloads(CommonOpts),
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum FirefoxCommand {
    Urls(CommonOpts),
    Visits(CommonOpts),
    Searches(CommonOpts),
    Bookmarks(CommonOpts),
    Summary(SummaryOpts),
}

#[derive(clap::Subcommand)]
enum SafariCommand {
    Urls(CommonOpts),
    Visits(CommonOpts),
    Summary(SummaryOpts),
}

#[derive(Parser, Clone)]
pub struct CommonOpts {
    #[arg(short, long)]
    pub from: Option<String>,
    #[arg(short, long)]
    pub to: Option<String>,
    #[arg(short = 'n', long, default_value = "100")]
    pub limit: i64,
    #[arg(long, default_value = "tsv", value_parser = parse_format)]
    pub format: String,
}

#[derive(Parser, Clone)]
pub struct SummaryOpts {
    #[arg(short, long)]
    pub from: Option<String>,
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
            match command.ok_or_else(|| anyhow::anyhow!("No command specified. Use --help for usage."))? {
                ChromeCommand::Profiles => browser::chrome::list_profiles(),
                ChromeCommand::Urls(o) => browser::chrome::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Visits(o) => browser::chrome::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Searches(o) => browser::chrome::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Downloads(o) => browser::chrome::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Annotations(o) => browser::chrome::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Contexts(o) => browser::chrome::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromeCommand::Summary(o) => browser::chrome::summary(o.from.as_deref(), o.to.as_deref(), p),
            }
        }
        BrowserCmd::Chromium { profile, command } => {
            let p = profile.as_deref();
            match command.ok_or_else(|| anyhow::anyhow!("No command specified. Use --help for usage."))? {
                ChromiumCommand::Profiles => browser::chromium::list_profiles(),
                ChromiumCommand::Urls(o) => browser::chromium::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Visits(o) => browser::chromium::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Searches(o) => browser::chromium::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Downloads(o) => browser::chromium::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Annotations(o) => browser::chromium::annotations(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Contexts(o) => browser::chromium::contexts(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                ChromiumCommand::Summary(o) => browser::chromium::summary(o.from.as_deref(), o.to.as_deref(), p),
            }
        }
        BrowserCmd::Edge { profile, command } => {
            let p = profile.as_deref();
            match command.ok_or_else(|| anyhow::anyhow!("No command specified. Use --help for usage."))? {
                EdgeCommand::Profiles => browser::edge::list_profiles(),
                EdgeCommand::Urls(o) => browser::edge::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                EdgeCommand::Visits(o) => browser::edge::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                EdgeCommand::Searches(o) => browser::edge::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                EdgeCommand::Downloads(o) => browser::edge::downloads(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format, p),
                EdgeCommand::Summary(o) => browser::edge::summary(o.from.as_deref(), o.to.as_deref(), p),
            }
        }
        BrowserCmd::Firefox { command } => {
            match command.ok_or_else(|| anyhow::anyhow!("No command specified. Use --help for usage."))? {
                FirefoxCommand::Urls(o) => browser::firefox::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                FirefoxCommand::Visits(o) => browser::firefox::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                FirefoxCommand::Searches(o) => browser::firefox::searches(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                FirefoxCommand::Bookmarks(o) => browser::firefox::bookmarks(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                FirefoxCommand::Summary(o) => browser::firefox::summary(o.from.as_deref(), o.to.as_deref()),
            }
        }
        BrowserCmd::Safari { command } => {
            match command.ok_or_else(|| anyhow::anyhow!("No command specified. Use --help for usage."))? {
                SafariCommand::Urls(o) => browser::safari::urls(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                SafariCommand::Visits(o) => browser::safari::visits(o.from.as_deref(), o.to.as_deref(), o.limit, &o.format),
                SafariCommand::Summary(o) => browser::safari::summary(o.from.as_deref(), o.to.as_deref()),
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
