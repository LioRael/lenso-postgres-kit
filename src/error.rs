use thiserror::Error;

use crate::PlanError;

/// A `PostgreSQL` schema lifecycle operation failed without applying a fallback.
#[derive(Debug, Error)]
pub enum PostgresKitError {
    /// The authored schema plan is invalid.
    #[error(transparent)]
    InvalidPlan(#[from] PlanError),
    /// Connection setup or a database operation failed.
    #[error("PostgreSQL operation `{operation}` failed")]
    Database {
        operation: &'static str,
        #[source]
        source: sqlx::Error,
    },
    /// The schema exists, but was not created by this lifecycle protocol.
    #[error("schema `{schema}` exists without the Lenso migration ledger")]
    UnmanagedSchema { schema: String },
    /// The configured database role does not own the Module schema.
    #[error("schema `{schema}` is owned by `{owner}`, not current role `{current_role}`")]
    OwnershipMismatch {
        schema: String,
        owner: String,
        current_role: String,
    },
    /// Runtime preparation requires an explicit setup operation first.
    #[error("schema `{schema}` has not been set up")]
    SetupRequired { schema: String },
    /// Runtime preparation never applies pending migrations.
    #[error(
        "schema `{schema}` is at version {current}; explicit upgrade to {expected} is required"
    )]
    UpgradeRequired {
        schema: String,
        current: u64,
        expected: u64,
    },
    /// Applied migration history no longer matches the immutable authored plan.
    #[error("schema `{schema}` migration history diverged at version {version}")]
    HistoryDiverged { schema: String, version: u64 },
    /// The database is newer than the linked Module implementation.
    #[error("schema `{schema}` is at version {actual}, newer than supported version {expected}")]
    SchemaAhead {
        schema: String,
        actual: u64,
        expected: u64,
    },
}

impl PostgresKitError {
    pub(crate) const fn database(operation: &'static str, source: sqlx::Error) -> Self {
        Self::Database { operation, source }
    }
}

/// Result of explicitly setting up an owned schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupOutcome {
    /// A new schema was created at the current authored version.
    Created { version: u64, applied: usize },
    /// The existing managed schema already matched the authored plan.
    AlreadyCurrent { version: u64 },
}

/// Result of explicitly upgrading an owned schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeOutcome {
    /// Pending migrations were applied atomically.
    Applied { from: u64, to: u64, applied: usize },
    /// The existing managed schema already matched the authored plan.
    AlreadyCurrent { version: u64 },
}
