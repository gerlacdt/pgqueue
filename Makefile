.PHONY=build release run dev test migrate start-db
SHELL := /bin/bash

build:
	set -a && source files/env/dev.env && cargo clippy && cargo build

release:
	set -a && source files/env/dev.env && cargo build --release

run:
	set -a && source files/env/dev.env && cargo run

dev:
	set -a && source files/env/dev.env && cargo watch -x 'test -- --test-threads 1'

test:
	set -a && source files/env/dev.env && cargo test  -- --test-threads 1

migrate:
	set -a && source files/env/dev.env && sqlx migrate run

clear-db:
	set -a && source files/env/dev.env && ./files/db_init/pg_execute.sh ./files/db_init/clear_tables.sql

init-db:
	set -a && source files/env/dev.env && sqlx database create && sqlx migrate run

reset-db:
	set -a && source files/env/dev.env && sqlx database reset
