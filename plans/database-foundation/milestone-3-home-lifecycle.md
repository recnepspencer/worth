# Application home and host lifecycle implementation

Specification: [Milestone 3](milestone-3.md), Phase 3, D1–D17; the Phase 4
handoff and shared roadmap requirements also bind the work.

## Cold member custody retirement prerequisite

This prerequisite retires one actual source-issued registration, or one local
share of it. It does not establish cold graph retirement, NativeClosed,
ApplicationHome close, physical close, or completion of D11. The accepted
Memory retirement path remains separate from cold provider obligations.

The adversary is the original Store guard retaining selected checkpoint
members while another independent cold reader remains live. Hold its actual
registry lock, stop after a partial member release, switch sources, clone the
original token, and finally abandon an incomplete task. Bypassing issuance
identity must fail when an unrelated provider falsely returns Complete;
neither callback on that provider may execute. A source with the same Serving
runtime but a different registry is also foreign. Successful local release
must preserve the other reader, actual cold reads and checkpoint generation.

The Native facade keeps `RelationalDurableMemberCustody` opaque. Its existing
`new(owner)` constructor remains supported but cannot issue bounded retirement
for an unknown provider. Add `new_from_source(owner, Weak<dyn
RelationalDurableColdSource>)`; the original owner and weak issuing-source
identity reside together in one sized Arc cell containing the boxed original
payload. Acquisition pays one additional allocation and indirection. Cloning
custody retains the same actual registration, rather than registering again.

Consuming `into_retirement()` returns a non-Clone
`RelationalDurableMemberRetirement`. `advance(&mut self, &ExecutionRequest,
&dyn RelationalDurableColdSource)` returns the typed observations Advanced,
Busy, Stopped(original cause), Refused(typed cause), or Complete. The public
refusal distinguishes UnsupportedProvider, ForeignSource and ForeignCustody;
no callback may translate these into successful release. An already recorded
Complete is observed without further callbacks or effects. Otherwise check the
original request, then compare the borrowed source's data address against the
retained issuing Weak's data address before any callback or shared-token
consumption. Never upgrade the Weak: its retained allocation prevents address
reuse without introducing a temporary last strong source owner into advance.

The existing cold-source trait gains default-Unsupported provider callbacks:
`validate_member_custody_retirement(&self, &(dyn Any + Send + Sync))` returns
the typed refusal or success; `advance_member_custody_retirement(&self,
&ExecutionRequest, &mut (dyn Any + Send + Sync))` returns the same typed
advancement observations. Only the provider sees the borrowed opaque payload;
there is no caller getter or downcast door. Store downcasts its original
private guard and validates BOTH registry and Serving Arc identity before
effects. Immutable validation performs no IO and acquires no locks; every
advancement lock is nonwaiting. Every effectful call repeats issuance validation, preventing a source
switch after partial progress.

Use `Arc::into_inner` for atomic local-share release or unique-cell transfer,
never `strong_count` to decide ownership. Complete for a nonlast share means
only that this local token relinquished custody. For a unique guard, retain
the original member Vec, registry, cursor and optional Serving Arc. A step
uses `try_write` on the actual registry, decrements/removes one original member
and advances the cursor under that same lock. Busy, stop and refusal retain
the authentic remaining payload. On the final member detach the guard's extra
Serving Arc while the matching borrowed Store source still owns Serving.
Detach whenever the cursor reaches the original members.len(), including an
originally empty registration.
Keep the disarmed guard, original Vec and registry opaque until explicit
unbudgeted reclamation. Ordinary Drop retires only the remaining suffix;
incomplete task Drop is unbudgeted abandonment and proves no successful close.
The composite Native `RegionMemberExport` remains UnsupportedProvider here.

`into_retired()` returns the opaque retired custody only after Complete,
otherwise returns the original task. `retired.reclaim()` is explicitly
unbudgeted. Advancing performs no new allocation, reservation, rehash or
shrink, arbitrary payload destruction or last physical-provider destruction.
Each actual member step is O(log distinct registered members); BTreeMap
removal may rebalance and deallocate nodes, and Arc share release may reclaim
a small control allocation. There is no hard O(1), no-deallocation, or
destructor wall-clock claim.

Destination topology (existing parent modules remain canonical):

```text
worth-relational/src/durability/medium/
  member_custody.rs                [revise: opaque acquisition/clone]
  member_custody/retirement.rs     [create: consuming retirement protocol]
  cold_source.rs                  [revise: provider callbacks]
worth-store-relational/src/
  cold_source.rs                  [revise: cyclic source construction]
  cold_source/member_custody.rs    [revise: original guard/cursor]
  cold_source/member_custody/retirement.rs [create: actual registry effects]
  cold_source/member_custody/retirement_tests.rs [create: private proof]
  cold_source/member_custody/retirement_tests/directory.rs [create: genuine FS fixture]
```

Store's private `new(serving) -> Arc<Self>` uses `Arc::new_cyclic` and retains
weak_self. The existing certification observer is supplied by private
`new_with_commit_read_observer` before construction; remove its mutable
postconstruction setter. Cut over all four existing construction sites in
production attachment, certification observer attachment and the two private
test modules. No public source extraction, new control, registry, worker,
counter, scheduler, or ownership mode belongs in this prerequisite.

The following exact three private tests are a narrow specification exception
to Iteration Law 2's foreign-runtime construction prohibition. They live only
in `worth-store-relational/src/cold_source/member_custody/retirement_tests.rs`:

- `cold_source::member_custody::retirement_tests::selected_directory_guard_retirement_preserves_partial_custody_and_cold_reads`
- `cold_source::member_custody::retirement_tests::issued_source_identity_refuses_fake_and_switched_sources_before_callbacks`
- `cold_source::member_custody::retirement_tests::shared_local_retirement_preserves_other_owners_and_unbudgeted_drop_retires_only_remaining_suffix`

These tests alone may construct genuine foreign Native/Store runtimes because
the original guard, registry lock, cursor and detached Serving field cannot be
observed through the public court without an extraction door. This grants no
family exemption and no new public test access. Their private `directory.rs`
child may hold only their shared concrete filesystem producer/recovery fixture
and cold-read/generation oracle; this exact helper is included in the same
construction exception, introduces no additional tests, and is used only by
the three named tests. Keep its Native builder/schema/ordinary commits,
StoreJournalMedium, production external-member capture/preparation, anchored
publication/read and recovery calls explicit in the review inventory. Split
this fixture to obey the ordinary 400-line cap; do not add a test family.
The registry's private production acquisition core may return its genuine
registered guard to the source-local wrapper before opaque Native issuance;
the same core supplies the named private negative affinity test. No public
payload access is added. Same-Serving/different-registry validation must be
tested independently of the earlier issuance check. A negative-only test may
temporarily substitute another actual filesystem Serving Arc in that genuine
guard, require ForeignCustody, then restore the original before Drop. This
does not fabricate successful custody, counts or publication evidence.
Use the existing
`cold_source::test_store::initialize`: its `physical_record_journeys::media`
qualifies actual filesystem media. Produce Native schema and ordinary
transactions, production section capture/encoding, actual anchored publication
and selected membership, then production recovery with that same Store source.
Read actual cold record/history data. Fabricated member bytes, locators,
snapshots, authorities and registry-count-only success are insufficient.

Cover registry Busy, partial stop/rejoin, fake-source and source-switch refusal
before callbacks, same-Serving foreign registry, local shared release,
remaining-suffix Drop exactly once, independent reader protection, retained
physical protection after source loss and unchanged checkpoint generation.
This is a filesystem provider integration proof, not an ApplicationHome proof.
Update `crates/worth-relational/README.md`, `docs/capability-index.md` and
`workspaces/worth-store/crates/worth-store-relational/README.md` with
the facade, issuer affinity, local-share meaning, unsupported composite case,
cost and unbudgeted reclamation. Verify those documents against actual exports.

Implement acquisition/trait/progress first, then Store guard/construction
cutover and honest fixtures, then run focused provider tests, relevant Native
regressions, formatting, scoped line caps, boundary and context checks. Fresh
independent source review must approve the coherent batch before integration.

## Existing Query publication and service readmission

This batch follows the real Store NoPendingOwnerAdmission gate specified in
[Durable owner contracts](milestone-3-durable-owner-contracts.md). It reopens
the same initialized filesystem Home without initializer, seed-supplier or
business-handler replay. It supports zero actual retained World recovery
custody and explicitly admits only the supported persisted Query families.
Full recovered unpublished custody, original-receipt retry, automatic
checkpointing, transitions and full D8/D9/D11 certification remain successors.

The adversary has genuine Native history beyond the selected World root.
Business reads must use the selected World Native basis; the latest-Native
scan independently restores durable outcome identity frontiers. Native 9 /
World 8 with unpublished custody remains an owning refusal before service.
An empty new Query cache or completion counts cannot erase that custody.

Freeze the following construction boundary:

- `PreparedHomeNativeMeaning::finish_existing_home_native_readmission` consumes
  the prepared schema/program meaning, original phase, actual Native runtime,
  recovered authority and recovery outcome. It refuses pending codec state,
  performs real `readmit_recovered_initial_schema_installation`, and uses
  `IndexInstallationPosture::RequireRecovered`. Its result is
  `ReadmittedExistingHomeNativeMeaning`, never a fresh initializer capability.
- The existing-only source attachment returns
  `PreparedExistingHomeQueryRuntime`, with no initializer or seed-publication
  methods. Share private product-equivalent mechanics where appropriate,
  without converting existing preparation to the public fresh preparation
  type or introducing an ownership mode. Only fresh attachment creates
  `WorthQueryProgramActivationSeed`.
- Existing attachment consumes an actual selected `ProductBranchObservation`.
  Both Home preflight and source preparation use its admitted relational
  basis through the existing owner-proven basis bind. Add only the source-owner
  `prepare_product_source_at_basis(execution, &AdmittedRelationalBranchBasis)`
  operation. Remove current-head lookup from Directory Home attachment; keep
  existing unrelated Memory construction behavior.

The one new Native Query entity is `worth-query-bootstrap-publication`, aspect
`bootstrap-publication`, contract revision 1. Required fields are
`format-version: UInt64` (exactly 1), `binding: String`,
`terminal-bootstrap-commit: UInt64`, and UInt64 counts for principal bindings, identity
indexes, application equality indexes, policy entities and policy relations.
The binding is exactly the admitted portable ApplicationHomeBinding described
in the shared contract, encoded by Query's closed versioned codec with checked
lengths, canonical digest rendering, canonical roster order and no trailing
data. Bound decoding/allocation by the actual row and original execution
resources. Reconstruct through ApplicationHomeBinding's validator and readmit
against the actual installed meaning; do not copy Store's private descriptor
codec or serialize live binding ordinals/generations/build UUIDs.

Append this entity after the maximum actually lowered platform kind and its
aspect after all existing platform, projected and aggregate reservations.
Preserve existing IDs; workflow extends beyond inbound completion through
workflow-first + 40, so inbound+1 is not free. Derive allocation from actual
lowering/registry state rather than duplicating offset arithmetic. Missing or
unknown-format records and incompatible recovered schemas return owned refusal;
recordless Homes never regain initialization opportunity or migrate silently.

After all genuine seed batches and index construction, the fresh Directory
`publish_home_query_seed` path commits the completion record in one additional
ordinary Native transaction BEFORE World genesis. Its final-seed locator is
the actual terminal bootstrap commit; the record transaction is a distinct successor
and World genesis binds that record-containing basis. Do not impose this
extra publication on ordinary Memory bootstrap. A failed completion publication
retains the actual prepared state and causes, without retrying accepted seed
work or exposing a service/root missing completion.

The existing seed producer consumes initial activation before its first real
row commit. Thus genuine all-rows-flushed multi-batch publication already has
no pending activation seed, and its accepted last-seed receipt is terminal.
Preserve and verify that actual producer invariant; a synthetic reinstalled
activation seed is not a lawful recovery fixture. Use the issuer-derived
publication.bootstrap_commit_id, build/admit indexes at that terminal basis,
and require the completion transaction's actual parent to match. Do not guess
a current head or alter ordinary Memory publication without a demonstrated
causal need. Include genuine all-rows-flushed and still-pending multi-batch
cases in the filesystem proof. No activation-only-tail support is claimed.

At the selected World basis, require exactly one completion entity. Admit its
format/binding and both its creation and terminal-bootstrap immutable Native receipts;
the completion transaction's sole parent must be that terminal bootstrap commit on
the expected branch. Read actual program activation at the SAME selected
basis, admit its revision and install its real entity identity in the fresh
activation cell. Then construct the Query publication with the NEW installed
binding identity, original terminal bootstrap commit and descriptive aggregate counts.
Do not use checkpoint `recover_publication`, whose one-commit census cannot
recover multi-batch aggregate publication. No index registration or bootstrap
publication occurs on reopen.

Before service, read the actual latest-Native Query idempotency rows with the
existing bounded entity scan and cumulative original request. Validate exact
field/intent shape, unique keys, unique nonzero outcome identities, checked
effect counts and actual immutable creation receipts. Extend the existing
`application_attempt/idempotency/recorded_intent.rs` owner decoder rather than
add another grammar. Restore each actual outcome through the existing identity
frontier operation before any new outcome may be minted. Zero and MAX refuse;
MAX-1 leaves the next mint honestly exhausted. A partially advanced allocator
frontier grants no service. This is a bounded full recovery scan, with typed
owned exhaustion, not resumable paging. Leave completed receipt caches empty;
an old exact retry may return CommittedReceiptNotRetained without handler replay.

Also scan and refuse persisted unsupported Native families: actual outbox,
inbound completion and aftermath entity kinds, and all workflow entity/relation
kinds. Derive the thirteen workflow entity and twenty-seven relation kinds
from exhaustive actual `WorkflowLayoutFields` and nested output/input layouts,
without `..`, offsets, name matching or a second registry. This includes
persisted workflow definition rows because their publication lifecycle has not
been readmitted. Compiled workflow declarations remain allowed. Existing live
outbox/aftermath support is unchanged; this is a stricter cold-readmission
posture. Ordinary application principal/capability truth, activation,
completion and admitted idempotency rows remain supported.

Destination topology, extending existing owners:

```text
primary_graph/schema_layout/bootstrap_publication.rs [create: completion schema]
primary_graph/bootstrap/home_publication/
  record.rs       [create: descriptive stored record]
  codec.rs        [create: closed portable binding grammar]
  publication.rs  [create: fresh ordinary Native completion commit]
  readmission.rs  [create: selected-basis record/activation admission]
primary_graph/application_installation/home_preparation/
  native.rs, source.rs [revise: fresh-only construction]
  existing.rs         [create: existing-only concrete carriers/orchestration]
primary_graph/workflow/schema/home_readmission.rs [create: owner-derived kind inventory]
worth-query-host/src/application_home/opening/
  fresh.rs       [move: same fresh recipe]
  existing.rs    [create: actual Store-origin existing recipe]
  service.rs     [move: common real custody/transition completion]
```

Keep latest-Native idempotency admission under its existing Query attempt
owner; do not mix it into the selected-basis completion codec. Add semantic
children there as needed to preserve the 400-line limit. Commit known
successor recovery populations under their actual Query owners, without
creating empty placeholders now.

Host branches only on actual Store origin/refusal and consumes its same Home,
incarnation and lease. ProvenAbsent keeps the one fresh opportunity;
ExistingArtifacts invokes the existing-only recipe; consumed initialization
remains refused. Extend the existing owning refusal and failure-close routes
for each real preparation state. Common completion requires zero actual World
custody and consumes the Store no-pending carrier into the concrete existing
CustodyReadmitted/TransitionSettled progression before exposing public service.
Seal new intermediate factories to exact Host orchestration paths and named
certification controls; ordinary applications still import only Decl and Host.

The external Decl/Host court and existing application_home testkit must prove:
real multi-batch seed/commit/close/same-directory reopen/read/new commit; no
initializer or supplier replay; separate writer/reader processes and restored
identity frontiers; old-key exact retry with unchanged value and truthful
receipt-not-retained refusal; unchanged meaning with changed build UUID;
owned missing/unsupported completion, foreign binding, missing recovered index,
activation mismatch, original-request stop/exhaustion and retained-World
refusals. Produce actual persisted rows for each refused family. Require
mutation-sensitive observations for both selected-basis attachment consumers
and the D17 scan. No process-local cache or fabricated authority can stand in
for recovery. Keep all failure logs; run scoped courts, formatter, line caps,
boundary/context and affected compilation once the coherent patch is ready.
Update `docs/build-an-application.md`, `docs/capability-index.md` and the Host
README with existing-open semantics, ownership on refusal, stricter persisted
family posture and receipt limitation, checking examples against real exports.

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
   The stream publishes its own positioned subject through the installed
   medium and original allocation policy; it accepts no callback that could
   substitute another committed subject for the prepared route.
4. Thread positioned branch transitions through live publication and consume
   their proof at append/cutover. Keep explicit memory-only and replay paths,
   the existing reservation, canonical ordering and pre-effect checks.
   Seal the step to its live and fidelity-checked replay implementations;
   publication variants stay private to that owner. The concurrency fixture
   parks the real live step through its owned unit-test control.
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
tests. Ordinary focused process correctness runs may overlap under Root
coordination. Controlled timing comparisons require an exclusive agreed window.

### Next ownership transfer and reserved handoff

The next concrete batch is specified in
[Application home ownership](milestone-3-application-home.md). Its shared
identity, issuer, frame and recovery contracts are in
[Durable owner contracts](milestone-3-durable-owner-contracts.md).

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
