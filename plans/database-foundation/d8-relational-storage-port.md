# D.8 Two-tier Relational Storage Port

The design note for [D.8](roadmap.md#d8-two-tier-relational-storage-port). The roadmap's
Standing Rules, Warm And Cold Data, D.5, D.9, and D.10 govern this note. D.8 establishes the
Relational contract and the final resident backend; D.9 supplies the Store binding. D.8 does
not claim disk persistence or a larger-than-memory physical backend.

Source references use `R/` = `crates/worth-relational/src/`, `B/` =
`crates/worth-runtime-bridge/src/`, `W/` = `crates/worth-runtime-world/src/`, `Q/` =
`workspaces/worth-query/crates/`, and `E/` =
`Q/worth-query-execution/src/domain_computation/primary_graph/`. Every reference below expands
to a repository `path:line`. New names and paths are design decisions, not claims that they
already exist. Source inventory checked on 2026-10-06. Cargo execution and command durations
are unverified: this note's authoring brief forbids running Cargo.

## 1. Decisions

### Placement and warmth

**A placement group is the existing `PartitionId`; warmth is a bounded head-column chunk
within that group.** Do not introduce a second spelling of partition identity. An existing
containing entity selects its partition before identity allocation. A creation with no
containing entity uses the creation partition issued for its admitted application attempt.
Direct Relational callers continue to declare the partition in their creation intent.
Placement is sealed during lowering, before slot reservation; subsequent attention, eviction,
relation changes, and branch moves never rewrite record IDs. Relations retain the partition
chosen by their creation intent; adjacency entries may name endpoints in other partitions.
Placement grants no scope or mutation authority.

This retains the real Query policy: `create_entity` selects `Issued`, and
`create_entity_in_context` selects an existing context's partition
(`E/application_attempt/effect_program/entity_creation.rs:21,32,47`). The `main()` at line 95
is a candidate reference before remapping, not proof of final placement. Provider lowering
resolves it against the issued partition (`E/application_attempt/provider_binding/
effect_accumulator.rs:43`, `E/application_attempt/provider_binding/effect_lowering.rs:106`); issuance is checked increment of a host
counter (`E/application_runtime.rs:228`). Reject exhaustion; do not hash a containing owner
into a possibly colliding partition. D.10 persists this issuer under D.2's reopen inventory;
D.8 does not invent another issuer. The single-candidate placement restriction at
`E/application_attempt/effect_program/entity_creation.rs:64` remains; relaxing it is a separate product change.

The warm unit is `(exact root generation, PartitionId, entity-or-relation domain, slot range,
column selection)`. A native chunk has a declared byte ceiling and slot ceiling in Relational's
residency configuration. Variable aspect columns and bulk values have independently bounded
chunks; membership, generation, lifecycle, kind, and revision columns identify their records.
No unit contains metadata history. The configuration is explicit, with checked nonzero bounds,
and never larger than the residency budget. This extends the current SoA columns
(`R/storage/substrate/record_arena/arena/mod.rs:38`) rather than treating an arena or partition
as a pin. The AoSoA widths at `R/authority/commit/phases/prepare.rs:34` are candidate layout
choices, not a resident memory contract.

**An oversized group remains one stable placement group and is worked on chunk by chunk.**
Evict unguarded, least recently demanded chunks before admitting a new chunk. Reserve bytes
before allocation, counting retained native data, in-flight fills, and concurrent guards
together. Guarded chunks are never evicted. If the requested simultaneous native working set
does not fit, return `ResidencyBudgetExceeded` before filling; ordinary cursor reads remain
available. A value too large for one native chunk is streamed as bounded column fragments;
a caller requiring it contiguous must admit that whole allocation or receive a denial.
Never split identities, exceed the budget, pin an entire group, or promote history to solve
pressure. D.10 maps output-demand sources onto these units and prioritizes their attention.
An open branch is eligible for warmth; opening a branch cannot pin its entire head.

**Warm state is derived.** A warm chunk carries the exact immutable target and root generation
it reflects. The committed semantic batch supplies its update. Apply only to the matching
predecessor stamp; otherwise discard and rewarm from the selected successor. A retained old
root and a current root may have different chunks; a branch name alone is never a cache key.
Deleting a branch does not erase an older leased root. This preserves exact-basis ownership
in `R/branch/root_selection.rs:43` and branch registry ownership in `R/branch/registry.rs:9`.

### Roots, commitments, and history

**A root region holds `RelationalStorageRoot` plus its partition selector and summaries.**
It holds no partition payload or `Arc<PartitionState>`. The handle is an owner-admitted exact
immutable target with backend custody; it is not a page ID, digest, mutable branch selector,
or native residency promise. A retained handle preserves availability, not memory residency
or currentness. D.9 attaches its backend custody to a leased tree root and family set without
changing the Relational signature. The branch cell remains the sole currentness owner.

These are all explicit `Arc<PartitionState>` holdings and escape signatures in current
Relational source:

| Current holder or escape | Change |
|---|---|
| `R/branch/root_region.rs:13`; construction at 83 | Replace the payload with the port root handle. |
| `R/runtime/state/subsystems/storage/partition_edition.rs:11,32` (`PartitionMap`, held through `Arc<PartitionMap>`) | Move resident authority inside the backend; runtime holds a port and selected target. |
| `R/runtime/state/subsystems/storage/partition_edition.rs:52` (`shared_partition`) | Delete the Arc escape; acquire a guard or cursor. |
| `R/runtime/state/subsystems/storage/mod.rs:52` (`partition`) | Delete this Arc-returning surface. |
| `R/runtime/state/subsystems/storage/edition_writer.rs:98,124` (`remove`, `own_partition`) | Backend-only mutation mechanics; no caller owns a whole partition. |
| `R/inspection/access/storage.rs:16` (`current_partition_state`) | Replace with fallible bounded inspection cursor. |

Indirect root custody also retains that Arc today: `R/branch/root_regions.rs:38` stores an
`Arc<RelationalRootRegion>`; `R/branch/root.rs:128,141,145` holds regions, admitted root state,
and prepared capture. Root access at `R/branch/root.rs:301` lends the partition. Capture at
`R/branch/root_capture.rs:109`, allocation accounting at `R/branch/root_region.rs:51,108`,
and slot totals at `R/branch/root_capture_preparation.rs:128` dereference it. Convert these to
batch-carried summaries and backend allocation evidence. `Arc<RelationalBranchRoot>` remains
lawful immutable metadata custody. Do not replace it with a stronger physical pin.

**Persist the incremental commitment, including its update structure, beside
`partition_image_digest`.** `DurableBranchRootImage` currently carries the image digest but
not the accelerator (`R/durability/data/checkpoint_images.rs:135,144`). Store a versioned,
canonically ordered per-partition commitment image containing record/history digest nodes,
adjacency digest nodes, counts, and update roots. A single final hash cannot support arbitrary
incremental replacement. Bind this image into `root_image_digest`; validate both its grammar
and correspondence to authoritative images on native checkpoint readmission. Bump the current
format version (`R/durability/data/checkpoint_images.rs:152`), and refuse unsupported images before effects.
Change capture (`R/durability/authority/checkpoint_image.rs:131`), alias serialization
(`R/durability/log/persisted_checkpoint/partition_aliases.rs:114`), and restore
(`R/durability/authority/runtime_rebuild/checkpoint_restore/branch_root_images.rs:45`).

Replace the previous `&PartitionState` requirement in
`R/storage/overlay/partition_content.rs:33` and `R/branch/root_region.rs:35` with sealed batch
changes and persisted digest-node reads. Each changed record supplies its before/after
digest contribution, lifecycle delta, and allocation delta; histories append or retire
bounded version entries. Update only the affected commitment paths. Preparing may read a
touched old value through the port; publishing never reacquires the previous partition.
The digest never becomes admission or currentness authority. Independent full recomputation
remains a certification/verification lane, as `R/branch/root_axes.rs:14` already distinguishes.
D.9 persists commitment nodes with their semantic batch, not a heap-sized accelerator rebuilt
at every open.

**Split head columns from version storage.** Current `RecordArena` owns
`metadata_history: SharedColumn<SharedColumn<K::Meta>>`
(`R/storage/substrate/record_arena/arena/mod.rs:45`); checkpoint images repeat it
(`R/durability/data/checkpoint_images.rs:77`). The backend's head contains the selected record's
current payload, lifecycle, generation, kind, endpoints, and aspect/field revisions. Separate
version storage holds effective/retired intervals and previous aspect payloads. Both belong
to one immutable root and one semantic batch. A warm head never loads the version family.
The final in-memory backend stores both in resident persistent structures, but its version
reads use the cold interface and never create warm arenas.

Exactly these reads become cold reads:

- Entity and relation version selection in `R/visibility/materialization/read_records/reader/
  truth_record_access.rs:80,129`, historical kind scans in `R/visibility/materialization/read_records/reader/truth_kind_scan.rs:94` and
  `R/visibility/materialization/read_records/reader/truth_relation_kind_scan.rs:80`, and historical slot visibility in
  `R/visibility/materialization/read_records/visibility/slot_sets.rs:37,80` use version/kind cursors.
- Historical projection storage (`R/visibility/materialization/read_records/projection/historical_basis_reads.rs:28`), historical
  minimum counts (`R/validation/engine/evaluator/relation_cardinality/minimum_visible_counts.rs:115`),
  and historical traversal enumeration (`R/validation/engine/evaluator/relation_traversal/visible_entities.rs:29`) use
  the exact old root and bounded cursors, never the latest runtime edition.
- Relation dependency capture currently opens metadata history
  (`R/authorization/durable_dependencies/capture.rs:181`); it reads a bounded version/revision
  cursor. Head aspect and field revision probes can remain head reads.
- Every unguarded head point, adjacency range, kind range, wide record scan, checkpoint export,
  and verification walk uses the cold port. A retained old *exact root* can answer its head
  directly through that root's record cursor; it need not reconstruct all intervening versions.
  History means a version request, not merely an old lease.

### Prepare's complete storage read set

**Compile the read shape and ceilings before acquiring data.** `AllObserved` is an observation
scope, not permission to load every partition. `FullObservedScan` is currently declared for
derived-index packets (`R/indexes/authority/packet_planning.rs:79`); invariant packets use
`AllObserved` when their partition list is empty while declaring `SharedCommittedRead`
(`R/validation/execution/planning/execution_plan.rs:183`). Neither spelling proves locality.
Resolve both into explicit indexed ranges or deny an ordinary full-state scan.

Every class below has both a structural bound and an admitted finite work ceiling. Let `L`
be the backend-reported maximum navigation steps for the selected family, `T` the number of
sealed touched/read loci, `F` the affected unique locators, `A` the admitted adjacency
candidates, `K` the admitted matching kind entries, `P` the admitted partition descriptors,
and `V` the admitted version entries. Point costs include navigation plus decoded bytes;
an empty/missing lookup still charges navigation. Limits for work, bytes, candidates,
scratch, results, deadline, and cancellation live in the lowered read plan. The effective
ceiling for each class is its structural expression capped by the remaining operation
resource lease; there is no `u64::MAX` ordinary default. An unknown scan extent is charged
before each step and stops before step `ceiling + 1`. Parallel packets share the aggregate
meter, not a fresh budget per packet. Existing meter ownership is
`R/validation/custom_rule/work_meter.rs:19,29,230`.

| Prepare read class and current owner | Port and work ceiling |
|---|---|
| Branch locality, live/stale targets, kind and current aspects: `R/authority/commit/phases/prepare.rs:172`; `R/authority/intent_merge/record_lookup.rs:18`; `R/authority/mutation/stale_targets.rs:30`; mutation intents listed in section 4 | Guarded head probes or point cursors, at most `T * (L + 1)` record probes plus admitted payload bytes; deduplicate repeated loci. |
| Footprint derivation, touched scope, endpoint expansion: `R/mvcc/validation/proposal_touches.rs:339,354`; `R/authority/commit/touched_scope.rs:25,64`; `R/validation/engine/request/relation_integrity_scopes/incidence_collection.rs:23,37` | Point plus outgoing/incoming adjacency cursors, `T * (L + 1) + 2*T*L + A*(L + 1)` ceiling; separate relation-integrity candidate cap. Never collect an uncharged degree-sized vector. |
| Existing relation identity and duplicate edges: `R/authority/intent_merge/relation_validation/relation_identity_scan.rs:19,31,75` | Endpoint/kind adjacency ranges plus relation points; `2*T*L + A*(L + 1)`, with smallest available admitted direction. Same-proposal duplicate checks consume the lowered batch. |
| Working-set selection and copies: `R/authority/commit/phases/prepare.rs:271,282`; `R/storage/overlay/working_state/construction.rs:79` | Root summaries for counts; acquire only touched chunks/loci. `P*(L + 1) + T*(L + 1)` plus explicit chunk bytes. Delete full-partition clone fallback, including merge mode. |
| Unique fields: `R/validation/engine/evaluator/unique_entity_fields.rs:30` | Authoritative unique lookups, at most `2*T*F` old/new key probes, each `L + 1`; delta reduction and payload bytes separately charged. Zero unrelated record scans. |
| Kind membership for minimums/traversal and query preparation: `R/validation/engine/evaluator/relation_cardinality/minimum_current_index.rs:24`; `R/validation/engine/evaluator/relation_traversal/visible_entities.rs:29`; `R/visibility/materialization/read_records/reader/truth_kind_scan.rs:99` | Kind-index ranges, `L + K*(L + 1)`; only matching entries and needed adjacency. Delete construction of a whole-world `CurrentVersionMinimumIndex`. |
| `AllObserved` invariant observation without a journal: `R/validation/execution/planning/execution_plan.rs:183` | Exact-root summaries or indexed ranges specified in the next table; descriptor enumeration is capped at `P*(L + 1)`. Empty touched scope never silently grants a broad scan. |
| `FullObservedScan` derived-index preparation: `R/indexes/authority/packet_planning.rs:81` | Explicit bounded record/kind/adjacency cursor build, at most admitted `K`, `A`, and bytes. This is index construction, not mandatory ordinary prepare work. An ordinary freshness requirement without an admitted bounded strategy is denied. No hidden rebuild on open. |
| Custom invariant applicability and state overlay: `R/validation/execution/planning/custom_applicability.rs:46`; `R/validation/engine/state_view.rs:102,297`; `R/validation/engine/state_view/slot_resolution.rs:25` | Journal-first overlay plus selected-base point cursors, `2*T*(L + 1)` and declared custom traversal cap. Missing untouched overlay entries resolve against the base record, not the base partition. |
| Lineage, descriptive touches, index effects: `R/lineage/authority/commit_finalization.rs:253,291,315`; `R/authority/commit/pipeline/artifact_execution/descriptive_touches/merge.rs:34`; `R/authority/commit/pipeline/artifact_execution/descriptive_touches/index.rs:189` | Journal point reads and carried before/after projections; `2*T*(L + 1)`, with replace-target candidates bounded by the changed-record set. No whole selected state. |
| Schema, symbols, identity reservations, configuration, history parents: `R/authority/commit/phases/prepare.rs:174,220`; `R/branch/root_capture_preparation.rs:158` | Existing owner metadata handles and admission budgets, not warm record reads. Charge symbol/schema probes and parent count; D.9 persists these owners but they do not authorize record residency. |
| Root construction and commitment: `R/branch/root_capture.rs:104`; `R/storage/overlay/partition_content.rs:39` | Semantic batch plus digest-node cursor, bounded by changed digest leaves and their navigation paths; version work `V*(L + 1)`. Carry totals; publication performs zero partition reads. |

The native invariant set is exhaustively dispatched at
`R/validation/engine/evaluator/mod.rs:26`. Its full-scan cases are decided here:

| Invariant | Ordinary decision |
|---|---|
| `MaxMergedIntents`, `RelationIntegrityScopeBudget` | Lowered batch length/scope admission; no storage scan (`R/validation/engine/evaluator/mod.rs:36,61`). |
| `LiveRecordRequiresSidecar` | Validate every new/changed head before batch sealing. Carry per-root missing-sidecar counts for no-journal whole-observation checks; zero count is a complete answer. Delete fallback scanning at `R/validation/engine/evaluator/record_surface_rules/live_record_sidecars.rs:123`. Historical audit uses indexed version cursors with its own ceiling. |
| `MaxSnapshotEntities` | Carry authoritative live-entity count in root summaries, updated by lifecycle deltas in the same batch. Historical exact-root count comes from that root; a non-root version request uses bounded version visibility counting or is denied. Delete current live-bitset and historical whole-partition walks at `R/validation/engine/evaluator/record_surface_rules/snapshot_entity_limits.rs:21,43`. |
| `UniqueEntityAspectField` | Root-qualified authoritative unique index; no full scan. Initial schema introduction validates/builds it as an admitted schema transition, not an implicit ordinary commit rebuild. |
| `EndpointKindContract`, `PartitionIsolationContract` | Touched relation/endpoints, point reads; no global scan. Empty-journal audit uses the matching relation-kind index under the audit cap. |
| `CardinalityMaximumContract`, `UniquenessContract`, `SymmetryContract`, `EndpointDeletionIntegrityContract` | Indexed outgoing/incoming endpoint-kind ranges plus changed relations; charge every examined membership and endpoint point read. No partition scan. |
| `CardinalityMinimumContract` | Matching entity-kind index plus endpoint-kind adjacency/count summaries; include candidate entities newly made eligible by the proposal. A no-journal global check streams matching kinds under `K` and `A` ceilings. Replace full-world current/historical count builders (`R/validation/engine/evaluator/relation_cardinality/minimum_current_index.rs:24`, `R/validation/engine/evaluator/relation_cardinality/minimum_visible_counts.rs:115`). |
| `AcyclicityContract`, `ConnectivityMinimumContract` | Kind-index enumeration and both-direction adjacency cursors bound the complete declared graph. Admit traversal vertex/edge/scratch limits; deny when the graph needed for a verdict exceeds them. Partial traversal never certifies pass. An index narrows candidates; it is not a proof of acyclicity or connectivity. |
| Custom rules | Require a declared bounded point/index/traversal read contract. Deny an unindexed `FullObservedScan` on the ordinary lane before executing its kernel. Explicit certification scans remain bounded, fallible cursors. |

### Unique-field authority

**The unique-field index belongs to the immutable selected root and its semantic batch.**
Keys retain `AspectFieldLocator` and `AuthoritativeFieldComparisonKey` semantics
(`R/runtime/state/subsystems/indexing.rs:23`; `R/storage/data/
authoritative_field_comparison_key.rs:1`). All kinds and partitions governed by that locator
share its root-local uniqueness scope. Remove old keys, overlay all proposal deletes/updates,
then check all proposed new keys together, including colliding creates and legal swaps.
Duplicate checking cannot consult a sibling branch or discard same-batch collisions.

The constructor that seals record changes also seals unique, kind, adjacency, live-count,
sidecar-count, version, and commitment changes. The backend installs one successor or none.
The root cannot be exposed before any correctness index. Persist the unique index in native
checkpoint images; restore it as authority, never rebuild it at open. An absent or inconsistent
required index is a typed damaged-state denial. Remove the runtime-global
`entity_unique_aspect_field_index` at `R/runtime/state/subsystems/indexing.rs:38` and its
refresh/rebuild path at `R/indexes/unique_entity_aspect_field_index.rs:12,35,61`. Other
`IndexingState` generations remain derived unless D.9 explicitly changes their contract.

## 2. What the roadmap gets wrong or leaves unspecified

- Its historical premise that Query creates almost everything in `main()` is stale. Issued
  partitions and context placement are real production paths (section 1). Existing main-heavy
  data still needs chunk residency, so the larger-than-memory requirement is unchanged.
- An incremental commitment already exists, but it is destroyable in-memory acceleration and
  reads the previous partition (`R/storage/overlay/partition_content.rs:20,33`). Persisting only
  its final digest would not remove that dependency. Persist its update nodes and carry deltas.
- An existing unique index is not authoritative or branch-qualified; the evaluator deliberately
  scans selected state (`R/validation/engine/evaluator/unique_entity_fields.rs:17`). Promotion
  requires batch/checkpoint/schema-transition changes, not moving the existing map behind a trait.
- D.8's allowance for changed counts names publication and region capture, but uniqueness has
  explicit scan counts of 2/2/4 (`R/tests/complexity/contracts/commit_budgets/
  unique_invariant_lookup.rs:41,87,111`). Those must also be re-declared. Preserving them would
  preserve the scan D.8 requires removing.
- `get_partition` is already crate-visible rather than a consumer API
  (`R/storage/overlay/access.rs:7`). Private *to the backend*, plus removal of alternate
  `partition_state`, `partition`, `base_partition`, and arena escapes, is stronger than merely
  changing it to `pub(crate)`.
- "The current head of open branches is warm" cannot mean the whole head is resident: one
  group or many branches can exceed the budget. Eligibility and demand are granular; denied
  native acquisition remains honest while cold reads continue.
- D.9 assigns kind-index implementation and deletion of linear scans to itself. D.8 already
  needs a real kind-index contract and the resident backend's matching index to bound prepare;
  D.9 implements its physical encoding. It does not redesign the port.
- "Every read" includes inspection, checkpoint export, historical validation, native revision
  probes, allocation evidence, and borrowed callbacks, not only application record getters.
  Root retention currently promises resident immutable payloads
  (`crates/worth-relational/OWNER_COMPONENT_PORT.md:44`); D.8 revises that promise to leased
  availability while preserving exact targets, release terminality, and post-seal frozen reads.

These corrections belong in D.8 implementation's roadmap/documentation slice. This authoring
task changes only this note. No other roadmap file is edited here.

## 3. Port, topology, bug classes, and contracts

The stable facade is `worth_relational::facade::storage`. Only runtime composition and the
binding implementation need it; application declarations do not. Read sessions are selected
by an owner-admitted basis or observation. No method accepts a raw digest or ambient main
branch to acquire authority. `RelationalStorageRoot` owns exact-root custody. Sessions may
survive pauses with that lease and a typed resume key; they hold no resident pin.

The interface has these operation families, all in Relational vocabulary:

| Family | Contract |
|---|---|
| Residency | `with_warm(root, request, limits, for<'guard> FnOnce(&'guard RelationalResidencyGuard) -> Result<T, RelationalStorageDenial>) -> Result<T, RelationalStorageDenial>`. Reserve first. The guard exposes only head columns and is non-Clone, non-Send, and non-Sync. |
| Records | Point and ordered record-range sessions over entity/relation identities, with exact-root lifecycle and generation checks. `Result<Option<OwnedRecord>, RelationalStorageDenial>` separates absence from denial. |
| Versions | Record/version interval selection and resumable version ranges; no warming method. Owned bounded values or callback-scoped fragments. |
| Adjacency | Direction, endpoint, relation kind, and ordered other-endpoint ranges; both directions, structural revisions, and exact candidate costs. |
| Kinds | Entity/relation kind membership ranges and root counts; matching entries rather than linear record filtering. |
| Correctness | Unique key probes and authoritative summaries/commitment-node reads; selected root plus proposal overlay. |
| Mutation | An owner-authorized semantic batch prepares one immutable successor on the admitted predecessor. Publication consumes the existing prepared candidate and preserves its typed movement/settlement protocol. Fork shares custody; deletion releases a named reference, not leased state. |
| Export/verification | Explicit bounded reconstructive sessions, streaming image entries and independent commitment verification; no ordinary caller can request a whole `PartitionState`. |

The public backend contract is object-safe and exposes owned batches plus bounded read steps;
the facade/session owner implements higher-rank callbacks above it. D.9 can implement the
backend in another crate without constructing Relational branch authority. Backend root
custody is opaque mechanism data; only the Relational owner admits it into
`RelationalStorageRoot` for the requested target. Backend replacement is installation-time
composition, not a runtime fallback. No Store type appears in these signatures.

`RelationalStorageDenial` is an owned, typed, non-exhaustive cause with operation, family,
exact target, progress/work/bytes, and recovery posture. Its named causes are
`OwnerUnavailable`, `RootUnavailable`, `ResidencyBudgetExceeded`, `WorkBudgetExceeded`,
`ResultBudgetExceeded`, `Cancelled`, `DeadlineElapsed`, `StorageUnavailable`,
`DamagedStorage`, and `UnsupportedFormat`. It exposes no raw Store error. An expected absent
record is `Ok(None)`; exhaustion or damage can never become absence or a passing invariant.
Storage failure during preparation carries no movement. Existing publication and settlement
outcomes continue to distinguish performed work and repair; a failure after movement cannot
be relabeled a pre-effect read denial.

Add `Storage(RelationalStorageDenial)` to the existing read-denial families, including
`R/change_source/observation_read.rs:6`, borrowed record denial
(`R/visibility/materialization/read_records/projection/borrowed_records.rs:9`), and Query's
read/attempt denials. Fallible traversal callbacks distinguish callback denial from storage
denial. Bridge carries the typed cause in `BridgeSnapshotReadError`, whose existing fields
currently omit it (`B/snapshot/read_error.rs:8,22`). Replace the "foreign observation" string
translation (`B/relational_source/snapshot_reading.rs:161` and
`B/relational_source/runtime_source/retained_entity_projection.rs:53`) with an exhaustive typed mapping.
World's composite admission already carries Relational denial
(`W/basis/admission.rs:98,153`); preserve that cause through preparation and recovery.
Query's `E/application_query/read_execution/root_selection.rs:229`,
`E/application_query/read_execution/tree_materialization.rs:96,231`, `E/application_query/live/scope_identity.rs:59`, and
`E/application_query/resource_lifecycle/basis_registry/lease.rs:233` must propagate denial rather than infer missing
data. No `expect`, `unwrap`, ignored error, empty vector, or retry loop handles storage denial.

**Pause discipline matches D.5.** Acquisition may fault or wait only while holding no previous
native guard. A cursor step obtains its bounded data, charges it, returns an owned page of
results and a resume key, then drops all guards before yield, I/O, cancellation, or callback
handoff. Resume keys name canonical record/version/adjacency/kind keys and an exact root,
never a mutable ordinal. Warm projection callbacks are synchronous higher-rank closures;
`T` cannot contain their borrowed view. Returned futures cannot borrow the guard. Non-Send
alone does not prevent a same-thread future from suspending: expose no async guarded method,
and enforce a no-suspension/no-blocking-call rule in guarded callback bodies and backend
guard scopes with an automated source/AST check, alongside compile-fail escape/Send tests.
Multi-family reads drop one step's guard before faulting the next family. The caller can
carry an owned result across a pause only after charging its retained bytes.

### Destination tree

`storage/` is the Relational storage owner; its children separate contracts, native residency,
semantic batch formation, and backend mechanism. Facade files only export. Proposed tree:

```text
crates/worth-relational/src/
  facade/storage.rs                         created: stable internal audience facade
  storage/
    port/                                  created: stable domain contract axis
      mod.rs                               exports only
      root.rs                              exact-root custody, admission
      denial.rs                            typed causes and progress
      records.rs                           point/range sessions
      versions.rs                          version sessions
      adjacency.rs                         direction/range sessions
      kinds.rs                             membership sessions
      correctness.rs                       unique, summaries, commitment reads
      export.rs                            bounded checkpoint/verification session
    residency/                             created: derived native lifecycle owner
      admission.rs                         reserve/evict/fill orchestration
      guard.rs                             scope, release, linear custody
      head_columns.rs                      head-only entity/relation columns
      demand.rs                            unit attention and priority
    batch/                                 created: semantic write contract owner
      construction.rs                      records and indexes sealed together
      unique_fields.rs                     old/new key delta and conflict admission
      kind_membership.rs                   kind delta
      root_summaries.rs                     lifecycle/sidecar count delta
      commitment.rs                        changed contributions and digest paths
    backend/                               created: replaceable representation axis
      mod.rs                               backend contract exports only
      memory/                              moved/restructured resident representation
        roots.rs                           immutable record roots, sharing
        partitions.rs                      private get_partition mechanics
        records.rs                         authoritative head storage
        versions.rs                        separate authoritative version storage
        adjacency.rs                       both directions
        kinds.rs                           root-local membership
        unique_fields.rs                   root-local unique authority
        commitments.rs                     persisted incremental update structure
        checkpoint.rs                      native image import/export
    overlay/working_state/                 existing, changed: bounded journal overlay
  branch/root_region.rs                     replaced: handle plus summaries
  runtime/state/subsystems/storage/          replaced: installed port, no raw payload escapes
  visibility/...                            existing, converted by module below
crates/<D.9 binding crate>/                  committed successor; no placeholder in D.8
  src/storage_binding/                      implements backend over physical facade
    roots.rs; records.rs; versions.rs; adjacency.rs; kinds.rs; unique_fields.rs
```

Keep pure reusable identity/value vocabulary in `worth-foundational`; legal authority in
`worth-proof`; guards, counters, tables, custody, and Drop in Relational. Its manifest already
depends on both substrate crates (`crates/worth-relational/Cargo.toml:21`). No new one-site
authority is needed. The memory backend is the final standalone resident backend and remains
available for Relational parity/certification. D.10 removes the old application runtime
composition path, not this backend contract. No temporary backend, adapter to the old public
getters, second codec, or second mutable map is added.

### Recurring bug classes

| Class | Contract and enforcement owner |
|---|---|
| Raw read bypass | Backend-private representation and `get_partition`; remove `PartitionAccess` export, enforce facade visibility and boundary-check source rules. Also reject the alternate raw escapes listed above. |
| Forged root or mutation authority | Existing concrete branch authority (`R/branch/authority.rs:4,10,14`) uses `worth-proof` sealed markers (`crates/worth-proof/src/proof/marker_authoring.rs:45`). Root bytes/digests never mint a witness. No generic `AuthorityMarker` admission. |
| Guard leak, double release, budget drift | A runtime-owned residency obligation uses `LinearResource` internally, with owner release on Drop and explicit terminal consumption (`crates/worth-proof/src/linear.rs:52,75,91,106`). No live registry in the proof crate; Relational owns budget and custody. |
| Borrow or pin crossing a pause | Higher-rank callback lifetime, private guard constructor, non-Send/non-Sync guard, pause-scope enforcement described above. A session retains only a root lease and key. |
| Cold read promoting/materializing | Version/cold sessions have no residency capability; bounded result/fragment types and memory admission before allocation. Delete wide `Vec` and full-bitset materialization paths. |
| Denial erased as missing data or success | `Result` at every storage crossing; aggregate operation preserves the cause. Use `TransitionOutcome` for multi-arm admission/lifecycle transitions (`crates/worth-proof/src/transition/outcomes.rs:21`), not a bool or string. Its `unwrap` at line 89 is not permitted on this lane. |
| Published records without index truth | One sealed semantic batch; immutable successor includes every correctness family. Existing prepared candidate is the consumed phase artifact; no independently installed unique map. |
| Prepared or denied work mistaken for movement | Preserve `PerformedRelationalCommit`, already backed by `Performed` (`R/mvcc/publication/outcome.rs:43,101`; `crates/worth-proof/src/effect/performed.rs:46,110`). Record only after owner movement; warm application consumes the exact committed batch. |
| Old warm data served for a new root | Exact target/generation stamp and batch predecessor check, not branch-name or hash equality; mismatched copy is dropped. |
| Oversized demand or broad invariant tax | Chunk reservation and aggregate work/scratch ceilings; fail before effects. Indexed global invariants and denial on incomplete traversal preserve correctness. |
| Cold commitment triggering full reconstruction | Persist update nodes and carry before/after contributions; publication interface receives a sealed batch and root metadata, with no record-read callback. |

## 4. Complete `get_partition` conversion inventory

There are **98 call expressions in 60 files**, of which 95 are production/support expressions
and three are in named test files. There are **13 definitions in 8 files**, including three
test definitions; definitions are not call expressions. A search of all `.rs` files in
Relational, Bridge, World, and `Q/` finds **zero calls in the three consumer trees**. This
inventory uses the exact word `get_partition`; `target_partition` is unrelated. Each line
below is a call-site citation. Several calls have both head and historical branches.

Read shapes: **P** point (`with_warm` head guard or record cursor), **R** range (adjacency or
record range cursor), **H** history (version cursor), **S** wide scan (bounded record/kind
cursor or summary). The row states the replacement, not only today's superficial getter.

| Module under `R/` | Calls and source lines | Shape and replacement |
|---|---|---|
| `authority/commit/phases/prepare.rs` | 2: 271,282 | S -> root counts plus bounded touched-unit acquisition |
| `authority/commit/pipeline/artifact_execution/descriptive_touches/index.rs` | 2: 189,200 | P -> before/after record projection |
| `authority/commit/pipeline/artifact_execution/descriptive_touches/merge.rs` | 2: 34,35 | P/H -> journal record/version cursors |
| `authority/commit/touched_scope.rs` | 2: 25,64 | R/P -> adjacency range, endpoint point |
| `authority/intent_merge/record_lookup.rs` | 1: 18 | P -> live record point |
| `authority/intent_merge/relation_validation/relation_identity_scan.rs` | 3: 19,31,75 | R/P -> two-direction adjacency ranges and relation point |
| `authority/mutation/intents/apply_entity_aspect_patch.rs` | 1: 23 | P -> touched entity head guard |
| `authority/mutation/intents/apply_relation_aspect_patch.rs` | 1: 24 | P -> touched relation head guard |
| `authority/mutation/intents/update_entity_fields.rs` | 1: 24 | P -> touched entity head guard |
| `authority/mutation/intents/update_relation_endpoints.rs` | 1: 36 | P -> touched relation head guard |
| `authority/mutation/record_changes/relation_lifecycle.rs` | 3: 74,86,104 | P -> lifecycle/endpoints point |
| `authority/mutation/record_changes.rs` | 1: 139 | P -> attached relation kind point |
| `authority/mutation/stale_targets.rs` | 1: 30 | P -> identity/lifecycle point |
| `authorization/durable_dependencies/capture.rs` | 1: 174 | P/H -> relation version/revision cursor |
| `branch/root_selection.rs` | 1: 56 | P/R/H/S delegation -> selected root session; delete raw trait impl |
| `indexes/access/admitted_entity_field_lookup.rs` | 1: 268 | S -> bounded matching-kind/record cursor parity lane |
| `indexes/projected_field_values/index_projection_source.rs` | 1: 124 | P -> guarded kind/lifecycle projection |
| `lineage/authority/commit_finalization.rs` | 3: 253,291,315 | P -> changed-record lineage points |
| `merge/execution_mutation_plan/current_snapshots.rs` | 1: 9 | P -> admitted selected-base point, no ambient latest edition |
| `mvcc/validation/proposal_touches.rs` | 2: 339,354 | P -> kind/lifecycle points |
| `storage/authority/publication.rs` | 2: 74,80 | S -> sealed batch successor; delete whole-state projection |
| `storage/overlay/access.rs` | 3: 86,87,184 | P/R/H/S -> journal-first overlay record routing; delete raw base partition |
| `storage/overlay/working_state/construction.rs` | 1: 79 | S -> touched records/chunks; delete clone fallback |
| `storage/partition/adjacency_queries.rs` | 3: 17,28,39 | R -> outgoing/incoming/combined ranges |
| `validation/engine/evaluator/common/entity_kinds.rs` | 1: 22 | P -> endpoint kind point |
| `validation/engine/evaluator/record_surface_rules/live_record_sidecars.rs` | 1: 69 | P/S -> touched heads or root sidecar summary |
| `validation/engine/evaluator/record_surface_rules/snapshot_entity_limits.rs` | 2: 21,43 | S/H -> root count or bounded version visibility cursor |
| `validation/engine/evaluator/relation_cardinality/minimum_current_index.rs` | 1: 28 | S -> matching kind/adjacency index; delete global builder |
| `validation/engine/evaluator/relation_cardinality/minimum_visible_counts.rs` | 1: 115 | S/H -> historical kind/version/adjacency cursors |
| `validation/engine/evaluator/relation_traversal/visible_entities.rs` | 1: 29 | S/H -> matching kind cursor |
| `validation/engine/evaluator/unique_entity_fields.rs` | 1: 34 | S -> unique key probes, delete scan |
| `validation/engine/request/relation_integrity_scopes/incidence_collection.rs` | 2: 23,37 | R -> charged adjacency ranges |
| `validation/engine/state_view/slot_resolution.rs` | 3: 25,44,70 | P/H -> overlay/base record-version routing |
| `validation/engine/state_view/structural_adjacency.rs` | 2: 65,98 | R -> merged adjacency cursor and counts |
| `validation/engine/state_view/tests.rs` | 2: 143,181 | P/R/H/S -> test backend/session implementations |
| `validation/engine/state_view.rs` | 4: 102,179,271,297 | P/S -> touched record sources and counts |
| `validation/execution/planning/custom_applicability.rs` | 1: 46 | P -> changed old/new kind points |
| `visibility/materialization/read_records/projection/adjacency_revision.rs` | 1: 61 | P -> indexed adjacency revision probe |
| `.../projection/aspect_versions.rs` | 1: 19 | P -> head revision point |
| `.../projection/borrowed_adjacency.rs` | 2: 61,98 | R/P -> scoped adjacency cursor/revision |
| `.../projection/borrowed_records.rs` | 2: 75,121 | P -> guarded callback or owned point result |
| `.../projection/entity_adjacency.rs` | 1: 66 | R -> bounded adjacency cursor |
| `.../projection/entity_projection/tests.rs` | 1: 66 | P -> guarded test observation |
| `.../projection/entity_projection.rs` | 1: 22 | P -> guarded head projection |
| `.../projection/entity_retirement.rs` | 1: 39 | P/H -> exact-root lifecycle or version cursor |
| `.../projection/exact_basis_reads.rs` | 2: 115,128 | S -> bounded record ranges; remove unbounded Vec API |
| `.../projection/field_revisions/admitted_entity.rs` | 1: 36 | P -> admitted head revision point |
| `.../projection/field_revisions.rs` | 1: 42 | P -> relation head revision point |
| `.../projection/frontier_adjacency.rs` | 1: 101 | R -> bounded frontier adjacency cursor |
| `.../projection/historical_basis_reads.rs` | 1: 28 | H -> old-root version session; delete raw trait impl |
| `.../projection/streamed_basis_reads.rs` | 4: 22,58,124,132 | S/P/H -> record/version ranges and bounded size metadata |
| `.../reader/native_output_probe.rs` | 2: 28,60 | P -> exact head kind/aspect revision point |
| `.../reader/query_execution/leased_explicit.rs` | 1: 51 | P -> leased record point |
| `.../reader/query_fragment_work/packet_execution.rs` | 2: 190,214 | P -> packet-owned session point |
| `.../reader/query_plan_execution.rs` | 1: 384 | S -> partition summary cursor, no bitset walk |
| `.../reader/truth_kind_scan.rs` | 1: 94 | S/H -> entity kind cursor |
| `.../reader/truth_record_access.rs` | 4: 13,31,80,129 | P/H -> entity/relation point/version cursors |
| `.../reader/truth_relation_kind_scan.rs` | 1: 80 | S/H -> relation kind cursor |
| `.../visibility/slot_sets.rs` | 2: 37,80 | S/H -> visibility cursor; remove whole-sized bitsets |
| `visibility/snapshot_states/exact_state_building.rs` | 2: 47,58 | S -> lease root and select lazily; delete copied partition bitsets |

Here `.../` expands only to `visibility/materialization/read_records/`. Module totals,
including the three test calls, are: authority 21; authorization 1; branch 1; indexes 2;
lineage 3; merge 1; MVCC 2; storage 9; validation 22; visibility 36 = 98.
Read-shape totals are P 67, R 22, H 30, and S 36 call expressions. These overlap:
a delegated or conditional call can require multiple shapes, so they do not sum to 98.

Definition inventory: `R/storage/overlay/access.rs:8,51,84,209` (trait, map, overlay, test);
`R/storage/overlay/working_state/mutation_tracking.rs:65`;
`R/runtime/state/subsystems/storage/partition_edition.rs:104`;
`R/branch/root_partition_access.rs:6,20`; `R/branch/root_selection.rs:52`;
`R/validation/engine/state_view/structural_adjacency.rs:129`;
`R/validation/engine/state_view/tests.rs:142,180`; and
`R/visibility/materialization/read_records/projection/historical_basis_reads.rs:26`.
Delete these trait definitions/impls outside the backend. Any surviving memory lookup is
`pub(in crate::storage::backend::memory)` or narrower in `backend/memory/partitions.rs`.
Do not use `pub(crate)`. Architecture 1/15 and Domain Structure 8/9 govern; boundary-check's
new storage representation rule rejects raw partition/arena imports outside backend and
the scoped native residency implementation. Rust privacy is the primary enforcement.

The count is a conversion inventory, not the full escape inventory: `partition()`,
`shared_partition()`, `partition_state()`, `base_partition()`, root allocation visits, and
direct `metadata_history` access also require conversion. A completion search includes all
of them. Backend checkpoint/test mechanics may inspect representation; consumer tests use
the real facade. No test-only bypass provides application authority.

## 5. Slices

Each slice lands green with one implementer and a fresh independent reviewer. Keep each
touched code/test file at 400 lines or less, splitting by the tree's named responsibilities.
Do not solve the conversion with blanket error swallowing or compatibility getters. During
module conversion, unchanged sites keep their existing access to the **same** resident state;
new sites replace and delete their old read implementation in that slice. No old-to-new
delegating adapter or duplicate truth lane is introduced. The last conversion removes raw
visibility. Root representation cutover follows caller conversion so it needs no raw proxy.

Commands below are separate iteration steps, from the repository root. New filters are
required test families to add, not claims of existing tests. First list each filter and fail
on zero matches, then run it. Each step has a 300-second ceiling including build; durations
have not been measured here. If a target exceeds it, split its scenario family/compile target
before proceeding, preserving the full roster. Heavy scheduled lanes run once per slice,
also split into sub-five-minute groups; never substitute a no-op timeout for evidence.

For each slice with a boundary/visibility change also run these separately:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
```

Run `cargo fmt --all -- --check` for root changes and
`cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --all -- --check` for Query
changes. Generated `AGENT_CONTEXT.md` is regenerated by its tool, never hand-edited.
Implementation reviewers must reconcile required broader lanes with the touched module;
do not run an unfiltered multi-workspace suite on every iteration.

| Slice | Crates, landed change, and deleted path | Exact focused commands |
|---|---|---|
| **D.8.1 Port and resident backend ownership** | `worth-relational`: install the real read-session/backend contract over existing immutable root data; move representation mechanisms into `storage/backend/memory/`. Add typed denial and budgeted cursor primitives. Delete moved runtime storage implementations in their old locations; preserve one owner. Add passing/failing port lifetime/authority tests to grouped UI target. | `cargo test -p worth-relational --lib storage::port::`; `cargo test -p worth-relational --test ui storage_port` |
| **D.8.2 Native residency** | `worth-relational`: bounded head-column warm units, aggregate reservations, LRU attention, generation stamps, guard scope. Delete unguarded native head borrowing in the new residency owner and any whole-group admission assumption. History fields remain in backend authority until D.8.14 removes them from the old arena representation. | `cargo test -p worth-relational --lib storage::residency::`; `cargo test -p worth-relational --test ui storage_port` |
| **D.8.3 Point projections and observation reads** | `worth-relational`, `worth-runtime-bridge`, Query execution/core: convert borrowed records, entity projection, aspect/field revision, retirement, native probes, and truth record points. Replace existing facade signatures with fallible guarded/owned reads and migrate their entire caller closure, including Bridge snapshot and retained projection adapters. Delete the corresponding raw getter bodies and foreign-only error flattening. | `cargo test -p worth-relational --lib visibility::materialization::read_records::projection`; `cargo test -p worth-runtime-bridge --lib relational_source::bridge_snapshot_reader_tests`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_point`; `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --all-targets` |
| **D.8.4 Adjacency projections** | `worth-relational`, Query execution: borrowed/entity/frontier adjacency and revisions, `storage/partition/adjacency_queries.rs`; migrate query tree/path-union callers with typed causes. Delete adjacency slices borrowed from a partition and ledger paths that collect an unbounded list. | `cargo test -p worth-relational --lib adjacency`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_adjacency` |
| **D.8.5 Streaming and snapshots** | `worth-relational`, Query execution: exact/historical/streamed basis reads, slot sets, exact state building, snapshot liveness consumers. Delete unbounded all-record Vec and copied partition bitset selection paths; sessions retain root/key. | `cargo test -p worth-relational --lib visibility::snapshot_states`; `cargo test -p worth-relational --lib storage_port_history`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_basis` |
| **D.8.6 Kind and leased query reads** | `worth-relational`, Query execution/core: real resident kind membership in the semantic batch, entity/relation kind scans, leased explicit/fragment packet/query-plan execution. Delete linear kind filtering and bitset-derived partition discovery; D.9 later encodes the same index physically. | `cargo test -p worth-relational --lib storage_port_kinds`; `cargo test -p worth-relational --lib query_plan`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_selection`; `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --all-targets` |
| **D.8.7 Overlay and working-set preparation** | `worth-relational`, `worth-runtime-world`: journal-first record overlay; explicit read set and aggregate ceilings; root count reads; fallible touched-chunk construction. Delete full-partition clone modes and raw base-partition routing for these sites. Carry preparation storage denial through World's operation error. | `cargo test -p worth-relational --lib authority::commit::phases::prepare`; `cargo test -p worth-relational --lib storage::overlay`; `cargo test -p worth-runtime-world --features test-durability-faults --test runtime_world_certification storage_denial_prepare` |
| **D.8.8 Intent validation and mutation** | `worth-relational`: convert intent merge lookup/identity scan, stale targets, entity/relation patches, endpoints, relation lifecycle, and retirement cascades. Use bounded base cursors plus already admitted touched native columns. Delete each direct partition read from `authority/mutation` and `authority/intent_merge`. | `cargo test -p worth-relational --lib storage_port_mutation`; `cargo test -p worth-relational --test relational_certification root_named_delta_cow` |
| **D.8.9 Invariant state and applicability** | `worth-relational`, Query execution: state_view/slot_resolution/structural_adjacency, incidence collection, custom applicability, custom rules. Delete `SinglePartitionAccess`, uncharged incidence vectors, and raw applicability presence checks. Resolve AllObserved/FullObservedScan read contracts before execution. | `cargo test -p worth-relational --lib validation::engine::state_view`; `cargo test -p worth-relational --lib validation::execution::planning`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_invariant` |
| **D.8.10 Unique-field authority and checkpoint** | `worth-relational`: branch-root unique authority, same-batch delta checking, checkpoint import/export and schema introduction admission. Delete runtime-global refresh/rebuild, unique full-scan evaluator, and its rebuilt-index test setup. Re-declare the three uniqueness costs in section 6. Format bump and corrupt/missing index refusal land together. | `cargo test -p worth-relational --lib unique_invariant_lookup`; `cargo test -p worth-relational --test relational_certification invariant_global_uniqueness`; `cargo test -p worth-relational --lib storage_port_unique_checkpoint` |
| **D.8.11 Other built-in invariant bounds** | `worth-relational`: sidecar/live-count summaries, minimum matching kinds, all relation/graph rule cases in section 1. Delete snapshot full scan and current-world minimum builder; explicit graph denial replaces incomplete success. Convert evaluator entity-kind points. | `cargo test -p worth-relational --lib storage_port_invariant_bounds`; `cargo test -p worth-relational --lib relation_integrity_counters`; `cargo test -p worth-relational --test relational_certification invariant_proposed_state` |
| **D.8.12 Index, lineage, merge, and dependency consumers** | `worth-relational`, Query execution: index projection/lookup, lineage finalization, proposal touches, touched_scope, descriptive touches, merge current snapshots, authorization dependency capture. Delete ambient current-edition selection and raw kind/history reads. Derived-index full builds become explicit capped cursor work. | `cargo test -p worth-relational --lib storage_port_artifacts`; `cargo test -p worth-relational --lib durable_dependencies`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial_authorization` |
| **D.8.13 Persisted incremental commitment** | `worth-relational`: versioned commitment-node image, authenticated checkpoint, old/new semantic contributions, count/allocation summaries. Delete previous-partition arguments from commitment update and capture preparation. Re-declare commitment costs and replace whole-state publication projection with sealed batch construction. | `cargo test -p worth-relational --lib storage::overlay::partition_content`; `cargo test -p worth-relational --lib storage_port_commitment_checkpoint`; `cargo test -p worth-relational --test relational_certification root_cost_scale_axes` |
| **D.8.14 Head/history representation cutover** | `worth-relational`: migrate remaining arena mutation, retention, checkpoint, and verification consumers to separate resident version storage. Remove `metadata_history` from head arenas and warm copies; delete old history-in-head accessors and codecs. One batch changes both stores, no dual-write history format. | `cargo test -p worth-relational --lib storage_port_head_history`; `cargo test -p worth-relational --lib tests::durability::contracts::branch_root_checkpoint`; `cargo test -p worth-relational --lib storage_port_retention` |
| **D.8.15 Root-handle cutover and raw privacy** | `worth-relational`: replace root-region partition Arc, root capture, allocation evidence, inspection, partition edition, and all remaining trait/test definitions in section 4. Delete `root_partition_access.rs`, the outside-backend `PartitionAccess` surface, and every alternate raw escape. Change surviving get_partition visibility to backend-private; enforcement rejects recurrence. | `cargo test -p worth-relational --lib storage_port_root`; `cargo test -p worth-relational --test relational_certification root_fork_sharing`; `cargo test -p worth-relational --test relational_certification root_authoritative_accounting`; `cargo test -p worth-relational --test ui storage_port` |
| **D.8.16 Consumer denial court** | Bridge, World, Query execution/core/host/certification and bank-server: run fault matrix through all converted public callers, including exhaustive error mappings and managed continuations. Delete infallible assumptions and denial-to-absence translations found by the matrix. Bank-server gate runs for the public enum changes; no Store dependency. | `cargo test -p worth-runtime-bridge --features worth-relational/test-durability-faults --lib storage_denial`; `cargo test -p worth-runtime-world --features test-durability-faults --test runtime_world_certification storage_denial`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features test-durability-faults,test-primary-graph-faults --lib storage_denial`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-host --features test-durability-faults,test-output-delivery-faults --test temporal_conditional_operation storage_denial`; `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --test application_graph storage_denial`; `cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml -p bank-server --test identity_denials` |
| **D.8.17 Costs, docs, and milestone closure** | Relational and all affected consumer crates: complete section 6's focused roster, runtime pause-scope check, facade snapshots, and doc examples. Revise `OWNER_COMPONENT_PORT.md`, `BRANCH_LOCAL_MVCC.md`, `README.md`, `docs/api.md` section 7, Query read/error docs, and roadmap corrections from section 2; remove resident-root/infallible-read claims. Each example carries denial and limits. | `cargo test -p worth-relational --test relational_certification root_cost_scopes`; `cargo test -p worth-relational --test relational_certification root_persistent_path_accounting`; `cargo test -p worth-relational --features test-durability-faults --lib storage_port_faults`; `cargo test -p worth-relational --doc storage`; `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-host --all-targets`; `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --all-targets` |

The broader Relational roster is the existing `--lib tests::<owner>` family per touched owner,
and the grouped `--test relational_certification <module>` families, not new test binaries
per scenario. Keep existing owner-service, publication, branch isolation, schema, history,
checkpoint, and cancellation evidence green. The scheduled retained-root ceiling test
(`crates/worth-relational/tests/relational_certification/root/cost/scale_axes.rs:139`) runs once in its scheduled lane;
split any excessive case without lowering its lawful retention ceiling. Filter names for
existing courts come from `crates/worth-relational/tests/relational_certification.rs:94,100,112`.

## 6. Acceptance against the roadmap

### Relational suites green

Preserve identity, branch isolation, exact retained observations, fork sharing, schema
authority, canonical effects, cancellation posture, and settlement obligations. Ordinary
published reads remain available after seal while owner-admitting work is denied
(`crates/worth-relational/OWNER_COMPONENT_PORT.md:360`). Keep the existing Court Supply Chain
oracle and real publication facade, rather than reconstructing assertions from the new
backend. Re-run its root/isolation, invariants, owner-service, and scoped cost families.
Root retention means leased availability; a lease must no longer pin every record.

For boundedness, create a production-issued partition with head data ten times a configured
native budget, a history of 1,024 versions, and two diverged forks retaining old observations.
Demand two fitting head chunks, stream history concurrently, evict everything else, update
one record, then rewarm. Assert exact result parity against independently authored state,
peak native bytes no greater than the budget, zero version entries read by warming, zero
history promotion, and zero prior-partition materializations at publication. At `budget`,
`budget + 1`, and one oversized value, demand either fits exactly or returns the named
denial with zero allocation beyond reservation. The resident backend proves the port's
native/scratch bounds; it cannot prove disk page faults or total authoritative bytes smaller
than memory. D.9/D.10 repeat this through the physical backend and real application home.

### Certification cost counts

Re-declare these named costs, and only these costs or directly affected representation
navigation costs. Preserve unrelated semantic operation counts and zero-cost promises.

| Count | New declaration and reason |
|---|---|
| `publication_touched_region_count`, `publication_reused_region_count` | Regions continue to count changed/reused partition selectors, not warm chunks. Root metadata reuse remains exact. Native chunk fills/updates get separate counts so their amplification is not hidden in region counts. Source: `R/branch/root_capture.rs:156,157`, `R/inspection/mvcc/sharing_observation.rs:151`. |
| `publication_persistent_index_path_nodes` | Count backend immutable region/digest index nodes actually copied once per batch. Preserve the region radix's fixed-key path accounting where that structure remains; add commitment-node navigation separately. Do not claim Store page counts for memory allocations. Source: `R/branch/root.rs:76`, `R/branch/root_regions.rs:212`. |
| `publication_new_authoritative_bytes`, `copied_truth_bytes` | Authoritative backend head/version/index/commitment allocations, excluding derived warm bytes; copy each shared changed granule once. Replace old PartitionState/header accounting with handle/backend allocation evidence and exact independent allocation deltas. Source: `R/branch/root_capture.rs:130,163,166`; `crates/worth-relational/tests/relational_certification/root/cost/scale_axes.rs:65,74,76`. |
| `content_values_hashed` | Changed canonical head/version/adjacency/unique/kind contributions only, plus separately counted digest-node work. Persisting commitment removes full reconstruction on first post-checkpoint publish. Source: `R/branch/root.rs:77`, `R/storage/overlay/partition_content.rs:39`. |
| `partitions_cloned`, `entity_slots_cloned`, `relation_slots_cloned`; `aosoa_entity_chunks_staged`, `aosoa_entity_chunk_slots_materialized`, `aosoa_entity_chunks_published`, `aosoa_entity_slot_soa_merges`, `aosoa_prepare_soa_merge_count`, `aosoa_publish_soa_merge_count` | Replace full-partition clone counts with zero and separately count touched native records/chunks and copied bytes. Retain actual AoSoA staging/publication work under these existing names only while that layout still performs it; deleted merge mechanisms have zero counts, not relabeled fills. Sources: `R/performance/data/runtime_complexity_counters.rs:11,14`; `R/performance/access/working_state_counters.rs:5,18,25`; full clone path at `R/authority/commit/phases/prepare.rs:323`. |
| `invariant_entity_slot_scans` in uniqueness tests | Re-declare 2/2/4 as **zero unrelated slot scans**, zero authoritative read-record materializations, and exact unique probes. For a one-field update, admit one old key and one new key probe (two); no same-root full scan. For two colliding creates, compare proposal keys together before backend lookup. Source: `R/tests/complexity/contracts/commit_budgets/unique_invariant_lookup.rs:41,87,111`. |
| `invariant_entity_slot_scans`, `invariant_relation_slot_scans` outside uniqueness; `visibility_entity_slot_scans`, `visibility_relation_slot_scans`, `visibility_exact_state_materializations`, `visibility_cache_miss_reconstructions` | Keep zero unrelated-kind scans and zero exact-state full-bitset copies. Historical reconstruction counts still charge real bounded reconstruction; lazy exact-root selection is not reconstruction. Charge matching kind entries, adjacency candidates, version entries, summary probes, and explicit denial progress separately. Sources: `R/performance/data/runtime_complexity_counters.rs:27,55,61`; `R/validation/engine/evaluator/relation_cardinality/minimum_current_index.rs:35,61`; `R/validation/engine/evaluator/record_surface_rules/snapshot_entity_limits.rs:55`; `R/visibility/snapshot_states/exact_state_building.rs:47,58`. |
| Exact borrowed/read navigation work bounds | Re-declare fixed 33/38/39 arithmetic as actual selected-backend navigation plus record/schema steps. Preserve the independent semantic probes; root lookup can no longer assume an in-memory radix for every backend. Source: `R/visibility/materialization/read_records/projection/borrowed_records.rs:38,57`; `R/visibility/materialization/read_records/projection/field_revisions/admitted_entity.rs:35`. |

`copied_commit_envelopes = 0`, fork truth-copy bytes = 0, branch population scans = 0, and
one validation/preparation/publication attempt per successful attempt remain unchanged
(`crates/worth-relational/tests/relational_certification/root/cost/scale_axes.rs:53`, `crates/worth-relational/tests/relational_certification/root/sharing/fork.rs:29`).
Retained-history length must not change ordinary publication work
(`crates/worth-relational/tests/relational_certification/root/cost/scale_axes.rs:194,215`). Re-declarations replace dishonest representation costs with
measured granules, never simply loosen a threshold. Native fill/eviction, owned-result bytes,
version entries, unique probes, kind candidates, commitment reads, and peak scratch each
have distinct counters. D.9 adds physical pages/bytes/faults alongside those logical counts.

### Every caller carries storage denial

The fault backend decorates the final resident backend through the production backend
contract. It uses actual owner-issued roots, records, branch observations, and resources;
it does not forge a basis or install another composition root. Fault controls exist only
in Relational's established `test-durability-faults` feature
(`crates/worth-relational/Cargo.toml:14`); Bridge enables that dependency feature directly.
World and Query use their existing forwarding features; certification already enables
the execution fault features in its dev-dependency. No production fallback is installed. An operation selector injects a typed denial before the
first step or after a chosen number of successful steps, for each family: residency,
records, versions, adjacency, kinds, unique/summaries, commitment, and export. It records
steps, allocation reservations, current guards, and release terminality.

For every section 4 caller family and facade propagation family in section 3, execute a
successful counterpart and faults at first/middle/final relevant reads. Assert the exact
cause and progress at the outer API, no panic, no conversion to None/empty/pass, no head
movement during failed prepare, no new snapshot/result falsely reported complete, and all
guards/reservations released. A partial streaming page can be reported only as partial,
with the last successfully emitted key; a retry starts at that key on the same leased root.
Inject damage separately from work exhaustion and cancellation so callers cannot flatten
them into one retryable class. A deliberately erased error must fail the outer assertion.

The final source deletion inventory and compiler failures prevent bypass; the fault court
proves the exercised callers propagate denials. A finite suite alone cannot prove every
possible future caller, so all new read crossings have fallible signatures and mandatory
exhaustive mappings. Inspect all caller closure changes, including Bridge/World/Query
private adapters and public facades. Compile bank-server when Query denial enums change.

QA focuses on authority/currentness, native budget and pause safety, exact historical
selection, same-batch indexes, and denial propagation. Happy-path parity alone is insufficient:
the oversized partition, retained fork, checkpointed commitment, and mid-stream fault cases
must fail if a full-partition copy, stale warm reuse, open-time rebuild, or swallowed error
is reintroduced.

## 7. What D.9 needs without a port change

D.9 implements the same backend contract over D.6 tree reads/writes. Relational root custody
binds to the exact D.4 root generation and family set; long readers keep leases and keys,
with D.5's bounded page guards inside each step. The binding owns order-preserving encoding
for records, versions, both adjacency directions, kind membership, unique keys, aspects,
summary/commitment entries, and column chunks. Relational chooses semantic ranges and limits;
the binding reports logical work and physical amplification. Relational and Query core
never import Store, and Store never imports their meaning.

Preparation yields the canonical semantic batch with all correctness changes and expected
predecessor. D.9 lowers it once into a sorted physical batch, includes its pending settlement
record, publishes by root CAS, and resolves fate through D.6. Warm copies consume the same
committed semantic batch after known movement; indeterminate fate cannot install speculative
successor warmth. A fork shares the root handle; drop/name retirement share D.9's durable
batch. Symbols, allocator reservations, lineage, schema images, and index definitions remain
separate owner metadata obligations from D.9's list; their durable binding cannot require
reopening a resident record partition. D.10 extends the same pattern to World/Query inventory
and supplies demand-to-unit mapping. Neither milestone changes the D.8 read/session/guard API.

The D.9 court repeats Relational parity against resident and tree backends, then crash/fate
tests on file media. D.8's fault backend certifies caller behavior only; it does not certify
the real binding, WAL, page faults, or fresh-process recovery. Local-file envelope durability
is removed in D.9 as the roadmap requires; D.8 does not wire it into an application home.

## 8. Owner decisions still needed

- **Native profile values:** recommend explicit operator/profile budgets for total residency,
  per-unit native bytes, concurrent fills, and per-read work/result/scratch. No silent default
  or permission to exceed the budget. Exact production values require the owner's workload
  distribution; the chunk/budget/denial behavior above is settled.
- **Unindexed user global invariants:** recommend the ordinary rejection policy above, with an
  indexed bounded declaration or explicit certification audit as the repair. Confirm the
  public diagnostic wording and schema-introduction resource policy; do not weaken pass
  completeness or restore full-state ordinary scans to preserve an old happy path.

There is no open placement, warm-unit, root authority, unique-index, history, or Store
dependency decision. Those follow from the roadmap, the existing caller semantics, and laws.
