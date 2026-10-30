# Application home and host lifecycle implementation

Specification: [Milestone 3](milestone-3.md), Phase 3, D1–D17; the Phase 4
handoff and shared roadmap requirements also bind the work.

Baseline: `083ab6f50f`. The branch is `codex/m3-phase3-home`. The prior
`codex/m3-production-host` branch and its requirements-tracing commit
`1111d78663` remain preserved. This plan adopts that commit's distinction:
requirement identifiers belong in the requirement map, while test names state
behavior. Phase 3 and Phase 4 land together, after coordinator certification.

## Boundary review

The adversarial constraint is an owner effect acknowledged by its durable
medium while live publication remains incomplete. Neither a refused or
indeterminate answer, a memory-only operation, nor an unrelated committed
subject may authorize that subject's durable live cutover.

Current authoritative paths:

- `worth-query-host` is an audience facade. Its application-installation
  export names Execution's constructors; it currently has no home composition
  module. Applications must continue to import only Decl and Host.
- Execution's `application_installation/construction.rs` creates its own
  Relational runtime with `builder.build()`. The future host must inject the
  one readmitted runtime through an owner-defined construction port. Holding
  another home beside the current constructor would create two runtimes.
- `worth-store-relational/src/home.rs` currently owns physical opening,
  descriptor admission, incarnation advancement, recovery and the runtime.
  Descriptor binding is derived from a constructed Relational runtime.
  Physical media are qualified under the mutation lease before newness is
  tested; recovery moves the held lease into serving. Preserve that seam.
- `checkpoint_record.rs` selects `relational.checkpoint`, prepares native
  members, holds the selection fence, publishes the root and advances member
  selection. `cold_source.rs` also names this anchor. The application owner
  must ultimately select one `home.checkpoint`; adding another selector is
  not a cutover. Keep the unchanged Relational native section format.
- Current close consumes the home and may return `StillShared` after dropping
  its runtime. That is not D11's usable refused-close posture.
- Relational owns canonical commit/transition meaning, its medium trait,
  canonical encoding and artifact readmission. Store implements this port
  over its physical journals and never interprets owner payloads.

The earliest missing contract is in `durability/medium/binding.rs`:
`BindingState::settle` reduces `Committed` to `Ok(())`. A commit returns
placement/readmission data, but not a consumed durability proof. A transition
returns only unit success. `performed_stream.rs` and
`branch_transition_stream.rs` then perform live cutover. D17 requires a
concrete carried proof at this boundary, before the new home builds on it.

Search of `worth-proof/src` and `worth-foundational/src` at this baseline
found no `DurablyCommitted`, `DurablePublicationFate`, `DurableIdentity`,
`HomeIncarnation` or recovery-stage proof names from D17. Both substrate
crates are already Relational dependencies. Existing sealed owner markers and
witness minting are available; a public unit marker would not be a seal.

Independent Sol-high boundary review inspected these paths. It found that
home extraction must transfer actual lifecycle/checkpoint authority rather
than rename the old home, and accepted the publication-proof prerequisite as
a coherent first vertical slice. The coordinator confirmed this scope is
disjoint from the landed B3 data and B4's tooling work.

## First slice: publication fate to consumed live cutover

This slice establishes the existing Store-to-Relational publication contract.
It does not introduce a directory Query host or claim Phase 3 completion.

### Result and type placement

- Add one closed `DurablePublicationFate<Detail>` in `worth-foundational`,
  exported through its facade. Its exhaustive variants are `Committed`,
  `ProvenNoEffect` and `Indeterminate`. Preserve existing owner detail and
  failure ordering. Remove the separate Relational fate enum and update its
  callers; keep no compatibility alias or dead variants.
- Add plain, move-only `DurablyCommitted<OwnerPublication>` in `worth-proof`.
  Its single shallow parameter retains the actual owner-sealed publication,
  not a boolean, unit success, or independently selectable subject.
- Concrete Relational committed carriers stay in Relational: they retain live
  cold-source/readmission custody and the exact positioned subject. They have
  private construction, no `Clone`/`Copy`, and no public conversion from an
  artifact or source. Only the medium binding's `Committed` branch creates
  them. Reuse existing owner-marker sealing; add no generic authority door,
  public unbound issuer, macro or blanket implementation.
- Durable cutover signatures require the concrete Relational specialization
  and apply the subject it carries. Proof/carrier authority is consumed and
  cannot escape after cutover. Subject data may remain ordinary history.
- Memory-only publication remains an explicit, separate admission path. An
  absent medium never mints `DurablyCommitted`. Existing replay/readmission
  remains distinct from a new durable publication and never fabricates its
  witness either.

### Ordered implementation

1. Add shared fate and proof vocabulary with facade exports. Include valuable
   negative compile proofs for fabrication, a foreign specialization and
   repeated consumption, with a valid counterpart.
2. Make Relational's medium binding preserve the actual committed subject in
   the proof. No-effect still returns the same refusal; indeterminate retains
   its original unresolved subject and blocks later publication as before.
3. Thread the proof through canonical commit publication and consume it at
   the first live cutover. Bind journal readmission from that same subject.
4. Thread positioned branch transitions through live publication and consume
   their proof at append/cutover. Keep explicit memory-only and replay paths,
   the existing reservation, canonical ordering and pre-effect checks.
5. Update the Store medium and exact existing test/testkit consumers to the
   shared fate. The initial exact identifier search found 27 Rust files:
   Relational port/binding/facade and support, two World refusal fixtures,
   one Relational UI fixture, and Store's journal adapter. This is a type
   cutover, not a fixture redesign or A2 migration.
6. Verify the coherent batch once, fix actual failures, commit with the agreed
   coauthor footer, and obtain a fresh bounded Sol-high independent review.

### Proof and verification

| Requirement | Distinguishing evidence |
|---|---|
| D2/D17 shared, exhaustive fates | Owner port and Store adapter compile with the single shared enum; no old enum/alias remains. |
| D4/D17 actual durable proof before live | Sealed concrete committed payload reaches commit and transition cutover; negative compile cases reject fabrication/foreign proof/reuse. |
| No effect and indeterminate grant no cutover | Existing durable-before-live, unresolved-fate and branch-transition controls observe original heads, journals, sequence reuse and typed denials. |
| D13 memory-only semantics | Existing memory-only publication and branch lifecycle controls succeed without a medium or durable witness. |
| World handoff unchanged | The two affected World unresolved-publication fixtures compile and their relevant custody tests pass. |
| Real Store port | A focused existing filesystem Relational-home journey commits, reopens and continues through the actual journal adapter. |

The requirement map's concrete behavior tests are:

- D4/D17 committed subject:
  `every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position`.
- D4/D17 proven-no-effect:
  `a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live` and
  `a_refused_position_is_not_consumed`.
- D4/D17 indeterminate:
  `an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike`,
  `reopening_resolves_an_absent_unresolved_commit_as_never_committed` and
  `reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`.
- D4/D17 transitions:
  `every_transition_fate_preserves_the_durable_before_live_boundary` and
  `an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`.
- D13 memory/replay:
  `a_memory_only_runtime_publishes_without_a_medium`,
  `memory_commit_waits_behind_a_transition_without_consuming_its_candidate` and
  `partial_replay_includes_an_intermediate_fork_without_its_own_commit`.
- Medium ownership:
  `a_forked_runtime_never_inherits_the_medium`,
  `recovery_moves_the_medium_into_the_rebuilt_runtime` and
  `an_attachment_the_medium_moved_out_of_publishes_nothing`.
- World handoff:
  `unresolved_relational_durability_retains_a_possible_effect` and
  `an_unresolved_durable_fork_retains_its_attempt_until_reopen`.
- Store port:
  `reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition`.

The fabrication, foreign-specialization and reuse compiler fences, with their
valid counterpart, are executable `DurablyCommitted` API examples in
`crates/worth-proof/src/durable_publication.rs`.

Use the existing runners and scope discovery. The planned `loop` executable
is not present at this baseline (scoped filename search under scripts/tools
found no developer-loop entry); do not implement a substitute framework.
Tier 1 uses focused touched-owner compilation and named affected tests, not
the full Store ordinary suite or cumulative oracle. Run full boundary,
generated-context, formatting and dirty/range line-cap checks explicitly at
completion of this planned slice, not after individual writes. Record exact
commands, head, clean/dirty state, times, exit and test counts in lane logs.
Use pinned Rust 1.99.1 and committed profiles, preserve MSRV 1.94, and export
no artifact-changing settings. Never run ignored, Query-scale or worth-ui
tests. Timing-sensitive/process captures require an exclusive agreed slot.

### Next ownership transfer and reserved handoff

After this prerequisite, `worth-store-application` takes physical lifecycle,
descriptor/registry and single checkpoint selection. `worth-store-relational`
retains only its medium, cold source and narrow section preparation/readmission
contracts; delete `RelationalStoreHome` rather than wrap it. Keep private cold
internals private. The future descriptor admits declared program/schema/roster
binding before runtime construction and advances incarnation before service.

D1's exact future config edits are confined to `road1.toml` dependency denials
and source/dependency allowlists: only the Host production composition module
names Store Application; other Query crates cannot depend on Store; Store
Application implements owner ports without depending on Host; the application
oracle is a leaf allowed to depend on Host and Application. This prerequisite
does not need those new dependencies or config edits.

The host then composes the one supplied, readmitted runtime, Store-proven
new/existing disposition, program/roster, trusted services and original
execution authority. Only genuine newness runs initial state. D15 refusals,
the frozen interim checkpoint door, typed transitions/discard/fencing, honest
close/checkpoints and public docs remain explicit Phase 3 work.

Phase 4 owns World durable identities, occurrences, attempt associations,
custody reconciliation and original-result retry. Reserve its owner sections
and ordering, but do not implement its catalogs or mint recovered application
history from graph/checkpoint presence in this slice.

B3's measured catalogs, compiler goldens and test-cost plan, B4's tooling,
worth-ui, broad A2 migrations and full phase/landing certification remain
outside this slice. Any shared manifest, testkit or gate overlap is coordinated
before editing; the coordinator owns integration and final landing.
