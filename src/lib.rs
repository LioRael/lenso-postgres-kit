//! `PostgreSQL` lifecycle support for storage owned by one Lenso Plugin.
//!
//! This crate deliberately is not a shared State Plugin, SQL Capability, or
//! repository abstraction. A Plugin keeps ownership of its data model,
//! migrations, queries, and transaction boundaries. The kit only makes the
//! repetitive `PostgreSQL` schema lifecycle explicit and fail-closed.

mod error;
mod lifecycle;
mod plan;

pub use error::{PostgresKitError, SetupOutcome, UpgradeOutcome};
pub use lifecycle::{OwnedPostgres, SchemaOperator};
pub use plan::{Migration, PlanError, SchemaPlan};

/// Declares immutable migrations whose SQL bodies live in dedicated files.
///
/// Paths are resolved from the owning crate's `CARGO_MANIFEST_DIR`. Keeping the
/// ordered version and stable name explicit makes review and checksum drift
/// behavior unchanged.
///
/// ```ignore
/// use lenso_postgres_kit::{Migration, sql_migrations};
///
/// const MIGRATIONS: &[Migration] = sql_migrations![
///     (1, "create-orders", "migrations/001_create_orders.sql"),
///     (2, "add-order-status", "migrations/002_add_order_status.sql"),
/// ];
/// ```
#[macro_export]
macro_rules! sql_migrations {
    (
        $(
            ($version:literal, $name:literal, $path:literal $(,)?)
        ),+ $(,)?
    ) => {
        &[
            $(
                $crate::Migration::new(
                    $version,
                    $name,
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)),
                ),
            )+
        ]
    };
}
