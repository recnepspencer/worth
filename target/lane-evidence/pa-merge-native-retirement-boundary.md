# Native retirement warning boundary

Incoming41ab134fa1 retires readmit_for_request and its sole direct caller test.
At resolved2a51552fc5 merge, Root check emits55 warnings at32unique locations;
Query and Store each repeat32Native warnings. Full diagnostics are in
pa-merge-{root,query,store}-diagnostics.json. No warning suppression is added.

Independent reviewer pa_safe_merge_intent_review read the warning graph without
builds or edits. No Root feature revives original full-root mutation replay.
The disconnected original dispatcher/revision/deletion/adjacency chain can go,
but these live shared subjects must stay: apply_prepared_row/apply_relation_row,
ordinary delete/replace apply_using, PendingRecordAllocations::finish_mutation,
unchanged-root request readmission, required_text::validate, inherent interner
retained_text, FullKindProjection::capture/insert and ordinary residency creation.

Nine old tests/assertion clusters need truthful current coverage:
- Three original allocation cursor tests: exact IDs/origins/consumption;
  class/partition/ordinal/origin-generation negatives; missing/excess evidence
  and reserve/record/abandon/retry behavior.
- Original symbol IDs, recorded-reference subset and lookup-without-reservation.
- Exact region control charge through final real Weak; exact refusal before Arc.
- Independent full-projection MessagePack preimage/history/content/hash counts.
- Escaped whole PartitionContentCommitment exact allocation_bytes lifetime.
- Genuine Head-prefix InvalidHistoryCoverage without extra source reads.

Current image operational and region_memory courts preserve exact created ID,
receipt/bases/field/successor and positive/final-zero retained memory, but not all
nine exact claims. Shipping record_allocation_recovery courts cover reuse,
burned frontiers, missing/excess evidence, live-slot refusal and wrong canonical
effect; these are not blanket substitutes for old cursor mechanics.

Current narrow owners are cell/selected_readmission.rs::control_memory with
RegionCellWeak, FullKindProjection::capture + kind_commitment::digest_full,
and PartitionContentCommitment::capture_full_for_request with its real clone.
No new public observer is needed for their remaining funding/value claims.
All three checkpoint_capture/complete/tests.rs fixtures also call obsolete
initial_for_request; their fixture must move to a current owner before using
those tests as independent replacement evidence.

Two contract gaps require a ruling: image restoration has no record-before-
reserve/retry/abandon cursor, and no recorded-reference reservation-input symbol
interface. A synthetic adapter/door or cfg(test)-only legacy interpreter would
not establish a shipping claim. An asynchronous question asks whether to retire
those obsolete interface contracts under41ab while preserving remaining claims,
or route this warning family to P-A landing2. No dependent repair is authorized
by silence. Other compilation/Tier2 collection proceeds unchanged.
