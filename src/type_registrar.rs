use std::rc::Rc;

use heck::ToUpperCamelCase;
use indexmap::{IndexMap, map::Entry};
use postgres_types::{Kind, Type};

use crate::{
    codegen::{DependencyAnalysis, GenCtx, idx_char},
    config::{Config, TypeMapping},
    parser::Span,
    read_queries::ModuleInfo,
    utils::SchemaKey,
};

use self::error::Error;

/// A struct containing a postgres type and its Rust-equivalent.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum CornucopiaType {
    Simple {
        pg_ty: Type,
        rust_name: String,
        /// The borrowed counterpart type name, with explicit lifetime (e.g., `MyType<'a>`)
        borrowed_name: Option<String>,
        is_copy: bool,
    },
    Array {
        inner: Rc<CornucopiaType>,
    },
    Domain {
        pg_ty: Type,
        inner: Rc<CornucopiaType>,
    },
    Custom {
        pg_ty: Type,
        struct_name: String,
        is_copy: bool,
        is_params: bool,
    },
}

impl CornucopiaType {
    /// Is this type need a generic lifetime
    pub fn is_ref(&self) -> bool {
        match self {
            CornucopiaType::Simple {
                pg_ty:
                    Type::BYTEA
                    | Type::TEXT
                    | Type::VARCHAR
                    | Type::BPCHAR
                    | Type::NAME
                    | Type::JSON
                    | Type::JSONB,
                ..
            } => false,
            CornucopiaType::Simple { pg_ty: ty, .. }
                if (ty.name() == "citext"
                    || ty.name() == "ltree"
                    || ty.name() == "lquery"
                    || ty.name() == "ltxtquery") =>
            {
                false
            }
            CornucopiaType::Simple {
                borrowed_name: Some(_),
                ..
            } => true,
            CornucopiaType::Simple { .. } => !self.is_copy(),
            CornucopiaType::Domain { inner, .. } | CornucopiaType::Array { inner } => {
                inner.is_ref()
            }
            _ => !self.is_copy(),
        }
    }

    /// Is this type copyable
    pub fn is_copy(&self) -> bool {
        match self {
            CornucopiaType::Simple { is_copy, .. } | CornucopiaType::Custom { is_copy, .. } => {
                *is_copy
            }
            CornucopiaType::Domain { inner, .. } => inner.is_copy(),
            CornucopiaType::Array { .. } => false,
        }
    }

    /// Can this used in parameters as it is
    pub fn is_params(&self) -> bool {
        match self {
            CornucopiaType::Simple { .. } => true,
            CornucopiaType::Array { .. } => false,
            CornucopiaType::Domain { inner, .. } => inner.is_params(),
            CornucopiaType::Custom { is_params, .. } => *is_params,
        }
    }

    /// Wrap type to escape domains in parameters
    pub(crate) fn sql_wrapped(&self, name: &str) -> String {
        match self {
            CornucopiaType::Domain { inner, .. } => {
                format!("&crate::Domain({})", inner.sql_wrapped(name))
            }
            CornucopiaType::Array { inner } => match inner.as_ref() {
                CornucopiaType::Domain { inner, .. } => {
                    format!("&crate::DomainArray({})", inner.sql_wrapped(name))
                }
                _ => name.to_string(),
            },
            _ => name.to_string(),
        }
    }

    /// Wrap type to escape domains when writing to sql
    pub(crate) fn accept_to_sql(&self, ctx: &GenCtx) -> String {
        match self {
            CornucopiaType::Domain { inner, .. } => {
                format!("crate::Domain::<{}>", inner.accept_to_sql(ctx))
            }
            CornucopiaType::Array { inner } => match inner.as_ref() {
                CornucopiaType::Domain { inner, .. } => {
                    let ty = inner.accept_to_sql(ctx);
                    format!("crate::DomainArray::<{ty}, &[{ty}]>")
                }
                _ => self.param_ty(false, ctx),
            },
            _ => self.param_ty(false, ctx),
        }
    }

    /// Corresponding postgres type
    pub(crate) fn pg_ty(&self) -> &Type {
        match self {
            CornucopiaType::Simple { pg_ty, .. }
            | CornucopiaType::Custom { pg_ty, .. }
            | CornucopiaType::Domain { pg_ty, .. } => pg_ty,
            CornucopiaType::Array { inner } => inner.pg_ty(),
        }
    }

    /// Code to transform its borrowed type to its owned one
    pub(crate) fn owning_call(
        &self,
        name: &str,
        is_nullable: bool,
        is_inner_nullable: bool,
    ) -> String {
        if self.is_copy() {
            return name.into();
        }

        if is_nullable {
            let into = self.owning_call("v", false, is_inner_nullable);
            return format!("{name}.map(|v| {into})");
        }

        match self {
            CornucopiaType::Simple { pg_ty, .. } if matches!(*pg_ty, Type::JSON | Type::JSONB) => {
                format!("serde_json::from_str({name}.0.get()).unwrap()")
            }
            CornucopiaType::Array { inner, .. } => {
                let inner = inner.owning_call("v", is_inner_nullable, false);
                format!("{name}.map(|v| {inner}).collect()")
            }
            CornucopiaType::Domain { inner, .. } => inner.owning_call(name, is_nullable, false),
            _ => {
                format!("{name}.into()")
            }
        }
    }

    /// Corresponding owned type
    pub(crate) fn own_ty(&self, is_inner_nullable: bool, ctx: &GenCtx) -> String {
        match self {
            CornucopiaType::Simple { rust_name, .. } => (*rust_name).to_string(),
            CornucopiaType::Array { inner, .. } => {
                let own_inner = inner.own_ty(false, ctx);
                if is_inner_nullable {
                    format!("Vec<Option<{own_inner}>>")
                } else {
                    format!("Vec<{own_inner}>")
                }
            }
            CornucopiaType::Domain { inner, .. } => inner.own_ty(false, ctx),
            CornucopiaType::Custom {
                struct_name, pg_ty, ..
            } => ctx.custom_ty_path(pg_ty.schema(), struct_name),
        }
    }

    /// Corresponding borrowed ergonomic parameter type (using traits if possible)
    pub(crate) fn param_ergo_ty(
        &self,
        is_inner_nullable: bool,
        traits: &mut Vec<String>,
        ctx: &GenCtx,
    ) -> String {
        match self {
            CornucopiaType::Simple { pg_ty, .. } => match *pg_ty {
                Type::BYTEA => {
                    traits.push("crate::BytesSql".to_string());
                    idx_char(traits.len())
                }
                Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
                    traits.push("crate::StringSql".to_string());
                    idx_char(traits.len())
                }
                Type::JSON | Type::JSONB => {
                    traits.push("crate::JsonSql".to_string());
                    idx_char(traits.len())
                }
                ref ty
                    if (ty.name() == "citext"
                        || ty.name() == "ltree"
                        || ty.name() == "lquery"
                        || ty.name() == "ltxtquery") =>
                {
                    traits.push("crate::StringSql".to_string());
                    idx_char(traits.len())
                }
                _ => self.param_ty(is_inner_nullable, ctx),
            },
            CornucopiaType::Array { inner, .. } => {
                let inner = inner.param_ergo_ty(is_inner_nullable, traits, ctx);
                let inner = if is_inner_nullable {
                    format!("Option<{inner}>")
                } else {
                    inner
                };
                traits.push(format!("crate::ArraySql<Item = {inner}>"));
                idx_char(traits.len())
            }
            CornucopiaType::Domain { inner, .. } => {
                inner.param_ergo_ty(is_inner_nullable, traits, ctx)
            }
            CornucopiaType::Custom { .. } => self.param_ty(is_inner_nullable, ctx),
        }
    }

    /// Corresponding borrowed parameter type
    pub(crate) fn param_ty(&self, is_inner_nullable: bool, ctx: &GenCtx) -> String {
        match self {
            CornucopiaType::Simple {
                pg_ty,
                borrowed_name,
                ..
            } => {
                // If user explicitly provides a borrowed type, use it
                if borrowed_name.is_some() {
                    return self.brw_ty(is_inner_nullable, true, ctx);
                }
                // Otherwise use default param type based on pg_ty
                match *pg_ty {
                    Type::JSON | Type::JSONB => "&'a serde_json::value::Value".to_string(),
                    _ => self.brw_ty(is_inner_nullable, true, ctx),
                }
            }
            CornucopiaType::Array { inner, .. } => {
                let inner = inner.param_ty(is_inner_nullable, ctx);
                let inner = if is_inner_nullable {
                    format!("Option<{inner}>")
                } else {
                    inner
                };
                // Its more practical for users to use a slice
                format!("&'a [{inner}]")
            }
            CornucopiaType::Domain { inner, .. } => inner.param_ty(false, ctx),
            CornucopiaType::Custom {
                is_params,
                is_copy,
                pg_ty,
                struct_name,
                ..
            } => {
                if !is_copy && !is_params {
                    let path = ctx.custom_ty_path(pg_ty.schema(), struct_name);
                    format!("{path}Params<'a>")
                } else {
                    self.brw_ty(is_inner_nullable, true, ctx)
                }
            }
        }
    }

    /// String representing a borrowed rust equivalent of this type. Notably, if
    /// a Rust equivalent is a String or a Vec<T>, it will return a &str and a &[T] respectively.
    pub(crate) fn brw_ty(
        &self,
        is_inner_nullable: bool,
        has_lifetime: bool,
        ctx: &GenCtx,
    ) -> String {
        let lifetime = if has_lifetime { "'a" } else { "" };
        match self {
            CornucopiaType::Simple {
                pg_ty,
                rust_name,
                borrowed_name,
                ..
            } => {
                // If user explicitly provides a borrowed type, use it
                if let Some(borrowed) = borrowed_name {
                    return borrowed.clone();
                }
                // Otherwise use default borrowed type based on pg_ty
                match *pg_ty {
                    Type::BYTEA => format!("&{lifetime} [u8]"),
                    Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
                        format!("&{lifetime} str")
                    }
                    Type::JSON | Type::JSONB => {
                        format!("postgres_types::Json<&{lifetime} serde_json::value::RawValue>")
                    }
                    ref ty
                        if (ty.name() == "citext"
                            || ty.name() == "ltree"
                            || ty.name() == "lquery"
                            || ty.name() == "ltxtquery") =>
                    {
                        format!("&{lifetime} str")
                    }
                    _ => match rust_name.as_str() {
                        "String" => format!("&{lifetime} str"),
                        "Vec<u8>" => format!("&{lifetime} [u8]"),
                        _ => rust_name.to_string(),
                    },
                }
            }
            CornucopiaType::Array { inner, .. } => {
                let inner = inner.brw_ty(is_inner_nullable, has_lifetime, ctx);
                let inner = if is_inner_nullable {
                    format!("Option<{inner}>")
                } else {
                    inner
                };
                // Its more practical for users to use a slice
                let lifetime = if has_lifetime { lifetime } else { "'_" };
                format!("crate::ArrayIterator<{lifetime}, {inner}>")
            }
            CornucopiaType::Domain { inner, .. } => inner.brw_ty(false, has_lifetime, ctx),
            CornucopiaType::Custom {
                is_copy,
                pg_ty,
                struct_name,
                ..
            } => {
                let path = ctx.custom_ty_path(pg_ty.schema(), struct_name);
                if *is_copy {
                    path
                } else {
                    format!("{path}Borrowed<{lifetime}>")
                }
            }
        }
    }
}

/// Data structure holding all types known to this particular run of Cornucopia.
#[derive(Debug, Clone)]
pub(crate) struct TypeRegistrar {
    pub types: IndexMap<(String, String), Rc<CornucopiaType>>,
    pub dependency_analysis: DependencyAnalysis,
    /// Domains used through a `col: domain_name` row override, keyed by domain name. Each
    /// one needs a row-decode wrapper generated in `types.rs`; see `domain_row_wrapper_name`.
    pub domain_row_overrides: IndexMap<String, String>,
    config: Config,
}

impl TypeRegistrar {
    pub(crate) fn new(config: Config) -> Self {
        Self {
            types: IndexMap::default(),
            dependency_analysis: DependencyAnalysis::default(),
            domain_row_overrides: IndexMap::default(),
            config,
        }
    }

    /// Returns the type mapping for a specific type
    pub(crate) fn get_type_mapping(&self, ty: &Type) -> Option<&TypeMapping> {
        self.config.get_type_mapping(ty)
    }

    /// Resolves a `col: domain_name` row override into the `CornucopiaType` its field should
    /// use, and records that the domain needs a row-decode wrapper generated.
    ///
    /// `col_ty` is the type PostgreSQL actually reported for the column (always the domain's
    /// base type, per the `types.domains` doc comment): see `resolve_row_domain_override` for
    /// what is checked.
    pub(crate) fn resolve_row_domain_override(
        &mut self,
        client: &tokio_postgres::Client,
        domain_name: &str,
        col_ty: &Type,
    ) -> Result<Rc<CornucopiaType>, Error> {
        let ty = self::row_domain_override_type(client, &self.config, domain_name, col_ty)?;
        let CornucopiaType::Simple { rust_name, .. } = &ty else {
            unreachable!("resolve_row_domain_override always returns CornucopiaType::Simple")
        };
        self.domain_row_overrides
            .entry(domain_name.to_string())
            .or_insert_with(|| rust_name.clone());
        Ok(Rc::new(ty))
    }

    fn resolve_type(
        &mut self,
        ty: &Type,
        name: &str,
        query_name: &Span<String>,
        module_info: &ModuleInfo,
        default_is_copy: bool,
        default_is_params: bool,
    ) -> Result<&Rc<CornucopiaType>, Error> {
        fn custom(ty: &Type, is_copy: bool, is_params: bool) -> CornucopiaType {
            let rust_ty_name = ty.name().to_upper_camel_case();
            CornucopiaType::Custom {
                pg_ty: ty.clone(),
                struct_name: rust_ty_name,
                is_copy,
                is_params,
            }
        }

        fn domain(ty: &Type, inner: Rc<CornucopiaType>) -> CornucopiaType {
            CornucopiaType::Domain {
                pg_ty: ty.clone(),
                inner,
            }
        }

        Ok(match ty.kind() {
            Kind::Enum(_) => self.insert(ty, || custom(ty, true, true)),
            Kind::Array(inner_ty) => {
                let inner = self
                    .register(name, inner_ty, query_name, module_info)?
                    .clone();
                self.insert(ty, || CornucopiaType::Array {
                    inner: inner.clone(),
                })
            }
            Kind::Domain(inner_ty) => {
                let inner = self
                    .register(name, inner_ty, query_name, module_info)?
                    .clone();
                self.insert(ty, || domain(ty, inner.clone()))
            }
            Kind::Composite(composite_fields) => {
                let mut is_copy = default_is_copy;
                let mut is_params = default_is_params;
                for field in composite_fields {
                    let field_ty = self.register(name, field.type_(), query_name, module_info)?;
                    is_copy &= field_ty.is_copy();
                    is_params &= field_ty.is_params();
                }
                self.insert(ty, || custom(ty, is_copy, is_params))
            }
            Kind::Simple => {
                let (rust_name, is_copy) = match *ty {
                    Type::BOOL => ("bool", true),
                    Type::CHAR => ("i8", true),
                    Type::INT2 => ("i16", true),
                    Type::INT4 => ("i32", true),
                    Type::INT8 => ("i64", true),
                    Type::FLOAT4 => ("f32", true),
                    Type::FLOAT8 => ("f64", true),
                    Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => ("String", false),
                    Type::BYTEA => ("Vec<u8>", false),
                    Type::TIMESTAMP => ("chrono::NaiveDateTime", true),
                    Type::TIMESTAMPTZ => ("chrono::DateTime<chrono::FixedOffset>", true),
                    Type::DATE => ("chrono::NaiveDate", true),
                    Type::TIME => ("chrono::NaiveTime", true),
                    Type::JSON | Type::JSONB => ("serde_json::Value", false),
                    Type::UUID => ("uuid::Uuid", true),
                    Type::INET => ("std::net::IpAddr", true),
                    Type::MACADDR => ("eui48::MacAddress", true),
                    Type::NUMERIC => ("rust_decimal::Decimal", true),
                    ref ty
                        if (ty.name() == "citext"
                            || ty.name() == "ltree"
                            || ty.name() == "lquery"
                            || ty.name() == "ltxtquery") =>
                    {
                        ("String", false)
                    }
                    _ => {
                        return Err(Error::UnsupportedPostgresType {
                            src: module_info.clone().into(),
                            query: query_name.span,
                            col_name: name.to_string(),
                            col_ty: ty.to_string(),
                        });
                    }
                };
                self.insert(ty, || CornucopiaType::Simple {
                    pg_ty: ty.clone(),
                    rust_name: rust_name.to_string(),
                    borrowed_name: None,
                    is_copy,
                })
            }
            _ => {
                return Err(Error::UnsupportedPostgresType {
                    src: module_info.clone().into(),
                    query: query_name.span,
                    col_name: name.to_string(),
                    col_ty: ty.to_string(),
                });
            }
        })
    }

    pub(crate) fn register(
        &mut self,
        name: &str,
        ty: &Type,
        query_name: &Span<String>,
        module_info: &ModuleInfo,
    ) -> Result<&Rc<CornucopiaType>, Error> {
        self.dependency_analysis.analyse(ty);

        if let Some(idx) = self.types.get_index_of(&SchemaKey::from(ty)) {
            return Ok(&self.types[idx]);
        }

        // check if there's a user-defined mapping first
        let mapping_result = if let Some(mapping) = self.config.get_type_mapping(ty) {
            match mapping {
                TypeMapping::Simple(name) => Some((name.to_string(), None, true)),
                TypeMapping::Detailed {
                    rust_type,
                    borrowed_type,
                    is_copy,
                    ..
                } => Some((rust_type.to_string(), borrowed_type.clone(), *is_copy)),
            }
        } else if let Kind::Domain(_) = ty.kind() {
            // A domain-specific mapping overrides the default of transparently falling
            // back to the base type. The mapped type is used as-is (not wrapped), so it
            // has to accept the domain itself; see the `types.domains` doc comment.
            self.config
                .types
                .domains
                .get(ty.name())
                .map(|rust_type| (rust_type.clone(), None, false))
        } else {
            None
        };

        if let Some((rust_name, borrowed_name, is_copy)) = mapping_result {
            return Ok(self.insert(ty, || CornucopiaType::Simple {
                pg_ty: ty.clone(),
                rust_name,
                borrowed_name,
                is_copy,
            }));
        }

        self.resolve_type(ty, name, query_name, module_info, true, true)
    }

    pub(crate) fn ref_of(&self, ty: &Type) -> Rc<CornucopiaType> {
        self.types
            .get(&SchemaKey::from(ty))
            .expect("type must already be registered")
            .clone()
    }

    fn insert(&mut self, ty: &Type, call: impl FnOnce() -> CornucopiaType) -> &Rc<CornucopiaType> {
        let index = match self
            .types
            .entry((ty.schema().to_owned(), ty.name().to_owned()))
        {
            Entry::Occupied(o) => o.index(),
            Entry::Vacant(v) => {
                let index = v.index();
                v.insert(Rc::new(call()));
                index
            }
        };
        &self.types[index]
    }
}

impl std::ops::Index<&Type> for TypeRegistrar {
    type Output = Rc<CornucopiaType>;

    fn index(&self, index: &Type) -> &Self::Output {
        &self.types[&SchemaKey::from(index)]
    }
}

/// Checks that every domain named in `types.domains` exists in the schema. Domains that are
/// never referenced by a query are otherwise never looked at, so this cannot rely on type
/// registration alone and queries `pg_type` directly.
pub(crate) fn validate_domain_mappings(
    client: &tokio_postgres::Client,
    config: &Config,
) -> Result<(), Error> {
    if config.types.domains.is_empty() {
        return Ok(());
    }

    let rows = futures::executor::block_on(
        client.query("SELECT typname FROM pg_type WHERE typtype = 'd'", &[]),
    )?;
    let known_domains: std::collections::HashSet<String> =
        rows.iter().map(|row| row.get::<_, String>(0)).collect();

    for name in config.types.domains.keys() {
        if !known_domains.contains(name) {
            return Err(Error::UnknownDomain { name: name.clone() });
        }
    }

    Ok(())
}

/// Resolves the [`CornucopiaType`] a `col: domain_name` row override produces for `col_ty`,
/// the type PostgreSQL actually reported for that result column.
///
/// A result column of domain type is always reported as its base type (see the `types.domains`
/// doc comment), so this is the only way to recover the domain: the caller states it, and this
/// checks the statement against the schema. Two things can go wrong: the named domain has no
/// `types.domains` mapping, or `col_ty` isn't actually that domain's base type (a stale
/// annotation after the column's type changed).
fn row_domain_override_type(
    client: &tokio_postgres::Client,
    config: &Config,
    domain_name: &str,
    col_ty: &Type,
) -> Result<CornucopiaType, Error> {
    let Some(rust_type) = config.types.domains.get(domain_name) else {
        return Err(Error::DomainOverrideNotMapped {
            name: domain_name.to_string(),
        });
    };

    let quoted = domain_name.replace('"', "\"\"");
    let stmt = futures::executor::block_on(client.prepare(&format!("SELECT $1::\"{quoted}\"")))?;
    let domain_ty = stmt.params()[0].clone();
    let base = match domain_ty.kind() {
        Kind::Domain(base) => base,
        // `types.domains` is already validated against the schema, so a name found there
        // that isn't actually a domain would be a Cornucopia bug, not a user error.
        _ => unreachable!("`{domain_name}` is validated to be a domain"),
    };

    if base.name() != col_ty.name() || base.schema() != col_ty.schema() {
        return Err(Error::DomainOverrideBaseMismatch {
            domain: domain_name.to_string(),
            expected_base: base.name().to_string(),
            actual: col_ty.name().to_string(),
        });
    }

    Ok(CornucopiaType::Simple {
        // Keep the real domain `Type` (`Kind::Domain`, not `col_ty`'s base kind) so the
        // `Type::TEXT`/`BYTEA`/etc special cases in `brw_ty`/`is_ref` don't fire for it: those
        // key off `pg_ty` alone and would otherwise force e.g. `&str` regardless of `rust_name`.
        pg_ty: domain_ty,
        rust_name: rust_type.clone(),
        borrowed_name: None,
        is_copy: false,
    })
}

/// Name of the `FromSql` wrapper generated in `types.rs` for a domain used through a row
/// override. It reinterprets the reported (base) type as the domain, then defers to the
/// mapped Rust type's own `FromSql`, exactly as a parameter or composite field would see it.
pub(crate) fn domain_row_wrapper_name(domain: &str) -> String {
    format!("{}RowValue", domain.to_upper_camel_case())
}

pub(crate) mod error {
    use std::sync::Arc;

    use miette::{Diagnostic, NamedSource, SourceSpan};
    use thiserror::Error as ThisError;

    #[derive(Debug, ThisError, Diagnostic)]
    #[error("Couldn't register SQL type.")]
    pub enum Error {
        Db(#[from] tokio_postgres::Error),
        UnsupportedPostgresType {
            #[source_code]
            src: NamedSource<Arc<String>>,
            #[label("this query contains an unsupported type (name: {col_name}, type: {col_ty})")]
            query: SourceSpan,
            col_name: String,
            col_ty: String,
        },
        #[error("unknown domain `{name}` in `types.domains`: no such domain exists in the schema")]
        UnknownDomain {
            name: String,
        },
        #[error("domain `{name}` named in a row override has no `types.domains` mapping")]
        DomainOverrideNotMapped {
            name: String,
        },
        #[error(
            "row override names domain `{domain}`, whose base type is `{expected_base}`, but the column's reported type is `{actual}`"
        )]
        DomainOverrideBaseMismatch {
            domain: String,
            expected_base: String,
            actual: String,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::codegen::{GenCtx, ModCtx};

    fn dummy_span(value: &str) -> Span<String> {
        Span {
            span: (0..0).into(),
            value: value.to_string(),
        }
    }

    fn dummy_module_info() -> ModuleInfo {
        ModuleInfo {
            path: "test.sql".into(),
            name: "test".to_string(),
            full_module_path: "test".to_string(),
            content: Arc::new(String::new()),
        }
    }

    /// An unmapped domain has to be transparently resolved to its base type: this is what
    /// lets an unmapped domain-typed parameter work without an explicit SQL cast, and a
    /// domain-typed column read back as the base Rust type.
    #[test]
    fn unmapped_domain_falls_back_to_its_base_type() {
        let mut registrar = TypeRegistrar::new(Config::default());
        let domain = Type::new(
            "my_domain".to_string(),
            100_000,
            Kind::Domain(Type::TEXT),
            "public".to_string(),
        );

        let ty = registrar
            .register("col", &domain, &dummy_span("q"), &dummy_module_info())
            .unwrap()
            .clone();

        assert!(matches!(&*ty, CornucopiaType::Domain { .. }));
        assert_eq!(
            ty.own_ty(false, &GenCtx::new(ModCtx::Types, true)),
            "String"
        );
        // The `Domain` wrapper is what lets the value be sent without a `::text` cast.
        assert_eq!(ty.sql_wrapped("value"), "&crate::Domain(value)");
    }

    /// A domain named in `types.domains` bypasses the base-type fallback entirely: the
    /// mapped type is used as-is, unwrapped, so it must accept the domain itself.
    #[test]
    fn mapped_domain_uses_the_configured_rust_type() {
        let config = Config::builder()
            .add_domain_mapping("iccid", "vibe_sim::Iccid")
            .build();
        let mut registrar = TypeRegistrar::new(config);
        let domain = Type::new(
            "iccid".to_string(),
            100_001,
            Kind::Domain(Type::TEXT),
            "public".to_string(),
        );

        let ty = registrar
            .register("iccid", &domain, &dummy_span("q"), &dummy_module_info())
            .unwrap()
            .clone();

        assert!(matches!(&*ty, CornucopiaType::Simple { .. }));
        assert_eq!(
            ty.own_ty(false, &GenCtx::new(ModCtx::Types, true)),
            "vibe_sim::Iccid"
        );
        assert!(!ty.is_copy());
        assert_eq!(ty.sql_wrapped("value"), "value");
    }

    /// Mapping one domain must not affect an unrelated, unmapped one.
    #[test]
    fn domain_mapping_does_not_affect_other_domains() {
        let config = Config::builder()
            .add_domain_mapping("iccid", "vibe_sim::Iccid")
            .build();
        let mut registrar = TypeRegistrar::new(config);
        let domain = Type::new(
            "my_domain".to_string(),
            100_002,
            Kind::Domain(Type::TEXT),
            "public".to_string(),
        );

        let ty = registrar
            .register("col", &domain, &dummy_span("q"), &dummy_module_info())
            .unwrap()
            .clone();

        assert!(matches!(&*ty, CornucopiaType::Domain { .. }));
    }

    #[test]
    fn domain_row_wrapper_name_upper_camel_cases_the_domain() {
        assert_eq!(domain_row_wrapper_name("iccid"), "IccidRowValue");
        assert_eq!(domain_row_wrapper_name("sim_note"), "SimNoteRowValue");
    }
}
