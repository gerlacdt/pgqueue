#!/bin/bash

set -x
set -eo pipefail

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=postgres}"
DB_NAME="${POSTGRES_DB:=pgqueuedb}"
DB_PORT="${POSTGRES_PORT:=5433}" # don't use default port
export PGPASSWORD="${DB_PASSWORD}"
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
export DATABASE_URL

psql -U $DB_USER -h localhost -d $DB_NAME -p $DB_PORT -f $1
