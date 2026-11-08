# Deferred generation publication - revised implementation plan

Source base:307539feba. Earlier read-only stop premise is superseded: independent boundary review identified the existing, unheld preflight/cutover/settlement-custody route. Heldport.rs andport_handoff.rs remain untouched. Exact grant commands/results are in deferred-geometry-grants.json and the full audit logs; every file edited below must pass both peers.

## Boundary

Initial preparation derives/funds the generation payload. Its metadata is installed later against a live Native catalog. A preparation-time entry count cannot authorize later geometry; usize::MAX overquotes it. Actual publication needs a catalog-owner gate from admission through installation. The borrowed Native token must perform no owning payload cleanup. Settlement can be delayed, so neither the gate nor actual publication may wait for settlement.

Existing analogue: Native definition-publication admission before the borrowed handoff. The new route uses the already-owned completion at port_preflight, the existing linear pending-settlement reservation at cutover, and the finalizer for retired cleanup. It adds no alternate owner or caller door.

## Complete source batch

1. Catalog owner captures a sealed, shallow actual-geometry quote from checked retained-entry count plus the batch and its global/branch scopes. Remove the static usize::MAX deferred quote. Quotation callbacks run outside the IndexingState lock.
2. Add a private Native generation-publication gate and owned Drop permit. Poll original request controls and candidate TTL between bounded condvar waits. Every live catalog grower/install/cold merge/cfg(test) corruption joins gate; detached checkpoint-copy restoration remains exclusive. Fork gets a fresh gate; binding clones share the same live gate. No IndexingState guard crosses I/O.
3. PreparedGenerationBatch retains its Vec and catalog binding, with exhaustive Unadmitted/Admitted/Published local state. Initial preparation funds payload/binding only. Admission in Native preflight obtains gate and quotes current catalog using the same concrete original ExecutionRequest and a metadata memory ticket. All checked work/memory denial occurs before lending the borrowed token.
4. `port_preflight.rs` calls completion admission before installing its Reserved record. No held handoff edit. TTL and original controls govern waits; existing perform_before checks its later narrowed deadline before effects.
5. Cutover invokes the linear settlement reservation's mutable publish step immediately after successful canonical movement and BEFORE read-state release / after-linearization pause / settlement authorization. Record retains payload ownership; no checkout or owned temporary carries it through the caller guard. Installation drains Vec into the catalog, retains Vec allocation, identifiers and tickets in the record, and releases only the lightweight permit immediately. Fresh owner-reserved generation identities prevent replacing/destroying an existing generation payload. No fallible funding, user callback, I/O, or owning prepared-payload cleanup occurs in this step.
6. Settlement finalizer consumes already-published retirement cleanup; it never publishes a second time. It drops all prepared data before funding tickets. Pre-effect refusals/unwinds keep cleanup in Reserved custody until outer preflight cleanup, outside the borrowed handoff.
7. Owner-level focused evidence covers actual geometry after intervening catalog growth, cancellation while waiting, abandoned admission releasing its gate, and retained Vec backing after install. These are catalog/coordination kernel claims using plain ordering inputs, not fabricated Native commit authority. Unchanged public Topology/D18/graceful/Bank courts prove the real route.

## Planned layout

Existing actual quote owner: indexing/generation_catalog/admitted_publication.rs. Gate coordination: new indexing/generation_publication_gate.rs. Index owner fields/clone/fork and live writers: indexing.rs, indexing/admitted_selection.rs, indexing/cold_checkpoint.rs. Payload phases: indexing/prepared_publication.rs. Original-request metadata payment: indexes/authority/maintenance/candidate.rs. Completion methods/final retirement: authority/commit/pipeline/publication_execution.rs. Linear record handoff: runtime/state/runtime_state/publication_settlement_registry{,/record}.rs. Named effect/preflight calls: mvcc/publication/port_preflight.rs andport_cutover.rs. Tests stay adjacent to their actual owners. No public consumer API, feature, grant, Full proof, court assertion or gate baseline change.

## Verification budget

Batch everything before one Root Native --tests check; fix all diagnostics together if needed. Then serial focused builds (C:>=25GiB) and one named portfolio: affected Native owner tests, five Topology parents, D18three+registered graceful exclusive0..2, Banktwo default-stack parents. No ignored/exhaustive/scenario/query-scale. Focused failed-name reruns only if required. Fmt/dirty line caps/boundary/context at completion, fresh independent review without reviewer builds, coherent source/evidence commits. If accurate/proportional work still exhausts, measure unchanged failing request/state and stop; never raise8M.


Before first compilation: all16Rustfiles fit400lines. Existing catalog installation was split into its named installation owner; prepared branch lookup keys are allocated before handoff and unused keys stay parked, avoiding transient owned-string destruction in the borrowed cutover. Original control checks precede candidateTTL checks; abandoned admission drops its permit before owning payload fields. Kernel cancellation test exercises a contended re-poll rather than refusing its first poll. No source public facade,8M allowance or court was changed.

Final compiler contract: publication RequestStopped preserves the concrete original WorkCeilingDenial; metadata charge overflow is Admission(ChargedBytesOverflow), resize refusal is Admission(MemoryExhausted), and actual checkpoint stops remain Stopped. The first check found five incorrect Native-cause conversions, fixed as one family before any runtime test.
