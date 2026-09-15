// This file was generated with `cornucopia`. Do not modify.

#[doc(hidden)]
pub struct IccidRowValue(pub iccid_type::Iccid);
impl<'a> postgres_types::FromSql<'a> for IccidRowValue {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let domain_ty = postgres_types::Type::new(
            "iccid".to_string(),
            0,
            postgres_types::Kind::Domain(ty.clone()),
            ty.schema().to_string(),
        );
        Ok(IccidRowValue(
            <iccid_type::Iccid as postgres_types::FromSql>::from_sql(&domain_ty, raw)?,
        ))
    }
    fn accepts(_ty: &postgres_types::Type) -> bool {
        true
    }
}
#[derive(Debug, postgres_types::FromSql, Clone, PartialEq)]
#[postgres(name = "sim_info")]
pub struct SimInfo {
    #[postgres(name = "iccid")]
    pub iccid: iccid_type::Iccid,
    #[postgres(name = "note")]
    pub note: String,
}
#[derive(Debug)]
pub struct SimInfoBorrowed<'a> {
    pub iccid: iccid_type::Iccid,
    pub note: &'a str,
}
impl<'a> From<SimInfoBorrowed<'a>> for SimInfo {
    fn from(SimInfoBorrowed { iccid, note }: SimInfoBorrowed<'a>) -> Self {
        Self {
            iccid: iccid.into(),
            note: note.into(),
        }
    }
}
impl<'a> postgres_types::FromSql<'a> for SimInfoBorrowed<'a> {
    fn from_sql(
        ty: &postgres_types::Type,
        out: &'a [u8],
    ) -> Result<SimInfoBorrowed<'a>, Box<dyn std::error::Error + Sync + Send>> {
        let fields = match *ty.kind() {
            postgres_types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };
        let mut out = out;
        let num_fields = postgres_types::private::read_be_i32(&mut out)?;
        if num_fields as usize != fields.len() {
            return std::result::Result::Err(std::convert::Into::into(format!(
                "invalid field count: {} vs {}",
                num_fields,
                fields.len()
            )));
        }
        let _oid = postgres_types::private::read_be_i32(&mut out)?;
        let iccid = postgres_types::private::read_value(fields[0].type_(), &mut out)?;
        let _oid = postgres_types::private::read_be_i32(&mut out)?;
        let note = postgres_types::private::read_value(fields[1].type_(), &mut out)?;
        Ok(SimInfoBorrowed { iccid, note })
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "sim_info" && ty.schema() == "public"
    }
}
impl<'a> postgres_types::ToSql for SimInfoBorrowed<'a> {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let SimInfoBorrowed { iccid, note } = self;
        let fields = match *ty.kind() {
            postgres_types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };
        out.extend_from_slice(&(fields.len() as i32).to_be_bytes());
        for field in fields {
            out.extend_from_slice(&field.type_().oid().to_be_bytes());
            let base = out.len();
            out.extend_from_slice(&[0; 4]);
            let r = match field.name() {
                "iccid" => postgres_types::ToSql::to_sql(iccid, field.type_(), out),
                "note" => postgres_types::ToSql::to_sql(&crate::Domain(note), field.type_(), out),
                _ => unreachable!(),
            };
            let count = match r? {
                postgres_types::IsNull::Yes => -1,
                postgres_types::IsNull::No => {
                    let len = out.len() - base - 4;
                    if len > i32::MAX as usize {
                        return Err(Into::into("value too large to transmit"));
                    }
                    len as i32
                }
            };
            out[base..base + 4].copy_from_slice(&count.to_be_bytes());
        }
        Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "sim_info" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Composite(ref fields) => {
                if fields.len() != 2 {
                    return false;
                }
                fields.iter().all(|f| match f.name() {
                    "iccid" => <iccid_type::Iccid as postgres_types::ToSql>::accepts(f.type_()),
                    "note" => <crate::Domain<&'a str> as postgres_types::ToSql>::accepts(f.type_()),
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
