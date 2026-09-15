//! A minimal `ToSql`/`FromSql` implementation for a domain-mapped type, used by the
//! `domain_mapping` integration test. Cornucopia never wraps a `types.domains` mapped
//! type, so `accepts` has to match the domain itself (`Kind::Domain`), not its base type.

use bytes::BytesMut;
use postgres_types::{FromSql, IsNull, Kind, ToSql, Type, to_sql_checked};

/// Stand-in for a consumer-defined newtype validating the same rule as the `iccid` domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iccid(pub String);

impl ToSql for Iccid {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        <String as ToSql>::to_sql(&self.0, base_type(ty), out)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "iccid"
            && matches!(ty.kind(), Kind::Domain(base) if <String as ToSql>::accepts(base))
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Iccid {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Iccid(<String as FromSql>::from_sql(base_type(ty), raw)?))
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "iccid"
            && matches!(ty.kind(), Kind::Domain(base) if <String as FromSql>::accepts(base))
    }
}

fn base_type(ty: &Type) -> &Type {
    match ty.kind() {
        Kind::Domain(base) => base,
        _ => ty,
    }
}
