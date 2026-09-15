--! insert_sim
INSERT INTO sims (iccid, note, info) VALUES (:iccid, :note, :info);

--! select_sims: (iccid, note, info)
SELECT iccid, note, info FROM sims;
