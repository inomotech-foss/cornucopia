// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct InsertSimParams<'a, T1: crate::StringSql> {
    pub iccid: iccid_type::Iccid,
    pub note: T1,
    pub info: crate::types::SimInfoBorrowed<'a>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SelectSims {
    pub iccid: String,
    pub note: String,
    pub info: crate::types::SimInfo,
}
pub struct SelectSimsBorrowed<'a> {
    pub iccid: &'a str,
    pub note: &'a str,
    pub info: crate::types::SimInfoBorrowed<'a>,
}
impl<'a> From<SelectSimsBorrowed<'a>> for SelectSims {
    fn from(SelectSimsBorrowed { iccid, note, info }: SelectSimsBorrowed<'a>) -> Self {
        Self {
            iccid: iccid.into(),
            note: note.into(),
            info: info.into(),
        }
    }
}
use crate::client::sync::GenericClient;
use postgres::fallible_iterator::FallibleIterator;
pub struct SelectSimsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c mut C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s postgres::Statement>,
    extractor: fn(&postgres::Row) -> Result<SelectSimsBorrowed, postgres::Error>,
    mapper: fn(SelectSimsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> SelectSimsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(SelectSimsBorrowed) -> R,
    ) -> SelectSimsQuery<'c, 'a, 's, C, R, N> {
        SelectSimsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub fn one(self) -> Result<T, postgres::Error> {
        let row = crate::client::sync::one(self.client, self.query, &self.params, self.cached)?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub fn all(self) -> Result<Vec<T>, postgres::Error> {
        self.iter()?.collect()
    }
    pub fn opt(self) -> Result<Option<T>, postgres::Error> {
        let opt_row = crate::client::sync::opt(self.client, self.query, &self.params, self.cached)?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub fn iter(
        self,
    ) -> Result<impl Iterator<Item = Result<T, postgres::Error>> + 'c, postgres::Error> {
        let stream = crate::client::sync::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )?;
        let mapped = stream.iterator().map(move |res| {
            res.and_then(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
        });
        Ok(mapped)
    }
}
pub struct IccidtypeIccidQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c mut C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s postgres::Statement>,
    extractor: fn(&postgres::Row) -> Result<iccid_type::Iccid, postgres::Error>,
    mapper: fn(iccid_type::Iccid) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> IccidtypeIccidQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(iccid_type::Iccid) -> R,
    ) -> IccidtypeIccidQuery<'c, 'a, 's, C, R, N> {
        IccidtypeIccidQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub fn one(self) -> Result<T, postgres::Error> {
        let row = crate::client::sync::one(self.client, self.query, &self.params, self.cached)?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub fn all(self) -> Result<Vec<T>, postgres::Error> {
        self.iter()?.collect()
    }
    pub fn opt(self) -> Result<Option<T>, postgres::Error> {
        let opt_row = crate::client::sync::opt(self.client, self.query, &self.params, self.cached)?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub fn iter(
        self,
    ) -> Result<impl Iterator<Item = Result<T, postgres::Error>> + 'c, postgres::Error> {
        let stream = crate::client::sync::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )?;
        let mapped = stream.iterator().map(move |res| {
            res.and_then(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
        });
        Ok(mapped)
    }
}
pub struct InsertSimStmt(&'static str, Option<postgres::Statement>);
pub fn insert_sim() -> InsertSimStmt {
    InsertSimStmt(
        "INSERT INTO sims (iccid, note, info) VALUES ($1, $2, $3)",
        None,
    )
}
impl InsertSimStmt {
    pub fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a mut C,
    ) -> Result<Self, postgres::Error> {
        self.1 = Some(client.prepare(self.0)?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c mut C,
        iccid: &'a iccid_type::Iccid,
        note: &'a T1,
        info: &'a crate::types::SimInfoBorrowed<'a>,
    ) -> Result<u64, postgres::Error> {
        client.execute(self.0, &[iccid, &crate::Domain(note), info])
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::sync::Params<
        'c,
        'a,
        's,
        InsertSimParams<'a, T1>,
        Result<u64, postgres::Error>,
        C,
    > for InsertSimStmt
{
    fn params(
        &'s self,
        client: &'c mut C,
        params: &'a InsertSimParams<'a, T1>,
    ) -> Result<u64, postgres::Error> {
        self.bind(client, &params.iccid, &params.note, &params.info)
    }
}
pub struct SelectSimsStmt(&'static str, Option<postgres::Statement>);
pub fn select_sims() -> SelectSimsStmt {
    SelectSimsStmt("SELECT iccid, note, info FROM sims", None)
}
impl SelectSimsStmt {
    pub fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a mut C,
    ) -> Result<Self, postgres::Error> {
        self.1 = Some(client.prepare(self.0)?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c mut C,
    ) -> SelectSimsQuery<'c, 'a, 's, C, SelectSims, 0> {
        SelectSimsQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &postgres::Row| -> Result<SelectSimsBorrowed, postgres::Error> {
                Ok(SelectSimsBorrowed {
                    iccid: row.try_get(0)?,
                    note: row.try_get(1)?,
                    info: row.try_get(2)?,
                })
            },
            mapper: |it| SelectSims::from(it),
        }
    }
}
pub struct SelectSimIccidStmt(&'static str, Option<postgres::Statement>);
pub fn select_sim_iccid() -> SelectSimIccidStmt {
    SelectSimIccidStmt("SELECT iccid FROM sims", None)
}
impl SelectSimIccidStmt {
    pub fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a mut C,
    ) -> Result<Self, postgres::Error> {
        self.1 = Some(client.prepare(self.0)?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c mut C,
    ) -> IccidtypeIccidQuery<'c, 'a, 's, C, iccid_type::Iccid, 0> {
        IccidtypeIccidQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get::<_, crate::types::IccidRowValue>(0)?.0),
            mapper: |it| it.into(),
        }
    }
}
