# Shared runtime example

Every generated crate normally carries its own copy of the client scaffold. A
project that generates several of them can move that scaffold into a single
crate instead, using the `shared-runtime` option.

`db_runtime` is generated once, without a database:

```bash
cornucopia runtime --config runtime.toml
```

`shared_runtime_codegen` is then generated as usual. Because `cornucopia.toml`
sets `shared-runtime`, it depends on `db-runtime` and re-exports the scaffold
rather than emitting it:

```bash
cornucopia schema schema.sql
```

The generated crate's public API is the same either way, as `src/main.rs`
shows.
