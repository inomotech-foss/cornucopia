// This example is the async basic example, generated against a shared runtime
// crate. The generated crate does not contain the client scaffold, it depends
// on `db-runtime` for it, but its public API is unchanged: everything below is
// reached through `cornucopia::` exactly as it would be otherwise.
use cornucopia::{
    client::Params,
    deadpool_postgres::{Config, CreatePoolError, Pool, Runtime},
    queries::{
        module_1::insert_book,
        module_2::{AuthorNameStartingWithParams, author_name_starting_with, authors, books},
    },
    tokio_postgres::NoTls,
    types::SpongebobCharacter,
};

#[tokio::main]
pub async fn main() {
    let pool = create_pool().await.unwrap();
    let client = pool.get().await.unwrap();

    let authors = authors().bind(&client).all().await.unwrap();
    dbg!(authors);

    insert_book().bind(&client, &"Moby Dick").await.unwrap();

    let uppercase_books = books()
        .bind(&client)
        .map(|book_title| book_title.to_uppercase())
        .all()
        .await
        .unwrap();
    dbg!(uppercase_books);

    // Named parameter structs come from the generated crate, the `Params`
    // trait they are used with comes from the runtime crate.
    let starting_with = author_name_starting_with()
        .params(&client, &AuthorNameStartingWithParams { start_str: "Jo" })
        .all()
        .await
        .unwrap();
    dbg!(starting_with);

    // Custom PostgreSQL types are still generated per crate
    dbg!(SpongebobCharacter::Patrick);
}

async fn create_pool() -> Result<Pool, CreatePoolError> {
    let mut cfg = Config::new();
    cfg.user = Some(String::from("postgres"));
    cfg.password = Some(String::from("postgres"));
    cfg.host = Some(String::from("127.0.0.1"));
    cfg.port = Some(5435);
    cfg.dbname = Some(String::from("postgres"));
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
}
