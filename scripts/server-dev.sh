#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

load_env() {
  if [[ -f "server/.env" ]]; then
    set -a
    source "server/.env"
    set +a
  fi

  : "${SQLITE_PATH:=monitor.db}"
  : "${SERVER_HOST:=0.0.0.0}"
  : "${SERVER_PORT:=3000}"

  if [[ -z "${BIND_ADDRESS:-}" ]]; then
    export BIND_ADDRESS="${SERVER_HOST}:${SERVER_PORT}"
  fi

  if [[ -z "${DATABASE_URL:-}" ]]; then
    export DATABASE_URL="sqlite:${SQLITE_PATH}"
  fi

  export SQLITE_PATH
}

db_path_from_url() {
  local url="$1"
  url="${url#sqlite:///}"
  url="${url#sqlite://}"
  url="${url#sqlite:}"
  url="${url%\?mode=rwc}"
  printf '%s\n' "$url"
}

sqlx_migrations_schema() {
  cat <<'SQL'
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
SQL
}

has_sqlx_tracking() {
  sqlite3 "$1" "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations' LIMIT 1;" | grep -q 1
}

is_empty_schema() {
  local count
  count="$(sqlite3 "$1" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations';")"
  [[ "$count" == "0" ]]
}

has_final_schema() {
  local db="$1"
  local checks=(
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='nodes';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='metrics';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='users';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='refresh_tokens';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='alert_rules';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='alerts';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='services';"
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='service_status';"
    "SELECT 1 FROM pragma_table_info('nodes') WHERE name='is_public';"
    "SELECT 1 FROM pragma_table_info('services') WHERE name='is_public';"
  )

  for query in "${checks[@]}"; do
    if [[ "$(sqlite3 "$db" "$query")" != "1" ]]; then
      return 1
    fi
  done

  return 0
}

record_migration() {
  local db="$1"
  local file="$2"
  local basename version description checksum
  basename="$(basename "$file")"
  version="${basename%%_*}"
  description="${basename#*_}"
  description="${description%.sql}"
  checksum="$(shasum -a 384 "$file" | awk '{print $1}')"

  sqlite3 "$db" <<SQL
INSERT OR IGNORE INTO _sqlx_migrations (version, description, success, checksum, execution_time)
VALUES (${version}, '${description}', 1, X'${checksum}', 0);
SQL
}

bootstrap_db() {
  local db_path
  db_path="$(db_path_from_url "$DATABASE_URL")"
  mkdir -p "$(dirname "$db_path")"
  touch "$db_path"

  if has_sqlx_tracking "$db_path"; then
    return 0
  fi

  sqlite3 "$db_path" "$(sqlx_migrations_schema)"

  if is_empty_schema "$db_path"; then
    for migration in server/migrations/*.sql; do
      sqlite3 "$db_path" < "$migration"
      record_migration "$db_path" "$migration"
    done
    return 0
  fi

  if has_final_schema "$db_path"; then
    for migration in server/migrations/*.sql; do
      record_migration "$db_path" "$migration"
    done
    return 0
  fi

  cat >&2 <<EOF
Database '$db_path' has partial schema but no _sqlx_migrations metadata.
Please back it up and remove it, or complete the schema manually before retrying.
EOF
  return 1
}

run_check() {
  cargo check -p vespera-server
}

run_server() {
  cargo run -p vespera-server
}

usage() {
  cat <<'EOF'
Usage:
  bash scripts/server-dev.sh check
  bash scripts/server-dev.sh run
  bash scripts/server-dev.sh bootstrap
EOF
}

main() {
  load_env
  case "${1:-}" in
    check)
      bootstrap_db
      run_check
      ;;
    run)
      bootstrap_db
      run_server
      ;;
    bootstrap)
      bootstrap_db
      printf 'Bootstrapped %s\n' "$(db_path_from_url "$DATABASE_URL")"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "${1:-}"
