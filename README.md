# pgqueue

A Rust library implementing a task queue based on [PostgreSQL](https://www.postgresql.org/).

### Development

#### Prerequisites

- [Rust 1.71+](https://www.rust-lang.org/)
- [sqlx](https://github.com/launchbadge/sqlx) is used to run the SQL migrations

#### Instructions

```bash
# build the project with clippy warnings
make

# run test
make test

# watch changes, live build and test during development
make dev

# run DB migration
make migrate

# clear database tables
make clear-db

# setup database from scratch with sqlx
make init-db

# re-create database with sqlx
make reset-db

```

### References

- [Postgres Task Queues: The Secret Weapon](https://www.fforward.ai/blog/posts/postgres-task-queues-the-secret-weapon-killing-specialized-queue-services)
