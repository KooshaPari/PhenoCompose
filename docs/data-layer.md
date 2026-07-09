# PhenoCompose Data Layer

`sqlx` for type-safe SQL + `sea-orm` for ORM, with migrations.

## Stack

- `sqlx` v0.7+ (compile-time checked queries, async, native)
- `sea-orm` v0.12+ (optional, for relational modeling)
- `refinery` (alternative for embedded migrations)

## Schema versioning

```rust
// migrations/V001__init.sql
CREATE TABLE secrets (
    id          UUID PRIMARY KEY,
    namespace   TEXT NOT NULL DEFAULT '',
    name        TEXT NOT NULL,
    value       BYTEA NOT NULL,
    version     BIGINT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (namespace, name)
);
```

```rust
// migrations/V002__add_labels.sql
ALTER TABLE secrets ADD COLUMN labels JSONB NOT NULL DEFAULT '{}'::jsonb;
```

## Migrations

```rust
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<()> {
    let pool = sqlx::PgPool::connect("postgres://...").await?;
    MIGRATOR.run(&pool).await?;
    Ok(())
}
```

## Type-safe queries

```rust
use sqlx::query_as;

#[derive(sqlx::FromRow)]
struct SecretRow {
    id: Uuid,
    namespace: String,
    name: String,
    value: Vec<u8>,
    version: i64,
}

async fn get_secret(pool: &PgPool, namespace: &str, name: &str) -> Result<SecretRow> {
    query_as!(SecretRow, "SELECT * FROM secrets WHERE namespace = $1 AND name = $2", namespace, name)
        .fetch_one(pool)
        .await
}
```

## Seeding

```rust
async fn seed_dev_data(pool: &PgPool) -> Result<()> {
    query_as!(SecretRow,
        "INSERT INTO secrets (namespace, name, value) VALUES ($1, $2, $3)",
        "default", "dev-secret", b"dev-value"
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

## Backups

- Daily: `pg_dump --format=custom`
- WAL archiving: continuous
- Retention: 30 days hot, 1 year cold

## Determinism in tests

Use `sqlx::test` for isolated test database per-test:

```rust
#[sqlx::test]
async fn test_secret_create(pool: PgPool) {
    // Fresh DB per test
}
```
