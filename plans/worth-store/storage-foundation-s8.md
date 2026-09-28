# Storage Foundation S.8 Engineering Spec: Index, Layout, And Access-Path Discipline

## Goal

Make Worth Store's physical access structures explicit, typed, costed,
rebuildable, versioned, and auditable per durable artifact family.

`S.8` is not a blob-only milestone. It is the storage-wide layout discipline
layer that prevents pages, WAL, recovery records, blob indexes, reclaim maps,
security/custody lookup structures, and future physical artifacts from
accidentally sharing a generic index story that hides cost or authority.

## Why This Milestone Exists

After `S.7`, Worth Store has native blob storage, lifecycle evidence, recovery
surfaces, streaming paths, reachability, reclaim, and physical placement
pressure. Those systems cannot scale safely if the next lookup path is chosen
by convenience.

This milestone turns physical layout into an explicit proof flow:

1. A durable artifact family declares what it is.
2. Store admits the layout strategy for that family.
3. Store lowers an access request into a typed access plan.
4. Store executes the plan through the declared path.
5. Store records exact access-path and amplification counters.
6. Store can rebuild, migrate, or deny the structure without losing authority.

The goal is not to add more proof nouns. The goal is proof grammar: visible
states, named transitions, exact denial cases, and public APIs that teach the
physical lifecycle.

## Non-Goals

- Do not build the application-facing query/indexing layer.
- Do not guess industry-specific semantic indexes.
- Do not expose a generic plugin callback that can smuggle an unproven index
  into the engine.
- Do not make certification the source of layout law.
- Do not treat Foundational layout or performance claims as executed Store
  proof.
- Do not make JSON, serde, logs, diagnostics, or report rows admission
  authority.

## Governing Constraint

No durable artifact family may be readable, searchable, streamed, mutated,
migrated, rebuilt, repaired, or certified through an implicit access path whose
key domain, comparator law, structure invariants, authority role, derivation
status, materialization state, coverage basis, absence proof, pre-execution
budget, observed cost, rebuild source, corruption behavior, mutation protocol,
version policy, or trust-boundary posture is undeclared.

Broad scans are allowed only when the scan itself is the declared access shape,
or when running a typed verifier, rebuild, migration, or diagnostic lane with
exact counters and no authority upgrade from the scan result alone.

An explicit degraded exact scan is allowed only as its own caller-visible,
budgeted, counter-backed, non-indexed outcome class. It must not masquerade as
point, prefix, range, streaming, locality-bounded, or cheap foreground access.

## Architectural Frame

### Store Owns Layout Law

Worth Store owns the physical artifact vocabulary, layout admission rules,
access-path lowering rules, and runtime counter semantics. Lower crates must
carry the law. Certification may prove the law was followed, but certification
must not define the law.

### Foundational Usage

Worth Foundational is used where Store crosses shared platform boundaries:

- `FoundationalLayoutIntentClaim` may describe the intended layout shape or
  performance posture, but it is not proof of Store execution.
- `FoundationalPolicyAdmissionReceipt` may record that a layout policy was
  admitted, but it is not enough to prove the access path executed correctly.
- `FoundationalCounterBackedPerformanceReceipt` is produced only from executed
  Store counters.
- `FoundationalPerformanceCounterRow` is used for boundary/reporting rows, not
  as internal Store authority.
- `FoundationalPerformanceReportPlan` and `MaterializedPerformanceReport` are
  terminal/reporting surfaces unless readmitted into a Store-owned witness.
- Certified or readmitted performance bundles may support trust-boundary
  movement, but Store must still preserve native authority internally.

The ladder is:

`layout intent claim -> Store layout admission -> Store executed counters ->
Foundational counter-backed receipt -> report/certification/readmission`.

The reverse direction is forbidden.

### Worth Proof Usage

Worth Proof is used to encode transition grammar:

- unresolved layout declaration
- resolved artifact-family layout declaration
- admitted layout strategy
- lowered access plan
- execution-ready access path
- executed access evidence
- counter-backed performance evidence
- rebuild, migration, rollback, or denial outcome

Use `TransitionOutcome` faithfully. Denied, deferred, stale, rebind-required,
and failed cases must remain distinct. A broad scan, stale layout version, or
incompatible authority source must not be collapsed into ordinary success.

### Authority And Projection

Physical indexes and layout projections are derived unless a Store-owned type
explicitly classifies a durable artifact as authoritative. Derived indexes may
accelerate access, but they must not become source truth through convenience.

### Physical Mechanics Lock

S.8 is a physical database foundation milestone, not only a proof-vocabulary
milestone. If Store admits B-tree, LSM, range, prefix, hash, chunk-tree,
manifest-walk, cache-assisted, or streaming access, the admitted strategy must
have enough concrete mechanics to make the claim mechanically testable.

S.8 must therefore introduce or consume Store-owned law for:

- physical key domains, canonical key encodings, comparators, prefix bounds,
  range bounds, composite-key ordering, hash collision behavior, and tenant/key
  scope partitioning
- baseline B-tree and LSM strategy families with real lookup, publication,
  mutation, integrity, and recovery invariants
- strategy invariant suites that make corruption, rebuild, migration, and cost
  expectations testable before a strategy is admitted
- materialization state, coverage basis, freshness watermarks, absence proofs,
  and derived accuracy class propagation
- deterministic physical plan selection, plan fingerprints, pre-execution cost
  budgets, planned counter envelopes, and planned-versus-observed receipts
- live index maintenance modes, mutation protocols, and publication states
- bootstrap catalog discovery that is tiny, fixed, versioned, checksummed, and
  incapable of answering ordinary access
- legacy access-path disposition and bypass proof for old Store surfaces
- layout/access-path hazard inventory for S.12 certification

## Domain Skeleton Contract

S.8 must be implemented through the current dedicated Store crate family, not
through a new catch-all layout crate and not by collapsing existing domain
crates into a proof-vocabulary bag.

The skeleton below is the selected architecture and target topology. Phase 0
creates the missing architectural homes and verifies the existing parts line up
with them. Later phases implement inside this skeleton. Individual files may
become directories, and local decomposition may vary where the domain shape
earns it, but ownership, lifecycle order, and authority direction do not
reopen unless the spec is explicitly amended.

### Workspace Crate Responsibilities

- `worth-store-contracts`: shared Store vocabulary, ids, lightweight enums,
  and cross-crate contract names. It may define words such as artifact-family
  ids or authority-role names, but it must not mint admitted layout authority,
  execute access, or construct performance receipts.
- `worth-store-layout-indexes`: the S.8 layout-law and access-path grammar
  crate. It owns artifact-family declaration, key-domain law, strategy
  declaration, strategy invariant suites, access-shape contracts, deterministic
  plan selection, budget admission, lowered/ready/executed access evidence,
  planned-versus-observed counter receipts, derived-index rebuild/parity,
  migration/rollback vocabulary, degraded exact scan posture, cache/advisory
  non-authority posture, legacy disposition, and S.9 handoff vocabulary.
- `worth-store-physical-format`: page, frame, segment, extent, record,
  manifest, root-discovery, binary-format, and offline physical-format layout
  families. It supplies Store-owned physical byte authority and consumes S.8
  layout law where those families need declared access paths.
- `worth-store-wal`: WAL, checkpoint, durable mutation, record-family, and
  replay-tail layout families. It keeps WAL authority local and consumes S.8
  strategy/access grammar rather than letting layout indexes own WAL truth.
- `worth-store-recovery-physics`: recovery, replay, crash-boundary,
  checkpoint-cutover, readmission, and partial-publication layout families. It
  owns recovery physics and consumes S.8 layout/access state machines.
- `worth-store-buffer-pool`: resident frame, page lease, dirty state,
  zero-copy/bounded-copy view, read-ahead, and write-behind access families. It
  owns memory-residency authority and feeds exact residency/allocation
  evidence into S.8 counters.
- `worth-store-physical-integrity`: pre-decode integrity, checksum coverage,
  damage handoff, corruption localization, and physical-integrity admission
  families. It owns byte-integrity authority before logical decode.
- `worth-store-physical-isolation`: stable read execution, reclaim barriers,
  compaction interlocks, movable stability, orphan reclaim, and S.6/S.8 I/O
  isolation handoffs. It owns physical stability and interference boundaries.
- `worth-store-io-scheduler`: foreground/background admission, pacing,
  reservation, and I/O queue execution. It consumes S.8 access budgets and
  counter envelopes; it does not become semantic or layout authority.
- `worth-store-blob-chunks`: blob object, chunk identity, chunk integrity,
  chunk tree, dedupe, streaming, publication, reachability, placement,
  compaction, corruption, export/import, capsule readiness, and closeout
  families. It owns blob lifecycle authority and consumes S.8 layout/index law
  for blob access paths.
- `worth-store-security`: S.5.1 tenant scope, key scope, custody,
  authenticity, repair blast-radius, and readmission scope families. It owns
  security-scope posture consumed by S.8 key domains, tenant partitions, and
  trust-boundary readmission.
- `worth-store-operations`: backup, restore, import/export, repair,
  capsule, and operator workflow layout families. It owns operational workflow
  posture and consumes S.8 layout/readmission grammar.
- `worth-store-offline-verifier`: terminal/offline verification projections
  and verifier observations. It may observe persisted layout evidence, but it
  cannot mint foreground Store authority.
- `worth-store-readiness`: cross-milestone readiness and handoff objects,
  including the S.8 -> S.9 readiness handoff where the handoff is lower-crate
  vocabulary rather than certification-only testimony.
- `worth-store-physical-certification`: executable physical harness,
  scenarios, drivers, faults, observers, oracles, coverage rows, transcripts,
  and S.8 simulation vocabulary. It consumes production witnesses and executed
  evidence; it must remain a courtroom/harness layer.
- `worth-store-certification`: certification courtroom and closeout. It
  assembles proof that production law was followed. It must not define the
  production law or expose constructors that lower crates rely on.
- `worth-store-test-support`: honest fixtures and builders over production
  facades. It may help tests reach real states, but it must not mint authority
  that production code cannot obtain.
- Existing specialized crates such as `worth-store-maintenance`,
  `worth-store-retention`, `worth-store-tiering`, `worth-store-snapshots`,
  `worth-store-branch-deltas`, `worth-store-compatibility`,
  `worth-store-replication`, `worth-store-bulk`, and
  `worth-store-live-query` must consume S.8 layout/access contracts only for
  the artifact families they own. S.8 may add narrow dependencies or wrappers
  where those families need layout posture, but it must not turn
  `worth-store-layout-indexes` into a semantic feature crate.

### Selected Layout-Indexes Skeleton

`worth-store-layout-indexes` must become the central S.8 grammar crate with
this target directory skeleton. The names below are architectural homes, not a
promise that every leaf remains a single file forever. A listed `.rs` file may
become a directory with a `mod.rs` and focused children when the implementation
needs more room, but it must keep the same responsibility and facade position:

```text
crates/worth-store-layout-indexes/src/
  lib.rs
  artifact_family/
    mod.rs
    declaration.rs
    authority.rs
    lifecycle.rs
    inventory.rs
    denial.rs
    tests.rs
  key_domain/
    mod.rs
    encoding.rs
    comparator.rs
    prefix.rs
    range.rs
    composite.rs
    hash_collision.rs
    tenant_partition.rs
    denial.rs
    tests.rs
  strategy/
    mod.rs
    family.rs
    declaration.rs
    capability.rs
    invariant_suite.rs
    admission.rs
    denial.rs
    tests.rs
  strategy/btree/
    mod.rs
    node_format.rs
    separator.rs
    split_merge.rs
    root_publication.rs
    invariants.rs
    tests.rs
  strategy/lsm/
    mod.rs
    memtable_wal.rs
    run_publication.rs
    tombstone.rs
    compaction_ordering.rs
    invariants.rs
    tests.rs
  materialization/
    mod.rs
    state.rs
    coverage.rs
    watermark.rs
    absence.rs
    completeness.rs
    denial.rs
    tests.rs
  access_shape/
    mod.rs
    contract.rs
    point.rs
    range.rs
    prefix.rs
    scan.rs
    streaming.rs
    append.rs
    verifier.rs
    repair.rs
    quarantine.rs
    denial.rs
    tests.rs
  planning/
    mod.rs
    lowering_request.rs
    alternative.rs
    selection_policy.rs
    plan_fingerprint.rs
    selection_receipt.rs
    tests.rs
  budget/
    mod.rs
    estimate.rs
    admitted_budget.rs
    planned_counter_envelope.rs
    violation.rs
    tests.rs
  execution/
    mod.rs
    lowered_plan.rs
    ready_plan.rs
    executed_evidence.rs
    planned_vs_observed.rs
    amplification_receipt.rs
    denial.rs
    tests.rs
  maintenance/
    mod.rs
    rebuild.rs
    parity.rs
    mutation_plan.rs
    maintenance_mode.rs
    publication_protocol.rs
    lag.rs
    failure.rs
    tests.rs
  migration/
    mod.rs
    version.rs
    compatibility.rs
    migration_plan.rs
    rollback_plan.rs
    stale_rebind.rs
    tests.rs
  corruption/
    mod.rs
    classification.rs
    quarantine.rs
    readmission.rs
    denial.rs
    tests.rs
  bootstrap/
    mod.rs
    catalog.rs
    root_discovery.rs
    catalog_read_admission.rs
    bootstrap_only_path.rs
    tests.rs
  degraded_access/
    mod.rs
    exact_scan.rs
    ephemeral_aid.rs
    advisory_aid.rs
    cache_hit.rs
    never_authority.rs
    denial.rs
    tests.rs
  legacy_disposition/
    mod.rs
    inventory.rs
    disposition.rs
    bypass.rs
    tests.rs
  skeleton/
    mod.rs
    crate_role.rs
    responsibility_map.rs
    topology_closeout.rs
    authority_flow.rs
    phase_obligation.rs
    tests.rs
  handoff/
    mod.rs
    hazard_inventory.rs
    s9_layout_handoff.rs
    tests.rs
  facade/
    mod.rs
    declarations.rs
    key_domains.rs
    strategy_admission.rs
    access_planning.rs
    access_execution.rs
    maintenance.rs
    migration.rs
    readmission.rs
    closeout.rs
  compile_fail/
    mod.rs
    raw_construction.rs
    facade_bypass.rs
    certification_authority.rs
```

`lib.rs` may only declare modules and re-export `facade::*` plus documented
compile-fail entrypoints. Business logic belongs in the domain directories
above.

### Required Family-Crate S.8 Homes

Existing family crates keep execution authority. S.8 work in those crates must
land in these target homes. A listed file may become a directory when the local
family needs declaration, classification, transition, receipt, counter, or test
submodules, but the family must stay under its named `layout_access` home:

```text
worth-store-physical-format/src/layout_access/
  page_family.rs
  frame_family.rs
  segment_family.rs
  extent_family.rs
  record_family.rs
  manifest_family.rs
  root_discovery_family.rs
  format_family_closeout.rs

worth-store-wal/src/layout_access/
  wal_record_family.rs
  wal_segment_family.rs
  checkpoint_family.rs
  durable_mutation_family.rs
  replay_tail_family.rs
  wal_layout_closeout.rs

worth-store-recovery-physics/src/layout_access/
  recovery_source_family.rs
  replay_index_family.rs
  crash_boundary_family.rs
  checkpoint_cutover_family.rs
  readmission_family.rs
  recovery_layout_closeout.rs

worth-store-buffer-pool/src/layout_access/
  resident_frame_family.rs
  page_lease_family.rs
  dirty_state_family.rs
  zero_copy_view_family.rs
  read_ahead_family.rs
  write_behind_family.rs
  buffer_pool_layout_closeout.rs

worth-store-physical-integrity/src/layout_access/
  checksum_family.rs
  pre_decode_family.rs
  scrub_family.rs
  damage_map_family.rs
  quarantine_family.rs
  integrity_layout_closeout.rs

worth-store-physical-isolation/src/layout_access/
  stable_read_family.rs
  reclaim_barrier_family.rs
  compaction_interlock_family.rs
  movable_stability_family.rs
  orphan_reclaim_family.rs
  isolation_layout_closeout.rs

worth-store-io-scheduler/src/layout_access/
  foreground_admission_family.rs
  background_reservation_family.rs
  queue_execution_family.rs
  pacing_family.rs
  io_layout_closeout.rs

worth-store-blob-chunks/src/layout_access/
  blob_object_family.rs
  chunk_tree_family.rs
  streaming_family.rs
  dedupe_family.rs
  reachability_family.rs
  retention_family.rs
  reclaim_family.rs
  compaction_family.rs
  export_import_family.rs
  capsule_family.rs
  blob_layout_closeout.rs

worth-store-security/src/layout_access/
  tenant_scope_family.rs
  key_scope_family.rs
  custody_family.rs
  authenticity_family.rs
  repair_blast_radius_family.rs
  security_layout_closeout.rs

worth-store-operations/src/layout_access/
  backup_family.rs
  restore_family.rs
  import_family.rs
  export_family.rs
  repair_family.rs
  capsule_operation_family.rs
  operations_layout_closeout.rs
```

These `layout_access` modules adapt local authority into the S.8 grammar and
execute local family behavior. They must not become duplicate layout-law
crates. Shared S.8 nouns stay in `worth-store-layout-indexes`; local executed
evidence and closeout stay in the owning family crate.

### Required Courtroom And Test Skeleton

The courtroom/test crates must use these target homes. Files may split into
directories when scenarios, oracles, coverage, or closeout evidence need
finer-grained structure, but the courtroom/test modules must stay under these
milestone-specific roots:

```text
worth-store-physical-certification/src/harness/by_milestone/s8_layout_access/
  mod.rs
  scenario.rs
  actors.rs
  drivers.rs
  faults.rs
  observers.rs
  oracles.rs
  coverage.rs
  transcript.rs
  simulation.rs
  shortcut_denials.rs
  heavy_blob_profile.rs
  tests.rs

worth-store-certification/src/s8_layout_closeout/
  mod.rs
  sources.rs
  classifier.rs
  verifier.rs
  certificate.rs
  handoffs.rs
  denial.rs

worth-store-test-support/src/harness/milestone/s8_layout_access/
  mod.rs
  fixtures.rs
  family_builders.rs
  scenario_builders.rs
  adversarial_inputs.rs
```

These modules may consume production S.8 witnesses and executed evidence. They
must not define constructors or authority vocabulary that production crates
depend on.

### Proof-Flow Rule Inside Each Home

Every family module listed above must keep this shape internally. If the module
is large enough to split, it splits along these names; if it stays as one file,
the file must still expose functions in this order:

```text
declare_*            raw Store-owned declaration
classify_*           named decision case
verify_*             pure transition requirements
admit_* or lower_*   state transition returning next capability
build_*_receipt      receipt/certificate after successful transition
publish_*            facade-visible next capability where relevant
```

Do not use `types`, `proofs`, `receipts`, `helpers`, or `support` as junk
drawers. If those names appear, they must be local to a single family and
narrow enough that a reviewer can still answer:

- what source authority enters
- what evidence is admitted
- what case is classified
- what transition is verified
- what receipt or witness is constructed
- what next capability is exposed
- what denial cases are explicit

### Cross-Crate Flow Shape

Every S.8 implementation phase must preserve this direction:

```text
family crate authority
-> worth-store-layout-indexes declaration/admission/access grammar
-> family crate execution/readiness/evidence
-> Foundational boundary receipt or report only after Store execution
-> physical-certification/certification courtroom proof
```

The reverse direction is forbidden. Certification, offline verifier reports,
Foundational layout claims, JSON/serde declarations, copied counter rows, and
test-support fixtures may never become production S.8 authority unless a
Store-owned readmission transition explicitly admits them.

### New Crate Rule

S.8 should prefer the existing crate family. Add a new crate only when all of
these are true:

- the new responsibility is not already owned by an existing Store crate
- placing it in `worth-store-layout-indexes` would make that crate own
  execution authority rather than layout/access grammar
- placing it in a family crate would create an upward dependency cycle
- the public contract can be named in one sentence without using "misc",
  "common", "support", "proofs", or "helpers"

Otherwise, add narrow modules to the existing owning crate and wire them
through lifecycle-shaped facades.

## Known API Surface Map

This milestone must not leave API discovery to implementation time. The exact
S.8 type names may evolve during implementation, but the implementation must
start from these known surfaces and either consume them directly, adapt them
through narrow Store-owned wrappers, or introduce the planned S.8 surfaces named
below.

### Worth Proof Surfaces

Use `worth_proof` for progression law, not storage execution:

- `use worth_proof::prelude::*`
- `recipe(payload)`
- `.resolve_with(authority, basis)`
- `.lower_with(capability)`
- `.admit_with(authority)`
- `.ready_with(authority, runtime)`
- `.execute()`
- checked progression: `.try_resolve_ready(...)`, `.try_lower_ready(...)`,
  `.try_admit_ready(...)`, `.try_ready_now(...)`, `.try_execute()`
- trust-boundary/freshness progression:
  `.bridge_trust_boundary()`, `.rebind_with(...)`, `.readmit_with(...)`
- raw lane when the phase needs explicit transition nouns:
  `worth_proof::raw::*`, `Recipe<Unresolved, T>`, `Recipe<Resolved, T>`,
  `Recipe<Lowered, T>`, `ExecutionReadyRecipe<T, A>`, and
  `ExecutedRecipe<T, A>`
- non-success topology:
  `TransitionOutcome`, `SuccessfulTransitionOutcome`,
  `DenialTransitionOutcome`, `DeferredTransitionOutcome`,
  `FreshnessTransitionOutcome`, `ProofOutcome`, and `ProofOutcomeKind`
- deterministic shape helpers:
  `Pair<T>`, `NonEmpty<T>`, `CanonicalVec<T>`, `UniqueVec<T>`,
  `join_ready(...)`, `compose_ready(...)`, `family_pair(...)`, `create(...)`,
  `rewrite(...)`, `supersede(...)`, `retire(...)`, `sym(...)`, and
  `member(...)`

S.8 must use these surfaces for:

- artifact-family declaration progression
- layout strategy admission
- access-plan lowering
- execution readiness
- executed access evidence
- stale/rebind/readmission handling
- deterministic family composition when one Store operation touches a fixed set
  of layout families

### Worth Foundational Surfaces

Use `worth_foundational` for shared boundary vocabulary, not Store-owned layout
law:

- `worth_foundational::performance_api::common_path`
- `worth_foundational::performance_api::common_path::performance()`
- `FoundationalPerformanceClaimAuthoringFrontDoor`
- `FoundationalLayoutIntentClaim`
- `worth_foundational::performance_api::lower_lane::policy`
- `FoundationalPolicyAdmissionReceipt`
- `worth_foundational::performance_api::lower_lane::basis`
- `worth_foundational::performance_api::lower_lane::receipts`
- `counter_backed_performance_receipt(bundle)`
- `FoundationalCounterBackedPerformanceReceipt`
- `FoundationalPerformanceCounterRow`
- `worth_foundational::performance_api::lower_lane::reports`
- `FoundationalPerformanceReportPlan`
- `FoundationalMaterializedPerformanceReport`
- `worth_foundational::performance_api::stronger_lane::certified`
- `worth_foundational::performance_api::stronger_lane::readiness`
- `worth_foundational::performance_api::performance_public_surface_inventory()`
- `worth_foundational::canonicalization_api::{common_path, lower_lane,
  stronger_lane}`
- `worth_foundational::boundary_evidence_api::{common_path, lower_lane,
  stronger_lane}`
- `worth_foundational::aspects()`, `worth_foundational::compatibility()`, and
  native aspect carriers such as `AspectValue`, `StructAspectValue`,
  `AspectKey`, `ContractValidatedAspectValue`, and admitted authoritative
  aspect-state surfaces when S.8 evidence crosses aspect-native boundaries

The Foundational strength ladder for S.8 is fixed:

```text
Store layout declaration
-> Store layout admission
-> Store lowered access plan
-> Store executed access counters
-> FoundationalCounterBackedPerformanceReceipt
-> FoundationalMaterializedPerformanceReport or certified/readmitted bundle
```

The reverse direction is not legal.

### Existing Worth Store Surfaces To Consume Or Refactor

S.8 must inspect and either consume, wrap, migrate, or explicitly supersede
these current Store surfaces:

- public facade: `WorthStore`, `WorthStoreBuilder`, and the
  `crates/worth-store/src/facade` and `backend/facade` families
- current layout/chunk model:
  `AspectLayoutReadRequest`, `AdmittedAspectLayoutReadPlan`,
  `RejectedAspectLayoutReadPlan`, `AspectLayoutReadExecutionDecision`,
  `AspectLayoutReadExecutionResult`, `AspectReadRegime`,
  `AspectLayoutFallbackClass`, `AspectLayoutPerformanceEnvelope`,
  `Milestone6LayoutMaterialization`, `Milestone6PreparedLayoutSupport`,
  `Milestone6ResolvedLayoutSupportLane`, `ChunkModelFrozenPhysicalLayout`,
  `ChunkDeterminismWitness`, `PhysicalChunkId`, `StructuralBlockId`,
  `StructuralBlockLookup`, `DedupAdmittedBlockReuse`,
  `DedupBackedReadResult`, `Milestone7IndependentLayoutReference`, and
  `Milestone9PhysicalChunkReference`
- current media/WAL surfaces:
  `DurabilityBarrierClass`, `DurableBackendFamily`, `DurableMediaReport`,
  `WalRecord`, `WalRecordFamily`, `WalRecordPayload`, `DurableMutationId`,
  `DurablePublicationPhase`, `RecoveryDecisionClass`, and
  `CURRENT_WAL_VERSION`
- current compatibility/versioning surfaces:
  `ArtifactFamilyId`, `ArtifactFormatVersion`, `ArtifactSemanticVersion`,
  `ArtifactCompatibilityWindow`, `CompatibilityFamilyDeclaration`,
  `CompatibilityRegistry`, `CompatibilityReadIntent`,
  `CompatibilityWriteIntent`, `ReadCompatibilityReceipt`,
  `WriteCompatibilityReceipt`, `CompatibilityReadAdmissionOutcome`,
  `CompatibilityWriteAdmissionOutcome`, `CompatibilityRejection`,
  `CompatibilityRejectionKind`, `CompatibilityDerivedRebuildRequest`,
  `CompatibilityDerivedRebuildOutcome`, `RollingUpgradePolicy`,
  `RollingUpgradeAdmissionPlan`, `RestoreCompatibilityPlan`, and
  `MixedVersionStorePosture`
- current maintenance/retention/compaction surfaces:
  `MaintenanceDeclaration`, `AdmittedMaintenanceDeclaration`,
  `LoweredRetentionMaintenanceBatch`, `LoweredCompactionDeclaration`,
  `LoweredRebuildDeclaration`, `LoweredReclaimDeclaration`,
  `CompactionPlan`, `CompactionCutoverWitness`, `PublishedCompactionProduct`,
  `ReclaimEligibilityWitness`, `ReclaimExecutionReport`,
  `RetentionPlanningReport`, `RetentionClosureWitness`,
  `RetentionMaintenanceVerification`, `DerivedFamilyRetentionPolicy`, and
  `LayoutFamilyCompactionUnit`
- current recovery/snapshot/live-query/tiering surfaces:
  `RecoveryStatusReport`, `RecoverySourceReport`, `RecoveryQuarantineScope`,
  `DurableRecoveryOutcome`, `SnapshotImageBundle`, `SnapshotReadRequest`,
  `SnapshotReadResult`, `StableBasisReadRequest`, `StableBasisReadPlan`,
  `ContinuationBatchResult`, `ContinuationRetentionStatus`,
  `PlacementBoundArtifactRef`, `PlacementResolvedReadHandle`,
  `ResidentReadLease`, `ColdRecallLease`, `TierPlacementEvidence`, and
  `RecallAmplificationBudget`
- current counters and certification evidence:
  `StoreCounterSnapshot`, `Milestone6CounterContract`,
  `Milestone7CounterContract`, `Milestone10CounterContract`,
  `Milestone11CounterContract`, `Milestone12CounterContract`,
  `Milestone6AccessStructureVerification`,
  `Milestone7AccessStructureVerification`, and the S.4.5 harness/certification
  modules under `crates/worth-store/src/tests/harness`
- S.0 handoff and Roadmap 2 source-boundary surfaces:
  `StorageFoundationS1Handoff`, `EvidenceBundleReadiness`,
  `RoadmapSequenceStatusMatrix`, `S0ValidatedStorageFoundationS1HandoffArtifact`,
  `S0CounterSnapshot`, `S0ComplexityContract`, and `S0StableDigest`

### Planned Store Surfaces From S.1 Through S.7

S.8 is allowed to depend on the planned closed outputs of S.1 through S.7, and
must preserve their authority distinctions:

- S.1 physical substrate:
  `PhysicalPageId`, `PhysicalSegmentId`, `PhysicalExtentId`,
  `PhysicalFrameId`, `PhysicalRecordSlot`, `PhysicalGeneration`,
  `PhysicalEpoch`, `PhysicalReference`, `PhysicalRootReference`,
  `PhysicalPageHeader`, `PhysicalFrameHeader`, `PhysicalRootManifest`,
  `PhysicalManifestIndex`, `PhysicalFreeSpaceSearchPolicy`,
  `PhysicalFragmentationPressureReport`, `PlatformPhysicalFacade`,
  `OfflinePhysicalVerifier`, and `S2PhysicalSubstrateReadiness`
- S.2 buffer/memory surfaces:
  page leases, pin/unpin handles, resident-byte counters, pinned-page counters,
  dirty-page counters, allocation-scope counters, zero-copy/bounded-copy record
  views, read-ahead/write-behind admission, and OOM/admission denials
- S.3 integrity surfaces:
  page/frame/chunk checksums, pre-decode integrity validation, scrub evidence,
  corruption localization, quarantine witnesses, and readmission evidence
- S.4 WAL/recovery surfaces:
  WAL segments, page-LSN and checkpoint manifests, recovery-source precedence,
  replay receipts, recovery source reports, and bounded-recovery counters
- S.5 read-stability surfaces:
  stable physical read plans, physical leases, COW publication, generation
  validation, and reclaim barriers
- S.5.1 security-scope surfaces:
  key scope, key version, tenant scope, authenticity class, custody posture,
  repair blast-radius readiness, and import/readmission scope witnesses
- S.6 I/O isolation surfaces:
  foreground latency guard, I/O scheduler admission, direct-I/O posture,
  background work reservations, physical backend evidence, and
  `S6IoQosIsolationReadiness`
- S.7 blob surfaces:
  `BlobObjectId`, `BlobGeneration`, authoritative/derived blob
  classification, chunk identity, `LogicalContentDigest`, `StoredChunkDigest`,
  `AuthenticatedFrameDigest`, chunk-tree root, canonical chunking rule,
  `BlobGenerationPublished`, resumable ingest session states, dedupe admission,
  reachability edges, retention holds, reclaim receipts, placement movement,
  blob compaction, export/import bundle surfaces, heavy-blob fixture evidence,
  and sealed `S7NativeBlobStoreCloseout`
- S.7.1 proof-flow cleanup:
  `S7_1ProofFlowCleanupCloseout` or equivalent closeout evidence proving named
  transitions, phase-shaped topology, facade-only public access, and
  certification-as-courtroom boundaries before S.8 implementation begins

### S.8 Surfaces To Introduce

The implementation should introduce Store-owned surfaces equivalent to:

- `PhysicalArtifactFamily`
- `PhysicalArtifactFamilyDeclaration`
- `ArtifactFamilyAuthorityClass`
- `AuthorityRole`
- `ArtifactFamilyLifecycleClass`
- `ArtifactFamilyAccessLane`
- `DerivedAccuracyClass`
- `PhysicalKeyDomain`
- `CanonicalKeyEncoding`
- `ComparatorLaw`
- `PrefixLaw`
- `RangeBoundLaw`
- `HashCollisionLaw`
- `CompositeKeyOrderingLaw`
- `TenantScopedKeyDomain`
- `LayoutStrategyFamily`
- `LayoutStrategyDeclaration`
- `LayoutStrategyCapability`
- `LayoutStrategyInvariantSuite`
- `BTreeLayoutStrategy`
- `BTreeNodeFormatLaw`
- `BTreeSeparatorLaw`
- `BTreeSplitMergeLaw`
- `BTreeRootPublicationLaw`
- `LsmLayoutStrategy`
- `LsmMemtableWalLaw`
- `LsmRunPublicationLaw`
- `LsmTombstoneLaw`
- `LsmCompactionOrderingLaw`
- `LayoutVersion`
- `LayoutCompatibilityWindow`
- `LayoutMigrationPlan`
- `LayoutRollbackPlan`
- `LayoutMaterializationState`
- `PhysicalCoverageBasis`
- `LayoutCoverageWitness`
- `IndexWatermark`
- `CoverageGapWitness`
- `PhysicalAbsenceProof`
- `RangeCompletenessWitness`
- `PrefixCompletenessWitness`
- `LayoutAdmissionRequest`
- `AdmittedLayoutStrategy`
- `LayoutAdmissionDenial`
- `AccessShape`
- `AccessShapeContract`
- `PhysicalMutationShape`
- `AccessLoweringRequest`
- `AdmittedLayoutStrategySet`
- `AccessPathAlternative`
- `DeterministicPlanSelectionPolicy`
- `PlanSelectionReceipt`
- `PlanFingerprint`
- `AccessPlanCostEstimate`
- `AccessPlanBudget`
- `PlannedCounterEnvelope`
- `PlannedVsObservedCounterReceipt`
- `CostEnvelopeViolationOutcome`
- `LoweredAccessPlan`
- `ExecutionReadyAccessPlan`
- `ExecutedAccessPathEvidence`
- `AccessPathCounterSnapshot`
- `AccessPathAmplificationReceipt`
- `DerivedIndexRebuildPlan`
- `DerivedIndexParityWitness`
- `LayoutMutationPlan`
- `IndexMaintenanceMode`
- `IndexPublicationProtocol`
- `IndexLagWitness`
- `IndexMaintenanceFailureOutcome`
- `LayoutCorruptionClassification`
- `LayoutQuarantineWitness`
- `LayoutReadmissionWitness`
- `BootstrapLayoutCatalog`
- `MinimalRootDiscoveryLayout`
- `CatalogReadAdmission`
- `BootstrapOnlyAccessPath`
- `LegacyAccessPathBypassInventory`
- `LegacySurfaceDisposition`
- `ExplicitDegradedExactScan`
- `EphemeralAccessAid`
- `AdvisoryAccessAid`
- `CacheHitEvidence`
- `NeverAuthorityCacheClass`
- `S8LayoutHazardInventory`
- `StoreLayoutPerformanceReceipt`
- `S8DomainSkeletonInventory`
- `S8CrateResponsibilityMap`
- `S8SubsystemTopologyCloseout`
- `S8CrossCrateAuthorityFlowReport`
- `StorageFoundationS9LayoutHandoff`

Names may change, but the conceptual surfaces may not disappear into raw
fields, untyped enums, or certification-only rows.

## Phase 0: Domain Skeleton And Subsystem Boundary Contract

### Purpose

Materialize the standing Domain Skeleton Contract before S.8 adds layout law,
index grammar, or access-path machinery.

This phase is architecture implementation. It creates the selected S.8 skeleton
in code before feature work starts, even where the first modules contain only
sealed vocabulary, facade exports, or closeout placeholders. It must leave the
workspace in a shape where later phases add behavior into already-chosen homes
rather than inventing homes while implementing.

### Relevant APIs

- Current dedicated Store workspace crates:
  `worth-store-contracts`, `worth-store-layout-indexes`,
  `worth-store-physical-format`, `worth-store-wal`,
  `worth-store-recovery-physics`, `worth-store-buffer-pool`,
  `worth-store-physical-integrity`, `worth-store-physical-isolation`,
  `worth-store-io-scheduler`, `worth-store-blob-chunks`,
  `worth-store-security`, `worth-store-operations`,
  `worth-store-offline-verifier`, `worth-store-readiness`,
  `worth-store-physical-certification`, `worth-store-certification`, and
  `worth-store-test-support`
- Current S.7.1 topology evidence:
  `S7_1ProofFlowCleanupCloseout` or equivalent closeout evidence,
  lifecycle-ordered blob exports, certification-as-courtroom boundaries, and
  public facade construction-boundary proofs
- Existing workspace authority surfaces:
  `PhysicalLayoutFamily`, physical-format facade exports, blob lifecycle
  exports, S.5.1 security-scope readiness, S.6 I/O readiness, S.7 closeout,
  certification closeout surfaces, physical-certification harness surfaces,
  and test-support harness authority proofs
- S.8 surfaces to introduce:
  `S8DomainSkeletonInventory`, `S8CrateResponsibilityMap`,
  `S8SubsystemTopologyCloseout`, and `S8CrossCrateAuthorityFlowReport`

### Required Work

Restructure `worth-store-layout-indexes` into the selected target skeleton:

- create every top-level architectural home named in
  `Selected Layout-Indexes Skeleton`
- move the current `PhysicalLayoutFamily` out of the flat seed file and into
  `artifact_family` or `strategy/family` as appropriate
- make `lib.rs` a facade-only aggregation file
- add `facade` modules that export declarations, key-domain law, strategy
  admission, access planning, access execution, maintenance, migration,
  readmission, and closeout groups
- add `skeleton` modules that define `S8DomainSkeletonInventory`,
  `S8CrateResponsibilityMap`, `S8SubsystemTopologyCloseout`,
  `S8CrossCrateAuthorityFlowReport`, and `S8PhaseSkeletonObligation`
- add compile-fail entrypoint modules for raw construction, facade bypass, and
  certification authority misuse

Create the `layout_access` home in each owning family crate listed in
`Required Family-Crate S.8 Homes`. Phase 0 does not need to implement every
family behavior, but each home must compile, expose its closeout placeholder,
and declare which S.8 grammar modules it will consume.

Create the courtroom/test skeleton under the milestone-specific roots
specified here:

- `worth-store-physical-certification/src/harness/by_milestone/s8_layout_access`
- `worth-store-certification/src/s8_layout_closeout`
- `worth-store-test-support/src/harness/milestone/s8_layout_access`

Add one code-level responsibility map that is opinionated, not generated from
whatever files happen to exist. It must classify each relevant crate as one of:

- `SharedContractVocabulary`
- `LayoutAccessGrammar`
- `FamilyExecutionAuthority`
- `ReadinessHandoff`
- `TerminalOfflineObservation`
- `PhysicalCertificationCourtroom`
- `CertificationCloseoutCourtroom`
- `HonestTestFixtureSupport`
- `SpecializedConsumer`

Each responsibility-map row must name the crate, its primary responsibility,
the authority it may mint, the authority it may consume, its allowed projection
outputs, its public S.8 facade home, and the S.8 phases expected to touch it.

Add a phase-obligation object that later phases must update when they add S.8
work. The obligation must record:

- phase number
- owning crate
- owning module path
- public facade path
- consumed authority
- minted authority
- courtroom/test-support boundary
- compile-fail or runtime shortcut proof

The New Crate Rule remains active, but Phase 0 should not create a new crate.
The selected architecture already has a crate for S.8 grammar
(`worth-store-layout-indexes`) and family crates for execution authority.

### Tests

- Skeleton inventory tests prove every crate in the responsibility map has one
  primary role and no ambiguous "misc/support/proof bag" role.
- Topology tests prove every selected `worth-store-layout-indexes` directory
  exists, is wired through the facade where appropriate, and keeps `lib.rs`
  aggregation-only.
- Family-home tests prove every required `layout_access` module exists in the
  owning family crates and exposes a closeout placeholder.
- Boundary tests or compile-fail coverage prove certification/test-support
  cannot construct production S.8 authority.
- Dependency-direction tests or scan checks prove `worth-store-layout-indexes`
  does not depend on certification/test-support and family crates do not depend
  on certification as production law.
- Public-export tests prove S.8 skeleton surfaces are reachable through
  lifecycle-shaped facades, not only broad `lib.rs` dumps.
- Spec conformance tests or closeout assertions prove every later phase can
  reference the skeleton inventory and must update it if the crate map changes.

### Closeout Gate

This phase is done when the selected architectural homes physically exist in
the workspace, compile, have facade wiring, have responsibility-map tests, and
give later phases clear homes for their work. It is not done if it only
produces an inventory, report, checklist, markdown explanation, or broad
`mod.rs` wrapper.

## Phase 1: Durable Artifact Family Inventory

### Purpose

Create the Store-owned vocabulary for durable artifact families before any
layout is admitted.

### Relevant APIs

- Existing Store inventory inputs: `WorthStore`, `StoreCounterSnapshot`,
  `StorageFoundationS1Handoff`, `EvidenceBundleReadiness`,
  `RoadmapSequenceStatusMatrix`, `S0StableDigest`
- Existing artifact families to classify: `WalRecordFamily`,
  `CompatibilityFamilyKind`, `MaintenanceArtifactFamily`,
  `SupportArtifactFamily`, `PlacementArtifactFamily`,
  `PublicationFamily`, `DerivedFamilyRetentionPolicy`,
  `LayoutFamilyCompactionUnit`
- Planned S.1/S.7 family nouns:
  `PhysicalPageId`, `PhysicalSegmentId`, `PhysicalExtentId`,
  `PhysicalRootManifest`, `BlobObjectId`, `BlobGeneration`, chunk-tree root,
  reachability edge, retention hold, and reclaim receipt families
- S.8 surfaces to introduce: `PhysicalArtifactFamily`,
  `PhysicalArtifactFamilyDeclaration`, `ArtifactFamilyAuthorityClass`,
  `ArtifactFamilyLifecycleClass`, `ArtifactFamilyAccessLane`
- Proof surfaces: `recipe(...)`, `.resolve_with(...)`, `TransitionOutcome`

### Required Work

Define typed inventory surfaces for physical artifact families, including at
least:

- page, segment, extent, and root manifest artifacts
- WAL, checkpoint, replay, and recovery record artifacts
- blob chunk, blob manifest, blob stream, and chunk-tree artifacts
- dedupe, reachability, retention, reclaim, placement, and residency artifacts
- corruption, quarantine, repair, and readmission artifacts
- security/custody lookup artifacts when physical access depends on them
- export, import, capsule, and offline verification artifacts when durable

Each family must declare:

- whether it is authoritative, derived, diagnostic, terminal, or certification
  evidence
- whether it is hot-path, maintenance-path, verifier-path, or terminal-path
- whether it can be rebuilt, partially rebuilt, migrated, rolled back, or only
  quarantined
- which lower-crate boundary owns the production contract

### Implementation Shape

The expected shape is a small Store-owned vocabulary module, not a broad bag of
layout names:

```rust
let family = declare_physical_artifact_family(input);
let authority = classify_artifact_family_authority(&family);
let lifecycle = classify_artifact_family_lifecycle(&family);
let declaration = build_artifact_family_declaration(family, authority, lifecycle);
```

### Tests

- Every durable family currently admitted by Store has an explicit declaration.
- Derived families cannot be used as authority in compile-fail tests.
- Diagnostic, terminal, and certification artifacts cannot satisfy production
  layout admission.
- Missing family declarations produce a typed denial before access-path lowering.

### Closeout Gate

This phase is done when reviewers can list Store's durable physical families
without reading implementation predicates or certification harness code.

## Phase 2: Artifact Authority And Lifecycle Classification

### Purpose

Separate inventory from authority classification. Phase 1 names durable
families; this phase proves what each family is allowed to mean before any
layout strategy can be admitted.

### Relevant APIs

- Existing authority and projection inputs:
  `AuthoritativeExportBundle`, `VerifiedAuthoritativeAppend`,
  `PersistedAuthoritativeCommit`, `Milestone6LayoutMaterialization`,
  `CompatibilityDerivedRebuildOutcome`, `PublishedCompactionProduct`,
  `SubscriptionSupportAccessStructure`, `PlacementNonAuthorityWitness`
- Existing classification surfaces:
  `CompatibilityAuthorityClassification`, `CompatibilityFamilyKind`,
  `DerivedCompatibilityLaneKind`, `PublicationFamily`,
  `MaintenanceDeclarationClass`, `SupportTrustClass`
- Planned S.1/S.7 authority nouns:
  `PhysicalRootManifest`, segment/extent manifests, `BlobGenerationPublished`,
  authoritative blob classification, derived blob classification
- S.8 surfaces to introduce:
  `ArtifactFamilyAuthorityClass`, `ArtifactFamilyLifecycleClass`,
  `ArtifactFamilyAccessLane`, `ArtifactFamilyAuthorityWitness`
- Proof surfaces:
  `AuthorityWitness`, `Proof<...>`, `TransitionOutcome::denied(...)`

### Required Work

For every `PhysicalArtifactFamilyDeclaration`, classify:

- source truth versus derived projection
- diagnostic or terminal evidence versus production authority
- foreground, background, verifier, repair, export, or certification lane
- rebuildable, partially rebuildable, migrated, readmitted, quarantined, or
  non-rebuildable lifecycle posture
- whether a stale version can be read, must rebind, or must deny

This phase must produce the proof-bearing authority classification that later
strategy admission consumes. Later phases must not reclassify authority from
raw family names.

### Tests

- Authority parity: destroying all derived families and rebuilding them from
  declared authoritative sources yields the same derived access answers,
  physical coverage witnesses, declared accuracy classes, and parity
  witnesses. Family classification remains stable because it is
  declaration-derived, not projection-derived.
- Authority denial: a derived, diagnostic, terminal, or certification family
  cannot satisfy APIs that require production authority.
- Lifecycle denial: a stale or quarantined family cannot enter layout strategy
  admission without the required rebind or readmission witness.

### Closeout Gate

This phase is done when every durable family has one Store-owned authority and
lifecycle classification, and later phases consume that classification rather
than reconstructing it from names, counters, or records.

## Phase 3: Authority Roles, Accuracy Classes, And Scope Partitioning

### Purpose

Refine "authoritative versus derived" into the physical roles Store actually
needs before any layout strategy is declared.

### Relevant APIs

- Existing authority-adjacent surfaces:
  `PhysicalRootManifest`, `WalRecord`, `RecoverySourceReport`,
  `SnapshotImageBundle`, `TierPlacementEvidence`,
  `FoundationalCounterBackedPerformanceReceipt`,
  `AuthoritativeExportBundle`, `SupportTrustAccessPath`
- Planned S.5.1 metadata:
  key scope, key version, tenant scope, authenticity class, custody posture,
  repair blast-radius readiness, and import/readmission scope witnesses
- S.8 surfaces to introduce:
  `AuthorityRole`, `DerivedAccuracyClass`, `TenantScopedKeyDomain`,
  `ArtifactFamilyAuthorityWitness`, and scope-partitioned family declarations
- Proof surfaces:
  law 41 proof-bearing wrappers, `AuthorityWitness`,
  `TransitionOutcome::denied(...)`

### Required Work

Every physical family and strategy must declare its authority role, not only
whether it is broadly authoritative or derived. Roles include:

- semantic authority consumer, but not semantic authority owner
- physical discovery authority
- allocation authority
- recovery authority
- custody evidence authority
- performance evidence authority
- terminal transport evidence
- certification evidence

Every derived family must also declare its accuracy class:

- exact
- conservative
- approximate
- heuristic
- advisory

Accuracy class must flow into family declarations, strategy capabilities,
access shape contracts, access lowering, parity witnesses, and absence proofs.
Approximate, heuristic, or advisory structures may guide planning only under
explicit rules; they may not produce exact absence, exact reachability, delete
authority, reclaim authority, custody authority, or production read authority.

S.5.1 scope metadata must not be isolated to security lookup families. Every
ordinary physical layout strategy must declare whether it is single-tenant,
tenant-partitioned, cross-tenant with admitted equivalence, single-key-scope,
cross-key-scope denied, or cross-key-scope admitted under a named policy.

### Tests

- Accuracy denial: approximate/advisory structures cannot produce exact
  absence, exact reachability, reclaim authority, custody authority, or
  production read authority.
- Scope denial: a shared B-tree, hash index, chunk lookup, range map, or dedupe
  map cannot mix tenant/key scopes without an admitted scope-partition policy.
- Authority-role parity: root manifest physical discovery authority does not
  become semantic truth, and WAL recovery authority does not become final
  semantic authority.

### Closeout Gate

This phase is done when authority role, derived accuracy class, and tenant/key
scope posture are carried by every later layout and access proof type.

## Phase 4: Physical Key-Domain Law

### Purpose

Define how Store physical keys compare, encode, prefix-match, hash, and bound
ranges before any ordered or hashed strategy can be admitted.

### Relevant APIs

- Planned S.1/S.5.1/S.7 inputs:
  page ids, segment ids, extent ids, physical references, tenant scope, key
  scope, blob object ids, blob generations, chunk identity, root manifest keys
- Existing key-like surfaces:
  `ArtifactFamilyId`, `ArtifactFormatVersion`, `PhysicalChunkId`,
  `StructuralBlockId`, `DurableMutationId`, `WalRecordFamily`
- S.8 surfaces to introduce:
  `PhysicalKeyDomain`, `CanonicalKeyEncoding`, `ComparatorLaw`, `PrefixLaw`,
  `RangeBoundLaw`, `HashCollisionLaw`, `CompositeKeyOrderingLaw`,
  `TenantScopedKeyDomain`
- Foundational/proof surfaces:
  canonicalization readiness for boundary movement only; Store-owned key law
  remains the production authority

### Required Work

Define physical key-domain law for point, range, prefix, hash/equality,
composite, and tenant/key-scoped access. The law must specify:

- byte encoding and version bytes
- field order for composite keys
- null/sentinel/end-bound representation where relevant
- comparator behavior
- inclusive and exclusive range-bound behavior
- prefix boundary behavior
- hash collision behavior and collision verification
- tenant/key-scope prefixes or partitioning
- canonical ordering stability across restart, migration, and import

Range and prefix access are not legal claims until the relevant key-domain law
is admitted. Hash/equality access is not legal until collision behavior is
declared and tested.

### Tests

- Comparator parity: canonical key ordering is identical across restart,
  migration replay, and certification replay for the same key domain.
- Boundary denial: a range or prefix lookup cannot lower without admitted
  range-bound and prefix law.
- Collision denial: hash/equality lookup cannot claim exact identity without
  collision verification or a declared impossible-collision basis.
- Scope denial: tenant/key-scope prefixes must prevent cross-scope range,
  prefix, and hash reuse unless an admitted equivalence policy exists.

### Closeout Gate

This phase is done when range, prefix, hash, and composite access have physical
key law instead of relying on ad hoc byte or string comparisons.

## Phase 5: Strategy Invariant Suites And Baseline B-Tree/LSM Algorithms

### Purpose

Make B-tree and LSM claims real physical algorithms instead of vague strategy
labels.

### Relevant APIs

- Planned S.1/S.2/S.3/S.4/S.5 inputs:
  page/frame formats, buffer leases, checksums, WAL/page-LSN records,
  checkpoint manifests, stable physical read plans, and crash-replay evidence
- S.8 surfaces to introduce:
  `LayoutStrategyInvariantSuite`, `BTreeLayoutStrategy`,
  `BTreeNodeFormatLaw`, `BTreeSeparatorLaw`, `BTreeSplitMergeLaw`,
  `BTreeRootPublicationLaw`, `LsmLayoutStrategy`, `LsmMemtableWalLaw`,
  `LsmRunPublicationLaw`, `LsmTombstoneLaw`, `LsmCompactionOrderingLaw`
- Proof/counter surfaces:
  `TransitionOutcome`, `AccessPathCounterSnapshot`,
  `PlannedCounterEnvelope`, `PlannedVsObservedCounterReceipt`

### Required Work

S.8 must implement minimal but real baseline B-tree and LSM strategy families.
The milestone does not need every future optimization, but it must establish
the concrete algorithmic core.

The B-tree baseline must define:

- page/node format
- separator key law
- search and insertion path
- split behavior
- root publication behavior
- sibling-link posture, including explicit absence if not used
- occupancy constraints
- tombstone posture, if any
- stable-read behavior
- checksum scope and corruption localization
- rebuild and migration behavior

The LSM baseline must define:

- memtable/WAL relationship
- immutable sorted-run publication
- run lookup and merge order
- tombstone behavior
- manifest update protocol
- compaction ordering
- filter/bloom/advisory authority class if present
- stale run cleanup
- write-amplification accounting
- crash-safe publication and replay behavior

No strategy may be admitted because it merely advertises range or write
optimization. It must present an invariant suite that makes lookup, mutation,
corruption, rebuild, migration, and counter expectations mechanically testable.

### Tests

- B-tree invariant tests cover separator ordering, split/root publication,
  checksum localization, stable read behavior, and replay after interruption.
- LSM invariant tests cover WAL/memtable recovery, sorted-run lookup,
  tombstone preservation, manifest publication, compaction ordering, and stale
  run cleanup.
- Strategy denial: B-tree and LSM declarations without concrete baseline
  algorithms and invariant suites cannot enter layout admission.
- Counter parity: baseline B-tree and LSM lookups produce planned and observed
  counter receipts tied to the declared strategy.

### Closeout Gate

This phase is done when B-tree and LSM are concrete Store-owned baseline
strategies with tested invariants, not names over future work.

## Phase 6: Materialization, Coverage, Freshness, And Absence Proofs

### Purpose

Model the state and coverage of a specific layout instance before access plans
can claim exact answers.

### Relevant APIs

- Existing and planned state surfaces:
  `Milestone6LayoutMaterialization`, `SnapshotImageBundle`,
  `StableBasisReadPlan`, `RecoverySourceReport`, `BlobGenerationPublished`,
  checkpoint manifests, root epochs, WAL LSNs, and branch/snapshot bases
- S.8 surfaces to introduce:
  `LayoutMaterializationState`, `PhysicalCoverageBasis`,
  `LayoutCoverageWitness`, `IndexWatermark`, `CoverageGapWitness`,
  `PhysicalAbsenceProof`, `RangeCompletenessWitness`,
  `PrefixCompletenessWitness`
- Proof surfaces:
  freshness outcomes, rebind/readmission transitions, and exact/non-exact
  transition variants

### Required Work

Every specific layout instance must expose materialization state:

- declared only
- absent
- empty initialized
- building
- partially covered
- exact
- exact through physical basis
- lagged
- stale
- rebuild required
- migrating
- quarantined
- retired

Coverage must be physical and explicit. Examples include exact-through-LSN,
exact-through-root-epoch, exact-through-blob-generation, exact-through-checkpoint,
and partially covered range. Store does not decide semantic visibility here,
but it must honestly report the physical basis.

Absence must be proof-bearing. "Not found" from an exact index, "not found"
from a partially built projection, "not found" after an allowed bounded scan,
and "not found while the relevant range is quarantined" are different outcome
classes.

### Tests

- Absence denial: partially covered, quarantined, or stale layouts cannot
  return exact absence.
- Coverage parity: exact-through-LSN/root-epoch/blob-generation witnesses
  survive reopen and certification replay.
- Freshness denial: a correct format version with stale physical coverage must
  return stale/rebind/lagged evidence, not success.
- Gap localization: a coverage gap identifies the physical range, basis, and
  family that prevent exact access.

### Closeout Gate

This phase is done when exact read, exact absence, and exact parity are
impossible without materialization and coverage witnesses.

## Phase 7: Layout Strategy Vocabulary

### Purpose

Define the set of physical layout strategies Store can admit.

### Relevant APIs

- Existing layout surfaces:
  `AspectLayoutReadRequest`, `AdmittedAspectLayoutReadPlan`,
  `RejectedAspectLayoutReadPlan`, `AspectReadRegime`,
  `AspectLayoutFallbackClass`, `AspectLayoutPerformanceEnvelope`,
  `ChunkModelFrozenPhysicalLayout`, `StructuralBlockLookup`,
  `DedupAdmittedBlockReuse`
- Existing compatibility surfaces:
  `CompatibilityFamilyDeclaration`, `CompatibilityRegistry`,
  `ArtifactFormatVersion`, `ArtifactCompatibilityWindow`,
  `MixedVersionStorePosture`
- Foundational surfaces:
  `worth_foundational::performance_api::common_path::performance()`,
  `FoundationalLayoutIntentClaim`
- S.8 surfaces to introduce: `LayoutStrategyFamily`,
  `LayoutStrategyDeclaration`, `LayoutStrategyCapability`,
  `PhysicalKeyDomain`, `CanonicalKeyEncoding`, `ComparatorLaw`, `PrefixLaw`,
  `RangeBoundLaw`, `LayoutStrategyInvariantSuite`, `BTreeNodeFormatLaw`,
  `BTreeSeparatorLaw`, `BTreeSplitMergeLaw`, `BTreeRootPublicationLaw`,
  `LsmMemtableWalLaw`, `LsmRunPublicationLaw`, `LsmTombstoneLaw`,
  `LsmCompactionOrderingLaw`, `LayoutVersion`, `LayoutMaterializationState`,
  `LayoutAdmissionRequest`, `AdmittedLayoutStrategy`,
  `LayoutAdmissionDenial`, `PlannedCounterEnvelope`
- Proof surfaces: `Recipe<Resolved, T>`, `.lower_with(...)`,
  `.admit_with(...)`, `ProofOutcomeKind`

### Required Work

Introduce explicit strategy families such as:

- append log
- heap or slotted-page file
- page table
- baseline B-tree range structure
- baseline LSM write-optimized structure
- sparse index
- chunk tree
- manifest table
- bitmap, free-space map, or allocation map
- hash/equality index
- range map
- quarantine or corruption map
- streaming cursor index

Each strategy must declare:

- supported access shapes
- required physical key domains
- canonical key encoding and comparator law for any ordered or equality access
- prefix and range-bound law for prefix/range access
- required invariant suite
- B-tree laws when the strategy is B-tree based:
  node format, separator law, split/merge law, and root publication law
- LSM laws when the strategy is LSM based:
  memtable/WAL law, immutable run publication law, tombstone law, and
  compaction ordering law
- expected locality properties
- expected read and write amplification dimensions
- pre-execution counter envelope dimensions
- materialization-state compatibility
- rebuild source requirements
- corruption isolation behavior
- versioning and migration posture
- whether it is allowed on foreground, background, verifier, or terminal lanes

### Foundational And Proof Usage

Store may emit a `FoundationalLayoutIntentClaim` for shared boundary language,
but the Store-owned `LayoutStrategyDeclaration` is the production contract.
Worth Proof should encode declaration resolution and admission as separate
states.

### Tests

- A family cannot select an unsupported strategy.
- A strategy cannot advertise point, range, prefix, scan, or streaming support
  unless it declares the required key law, invariant suite, and counters.
- A terminal/report-only strategy cannot become hot-path execution authority.
- A strategy declaration without planned counter envelope fields cannot enter
  admission even before the counter phase deepens executed counter evidence.
- Compile-fail tests prove external crates cannot construct admitted layout
  strategies from raw fields.

### Closeout Gate

This phase is done when every allowed strategy has explicit access, key-domain,
invariant, cost, rebuild, corruption, and migration semantics.

## Phase 8: Layout Strategy Admission Registry

### Purpose

Turn strategy vocabulary into an admission registry that can reject unsupported
family/strategy combinations before access lowering exists.

### Relevant APIs

- Existing layout and compatibility inputs:
  `CompatibilityRegistry`, `CompatibilityFamilyDeclaration`,
  `CompatibilityAdmissionPlan`, `CompatibilityAdmissionReceipt`,
  `AspectLayoutPerformanceEnvelope`, `Milestone6AccessStructureContract`,
  `Milestone7AccessStructureContract`
- Foundational performance inputs:
  `FoundationalLayoutIntentClaim`,
  `FoundationalPolicyAdmissionReceipt`,
  `worth_foundational::performance_api::lower_lane::policy`
- S.8 surfaces to introduce:
  `LayoutAdmissionRequest`, `AdmittedLayoutStrategy`,
  `LayoutAdmissionDenial`, `LayoutStrategyCapability`,
  `LayoutStrategyRegistrySnapshot`, `LayoutMaterializationState`,
  `LayoutCoverageWitness`, `PhysicalAbsenceProof`, `AccessPlanBudget`,
  `IndexMaintenanceMode`, `PhysicalMutationShape`
- Proof surfaces:
  `.admit_with(...)`, `.try_admit_ready(...)`,
  `TransitionOutcome::{Success, Denied, Deferred}`

### Required Work

Build a registry that admits strategy declarations only after it can prove:

- the artifact family authority class allows the strategy
- the lifecycle class allows the strategy
- the strategy supports each requested access shape
- the required physical key-domain law exists
- the required invariant suite exists
- canonical key encoding, comparator, prefix, range-bound, and hash-collision
  law are compatible with the requested access shapes
- the requested strategy can produce the required materialization and coverage
  evidence for its authority role
- the strategy can produce the required physical absence proof class for any
  exact-not-found claim it advertises
- required counters and rebuild sources are declared
- pre-execution budget fields exist for planned access
- tenant/key/custody scope partitioning is compatible with the family and
  admitted access lane
- mutation and maintenance modes are compatible with the strategy's invariant
  suite and publication protocol
- version and migration posture are compatible with the family
- foreground/background/verifier/repair/terminal lane restrictions are honored

### Tests

- Admission parity: the same family declaration and strategy declaration admit
  deterministically across restart and certification replay.
- Admission denial: copied strategy rows, layout intent claims, policy receipts,
  or report rows cannot mint `AdmittedLayoutStrategy`.
- Capability denial: a strategy without streaming support cannot be admitted
  for blob streaming, and a strategy without range support cannot be admitted
  for ordered traversal.
- Key-law denial: a range, prefix, hash/equality, or composite-key strategy
  cannot admit unless the corresponding key-domain law is compatible with the
  family and requested lane.
- Materialization denial: a strategy that cannot produce the required
  materialization, coverage, or absence proof cannot advertise exact access.
- Scope denial: tenant/key/custody partitioning failures deny admission before
  lowering, even for ordinary B-tree, LSM, hash, manifest, chunk, or dedupe
  indexes.

### Closeout Gate

This phase is done when no later access phase can choose a strategy directly;
it must consume `AdmittedLayoutStrategy` or a stronger Store-owned proof.

## Phase 9: Customization Boundary For Future Higher Layers

### Purpose

Create a narrow customization boundary without guessing user or industry
indexes.

### Relevant APIs

- Existing higher-layer-adjacent surfaces:
  `AspectLayoutTarget`, `AspectProjectionSet`, `AspectScopeClass`,
  `StableBasisReadRequest`, `StableBasisReadPlan`,
  `SubscriptionSupportAccessStructure`, `SupportTrustAccessPath`,
  `SupportTrustAccessStructurePlan`
- Foundational aspect surfaces:
  `worth_foundational::aspects()`, `AspectKey`, `StructAspectValue`,
  `ContractValidatedAspectValue`, projection/mutation/diagnostic mask
  admission surfaces
- S.8 surfaces to introduce: `AccessShape`, `AccessShapeContract`,
  `LayoutStrategyCapability`, `LayoutAdmissionDenial`
- Proof surfaces: `CapabilityWitness`, `TransitionOutcome::denied(...)`,
  `TransitionOutcome::deferred(...)`

### Required Work

Define typed request vocabulary for future higher layers to ask for physical
capabilities without directly choosing unsafe implementation details.

Examples:

- point lookup over a declared key domain
- ordered range traversal over a declared key domain
- prefix traversal over a declared prefix domain
- streaming over a declared chunk or segment domain
- rebuildable secondary projection over a declared authority source
- verifier-only scan over a declared corpus

The boundary must make clear that higher layers may request access shapes and
workload envelopes, but Store admits the actual physical strategy.

### Tests

- A higher-layer request cannot inject a custom comparator, serializer, or scan
  callback as authority.
- A request for range access cannot be silently served by a whole-family scan
  unless the verifier/diagnostic lane is explicit.
- Unsupported capability requests return typed denials with the missing layout
  fact visible.

### Closeout Gate

This phase is done when S.8 gives future query/security/application layers an
extension point without turning physical layout into an untyped plugin system.

## Phase 10: Access Shape Contracts

### Purpose

Name access shapes before any lowering or execution path exists.

### Relevant APIs

- Existing Store access surfaces:
  `AspectLayoutReadRequest`, `AdmittedAspectLayoutReadPlan`,
  `AspectLayoutReadExecutionDecision`, `AspectLayoutReadExecutionResult`,
  `BranchDeltaReadRequest`, `BranchDeltaReadPlan`, `SnapshotReadRequest`,
  `SnapshotReadResult`, `PlacementResolvedReadHandle`, `ResidentReadLease`,
  `ColdRecallLease`
- Planned S.1/S.6/S.7 inputs:
  `PlatformPhysicalFacade`, physical read leases, stable physical read plans,
  `S6IoQosIsolationReadiness`, blob streaming cursor/resume indexes, chunk-tree
  root
- S.8 surfaces to introduce: `AccessShape`, `AccessShapeContract`,
  `PhysicalMutationShape`, `AccessShapeUnsupportedDenial`,
  `AccessLaneClassification`, `ExplicitDegradedExactScan`
- Proof surfaces: `CapabilityWitness`, `TransitionOutcome::denied(...)`,
  `TransitionOutcome::deferred(...)`

### Required Work

Define access shape contracts for:

- point lookup
- batch point lookup
- sorted batch lookup
- range lookup
- multi-range lookup
- prefix lookup
- grouped prefix lookup
- coalesced page read
- chunk-tree walk
- manifest graph walk
- bounded scan
- full declared scan
- streaming read
- streaming continuation read
- append
- compaction read
- rebuild read
- verifier read
- repair/quarantine read

Mutation shapes are not ordinary read access shapes. Update-in-place must live
under `PhysicalMutationShape` and requires WAL-before-data proof, page latch or
stable-read exclusion proof, page-LSN update behavior, checksum rewrite
behavior, crash replay behavior, and torn-write handling before a family may
admit it.

Explicit degraded exact scans are legal only as caller-visible, budgeted,
counter-backed outcomes. They cannot be selected as silent fallback for point,
range, prefix, streaming, or locality-bounded APIs.

Lowering must bind:

- artifact family
- admitted layout strategy
- layout version
- authority or derivation posture
- access shape
- foreground/background/verifier/terminal lane
- expected counter dimensions
- stale/rebind behavior

### Tests

- An unsupported access shape returns `Denied`, not a fallback scan.
- A verifier, rebuild, repair, or terminal access shape cannot be called from a
  foreground API without an explicit lane contract.
- Batch/coalescing denial: many legal point lookups cannot be lowered as a
  loop of scalar accesses when a batch, sorted-batch, multi-range, grouped
  prefix, coalesced page, chunk-tree walk, or manifest walk shape is required.
- Mutation denial: update-in-place cannot be admitted without the stronger
  physical mutation proof requirements.
- Shape parity: equivalent point/range/prefix/scan/streaming requests classify
  to the same access shape across replay.

### Closeout Gate

This phase is done when every access request first becomes a named shape with
declared lane, cost dimensions, and unsupported-denial behavior.

## Phase 11: Deterministic Plan Selection And Pre-Execution Budget Admission

### Purpose

Choose between admitted physical alternatives before execution and deny unsafe
plans before they run.

### Relevant APIs

- Existing planning/cost surfaces:
  `AspectLayoutPerformanceEnvelope`, `RecallAmplificationBudget`,
  `Milestone6CounterContract`, `Milestone7CounterContract`,
  `StoreCounterSnapshot`, `S6IoQosIsolationReadiness`
- S.8 surfaces to introduce:
  `AdmittedLayoutStrategySet`, `AccessPathAlternative`,
  `DeterministicPlanSelectionPolicy`, `PlanSelectionReceipt`,
  `PlanFingerprint`, `AccessPlanCostEstimate`, `AccessPlanBudget`,
  `PlannedCounterEnvelope`, `CostEnvelopeViolationOutcome`,
  `EphemeralAccessAid`, `AdvisoryAccessAid`, `CacheHitEvidence`,
  `NeverAuthorityCacheClass`
- Foundational/proof surfaces:
  performance readiness and counter-backed receipts after Store execution,
  not before; `TransitionOutcome::Deferred` and denied/degraded outcomes for
  budget or envelope failures

### Required Work

When more than one admitted strategy can answer a physical access shape, S.8
must produce a deterministic plan-selection receipt before lowering. Selection
must name:

- candidate strategies and their authority roles
- why each candidate is eligible or rejected
- deterministic tie-break or priority rule
- selected strategy
- plan fingerprint
- planned counter envelope
- memory, page, chunk, range, and byte budget
- allowed degraded exact scan posture, if any
- ephemeral or advisory access aids, if any, including why they are never
  authority

The executor must consume the selected, budget-admitted plan. It must not
re-run planner logic or silently broaden when observed counters exceed the
planned envelope. Exceeding the envelope must produce a distinct
`CostEnvelopeViolationOutcome`, `Deferred`, or typed denial.

Buffer-pool state, read-ahead state, hotness maps, bloom filters, plan caches,
and ephemeral cursors may participate only as declared access aids. A cache hit
must carry `NeverAuthorityCacheClass` or equivalent proof that it accelerates a
declared path without replacing the path's authority, coverage, or absence
proof.

### Tests

- Selection parity: the same strategy set, request, and physical basis produce
  the same plan fingerprint across restart, migration replay, and
  certification replay.
- Budget denial: a plan whose estimated page/chunk/range/byte budget exceeds
  the admitted envelope denies or defers before execution.
- Broadening denial: a selected point/range/prefix/streaming plan cannot widen
  into an undeclared scan when its primary path is unavailable.
- Cache denial: cache hits, bloom filters, hotness maps, and plan caches cannot
  answer as production authority or exact absence without the underlying
  admitted path proof.
- Regression guard: plan fingerprints detect unexpected strategy-selection
  drift after format migration or index rebuild.

### Closeout Gate

This phase is done when execution receives one deterministic, budget-admitted
plan instead of a set of tempting alternatives.

## Phase 12: Access Lowering And Proof Progression

### Purpose

Make every physical access path pass through a typed lowering and readiness
chain after shape classification.

### Relevant APIs

- Existing Store access surfaces:
  `AspectLayoutReadRequest`, `AdmittedAspectLayoutReadPlan`,
  `AspectLayoutReadExecutionDecision`, `AspectLayoutReadExecutionResult`,
  `BranchDeltaReadRequest`, `BranchDeltaReadPlan`, `SnapshotReadRequest`,
  `SnapshotReadResult`, `PlacementResolvedReadHandle`, `ResidentReadLease`,
  `ColdRecallLease`
- Planned S.1/S.6/S.7 inputs:
  `PlatformPhysicalFacade`, physical read leases, stable physical read plans,
  `S6IoQosIsolationReadiness`, blob streaming cursor/resume indexes, chunk-tree
  root
- S.8 surfaces to introduce: `AccessLoweringRequest`, `PlanSelectionReceipt`,
  `AccessPlanBudget`, `PlannedCounterEnvelope`, `LayoutCoverageWitness`,
  `LoweredAccessPlan`, `ExecutionReadyAccessPlan`,
  `ExecutedAccessPathEvidence`
- Proof surfaces:
  `Recipe<Unresolved, T>`, `Recipe<Resolved, T>`, `Recipe<Lowered, T>`,
  `ExecutionReadyRecipe<T, A>`, `ExecutedRecipe<T, A>`,
  `.ready_with(...)`, `.execute()`, `FreshnessTransitionOutcome`

### Required Work

Implement the proof progression:

```rust
let declaration = resolve_layout_declaration(family)?;
let admitted = admit_layout_strategy(declaration)?;
let access_shape = classify_access_shape(admitted, request)?;
let alternatives = enumerate_admitted_access_path_alternatives(access_shape)?;
let selected = select_access_path_deterministically(alternatives, policy)?;
let budgeted = admit_access_plan_budget(selected, workload_envelope)?;
let coverage = prove_layout_coverage(budgeted)?;
let access_plan = lower_access_shape(coverage)?;
let ready = prepare_access_execution(access_plan)?;
let executed = execute_access_path(ready)?;
let receipt = compare_planned_vs_observed(budgeted, executed)?;
```

The execution API may consume only `ExecutionReadyAccessPlan` or a stronger
type. It must not re-decide layout strategy, lane, broad-scan fallback,
version posture, materialization state, coverage basis, budget envelope, or
counter dimensions. Planned-versus-observed comparison is part of the proof
chain, not a diagnostics afterthought.

### Tests

- Stage denial: unresolved, resolved, admitted, or lowered access artifacts
  cannot be passed to execution APIs.
- Freshness denial: stale layout bindings return `RebindRequired`, not success
  or generic failure.
- Coverage denial: lowered plans without exact-enough materialization and
  coverage for their claimed outcome cannot become execution-ready.
- Cost denial: a plan that has no admitted budget or planned counter envelope
  cannot become execution-ready.
- Replay honesty: executing the same admitted access plan against the same
  authority basis yields the same executed evidence and counter dimensions.

### Closeout Gate

This phase is done when access execution is impossible without a lowered,
execution-ready access plan.

## Phase 13: Rebuild Source And Derived Index Consistency

### Purpose

Make derived structures honest about where they come from and how they recover.

### Relevant APIs

- Existing derived/rebuild surfaces:
  `Milestone6DerivedArtifactRebuildReport`,
  `CompatibilityDerivedRebuildRequest`,
  `CompatibilityDerivedRebuildOutcome`, `DerivedRebuildRequirement`,
  `LoweredRebuildDeclaration`, `RebuildDebtSummary`,
  `SupportRebuildAdmissionWitness`, `SupportRebuildEquivalenceWitness`
- Existing authority inputs:
  `AuthoritativeExportBundle`, `VerifiedAuthoritativeAppend`,
  `PhysicalRootManifest`, `WalRecord`, `SnapshotImageBundle`,
  `BlobGenerationPublished`
- S.8 surfaces to introduce: `DerivedIndexRebuildPlan`,
  `DerivedIndexParityWitness`, `LayoutCorruptionClassification`
- Proof surfaces: `Proof<...>`, `AuthorityWitness`, `UniqueVec<T>` for parity
  sets where order/uniqueness matters

### Required Work

For every derived index or layout projection, define:

- canonical authority source
- optional physical authority source if the family is explicitly authoritative
- rebuild scope
- partial rebuild key space
- parity check strategy
- corruption classification
- result identity after rebuild

Derived rebuild must preserve the authority/projection distinction. A rebuilt
projection may prove parity with authority, but it does not become authority
unless the family declaration already allows that.

### Tests

- Corrupt derived indexes rebuild to parity with authority.
- Corrupt authoritative artifacts quarantine or deny rather than silently
  rebuilding from projection data.
- Rebuild from certification rows, reports, logs, or JSON is rejected.
- Rebuild parity checks verify key identity, value identity, ordering identity,
  coverage identity, and declared cost-envelope compliance. Exact counter
  identity is required only for strategy families that explicitly claim
  deterministic physical shape.

### Closeout Gate

This phase is done when every derived access structure has a visible rebuild
source and a visible parity proof.

## Phase 14: Live Index Maintenance And Publication Protocols

### Purpose

Define how physical projections remain honest while writes, compaction,
migration, and background maintenance are happening.

### Relevant APIs

- Existing maintenance inputs:
  `LoweredRetentionMaintenanceBatch`, `LoweredCompactionDeclaration`,
  `LoweredRebuildDeclaration`, `CompactionPlan`, `CompactionCutoverWitness`,
  `PublishedCompactionProduct`, `ReclaimExecutionReport`
- Planned S.4/S.5/S.6/S.7 inputs:
  WAL/page-LSN evidence, checkpoint manifests, stable physical read plans,
  reclaim barriers, I/O scheduler readiness, blob generation publication,
  reachability edges, retention holds, and compaction evidence
- S.8 surfaces to introduce:
  `LayoutMutationPlan`, `IndexMaintenanceMode`,
  `IndexPublicationProtocol`, `IndexLagWitness`,
  `IndexMaintenanceFailureOutcome`, `PhysicalMutationShape`
- Proof surfaces:
  transition outcomes for synchronous update, async lag, rebuild-only,
  advisory-only, verifier-only, migration-only, cutover, denial, and deferral

### Required Work

Every derived physical layout must declare a maintenance mode:

- synchronous exact maintenance
- asynchronous exact but lagged maintenance
- rebuild-only projection
- lazy/materialized-on-demand projection
- advisory maintenance
- verifier-only projection
- migration-only projection

Every maintained layout must also declare its publication protocol and failure
outcomes. A live index may not be treated as exact unless its maintenance mode,
lag witness, publication protocol, and coverage basis justify that exactness.

Mutation support must be explicit. Update-in-place requires
`PhysicalMutationShape` admission with WAL-before-data proof, latch or stable
read/COW proof, page-LSN behavior, checksum rewrite behavior, torn-write
handling, and crash replay behavior.

### Tests

- Maintenance honesty: async-lagged, rebuild-only, lazy, advisory,
  verifier-only, and migration-only layouts cannot answer as synchronous exact
  indexes.
- Publication replay: index publication, compaction cutover, and migration
  cutover converge after crash/restart to the declared materialization state.
- Mutation denial: update-in-place without WAL/page-LSN/checksum/stable-read
  proof cannot be admitted as an execution-ready mutation.
- Lag localization: an index lag witness names the family, range, basis, and
  maintenance mode responsible for non-exact access.

### Closeout Gate

This phase is done when live index maintenance is a typed protocol rather than
an optimistic rebuild story.

## Phase 15: Corruption, Quarantine, And Readmission Semantics

### Purpose

Separate corruption handling from ordinary miss, stale, and unsupported cases.

### Relevant APIs

- Existing corruption/recovery surfaces:
  `RecoveryQuarantineScope`, `RecoveryStatusReport`, `RecoverySourceReport`,
  `DurableRecoveryOutcome`, `QuarantinedDecodedArtifact`,
  `CompatibilityRejection`, `CompatibilityRejectionKind`,
  `ManifestDigestMismatch`, `StaleDerivedVersionRejection`
- Planned S.3/S.5.1/S.7 surfaces:
  pre-decode integrity validation, scrub evidence, quarantine witnesses,
  readmission evidence, security-scope witnesses, blob corruption localization
- Foundational readmission surfaces:
  `worth_foundational::boundary_evidence_api::stronger_lane::readmission`,
  `worth_foundational::canonicalization_api::stronger_lane`
- S.8 surfaces to introduce: `LayoutCorruptionClassification`,
  `LayoutQuarantineWitness`, `LayoutReadmissionWitness`
- Proof surfaces:
  `.bridge_trust_boundary()`, `.rebind_with(...)`, `.readmit_with(...)`,
  `TransitionOutcome::stale(...)`, `TransitionOutcome::rebind_required(...)`

### Required Work

Define typed outcomes for:

- clean access
- not found
- unsupported access shape
- stale layout binding
- corrupted derived projection
- corrupted authoritative artifact
- quarantine required
- readmission required
- rebuild required
- migration required

Corruption must be localized to the family, layout strategy, version, and access
shape that observed it.

### Foundational Usage

Use Foundational boundary evidence only when corruption, quarantine, or
readmission crosses a trust boundary. Internally, Store keeps native typed
evidence.

### Tests

- Not-found cannot mask corruption.
- Corruption cannot be converted into an empty result.
- Quarantined physical artifacts cannot be read through ordinary access APIs.
- Readmission after backup/import or trust-boundary movement requires native
  Store witnesses before access resumes.

### Closeout Gate

This phase is done when the corruption path is a first-class state machine, not
an error string attached to an index lookup.

## Phase 16: Versioning, Migration, And Rollback

### Purpose

Make physical format evolution explicit before the engine depends on many
layout families.

### Relevant APIs

- Existing compatibility/version surfaces:
  `ArtifactFormatVersion`, `ArtifactSemanticVersion`,
  `ArtifactCompatibilityWindow`, `CompatibilityManifestIndex`,
  `CompatibilityRegistrySnapshot`, `RollingUpgradePolicy`,
  `RollingUpgradeAdmissionPlan`, `RollingWindowCompatibilityReceipt`,
  `RestoreCompatibilityPlan`, `RestoreCompatibilityReceipt`,
  `BackwardReadCompatibilityWitness`, `ForwardReadCompatibilityWitness`
- Planned S.1/S.4/S.7 version inputs:
  physical format version, root manifest version, WAL/checkpoint version,
  blob chunk-tree version, blob generation publication version
- S.8 surfaces to introduce: `LayoutVersion`,
  `LayoutCompatibilityWindow`, `LayoutMigrationPlan`, `LayoutRollbackPlan`
- Proof surfaces: `.try_resolve_ready(...)`, `.try_lower_ready(...)`,
  `.try_ready_now(...)`, `TransitionOutcome::RebindRequired`

### Required Work

Each admitted layout must declare:

- format version
- compatibility window
- read-old/write-new behavior, if supported
- dual-read or dual-write posture, if supported
- migration source and target
- rollback source and target
- interruption behavior
- stale binding detection

### Worth Proof Usage

Migration and rollback are proof transitions. They must not be plain helper
functions that mutate a layout descriptor in place.

### Tests

- Old compatible layouts can be read through an explicit compatibility lane.
- Incompatible layouts deny with a typed reason.
- Interrupted migration resumes or rolls back according to declaration.
- Rollback preserves authority and does not use derived projections as truth.

### Closeout Gate

This phase is done when layout evolution can be reasoned about without reading
ad hoc migration code.

## Phase 17: Amplification And Exact Counter Evidence

### Purpose

Attach real cost to every physical access path.

### Relevant APIs

- Existing counter surfaces:
  `StoreCounterSnapshot`, `Milestone6CounterContract`,
  `Milestone7CounterContract`, `Milestone10CounterContract`,
  `Milestone11CounterContract`, `Milestone12CounterContract`,
  `AspectLayoutPerformanceEnvelope`, `RecallAmplificationBudget`,
  `RetentionPlanningReport`, `ReclaimExecutionReport`
- Foundational performance surfaces:
  `FoundationalPerformanceCounterRow`,
  `FoundationalCounterBackedPerformanceReceipt`,
  `worth_foundational::performance_api::lower_lane::basis`,
  `worth_foundational::performance_api::lower_lane::receipts`,
  `counter_backed_performance_receipt(bundle)`,
  `FoundationalPerformanceReportPlan`,
  `FoundationalMaterializedPerformanceReport`
- S.8 surfaces to introduce: `AccessPlanCostEstimate`, `AccessPlanBudget`,
  `PlannedCounterEnvelope`, `AccessPathCounterSnapshot`,
  `AccessPathAmplificationReceipt`, `PlannedVsObservedCounterReceipt`,
  `CostEnvelopeViolationOutcome`, `StoreLayoutPerformanceReceipt`
- Proof surfaces: `ExecutedRecipe<T, A>` and `SuccessfulTransitionOutcome`
  only after Store execution has produced the counter rows

### Required Work

Define exact counters for at least:

- page touches
- index probes
- key comparisons
- range steps
- prefix steps
- chunk-tree node reads
- manifest reads
- bytes read
- bytes written
- write fanout
- read amplification
- write amplification
- compaction debt created or retired
- rebuild bytes read and written
- verifier bytes read
- cache or buffer residency effects where Store already models them

This phase deepens counter evidence; it does not introduce cost discipline for
the first time. Strategy declaration, strategy admission, plan selection,
access lowering, and execution readiness must already carry the planned
counter dimensions required for their stage.

Counters must be attached to transition outcomes, not incidental branches.
Successful execution must produce a planned-versus-observed receipt that names
the admitted plan fingerprint, planned envelope, observed counters, and any
deviation outcome.

### Foundational Usage

Only executed Store counters may produce
`FoundationalCounterBackedPerformanceReceipt`. Layout intent claims and policy
admission receipts are not counter proof.

### Tests

- Each admitted access family has counter assertions.
- A hidden broad scan fails the counter contract.
- A plan whose observed counters exceed the admitted envelope produces a typed
  violation outcome rather than ordinary success.
- Counter rows cannot be copied into readiness without executed Store evidence.
- Denied, stale, deferred, and failed outcomes still report the cost already
  spent where applicable.

### Closeout Gate

This phase is done when access-path cost is mechanically visible and cannot be
hand-waved by a claimed layout.

## Phase 18: Bootstrap Catalog And Minimal Root Discovery

### Purpose

Define the tiny fixed access path that discovers the layout catalog without
becoming an ordinary generic fallback.

### Relevant APIs

- Planned S.1 inputs:
  `PhysicalRootReference`, `PhysicalRootManifest`, `PhysicalManifestIndex`,
  `PhysicalPageHeader`, `PhysicalFrameHeader`, physical format version,
  checksums, and root-open evidence
- S.8 surfaces to introduce:
  `BootstrapLayoutCatalog`, `MinimalRootDiscoveryLayout`,
  `CatalogReadAdmission`, `BootstrapOnlyAccessPath`,
  `LayoutMaterializationState`
- Foundational/proof surfaces:
  boundary readiness only after Store has produced native root/catalog evidence

### Required Work

The bootstrap path must be:

- tiny
- fixed
- versioned
- checksummed
- readmission-gated after import/restore
- incapable of answering ordinary family access
- capable only of discovering the root manifest and layout catalog state

The bootstrap catalog must produce typed catalog read admission before ordinary
layout declarations, strategies, materialization states, or access plans can be
resolved. It must not become an alternate page table, manifest walker, or
fallback scan lane.

### Tests

- Bootstrap denial: bootstrap-only access cannot answer page, WAL, blob,
  security, export, maintenance, or recovery access directly.
- Catalog replay: the minimal root discovery path yields the same catalog state
  across reopen, crash replay, and certification replay.
- Corruption denial: corrupt bootstrap/root/catalog bytes quarantine or deny
  before ordinary layout resolution.
- Readmission denial: imported or restored catalog/root evidence cannot enter
  ordinary access without explicit readmission.

### Closeout Gate

This phase is done when Store can discover the layout catalog through a
mechanically bounded bootstrap path and no broader access path can hide inside
that exception.

## Phase 19: Page, Frame, Segment, And Extent Layout Families

### Purpose

Apply the new layout law to core physical payload containers.

### Relevant APIs

- Planned S.1 physical APIs:
  `PhysicalPageId`, `PhysicalSegmentId`, `PhysicalExtentId`,
  `PhysicalFrameId`, `PhysicalRecordSlot`, `PhysicalGeneration`,
  `PhysicalReference`, `PhysicalRootReference`, `PhysicalPageHeader`,
  `PhysicalFrameHeader`, `PhysicalRootManifest`, `PhysicalManifestIndex`,
  `PhysicalFreeSpaceSearchPolicy`, `PlatformPhysicalFacade`,
  `OfflinePhysicalVerifier`, `S2PhysicalSubstrateReadiness`
- Existing Store physical-adjacent APIs:
  `DurableBackendFamily`, `DurableMediaReport`, `DurabilityBarrierClass`,
  `PlacementBoundArtifactRef`, `PlacementResolvedReadHandle`,
  `TierPlacementEvidence`, `FamilyLocalPlacementPlan`
- S.8 surfaces to introduce: `PhysicalArtifactFamilyDeclaration`,
  `LayoutStrategyDeclaration`, `AccessShapeContract`,
  `AccessPathCounterSnapshot`
- Foundational surfaces:
  `FoundationalLayoutIntentClaim` for boundary description only and
  `FoundationalCounterBackedPerformanceReceipt` after executed counters

### Required Work

Admit layouts for:

- pages
- segments
- extents

Each family must have declared point/range/scan behavior where relevant,
declared authority posture, rebuild behavior, version posture, and counters.

### Tests

- Page and segment access cannot bypass layout admission.
- Frame and slot access cannot bypass layout admission.
- Page/segment/extent access has exact probe and byte counters.

### Closeout Gate

This phase is done when page, frame, segment, and extent access no longer use
implicit lookup helpers.

## Phase 20: Root Manifest, Allocation, Free-Space, And Placement Layout Families

### Purpose

Apply layout law to the discovery and allocation structures that tell Store
where physical artifacts live.

### Relevant APIs

- Planned S.1 physical APIs:
  `PhysicalRootManifest`, `PhysicalRootReference`, `PhysicalManifestIndex`,
  `PhysicalFreeSpaceSearchPolicy`, `PhysicalFragmentationPressureReport`,
  segment manifests, extent manifests, free-space maps, allocation classes
- Existing placement/tiering APIs:
  `PlacementBoundArtifactRef`, `PlacementResolvedReadHandle`,
  `TierPlacementEvidence`, `FamilyLocalPlacementPlan`,
  `CanonicalResidencyManifest`, `ResidentReadLease`, `ColdRecallLease`
- S.8 surfaces to introduce:
  manifest `PhysicalArtifactFamilyDeclaration`,
  allocation `LayoutStrategyDeclaration`, placement `AccessShapeContract`,
  placement `AccessPathCounterSnapshot`

### Required Work

Admit layouts for:

- root manifests
- segment and extent manifest indexes
- allocation classes
- free-space maps
- fragmentation reports
- placement and residency maps

Root manifest layout is discovery authority. Allocation, free-space, and
placement structures may accelerate reads and writes, but they must not become
data authority.

### Tests

- Discovery parity: reopen and offline verification discover the same admitted
  manifest graph from `PhysicalRootManifest`.
- Authority denial: free-space, placement, or residency projections cannot
  satisfy data-authority APIs.
- Broad-scan denial: append and locate paths cannot scan all pages, all
  segments, or all placement rows unless using a typed verifier/rebuild lane.

### Closeout Gate

This phase is done when physical discovery, allocation, free-space, and
placement layouts are declared separately from page/segment/extent payload
layouts.

## Phase 21: WAL And Checkpoint Layout Families

### Purpose

Apply S.8 to write-ahead and checkpoint structures before recovery and replay
indexes are admitted.

### Relevant APIs

- Existing WAL/media APIs:
  `WalRecord`, `WalRecordFamily`, `WalRecordPayload`, `DurableMutationId`,
  `DurablePublicationPhase`, `RecoveryDecisionClass`, `CURRENT_WAL_VERSION`,
  `DurabilityBarrierClass`, `DurableBackendFamily`
- Planned S.4 APIs:
  WAL segment declarations, checkpoint manifests, page-LSN replay law,
  durable publication records, checkpoint compatibility witnesses
- S.8 surfaces to introduce:
  WAL `PhysicalArtifactFamilyDeclaration`, checkpoint
  `LayoutStrategyDeclaration`, append `AccessShapeContract`
- Proof surfaces:
  `TransitionOutcome::{Denied, Stale, RebindRequired, Failed}` must remain
  distinct through replay and recovery lowering

### Required Work

Admit layouts for:

- WAL records
- checkpoints
- recovery manifests
- crash-boundary access structures

The implementation must distinguish WAL/checkpoint authority from later replay
indexes and must preserve crash/recovery ordering semantics.

### Tests

- Checkpoint indexes cannot replace WAL authority.
- WAL append and checkpoint lookup expose exact range, byte, and barrier
  counters.
- Corrupt checkpoint projections quarantine or rebuild without masking WAL
  authority corruption.

### Closeout Gate

This phase is done when WAL and checkpoint layouts have explicit authority,
access, version, corruption, and counter contracts before replay indexes are
introduced.

## Phase 22: Recovery, Replay, And Crash-Boundary Index Families

### Purpose

Apply S.8 to recovery and replay projections after WAL/checkpoint authority is
declared.

### Relevant APIs

- Existing recovery APIs:
  `RecoveryStatusReport`, `RecoverySourceReport`, `RecoverySourceKind`,
  `RecoveryQuarantineScope`, `DurableRecoveryOutcome`,
  `DurableRecoverySourceSummary`, `BackupRestoreCompatibilityReport`
- Existing snapshot APIs:
  `SnapshotImageBundle`, `SnapshotReadRequest`, `SnapshotRestorePlan`
- Planned S.4 APIs:
  recovery-source precedence witnesses, replay receipts, bounded-recovery
  counters, replay source reports
- S.8 surfaces to introduce:
  replay `PhysicalArtifactFamilyDeclaration`, replay `AccessShapeContract`,
  `DerivedIndexRebuildPlan`, `LayoutQuarantineWitness`
- Proof surfaces:
  `TransitionOutcome::{Denied, Stale, RebindRequired, Failed}` must remain
  distinct through replay and recovery lowering

### Required Work

Admit layouts for:

- replay indexes
- recovery records
- recovery source reports
- crash-boundary verifier indexes
- bounded WAL-tail lookup structures

Replay indexes are derived unless a lower Store type explicitly classifies a
recovery artifact as authoritative. Recovery scans are verifier/rebuild lanes,
not foreground fallbacks.

### Tests

- Replay parity: derived replay indexes rebuild from WAL/checkpoint authority
  and produce the same replay frontier.
- Authority denial: replay index rows cannot replace WAL/checkpoint authority.
- Crash-boundary denial: backend residue, leftover files, and successful
  filesystem enumeration cannot satisfy recovery layout admission.

### Closeout Gate

This phase is done when crash/recovery access has the same layout discipline as
ordinary foreground access without letting replay projections become authority.

## Phase 23: Snapshot, Branch, And Continuation Layout Families

### Purpose

Apply S.8 to durable read-support structures that are easy to forget because
they are not pages, WAL, or blobs.

### Relevant APIs

- Existing snapshot APIs:
  `SnapshotImageBundle`, `SnapshotReadRequest`, `SnapshotReadResult`,
  `SnapshotRestorePlan`, `PublishedSnapshotHandle`
- Existing branch/delta APIs:
  `BranchDeltaReadRequest`, `BranchDeltaReadPlan`, `BranchDeltaReadResult`,
  `BranchDeltaRewritePlan`, `BranchDeltaRebuildReceipt`,
  `SameBranchDescendantWitness`
- Existing live-query/continuation APIs:
  `StableBasisReadRequest`, `StableBasisReadPlan`, `ContinuationBatchResult`,
  `ContinuationRetentionStatus`, `CursorContinuationPlan`,
  `BroadenedBatchReceipt`, `AdmittedNarrowBatchReceipt`
- S.8 surfaces to introduce:
  snapshot, branch-delta, and continuation `PhysicalArtifactFamilyDeclaration`
  values; read-support `AccessShapeContract`; support `AccessPathCounterSnapshot`

### Required Work

Admit layouts for:

- snapshot images and snapshot indexes
- branch delta layers and delta rewrite products
- live-query stable-basis indexes
- cursor continuation support structures
- subscription-support access structures that act as durable read support

These families often look like semantic support, but S.8 owns their physical
access paths and derived-layout honesty.

### Tests

- Replay/convergence: branch, snapshot, and continuation read-support layouts
  rebuild from their declared authority and converge to the same read frontier.
- Authority denial: continuation, support, or snapshot projections cannot
  become canonical commit authority.
- Counter denial: broadened continuation reads cannot hide whole-family scans
  behind support-row counters.

### Closeout Gate

This phase is done when durable read-support structures have first-class
physical layout declarations instead of inheriting whatever layout their
feature originally used.

## Phase 24: Blob Object, Chunk-Tree, And Streaming Layout Families

### Purpose

Apply S.8 to the S.7 blob object and streaming structures before admitting
maintenance indexes that derive from them.

### Relevant APIs

- Planned S.7 blob APIs:
  `BlobObjectId`, `BlobGeneration`, authoritative/derived blob
  classification, chunk identity, `LogicalContentDigest`, `StoredChunkDigest`,
  `AuthenticatedFrameDigest`, chunk-tree root, canonical chunking rule,
  `BlobGenerationPublished`, resumable ingest states,
  `HeavyBlobFixturePlan`, and sealed `S7NativeBlobStoreCloseout`
- Existing Store chunk/layout APIs:
  `PhysicalChunkId`, `ChunkModelFrozenPhysicalLayout`,
  `ChunkDeterminismWitness`, `Milestone6ChunkModelExport`,
  `StructuralBlockLookup`, `DedupAdmittedBlockReuse`,
  `DedupBackedReadResult`, `Milestone7IndependentLayoutReference`,
  `Milestone9PhysicalChunkReference`
- S.8 surfaces to introduce: chunk-tree `LayoutStrategyDeclaration`,
  streaming `AccessShapeContract`, blob `AccessPathCounterSnapshot`,
  `DerivedIndexParityWitness`
- Foundational/certification surfaces:
  `FoundationalCounterBackedPerformanceReceipt` for executed blob counters and
  S.4.5 simulation replay bundles for heavy-blob evidence

### Required Work

Admit layouts for:

- blob chunk trees
- blob manifests
- streaming cursors or resume indexes
- blob generation publication records
- stored chunk lookup structures

Large blob tests must include a real multi-gigabyte locally generated corpus
through the S.7 harness plan, not a tiny synthetic fixture pretending to cover
large-blob behavior.

### Tests

- Streaming reads use declared streaming access paths.
- Blob identity is preserved across chunk-tree, streaming, and generation
  publication structures.
- Multi-gigabyte blob access records exact counters and does not silently route
  through whole-store scans.

### Closeout Gate

This phase is done when blob object, chunk-tree, and streaming layouts are
declared separately from blob maintenance projections.

## Phase 25: Blob Dedupe, Reachability, Retention, Reclaim, And Compaction Layout Families

### Purpose

Apply S.8 to blob maintenance projections only after the authoritative blob
object and chunk-tree families are explicit.

### Relevant APIs

- Planned S.7 blob maintenance APIs:
  dedupe admission, reachability edges, retention holds, reclaim receipts,
  placement movement, blob compaction, corruption quarantine, export/import
  bundle surfaces, and sealed `S7NativeBlobStoreCloseout`
- Existing maintenance/retention APIs:
  `RetentionPlanningReport`, `RetentionClosureWitness`,
  `ReclaimEligibilityWitness`, `ReclaimExecutionReport`,
  `CompactionPlan`, `CompactionCutoverWitness`, `PublishedCompactionProduct`,
  `LoweredCompactionDeclaration`, `LoweredReclaimDeclaration`
- S.8 surfaces to introduce:
  maintenance-family `PhysicalArtifactFamilyDeclaration`,
  `DerivedIndexParityWitness`, `LayoutQuarantineWitness`,
  `AccessPathAmplificationReceipt`, and maintenance counter snapshots
- Foundational/certification surfaces:
  counter-backed performance receipts for executed maintenance paths and S.4.5
  replay bundles for corruption, compaction, retention, and reclaim evidence

### Required Work

Admit layouts for:

- dedupe indexes
- reachability maps
- retention and hold maps
- reclaim queues
- compaction support structures
- corruption/quarantine maps

Dedupe, reachability, retention, reclaim, compaction, and quarantine structures
are derived maintenance evidence. They may guide work, but they do not become
blob identity or blob authority.

### Tests

- Dedupe indexes cannot become blob authority.
- Reachability and reclaim maps cannot delete or retain without the required
  authority witness.
- Compaction support structures cannot mutate the blob generation they derive
  from without an admitted cutover transition.
- Reclaim queues cannot convert missing reachability evidence into delete
  authority.

### Closeout Gate

This phase is done when every blob maintenance layout is visibly derived from
the blob authority graph and cannot masquerade as source truth.

## Phase 26: Maintenance, Tiering, And I/O Scheduler Layout Families

### Purpose

Apply S.8 to background work, tier movement, and I/O scheduling structures
after the concrete artifact families they operate on are explicit.

### Relevant APIs

- Existing maintenance and tiering APIs:
  `CompactionPlan`, `PublishedCompactionProduct`,
  `LoweredCompactionDeclaration`, `LoweredReclaimDeclaration`,
  `TierPlacementEvidence`, `FamilyLocalPlacementPlan`,
  `CanonicalResidencyManifest`, `ResidentReadLease`, `ColdRecallLease`
- Planned S.6 I/O QoS APIs:
  foreground/background admission, direct-I/O posture, compaction interference
  counters, scheduler readiness, and published I/O admission evidence
- S.8 surfaces to introduce:
  maintenance queue declarations, scheduler reservation indexes, tier placement
  layout declarations, recall queue access shapes, and interference counter
  snapshots
- Foundational/proof surfaces:
  performance readiness, counter-backed receipts, non-success scheduler
  outcomes, and replay evidence for background work admission

### Required Work

Admit layouts for:

- maintenance queues
- scheduler reservation indexes
- tier placement manifests
- cold recall queues
- working-set and recall amplification indexes
- foreground interference accounting structures

Foreground admission remains a scheduler decision backed by executed evidence.
Maintenance, tiering, and recall indexes may accelerate decisions, but they do
not publish I/O readiness by themselves.

### Tests

- Scheduler readiness consumes executed Store-published evidence, not copied
  queue rows.
- Tier placement and residency projections cannot replace artifact authority.
- Foreground interference counters are exact for the declared access shape and
  cannot hide background scans.

### Closeout Gate

This phase is done when maintenance, tiering, recall, and I/O scheduler layouts
are explicit accelerators with bounded authority.

## Phase 27: Security And Custody Lookup Layout Families

### Purpose

Apply layout discipline to physical structures that touch security and custody
without making Store an identity provider.

### Relevant APIs

- Planned S.5.1 security-scope APIs:
  key scope, key version, tenant scope, authenticity class, custody posture,
  repair blast-radius readiness, import/readmission scope witnesses
- Existing support/security-adjacent APIs:
  `RawSupportTrustRequest`, `SupportTrustRequestAdmitted`,
  `SupportTrustAccessPath`, `SupportTrustAccessStructurePlan`,
  `SupportTrustClassificationWitness`, `SupportTrustFreshnessWitness`,
  `SupportTrustReadmissionStatus`, `SupportImportAdmissionWitness`,
  `SupportManifestAdmissionWitness`, `SupportRoadmapPhysicalReadinessPosture`
- S.8 surfaces to introduce: security/custody
  `PhysicalArtifactFamilyDeclaration`, `LayoutReadmissionWitness`,
  `AccessShapeContract`, `LayoutQuarantineWitness`

### Required Work

Admit layouts for physical access structures tied to:

- tenant scope lookup
- key scope lookup
- custody posture lookup
- repair blast-radius lookup

These structures may accelerate security-boundary decisions, but they must not
turn identity-provider claims, JWT subjects, app org IDs, IAM roles, KMS key
IDs, or operator identities into Store authority.

This phase does not localize tenant/key/custody scope to security lookup
families. Phase 3 and Phase 8 require every ordinary physical layout strategy,
including shared B-trees, LSM runs, hash indexes, dedupe indexes, chunk
lookups, manifest indexes, and range maps, to declare whether it is
single-tenant, tenant-partitioned, cross-tenant-denied,
cross-tenant-admitted, single-key-scope, or cross-key-scope admitted under an
explicit policy. This phase only admits the lookup families that help inspect
or accelerate those scope decisions.

### Tests

- Security/custody projections cannot satisfy Store authority directly.
- JWT subject, application org ID, KMS key ID, IAM role, and operator identity
  examples are denied as Store authority.
- Repair blast-radius readiness proves where repair may physically observe or
  read; it does not prove who may initiate repair.

### Closeout Gate

This phase is done when security-adjacent layout structures are fast without
becoming an accidental auth system.

## Phase 28: Export, Import, Capsule, And Offline Verifier Layout Families

### Purpose

Apply S.8 to terminal and offline structures with explicit readmission at every
trust boundary.

### Relevant APIs

- Existing export/import/compatibility APIs:
  `AuthoritativeExportBundle`, `AuthoritativeExportRestoreRequest`,
  `BackupCompatibilityManifest`, `RestoreCompatibilityPlan`,
  `RestoreCompatibilityReceipt`, `RestoreBackupScope`,
  `CompatibilityRestoreExecutionOutcome`
- Planned S.7/S.10 export/import APIs:
  blob export/import bundle surfaces, capsule manifests, compatibility
  manifests, backup/restore evidence, and offline verifier evidence
- Foundational APIs:
  `worth_foundational::canonicalization_api::lower_lane::export`,
  `worth_foundational::canonicalization_api::stronger_lane`,
  `worth_foundational::boundary_evidence_api::stronger_lane::readmission`,
  identity and boundary-artifact surfaces for bridged/readmitted evidence
- S.8 surfaces to introduce:
  export/import `PhysicalArtifactFamilyDeclaration`,
  `LayoutReadmissionWitness`, terminal declaration parsing, offline verifier
  access shapes, and readmission counter snapshots

### Required Work

Admit layouts for:

- export/import bundle indexes
- capsule manifests
- offline verifier indexes
- backup/restore layout evidence
- imported layout readmission evidence

Serde and JSON representations are terminal declarations only. Deserialization
may produce raw declarations, but every deserialized security or layout value
must be readmitted before it can become a witness. Trust boundaries include a
different deployment, different Store instance, different key-scope generation,
different tenant-scope authority, different custody domain, offline
export/import, and backup restoration after key rotation.

### Tests

- Imported or restored layout evidence crossing a trust boundary requires
  explicit readmission.
- Offline verifier indexes cannot be used as foreground authority.
- Terminal serde/JSON declarations cannot become admission authority without
  readmission.
- Backup after key rotation, different Store instance, different tenant
  authority, and different custody domain are all denied without readmission.

### Closeout Gate

This phase is done when terminal/offline layout evidence is useful for
transport and verification but never enters Store authority without
readmission.

## Phase 29: Legacy Surface Disposition And Dedicated Workspace Boundary

### Purpose

Classify every old access-path surface before shaping the final S.8 public
facades.

### Relevant APIs

- Existing Store surfaces to disposition:
  `WorthStore`, `WorthStoreBuilder`, `AspectLayoutReadRequest`,
  `AdmittedAspectLayoutReadPlan`, `AspectLayoutReadExecutionDecision`,
  `Milestone6LayoutMaterialization`, `Milestone6PreparedLayoutSupport`,
  `ChunkModelFrozenPhysicalLayout`, `StructuralBlockLookup`,
  `DedupAdmittedBlockReuse`, `Milestone7IndependentLayoutReference`,
  `Milestone9PhysicalChunkReference`, compatibility registries, maintenance
  declarations, support-trust access structures, and certification helpers
- Roadmap 2 boundary:
  the dedicated Store workspace/crate family is the architectural foundation;
  old topology is migration input, not precedent
- S.8 surfaces to introduce:
  `LegacyAccessPathBypassInventory`, `LegacySurfaceDisposition`,
  `LegacySurfaceDispositionAndDedicatedWorkspaceBoundary`

### Required Work

Every legacy surface must receive one explicit disposition:

- consumed as production authority
- consumed as input only
- wrapped behind S.8 facade
- superseded and forbidden
- certification only
- terminal only
- deprecated debt, only if the debt rules are actually satisfied
- forbidden as authority

The inventory must identify old helpers, fallback classes, broad public
exports, certification fixtures, compatibility bridges, and support structures
that could skip declaration, admission, lowering, readiness, execution, or
readmission.

S.8 production authority must live in the dedicated Store workspace/crate
family. Existing `crates/worth-store/src/...` paths may be consumed or migrated,
but they may not define topology precedent.

### Tests

- Legacy bypass denial: old access structures, fallback classes, certification
  helpers, and broad facades cannot skip the S.8 declaration -> admission ->
  selection -> budget -> lowering -> readiness -> execution chain.
- Disposition parity: each legacy surface named in the inventory has a test or
  compile-fail proof matching its disposition.
- Dedicated-boundary denial: old topology paths cannot be deep-imported as
  production S.8 authority.

### Closeout Gate

This phase is done when there is no ambiguous old surface left that a future
caller could treat as a valid physical access-path shortcut.

## Phase 30: Public Facades And Directory Shape

### Purpose

Make the module topology teach the lifecycle.

### Relevant APIs

- Existing facades:
  `WorthStore`, `WorthStoreBuilder`, `crates/worth-store/src/facade`,
  `crates/worth-store/src/backend/facade`, `layout::mod`,
  `media::mod`, `wal::mod`, `maintenance::mod`, `retention::mod`,
  `tiering::mod`, `compatibility::mod`
- Existing public export pressure:
  the broad `crates/worth-store/src/lib.rs` re-export surface must be split or
  curated so S.8 public imports reveal lifecycle order
- S.8 public facade surfaces to introduce:
  `layout_families`, `layout_strategy_admission`, `access_lowering`,
  `access_execution`, `layout_rebuild`, `layout_migration`,
  `layout_counters`, `layout_readmission`, and `layout_certification`
  facades or equivalent lifecycle-shaped modules
- Compile-fail support:
  the S.8 UI suite must prove raw fields cannot construct admitted strategies,
  ready access plans, executed counter receipts, or readmitted witnesses

### Required Work

The public surface should be phase-shaped and authority-shaped. It should make
these questions obvious:

- Am I declaring a family, admitting a layout, lowering access, executing
  access, rebuilding, migrating, reporting counters, or certifying?
- Is this source truth, derived projection, diagnostic evidence, terminal
  export, or certification harness vocabulary?
- Is this foreground, background, verifier, repair, or terminal access?

Avoid broad `mod.rs` business logic. Avoid helper buckets. Keep classifiers,
transition functions, receipts, counters, and facade exports in directories that
match their semantic responsibility.

### Tests

- Compile-fail coverage proves external callers cannot construct admitted
  layout, ready access, executed counter receipts, or readmitted witnesses from
  raw fields.
- Public imports reveal lifecycle order rather than one flat ontology.
- Code-quality QA passes with composition, topology, and god-function findings
  cleared.

### Closeout Gate

This phase is done when an implementer can find the correct access-path phase
from the directory tree before reading function bodies.

## Phase 31: Certification Harness Vocabulary And Scenario Skeleton

### Purpose

Teach the harness vocabulary to describe layout law without becoming layout
law.

### Relevant APIs

- Existing harness/certification surfaces:
  `crates/worth-store/src/tests/harness`, corruption local-file helpers,
  `Milestone6AccessStructureVerification`,
  `Milestone7AccessStructureVerification`, `Milestone6CertificationBundle`,
  `Milestone7CertificationBundle`, `Milestone12CertificationRunner`,
  `Milestone12CertificationLaneInput`, `Milestone12CertificationLaneOutcome`
- S.4.5 harness concepts:
  scenario plans, production actors, fault injectors, observers, oracles,
  transcripts, replay bundles, coverage rows, shortcut denials
- Foundational readiness surfaces:
  `worth_foundational::performance_api::stronger_lane::readiness`,
  `worth_foundational::boundary_evidence_api::stronger_lane::readiness`,
  `worth_foundational::canonicalization_api::stronger_lane::readiness`
- S.8 surfaces to introduce:
  layout certification scenario fixtures, scenario skeleton vocabulary, layout
  performance evidence bundle, and `StorageFoundationS9LayoutHandoff`
- Proof surfaces:
  `ProofOutcomeKind`, `TransitionOutcome`, `ExecutedRecipe<T, A>` in harness
  assertions without letting harness code mint production authority

### Required Work

Extend the S.4.5 harness vocabulary so it can describe:

- layout declaration inventory tests
- access-shape denial tests
- broad-scan rejection tests
- exact counter tests
- corruption/rebuild parity tests
- migration/rollback interruption tests
- trust-boundary readmission tests
- multi-artifact integration simulations

Certification should consume production witnesses and executed evidence. It
must not define production constructors that lower crates depend on.

### Tests

- Harness helpers cannot mint production layout authority.
- Scenario skeletons identify production APIs, actors, fault injectors,
  oracles, counters, and closeout evidence before runtime simulations exist.
- Certification fixtures declare which production transition states ordinary
  Store code must pass through.

### Closeout Gate

This phase is done when the harness can describe S.8 evidence while remaining
the courtroom, not the law.

## Phase 32: Compile-Fail Authority And Facade Proofs

### Purpose

Prove the public S.8 surface cannot be bypassed from external crates.

### Relevant APIs

- Public facade APIs introduced in Phase 30
- Sealed S.8 authority surfaces:
  `AdmittedLayoutStrategy`, `LoweredAccessPlan`,
  `ExecutionReadyAccessPlan`, `ExecutedAccessPathEvidence`,
  `AccessPathCounterSnapshot`, `AccessPathAmplificationReceipt`,
  `DerivedIndexParityWitness`, and `LayoutReadmissionWitness`
- Existing compile-fail style from the S.4.5/S.5/S.6/S.7 UI suites
- Worth Proof surfaces:
  `TransitionOutcome`, non-success evidence, and executed recipe/certificate
  progression where applicable

### Required Work

Add compile-fail coverage proving external crates cannot:

- construct admitted layouts from raw fields
- execute weak artifact declarations
- mint counter receipts from copied rows
- readmit terminal/export/offline values without the readmission transition
- use certification fixtures as production authority
- deep-import internal family modules to skip facade order

### Tests

- Compile-fail tests cover every forbidden constructor class above.
- Public imports reveal lifecycle order rather than one flat ontology.
- Facade bypass attempts fail at compile time, not through runtime denial
  wrappers.

### Closeout Gate

This phase is done when external callers can only move through the phase graph
with the returned proof object from the previous valid transition.

## Phase 33: Runtime Simulation And Integration Certification

### Purpose

Run adversarial S.8 simulations only after the vocabulary, facades, and
compile-fail boundary are shaped.

### Relevant APIs

- S.4.5 harness runtime APIs:
  scenario plans, production actors, fault injectors, observers, oracles,
  transcripts, replay bundles, coverage rows, and shortcut denials
- Executed S.8 surfaces:
  layout declarations, admitted strategies, lowered plans, execution-ready
  plans, executed evidence, counter receipts, parity witnesses, readmission
  witnesses, and S.9 handoff
- Foundational surfaces:
  performance readiness, boundary-evidence readiness, canonicalization
  readiness, counter-backed performance receipts, and materialized reports
- S.7 heavy fixture surface:
  `HeavyBlobFixturePlan` or equivalent real multi-gigabyte local corpus plan

### Required Work

Run runtime simulations for:

- page, frame, segment, extent, root manifest, allocation, and placement
- WAL, checkpoint, recovery, replay, crash-boundary, snapshot, branch, and
  continuation
- blob object, chunk-tree, streaming, dedupe, reachability, reclaim, retention,
  compaction, and quarantine
- maintenance, tiering, recall, and I/O scheduler structures
- security, custody, export, import, capsule, and offline verifier boundaries

Each simulation must identify production APIs, exact counters, fault injection,
replay/rebuild/readmission expectations, and broad-scan rejection evidence.

### Tests

- Every admitted `LayoutStrategyFamily` has coverage for success,
  unsupported-shape denial, stale/rebind, corrupt-derived, corrupt-authority,
  rebuild, migration/rollback, hidden-scan denial, readmission, and
  cost-envelope violation unless the spec names an explicit equivalence class
  and proves why that family is covered by it.
- Integration simulations cover core physical families, recovery families, blob
  families, maintenance/I/O families, and security/custody/export boundary
  families through the coverage matrix rather than a single representative
  happy path.
- Fault simulations prove corrupted derived projections quarantine or rebuild
  without masking authoritative corruption.
- Scale simulations use the S.7 heavy fixture plan for large-blob access and
  exact counter evidence.

### Closeout Gate

This phase is done when S.8 has executed evidence across the major lifecycle
families, not just compile-time shape.

## Phase 34: Complete Layout Runtime Authority

### Purpose

Complete the ordinary read-side layout path from durable-family declaration
through executed access evidence, with every decision issued by the permanent
domain operation that owns it.

### Relevant APIs

- `PhysicalArtifactFamilyDeclaration`, `ArtifactFamilyAuthorityWitness`,
  `PhysicalKeyDomainWitness`, `AdmittedLayoutStrategy`
- `LayoutMaterializationState`, `LayoutCoverageWitness`,
  `MaterializationCompleteness`, `PhysicalAbsenceProof`
- access-shape admission, candidate classification, deterministic plan
  selection, `PreExecutionBudgetAdmissionReceipt`, and
  `PlannedCounterEnvelope`
- indexed and degraded lowering, execution readiness, counter admission,
  executed evidence, and planned-versus-observed receipts
- `worth_proof::AuthorityWitness`, `worth_proof::TransitionOutcome`, and the
  existing Foundational canonical-basis, receipt, diagnostic, and performance
  evidence surfaces already adopted by Store

### Required Work

Implement one typed capability chain:

```text
admitted family
-> admitted key domain
-> admitted strategy and invariant suite
-> admitted materialization/coverage posture
-> selected access plan
-> admitted budget and planned counter envelope
-> owner-specific lowered operation
-> owner-specific readiness
-> owner-observed counters
-> executed access evidence
```

Catalog rows and declarations are inputs, not authority. Key laws become
usable only after keyspace admission. Strategy declarations become usable only
after invariant and scope admission. Materialization labels, counter rows, and
absence-shaped results cannot substitute for their owner-issued witnesses.

Indexed access and degraded exact scan are separate owner operations. A shared
read-only view may project them for callers, but no shared constructor may
inspect a payload or path-kind field after construction to decide which owner
supposedly issued it.

Every operation owns its private case enum and private issuance functions in
the same module as the decision. `pub(crate)` outcome constructors are not an
acceptable boundary: sibling modules, certification, and test support must be
unable to mint an outcome. `TransitionOutcome` may expose success, denial,
deferred, stale, rebind, or failure as a projection after issuance; it may not
be the raw authority exchanged between unrelated operations.

Rename every touched type, module, export, compile-fail fixture, and public
documentation surface into permanent database vocabulary. The phase must
remove obsolete root compatibility facades for the migrated read path rather
than retaining milestone-named aliases.

Planning must resolve strategy, locality, exactness, and budget before the hot
path. Execution consumes the narrowest admitted operation and may not repeat
candidate selection, materialization classification, or key-law validation
inside the same trust boundary. Exact structural counters remain attached to
the executed result.

This phase owns production completeness for the read-side capability chain and
critical proof at each authority boundary. Every owner case declared by this
chain must be ordinarily reachable before the phase closes. Phase 37 owns exact
cross-product accounting, the complete hostile matrix, mutation testing, and
large-scale simulations; Phase 38 owns the workspace-wide residue audit and
final courtroom verdict. Those later phases may expose a real defect here, but
they do not justify leaving a production operation or authority boundary
unfinished.

### Tests

- Reach every declared read-side owner case through its ordinary production
  facade, including unsupported-shape, missing-strategy, stale, rebind,
  budget-denied, hidden-scan, corrupt-derived, and degraded-exact cases. This
  establishes ordinary reachability; Phase 37 owns exact cross-product and
  coverage-receipt closure.
- Prove identical semantic requests produce deterministic selection and plan
  fingerprints independent of declaration order, registry iteration order, or
  certification scenario ordering.
- Prove copied declarations, copied witnesses, copied counters, copied
  materialization labels, and recomputed digests cannot enter lowering,
  readiness, or execution.
- Prove indexed evidence cannot be minted from degraded execution and degraded
  evidence cannot claim bounded indexed locality.
- Assert exact page-touch, index-probe, key-comparison, byte-read, allocation,
  and amplification counters for baseline B-tree and LSM operations. Use
  scale-sensitive workloads to prove locality and amplification slopes only
  for algorithms whose work should vary with key count, tree height, run
  count, result width, or bytes read; use exact fixed counters for constant
  admission and metadata transitions.
- Add focused compile-fail cases for the strongest read-side authority seams:
  external callers and sibling domains cannot invoke issuance, construct
  private cases, pair payloads with outcomes, or skip budget admission. Every
  fixture must fail for the intended privacy/type error, never `E0463`; Phase
  37 owns the complete compile-fail matrix.
- Add local structural checks for the migrated read-path modules, rejecting
  milestone vocabulary, crate-wide outcome issuers, payload-derived owner
  selection, generic transition catalogs, and business logic in root or
  `mod.rs` facades. Phase 38 owns the final workspace-wide residue audit.

### Closeout Gate

This phase is done when the complete ordinary read path is expressed by
permanent owner-issued capabilities, every declared read-side owner case is
ordinarily reachable, critical substitutions are rejected at their real
boundaries, algorithm-sensitive counters prove the claimed read-path costs,
and neither sibling modules nor projections can manufacture a stronger stage.

## Phase 35: Complete Layout Mutation And Recovery Authority

### Purpose

Complete the write-side and exceptional-state layout graph so derived indexes
can be maintained, failed, rebuilt, evolved, quarantined, and readmitted
without broad transition bags or certification-authored authority.

### Relevant APIs

- live mutation admission, maintenance mode, lag witness, lowered publication,
  exact publication authority, and mutation counter evidence
- derived rebuild request/plan/receipt and parity identity, coverage, ordering,
  counter-shape, and cost-envelope witnesses
- migration request resolution, compatibility admission, lowering, readiness,
  interruption, rollback, stale binding, and explicit rebind
- corruption classification, quarantine authority, record-backed recovery,
  offline evidence, import readmission, and materialization restoration
- bootstrap catalog discovery, admitted root reads, and bounded legacy-surface
  disposition

### Required Work

Complete the owner state machines in this order. Each step must leave its
production capability usable through the ordinary facade and must close its
critical local authority tests before the next step begins. These are ordered
implementation boundaries inside Phase 35, not duplicate final-certification
gates:

1. maintenance admission, lowering, deferred/lagged handling, and exact
   publication;
2. rebuild execution followed by independent parity verification;
3. migration readiness, interruption/resume, rollback, and stale rebind;
4. corruption classification, quarantine, and each source-specific
   readmission boundary; and
5. bootstrap discovery followed by explicit removal, denial, or bounded
   ownership of every legacy surface.

An owner boundary is complete here when the production operation exists, its
ordinary cases are reachable, its strongest local forgery is rejected, and its
replay or convergence obligation passes. Exhaustive family cross-products,
complete hostile and compile-fail matrices, mutation testing, and large-scale
simulation remain centralized in Phase 37.

Split maintenance into owner operations for mutation admission, publication
lowering, lag/deferred handling, rebuild admission, rebuild execution, and
parity verification. Exact, lagged, advisory, verifier-only, migration-only,
rebuild-only, and deferred modes remain distinct capabilities. One broad
maintenance outcome must not erase which operation established the result.

Rebuild and parity are separate transitions. Rebuild consumes canonical or
declared physical authority and produces a candidate derived artifact. Parity
compares that artifact against admitted authority and alone may produce parity
evidence. A rebuild receipt cannot imply parity, exact publication, or current
coverage.

Migration and rollback remain separate owner operations even where they share
algorithms. Resolve declaration and compatibility before lowering; lower
before readiness; require rebind or readmission whenever store authority,
tenant/key scope, version, strategy, coverage, or freshness changes. A generic
`TransitionOutcome` may project an issued operation result but may not be
unwrapped and repackaged by a facade into stronger authority.

Corruption classification adapts lower physical-integrity evidence without
claiming ownership of physical damage truth. Quarantine, record-backed recovery
readmission, offline-evidence readmission, and import readmission are distinct
operations with distinct trust boundaries. Deserialized declarations and
terminal projections are hostile inputs and must be readmitted before they can
become layout witnesses.

Bootstrap discovery may issue materialization evidence only after an admitted
catalog read from the physical root. Legacy disposition is explicit: every old
surface is removed, denied, or isolated behind a bounded compatibility owner;
no adapter may preserve a raw-result or milestone authority lane.

All touched types, files, exports, and tests adopt permanent vocabulary and
owner-shaped directories. Facades delegate; they do not classify corruption,
select maintenance cases, or issue outcomes.

This phase contains several separate owner state machines because maintenance,
rebuild/parity, migration/rollback, corruption/readmission, bootstrap, and
legacy disposition have different authorities and failure modes. They must not
be collapsed into one implementation category or one broad outcome family.
The phase gate is complete production capability, preserved authority, replay
convergence, ordinary reachability of every declared owner case, and critical
adversarial proof at each trust boundary. Phase 37 owns exhaustive family
cross-products, the complete hostile and compile-fail matrices, mutation
testing, and scale simulations.

### Tests

- Exercise exact, lagged, advisory, verifier-only, migration-only,
  rebuild-only, and deferred maintenance through ordinary production requests;
  prove every declared maintenance owner case is ordinarily reachable and none
  can be substituted for exact publication.
- Prove rebuild followed by parity converges to the same derived identity as
  uninterrupted maintenance. Exact counters must prove each path's own
  declared work; rebuild and mutation counters must not be forced into false
  equality merely because their semantic result converges. A forged rebuild
  receipt or copied parity row cannot publish.
- Crash and resume migration and rollback at every declared interruption
  boundary; independent replay must converge on the same compatibility,
  binding, and publication posture.
- Reject cross-store, cross-tenant, cross-key-scope, stale-version,
  stale-coverage, recomputed-digest, and equal-looking authority substitutions
  before mutation, rollback, or readmission.
- Prove physical corruption cannot be converted into empty results, stale
  materialization, or successful readmission, and derived corruption cannot be
  promoted into physical authority.
- Prove bootstrap cannot use an ordinary family path before catalog admission
  and cannot construct current coverage from copied root rows.
- Add focused compile-fail tests at the strongest mutation, migration,
  readmission, and bootstrap authority seams, proving sibling modules,
  certification, and test support cannot construct owner outcomes. Phase 37
  owns exhaustive constructor and lifecycle-stage compile-fail coverage.
- Prove scale slopes with exact structural counters for mutation, publication,
  rebuild, parity, and any migration data path whose work legitimately varies
  with records, pages, runs, or bytes. Use exact fixed-cost counters for
  migration and rollback metadata transitions, quarantine classification,
  readmission, compatibility admission, and bootstrap discovery. Do not create
  synthetic scale workloads for operations whose contract is fixed-cost
  metadata work.
- Add local structural checks for the owner modules changed in this phase;
  Phase 38 owns the final workspace-wide residue audit.

### Closeout Gate

This phase is done when every write-side and exceptional-state production
capability exists, every declared owner case is ordinarily reachable through
its permanent owner, critical authority substitutions are denied, recovery and
evolution converge under replay, and no broad outcome, facade, compatibility
adapter, or test helper can mint a stronger layout capability. Exhaustive
cross-product certification remains Phase 37 work, not a duplicated local
gate.

## Phase 36: Complete Physical Compaction And Durable Membership Authority

### Purpose

Join layout execution to real physical compaction, WAL, checkpoint, and
manifest authority so LSM membership, publication, retirement, recovery
visibility, and reclaim cannot be authored inside layout code.

### Relevant APIs

- physical-isolation compaction lowering, cutover, publication, stable-read,
  recovery-visibility, deferred-reclaim, drain, and mutation-denial outcomes
- `BaselineLsmWalIndexSession`, `BaselineLsmAdmittedRecord`,
  `BaselineLsmAdmittedKey`, `BaselineLsmDurableInputs`,
  `BaselineLsmExecutionWitness`, and `BaselineLsmCompactionPlan`
- admitted WAL append and checkpoint publication receipts, manifest membership,
  store/root identity, segment generation, LSN coverage, frame digest, persisted
  path, byte count, and checksum
- layout's read-only physical compaction projection and downstream formal-model
  observation surfaces

### Required Work

Physical isolation and WAL remain the sole owners of compaction cutover,
tombstone retention, publication ordering, recovery visibility, stable-reader
protection, reclaim deferral, reclaim drain, and mutation-lane denial. Layout
may expose a read-only vocabulary projection for planning or formal modeling;
the projection may not add, omit, merge, split, or reinterpret a physical
case.

Introduce a WAL-owned persisted store handle or equivalent lower-authority
seam that owns baseline LSM membership. An append-only journal may exist as a
derived corruption-detecting cache, but reopen and replay must rederive or
readmit membership from authoritative WAL/checkpoint/manifest artifacts.

Persisted membership binds tenant and key scope, full canonical key bytes, WAL
store identity, segment id and generation, LSN range, frame and record digest,
checkpoint/manifest publication identity, admitted path under the store root,
persisted byte count, and checksum over the complete envelope.

Retirement and tombstone removal derive from authoritative manifest membership
over the same store, scope, checkpoint, coverage, component identity, and
stable-reader horizon. Equal-looking digests, copied rows, rewritten journal
lines, or layout-authored retirement declarations cannot retire membership.

Compaction ordinary operations themselves must be the source of the physical
case vocabulary. A static `all()` list or state-machine table may support
exhaustive iteration, but it cannot advertise a case that no production
operation emits. Mutation denials retain exact case identity through the
physical outcome and layout projection.

Formal-model input is an observation over owner-issued outcomes and durable
artifacts, not a production milestone handoff. Runtime crates must not expose
`S9` inventories, roadmap state machines, or generic transition arrays.

This phase owns the complete production join between layout observation and
physical/WAL authority. Every physical owner case and its layout projection
must be ordinarily reachable, and the critical identity, replay, retirement,
and locality boundaries must be proven here. Phase 37 owns the exhaustive
field-substitution cross-product, exact global case accounting, complete
compile-fail matrix, mutation testing, and large-scale simulations.

### Tests

- Destroy the derived in-memory index and journal cache, reopen from admitted
  WAL/checkpoint/manifest authority, and prove identical key-local membership,
  visibility, retirement, and exact counters.
- Reach every declared physical compaction success and denial through its
  ordinary owner operation. Preserve unique operation and case identity in the
  read-only layout projection; Phase 37 owns exact global set equality and the
  complete projection cross-product.
- Prove the physical-to-layout projection is bijective for each operation
  family with critical cutover, publication, visibility, reclaim, and
  mutation-denial cases. No projection may merge distinct lower-owner cases.
- Reject critical syntactically valid substitutions across security scope,
  canonical key, store identity, segment generation, LSN/checkpoint coverage,
  manifest membership, persisted path, and complete-envelope integrity during
  ordinary reopen/readmission. Phase 37 expands these axes into the exhaustive
  hostile field matrix.
- Reject early tombstone retirement, active-reader reclaim, copied manifest
  membership, equal-digest cross-store retirement, duplicate replay, and
  checkpoint/coverage disagreement.
- Prove key-local lookup, retirement, reopen, and compaction do not require a
  broad scan where locality-bounded behavior is claimed, using exact
  page/run/manifest/record/byte counters.
- Focused compile-fail proof must prevent layout, certification, and test
  support from constructing the principal physical outcome, durable
  membership, publication authority, or reclaim-readiness capabilities from
  raw fields. Phase 37 owns complete constructor and lifecycle-stage coverage.

### Closeout Gate

This phase is done when all physical compaction and durable-membership
capabilities exist, every declared owner case is ordinarily reachable,
physical operations and admitted WAL/manifest artifacts are the only sources
of truth, layout projection preserves lower-owner identity without
reinterpretation, and reopen reconstructs the same bounded membership without
certification or cache authority.

## Phase 37: Exhaustive Runtime And Hostile Coverage

### Purpose

Centralize the exhaustive observation, hostile cross-product, mutation, and
scale proof for the production capabilities completed in Phases 34-36. This is
the one phase that turns ordinary owner-case reachability into exact milestone
coverage rather than making each implementation phase independently repeat the
entire final standard.

### Relevant APIs

- owner-private case declarations and read-only case observations from every
  operation completed in Phases 34-36
- S.4.5 scenario descriptors, deterministic schedules, production drivers,
  observers, oracles, shrink traces, and exact counter receipts
- B-tree, LSM, blob, recovery, physical-compaction, security-scope, and
  compatibility scenario families
- compile-fail suites and structural source guards for layout authority

### Required Work

Each owner operation exposes a sealed observation identity suitable for cold
coverage accounting. The declared case set comes from the same private
declaration that defines the operation's cases; the harness cannot author or
extend it. The observed case set is populated only after the ordinary
production facade executes and the owner issues the outcome.

For every operation, require exact set equality:

```text
declared owner cases == cases observed through ordinary production execution
```

The matrix covers all success, denial, deferred, stale, rebind, readmission,
quarantine, compatibility, bootstrap, compaction, publication, retirement, and
reclaim cases. Equivalence classes are allowed only when the spec names the
shared algorithm and proves the class preserves authority, denial behavior,
counter shape, and recovery semantics. One representative case per subsystem
is not coverage.

Build scenarios from existing harness architecture: immutable scenario
descriptors, deterministic schedule generation, production-backed drivers,
separate observers and oracles, replay bundles, shrinkable failure traces, and
exact counter evidence. Test support may prepare bytes, persisted artifacts,
fault schedules, hostile declarations, and workload inputs. It may not issue
outcomes, witnesses, membership, publication, retirement, readiness, or case
observations.

The hostile matrix must include forged/cross-scope authority, copied payload
and receipt pairing, declaration-order drift, payload-derived owner selection,
static case-table substitution, stale/rebind bypass, false absence, hidden
broad scans, B-tree separator misrouting, LSM tombstone loss, corruption-to-
empty conversion, same-root cross-store redirection, duplicate replay,
recomputed checksums, and checkpoint/coverage disagreement.

Add mechanical residue guards for milestone vocabulary, broad generic outcomes,
crate-wide outcome issuers, business logic in facades or `mod.rs`, independent
transition arrays, certification-authored lower evidence, and compatibility
aliases preserving removed paths.

Phase 37 owns exhaustive owner-family accounting, all named hostile
substitution combinations, the complete compile-fail authority matrix,
mutation testing, shrink/replay evidence, and large-scale simulation. If this
work discovers a missing production capability or authority boundary, the fix
lands in the permanent owner from Phases 34-36 and its focused tests are rerun;
the harness must not compensate by authoring a synthetic outcome or alternate
authority lane.

Performance proof is algorithm-aware. Scale-sensitive slopes are required for
lookup, mutation, rebuild, B-tree traversal, LSM lookup/compaction,
publication, reopen/locality, and blob-touching layout paths whose work should
vary with data shape. Migration/rollback metadata, compatibility admission,
quarantine classification, readmission, and bootstrap use exact fixed-cost
counters unless the production algorithm actually traverses variable data.
Workload size alone must not be used to manufacture a meaningless slope.

### Tests

- Exact matrix proof for every owner family, including explicit failure output
  naming missing and unexpectedly observed cases.
- Replay/convergence proof across independent process reconstruction, schedule
  order, registry order, and supported access-family equivalence classes.
- Runtime forgery proof for every copied, cross-scope, stale, reordered,
  recomputed, duplicate, and projection-as-authority attack named above.
- Scale simulations proving claimed locality and amplification slopes with
  exact structural counters for scale-sensitive algorithms, including S.7
  multi-GB blob fixtures where layout behavior touches blob storage. Fixed-cost
  metadata transitions instead prove exact invariant counter shapes across
  adversarial inputs and replay.
- Compile-fail proof that external callers, sibling domains, certification, and
  test support cannot issue outcomes, construct owner cases, mint authority,
  skip lifecycle stages, or use a projection as a stronger input. Every case
  must assert the intended privacy/type failure and reject `E0463` evidence.
- Mutation testing or equivalent deliberate fault injection proving each
  critical oracle fails when its production invariant is removed or inverted.
- Test-quality QA must inspect the matrix for synthetic execution, private
  constructor use, assertion tautology, copied production logic, and weak
  representative coverage before the phase can close.

### Closeout Gate

This phase is done when every declared owner case has exact ordinary-production
evidence, the complete hostile and compile-fail matrices reject every named
forgery at its real boundary, mutation tests demonstrate oracle sensitivity,
algorithm-sensitive scale claims and fixed-cost metadata claims have honest
exact counters, and test support cannot manufacture the facts it observes.

## Phase 38: Certification Courtroom, Formal-Model Input, And S.8 Closeout

### Purpose

Close S.8 through certification-side verdicts over ordinary owner evidence and
produce formal-model observations without introducing a production milestone
handoff, generic transition ontology, or hidden layout fallback.

### Relevant APIs

- production evidence from every permanent operation completed in Phases
  34-36 and exact coverage receipts from Phase 37
- certification courtroom scenario, transcript, oracle, verdict, hazard,
  residual-risk, and closeout-report surfaces
- read-only formal-model observations over owner-issued outcomes, durable
  membership, publication ordering, and recovery/reclaim behavior
- Foundational closeout must consume:
  `FoundationalCounterBackedPerformanceReceipt`,
  `FoundationalPerformanceReportPlan`,
  `FoundationalMaterializedPerformanceReport`, certified/readmitted
  performance bundles, boundary-evidence readiness, and canonical readiness
- Proof closeout must consume:
  executed proof progression, non-success outcome evidence, stale/rebind
  evidence, and compile-fail proof that weaker stages cannot enter stronger
  APIs
- Store closeout must cover:
  page/frame/segment/extent/root-manifest/allocation/placement families,
  WAL/checkpoint/replay/recovery/crash-boundary families,
  snapshot/branch/continuation families, blob/chunk/streaming/reachability/
  retention/reclaim/compaction families, maintenance/tiering/I/O scheduler
  families, security/custody/export/import/offline families, and
  compatibility/version families

### Required Work

Certification acts only as courtroom. It consumes immutable owner observations,
durable artifacts, executed counter evidence, replay bundles, and hostile
scenario results. It may assemble coverage matrices, hazard rows, verdicts,
diagnostics, residual-risk records, and terminal reports. It may not mint
layout outcomes, physical compaction outcomes, WAL/manifest membership,
publication or retirement authority, readiness, or formal-model truth by
pairing independently supplied fields.

Build the S.8 hazard inventory in certification, not runtime. It must cover at
least hidden broad scans, stale-as-exact acceptance, partial false absence,
projection-as-authority, cross-scope index reuse, corruption converted into an
empty result, derived rollback authority, B-tree separator misrouting, LSM
tombstone loss, cache admission bypass, compatibility-path readiness, copied
counter evidence, static case substitution, and certification-authored lower
authority. Every hazard names detection, containment, recovery action, executed
proof lane, and any genuinely residual risk.

Formal modeling receives a cold, read-only inventory derived from owner
observations and durable artifacts. It may identify operations, cases,
ordering, and invariant relationships, but it cannot become runtime admission
and cannot require production types named for S.8, S.9, phases, milestones, or
handoffs. There is no `StorageFoundationS9LayoutHandoff` or production
state-machine inventory.

Perform a workspace-wide hard-cutover audit. Production Rust identifiers,
module paths, filenames, exports, and public code documentation use permanent
domain vocabulary. Root facades and `mod.rs` files aggregate only. Compatibility
aliases, generic transition facts, crate-wide issuers, static authority tables,
and certification/test authority shortcuts are removed rather than deprecated.

Phase 38 consumes the production evidence from Phases 34-36 and the exact
coverage receipts from Phase 37. It does not restate or independently rebuild
their owner-case matrices, hostile cross-products, or scale simulations. Its
job is to prove that certification can judge those artifacts without becoming
authority, that formal-model observations are faithfully derived, and that the
workspace has no surviving shortcut around the completed architecture.

Run final integration and traceability closeout across production facades,
owner-local visibility, Phase 37 coverage receipts, replay bundles,
planned-versus-observed performance evidence, B-tree and LSM invariants,
materialization/absence, rebuild/parity, durable membership, compaction
bijection, legacy disposition, hazard coverage, docs/roadmap consistency,
`qa-loop`, hostile test QA, and code-quality QA. A finding may reveal a real
defect in an earlier owner; repair that owner and regenerate the affected
evidence. Certification may not hide the defect, duplicate the owner, or turn
Phase 38 into another implementation authority. Repeat the affected focused
proof and all three final QA passes until clean.

### Required Verification

The exact command list may evolve with crate names, but closeout must include
focused checks equivalent to:

```text
cargo check -p worth-store-physical-isolation --features certification-authority
cargo check -p worth-store-layout-indexes
cargo check -p worth-store-certification
cargo test -p worth-store-layout-indexes --lib
cargo test -p worth-store-certification --test s8_layout_access_path_harness
cargo test -p worth-store-certification --test s8_layout_corruption_rebuild
the physical compaction exact-case and projection-bijection suites
the WAL/manifest reopen, membership, and retirement suites
the relevant compile-fail UI suite for S.8 layout authority
the workspace Rust line-cap and code-quality guards
```

### Closeout Gate

`S.8` is not closed until each admitted durable artifact family has:

- a permanent owner, consumer, public facade, courtroom/test-support boundary,
  and mechanically forbidden shortcut path
- declared layout strategy
- admitted physical key-domain law
- tested strategy invariant suite, including baseline B-tree/LSM where claimed
- declared access shapes
- declared authority or derivation posture
- declared authority role, accuracy class, and tenant/key-scope partition
- materialization state, coverage basis, freshness, and absence-proof behavior
- deterministic plan-selection receipt and plan fingerprint
- pre-execution cost budget and planned counter envelope
- bounded access-path counters
- planned-versus-observed counter receipt
- live maintenance/publication protocol where derived state is maintained
- corruption and rebuild behavior
- version and migration posture
- trust-boundary behavior where relevant
- bootstrap/catalog behavior where relevant
- legacy-surface disposition where relevant
- certification-side hazard inventory coverage
- compile-fail protection against raw construction
- runtime tests proving the honest path and adversarial denials

No required family may fall back to an implicit whole-store scan where the
roadmap claims indexed, locality-bounded, streaming, or strategy-bound access.

Closeout also requires all of the following:

- no production identifier, module, file, export, or public code comment uses
  roadmap order as domain vocabulary;
- no sibling module, certification crate, test-support crate, payload,
  projection, denial, receipt, or copied witness can issue an owner outcome;
- every owner case has exact ordinary-production coverage;
- physical compaction projection and durable membership preserve lower-owner
  authority without reinterpretation;
- certification remains courtroom and formal-model input remains observation;
- focused verification, full workspace tests, `qa-loop`, hostile test QA, and
  code-quality QA complete without warnings or unresolved findings.
