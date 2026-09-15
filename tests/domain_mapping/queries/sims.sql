--! insert_sim
INSERT INTO sims (iccid, note, info) VALUES (:iccid, :note, :info);

--! select_sims: (iccid, note, info)
SELECT iccid, note, info FROM sims;

-- PostgreSQL reports a plain `SELECT iccid` column as its base type (text), never as the
-- `iccid` domain: this override is the only way to get the mapped type back for it.
--! select_sim_iccid: (iccid: iccid)
SELECT iccid FROM sims;
