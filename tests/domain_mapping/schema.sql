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
