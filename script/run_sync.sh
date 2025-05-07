#!/bin/bash

# Location of your sync binary
SYNC_BIN="/usr/local/bin/load_all"

# Log file (make sure it's writable by the container user)
LOG_FILE="/var/log/cron.log"

# Timestamp for logs
timestamp() {
  date +"%Y-%m-%d %H:%M:%S"
}

echo "[`timestamp`] Running sync job..." >> "$LOG_FILE"

$SYNC_BIN >> "$LOG_FILE" 2>&1
STATUS=$?

if [ $STATUS -eq 0 ]; then
  echo "[`timestamp`] ✅ Sync completed successfully" >> "$LOG_FILE"
else
  echo "[`timestamp`] ❌ Sync failed with exit code $STATUS" >> "$LOG_FILE"
fi
