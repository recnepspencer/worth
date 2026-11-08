# Pre-HistoricalSelection stores

Recorded from public master `fb0870356f39e1e991adf6ca0d47987a8e558a7e` on
2026-11-05, before the HistoricalSelection origin was added. The writer's
production library tree was unchanged. The only overlay mounted the ignored
`record_pre_historical_selection_chk18_store` recorder in the existing Native
certification court. It used ordinary source creation, genuine installed producer
publication and Native checkpoint capture. The JSON records the actual issued
identities and publication meaning alongside the checkpoint bytes.

Recorder: `relational/authored_retirement/legacy_history_recorder.rs` in
`worth-court-relational-certification`. The original recorder was run explicitly in the pre-change checkout. These CHK18
bytes and their JSON remain archived unchanged. Master later displaced CHK18 with
CHK19 (436be541e6) and CHK20 (5deccad4ed); its decoder now requires CHK20.
No fixture bytes were hand-edited.

## Current CHK20 compatibility proof

`history_pre_historical_selection_chk20.chk20` and its matching JSON were recorded
from public master `81dd70ad6711f97aa1ff4dd3186c01752ce3756b` on 2026-11-07.
That writer has no HistoricalSelection origin variant. Its Native library tree
was unchanged; the only source overlay mounts the same ignored recorder
(`record_pre_historical_selection_chk20_store`) in the existing court module.
The recorder uses genuine source creation, owned publication and checkpoint
capture. Assets are copied byte-exactly from its output.

`chk20_store_written_before_historical_selection_still_opens` reads these bytes
with the HistoricalSelection reader and retains all identity, publication
membership, meaning and liveness assertions. This follows master's format
cutover while preserving the additive-origin compatibility proof. It neither
changes the Native format nor grants legacy-frame fallback.
