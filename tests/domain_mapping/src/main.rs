use codegen::{
    queries::sims::{
        InsertSimParams, insert_sim, select_sim_iccid, select_sim_id_and_iccid, select_sims,
    },
    types::SimInfoBorrowed,
};
use iccid_type::Iccid;
use postgres::{Client, Config, NoTls};

pub fn main() {
    let client = &mut Config::new()
        .user("postgres")
        .password("postgres")
        .host("127.0.0.1")
        .port(5435)
        .dbname("postgres")
        .connect(NoTls)
        .unwrap();

    test_mapped_and_unmapped_domains(client);
    test_row_override(client);
    test_row_override_with_copy_field(client);
}

// A mapped domain (`iccid`) is used by its configured Rust type, both as a parameter and,
// nested in a composite, as a row field. An unmapped domain (`sim_note`) falls back to its
// base type in both positions. Neither needs an explicit SQL cast.
pub fn test_mapped_and_unmapped_domains(client: &mut Client) {
    let params = InsertSimParams {
        iccid: Iccid("1234567890123456789".to_string()),
        note: "hello",
        info: SimInfoBorrowed {
            iccid: Iccid("9876543210987654321".to_string()),
            note: "nested",
        },
    };

    insert_sim()
        .bind(client, &params.iccid, &params.note, &params.info)
        .unwrap();

    let rows = select_sims().bind(client).all().unwrap();
    let row = rows.into_iter().next().unwrap();

    assert_eq!(row.iccid, "1234567890123456789");
    assert_eq!(row.note, "hello");
    assert_eq!(row.info.iccid, Iccid("9876543210987654321".to_string()));
    assert_eq!(row.info.note, "nested");
}

// PostgreSQL reports a plain `SELECT iccid FROM sims` column as its base type (text), never
// as the `iccid` domain: only a `col: domain_name` row override recovers the mapped type here.
pub fn test_row_override(client: &mut Client) {
    let iccids = select_sim_iccid().bind(client).all().unwrap();
    assert_eq!(iccids, vec![Iccid("1234567890123456789".to_string())]);
}

// A named (multi-field) row whose only non-`Copy` field is a mapped domain: this only compiles
// if the generated `SelectSimIdAndIccidBorrowed` doesn't declare an unused lifetime parameter.
pub fn test_row_override_with_copy_field(client: &mut Client) {
    let rows = select_sim_id_and_iccid().bind(client).all().unwrap();
    let row = &rows[0];
    assert_eq!(row.id, 1);
    assert_eq!(row.iccid, Iccid("1234567890123456789".to_string()));
}
