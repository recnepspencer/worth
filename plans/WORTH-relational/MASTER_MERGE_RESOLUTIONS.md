# Master integration into the approved Relational cutover

Merged milestone `562d79efeb`, including master `a7c68255a9`, onto the approved sweep and recovery follow-up `b911880cc7`.

The non-trivial resolutions are:

- Authorization adjacency retains master's `Acquired` visit. Both preparation-only cases return before the later match; the retained unreachable proof names those exact control-flow branches.
- Structural identity recovery uses master's actually published client-key symbol, preserving checkpointed meaning instead of adding an unrelated unpublished symbol. The test and all assertions remain.
- Invariant execution retains master's separated native/custom stages, metrics owner, and completed reduction result. The moved packet-context proof belongs beside the assertion in `engine/metrics.rs`; the obsolete duplicate engine methods are removed.
- Relation integrity retains master's `relation_changes` owner and affected-pair tracking for create, revalidate and delete. Scope-construction proofs remain with the two minimum-scope lookups. The duplicate old collection methods are removed.
- Checked planning retains master's actual request-memory admission and stage parameter. The scratch/retention boundary test from this branch remains compiled, adapted to two simultaneously live extents under that real request policy. `0c2bc87e4a` retired the arbitrary half-ceiling and its `CheckedPlan` adapter; those obsolete production mechanisms are not restored.
- The custom scratch-refusal fixture preserves master's exact PolicyMemoryExhausted assertion and its requested/admitted evidence, plus successful extraction of typed native construction. Its test is retained once under the explicit memory child.
- Structural custom-invariant tests retain master's shared-versus-separate real worlds and every behavioral assertion, with construction Results extracted in the new world fixture.
- Query's custom-kernel fixture retains master's POLICY request and extracts native construction. All other incoming native constructors that require extraction are adapted in place.
- Master's prefix reducer replaces the two obsolete reducer files. All new retained extraction sites name their FnOnce or contiguous-stage construction proof; the second memory-lineage reservation returns the existing typed admission refusal rather than asserting ambient scope.
- Master's slot traversal stays on the Head-only CurrentPartitionAccess views and remains inside the existing probe court. D7 and D12 courts and their reviewed residency adaptations are retained.

No test was removed as a conflict resolution. The test-name reconciliation also accounted for upstream renames: the four stage-consult cases are now named per safe point; the index ceiling regression is master's `one_large_index_build_stops_at_its_request_memory_without_publication` with unchanged no-publication and prior-generation checks; projection mask rejection now asserts the typed admission denial. The constructor-only UnsupportedAspectFieldTarget presentation test was already retired by the approved sweep because `a707a48b1b` removed its producer; its attribution remains in CUTOVER_AUDIT.md and the live entity-field patch denial courts retain the actual typed target rejection.

Before merge, the approved expanded census was 363 sites: 297 expect/unwrap/panic/unreachable forms and 66 assertions. The final merged census is also 363: 295 earlier forms and 68 assertions, across 1,812 production modules. The final production inventory is target/lane-evidence/landing-panic-sites-catalog.json, recorded by landing-panic-census-catalog.log. Inline Proof comments remain authoritative.

Source follow-through commits 5e56469fae and 7939ff1ab2 extract incoming fixture Results and align the predecessor-frame diagnostic with native format 13 without weakening an assertion. Product corrections 76b041c976 and 726f20fc64 separate live reservations from the durable publication frontier, then place the latter in the catalog insertion owner so admitted batches cannot omit it. The exact cancellation court and all six previously failing Query restore cases pass. Final executable source is 7939ff1ab2; subsequent audit-only changes reuse its evidence. Every lane attempt, failure correction, count and source-reuse rationale is indexed in target/lane-evidence/INDEX.md.
