# Milestone S.7.1: Proof-Flow And Domain-Structure Cleanup Gate

## Goal

Make the Store code produced through S.7 structurally auditable before S.8
builds index, layout, and access-path discipline on top of it.

This milestone converts proof-heavy Store surfaces from broad bags of
authority-sounding artifacts into named transition systems: source authority
enters, evidence is collected, a case is classified, the transition is
verified, a receipt is constructed, and only the next valid capability is
exposed.

## Why This Milestone Exists

S.7 made blobs native to the physical Store substrate, but the implementation
pressure exposed a deeper architectural risk: proof vocabulary can multiply
faster than proof grammar. If S.8 builds layout and access-path contracts on
top of flat directories, broad facades, copied proof fields, and certification
surfaces acting like production law, the physical database foundation will
inherit structural ambiguity precisely where it needs machine-checkable
authority.

S.7.1 is therefore a cleanup gate, not a feature milestone. It freezes new
concept expansion and cleans the proof-flow topology before more physical
database layers depend on it.

## Governing Summaries

- `MENTALITY.md` protects hard-problem-first design. This spec treats proof
  theater and hidden topology as the adversarial problem rather than waiting
  for S.8 through S.12 to compound around it.
- `arch_laws.md` protects compiler-enforced proof progression. The spec is
  shaped most strongly by Laws 30, 37, 41, and 42: each transition must consume
  the prior proof type, produce the next proof type, and prevent identity or
  authority reconstruction from representation.
- `composition_laws.md` protects semantic step visibility inside files and
  functions. The spec requires major flows to read as named semantic steps, not
  inline predicates plus receipt assembly.
- `domain_structure_laws.md` protects the filesystem as responsibility
  architecture. The spec requires phase-shaped directories, narrow facades,
  test topology that falsifies production topology, and no broad buckets.
- `perf_laws.md` protects visible cost and bounded execution. The spec keeps
  counters tied to transition outcomes and prevents cleanup from hiding scans,
  allocations, or broad maintenance behind nicer names.
- `physical-database-roadmap.md` protects the physical database foundation order.
  S.7.1 belongs after S.7 because native blob proof flows exposed the topology
  problem, and before S.8 because layout/index discipline must not depend on
  unauditable proof surfaces.

## Adversarial Constraint

No Store flow may claim proof-bearing authority if a reviewer must reconstruct
the proof transition from raw predicates, copied fields, broad exports, flat
directories, `mod.rs` business logic, certification-only contracts, or
test-only helpers that bypass production topology.

## Product Decision Lock

S.7.1 must not add new blob product capability. Any new type, module, helper,
or test support surface must exist to make an existing proof flow auditable,
sealed, phase-shaped, or structurally enforceable.

## Cleanup Evidence Standard

This milestone closes on structural evidence, not feature count. Each phase
must prove that the cleaned code is easier to audit, harder to misuse, and more
faithful to the intended authority graph than the code it replaces.

Valid evidence includes code review findings, directory skeletons, public API
diffs, removed exports, smaller module boundaries, named decision tables,
named transition functions, compile-fail coverage, runtime tests, structural
scan checks, and focused verification commands. Tests are required where the
cleanup changes a construction boundary, public capability, runtime behavior,
or previously missing guardrail. Pure topology cleanup may close on structural
diffs and review evidence when no executable behavior changed.

## Phase Plan

### Phase 1: Structural Inventory And Concept Freeze

Freeze new Store concept expansion and produce the cleanup map that the rest of
the milestone must consume.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-certification`
- `worth-store-physical-format`
- `worth-store-physical-isolation`
- `worth-store-recovery-physics`
- `worth-store-io-scheduler`
- `worth-store-physical-backend`
- `worth-store-buffer-pool`
- `worth-store-physical-integrity`

**Relevant APIs**
- crate `lib.rs` and `exports.rs` surfaces
- public constructors for proof, authority, receipt, readiness, handoff, and
  test support objects

**Warnings**
- Do not count nouns as architecture. A type named `Proof`, `Authority`,
  `Receipt`, `Readiness`, or `Handoff` is suspect until its transition source,
  verifier, denial cases, and next capability are visible.
- Do not expand S.7 to finish unbuilt blob work during this phase. This phase
  maps and freezes; it does not invent new blob scope.

**Evidence requirements**
- Produce an inventory of critical crate roots, broad facades, wildcard
  exports, `mod.rs` business logic, helper swamps, copied-field constructors,
  and proof-flow god functions.
- Record which findings require tests, which require only structural edits, and
  which are explicitly out of scope because they are not blocking S.7.1.

**Engineering decisions**
- The cleanup map must classify findings by authority boundary, lifecycle
  phase, public surface, test topology, and cost/counter visibility.
- Findings must identify whether the problem is semantic collapse, directory
  topology, facade leakage, test bypass, certification overreach, or counter
  detachment.

**Open questions**
- None.

### Phase 2: Certification Courtroom Cleanup

Separate certification as evidence courtroom from production law, vocabulary
warehouse, or proof constructor authority.

**Relevant subsystems**
- `worth-store-certification`
- `worth-store-physical-certification`
- `worth-store-test-support`
- Store compile-fail harnesses

**Relevant APIs**
- certification scenario definitions
- certification evidence bundles
- test and certification authority helpers
- any public type used by production crates to satisfy lower authority
  contracts

**Warnings**
- Certification may report, replay, adversarially construct, and cross-examine
  evidence. It must not define the real production contract when that contract
  belongs in a lower Store crate.
- If production code depends on certification-only vocabulary to understand
  what is legal, the dependency direction is inverted.

**Evidence requirements**
- Show that production crates no longer depend on certification-only
  constructors or copied certification rows to mint production readiness.
- Where a construction boundary changes, add compile-fail coverage proving
  certification-only evidence cannot satisfy lower Store production law.
- Preserve certification replay parity where behavior is touched, proving it
  consumes lower-crate production proof objects rather than a structurally
  similar but lower-authority evidence bundle.

**Engineering decisions**
- Move shared production contract vocabulary out of certification when the
  contract is lower Store law.
- Keep adversarial scenario builders and courtroom-specific rows in
  certification, with names that make their non-production authority obvious.

**Open questions**
- None.

### Phase 3: Physical Format Topology Cleanup

Clean the physical-format crate before blob, index, and layout work depend on
its root as precedent.

**Relevant subsystems**
- `worth-store-physical-format`
- physical page/frame/manifest/blob manifest modules
- physical-format tests and compile-fail tests

**Relevant APIs**
- physical format `lib.rs`
- physical format facades
- page, frame, manifest, checksum, and blob-manifest public exports

**Warnings**
- A flat physical-format root teaches future S.8 layout work to add more root
  files instead of exposing artifact-family and lifecycle boundaries.
- `lib.rs` may aggregate and document the public facade, but it must not own
  business logic, format rules, or proof predicates.

**Evidence requirements**
- Document the final physical-format skeleton and show that `lib.rs`, facade
  exports, and root file count match the declared topology.
- Where raw construction lanes are removed or sealed, add compile-fail coverage
  proving external callers cannot bypass format admission with field copies.

**Engineering decisions**
- Physical-format structure should classify by stable artifact responsibility:
  page/frame, segment/manifest, record framing, checksums/integrity slots,
  format versioning, and blob manifest support.
- Public exports should teach format lifecycle order rather than mirror every
  internal module.

**Open questions**
- None.

### Phase 4: Blob Crate Lifecycle Tree

Reshape `worth-store-blob-chunks` so the directory tree teaches the blob
lifecycle before readers open individual files.

**Relevant subsystems**
- `worth-store-blob-chunks`
- blob chunk identity and canonical basis
- blob lifecycle authority and progression
- blob streaming, publication, recovery, reachability, reclaim, placement,
  compaction, and corruption modules

**Relevant APIs**
- `worth-store-blob-chunks/src/lib.rs`
- `worth-store-blob-chunks/src/exports.rs`
- all public blob proof, receipt, denial, counter, and authority types

**Warnings**
- A root containing dozens of same-level blob files is an ontology dump, not a
  lifecycle. It makes future edits land by grep and proximity instead of by
  proof responsibility.
- Folder names must encode lifecycle phase, authority regime, or operation
  family. Names like `helpers`, `common`, `logic`, or `tests` are not enough
  unless scoped under a real responsibility.

**Evidence requirements**
- Document the final blob crate skeleton, including any narrow exemptions, and
  show that root file count and module depth match the declared lifecycle tree.
- Where public/internal visibility changes, prove downstream crates use public
  blob facades and cannot deep-import lifecycle internals that should remain
  replaceable.

**Engineering decisions**
- The top-level blob structure should make these axes discoverable:
  identity/canonical basis, lifecycle admission, streaming, publication,
  recovery, reachability/retention/reclaim, placement/compaction, corruption,
  counters, and test support.
- Test support must live under the narrowest blob responsibility it serves, or
  in a clearly bounded harness crate when multiple crates share the same test
  authority for the same semantic reason.

**Open questions**
- None.

### Phase 5: Blob Public Facade And Export Grammar

Make blob public surfaces reveal legal lifecycle order instead of exposing every
proof noun at once.

**Relevant subsystems**
- `worth-store-blob-chunks`
- Store facade/export modules
- compile-fail UI tests

**Relevant APIs**
- blob chunk public facade
- exported lifecycle handles
- exported readiness and receipt types
- test authority exports

**Warnings**
- A public export that mirrors internals freezes accidental topology as API.
- Public proof-bearing types must expose read accessors and next capabilities,
  not raw construction lanes or field-copy compatibility.

**Evidence requirements**
- Show the before/after public facade diff and explain which lifecycle
  capabilities remain public.
- Where proof objects are sealed, add compile-fail coverage proving external
  crates cannot construct blob admission, publication, streaming,
  reachability, reclaim, placement, compaction, or corruption proofs from raw
  fields.
- Verify ordinary callers can follow the lifecycle through facade-returned
  capabilities without reaching into internal modules.

**Engineering decisions**
- Export by lifecycle capability and authority class, not alphabetically by
  every noun in the crate.
- Keep compatibility re-exports only when they preserve lifecycle order and do
  not expose construction power.

**Open questions**
- None.

### Phase 6: Chunk Identity And Dedupe Proof Grammar

Turn chunk identity, canonical basis, collision handling, and dedupe admission
into an explicit transition family.

**Relevant subsystems**
- blob chunk identity
- blob chunk canonical basis
- dedupe admission and collision verification
- dedupe counters and receipts

**Relevant APIs**
- chunk identity constructors
- dedupe evidence collection
- dedupe case classification
- dedupe verification
- dedupe receipt construction

**Warnings**
- Digest equality is evidence, not authority. It must not merge chunks across
  tenant scope, key scope, authenticity class, custody posture, or collision
  posture without admitted proof.
- Dedupe counters must attach to the classified transition outcome, not appear
  as decoration after a branch.

**Evidence requirements**
- Show the named dedupe transition sequence and the type or module that owns
  each step.
- Preserve or add only the focused tests needed to prove identical bytes inside
  the same admitted security scope converge to the same dedupe outcome and
  that copied digests, copied receipt rows, wrong tenant/key scope, stale key
  version, lower-authenticity posture, or collision-indeterminate bytes cannot
  mint dedupe admission.

**Engineering decisions**
- Use the grammar:
  collect chunk dedupe evidence -> classify dedupe case -> verify collision
  and scope transition -> construct dedupe receipt -> expose chunk capability.
- Keep raw byte comparison mechanics below the verification step and out of the
  public admission surface.

**Open questions**
- None.

### Phase 7: Publication, WAL, And Recovery Proof Grammar

Make blob publication and crash recovery read as a state machine rather than
scattered readiness, record, and replay objects.

**Relevant subsystems**
- blob publication commit
- blob recovery records
- `worth-store-wal`
- `worth-store-recovery-physics`
- partial publication recovery lanes

**Relevant APIs**
- publication commit request/admission
- recovery record generation
- replay-read admission
- WAL blob record surfaces

**Warnings**
- Publication is not a boolean. The code must distinguish prepared bytes,
  WAL-recorded bytes, committed root visibility, replayable recovery evidence,
  and denied/partial publication cases.
- Recovery records must not become an alternate authority path around
  publication verification.

**Evidence requirements**
- Show the publication/recovery state graph and the proof type consumed and
  produced at each transition.
- Preserve or add only the focused replay and denial tests needed to prove
  interrupted publication converges through crash replay and that missing WAL
  evidence, copied recovery ids, stale publication generations, and pre-replay
  reads are denied before visible blob authority is exposed.

**Engineering decisions**
- Publication transitions should consume prior proof types and produce the next
  publication capability with private fields.
- WAL and recovery adapters may translate records, but lower publication law
  remains in the blob/recovery Store crates rather than certification.

**Open questions**
- None.

### Phase 8: Streaming Ingest, Read, And Resume Proof Grammar

Separate streaming admission, chunk movement, frontier progress, verification,
and receipt construction.

**Relevant subsystems**
- blob streaming ingest
- blob streaming read
- blob streaming resume
- streaming memory/residency counters
- I/O scheduler pressure surfaces

**Relevant APIs**
- streaming request/admission
- read observation and verification
- resume session capability
- streaming performance/counter receipts

**Warnings**
- Streaming code is especially likely to mix setup, chunk IO, memory budgeting,
  proof checks, counters, and result construction inside one loop.
- Constant-memory claims are not comments. They must be carried by exact
  resident-byte, chunk-window, allocation, and foreground-pressure counters.

**Evidence requirements**
- Show the streaming loop shape and where admission, frontier advancement,
  chunk-window verification, memory counters, and receipt construction live.
- Preserve or add only the focused equivalence, budget, and scope tests needed
  to prove full-read, segmented-read, interrupted-read, resumed-ingest, and
  resumed-read converge without whole-object materialization or stale/copied
  authority.

**Engineering decisions**
- The public loop shape should expose:
  admit stream -> advance frontier -> verify chunk window -> emit receipt or
  resume capability.
- Read-side observation must not carry write authority or repair authority.

**Open questions**
- None.

### Phase 9: Reachability, Retention, And Reclaim Proof Grammar

Make blob reachability and reclaim a visible reference-state transition, not a
registry with scattered checks.

**Relevant subsystems**
- blob reachability edges
- blob reachability registry
- retention holds
- reclaim release
- dedupe reference edges
- physical-isolation orphan reclaim

**Relevant APIs**
- reachability edge admission
- reference accounting
- retention hold capability
- reclaim eligibility and release receipts

**Warnings**
- Reclaim must not treat absence of a visible reference as proof of
  reclaimability unless the relevant snapshot, publication, dedupe, retention,
  and recovery horizons are bound together.
- Retention holds, reachability edges, and dedupe references have different
  authority and must not collapse into one generic reference row.

**Evidence requirements**
- Show the reachability/reclaim decision table and the authority behind each
  outcome.
- Preserve or add only the focused convergence and denial evidence needed to
  prove reclaim outcomes stay stable across checkpoint, replay, dedupe release,
  retention hold release, and snapshot movement, and that orphan-looking chunks
  remain unreclaimable when required proof is missing, stale, copied, or
  cross-scope.

**Engineering decisions**
- Classify reachability cases before reclaim execution.
- Attach orphan, reachable, held, reclaimable, and denied counters to the
  classified outcome rather than scattered branch sites.

**Open questions**
- None.

### Phase 10: Placement, Movement, And Compaction Proof Grammar

Clean blob placement, cold movement, and compaction handoffs into movement
state transitions with explicit stability evidence.

**Relevant subsystems**
- blob placement admission
- blob placement movement
- blob compaction
- `worth-store-tiering`
- `worth-store-physical-isolation`
- S.6 background pacing and S.7 cold placement surfaces

**Relevant APIs**
- placement posture/admission
- movable stability proofs
- compaction movement plans
- cold-tier posture handoffs

**Warnings**
- Compaction and tier movement must not look like metadata rewrites. They move
  physical bytes under read stability, security scope, and I/O pacing
  constraints.
- Movement proofs must not rely on backend-private residue or scheduler logs.

**Evidence requirements**
- Show the placement/movement state graph and where stable-read, security
  scope, and I/O admission evidence enter.
- Preserve or add only the focused movement evidence needed to prove placement
  movement preserves blob identity, digest basis, security scope, and
  reachability, and that unstable leases, unsupported posture, wrong scope,
  copied stability evidence, or background pressure denial block movement.

**Engineering decisions**
- Movement phases should consume stable-read or movable-stability proof before
  byte movement, then produce a placement receipt that exposes only the next
  valid capability.
- I/O scheduler readiness is admission evidence for movement cost, not proof
  that the moved bytes are semantically valid.

**Open questions**
- None.

### Phase 11: Corruption, Quarantine, And Readmission Proof Grammar

Make blob corruption handling a localized physical damage state machine with
explicit readmission, not a set of error variants around normal reads.

**Relevant subsystems**
- blob corruption
- offline verifier blob observation
- physical integrity
- recovery physics
- backup/export/import readmission surfaces where already present

**Relevant APIs**
- corruption observation
- quarantine evidence
- readmission requests
- verifier and recovery handoffs

**Warnings**
- A checksum failure, authenticity failure, missing chunk, stale generation,
  and cross-scope import are distinct cases. Collapsing them into generic
  corruption hides recovery and repair posture.
- Readmission after crossing a trust boundary must rebuild current Store
  authority; it must not trust deserialized declarations or terminal
  projections as proof.

**Evidence requirements**
- Show the corruption/quarantine/readmission decision table and where logical
  decode is blocked.
- Preserve or add only the focused localization and readmission evidence needed
  to prove corrupted chunks quarantine only the affected physical region and
  that JSON/serde declarations, copied verifier rows, stale key scope, wrong
  tenant scope, missing authenticity, or post-rotation backup restore cannot
  become blob authority without explicit readmission.

**Engineering decisions**
- Corruption classification must happen before logical decode or publication
  visibility.
- Quarantine receipts should expose diagnostics and repair/readmission
  capability, not ordinary blob read authority.

**Open questions**
- None.

### Phase 12: Recovery, Isolation, Scheduler, And Backend Seam Cleanup

Clean the supporting seams S.7 depends on when blob flows cross recovery,
stable read, I/O scheduling, and backend observation boundaries.

**Relevant subsystems**
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-io-scheduler`
- `worth-store-physical-backend`
- `worth-store-offline-verifier`

**Relevant APIs**
- stable read execution and receipt surfaces
- checkpoint and compaction interlocks
- background pacing and foreground reservation
- backend blob observation
- partial publication and replay-read admission

**Warnings**
- These crates may provide lower Store law or mechanical admission. They must
  not become dumping grounds for S.7-specific cleanup helpers.
- Scheduler admission is about I/O cost and interference, not semantic proof of
  byte validity or security scope.

**Evidence requirements**
- Show the dependency-direction diff for each touched lower seam and identify
  which crate owns the production law after cleanup.
- Where lower seam construction is sealed, add compile-fail coverage proving
  blob code cannot bypass readiness by copying counters, receipts, proof ids,
  backend observation rows, or test authority witnesses.
- Preserve or add only the focused runtime boundary evidence needed to prove a
  scenario is denied at the correct lower seam when stable-read, recovery,
  scheduler, backend, or verifier evidence is missing.

**Engineering decisions**
- Keep dependency direction lower-to-higher honest: lower Store crates define
  their own production law; blob and certification adapt to it.
- Replace cross-crate field-copy constructors with sealed handoff types or
  phase-typed capability values where the proof has to cross crate boundaries.

**Open questions**
- None.

### Phase 13: Buffer Pool And Physical Integrity Targeted Cleanup

Clean only the buffer-pool and physical-integrity surfaces that directly affect
proof-flow readability, memory bounds, and corruption-first behavior.

**Relevant subsystems**
- `worth-store-buffer-pool`
- `worth-store-physical-integrity`
- physical record access and page integrity tests

**Relevant APIs**
- page lease and pin/unpin surfaces
- resident-memory and allocation counters
- checksum/integrity verification admissions
- corruption localization receipts

**Warnings**
- This is not a full S.2 or S.3 rewrite. The cleanup scope is limited to
  structural defects that make S.7/S.8 proof flows unauditable or make cost
  claims hard to prove.
- Integrity must precede logical decode; cleanup must not move corruption
  handling later for convenience.

**Evidence requirements**
- Show that cleanup did not introduce hidden allocation, page clone, or
  whole-object residency paths for blob-adjacent reads.
- Preserve or add only the focused memory-bound and corruption-first evidence
  needed to prove damaged page/frame/chunk bytes are denied before blob
  lifecycle, dedupe, publication, or read authority can observe decoded
  meaning.

**Engineering decisions**
- Keep buffer-pool counters and physical-integrity counters attached to the
  transition boundary that incurred the work or denied the operation.
- Do not introduce generic helper modules shared between buffer-pool and
  integrity unless the shared authority and lifecycle are identical.

**Open questions**
- None.

### Phase 14: Test And Certification Topology Cleanup

Make tests, fixtures, compile-fail suites, and certification harness support
falsify the production topology instead of bypassing it.

**Relevant subsystems**
- Store integration tests
- Store compile-fail UI tests
- `worth-store-test-support`
- `worth-store-physical-certification`
- blob harness vocabulary

**Relevant APIs**
- test support builders
- certification scenario vocabulary
- compile-fail UI fixtures
- structural scan tests

**Warnings**
- A test helper that mints production proof objects through a path production
  callers cannot use is not helpful. It is a false authority lane.
- Certification fixtures may be adversarial and synthetic only when the
  synthetic authority is named as courtroom evidence, not production proof.

**Evidence requirements**
- Show the final test-support and certification harness topology, including
  which helpers are production-facade users and which are named test authority.
- Where helper construction changes, add compile-fail coverage proving test
  support cannot construct private production proof fields, readiness requests,
  or next-capability handles except through explicitly named test-authority
  modules.
- Preserve or add only the focused harness-honesty evidence needed to prove
  real integration lanes use production facades and synthetic certification
  lanes label their authority, evidence source, and denial expectation.

**Engineering decisions**
- Test support topology should mirror production responsibility boundaries
  only where that helps locate failures; it must not become a parallel
  production API.
- Compile-fail tests are guardrails for construction and visibility; runtime
  tests still carry semantic, replay, convergence, cost, and corruption proof.

**Open questions**
- None.

### Phase 15: Structural Closeout And S.8 Readiness

Prove the cleanup has produced a stable enough foundation for S.8 layout and
access-path discipline.

**Relevant subsystems**
- all crates touched by S.7.1
- Store workspace verification scripts
- S.8 handoff surfaces

**Relevant APIs**
- S.7 public blob facades
- lower Store readiness and handoff types consumed by S.8
- structural scan outputs
- complexity/counter receipts for cleaned hot paths

**Warnings**
- Passing unit tests is insufficient. The closeout must show the public surface
  now teaches lifecycle order and the directory tree now rejects the next
  convenient bad edit.
- Do not close with broad debt language. Any remaining structural exception
  needs an explicit reason, owner, scope, and follow-on milestone.

**Evidence requirements**
- Produce a structural closeout bundle showing the cleaned Store crates satisfy
  file-count policy, no-business-logic aggregation policy, facade visibility
  policy, construction-boundary policy, and relevant S.7 runtime hostile lanes.
- Prove S.8 can consume blob/layout inputs only through cleaned S.7/S.7.1
  capabilities, not raw blob internals, copied counters, certification rows, or
  helper constructors.

**Engineering decisions**
- The closeout artifact should name the final directory skeleton for each
  cleaned critical crate and the public facade each external caller may use.
- The S.8 handoff must be a typed readiness/capability surface, not a markdown
  claim or certification summary.

**Open questions**
- None.

## Must Ship

- Roadmap-recognized S.7.1 cleanup milestone and spec.
- Structural inventory and freeze for critical S.7 and supporting Store
  subsystems.
- Cleaned certification role: courtroom and harness, not production law.
- Cleaned `worth-store-blob-chunks` lifecycle tree and public facade grammar.
- Named proof-flow transitions for dedupe, publication/recovery, streaming,
  reachability/reclaim, placement/compaction, and corruption/readmission.
- Targeted cleanup of physical-format, recovery/isolation/scheduler/backend
  seams, buffer-pool, physical-integrity, and test/certification topology where
  they block proof-flow auditability.
- Compile-fail, runtime, structural scan, and harness-honesty tests proving the
  cleanup cannot be bypassed by raw fields, copied counters, broad exports, or
  test-only authority.

## Must Preserve

- S.7 blob behavior, hostile proof lanes, security-scope posture, recovery
  behavior, and constant-memory claims.
- Store-owned physical byte survival authority.
- Foundational/Proof vocabulary adoption where those crates provide shared
  language, without moving Store physical law out of Store.
- Certification as evidence courtroom rather than law.
- No new blob feature expansion under the cover of cleanup.

## Acceptance Evidence

- `cargo check` for every touched Store crate.
- Focused Store tests for every cleaned proof family.
- Relevant compile-fail UI suites for construction, facade, and helper
  boundary violations.
- Structural scan proving declared directory/file-count/facade policies.
- S.8 readiness test proving future layout/index work consumes cleaned
  capabilities, not raw blob internals.

## Sequencing Notes

S.7.1 belongs immediately after S.7 and before S.8. S.7 exposed the structural
risk because native blobs forced many authority, recovery, dedupe, streaming,
reclaim, compaction, and corruption concepts into the workspace. S.8 will add
layout and access-path discipline across artifact families; it should inherit a
phase-shaped proof grammar, not a flat set of authoritative nouns.

This milestone may touch earlier Store crates only when their structure is
blocking S.7 proof-flow auditability or S.8 readiness. It must not reopen S.0
through S.6 semantics for feature expansion.
