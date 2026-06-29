# browser-history-cli

CLI tool to extract browsing history from multiple browsers via SQLite.

Supports: **Chrome**, **Edge**, **Firefox**, **Safari**

Written in Rust.

## Requirements

- Rust toolchain (cargo)

## Install

```bash
git clone <repo-url>
cd browser-history-cli
cargo build --release
```

The binary will be at `target/release/browser-history`. Note that on NixOS
this binary is dynamically linked against the Nix store's glibc and will **not**
run on other distros.

### Static (portable) build

To produce a fully static binary that runs on any x86_64 Linux distro
(no glibc / Nix store dependency), build against the musl target:

```bash
./build-static.sh
```

The static binary will be at
`target/x86_64-unknown-linux-musl/release/browser-history`.

```console
$ file target/x86_64-unknown-linux-musl/release/browser-history
ELF 64-bit LSB pie executable, x86-64, static-pie linked, stripped
$ ldd target/x86_64-unknown-linux-musl/release/browser-history
        statically linked
```

It's ~3 MB and has no runtime dependencies — just copy it to the target machine.

On NixOS the script bootstraps a `rustup` toolchain and a musl-targeting GCC via
`nix-shell` automatically. On other distros, install `rustup` (with the
`x86_64-unknown-linux-musl` target) and a musl C compiler (`musl-tools`), then
run the script.

## Usage

```bash
browser-history <browser> <command> [options]
```

### Supported Browsers

| Browser | Subcommand | DB Format |
|---------|-----------|-----------|
| Google Chrome | `chrome` | Chromium SQLite (WebKit timestamp) |
| Microsoft Edge | `edge` | Chromium SQLite (same as Chrome) |
| Mozilla Firefox | `firefox` | Mozilla places.sqlite (PRTime) |
| Apple Safari | `safari` | Core Data SQLite (macOS only) |

### Commands by Browser

| Command | Chrome | Edge | Firefox | Safari |
|---------|--------|------|---------|--------|
| `urls` | Yes | Yes | Yes | Yes |
| `visits` | Yes | Yes | Yes | Yes |
| `searches` | Yes | Yes | Yes | - |
| `downloads` | Yes | Yes | - | - |
| `annotations` | Yes | - | - | - |
| `contexts` | Yes | - | - | - |
| `bookmarks` | - | - | Yes | - |
| `summary` | Yes | Yes | Yes | Yes |

### Common Options

```
--from, -f <YYYY-MM-DD>    Start date (inclusive)
--to, -t <YYYY-MM-DD>      End date (inclusive)
--limit, -n <number>       Max rows (default: 100)
--format <tsv|csv>         Output format (default: tsv)
```

### Examples

```bash
# Chrome: List URLs visited in the last week (CSV)
browser-history chrome urls -f 2026-03-02 -t 2026-03-09 --format csv

# Edge: Search keywords
browser-history edge searches -n 50

# Firefox: Visit records with transition tracking
browser-history firefox visits -f 2026-03-01 -t 2026-03-09

# Firefox: Bookmarks
browser-history firefox bookmarks --format csv

# Safari: Summary statistics
browser-history safari summary -f 2026-03-01

# Pipe-friendly: extract with awk
browser-history chrome visits --format csv | awk -F, 'NR>1{print $3, $1}'

# Pipe-friendly: process with read
browser-history firefox urls --format csv | while IFS=',' read -r url title count last; do
  echo "$title ($count visits)"
done
```

### Custom DB Path

Override auto-detected paths with environment variables:

```bash
export CHROME_HISTORY_DB="/path/to/History"
export EDGE_HISTORY_DB="/path/to/History"
export FIREFOX_HISTORY_DB="/path/to/places.sqlite"
export SAFARI_HISTORY_DB="/path/to/History.db"
```

### Default DB Paths

**Chrome:**
- macOS: `~/Library/Application Support/Google/Chrome/Default/History`
- Linux: `~/.config/google-chrome/Default/History`
- Windows: `%LOCALAPPDATA%/Google/Chrome/User Data/Default/History`

**Edge:**
- macOS: `~/Library/Application Support/Microsoft Edge/Default/History`
- Linux: `~/.config/microsoft-edge/Default/History`
- Windows: `%LOCALAPPDATA%/Microsoft/Edge/User Data/Default/History`

**Firefox:**
- macOS: `~/Library/Application Support/Firefox/Profiles/*.default-release/places.sqlite`
- Linux: `~/.mozilla/firefox/*.default-release/places.sqlite`
- Windows: `%APPDATA%/Mozilla/Firefox/Profiles/*.default-release/places.sqlite`

**Safari:**
- macOS: `~/Library/Safari/History.db` (requires Full Disk Access)

## Architecture

```
browser-history-cli/
├── Cargo.toml
└── src/
    ├── main.rs          # CLI entry point (clap-based argument parsing + dispatch)
    ├── db.rs            # Shared DB utilities (copy to temp, date conversion, query)
    ├── output.rs        # TSV/CSV output formatting
    └── browser/
        ├── mod.rs       # Module declarations
        ├── chromium.rs  # Shared Chrome/Edge SQL queries (7 commands)
        ├── chrome.rs    # Chrome DB path detection + dispatch
        ├── edge.rs      # Edge DB path detection + dispatch
        ├── firefox.rs   # Firefox module (Mozilla places.sqlite schema)
        └── safari.rs    # Safari module (Core Data schema, macOS only)
```

## Output Format

- Default: TSV (tab-separated) for `awk` processing
- Optional: CSV with `--format csv` for spreadsheets and `read` loops
- First line is always a header row

## License

MIT
