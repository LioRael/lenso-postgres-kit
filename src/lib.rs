//! `PostgreSQL` lifecycle support for storage owned by one Lenso Module.
//!
//! This crate deliberately is not a shared State Module, SQL Capability, or
//! repository abstraction. A Module keeps ownership of its data model,
//! migrations, queries, and transaction boundaries. The kit only makes the
//! repetitive `PostgreSQL` schema lifecycle explicit and fail-closed.

mod error;
mod lifecycle;
mod plan;

pub use error::{PostgresKitError, SetupOutcome, UpgradeOutcome};
pub use lifecycle::{OwnedPostgres, SchemaOperator};
pub use plan::{Migration, PlanError, SchemaPlan};
