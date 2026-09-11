//! PostgreSQL-only `SQLx` query types used by the kit and its consumers.
//!
//! Import this module instead of the multi-database `sqlx` facade when composing
//! PostgreSQL Plugins with a Host that independently owns SQLite. These are
//! upstream types and functions; the kit adds no query or authorization layer.

pub use sqlx_core::acquire::Acquire;
pub use sqlx_core::connection::{ConnectOptions, Connection};
pub use sqlx_core::database::Database;
pub use sqlx_core::decode::Decode;
pub use sqlx_core::encode::Encode;
pub use sqlx_core::error::{Error, Result};
pub use sqlx_core::executor::{Execute, Executor};
pub use sqlx_core::from_row::FromRow;
pub use sqlx_core::pool::{self, Pool};
pub use sqlx_core::query::{query, query_with};
pub use sqlx_core::query_as::{query_as, query_as_with};
pub use sqlx_core::query_scalar::{query_scalar, query_scalar_with};
pub use sqlx_core::raw_sql::{RawSql, raw_sql};
pub use sqlx_core::row::Row;
pub use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};
pub use sqlx_core::transaction::Transaction;
pub use sqlx_core::types::{self, Type};
pub use sqlx_postgres::{self as postgres, PgConnection, PgPool, Postgres};
