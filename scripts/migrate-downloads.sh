#!/usr/bin/env bash
# Migrate downloads from one Chromium profile to another.
# Tested with: Chromium 149.0.7827.200
#
# Usage: bash migrate-downloads.sh [FROM_PROFILE] [TO_PROFILE]
#
# Defaults: FROM="Profile 2"  TO="Profile 3"
#           (migrate downloads from User → Rabbit)

set -euo pipefail

FROM="${1:-Profile 2}"
TO="${2:-Profile 3}"

DATA_DIR="$HOME/.config/chromium"

SRC_DB="$DATA_DIR/$FROM/History"
DST_DB="$DATA_DIR/$TO/History"
BACKUP="$DATA_DIR/$TO/History.bak.$(date +%Y%m%d_%H%M%S)"

RED='\033[1;31m'
GREEN='\033[1;32m'
NC='\033[0m'

echo "=== Migrate Downloads: $FROM → $TO ==="
echo ""
echo "Source:      $SRC_DB"
echo "Destination: $DST_DB"
echo "Backup:      $BACKUP"
echo ""

# --- Validate -------------------------------------------------
if [ ! -f "$SRC_DB" ]; then
    echo -e "${RED}ERROR: Source DB not found: $SRC_DB${NC}"
    exit 1
fi
if [ ! -f "$DST_DB" ]; then
    echo -e "${RED}ERROR: Destination DB not found: $DST_DB${NC}"
    exit 1
fi

# --- Safety: warn if Chromium might be running ------------------
if pgrep -x chromium >/dev/null 2>&1; then
    echo -e "${RED}WARNING: Chromium appears to be running.${NC}"
    echo "Modifying the History DB while Chromium is open WILL cause corruption."
    echo -n "Continue anyway? [y/N] "
    read -r ans
    case "$ans" in
        [yY]*) ;;
        *) echo "Aborted."; exit 1 ;;
    esac
fi

# --- Pre-flight counts -----------------------------------------
SRC_DL=$(nix-shell -p sqlite --quiet --run "sqlite3 '$SRC_DB' 'SELECT COUNT(*) FROM downloads;'" 2>/dev/null)
DST_DL=$(nix-shell -p sqlite --quiet --run "sqlite3 '$DST_DB' 'SELECT COUNT(*) FROM downloads;'" 2>/dev/null)
SRC_CH=$(nix-shell -p sqlite --quiet --run "sqlite3 '$SRC_DB' 'SELECT COUNT(*) FROM downloads_url_chains;'" 2>/dev/null)
DST_CH=$(nix-shell -p sqlite --quiet --run "sqlite3 '$DST_DB' 'SELECT COUNT(*) FROM downloads_url_chains;'" 2>/dev/null)

echo "Current state:"
echo "  $FROM downloads:     $SRC_DL"
echo "  $TO   downloads:     $DST_DL"
echo "  $FROM url_chains:    $SRC_CH"
echo "  $TO   url_chains:    $DST_CH"
echo ""

if [ "$SRC_DL" -eq 0 ]; then
    echo "No downloads in source profile ($FROM). Nothing to migrate."
    exit 0
fi

EXP_DL=$((DST_DL + SRC_DL))
EXP_CH=$((DST_CH + SRC_CH))

echo "After migration:"
echo "  $TO downloads:  $EXP_DL"
echo "  $TO url_chains: $EXP_CH"
echo ""

# --- Confirm ---------------------------------------------------
echo -n "Proceed? [y/N] "
read -r ans
case "$ans" in
    [yY]*) ;;
    *) echo "Aborted."; exit 0 ;;
esac
echo ""

# --- Backup ----------------------------------------------------
cp "$DST_DB" "$BACKUP"
echo -e "${GREEN}Backup created: $BACKUP${NC}"
echo ""

# --- Write SQL to temp file ------------------------------------
SQL_FILE=$(mktemp /tmp/migrate_downloads_XXXXXX.sql)
chmod 600 "$SQL_FILE"

cat > "$SQL_FILE" <<SQLEND
ATTACH '$SRC_DB' AS src;
ATTACH '$DST_DB' AS dst;

BEGIN;

-- Get current max download id from destination
CREATE TEMP TABLE _offset AS
  SELECT COALESCE((SELECT MAX(id) FROM dst.downloads), 0) AS base;

-- Insert downloads from source (let autoincrement assign new IDs)
INSERT INTO dst.downloads (
  guid, current_path, target_path, start_time, received_bytes,
  total_bytes, state, danger_type, interrupt_reason, hash, end_time,
  opened, last_access_time, transient, referrer, site_url,
  embedder_download_data, tab_url, tab_referrer_url, http_method,
  by_ext_id, by_ext_name, by_web_app_id, etag, last_modified,
  mime_type, original_mime_type
)
SELECT
  guid, current_path, target_path, start_time, received_bytes,
  total_bytes, state, danger_type, interrupt_reason, hash, end_time,
  opened, last_access_time, transient, referrer, site_url,
  embedder_download_data, tab_url, tab_referrer_url, http_method,
  by_ext_id, by_ext_name, by_web_app_id, etag, last_modified,
  mime_type, original_mime_type
FROM src.downloads
ORDER BY id;

-- Build old_id -> new_id mapping
-- (inserted sequentially, so new ids = base + row_number)
CREATE TEMP TABLE _map AS
  SELECT
    s.id                                               AS old_id,
    (SELECT base FROM _offset) + ROW_NUMBER() OVER ()  AS new_id
  FROM src.downloads s
  ORDER BY s.id;

-- Remap and insert url chains
INSERT INTO dst.downloads_url_chains (id, chain_index, url)
  SELECT m.new_id, c.chain_index, c.url
  FROM src.downloads_url_chains c
  JOIN _map m ON c.id = m.old_id;

-- Cleanup
DROP TABLE _offset;
DROP TABLE _map;

COMMIT;
SQLEND

# --- Run migration ---------------------------------------------
echo "Migrating..."
nix-shell -p sqlite --quiet --run "sqlite3 :memory: '.read $SQL_FILE'"

rm -f "$SQL_FILE"

# --- Post-flight counts ----------------------------------------
echo ""
NEW_DL=$(nix-shell -p sqlite --quiet --run "sqlite3 '$DST_DB' 'SELECT COUNT(*) FROM downloads;'" 2>/dev/null)
NEW_CH=$(nix-shell -p sqlite --quiet --run "sqlite3 '$DST_DB' 'SELECT COUNT(*) FROM downloads_url_chains;'" 2>/dev/null)

if [ "$NEW_DL" -eq "$EXP_DL" ] && [ "$NEW_CH" -eq "$EXP_CH" ]; then
    echo -e "${GREEN}✓ Success.${NC}"
    echo "  $TO downloads:     $DST_DL → $NEW_DL"
    echo "  $TO url_chains:    $DST_CH → $NEW_CH"
    echo ""
    echo "  Restore backup with: cp '$BACKUP' '$DST_DB'"
else
    echo -e "${RED}⚠ Row count mismatch!${NC}"
    echo "  downloads:     expected $EXP_DL, got $NEW_DL"
    echo "  url_chains:    expected $EXP_CH, got $NEW_CH"
    echo ""
    echo -e "${RED}Restore from backup: cp '$BACKUP' '$DST_DB'${NC}"
    exit 1
fi
