// This file was generated with `cornucopia`. Do not modify.

#[allow(clippy::all, clippy::pedantic)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(dead_code)]
pub mod queries;
#[allow(clippy::all, clippy::pedantic)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(dead_code)]
pub mod types;
pub use db_runtime::ArrayIterator;
pub use db_runtime::client;
pub(crate) use db_runtime::slice_iter;
pub use db_runtime::{ArraySql, BytesSql, IterSql, StringSql};
pub use db_runtime::{Domain, DomainArray};
#[cfg(feature = "deadpool")]
pub use deadpool_postgres;
#[cfg(not(feature = "deadpool"))]
pub use postgres;
#[cfg(not(feature = "deadpool"))]
pub use postgres::fallible_iterator;
#[cfg(feature = "deadpool")]
pub use tokio_postgres;
#[cfg(feature = "deadpool")]
pub use tokio_postgres::fallible_iterator;
