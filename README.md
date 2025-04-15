# Studious Maximus

## Developer Setup

Install sqlx-cli

```shell
cargo install sqlx-cli --no-default-features --features sqlite
```

Create DB and run migrations

```shell
cd crates/app
sqlx database create --database-url sqlite://db.sqlite
sqlx migrate run
```

Import data

```shell
cargo run --package app --bin load_students
cargo run --package app --bin load_courses
cargo run --package app --bin load_assignments
```
