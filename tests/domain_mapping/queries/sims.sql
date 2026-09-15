--! insert_sim
INSERT INTO sims (iccid, note, info) VALUES (:iccid, :note, :info);

--! select_sims: (iccid, note, info)
SELECT iccid, note, info FROM sims;

-- PostgreSQL reports a plain `SELECT iccid` column as its base type (text), never as the
-- `iccid` domain: this override is the only way to get the mapped type back for it.
--! select_sim_iccid: (iccid: iccid)
SELECT iccid FROM sims;

-- A named (multi-field) row whose only non-Copy field is a mapped domain: `id` is Copy, and
-- the mapped `iccid` field is owned (not borrowed), so the borrowed row must not declare a
-- lifetime parameter that no field uses.
--! select_sim_id_and_iccid: (iccid: iccid)
SELECT id, iccid FROM sims;

-- A named (multi-field) params struct whose only non-Copy field is a mapped domain: `:iccid`
-- is assigned to a column (so PostgreSQL reports it as the domain), `:id` is Copy, and the
-- mapped `iccid` field is owned, so the params struct must not declare an unused lifetime.
--! update_sim_iccid
UPDATE sims SET iccid = :iccid WHERE id = :id;

-- A composite (not just a row/params struct) whose only non-Copy field is a mapped domain:
-- `sim_ref`'s Borrowed/Params representation must not declare an unused lifetime either.
--! insert_sim_ref
INSERT INTO sim_refs (value) VALUES (:value);

--! select_sim_ref: (value)
SELECT value FROM sim_refs;
