# Worth Store Test Requirements

## Scope

This document defines the certification-grade store test requirements for
`worth-store`.

Roadmap 2 physical certification is additionally governed by
[test-requirements-2.md](./test-requirements-2.md),
which defines the adversarial simulation harness requirements needed to make
the `S.*` suites realistic rather than minimum-effective-dose tests.

Unlike the bridge document, this one begins at Milestone 1 because Worth Store
does not yet have an earlier shipped foundation whose proof obligations are
already closed elsewhere.

It governs milestone closeout for:

- Milestone 1 through Milestone 22, including Milestone 3.5, Milestone 3.6,
  and Milestones 13.1 through 13.3
- Roadmap 2 storage-foundation sequences `S.0` through `S.12`
- the generic certification program in Milestone 23
- the domain certification program in Milestone 24

## Purpose

Worth Store cannot be treated as shipped merely because:

- commits persist in happy-path tests
- recovery works once
- a snapshot round-trip succeeds
- blobs save and load
- replication transfers some files

The store is making much stronger claims:

- canonical durable artifacts remain authoritative across backends
- durable mode crash recovery is exact
- embedded mode persists artifacts without stealing semantic authority
- snapshots, deltas, and structural blocks remain rebuildable
- live-query continuation converges to the same truth as fresh reads
- first-class subscription-support artifacts remain exact, degraded, or
  non-resumable in explicit typed ways rather than through cursor folklore
- retention, compaction, and tiering never become shadow authority
- replication and import/export preserve canonical truth
- derived artifacts remain honest about accuracy and rebuildability
- budget controls fail explicitly instead of allowing silent degradation
- physical storage does not rely on full-store heap materialization,
  serde-loaded domain objects, backend-private residue guessing, unbounded
  memory, or unverified OS writeback behavior

Those are adversarial claims. They need certification tests, not only behavior
checks.

## Global Adversarial Constraint

The store certification suite must prove the following:

> Under crash-restart loops, backend variation, branch pressure, retention
> pressure, schema evolution, lineage-bearing truth, cursor resume, live-query
> continuation, first-class subscription-support survival, bulk-ingest
> interruption, replication/import-export, tier movement, derived-artifact
> rebuild, blob pressure, and budget exhaustion,
> the store must preserve canonical truth meaning, replay-safe artifacts,
> explicit authority boundaries, typed failures, and machine-checkable
> diagnostics without allowing physical layout, caching, heuristics, or
> acceleration structures to redefine truth.

If a store surface works only on one backend, one recovery path, one branch
shape, one retention profile, or one happy-path ingest, it is not certified.

## Meta-Rules

These tests are certification tests. They must:

- emit machine-checkable artifacts, not "logs looked good"
- compare canonical digests across independently produced equivalent runs
- compare intentionally different lanes and prove inequality where meaning
  truly differs
- prove typed failure localization for rejection paths
- prove derived artifacts are rebuildable from their declared authority basis
- prove diagnostics richness changes retained detail, not semantic truth
- verify exact counter contracts where the milestone makes boundedness or scale
  claims
- prove replay equivalence across different valid recovery/materialization paths

These requirements are mandatory, not advisory.

### Beta Gate Rule

Passing the milestone-mapped suites is necessary for milestone closeout.
Passing the cross-cutting beta suites in this document is required before Worth
Store can be considered beta-ready.

The beta gate exists because milestone-local correctness is not enough. A store
can pass clean parity tests and still fail under:

- long-running churn
- corruption and partial damage
- hostile interleavings
- upgrade/version-skew pressure
- operational boundedness failures under scale
- unimagined randomized histories
- misleading operator diagnostics

### Global Certification Shape

Every named suite must define at least these lanes unless it explicitly states a
stronger reason not to:

- `control_lane` — the canonical no-failure baseline
- `hostile_lane` — the adversarial variation being certified
- `rebuild_or_replay_lane` — replay, rebuild, restore, resume, or restart from
  canonical artifacts

### Mandatory Assertion Classes

Every named suite must include all applicable assertion classes:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally different semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden residue, forbidden fallback, or
  forbidden authority drift
- resource-envelope assertions for physical lanes: resident bytes, pinned pages,
  dirty pages, allocation count, WAL tail, recovery time, foreground
  interference, read amplification, write amplification, and streamed bytes

### Certification Bundle Rules

At minimum, certification bundles should emit digests or structured reports for
the fields appropriate to the suite's scope, including from this common pool:

- `truth_digest`
- `history_digest`
- `branch_heads_digest`
- `artifact_digest`
- `subscription_support_digest`
- `replay_digest`
- `restore_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`
- `physical_layout_report`
- `resource_envelope_report`
- `latency_envelope_report`
- `corruption_localization_matrix`
- `hardware_assumption_report`
- `formal_model_report`
- `hazard_analysis_report`

The exact bundle shape may vary by suite, but it must be sufficient for offline
evaluation of pass/fail without ambient runtime state.

### Counter Assertion Rule

Whenever a milestone claims boundedness, scale-path correctness, rebuild
correctness, retention discipline, or pacing discipline, the suite must assert
exact counter values for representative scenarios, including counters that must
remain zero.

### Anti-Fake-Test Rule

The following do not count as certification:

- asserting that a run completed successfully
- asserting only that a digest is present or non-empty
- comparing a value only to itself from the same run
- using logs as the primary proof artifact
- validating only a happy-path lane
- validating only a failure lane with no control basis
- using the same in-memory object graph as both writer and verifier
- proving physical behavior only through a backend that does not exercise the
  physical page/frame/chunk path
- testing large objects or large stores with inputs smaller than the declared
  memory budget
- accepting deserialization failure as corruption localization
- accepting elapsed-time-only benchmarks without explanatory structural counters

## Milestone-To-Suite Map

| Milestone | Required Named Suite(s) |
| --- | --- |
| M1 | Durable artifact authority equivalence test |
| M2 | Operating mode contract parity test |
| M3 | WAL crash boundary exactness test |
| M3.5 | Durable media and write-path certification test |
| M3.6 | Adversarial crash recovery and recovery source precedence test |
| M4 | Snapshot-plus-tail restore equivalence test |
| M5 | Branch delta proportionality and replay parity test |
| M6 | Aspect-layout narrowing and structural-block dedup integrity test |
| M7 | Schema/lineage/cursor durability test |
| M8 | Live-query basis continuation equivalence test |
| M9 | Bulk ingest and transform resume parity test |
| M10 | Retention/compaction/reclaim parity test |
| M11 | Background maintenance isolation and scheduling test |
| M12 | Artifact format evolution and rolling compatibility test |
| M13 | Tiering and working-set non-authority test |
| M13.1 | Durable subscription-support artifact and resume-contract test |
| M13.2 | Subscription-support retention/replication/compatibility/maintenance test |
| M13.3 | Subscription-support accuracy and certification test |
| M14 | Replication capsule equivalence and integrity test |
| M15 | Extensible durable artifact family containment test |
| M16 | Time-travel diff and merge-assistance parity test |
| M17 | Derived artifact accuracy classification test |
| M18 | Analysis checkpoint basis parity test |
| M19 | Correspondence/locality non-authority test |
| M20 | Blob identity retention and replication parity test |
| M21 | Budget admission honesty test |
| M22 | Operator repair, audit, and forensic recovery test |

Each milestone is not closeable until its required named suite passes.

## Roadmap 2 Storage-Foundation Suite Map

Roadmap 2 sequences are closeout gates for the physical database substrate.
They are not optional beta polish and they are not satisfied by the original
milestone suites unless the physical evidence below is emitted.

| Sequence | Required Named Suite(s) |
| --- | --- |
| S.0 | Shipped store capability reclassification test |
| S.1 | Physical page/segment/extent substrate test |
| S.2 | Buffer-pool and bounded-memory certification test |
| S.3 | Physical integrity and corruption-localization test |
| S.4 | WAL/checkpoint/LSN recovery-physics test |
| S.5 | Physical isolation and stable-read-plan interleaving test |
| S.6 | Hardware-aware I/O and foreground-QoS test |
| S.7 | Native blob chunk-store streaming and retention test |
| S.8 | Index/layout/access-path amplification test |
| S.9 | Formal model conformance test |
| S.10 | Operational safety, PITR, offline-verifier, and forensics test |
| S.11 | Security, tenant-boundary, key, and auditability test |
| S.12 | Physical database certification and performance program |

Each Roadmap 2 suite must include:

- `control_lane`: clean physical execution under declared backend assumptions
- `hostile_lane`: physical fault, pressure, or interleaving condition
- `reopen_lane`: reopen from persisted bytes, not from live heap state
- `offline_verifier_lane`: where applicable, verification without trusting the
  live store runtime
- `semantic_parity_lane`: compare logical truth against canonical authority
  after physical execution
- `forbidden_shortcut_lane`: prove the implementation did not satisfy the suite
  through heap materialization, backend residue guessing, unbounded allocation,
  or unsupported backend capability

Every Roadmap 2 suite must emit at least:

- `artifact_digest`
- `failure_digest`
- `counter_snapshot`
- `physical_layout_report`
- `resource_envelope_report`
- `hardware_assumption_report`

Additional outputs are required where the suite names corruption, latency,
formal modeling, hazard analysis, security, or operator repair.

### S.0. Shipped Store Capability Reclassification Test

Purpose

Prove that already-shipped Store milestones and backends are classified
honestly before physical foundation work starts.

Scenario

- audit Milestones 1 through 13.3 and their closeouts
- classify each backend as bootstrap, semantic-certification, compatibility,
  physical-foundation, or platform-grade
- identify every claim that implies physical database behavior

Must verify

- semantic guarantees remain listed separately from physical guarantees
- no heap-shaped backend is described as platform-grade
- every deferred physical guarantee maps to Roadmap 2 `S.*`

Required verification output

- `capability_tier_matrix`
- `semantic_physical_claim_report`
- `deferred_physical_guarantee_map`
- `counter_snapshot`

Pass condition

The existing roadmap and closeout language can be read without implying that
bootstrap persistence is already the platform-grade physical database.

### S.1. Physical Page/Segment/Extent Substrate Test

Purpose

Prove that the platform-grade backend persists and locates records through the
physical page/segment/extent substrate.

Scenario

- write representative authoritative and derived artifacts
- close and reopen the store
- scan and locate records by physical identifiers
- attempt the same lanes with full-store heap materialization disabled

Must verify

- records are persisted, reopened, scanned, and located through physical ids
- page/frame headers, generation counters, root manifests, and free-space
  structures are present and internally consistent
- stale generations are detected
- heap-shaped paths are excluded from the platform-grade backend tier

Required verification output

- `physical_layout_report`
- `artifact_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

The backend can perform required physical operations through pages, segments,
extents, frames, and manifests without whole-store domain deserialization.

### S.2. Buffer-Pool And Bounded-Memory Certification Test

Purpose

Prove that stores larger than memory operate inside declared resident-memory and
allocation budgets.

Scenario

- configure a memory budget smaller than the test store
- perform reads, writes, reopen, recovery, compaction planning, scrub, and blob
  streaming
- inject pressure through pinned pages, dirty pages, and eviction churn

Must verify

- resident bytes, pinned pages, dirty pages, and allocations stay within budget
- admitted hot paths meet zero-allocation or exact-allocation contracts
- pressure produces defer, deny, eviction, or pacing outcomes before OOM

Required verification output

- `resource_envelope_report`
- `counter_snapshot`
- `failure_digest`

Pass condition

The store remains operational and bounded when physical data exceeds the
configured memory envelope.

### S.3. Physical Integrity And Corruption-Localization Test

Purpose

Prove that damaged bytes fail at the physical boundary before logical decode.

Scenario

Inject corruption into:

- page headers
- frame payloads
- WAL frames
- segment manifests
- index pages
- blob chunks
- generation counters

Must verify

- every corruption class localizes to the correct physical boundary
- logical decoders are skipped for damaged physical bytes
- rebuildable derived damage and damaged authority are distinguished
- quarantine records and repair inputs are emitted

Required verification output

- `corruption_localization_matrix`
- `failure_digest`
- `physical_layout_report`
- `counter_snapshot`

Pass condition

Physical damage is detected and localized before semantic interpretation.

### S.4. WAL/Checkpoint/LSN Recovery-Physics Test

Purpose

Prove that recovery follows WAL, pageLSN, checkpoint, and flush-ordering rules.

Scenario

Crash at least around:

- WAL append
- data-page flush
- checkpoint manifest write
- checkpoint manifest cutover
- compaction cutover
- acknowledgment
- directory or rename durability boundary

Must verify

- acknowledged writes recover exactly once
- unacknowledged partial publications are rejected or completed through typed
  rules
- recovery is bounded by checkpoint interval and WAL tail
- replay is idempotent
- no lane trusts backend residue as authority

Required verification output

- `restore_digest`
- `failure_digest`
- `counter_snapshot`
- `recovery_physics_report`

Pass condition

Crash recovery is deterministic, bounded, and physical-rule driven.

### S.5. Physical Isolation And Stable-Read-Plan Interleaving Test

Purpose

Prove that physical readers remain stable while maintenance moves or rewrites
storage.

Scenario

Run foreground reads during:

- compaction
- checkpointing
- reclaim
- tier movement
- blob migration
- restart during cutover

Must verify

- readers do not observe half-published roots
- stale page generations reject or retry explicitly
- protected pages/chunks are not reclaimed
- latch, lease, epoch, and blocked-reclaim counters explain the interleaving

Required verification output

- `interleaving_matrix`
- `failure_digest`
- `counter_snapshot`
- `physical_layout_report`

Pass condition

Maintenance changes cost or timing only; it does not make physical read plans
observe unstable bytes.

### S.6. Hardware-Aware I/O And Foreground-QoS Test

Purpose

Prove that physical backend capability tiers and foreground latency envelopes
are real.

Scenario

Run foreground reads and writes while executing:

- compaction
- checkpointing
- scrub
- replication preparation
- blob ingest
- blob migration

Exercise each admitted backend capability tier.

Must verify

- backend durability assumptions are declared and checked
- unsupported capability claims fail typed
- foreground wait and latency stay inside declared envelopes
- background work yields, paces, or denies itself through explicit counters

Required verification output

- `latency_envelope_report`
- `hardware_assumption_report`
- `counter_snapshot`
- `failure_digest`

Pass condition

Foreground behavior is protected by explicit I/O contracts, not OS scheduler
hope.

### S.7. Native Blob Chunk-Store Streaming And Retention Test

Purpose

Prove that blob storage is native, chunked, streaming, checksummed, resumable,
deduped, and retention-safe.

Scenario

- stream a multi-GB blob through ingest, read, verify, export, and import
- interrupt and resume writes
- dedupe repeated content
- corrupt individual chunks
- move chunks across tiers
- reclaim orphaned chunks
- partially replicate blob-bearing artifacts

Must verify

- memory remains constant with respect to blob size
- chunk checksums and content digests both verify
- interrupted writes resume or reject typed
- referenced chunks survive retention and orphaned chunks reclaim safely

Required verification output

- `artifact_digest`
- `corruption_localization_matrix`
- `resource_envelope_report`
- `counter_snapshot`

Pass condition

Large blobs behave as native physical database objects, not sidecar files.

### S.8. Index/Layout/Access-Path Amplification Test

Purpose

Prove that each artifact family has an honest physical layout and access-path
cost model.

Scenario

- exercise point, range, prefix, scan, streaming, rebuild, and migration paths
  for each admitted layout family
- corrupt an index and rebuild from the declared authority basis
- attempt broad scans where bounded indexes are required

Must verify

- layout family declarations match observed access paths
- read and write amplification counters match expectations
- broad scans are rejected where the roadmap requires bounded access
- rebuilt indexes preserve semantic parity

Required verification output

- `physical_layout_report`
- `counter_snapshot`
- `artifact_digest`
- `failure_digest`

Pass condition

Physical access paths are explicit, measured, and rebuildable.

### S.9. Formal Model Conformance Test

Purpose

Prove that modeled physical state machines and implementation states remain
aligned.

Scenario

For each required model, exercise implementation lanes that cover legal and
illegal transitions:

- WAL/checkpoint/page flush
- recovery source precedence
- compaction cutover
- read leases and reclaim barriers
- repair/quarantine transitions
- replication/import admission where physical evidence matters

Must verify

- checked model artifacts exist
- implementation states map to model states
- illegal modeled transitions fail typed in code
- certification lanes would fail if transition rules were weakened

Required verification output

- `formal_model_report`
- `failure_digest`
- `counter_snapshot`

Pass condition

Formal models are executable engineering constraints, not detached documents.

### S.10. Operational Safety, PITR, Offline-Verifier, And Forensics Test

Purpose

Prove that damaged or restored stores can be inspected and repaired without
trusting the live runtime.

Scenario

- create online backups
- restore to point-in-time boundaries
- damage pages, chunks, manifests, and derived families
- run offline verifier
- generate repair, quarantine, rollback, and forensic bundles

Must verify

- trusted authority, degraded derived artifacts, quarantined regions, and
  unrecoverable damage are distinguished
- PITR candidates are exact and bounded
- operator repair plans do not mutate authority implicitly
- forensic bundles are machine-checkable

Required verification output

- `forensic_bundle_digest`
- `repair_plan_report`
- `quarantine_report`
- `failure_digest`
- `counter_snapshot`

Pass condition

Operators can determine trusted truth and admissible repair from offline
evidence.

### S.11. Security, Tenant-Boundary, Key, And Auditability Test

Purpose

Prove that physical security and compliance behavior survive real operations.

Scenario

- encrypt pages and blob chunks
- rotate keys
- attempt wrong-key reads
- exercise tenant-scoped placement, backup, restore, repair, and replication
- tamper with audit logs
- exercise secure-delete capability declarations

Must verify

- checksum success is not treated as authenticity success
- tenant boundaries remain visible under repair and restore
- audit-chain tampering localizes and fails typed
- key rotation preserves admitted access and rejects stale or wrong access

Required verification output

- `security_boundary_report`
- `hardware_assumption_report`
- `failure_digest`
- `counter_snapshot`

Pass condition

Security, tenancy, deletion, and auditability are store contracts, not
deployment assumptions.

### S.12. Physical Database Certification And Performance Program

Purpose

Prove that Roadmap 2 has produced a real physical database substrate.

Scenario

Run the complete physical certification program:

- power-loss simulation
- torn writes
- byte flips
- stale generations
- partial flushes
- large-store bounded-memory runs
- foreground load with background work
- recovery-time envelope tests
- blob-scale streaming
- cross-backend parity
- hazard-analysis lanes

Must verify

- every physical readiness claim has a declared backend/hardware assumption
- every performance envelope has explanatory counters
- residual risks are named in FMEA/STPA-style evidence
- unsupported capability claims fail typed or are marked non-platform-grade debt

Required verification output

- `physical_database_certification_bundle`
- `hazard_analysis_report`
- `hardware_assumption_report`
- `resource_envelope_report`
- `latency_envelope_report`
- `counter_snapshot`

Pass condition

The physical substrate is certified enough to unblock post-13.3 platform
roadmap work.

Milestones `23` and `24` are intentionally not listed in this table.

- `Milestone 23` is the generic certification program and is governed by the
  dedicated `Generic Store Certification Program` section in this document
  rather than by one single milestone-closeout suite row.
- `Milestone 24` is the domain certification program and is governed by the
  dedicated `Domain Store Certification Program` section in this document
  rather than by one single milestone-closeout suite row.

The table above is therefore complete for the feature and platform milestones
that close through one primary named suite. Milestones `23` and `24` close
through their broader certification-program requirements.

## Cross-Cutting Beta Suites

These suites are not tied to one milestone only. They are required for beta
readiness after the milestone-mapped suites are already passing.

### Beta 1. Long-Running Soak And Churn Test

Purpose

Prove that the store remains sane over long-running mixed-operation churn, not
just that it lands in the correct final state once.

Scenario

Run a long-duration workload mixing:

- branch creation and deletion
- snapshot creation and rebuild
- compaction and reclaim
- cursor movement and live-query continuation
- tier changes
- replication/import-export activity
- blob churn
- budget pressure

Must verify

- final truth parity still holds
- counters remain within declared operating ranges
- debt counters do not grow without explicit policy response
- WAL growth, memory, latency, compaction debt, rebuild debt, and background
  work remain sane for the admitted workload class
- no silent degradation appears over time

Required verification output

- `truth_digest`
- `replay_digest`
- `counter_snapshot`
- `soak_debt_report`
- `resource_envelope_report`
- `latency_envelope_report`

Pass condition

The store remains semantically correct and operationally sane under sustained
mixed churn.

### Beta 2. Corruption Injection And Partial-Damage Localization Test

Purpose

Prove that partial corruption fails sharply, localizes correctly, and never
bluffs semantic confidence.

Scenario

Inject at least these corruption classes:

- corrupted snapshot fragment
- corrupted structural block
- corrupted digest record
- truncated WAL
- partially written checkpoint
- missing derived artifact family
- blob file present with missing metadata
- blob metadata present with missing file

Must verify

- corruption is localized to the correct artifact boundary
- valid authoritative truth remains recoverable where the model says it should
- unrecoverable states fail explicitly and typed
- the store never treats a damaged derived artifact as authoritative truth

Required verification output

- `failure_digest`
- `corruption_localization_matrix`
- `artifact_boundary_report`
- `restore_digest`
- `diagnostics_digest`

Pass condition

Corruption is either rebuilt safely or rejected sharply with correct
localization.

### Beta 3. Hostile Interleaving Abuse Test

Purpose

Prove that ugly legal interleavings do not break correctness or hide authority
drift.

Scenario

Run adversarial interleavings such as:

- reads during compaction
- live-query continuation during tier movement
- branch mutation during snapshot materialization
- replication during reclaim
- bulk transform while budget pressure rises
- embedded checkpoint reception concurrent with durable export

Must verify

- all equivalent lanes preserve canonical truth
- typed failures occur where operations are intentionally non-admitted
- no hidden cross-phase residue appears
- no interleaving creates a shadow authority path

Required verification output

- `truth_digest`
- `failure_digest`
- `interleaving_matrix`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Interleaving affects cost or admitted timing only, not truth meaning.

### Beta 4. Upgrade, Compatibility, And Version-Skew Test

Purpose

Prove that upgrade-path hostility is explicit and safe rather than optimistic.

Scenario

Exercise:

- old data with new code
- new artifact with old reader rejection
- schema boundary compatibility
- mixed artifact versions
- partial multi-process or multi-machine upgrade states

Must verify

- compatible upgrades remain semantically exact
- incompatible upgrades fail explicitly and typed
- version-skew does not silently drift into partial truth acceptance
- capsule and replication readers honor declared compatibility boundaries

Required verification output

- `artifact_digest`
- `failure_digest`
- `compatibility_matrix`
- `version_skew_report`
- `diagnostics_digest`

Pass condition

Upgrade and version-skew behavior is explicit, bounded, and non-ambiguous.

### Beta 5. Recovery And Operational Boundedness Test

Purpose

Prove that the store stays correct within the operational envelope it claims to
support, not only in principle.

Scenario

Run admitted scale workloads and certify:

- recovery time
- snapshot-plus-tail restore amplification
- compaction debt behavior
- live-query catch-up debt
- branch-delta traversal before rewrite

Must verify

- recovery completes within the declared budget for the admitted workload class
- restore and traversal stay within declared amplification bounds
- debt cannot grow unbounded without explicit policy trigger
- boundedness claims are enforced by counters and policy, not hope

Required verification output

- `restore_digest`
- `counter_snapshot`
- `boundedness_report`
- `policy_trigger_report`
- `resource_envelope_report`

Pass condition

The store remains correct inside its declared operational envelope, and
explicitly surfaces when that envelope is exceeded.

### Beta 6. Fuzz And Property-Based Hostile History Test

Purpose

Prove that the store survives adversarial history shapes beyond curated
handwritten scenarios.

Scenario

Generate random but legal combinations of:

- commit histories
- branch topologies
- schema transitions
- crash points
- replay/rebuild path selection
- retention policies within legal bounds

Must verify

- core invariants hold across generated workloads
- replay/rebuild equivalence holds where required
- failure classes remain typed and localizable
- no generated case can smuggle derived artifacts into authority

Required verification output

- `truth_digest`
- `replay_digest`
- `failure_digest`
- `fuzz_invariant_report`
- `counter_snapshot`

Pass condition

Randomized hostile generation preserves the same architectural invariants as the
curated suites.

### Beta 7. Operator Observability Truthfulness Test

Purpose

Prove that operator-facing diagnostics tell the truth about what is happening.

Scenario

Exercise success, degradation, corruption, rejection, and budget-triggered
paths across multiple artifact families and operating modes.

Must verify

- diagnostics identify the correct failure family
- diagnostics localize the correct artifact boundary
- diagnostics report the correct policy trigger
- diagnostics distinguish degraded derived behavior from truth failure
- observability richness changes retained detail only, not semantic diagnosis

Required verification output

- `diagnostics_digest`
- `failure_digest`
- `operator_truthfulness_matrix`
- `policy_trigger_report`
- `counter_snapshot`

Pass condition

Operators can correctly distinguish truth failure, derived degradation, budget
pressure, and rebuildable corruption from the emitted diagnostics alone.

## Named Certification Suites

### 1. Durable Artifact Authority Equivalence Test

Purpose

Prove that canonical durable artifacts remain authoritative across backends and
rebuild paths.

Scenario

- persist the same canonical history through at least two backends or backend
  configurations
- rebuild from canonical authoritative artifacts
- compare the original and rebuilt stores

Strict interpretation for Milestone 1 closeout

- configuration variation inside one implementation family is necessary but not
  sufficient
- at least one parity lane must use a backend family that is structurally
  distinct from the baseline implementation, not merely a different file format
  or storage mode of the same internal state engine
- for Milestone 1, an in-memory baseline plus a file-backed encoding of that
  same state engine does not by itself satisfy the strongest closeout reading

Must verify

- canonical commit and history artifacts match
- branch heads match
- replay and query-visible truth match
- backend-local layout differences do not change authoritative conclusions

Required verification output

- `truth_digest`
- `history_digest`
- `branch_heads_digest`
- `artifact_digest`
- `replay_digest`

Pass condition

All semantically equivalent lanes converge to identical authoritative truth.

### 2. Operating Mode Contract Parity Test

Purpose

Prove that durable mode and embedded mode persist the same canonical artifact
meaning despite different lifecycle ownership.

Strict interpretation

- durable mode and embedded mode must converge at the same canonical append
  boundary, not merely produce vaguely similar persisted results
- absent mode must be proven as a real no-store lane, not a store-facade lane
  configured to "do nothing"
- embedded checkpoint persistence must be proven non-authoritative: checkpoint
  intake must not create alternate commit or branch-head meaning

Scenario

- commit equivalent truth through durable mode and embedded mode
- persist an embedded checkpoint path
- run an absent-mode control lane with no store
- exercise hostile misuse lanes that attempt:
  - checkpoint-as-commit confusion
  - durable-path API use from embedded-only handles
  - embedded artifact intake through durable-only handles
  - store-dependent absent-mode construction

Must verify

- durable and embedded mode persist equivalent canonical artifacts
- embedded mode does not gain semantic authority
- absent mode remains free of ambient persistence coupling
- embedded checkpoints do not redefine commit or branch-head authority
- mode misuse fails explicitly and typed
- forbidden cross-mode work remains zero by exact counter assertion where the
  milestone claims zero work

Required verification output

- `artifact_digest`
- `diagnostics_digest`
- `mode_contract_matrix`

Required additional bundle content for this suite

- `counter_snapshot`
- `failure_digest`
- `checkpoint_authority_report`

Pass condition

Lifecycle ownership changes orchestration only, not artifact meaning; absent
mode remains truly no-store; and embedded checkpoint persistence remains
non-authoritative.

### 3. WAL Crash Boundary Exactness Test

Purpose

Prove that crashes around the durable commit boundary do not duplicate, lose,
or partially publish truth.

Scenario

- crash before durable WAL intent survives
- crash after durable WAL intent but before canonical result durability
- crash after canonical result durability but before authoritative append
- crash after authoritative publication but before acknowledgment
- crash before acknowledgment
- crash after acknowledgment
- repeat crash-restart loops across several commits

Must verify

- acknowledged commits survive exactly once
- unacknowledged commits do not publish partially
- crash-after-publication but pre-ack lanes retain already published truth
- repeated restart after a durable mutation is already closed becomes quiescent
  instead of reopening the same durable work forever
- crash recovery and rebuild remain equivalent
- corrupted WAL lanes fail explicitly and typed with machine-checkable failure
  output

Required verification output

- `truth_digest`
- `replay_digest`
- `restore_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Durable acknowledgment is exact and crash-safe.

### 3.5. Durable Media And Write-Path Certification Test

Purpose

Prove that admitted backend families expose real durable media semantics rather
than optimistic write folklore.

Scenario

- write representative durable publication units through at least two admitted
  backend families
- inject:
  - truncated tail
  - torn record
  - partial publication marker durability
  - directory-entry durability gap where relevant
  - unsupported durable family version
- compare startup scan and acknowledgment classification across lanes

Must verify

- record framing distinguishes clean, truncated, torn, and unsupported states
- acknowledgment never outruns the declared backend-family barrier classes
- backend variation changes mechanics only, not retained truth meaning
- integrity-valid but authenticity-invalid lanes fail explicitly and typed
- the certification bundle carries typed observed media and source-admission
  failures rather than only a rolled-up failure digest
- startup tail handling never silently rewrites or normalizes damaged media
  into a clean state

Required verification output

- `truth_digest`
- `artifact_digest`
- `write_path_digest`
- `failure_digest`
- `counter_snapshot`

Required additional bundle content for this suite

- `ack_boundary_report`
- `certification_summary`
- `media_barrier_matrix`
- `tail_validation_report`
- `observed_failures`

Required counter assertions

- `durable_frame_scan_count` exactly matches the scanned framing work for the
  representative startup lanes
- `durable_truncated_tail_count` increments only for truncated-tail lanes
- `durable_torn_write_count` increments only for torn-write lanes
- `durable_ack_barrier_violation_count` remains zero in clean admitted lanes
  and increments in explicit barrier-violation lanes

Pass condition

Durable acknowledgment depends on declared barrier classes and framed durable
bytes, not ambient platform optimism.

### 3.6. Adversarial Crash Recovery And Recovery Source Precedence Test

Purpose

Prove that crash recovery chooses the right source of truth, handles
interrupted maintenance honestly, and becomes quiescent once work is closed.

Scenario

- run crash-restart lanes with:
  - authoritative artifacts intact and WAL incomplete
  - WAL intact and authoritative publication incomplete
  - interrupted snapshot publication
  - interrupted compaction publication
  - interrupted reclaim publication
  - interrupted capsule publication
  - same-scope conflicting recovery sources
  - quarantine-required damaged-media lanes
  - salvage-admitted lanes where policy allows them
- repeat restart after a terminal recovery conclusion has already been emitted
- compare crash recovery, authoritative rebuild, quarantine, and salvage lanes
  where admitted

Must verify

- recovery source choice follows the declared precedence rules
- lower-precedence or newer-looking derived artifacts never outrank higher
  precedence authority
- interrupted maintenance output does not displace the last known-good input
  before complete publication
- repeated restart becomes quiescent for already-terminal work
- quarantine and salvage remain explicit degraded outcomes rather than silent
  ordinary success
- retained-without-acknowledgment lanes remain explicit degraded outcomes with
  operator-visible follow-up action rather than being collapsed into clean
  success
- backup/restore and disaster-recovery inputs honor compatibility and admitted
  source rules

Required verification output

- `truth_digest`
- `restore_digest`
- `failure_digest`
- `counter_snapshot`

Required additional bundle content for this suite

- `recovery_source_report`
- `maintenance_recovery_report`
- `degraded_state_report`
- `certification_summary`
- `compatibility_digest`
- `quiescence_report`
- `recovery_status_report`
- `observed_failures`

Required counter assertions

- `recovery_source_precedence_resolution_count` exactly matches the number of
  precedence decisions exercised in the representative hostile lanes
- `recovery_quiescent_restart_count` increments on second and later restarts of
  already-terminal work
- `recovery_non_quiescent_restart_count` remains zero once the lane should be
  quiescent
- `interrupted_maintenance_recovery_count` increments only for the admitted
  interrupted-maintenance lanes
- `recovery_quarantine_count` and `recovery_salvage_count` distinguish degraded
  recovery classes without fabricating clean restart work

Pass condition

Recovery is deterministic, precedence-driven, degraded-state honest, and
quiescent once work is closed.

### 4. Snapshot-Plus-Tail Restore Equivalence Test

Purpose

Prove that snapshot persistence and snapshot-plus-tail restore converge to the
same truth as full replay from canonical commits.

Scenario

- capture snapshots
- restore from snapshot plus suffix history
- compare at least one structurally distinct backend family lane
- delete a snapshot image and rebuild from authoritative artifacts
- inject at least these hostile snapshot failure lanes:
  - corrupted snapshot image
  - missing published snapshot image with basis still present
  - unsupported snapshot family or basis version

Must verify

- snapshot restore matches canonical replay
- rebuilt snapshots match original snapshot-visible truth
- snapshots remain derived, not authoritative
- backend variation does not change snapshot-visible truth
- delete-and-rebuild lanes converge to the same truth-visible result as the
  originally published snapshot family
- hostile snapshot failure lanes fail explicitly and typed rather than
  broadening into fallback authority

Required verification output

- `truth_digest`
- `restore_digest`
- `artifact_digest`
- `counter_snapshot`
- `failure_digest`

Required counter assertions

- `snapshot_read_tail_commit_count` and `snapshot_read_tail_replay_count`
  exactly match the admitted suffix width for representative snapshot-tail
  reads
- `snapshot_restore_tail_commit_count` and
  `snapshot_restore_tail_replay_count` exactly match the admitted suffix width
  for representative restores
- hostile snapshot failure lanes increment the appropriate integrity or basis
  mismatch counters without fabricating successful restore work

Pass condition

All equivalent restore paths converge to the same truth.

### 5. Branch Delta Proportionality And Replay Parity Test

Purpose

Prove that branch storage scales with semantic delta and replays to the same
truth as canonical commit history.

Scenario

- create many branches with small and large edits
- compare storage growth and replay outcomes
- rewrite delta stacks where admitted

Must verify

- no-edit branches remain near-free
- growth tracks delta rather than copied base size
- replay from branch-local deltas matches canonical replay

Required verification output

- `truth_digest`
- `history_digest`
- `delta_storage_report`
- `counter_snapshot`

Pass condition

Branch persistence is proportional and replay-safe.

### 6. Aspect-Layout Narrowing And Structural-Block Dedup Integrity Test

Purpose

Prove that aspect-aware layout improves admitted partial reads and structural
blocks deduplicate cross-branch structure without changing truth.

Scenario

- run admitted aspect-scoped reads and fallback reads
- create cross-branch overlap regions
- compare dedup and non-dedup lanes

Must verify

- admitted fast paths match authoritative truth
- fallback broadening remains explicit
- block dedup does not change replay or restore conclusions

Required verification output

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Physical acceleration changes cost only, not meaning.

### 7. Schema/Lineage/Cursor Durability Test

Purpose

Prove that restart preserves schema-boundary meaning, lineage meaning, cursor
truth, and embedded checkpoints.

Scenario

- persist schema transitions
- persist lineage-bearing histories
- resume from durable cursor checkpoints
- restore embedded-mode checkpoints

Must verify

- schema boundary conclusions survive restart
- lineage resolution survives restart
- cursor resume remains deterministic
- embedded checkpoints persist without semantic drift

Required verification output

- `history_digest`
- `artifact_digest`
- `replay_digest`
- `diagnostics_digest`

Pass condition

Stored support artifacts remain authoritative for their declared role.

### 8. Live-Query Basis Continuation Equivalence Test

Purpose

Prove that reading from a stable basis and continuing by durable cursor
converges to the same truth as a fresh full read.

Scenario

- read from a stable basis
- continue with varying fetch widths
- restart mid-continuation

Must verify

- continued truth matches fresh truth
- basis mismatches fail explicitly
- narrowing acceleration does not invent new change meaning

Required verification output

- `truth_digest`
- `restore_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Live-query continuation is exact for equivalent semantic workloads.

### 9. Bulk Ingest And Transform Resume Parity Test

Purpose

Prove that interrupted bulk programs resume to the same final canonical truth
with bounded memory and no alternate commit model.

Scenario

- interrupt ingest and transform jobs mid-run
- resume from progress checkpoints
- compare to a logically serial control lane

Must verify

- final truth matches the control lane
- chunk boundaries remain deterministic
- WAL recovery remains parity-safe for interrupted runs

Required verification output

- `truth_digest`
- `history_digest`
- `restore_digest`
- `counter_snapshot`

Pass condition

Bulk paths are resumable, deterministic, and canonical.

### 10. Retention/Compaction/Reclaim Parity Test

Purpose

Prove that compaction and reclaim preserve retained authoritative truth and do
not become a second authority source.

Scenario

- retain some windows and reclaim others
- compact under pressure
- compare pre- and post-compaction read/replay results

Must verify

- retained truth remains exact
- reclaimed derived artifacts are rebuildable where policy says they are
- compaction rewrites physical storage without changing retained truth

Required verification output

- `truth_digest`
- `restore_digest`
- `artifact_digest`
- `counter_snapshot`

Pass condition

Retention policy is physically real and semantically honest.

### 11. Background Maintenance Isolation And Scheduling Test

Purpose

Prove that background maintenance remains operationally bounded and foreground
safe rather than becoming the hidden scheduler of truth visibility.

Scenario

- execute admitted compaction, rebuild, snapshot refresh, and
  replication-preparation work under active foreground read and write load
- compare isolated foreground lanes against hostile backlog lanes
- exercise deferred, escalated, and restart-recovered maintenance lanes
- submit duplicate or superseded work and verify coalescing or cancellation
- run locality-bounded lanes and explicit cross-locality escalation lanes
- run a tier-move flood lane against compaction-cutover work

Must verify

- foreground truth-visible results remain equal across isolated and hostile
  backlog lanes for equivalent admitted work
- background maintenance does not silently broaden or delay foreground work
- duplicate or superseded work is coalesced or cancelled through explicit
  equivalence rules
- restart-recovered backlog re-enters through the same admission model as fresh
  work
- queue order and worker timing change cost only, not semantic truth
- tier work does not bypass foreground reservations or hiddenly outrank
  compaction cutover or retained rebuild work

Required verification output

- `truth_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`
- `scheduler_topology_report`
- `maintenance_interference_matrix`
- `debt_escalation_report`

Pass condition

Background maintenance is explicit, bounded, restart-honest, and
foreground-safe.

### 12. Artifact Format Evolution And Rolling Compatibility Test

Purpose

Prove that old artifacts with new code, new artifacts with old readers, and
mixed-version stores or replicas remain semantically exact where admitted and
fail explicitly where not admitted.

Scenario

- read older authoritative artifacts with newer code
- reject newer authoritative artifacts from older readers where compatibility
  is not declared
- exercise derived-artifact rebuild invalidation after format drift
- run mixed-version store or replica lanes during rolling upgrade
- exercise backup, restore, and disaster-recovery lanes across admitted version
  windows

Must verify

- compatible artifacts preserve authoritative meaning across version windows
- incompatible artifacts fail explicitly and typed rather than drifting into
  partial truth acceptance
- derived families rebuild or invalidate exactly where the compatibility model
  requires
- rolling-upgrade and restore readers honor declared compatibility boundaries
- deserialization success alone is not accepted as compatibility proof

Required verification output

- `artifact_digest`
- `failure_digest`
- `compatibility_matrix`
- `version_skew_report`
- `diagnostics_digest`

Pass condition

Compatibility is explicit, machine-checkable, and semantically exact where
admitted.

### 13. Tiering And Working-Set Non-Authority Test

Purpose

Prove that hot/warm/cold movement and working-set adaptation change placement
only, not logical truth.

Scenario

- move artifacts across tiers
- adapt placement under repeated hot-region access
- compare pre- and post-movement truth surfaces

Must verify

- tier movement does not change replay, restore, or branch truth
- working-set adaptation remains advisory
- no hidden eviction of authoritative truth occurs

Required verification output

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Tiering and working-set logic affect cost only.

### 13.1. Durable Subscription-Support Artifact And Resume-Contract Test

Purpose

Prove that first-class subscription-support artifacts survive restart and
rebuild with explicit resumability meaning instead of degrading into raw cursor
folklore or host-local delivery memory.

Scenario

- persist admitted subscription-support artifacts linked to basis and cursor
  support truth
- restart and rebuild the store
- attempt exact, degraded, rebuild-required, and non-resumable resume lanes

Must verify

- subscription-support identity survives restart and rebuild
- resumability classification is explicit and typed
- cursor truth alone is insufficient to masquerade as exact subscription resume
- basis drift and subscription-support drift are distinguished mechanically

Required verification output

- `truth_digest`
- `artifact_digest`
- `subscription_support_digest`
- `replay_digest`
- `diagnostics_digest`

Pass condition

First-class subscription-support artifacts remain durable, basis-linked, and
honest about resumability.

### 13.2. Subscription-Support Retention, Replication, Compatibility, And Maintenance Test

Purpose

Prove that retention, replication, compatibility drift, and maintenance work
preserve or explicitly degrade subscription-support truth rather than silently
destroying exact resumability.

Scenario

- retain some subscription-support families and reclaim or compact others
- replicate or capsule-export admitted subscription-support scopes
- run compatibility-skew lanes and maintenance-triggered rebuild lanes
- compare exact-resume, degraded-resume, rebuild-required, and rejected lanes

Must verify

- retention and reclaim publish typed subscription resumability conclusions
- replicated subscription-support artifacts preserve declared identity where
  admitted
- version-skew and maintenance drift trigger typed degradation or rejection
- no lane silently reports exact resumability after support loss

Required verification output

- `truth_digest`
- `artifact_digest`
- `subscription_support_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Store programs that act on artifacts preserve or explicitly classify
subscription-support resumability rather than leaving it ambient.

### 13.3. Subscription-Support Accuracy And Certification Test

Purpose

Prove that subscription-support artifact families carry an enforced trust
posture for their declared support role and cannot be consumed as stronger
resume truth than their basis and rebuild posture allow.

Scenario

- build exact, degraded, rebuilt, replicated, and stale subscription-support
  variants
- certify admitted families against their declared support-role posture
- compare generic and domain-facing certification bundles

Must verify

- every admitted subscription-support family is classified explicitly
- stale or rebuilt variants do not present as exact resumability proof unless
  their declared rules allow it
- certification bundles are sufficient to audit subscription-support trust
  posture offline
- generic and domain certification coverage include first-class
  subscription-support lanes

Required verification output

- `artifact_digest`
- `subscription_support_digest`
- `diagnostics_digest`
- `counter_snapshot`
- `certification_summary`

Pass condition

Subscription-support durability is classified and certified mechanically rather
than by convention.

### 14. Replication Capsule Equivalence And Integrity Test

Purpose

Prove that replication and import/export capsules reconstruct the same
canonical truth, artifact identities, and admitted subscription-support
artifacts as the source store.

Scenario

- export capsules
- import into new stores
- replicate partial and full scopes
- run integrity-audit rebuild

Must verify

- imported and replicated scopes match source truth for the declared scope
- omitted scopes remain explicit
- integrity digests are sufficient for offline verification
- admitted subscription-support artifacts preserve their declared resumability
  posture or fail explicitly during transfer

Required verification output

- `truth_digest`
- `artifact_digest`
- `replay_digest`
- `diagnostics_digest`

Pass condition

Replication and capsules preserve canonical meaning and verifiable integrity.

### 15. Extensible Durable Artifact Family Containment Test

Purpose

Prove that extension-defined durable artifact families and storage strategies
cannot become shadow authority, bypass rebuild rules, or evade retention,
compatibility, replication, subscription-support, and certification
boundaries.

Scenario

- register extension-defined derived artifact families with declared authority,
  accuracy, rebuild, retention, export, and compatibility contracts
- rebuild extension families from their declared authority basis
- attempt stale, undeclared, over-privileged, or incompatible extension-family
  publication
- exercise retention and export participation for admitted extension families

Must verify

- extension families remain derived and cannot claim authoritative truth
- extension families that violate declared contracts fail explicitly and typed
- rebuild, retention, and export participation follow the declared family
  contract rather than extension-local heuristics
- extension-defined subscription-support families cannot invent stronger resume
  semantics than the declared platform contract admits
- stale or incompatible extension families trigger typed rejection or explicit
  fallback, never silent acceptance

Required verification output

- `artifact_digest`
- `failure_digest`
- `extension_family_matrix`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Extensibility expands platform breadth without weakening authority,
compatibility, or certification boundaries.

### 16. Time-Travel Diff And Merge-Assistance Parity Test

Purpose

Prove that diff acceleration and merge-assistance artifacts help historical and
merge-heavy workflows without changing canonical diff or merge conclusions.

Scenario

- compare accelerated and non-accelerated historical diff lanes
- rebuild assistance artifacts from authority basis

Must verify

- accelerated lanes match canonical diff/assist conclusions
- missing assistance artifacts fall back explicitly
- rebuilt assistance artifacts match original semantic conclusions

Required verification output

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Assistance artifacts improve cost only.

### 17. Derived Artifact Accuracy Classification Test

Purpose

Prove that every derived artifact family carries the correct enforced accuracy
class and cannot be consumed as stronger truth than its basis allows,
including subscription-support families in their declared support role.

Scenario

- build artifacts across all admitted accuracy classes
- inject stale, partial, and rebuilt variants

Must verify

- every artifact is tagged with the correct class
- stale or partial artifacts do not present as `Exact`
- rebuild and drift detection remain explicit
- subscription-support families do not present exact resumability when their
  declared support basis no longer justifies it

Required verification output

- `artifact_digest`
- `diagnostics_digest`
- `accuracy_class_matrix`
- `counter_snapshot`

Pass condition

Accuracy taxonomy is enforced mechanically, not by convention.

### 18. Analysis Checkpoint Basis Parity Test

Purpose

Prove that analysis checkpoints resume against the exact truth basis they
claim and never become shadow truth.

Scenario

- persist basis-pinned analysis checkpoints
- resume after interruption
- replay and rebuild from basis

Must verify

- resumed analysis matches its declared basis
- basis drift is explicit
- deleting analysis artifacts remains survivable

Required verification output

- `truth_digest`
- `artifact_digest`
- `restore_digest`
- `counter_snapshot`

Pass condition

Analysis lanes are durable, basis-pinned, and non-authoritative.

### 19. Correspondence/Locality Non-Authority Test

Purpose

Prove that correspondence indexes, structural fingerprints, and locality
clustering improve lookup/placement without changing lineage, diff, replay, or
restore truth.

Scenario

- compare indexed and non-indexed lanes
- vary locality placement and correspondence hits

Must verify

- canonical truth surfaces remain equal
- fallback broad scans remain explicit
- locality changes placement only

Required verification output

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Correspondence and locality programs remain derived and non-authoritative.

### 20. Blob Identity Retention And Replication Parity Test

Purpose

Prove that content-addressed blob storage preserves identity, live references,
retention semantics, and replication truth.

Scenario

- store referenced and orphanable blobs
- move blobs across tiers
- reclaim under policy
- replicate and import/export blob-bearing stores

Must verify

- blob identity remains stable by digest
- live references are preserved
- reclaim does not remove live blobs
- blob replication remains parity-safe

Required verification output

- `artifact_digest`
- `restore_digest`
- `diagnostics_digest`
- `counter_snapshot`

Pass condition

Blob storage behaves as a native content-addressed store, not a shadow system.

### 21. Budget Admission Honesty Test

Purpose

Prove that budget overruns and admission-control decisions fail explicitly
before silent degradation occurs.

Scenario

- exhaust branch, history, snapshot, derived-artifact, WAL, and blob budgets
- trigger defer, deny, archive, or explicit degradation policies

Must verify

- policy triggers are explicit and typed
- no hidden eviction of authoritative truth occurs
- admission decisions are deterministic and machine-checkable

Required verification output

- `failure_digest`
- `diagnostics_digest`
- `budget_decision_matrix`
- `counter_snapshot`

Pass condition

Budget pressure produces explicit policy outcomes rather than silent drift.

### 22. Operator Repair, Audit, And Forensic Recovery Test

Purpose

Prove that operator-facing audit, repair, quarantine, and forensic workflows
stay typed, bounded, and authority-safe under corruption, drift, and damaged
media conditions.

Scenario

- run offline audit and integrity-walk lanes over healthy and damaged stores
- generate repair plans for rebuildable derived damage and quarantinable
  unrecoverable damage
- exercise quarantine, salvage, and trusted-truth versus degraded-derived
  reporting lanes
- produce operator-visible forensic bundles for representative corruption and
  repair scenarios

Must verify

- audit and repair surfaces identify the correct trusted versus degraded
  artifact boundary
- repair plans do not mutate authority implicitly
- quarantine and salvage remain explicit typed outcomes rather than ordinary
  success
- operator-visible forensic bundles contain enough machine-checkable detail to
  distinguish trusted truth from rebuildable or unrecoverable damage

Required verification output

- `failure_digest`
- `diagnostics_digest`
- `repair_plan_report`
- `quarantine_report`
- `forensic_bundle_digest`
- `counter_snapshot`

Pass condition

Operator recovery tooling is explicit, auditable, and authority-safe under real
damage pressure.

## What These Suites Collectively Prove

Together, these suites prove that Worth Store is:

- authority-preserving across backends and recovery paths
- crash-safe in durable mode
- honest about embedded mode boundaries
- rebuildable across snapshots, deltas, blocks, and other derived artifacts
- exact for live-query continuation
- disciplined under retention, compaction, and tiering pressure
- exportable and verifiable through canonical capsules and replication
- honest about acceleration, assistance, analysis, correspondence, locality,
  blobs, and budgets
- resilient under long-running churn, corruption, hostile interleavings,
  version-skew, randomized histories, and operator-facing diagnosis

## Beta Readiness Rule

Worth Store is beta-ready only when:

- all milestone-mapped named suites pass
- all cross-cutting beta suites pass
- all required outputs are machine-checkable
- no suite still relies on human log interpretation for pass/fail
- the admitted operational envelope is explicit and tested rather than implied

## Milestone Closeout Rule

No Worth Store milestone should be considered closed until:

- its required named suite passes
- the suite emits machine-checkable output
- replay, rebuild, restore, or restart lanes are compared where applicable
- typed failure behavior is proven where applicable

Without that, the capability may be promising, but it is not yet trust-grade.
