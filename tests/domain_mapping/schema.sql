-- Mapped to a custom Rust type via `types.domains`.
CREATE DOMAIN iccid AS text CHECK (value ~ '^[0-9]{19,20}$');

-- Left unmapped, to exercise the base-type fallback alongside the mapped domain.
CREATE DOMAIN sim_note AS text;

-- A domain is only visible as itself in row position when nested in a composite: a
-- bare `SELECT iccid FROM sims` reports the base type, since PostgreSQL resolves
-- domains in a query's output columns before describing them.
CREATE TYPE sim_info AS (
    iccid iccid,
    note sim_note
);

CREATE TABLE sims (
    id serial PRIMARY KEY,
    iccid iccid NOT NULL,
    note sim_note,
    info sim_info
);

-- A composite with no borrowed field at all: `id` is Copy, and `iccid` (nested, so it keeps
-- its true domain type - see above) is a mapped owned type. Its Borrowed/Params structs must
-- not declare a lifetime parameter that no field uses either.
CREATE TYPE sim_ref AS (
    id integer,
    iccid iccid
);

CREATE TABLE sim_refs (
    id serial PRIMARY KEY,
    value sim_ref
);
