use std::{fmt, sync::Arc};

use sha2::{Digest, Sha256};
use thiserror::Error;

/// One immutable, ordered migration owned by a Module.
#[derive(Clone, Copy)]
pub struct Migration {
    version: u64,
    name: &'static str,
    sql: &'static str,
}

impl Migration {
    /// Defines one migration. Validation happens when constructing a [`SchemaPlan`].
    pub const fn new(version: u64, name: &'static str, sql: &'static str) -> Self {
        Self { version, name, sql }
    }

    /// Returns the monotonic migration version.
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Returns the stable migration name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the SQL applied inside the owned migration transaction.
    pub const fn sql(&self) -> &'static str {
        self.sql
    }

    pub(crate) fn checksum(&self) -> String {
        let mut digest = Sha256::new();
        digest.update(self.version.to_be_bytes());
        digest.update([0]);
        digest.update(self.name.as_bytes());
        digest.update([0]);
        digest.update(self.sql.as_bytes());
        hex::encode(digest.finalize())
    }
}

impl fmt::Debug for Migration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Migration")
            .field("version", &self.version)
            .field("name", &self.name)
            .field("checksum", &self.checksum())
            .finish_non_exhaustive()
    }
}

/// An immutable description of one Module-owned `PostgreSQL` schema.
#[derive(Clone)]
pub struct SchemaPlan {
    schema: Arc<str>,
    migrations: &'static [Migration],
}

impl SchemaPlan {
    /// Validates and creates a schema plan.
    pub fn new(
        schema: impl Into<Arc<str>>,
        migrations: &'static [Migration],
    ) -> Result<Self, PlanError> {
        let schema = schema.into();
        validate_schema_name(&schema)?;
        validate_migrations(migrations)?;
        Ok(Self { schema, migrations })
    }

    /// Returns the `PostgreSQL` schema name owned by the Module.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the current version declared by the Module.
    pub fn current_version(&self) -> u64 {
        self.migrations.last().map_or(0, Migration::version)
    }

    pub(crate) const fn migrations(&self) -> &'static [Migration] {
        self.migrations
    }
}

impl fmt::Debug for SchemaPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SchemaPlan")
            .field("schema", &self.schema)
            .field("current_version", &self.current_version())
            .field("migration_count", &self.migrations.len())
            .finish()
    }
}

/// A schema plan is invalid and cannot be used for setup or preparation.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("invalid owned schema name `{schema}`")]
    InvalidSchemaName { schema: Arc<str> },
    #[error("schema plan must contain at least one migration")]
    EmptyMigrations,
    #[error("migration `{name}` has version {actual}; expected {expected}")]
    NonContiguousVersion {
        name: &'static str,
        expected: u64,
        actual: u64,
    },
    #[error("migration version {version} has invalid name `{name}`")]
    InvalidMigrationName { version: u64, name: &'static str },
    #[error("migration version {version} has empty SQL")]
    EmptyMigrationSql { version: u64 },
    #[error("migration version {version} exceeds PostgreSQL bigint range")]
    MigrationVersionTooLarge { version: u64 },
}

fn validate_schema_name(schema: &str) -> Result<(), PlanError> {
    let valid_length = !schema.is_empty() && schema.len() <= 63;
    let mut bytes = schema.bytes();
    let valid_start = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase());
    let valid_rest =
        bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    let reserved =
        schema == "public" || schema == "information_schema" || schema.starts_with("pg_");
    if valid_length && valid_start && valid_rest && !reserved {
        Ok(())
    } else {
        Err(PlanError::InvalidSchemaName {
            schema: Arc::from(schema),
        })
    }
}

fn validate_migrations(migrations: &[Migration]) -> Result<(), PlanError> {
    if migrations.is_empty() {
        return Err(PlanError::EmptyMigrations);
    }

    for (index, migration) in migrations.iter().enumerate() {
        let expected = u64::try_from(index).expect("migration index fits u64") + 1;
        if migration.version != expected {
            return Err(PlanError::NonContiguousVersion {
                name: migration.name,
                expected,
                actual: migration.version,
            });
        }
        if i64::try_from(migration.version).is_err() {
            return Err(PlanError::MigrationVersionTooLarge {
                version: migration.version,
            });
        }
        let mut bytes = migration.name.bytes();
        let valid_start = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase());
        let valid_rest = bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        });
        if migration.name.len() > 128 || !valid_start || !valid_rest {
            return Err(PlanError::InvalidMigrationName {
                version: migration.version,
                name: migration.name,
            });
        }
        if migration.sql.trim().is_empty() {
            return Err(PlanError::EmptyMigrationSql {
                version: migration.version,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[Migration] = &[
        Migration::new(
            1,
            "create-items",
            "CREATE TABLE items (id bigint PRIMARY KEY)",
        ),
        Migration::new(2, "add-label", "ALTER TABLE items ADD COLUMN label text"),
    ];

    #[test]
    fn accepts_a_contiguous_owned_plan() {
        let plan = SchemaPlan::new("orders_module", VALID).unwrap();
        assert_eq!(plan.schema(), "orders_module");
        assert_eq!(plan.current_version(), 2);
        assert!(!VALID[0].checksum().is_empty());
    }

    #[test]
    fn rejects_shared_or_unsafe_schema_names() {
        for name in [
            "",
            "public",
            "pg_catalog",
            "Orders",
            "orders-module",
            "1orders",
        ] {
            assert!(matches!(
                SchemaPlan::new(name, VALID),
                Err(PlanError::InvalidSchemaName { .. })
            ));
        }
    }

    #[test]
    fn rejects_non_contiguous_migrations() {
        const GAP: &[Migration] = &[
            Migration::new(1, "create-items", "SELECT 1"),
            Migration::new(3, "skip-two", "SELECT 3"),
        ];
        assert!(matches!(
            SchemaPlan::new("orders", GAP),
            Err(PlanError::NonContiguousVersion {
                expected: 2,
                actual: 3,
                ..
            })
        ));
    }

    #[test]
    fn checksum_binds_version_name_and_sql() {
        let original = Migration::new(1, "create-items", "SELECT 1");
        let renamed = Migration::new(1, "create-records", "SELECT 1");
        let changed = Migration::new(1, "create-items", "SELECT 2");
        assert_ne!(original.checksum(), renamed.checksum());
        assert_ne!(original.checksum(), changed.checksum());
    }
}
