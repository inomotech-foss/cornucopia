// This file was generated with `cornucopia`. Do not modify.

mod array_iterator;
pub mod client;
mod domain;
mod type_traits;
mod utils;
pub use array_iterator::ArrayIterator;
#[cfg(feature = "deadpool")]
pub use deadpool_postgres;
pub use domain::{Domain, DomainArray};
pub use tokio_postgres;
pub use tokio_postgres::fallible_iterator;
pub use type_traits::{ArraySql, BytesSql, IterSql, JsonSql, StringSql};
pub use utils::slice_iter;
