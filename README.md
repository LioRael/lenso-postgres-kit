# lenso-postgres-kit

`lenso-postgres-kit` gives a stateful Lenso Module an explicit lifecycle for
its own PostgreSQL schema. It is intentionally not a shared State Module, a
generic SQL Capability, or an ORM.

The owning Module still defines:

- its data model and immutable ordered migrations;
- its SQL queries and transaction boundaries;
- its backup, restore, retention, and data-governance policy;
- the dedicated PostgreSQL role used to enforce schema access in production.

The kit supplies:

- atomic, advisory-locked schema setup and upgrade;
- a private migration ledger with checksum drift detection;
- fail-closed runtime preparation that never migrates automatically;
- a verified SQLx pool with the Module schema selected as its `search_path`.

## Author one owned schema

Keep SQL in the owning Module's `migrations/` directory and include it in the
binary at compile time. The Rust declaration remains the explicit ordered
schema plan; the SQL body stays reviewable as SQL:

```text
orders-module/
├── migrations/
│   ├── 001_create_orders.sql
│   └── 002_add_order_status.sql
└── src/
    └── lib.rs
```

```rust,no_run
use lenso_postgres_kit::{
    Migration, OwnedPostgres, SchemaOperator, SchemaPlan, sql_migrations,
};

const MIGRATIONS: &[Migration] = sql_migrations![
    (
        1,
        "create-orders",
        "migrations/001_create_orders.sql",
    ),
    (
        2,
        "add-order-status",
        "migrations/002_add_order_status.sql",
    ),
];

# async fn example(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
let plan = SchemaPlan::new("orders_module", MIGRATIONS)?;

// An operator runs this explicitly during installation or deployment.
SchemaOperator::connect(database_url, plan.clone()).await?.setup().await?;

// Module preparation only verifies the exact schema and then exposes its pool.
let postgres = OwnedPostgres::prepare(database_url, plan).await?;
let count: i64 = sqlx::query_scalar("SELECT count(*) FROM orders")
    .fetch_one(postgres.pool())
    .await?;
# let _ = count;
# Ok(())
# }
```

Paths are relative to the owning crate's `Cargo.toml`. `sql_migrations!`
expands to `include_str!` for each file. There is no runtime directory scan, so
missing files fail compilation, file changes trigger a rebuild, and the
existing migration checksum still binds version, stable name, and exact SQL.
Never edit an applied SQL file; append the next numbered file.

When a new migration is linked, preparation returns `UpgradeRequired`. Stop
the owning Module, run `SchemaOperator::upgrade`, and then prepare the new
generation. `setup` never adopts an existing unmanaged schema, and checksum
drift or a newer database fails closed.

## Isolation contract

`search_path` provides convenient unqualified queries; it is not a security
boundary. Give each Module a dedicated non-superuser PostgreSQL role, make that
role the owner of only its schema, and restrict grants at the database level.
The kit verifies that the current role owns the selected schema.

Sharing one physical PostgreSQL cluster does not grant one Module access to
another Module's tables. Cross-Module workflows belong in explicit Capability
calls and application-level coordination, not in shared SQL transactions.

## Validation

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
LENSO_POSTGRES_TEST_URL=postgres://... \
  cargo test --locked --test postgres_acceptance -- --ignored
```
