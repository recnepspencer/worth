# Storage Foundation S.1 Engineering Spec: Physical Page, Segment, And Extent Substrate

> **Status:** Planned
>
> **Roadmap parent:** [physical-database-roadmap.md](./physical-database-roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:**
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-2.md](./test-requirements-2.md)
>
> **Prerequisite storage-foundation sequence:** `S.0`
>
> **Follow-on storage-foundation sequence:** `S.2`
>
> **Primary architectural driver:** define the physical byte universe of
> `worth-store` before any platform-grade backend can claim to be a database.

## Goal

Make page, segment, extent, frame, manifest, and physical-reference addressing
the mandatory substrate for the platform-grade Worth Store backend.

S.1 is split into two closeout gates:

- `S.1A PhysicalSubstrateExists`
  proves the physical byte substrate exists: binary format law, physical
  references, page/frame headers, page-local slots, extent-backed records,
  segment/extent/root manifests, free-space generation posture, platform
  facade, and basic reopen/locate/scan.
- `S.1B PhysicalSubstrateCertification`
  proves the substrate honestly earns the database claim: story-readable
  certification harness, offline verifier parity, exact counters, locality and
  complexity contracts, shortcut rejection lanes, foundational evidence export,
  and S.2 handoff readiness.

S.1 is complete when both gates close. The platform-grade backend must persist,
reopen, scan, and locate representative authoritative and derived records
through physical ids without deserializing the whole store into heap domain
objects; all legacy heap/file/SQLite paths must be fenced into explicit
non-platform-grade capability tiers; and the certification evidence must be
readable as executable physical-law stories rather than opaque action chains.

## Why This Sequence Exists

The first Worth Store roadmap proved a large semantic durability program:
canonical authority, WAL-shaped recovery, snapshots, branch deltas, retention,
tiering, subscription support, and certification surfaces. That work remains
valuable, but it does not by itself make the store a physical database.

A real database needs a byte substrate that can be read, written, reopened,
verified, repaired, and evolved without relying on whole-store heap
materialization or backend-private residue. S.1 is the first foundation sequence
because every later Roadmap 2 capability depends on stable physical artifacts:

- S.2 needs pages and frames to bound resident memory.
- S.3 needs page/frame/chunk boundaries to reject damaged bytes before decode.
- S.4 needs physical references and manifests for WAL/checkpoint recovery.
- S.5 needs generation-bearing roots and references for stable read plans.
- S.8 needs artifact families to attach honest layout strategies.
- S.10 needs an offline verifier that can inspect bytes without live runtime
  construction.

If S.1 is weak, the rest of Roadmap 2 becomes decoration around a heap-shaped
store. This sequence exists to remove that ambiguity first.

## Minimum Closeout Spine

The closeout spine is the primary architecture of S.1. Every page type,
manifest, counter, story transcript, verifier row, foundational attachment,
receipt, and suite exists only to protect one edge in this chain:

`AcceptedHandoffReadiness`
-> `PhysicalScenarioQualityHarness`
-> `PhysicalFormatVocabulary`
-> `PhysicalReferenceAuthority`
-> `PhysicalHeaderDecodeWitness`
-> `FramedRecordPlacement`
-> `SegmentManifestAuthority`
-> `ExtentManifestAuthority`
-> `PhysicalRootManifest`
-> `PlatformPhysicalFacade`
-> `OfflinePhysicalVerifier`
-> `FoundationalPhysicalEvidenceBundle`
-> `S2PhysicalSubstrateReadiness`

No phase may introduce a public product that does not either become one of
these spine products or directly protect one spine transition. Internal helper
types are allowed, but they must not obscure which spine product carries
S.1-grade physical authority.

The spine is intentionally physical. Semantic commit envelopes may be stored as
representative authoritative payloads, but semantic truth does not enter the
S.1 proof chain as physical authority. The physical proof chain answers whether
bytes can be placed, located, reopened, scanned, verified, and described
without heap-shaped shortcuts.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial, hard-problem-first foundation work. S.1 therefore
  starts with full-store materialization and backend-residue dependency as the
  enemy, rather than wrapping current persistence with nicer structs.
- `arch_laws.md`
  protects proof-bearing boundaries and authority separation. S.1 must create
  typed physical proof before logical decode and keep physical placement from
  becoming semantic truth.
- `composition_laws.md`
  protects named orchestration and semantic-step decomposition. S.1 must not
  collapse header admission, reference validation, manifest traversal, record
  placement, verifier materialization, and evidence construction into one
  physical-format god path.
- `perf_laws.md`
  protects visible cost and honest access boundaries. S.1 must expose exact
  page-read, page-write, frame-decode, allocation, scan, and manifest-lookup
  counters instead of calling physical access "fast" by assertion.
- `domain_structure_laws.md`
  protects decomposition by reason-to-change. Pages, frames, segments, extents,
  allocation, manifests, references, legacy classification, and verification
  must be separate responsibilities because they fail and evolve differently.
- `worth_foundational_roadmap.md`
  protects shared Worth meaning without imposing shared hot-path
  representation. S.1 must use `worth-foundational` when physical evidence is
  exported, compared, certified, supported, or handed off: canonical basis,
  digest derivation, identity/locator categories, diagnostic ontology,
  profile/materialization posture, boundary evidence, receipt/provenance
  language, and performance-report vocabulary. Store-owned page, frame,
  segment, extent, manifest, reference, and byte-survival authority stays
  inside `worth-store`.
- Physical scenario harness design
  protects proof ergonomics across Roadmap 2. S.1 must start a shared
  scenario harness where readable physical-law stories lower into inspectable
  plans, drivers, observers, oracles, and transcripts so later WAL, blob,
  repair, security, and certification lanes extend the same proof architecture.
- `worth_store_vision.md`
  protects the thesis that Store makes truth survive without owning truth
  semantics. S.1 gives survival a physical byte substrate while preserving
  canonical commit envelopes as semantic authority.
- `runtime-integration-roadmap.md`
  requires Part II to consume the closed physical substrate without inheriting
  the retired semantic Store topology. S.1 belongs after S.0 reclassification
  and before S.2 memory residency because physical ids must exist before pages
  can be leased, pinned, or evicted.
- `physical-database-roadmap.md`
  protects the database-foundation backtrack. Its S.1 sketch requires fixed
  pages, variable extents, physical headers, slot directories, manifests,
  allocation classes, free-space maps, root manifests, and legacy-backend
  fencing.
- `test-requirements.md`
  protects machine-checkable closeout evidence. S.1 is not closeable until the
  `Physical page/segment/extent substrate test` proves physical access through
  persisted bytes and forbids heap-shaped shortcuts.
- `test-requirements-2.md`
  protects the adversarial harness realism. S.1 must use the storage-boundary
  interposer, adversarial backend, offline verifier, evidence bundle system,
  and coverage matrix rather than direct private-state mutation.

## Adversarial Constraint

S.1 must survive this hostile condition:

> A platform-grade backend stores representative authoritative and derived
> artifacts across multiple segments and extents, closes, reopens from persisted
> bytes, scans manifests, locates records by stable physical references, rejects
> stale generation references, emits physical layout evidence, and proves that
> no required lane used full-store heap materialization, full-domain-object
> deserialization, backend-private residue guessing, or legacy bootstrap paths
> to satisfy the database claim.

If a record can be found only by loading the whole store into domain structs,
if a physical reference can silently point to reused bytes, if the root manifest
is not self-describing enough for an offline verifier to walk the store, or if a
legacy backend can still present itself as platform-grade, S.1 is not closed.

## Product Decision Lock

- physical pages and extents are byte containers and access structures, not
  semantic truth
- canonical commit envelopes remain the semantic durability authority
- physical ids are stable within their generation and invalid across detected
  generation reuse
- page/frame headers carry physical decode evidence before any logical artifact
  decoder runs
- the root manifest is the entry point for physical discovery; backend-local
  directory layout is not authority
- slot directories or equivalent page-local record tables are mandatory for
  record addressing
- variable extents are admitted only through typed allocation classes and
  manifest membership
- legacy heap/file/SQLite paths may remain useful for bootstrap,
  compatibility, or semantic-certification lanes, but they may not satisfy S.1
  platform-grade physical-substrate claims
- S.1 reserves integrity fields such as checksum slots and generation fields,
  but full checksum enforcement belongs to S.3
- S.1 defines physical references and persisted record framing; buffer-pool
  pinning, eviction, resident-memory budgets, and zero-copy hot-path leases
  belong to S.2
- Store physical authority must be locally complete without foundational
  evidence: physical bytes, references, generations, headers, slots,
  extents, manifests, root discovery, verifier walks, and stale-reference
  denial must work through Store-owned witnesses before any exported evidence
  is lowered into shared vocabulary
- `worth-foundational` is required only at exported, compared, certified,
  support, or handoff evidence boundaries: S.1 layout reports, diagnostic
  rows, performance receipts, provenance, profiles, locators, and boundary
  receipts must lower into the current foundational facade when they leave the
  Store-local proof boundary or record explicit adoption debt when the exact
  surface is absent
- `worth-foundational` must not own Store physical authority: it may name
  shared boundary categories, canonical basis entries, profile families,
  diagnostic rows, performance claim posture, and receipt/provenance
  attachments, but it may not define page bytes, frame headers, slot
  directories, allocation rules, manifest authority, stale-generation law, or
  durable physical reference semantics
- S.1 physical bytes are Store-owned framed binary records, not JSON or serde
  domain objects. Any exported S.1 evidence, canonical basis, digest
  preparation, diagnostic row, receipt, performance receipt, or S.2 handoff
  must lower through aspect-native Store/Foundational value, identity, locator,
  and proof surfaces rather than JSON-shaped payloads, debug strings, display
  names, raw maps, or producer-private names.

## Production-Grade Implementation Bar

S.1 closes only on robust mechanisms suitable as the first byte-level substrate
for later financial, aerospace, and platform-grade Store claims. It does not
close on a restart demo.

Required posture:

- No phase may close with a toy, placeholder, best-effort, happy-path,
  scalar-only, single-page-only, single-segment-only, serde-blob-first,
  in-memory-map-first, backend-residue-first, stringly typed, review-only, or
  convention-enforced implementation.
- Every admitted physical structure must survive hostile references, stale
  generations, malformed headers, unknown kinds, length mismatches, deleted or
  moved slots, fragmented free space, root-manifest reorderings, verifier
  disagreement, legacy backend overclaims, and replay after close/reopen.
- Every authority boundary must be enforced by types, sealed constructors,
  visibility, compile-fail tests, verifier lanes, certification fixtures, or
  verified runtime denial. Documentation alone is not enforcement.
- Every critical physical operation must expose exact counters and a verified
  complexity contract before closeout.
- Every denial must be typed, localized, diagnosable, and stable across replay.
- Every simplified path must be rejected before construction or explicitly
  outside S.1 closeout. It cannot ship as the ordinary platform-grade lane.
- Every exported S.1 evidence surface must participate in foundational
  canonical basis, diagnostics, profiles, boundary evidence, provenance,
  support truth, or performance receipts where the shared vocabulary exists.
- Every format byte that matters to reopen, locate, verify, or later recovery
  must be specified by Store, not left to serializer behavior.
- "Good enough for now" is not an S.1 implementation posture.

If a phase requirement appears satisfiable by a small in-memory map, a serde
blob, a single-file append log, or a filesystem directory convention, that path
is noncompliant unless it also satisfies the hostile tests, counters,
compile-time fences, binary format rules, replay proofs, verifier independence,
and denial behavior named by this spec.

## S.1-Grade Authority Products

S.1 has a small public authority surface. Supporting types may be numerous, but
S.1-grade authority is compressed into these products:

- `PhysicalFormatVocabulary`
  - owns physical page, frame, segment, extent, slot, generation, publication,
    manifest, allocation, free-space, and binary-format terminology
- `PhysicalScenarioQualityHarness`
  - owns the Roadmap 2 executable proof architecture that S.1 starts and later
    sequences extend through lane families. It owns scenario definitions,
    lowered scenario plans, execution, drivers, observers, proof oracles,
    evidence bundles, and stable transcripts. S.1 consumes it first for
    physical-substrate certification; S.2 through S.12 must add lanes and
    oracles, not fork new harness architecture.
- `PhysicalReferenceAuthority`
  - owns sealed physical id construction, generation-bearing reference
    validation, stale-reference denial, and physical-reference canonical basis
- `PhysicalHeaderDecodeWitness`
  - owns admitted page/frame header decode, incompatible-kind denial,
    length-bound checks, reserved integrity/recovery fields, and payload-view
    eligibility
- `FramedRecordPlacement`
  - owns slot-directory state, framed payload bounds, record placement class,
    moved/deleted/free/reserved slot semantics, and extent-backed references
- `SegmentManifestAuthority`
  - owns segment membership, segment-local page discovery, segment format
    version posture, and segment traversal counters
- `ExtentManifestAuthority`
  - owns variable extent membership, extent-backed record discovery, large
    payload placement posture, and extent traversal counters
- `PhysicalRootManifest`
  - owns the physical discovery root for reopen, verifier walk, allocation
    classes, free-space maps, segment manifests, and extent manifests
- `PlatformPhysicalFacade`
  - owns append, read, scan, locate, root publication, and reopen entry points
    for the platform-grade S.1 backend
- `OfflinePhysicalVerifier`
  - owns independent persisted-byte inspection without live runtime state,
    backend-private maps, semantic decode, or heap materialization
- `FoundationalPhysicalEvidenceBundle`
  - owns foundational canonical basis, diagnostics, profiles, provenance,
    support truth, completed-boundary receipts, and counter-backed performance
    receipts for S.1 evidence crossing Store boundaries
- `S2PhysicalSubstrateReadiness`
  - owns the typed handoff to S.2 proving pages, frames, references, manifests,
    counters, and verifier evidence are ready for buffer-pool work

The detailed phase plan names supporting products that protect the spine.
Supporting products are not automatically public authority surfaces.

## Scope

### In Scope

- physical id vocabulary for pages, segments, extents, frames, manifests, root
  manifests, allocation classes, free-space maps, and physical references
- fixed-size page model for ordinary records
- variable-size extent model for large records and future chunk families
- page and frame headers with kind, version, length, checksum slot,
  generation, publication state, and reserved LSN fields
- slot directory or equivalent page-local record addressing
- record framing independent of serde domain object materialization
- segment manifests, extent manifests, free-space maps, and root manifests
- platform-grade backend facade for physical record append, scan, locate,
  reopen, and manifest traversal
- stale-generation detection for physical references
- explicit capability-tier fencing for existing heap/file/SQLite paths
- first offline-verifier read path for S.1 physical structures
- S.1 evidence bundle outputs and exact physical counters
- foundational canonical basis and digest participation for S.1 layout,
  manifest, reference, verifier, counter, and evidence surfaces
- foundational diagnostic, profile, provenance, receipt, boundary-artifact, and
  performance vocabulary at S.1 evidence boundaries
- Roadmap 2 scenario harness, quality lanes, drivers, observers, counter
  expectation harness, denial expectation harness, proof oracles, lowered
  scenario plans, and stable physical-law proof transcripts

### Explicitly Out Of Scope

- bounded buffer pool, eviction, page leases, dirty-page budgets, and
  resident-memory certification, which are S.2
- full checksum, scrub, quarantine, and corruption-localization behavior, which
  are S.3
- WAL segment redesign, pageLSN replay law, checkpoint manifests, and recovery
  source precedence, which are S.4
- latch discipline, stable physical read plans, COW publication, and reclaim
  barriers, which are S.5
- hardware-aware I/O QoS, queue-depth management, and fsync profile
  qualification, which are S.6
- native blob chunk trees and streaming blob lifecycle, which are S.7
- per-artifact-family index/layout strategy selection, which is S.8
- formal models, operator repair, security, tenant boundaries, and final
  physical certification, which are S.9 through S.12
- query semantics, subscription semantics, truth validation, identity
  semantics, and runtime MVCC, which remain outside Store physical authority

## Physical Authority Exclusivity Laws

### Handoff Consumption Law

S.1 consumes `AcceptedHandoffReadiness` from S.0 and nothing weaker. S.1 may
inspect S.0 projections for human diagnostics, but it must not parse them to
reconstruct source, claim, vocabulary, capability, or certification authority.

### Physical Story Harness Law

After Phase 2, Roadmap 2 certification tests must be expressible as named
physical law stories that lower into inspectable executable plans. Scenario
steps must describe the physical law being established or attacked, not private
implementation convenience. Every story emits a stable proof transcript,
counter trace, denial trace, and runtime/verifier parity trace when those
concepts participate in the lane.

The harness progression is mandatory:

`PhysicalScenarioDefinition`
-> `PhysicalScenarioPlan`
-> `PhysicalScenarioExecution`
-> `ObservedPhysicalTrace`
-> `PhysicalProofOracleVerdict`
-> `PhysicalStoryTranscript`

Definitions own readable intent. Plans own required capabilities, storage
boundary crossings, expected physical footprint, expected counters, expected
denial boundary, driver requirements, observer requirements, and artifact
policy before execution. Executors run lowered plans without re-deciding proof
strategy. Observers collect facts. Oracles judge facts by named physical laws.
Transcripts preserve what happened.

The harness is Roadmap 2 infrastructure. Later sequences add lane families,
drivers, observers, and oracles for buffer pools, integrity, WAL, isolation,
I/O, blobs, indexes, formal-model alignment, operations, security, and final
certification. They must not create sequence-local harnesses that bypass this
progression.

### Header Decode Exclusivity Law

After Phase 5, no logical decoder, record locator, verifier payload reader, or
physical facade method may consume raw backend bytes directly. They must
consume `PhysicalHeaderDecodeWitness` or a stronger admitted physical view.

### Physical Reference Exclusivity Law

After Phase 3, no persisted or reopen-capable physical address may be a raw
integer, filename, byte offset, SQLite row id, vector index, path string, or
semantic artifact id. It must be a Store-owned physical reference carrying
segment/page-or-extent/slot-or-frame/generation evidence.

### Generation Ownership Law

A physical reference carries the generation of the smallest reusable physical
cell it addresses. Page-slot references validate against slot or allocation
generation. Extent-backed references validate against extent or allocation
generation. Root references validate against root publication generation. Page
and segment generations may participate in broader admission, but they do not
substitute for cell-reuse generation.

### Record Locate Exclusivity Law

After Phase 6, ordinary record location consumes page-local slot state and
framed record placement evidence. It must not locate records by deserializing a
whole page, whole segment, whole file, or whole store into domain objects.

### Moved Slot Honesty Law

S.1 may represent moved slots only as typed denial or as a single-hop physical
forwarding record with bounded counters. Multi-hop forwarding, semantic
redirects, cross-page forwarding policy, and compaction policy are deferred
unless explicitly implemented, bounded, and certified in S.1. A moved slot must
never become an unbounded locate path.

### Manifest Discovery Exclusivity Law

After Phase 8, reopen, scan, locate, and offline verification consume
`PhysicalRootManifest` and admitted segment/extent manifests. They must not use
backend-private directory layout, leftover files, object maps, table schemas,
or successful filesystem enumeration as physical discovery authority.

### Root Publication Honesty Law

S.1 root publication proves deterministic publication under admitted clean-close
and explicit test-interposer scenarios. Crash recovery, torn-write repair, and
recovery source precedence belong to S.4. S.1 must reserve enough root
publication evidence for S.4 and must typed-deny ambiguous root states rather
than guessing.

### Platform Facade Exclusivity Law

After Phase 10, platform-grade S.1 lanes enter through the
`PlatformPhysicalFacade`. Private test-only append/read helpers may exist only
inside narrow fixtures and may not mint platform-grade evidence.

### Verifier Independence Law

After Phase 9, S.1 evidence must be independently inspectable from persisted
bytes by `OfflinePhysicalVerifier`. A verifier that shares live runtime caches,
backend-private maps, or semantic object graphs cannot close S.1.

### S.2 Entry Exclusivity Law

After S.1 closeout, S.2 consumes `S2PhysicalSubstrateReadiness` or stronger. S.2
must not consume raw pages, legacy backend handles, private physical-format
modules, or S.1 evidence projections as substitutes for page/frame/reference
authority.

## Planned Directory Skeleton

The implementation should preserve this ownership topology unless a later
implementation plan explains a stricter replacement.

`worth-store-contracts` owns cross-crate contracts and proof-bearing public
types:

```text
src/
  physical_identity/
  physical_format_contracts/
  physical_capability/
  physical_evidence/
  physical_handoff/
```

`worth-store-physical-format` owns Store physical byte layout and byte-level
mechanics:

```text
src/
  binary_encoding/
  headers/
  references/
  pages/
  slots/
  frames/
  segments/
  extents/
  manifests/
  free_space/
  facade/
```

`worth-store-readiness` owns S.1 entry and S.2 handoff workflows:

```text
src/
  s1_entry/
  physical_substrate_readiness/
  s2_handoff/
```

`worth-store-certification` owns Roadmap 2 proof meaning, scenario progression,
oracles, lanes, transcripts, and evidence materialization. S.1 contributes the
first lane family; later storage-foundation sequences extend the same harness:

```text
src/
  scenario_definition/
  scenario_planning/
  scenario_execution/
  proof_oracles/
  proof_transcripts/
  evidence_bundles/
  drivers/
    adversarial_byte_device/
    persisted_file_device/
    crash_interposer/
    legacy_backend_probe/
    platform_backend_driver/
    verifier_only_driver/
  observers/
    counter_observer/
    storage_boundary_observer/
    materialization_observer/
    runtime_layout_observer/
    offline_verifier_observer/
    denial_boundary_observer/
    evidence_export_observer/
  lanes/
    physical_substrate/
    buffer_pool/
    integrity/
    wal_recovery/
    physical_isolation/
    io_qos/
    blob_chunks/
    layout_indexes/
    formal_model_alignment/
    operations_repair/
    security_tenant/
    physical_certification/
```

`worth-store-test-support` owns reusable mechanical fixtures and deterministic
test machinery. It must not own story meaning, proof lanes, oracle semantics,
or certification evidence authority:

```text
src/
  byte_devices/
  fault_injection/
  persisted_fixtures/
  clocks_and_schedulers/
  workload_generators/
  counter_capture/
  temp_store_layouts/
```

Skeleton rules:

- public facades aggregate; they must not implement physical byte mechanics
- physical-format internals remain private unless exported through contracts or
  the platform facade
- certification fixtures may not become production constructors
- scenario setup may mutate private backend state only inside explicitly named
  adversarial fixtures; ordinary platform-grade lanes must use production
  constructors and facade operations
- story transcripts must remain stable enough to compare across replay
- new database capabilities add lane families, observers, drivers, and oracles;
  they do not add new harness architecture
- `worth-store-test-support` may be dumb and reusable, but
  `worth-store-certification` owns what the fixture proves
- no directory may be named after `s1`, `phase`, `helpers`, `utils`, or
  `common` unless the directory is a certification profile or artifact output
  where the sequence itself is the subject

## Required Contracts And Counters

### Roadmap 2 Scenario Harness Contract

Required surfaces:

- `PhysicalScenarioDefinition`
- `PhysicalScenarioPlan`
- `PhysicalScenarioExecution`
- `ObservedPhysicalTrace`
- `PhysicalProofOracleVerdict`
- `PhysicalStoryTranscript`
- `PhysicalScenarioDriver`
- `PhysicalScenarioObserver`
- `PhysicalProofOracle`

Rules:

- every certification story starts as a definition that names the physical law
  it proves or attacks
- every definition must lower into a scenario plan before execution
- every plan must expose required capabilities, driver requirements, observer
  requirements, storage boundary crossings, expected physical footprint,
  expected counters, expected denial boundary, expected verifier/runtime
  relationship, artifact policy, and transcript identity basis
- executors may run lowered plans but may not re-decide driver class, observer
  set, proof strategy, artifact policy, expected counters, or denial boundary
- observers collect facts and must remain independent of proof oracles
- proof oracles judge observed facts by named architectural laws, not by
  anonymous assertion chains
- transcripts are stable evidence artifacts and must be replay-comparable
- S.2 through S.12 extend the harness by adding lane families, drivers,
  observers, and oracles; they must not fork the definition/planning/execution
  architecture
- `worth-store-test-support` may provide reusable bytes, clocks, faults,
  workloads, temporary layouts, and counter capture; it may not own proof-lane
  meaning, oracle verdicts, or evidence-bundle authority

Naive trap this prevents:

- building a polished S.1 story DSL that becomes unusable for WAL crash
  matrices, blob streaming, repair forensics, tenant/security tests, and S.12
  certification, forcing each later sequence to invent its own harness.

### Foundational Adoption Contract

Required `worth-foundational` source surfaces at exported, compared,
certified, support, and handoff evidence boundaries:

- `worth_foundational::canonicalization_api::lower_lane::basis`
  for S.1 canonical basis sequences over physical layout reports, manifest
  summaries, reference identity, verifier observations, and evidence bundles
- `worth_foundational::canonicalization_api::lower_lane::digest`
  for derived digests of those canonical basis artifacts, with Store-owned
  digest-domain wrappers where Store needs stricter physical meaning
- `worth_foundational::canonicalization_api::lower_lane::comparison`
  for verifier/runtime parity and mismatch classification
- `worth_foundational::profiles_api::lower_lane`
  for diagnostic richness, support posture, compatibility posture,
  certification posture, retention/delivery posture, and materialization
  profile attachment
- `worth_foundational::boundary_evidence_api::lower_lane`
  for provenance, freshness posture, planned-versus-executed receipt
  categories, support truth, degraded/reconstructed evidence, and completed
  boundary receipts where S.1 has real completed-boundary evidence
- `worth_foundational::performance_api::lower_lane`
  for layout intent, performance claim posture, policy-admitted performance
  claims, exact counter-backed receipts, and materialized performance reports
- `worth_foundational` diagnostic primitives and categories exported through
  the facade for S.1 diagnostic rows, denial rows, support reports, and
  certified bundle attachments when the evidence crosses a support or
  certification boundary

Required Store-owned surfaces:

- `PhysicalPageId`, `PhysicalSegmentId`, `PhysicalExtentId`,
  `PhysicalFrameId`, `PhysicalRecordSlot`, `PhysicalGeneration`,
  `PhysicalEpoch`, `PhysicalReference`, and `PhysicalRootReference`
- `PhysicalPageHeader`, `PhysicalFrameHeader`, page/frame kind enums,
  publication-state enums, manifest types, slot directories, free-space maps,
  stale-reference denials, and physical substrate counters
- S.1 digest-domain wrappers that preserve whether a digest names a physical
  layout report, a root manifest, a segment manifest, an extent manifest, a
  verifier observation, a counter bundle, or a failure report

Rules:

- Store must consume foundational vocabulary through public facade or grouped
  public lanes. Deep imports into foundational internals are forbidden.
- Store may not mint local compatibility stand-ins for foundational canonical
  basis, diagnostic rows, profile sets, performance receipts, provenance, or
  boundary receipt categories unless an explicit S.1 adoption-debt row names
  the missing upstream surface and a removal gate.
- A foundational digest, receipt, profile, diagnostic row, or boundary evidence
  attachment may support a Store physical claim but cannot promote itself into
  Store physical authority.
- Store-owned physical references must preserve identity authority across
  persisted bytes. Foundational identity and locator categories describe the
  boundary meaning; Store reference types enforce byte placement, generation,
  and stale-reuse law.
- S.1 evidence bundles must be canonicalizable through foundational basis
  APIs before their digest is accepted as evidence identity.
- Diagnostic richness and support materialization must be profile-governed
  through foundational profile/materialization vocabulary, not ad hoc boolean
  flags or broad string modes.
- Exact physical counters must attach to foundational counter-backed
  performance receipts before any S.1 performance or boundedness claim may be
  exported.

Naive trap this prevents:

- replacing `worth-foundational` with local `StableDigest`, `DiagnosticRow`,
  `EvidenceBundle`, `Profile`, or `Receipt` lookalikes and then discovering
  that S.1 evidence cannot be compared, profiled, certified, or interpreted by
  the rest of Worth without producer-private folklore.

### Binary Physical Format Contract

Required surfaces:

- `PhysicalFormatMagic`
- `PhysicalFormatVersion`
- `PhysicalByteOrder`
- `PhysicalPageSizeClass`
- `PhysicalAlignmentClass`
- `PhysicalHeaderFieldWidth`
- `PhysicalReservedField`
- `PhysicalForwardCompatibilityPolicy`
- `PhysicalBinaryEncodingWitness`

Rules:

- all persisted S.1 format structures must declare byte order, integer widths,
  header length encoding, payload length encoding, generation width, id width,
  magic bytes, format version, and reserved-field policy
- page size policy must name admitted page-size classes and reject unsupported
  page sizes before allocation
- alignment requirements must be explicit for page starts, frame starts, slot
  directory offsets, extent starts, and manifest records
- unknown page/frame/manifest kinds fail typed unless a declared forward
  compatibility policy admits skip, preserve, or unsupported behavior
- format admission, format rejection, format preservation, format migration,
  and format downgrade refusal must be distinct postures. S.1 may declare full
  migration out of scope, but it must reserve an explicit future migration lane
  and must never treat downgrade or unknown-kind preservation as successful
  admission.
- reserved fields may be serialized only with a declared zero/preserved/ignored
  rule; they may not contain backend-private meaning
- binary format construction must not depend on serde map ordering, struct
  field layout, Rust enum discriminants, host endianness, pointer width, or
  platform path spelling
- canonical binary encoding evidence must lower into foundational canonical
  basis before a format digest is accepted

Naive trap this prevents:

- writing Rust structs or serde blobs to disk and discovering later that the
  format cannot be reopened, compared, verified, migrated, or decoded
  consistently across platforms.

### Scale And Locality Contract

Required surfaces:

- `PhysicalOperationComplexityContract`
- `PhysicalOperationCounterSnapshot`
- `PhysicalLocalityClass`
- `PhysicalManifestIndex`
- `PhysicalFreeSpaceSearchPolicy`
- `PhysicalFragmentationPressureReport`
- `PhysicalForegroundBoundednessReport`

Rules:

- locating a record by admitted physical reference must not scan unrelated
  pages, segments, extents, manifests, or domain objects
- appending an ordinary record must not scan all pages or all free-space entries
  when an allocation class or free-space index can bound candidate search
- root manifest open may scale with root entries, but later locate operations
  must consume indexed manifest state or admitted handles rather than repeat a
  full root walk
- multi-segment scan must be an explicit scan operation with scan counters; it
  must not be hidden behind locate, reopen, or append
- fragmented free space must produce bounded search, typed defer, typed denial,
  or a maintenance signal; it must not turn foreground append into an unbounded
  free-space walk
- every operation that can touch multiple segments, pages, extents, manifests,
  or free-space classes must expose separate counters for each touched family
- counters prove observed work for one execution; complexity verification also
  requires algorithm review, hostile fixtures, and scale/property evidence

Naive trap this prevents:

- building correct page structures whose foreground locate or append path still
  degrades into full-store metadata scans as the store grows.

### Physical Identity Contract

Required surfaces:

- `PhysicalPageId`
- `PhysicalSegmentId`
- `PhysicalExtentId`
- `PhysicalFrameId`
- `PhysicalGeneration`
- `PhysicalEpoch`
- `PhysicalRecordSlot`
- `PhysicalReference`
- `PhysicalRootReference`
- `StalePhysicalReference`

Rules:

- no physical reference may omit segment id, page or extent id, slot/frame
  position, and generation
- the generation in a physical reference belongs to the smallest reusable
  physical cell being addressed: slot/allocation generation for page-slot
  references, extent/allocation generation for extent-backed references, and
  root publication generation for root references
- page or segment generation may participate in admission but must not
  substitute for cell-reuse generation
- stale generation reuse must produce a typed stale-reference failure before
  logical decode
- physical ids are placement identities, not semantic artifact identities
- semantic artifact ids may point to physical references, but physical
  references may not replace semantic ids in canonical artifacts
- physical references may be serialized only through versioned framing owned by
  the physical substrate

Naive trap this prevents:

- reusing a row id, filename, SQLite rowid, or vector index as a durable
  database reference and discovering after compaction or restart that it was a
  placement hint, not a stable physical address.

### Page And Frame Header Contract

Required surfaces:

- `PhysicalPageHeader`
- `PhysicalFrameHeader`
- `PhysicalPageKind`
- `PhysicalFrameKind`
- `PhysicalFormatVersion`
- `PhysicalPublicationState`
- `PhysicalHeaderDecodeReport`
- `PhysicalHeaderDecodeWitness`

Header fields must include at minimum:

- kind
- format version
- header length
- payload length
- checksum slot or checksum descriptor reserved for S.3
- generation
- publication state
- segment id
- page or extent id
- reserved pageLSN or recovery cursor field for S.4

Rules:

- a header decode witness is required before frame payload access
- logical artifact decoders may consume only framed payload views, not raw
  backend bytes
- unknown page or frame kinds fail typed unless the format version declares an
  admitted compatibility posture
- publication state may describe only physical publication, never semantic
  commit truth
- S.1 may reserve checksum fields but must not pretend reserved checksum slots
  prove S.3 integrity behavior

Naive trap this prevents:

- storing a serde blob with a digest beside it and treating successful
  deserialization as proof that the physical storage substrate exists.

### Page-Local Record Addressing Contract

Required surfaces:

- `SlotDirectory`
- `SlotDirectoryEntry`
- `FramedRecordView`
- `FramedRecordPayload`
- `RecordPlacementClass`
- `RecordPlacementWitness`
- `RecordLocateReport`

Rules:

- ordinary records are located through page-local slot entries or an explicitly
  equivalent page-local addressing table
- record location must not require deserializing all records on a page
- deleted, moved, free, and reserved slots must be distinguishable in the slot
  directory
- record length and frame length must be checked before payload view creation
- a record view may expose bytes for later logical decode, but may not perform
  semantic interpretation inside the physical substrate
- extent-backed records must carry an extent reference from the page or
  manifest record that owns the large payload

Naive trap this prevents:

- scanning a whole file into `Vec<DomainRecord>` and then claiming the resulting
  vector offsets are page-local physical addressing.

### Segment, Extent, And Manifest Contract

Required surfaces:

- `SegmentManifest`
- `ExtentManifest`
- `PhysicalRootManifest`
- `SegmentMembershipRecord`
- `ExtentMembershipRecord`
- `AllocationClass`
- `FreeSpaceMap`
- `ManifestTraversalPlan`
- `ManifestTraversalReport`

Rules:

- every allocated page belongs to exactly one segment manifest
- every variable extent belongs to exactly one extent manifest or segment-owned
  extent membership record
- the root manifest is sufficient for a verifier to discover admitted segments,
  extents, format versions, allocation classes, and free-space maps
- allocation classes are physical-placement classes only; they may not encode
  semantic artifact authority
- free-space maps may report candidate placement and reuse state, but stale
  generation checks must still protect physical references
- manifest traversal emits exact counters for manifest reads, segment entries,
  extent entries, allocation-class entries, and free-space-map entries

Naive trap this prevents:

- letting the host filesystem directory tree, SQLite schema tables, or current
  backend object map be the real manifest while the Store manifest is only a
  decorative summary.

### Legacy Capability Fence Contract

Required surfaces:

- `StoreBackendCapabilityTier`
- `BootstrapBackend`
- `SemanticCertificationBackend`
- `CompatibilityBackend`
- `PhysicalFoundationBackend`
- `PlatformGradeBackend`
- `ForbiddenPlatformClaim`
- `LegacyBackendClassificationReport`

Rules:

- an existing heap/file/SQLite path may continue to run only under an explicit
  non-platform-grade capability tier until it satisfies Roadmap 2 gates
- certification lanes must be able to assert that the platform-grade S.1 suite
  did not route through legacy heap-shaped paths
- a backend may be useful and non-platform-grade at the same time; the tier is
  a claim boundary, not a value judgment
- a platform-grade backend must expose S.1 physical structures through the
  physical facade and offline verifier
- any API or documentation phrase that implies platform-grade posture for a
  legacy backend must map to an S.0/S.1 cleanup item

Naive trap this prevents:

- keeping old persistence paths around for compatibility while accidentally
  allowing tests, demos, or docs to use them as evidence that the real database
  substrate exists.

### S.1 Counter Contract

Required counters:

- `physical_page_read_count`
- `physical_page_write_count`
- `physical_frame_decode_count`
- `physical_header_decode_count`
- `physical_record_locate_count`
- `physical_manifest_lookup_count`
- `physical_manifest_traversal_count`
- `physical_segment_manifest_read_count`
- `physical_extent_manifest_read_count`
- `physical_allocation_count`
- `physical_free_space_map_update_count`
- `physical_root_manifest_read_count`
- `physical_root_manifest_publish_count`
- `physical_root_manifest_entry_count`
- `physical_manifest_index_probe_count`
- `physical_free_space_candidate_scan_count`
- `physical_free_space_class_probe_count`
- `physical_fragmentation_pressure_signal_count`
- `physical_stale_generation_rejection_count`
- `physical_unknown_kind_rejection_count`
- `physical_length_mismatch_rejection_count`
- `physical_binary_encoding_admission_count`
- `physical_logical_decode_after_invalid_header_count`
- `physical_whole_store_materialization_attempt_count`
- `physical_legacy_backend_platform_claim_rejection_count`

Rules:

- required certification lanes must assert exact counter values or exact zeroes
  where the scenario demands them
- `physical_logical_decode_after_invalid_header_count` must remain zero
- `physical_whole_store_materialization_attempt_count` must remain zero for
  platform-grade S.1 lanes
- `physical_legacy_backend_platform_claim_rejection_count` must match hostile
  legacy-claim lanes exactly
- any locate-by-reference lane must assert exact zeroes for unrelated page,
  segment, extent, and manifest scans
- append lanes must assert bounded free-space candidate scans under fragmented
  free-space fixtures

## Phases

### Phase 1: Establish Physical Vocabulary And Capability Fences

Phase 1 creates the naming, tiers, foundational adoption rows, and forbidden
claim boundaries that keep the rest of S.1 honest. This phase closes the
question "what may claim S.1 physical foundation evidence?" before any bytes
are written.

**Relevant subsystems**

- `worth-store-contracts`
- `worth-store-physical-format`
- `worth-store-readiness`
- `worth-store-certification`
- `worth-foundational`

**Relevant APIs**

- `AcceptedHandoffReadiness`
- `StoreCapabilityTier`
- `PlatformGradeClaimWitness`
- `StorePhysicalAuthorityWitness`
- `FoundationalVocabularyAdoptionMap`
- `ProofVocabularyAdoptionMap`
- `worth_foundational::profiles_api::lower_lane`
- `worth_foundational::boundary_evidence_api::lower_lane`
- `worth_foundational::performance_api::lower_lane`

**Required build shape**

- This phase adds physical id, generation, epoch, reference, page, frame,
  segment, extent, manifest, allocation-class, and free-space-map vocabulary.
- This phase adds `StoreBackendCapabilityTier` or extends the existing tier
  surface so legacy heap/file/SQLite paths are explicitly classified as
  bootstrap, semantic-certification, compatibility, physical-foundation, or
  platform-grade candidates.
- This phase adds S.1 adoption rows that name every foundational family S.1
  consumes: canonicalization, diagnostics, profiles, boundary evidence,
  provenance/receipts, and performance.
- This phase adds forbidden-claim reports for any backend or test lane that
  attempts to satisfy S.1 through heap-shaped persistence.
- This phase defines the S.1 physical facade separately from semantic Store
  facade methods.
- This phase defines the S.1 evidence bundle fields:
  `physical_layout_report`, `artifact_digest`, `failure_digest`,
  `counter_snapshot`, `resource_envelope_report`, and
  `hardware_assumption_report`.

**Warnings**

- Do not name crates, modules, or evidence after `S.1`; sequence numbers belong
  in docs and evidence metadata, not structural ownership names.
- Do not let `worth-foundational` vocabulary adoption become a second physical
  authority lane. It supports boundary meaning; Store owns byte survival.
- Do not allow old persistence paths to keep platform-grade language because
  they are useful for bootstrap or semantic certification.

**Test requirements**

- Adversarial parity: the S.1 foundational adoption map must canonicalize and
  digest through foundational basis APIs to the same identity across two
  independent construction paths.
- Adversarial denial: a legacy heap/file/SQLite backend attempting to present a
  platform-grade S.1 claim is rejected with a typed forbidden-claim report.
- Compile-fail: raw strings, local profile names, local diagnostic names, or
  local receipt lookalikes cannot satisfy the S.1 foundational adoption APIs.

**Engineering decisions**

- S.1 begins with claim and vocabulary authority because later phases consume
  those typed categories.
- Foundational adoption is modeled as evidence-bearing rows, not prose in the
  spec.
- Existing backends are fenced by capability tier before the platform-grade
  facade exists, so tests cannot accidentally prove the wrong thing.

**Open questions**

- None.

### Phase 2: Build Roadmap 2 Scenario Harness And Quality Grammar

Phase 2 builds the executable proof architecture Roadmap 2 uses to develop and
certify the database. This phase closes the risk that S.1 tests become
readable but sequence-local, or that later WAL, blob, repair, security, and
S.12 certification work invent incompatible harnesses.

**Relevant subsystems**

- `worth-store-certification`
- `worth-store-test-support`
- `worth-store-contracts`
- `worth-store-physical-format`
- `worth-foundational`

**Relevant APIs**

- `PhysicalScenarioQualityHarness`
- `PhysicalScenarioDefinition`
- `PhysicalScenarioPlan`
- `PhysicalScenarioExecution`
- `ObservedPhysicalTrace`
- `PhysicalProofOracleVerdict`
- `PhysicalStoryTranscript`
- `ScenarioCounterTrace`
- `ScenarioDenialTrace`
- `RuntimeVerifierParityTrace`
- `FixtureAdversaryReport`
- `PhysicalScenarioDriver`
- `PhysicalScenarioObserver`
- `PhysicalProofOracle`
- `AdversarialByteDeviceDriver`
- `PersistedFileDeviceDriver`
- `CrashInterposerDriver`
- `LegacyBackendProbeDriver`
- `PlatformBackendDriver`
- `VerifierOnlyDriver`
- `CounterObserver`
- `StorageBoundaryObserver`
- `MaterializationObserver`
- `RuntimeLayoutObserver`
- `OfflineVerifierObserver`
- `DenialBoundaryObserver`
- `EvidenceExportObserver`

**Required build shape**

- This phase implements the mandatory harness progression:
  `PhysicalScenarioDefinition` -> `PhysicalScenarioPlan` ->
  `PhysicalScenarioExecution` -> `ObservedPhysicalTrace` ->
  `PhysicalProofOracleVerdict` -> `PhysicalStoryTranscript`.
- This phase defines story definitions as readable intent. Definitions name the
  physical law being proved or attacked and may use builder progression for
  ordered proof steps, but they do not execute storage operations directly.
- This phase defines scenario plans as the accountability surface. A plan must
  expose required capabilities, driver requirements, observer requirements,
  storage boundary crossings, expected physical footprint, expected counters,
  expected denial boundary, expected verifier/runtime relationship, workload
  scale, fixture adversary posture, artifact policy, and transcript identity
  basis before execution.
- This phase defines scenario execution as a lowered-plan executor. The
  executor may run plans but may not re-decide proof strategy, capability
  requirements, artifact policy, observer set, driver class, or expected
  locality/counter contracts.
- This phase defines observers as independent fact collectors:
  `CounterObserver`, `StorageBoundaryObserver`, `MaterializationObserver`,
  `RuntimeLayoutObserver`, `OfflineVerifierObserver`,
  `DenialBoundaryObserver`, and `EvidenceExportObserver`.
- This phase defines proof oracles as named architectural judgments:
  `NoWholeStoreMaterialization`, `HeaderDecodePrecedesPayloadView`,
  `StaleGenerationDeniedBeforeLogicalDecode`,
  `VerifierRuntimeLayoutParity`,
  `LocateByReferenceIgnoresUnrelatedGrowth`, and
  `RootAmbiguityDeniedNotGuessed`.
- This phase defines drivers for execution substrates:
  adversarial byte device, persisted file device, crash/interruption
  interposer, legacy backend probe, platform backend candidate, and
  verifier-only reader.
- This phase defines story steps named by physical laws:
  `given_clean_physical_store`, `given_legacy_backend_attempts_platform_claim`,
  `when_authoritative_record_is_appended`,
  `when_store_closes_and_reopens_from_bytes`,
  `then_record_locates_by_physical_reference`,
  `then_no_whole_store_materialization_occurred`, and
  `then_offline_verifier_matches_runtime_layout`.
- This phase defines the initial `lanes/physical_substrate` family for S.1:
  `happy_authority_lane`, `hostile_reference_lane`,
  `hostile_format_lane`, `legacy_overclaim_lane`,
  `offline_verifier_lane`, `scale_locality_lane`,
  `foundational_export_lane`, and `s2_handoff_lane`.
- This phase reserves lane families for S.2 through S.12:
  `buffer_pool`, `integrity`, `wal_recovery`, `physical_isolation`, `io_qos`,
  `blob_chunks`, `layout_indexes`, `formal_model_alignment`,
  `operations_repair`, `security_tenant`, and `physical_certification`.
- This phase makes every story emit stable machine-checkable outputs:
  `physical_story_transcript`, `scenario_counter_trace`,
  `scenario_denial_trace`, `runtime_verifier_parity_trace`,
  `shortcut_rejection_trace`, and `fixture_adversary_report`.
- This phase maps every acceptance matrix row to one or more quality lanes so
  proof topology is visible before implementation starts.

**Warnings**

- Do not build a cute test DSL that hides physical authority. The grammar must
  name the law each step proves or attacks.
- Do not let the story layer own mechanics. Stories define proof intent; plans
  own resolved capability and cost; drivers own substrate execution; observers
  collect facts; oracles judge facts; transcripts preserve evidence.
- Do not let scenario setup mutate private backend state unless the story names
  an adversarial fixture boundary.
- Do not let story helpers become production constructors.
- Do not put lane meaning into `worth-store-test-support`; test support is
  reusable machinery, not certification authority.
- Do not add a WAL harness, blob harness, repair harness, or S.12 harness.
  Add lane families, observers, drivers, and oracles to the Roadmap 2 harness.

**Test requirements**

- Adversarial parity: the same scenario definition lowers into the same
  scenario plan, and replay through runtime and verifier observers emits
  stable transcript identity, counter trace identity, and parity-trace identity
  for admitted structures.
- Adversarial denial: a story that tries to satisfy platform-grade evidence
  through legacy overclaim, backend residue, whole-store materialization, or a
  sequence-local harness shortcut fails during plan admission or oracle
  judgment with the named boundary recorded in the denial trace.
- Harness-quality proof: every S.1 certification row is assigned to a named
  `physical_substrate` lane and emits a transcript that can be read without
  inspecting private helper calls.
- Roadmap-scale proof: stub lane-family definitions for S.2 through S.12 can
  register oracle families and driver requirements without changing the
  harness architecture or moving fixtures out of `worth-store-test-support`.

**Engineering decisions**

- The harness is a first-class Roadmap 2 authority product because it turns the
  physical database architecture into executable language.
- Friendly stories lower into inspectable plans before execution.
- Definitions, plans, execution, observers, oracles, transcripts, and evidence
  bundles are separate responsibilities.
- Counter assertions attach to the story step that caused the work.
- Denial assertions name the boundary where failure occurred.
- Runtime/verifier parity is a harness workflow, not a late closeout
  convenience.
- New database capabilities add lanes and oracles, not new harness structure.

**Open questions**

- None.

### Phase 3: Define Binary Physical Format Law

Phase 3 makes persisted bytes deterministic before any physical reference,
header witness, page, or manifest can claim reopen authority. This phase closes
the serializer-accident hole.

**Relevant subsystems**

- `worth-store-contracts`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-foundational` canonicalization APIs

**Relevant APIs**

- `PhysicalFormatMagic`
- `PhysicalFormatVersion`
- `PhysicalByteOrder`
- `PhysicalPageSizeClass`
- `PhysicalAlignmentClass`
- `PhysicalForwardCompatibilityPolicy`
- `PhysicalBinaryEncodingWitness`
- `worth_foundational::canonicalization_api::lower_lane::basis`
- `worth_foundational::canonicalization_api::lower_lane::digest`

**Required build shape**

- This phase defines byte order, integer widths, header length encoding,
  payload length encoding, generation width, id width, magic bytes, format
  version, and reserved-field policy for every persisted S.1 structure.
- This phase defines admitted page-size classes and alignment requirements for
  pages, frames, slot directories, extents, and manifests.
- This phase defines unknown-kind and forward-compatibility behavior before any
  header decode witness can exist.
- This phase prohibits serde map ordering, Rust enum discriminants, host
  endianness, pointer width, and struct field layout from becoming physical
  format authority.
- This phase emits binary encoding admission counters and lowers binary format
  evidence into foundational canonical basis.

**Warnings**

- Do not write Rust structs to disk as the physical format.
- Do not leave page size, id width, or generation width as implementation
  details.
- Do not allow reserved fields to carry backend-private meaning.

**Test requirements**

- Adversarial parity: the same physical format declaration constructed through
  two independent paths canonicalizes to the same foundational basis and Store
  format digest.
- Adversarial denial: host-endian, serde-order-dependent, unsupported page
  size, unknown reserved-field policy, and unsupported forward-compatibility
  declarations are rejected before any persisted format witness is produced.
- Cross-platform fixture proof: golden bytes decode identically under explicit
  byte-order and field-width rules.

**Engineering decisions**

- Binary format law precedes physical references because references are
  persisted bytes, not just in-memory handles.
- Store owns the binary format; Foundational owns canonical evidence over the
  format declaration.
- S.1 reserves recovery and integrity fields but does not claim S.3 or S.4
  behavior.

**Open questions**

- None.

### Phase 4: Define Physical Identity And Generation Semantics

Phase 4 makes physical identity and generation reuse precise before any header
or payload view can rely on an address. This phase closes the stale-reference
hole by defining exactly which reusable physical cell owns the generation a
reference carries.

**Relevant subsystems**

- `worth-store-contracts`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-foundational` canonicalization and identity/locator vocabulary

**Relevant APIs**

- `PhysicalReference`
- `StalePhysicalReference`
- `PhysicalGenerationOwner`
- `PhysicalCellReuseDomain`
- `PhysicalReferenceAdmissionWitness`
- `worth_foundational::canonicalization_api::lower_lane::basis`
- `worth_foundational::canonicalization_api::lower_lane::digest`
- `worth_foundational::canonicalization_api::lower_lane::comparison`

**Required build shape**

- This phase implements sealed construction for physical ids and references.
- This phase implements stale-generation-aware reference validation.
- This phase defines generation ownership for page-slot references,
  extent-backed references, root references, free-space reuse, and broader page
  or segment admission.
- This phase requires page-slot references to validate against slot/allocation
  generation, extent-backed references to validate against extent/allocation
  generation, and root references to validate against root publication
  generation.
- This phase rejects any implementation where page generation or segment
  generation substitutes for the smallest reusable cell generation.
- This phase lowers reference identities and generation-owner rows into
  foundational canonical basis entries only when exported, compared, or
  certified.
- This phase exposes physical-reference-validation and stale-generation
  counters.
- This phase adds compile-fail coverage for synthesizing physical decode
  references outside physical substrate authority.

**Warnings**

- Do not use semantic artifact digests as physical reference identity.
- Do not validate a slot-level reference using only page or segment generation.
- Do not let free-space reuse happen without generation evidence.

**Test requirements**

- Adversarial parity: two independently constructed references to the same
  admitted physical cell carry the same generation owner and canonical
  reference basis.
- Adversarial denial: a slot-reuse, extent-reuse, free-space-reuse, or
  root-publication-reuse fixture rejects the old reference before any header or
  logical decode can run.
- Compile-fail: callers cannot construct `PhysicalReference` or
  `PhysicalReferenceAdmissionWitness` from raw fields outside the proving
  module.

**Engineering decisions**

- Reference admission is the first proof-bearing physical identity transition.
- Store wraps foundational digest values where the digest participates in
  Store-specific physical authority.
- Reference validation consumes generation-bearing physical reference types and
  produces a narrower admitted reference type for downstream payload access.

**Open questions**

- None.

### Phase 5: Define Header Decode Witnesses

Phase 5 makes raw bytes unobservable until the physical substrate has proven
what they are. This phase closes the proof chain from admitted physical
reference to header-admitted payload view.

**Relevant subsystems**

- `worth-store-contracts`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-foundational` canonicalization and diagnostic vocabulary

**Relevant APIs**

- `PhysicalHeaderDecodeWitness`
- `PhysicalHeaderDecodeReport`
- `PhysicalPageHeader`
- `PhysicalFrameHeader`
- `PhysicalHeaderKind`
- `PhysicalPayloadViewAdmission`
- `worth_foundational::canonicalization_api::lower_lane::basis`
- `worth_foundational::canonicalization_api::lower_lane::digest`
- foundational diagnostic row/category surfaces from the facade

**Required build shape**

- This phase defines page and frame header formats with kind, version, length,
  checksum slot, generation fields, publication state, reserved integrity
  fields, and reserved recovery fields.
- This phase defines header decode reports and witnesses.
- This phase rejects unknown or incompatible page/frame kinds before logical
  decode.
- This phase checks frame length before payload view construction.
- This phase reserves checksum and LSN fields without claiming S.3 corruption
  localization or S.4 recovery physics.
- This phase lowers header reports into foundational canonical basis only when
  the report is exported, compared, certified, or handed off.
- This phase exposes header-decode, unknown-kind, length-mismatch, and
  logical-decode-after-invalid-header counters.
- This phase adds compile-fail coverage for synthesizing
  `PhysicalHeaderDecodeWitness` outside physical substrate authority.

**Warnings**

- Do not expose raw payload bytes as a convenience accessor before header
  witness construction.
- Do not make checksum slots imply S.3 physical integrity completion.
- Do not let a semantic decoder be the first code that notices malformed bytes.

**Test requirements**

- Adversarial parity: two independently decoded copies of the same admitted
  header produce the same header witness basis and Store physical digest
  wrapper.
- Adversarial denial: unknown frame kind, unsupported version, length mismatch,
  and reserved-field misuse are rejected before payload view or logical decode,
  and `physical_logical_decode_after_invalid_header_count` remains zero.
- Compile-fail: callers cannot construct `PhysicalHeaderDecodeWitness` from
  raw fields outside the proving module.

**Engineering decisions**

- Header admission is the first proof-bearing physical byte transition.
- Store owns header law; Foundational only receives exported evidence over the
  header report.
- Payload view access is typed as the output of header admission, not as a raw
  byte slice convenience.

**Open questions**

- None.

### Phase 6: Implement Page And Slot Record Framing

Phase 6 turns pages into addressable containers rather than serialized bags.
This phase closes the ordinary-record addressing contract that S.2 and S.3 will
later lease, pin, validate, and attack.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-contracts`
- `worth-store-certification`
- `worth-foundational` performance and diagnostics vocabulary

**Relevant APIs**

- `SlotDirectory`
- `SlotDirectoryEntry`
- `FramedRecordView`
- `FramedRecordPayload`
- `RecordPlacementWitness`
- `worth_foundational::performance_api::lower_lane::receipts`
- `worth_foundational::performance_api::lower_lane::reports`
- foundational diagnostic row/category surfaces from the facade

**Required build shape**

- This phase implements fixed-size page storage for ordinary record families.
- This phase implements slot directory or equivalent page-local addressing.
- This phase distinguishes occupied, deleted, moved, free, and reserved slot
  states.
- This phase implements framed record views with checked length and placement
  class.
- This phase supports extent-backed record references for large payloads
  without loading the extent into a domain object.
- This phase attaches page-read, page-write, frame-decode, record-locate, and
  page-local scan counters to foundational counter-backed performance receipts.
- This phase materializes typed diagnostic rows for out-of-range slots, moved
  records without extent references, and length mismatches.

**Warnings**

- Do not deserialize all page records to find one slot.
- Do not let a moved slot become an implicit semantic redirect.
- Do not materialize rich diagnostics on the hot path unless the resolved
  profile admits that materialization.

**Test requirements**

- Adversarial parity: appending and locating the same record by page-local slot
  after reopen yields the same physical reference and canonical framed-record
  basis without domain deserialization.
- Adversarial denial: out-of-range slots, length mismatches, and moved slots
  without admitted follow-on references deny before payload view construction.
- Performance proof: slot lookup counter rows match exact expected values and
  are accepted into a foundational counter-backed performance receipt.

**Engineering decisions**

- Record views expose framed bytes only; semantic decode remains a later
  consumer after physical admission.
- Slot state is structural state, not an optional flag on a broad record info
  object.
- Performance receipts are produced after executed physical work, not from
  planned work or comments.

**Open questions**

- None.

### Phase 7: Implement Extent-Backed Large Record Framing

Phase 7 gives large records their own physical framing instead of hiding them
inside page-local logic. This phase closes the boundary between ordinary
page-slot placement and variable-size extent placement.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-contracts`
- `worth-store-certification`
- `worth-foundational` performance and diagnostics vocabulary

**Relevant APIs**

- `ExtentRecordReference`
- `ExtentAllocationClass`
- `ExtentFrameHeader`
- `ExtentBackedRecordPlacement`
- `ExtentTraversalCounterSnapshot`
- foundational performance receipt surfaces from the facade
- foundational diagnostic row/category surfaces from the facade

**Required build shape**

- This phase implements variable-size extent-backed record framing for large
  payloads and future chunk-family readiness.
- This phase defines the physical reference shape for extent-backed records,
  including extent/allocation generation ownership.
- This phase requires extent payload length and extent membership to be checked
  before payload view construction.
- This phase distinguishes extent-backed records from page-local moved slots.
- This phase records extent reads, extent header decodes, extent membership
  checks, and extent-backed locate counters.

**Warnings**

- Do not load an extent into a domain object to prove large-record access.
- Do not treat an extent reference as a semantic blob or artifact identity.
- Do not let page-local moved slots become unbounded extent traversal.

**Test requirements**

- Adversarial parity: a large record written through an extent-backed reference
  reopens and locates with the same extent generation evidence and framed
  payload bounds.
- Adversarial denial: stale extent generation, extent length mismatch, missing
  extent membership, and moved-slot-to-extent misuse deny before payload view
  or logical decode.
- Performance proof: extent-backed locate counters remain within the declared
  extent-local bound and produce counter-backed receipt evidence when exported.

**Engineering decisions**

- Extents are a physical substrate for large framed records, not the S.7 blob
  chunk tree.
- Extent-backed placement is a peer to page-slot placement, not a special case
  hidden inside `SlotDirectory`.
- S.1 may reserve chunk-family allocation classes without claiming native blob
  semantics.

**Open questions**

- None.

### Phase 8: Implement Segments, Allocation Classes, Free Space, And Root Manifests

Phase 8 gives pages and extents a discoverable physical universe. This phase
closes the source of physical discovery so reopen and verification do not rely
on backend-private directory layout or object maps.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-contracts`
- `worth-store-certification`
- `worth-foundational` canonical export, provenance, and profile surfaces

**Relevant APIs**

- `SegmentManifest`
- `ExtentManifest`
- `PhysicalRootManifest`
- `AllocationClass`
- `FreeSpaceMap`
- `ManifestTraversalPlan`
- `ManifestTraversalReport`
- `worth_foundational::canonicalization_api::lower_lane::export`
- `worth_foundational::boundary_evidence_api::lower_lane::provenance`

**Required build shape**

- This phase implements segment manifests and segment membership records.
- This phase implements variable-size extent manifests and extent membership
  records.
- This phase implements root manifest discovery for admitted physical
  structures.
- This phase implements allocation classes for ordinary pages, large extents,
  manifests, free-space maps, and reserved future chunk families.
- This phase implements free-space maps with generation-safe reuse posture.
- This phase implements manifest traversal plans and reports.
- This phase canonicalizes manifest traversal reports through foundational
  basis/export APIs and attaches provenance describing root, segment, extent,
  and free-space-map sources.
- This phase exposes allocation, free-space-map, root-manifest,
  segment-manifest, and extent-manifest counters.
- This phase adds reopen-from-bytes lanes that discover the store through the
  root manifest, not backend-private directory or object maps.

**Warnings**

- Do not let filesystem layout or current backend object maps become the real
  manifest authority.
- Do not encode semantic artifact authority in allocation classes.
- Do not allow free-space reuse without generation evidence.

**Test requirements**

- Adversarial parity: a fresh opener and a verifier traverse the same root
  manifest into the same canonical manifest basis and physical layout report.
- Adversarial denial: backend residue not represented by root/segment/extent
  manifests is rejected as a physical discovery source.
- Generation proof: free-space reuse changes generation evidence so old
  references cannot silently resolve.

**Engineering decisions**

- The root manifest is the physical discovery authority for S.1.
- Allocation classes describe physical placement and future layout pressure;
  they do not classify semantic truth.
- Manifest reports use foundational provenance so blind consumers can see the
  basis without producer-private state.

**Open questions**

- None.

### Phase 9: Build Minimal Offline Manifest Verifier

Phase 9 proves that persisted roots, manifests, page headers, extent headers,
slot directories, and free-space maps are sufficient for an independent reader
before the platform facade becomes the ordinary operation surface.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-certification`
- `worth-store-contracts`
- `worth-foundational` comparison, diagnostics, provenance, and support truth

**Relevant APIs**

- `OfflinePhysicalVerifier`
- `PhysicalLayoutReport`
- `ManifestTraversalReport`
- `MinimalManifestVerifierReport`
- `RuntimeLayoutObserver`
- `OfflineVerifierObserver`
- `worth_foundational::canonicalization_api::lower_lane::comparison`
- `worth_foundational::boundary_evidence_api::lower_lane::support`

**Required build shape**

- This phase implements a minimal offline verifier over persisted S.1 bytes
  immediately after manifests exist.
- This phase walks the root manifest, segment manifests, extent manifests,
  page/frame headers, slot directories, and free-space maps without live
  runtime construction.
- This phase produces a minimal physical layout report and manifest traversal
  report independent of backend-private object maps.
- This phase compares minimal verifier observations with runtime layout
  observations through the story harness parity trace.
- This phase emits typed verifier diagnostics for missing roots, ambiguous
  roots, invalid manifest membership, malformed headers, and unsupported
  reserved-field posture.

**Warnings**

- Do not wait for the platform facade to discover whether manifests are
  sufficient.
- Do not reuse live runtime caches or backend object maps in the verifier.
- Do not make verifier success depend on semantic payload decode.

**Test requirements**

- Adversarial parity: after root/manifest construction, the minimal offline
  verifier and runtime observer walk the same persisted bytes into the same
  physical layout basis.
- Adversarial denial: missing root, ambiguous root, backend residue, malformed
  manifest membership, and header rejection fixtures fail in the verifier
  before semantic decode.
- Independence proof: verifier smoke tests pass with live runtime construction
  disabled and with backend-private maps unavailable.

**Engineering decisions**

- Minimal verification moves earlier than full facade parity so manifest
  sufficiency is proven before operations depend on it.
- Full runtime/verifier comparison and shortcut rejection remain Phase 11.
- Verifier reports are physical evidence; support truth may describe them but
  may not mutate them.

**Open questions**

- None.

### Phase 10: Build The Platform-Grade Physical Backend Facade

Phase 10 defines the backend-facing shape later Roadmap 2 sequences will harden.
This phase closes the executable API boundary for S.1 physical operations.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-readiness`
- `worth-store-contracts`
- `worth-store-certification`
- storage-boundary interposer from the Roadmap 2 test harness

**Relevant APIs**

- `AcceptedHandoffReadiness`
- `PlatformGradeClaimWitness`
- `PhysicalFoundationBackend`
- `PlatformGradeBackend`
- foundational profiles and performance report APIs

**Required build shape**

- This phase implements physical append, read, scan, locate,
  root-manifest-publish, and reopen operations for the S.1 platform-grade
  backend candidate.
- This phase defines root publication as a clean-close and explicit
  test-interposer guarantee only: interrupted publication must produce a typed
  ambiguous-root denial, not crash-recovery repair or source-precedence
  guessing.
- This phase reserves root publication evidence that S.4 can consume for WAL,
  checkpoint, and torn-publication recovery without letting S.1 claim those
  behaviors.
- This phase keeps semantic commit-envelope authority above the physical
  facade.
- This phase routes physical operations through production-like storage
  boundaries so the test interposer can observe reads, writes, appends,
  flushes, renames, and opens.
- This phase rejects full-store heap materialization in platform-grade physical
  lanes.
- This phase rejects backend-private residue guessing as a locate or reopen
  strategy.
- This phase exposes platform-grade physical counters at the facade boundary
  and materializes them through foundational performance reports.
- This phase preserves bootstrap/compatibility paths behind explicit tier
  checks.

**Warnings**

- Do not make the platform facade a broad storage god object.
- Do not allow physical append or locate APIs to return semantic truth objects.
- Do not let compatibility backend success stand in for physical-facade proof.

**Test requirements**

- Adversarial parity: physical append, close, reopen, scan, and locate produce
  identical layout evidence through the facade and through independent manifest
  traversal.
- Adversarial denial: full-store materialization and backend-residue guessing
  are instrumented and rejected with exact zero/positive counters as specified
  by each hostile lane.
- Capability proof: bootstrap and compatibility backends remain useful but
  cannot mint `PlatformGradeClaimWitness`.

**Engineering decisions**

- The facade is the only platform-grade S.1 operation surface.
- Physical operations produce evidence and framed bytes, not domain truth.
- Storage-boundary interposition is part of the spec, not a test convenience.

**Open questions**

- None.

### Phase 11: Prove Runtime/Verifier Parity And Shortcut Rejection

Phase 11 proves that the runtime and offline verifier agree while shortcut
lanes fail at named boundaries. This phase closes the claim that S.1 evidence
can be produced without trusting live runtime state or backend-private residue.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-certification`
- `worth-store-contracts`
- `worth-foundational` diagnostics, comparison, provenance, and support truth

**Relevant APIs**

- S.1 verifier facade
- `PhysicalLayoutReport`
- `ManifestTraversalReport`
- `worth_foundational::canonicalization_api::lower_lane::comparison`
- `worth_foundational::boundary_evidence_api::lower_lane::support`
- foundational diagnostic materialization surfaces

**Required build shape**

- This phase implements an S.1 verifier that walks root manifests, segment
  manifests, extent manifests, page headers, frame headers, slot directories,
  and free-space maps without constructing the live store runtime.
- This phase allows the verifier to produce a physical layout report
  independently from the live backend facade.
- This phase compares verifier-discovered physical references with
  runtime-discovered physical references through foundational comparison and
  mismatch vocabulary.
- This phase emits disagreement reports rather than hiding verifier/runtime
  mismatch.
- This phase ensures verifier parsing does not call semantic artifact decoders
  except where a test explicitly asks for semantic parity after physical
  validation.
- This phase materializes verifier diagnostics and support truth through
  foundational diagnostic/support surfaces.

**Warnings**

- Do not reuse live runtime caches or backend object maps in the verifier.
- Do not compare verifier/runtime output as raw strings or unordered debug
  dumps.
- Do not decode semantic payloads before physical verification.

**Test requirements**

- Adversarial parity: offline verifier layout reports and runtime layout
  reports compare equal through foundational canonical comparison for admitted
  structures.
- Adversarial denial: a controlled verifier/runtime mismatch produces a typed
  foundational diagnostic/support report instead of being ignored or normalized
  away.
- Independence proof: verifier execution succeeds with live runtime
  construction disabled.

**Engineering decisions**

- The verifier is a separate read-only consumer of persisted bytes.
- Verifier/runtime comparison is a structured mismatch classification, not a
  boolean equality check.
- Support truth is descriptive evidence and may not mutate physical authority.

**Open questions**

- None.

### Phase 12: Prove Scale, Locality, And Complexity Boundaries

Phase 12 proves that S.1 physical operations scale by the intended locality
surface rather than by whole-store metadata. This phase closes the gap between
correct physical structures and scalable physical structures.

**Relevant subsystems**

- `worth-store-physical-format`
- `worth-store-contracts`
- `worth-store-certification`
- `worth-foundational` performance APIs

**Relevant APIs**

- `PhysicalOperationComplexityContract`
- `PhysicalOperationCounterSnapshot`
- `PhysicalLocalityClass`
- `PhysicalFreeSpaceSearchPolicy`
- `PhysicalFragmentationPressureReport`
- `PhysicalForegroundBoundednessReport`
- `worth_foundational::performance_api::lower_lane`

**Required build shape**

- This phase defines named complexity contracts for header decode, reference
  validation, slot locate, manifest lookup, root open, append placement,
  manifest traversal, and offline verifier walk.
- This phase defines locality classes for page-local, segment-local,
  extent-local, root-manifest, free-space-class, and full-scan operations.
- This phase builds fragmented free-space and many-segment fixtures that would
  force naive append or locate paths into broad metadata scans.
- This phase attaches exact operation counters to foundational counter-backed
  performance receipts.
- This phase marks each S.1 hot physical operation `Verified` only after
  counter evidence, algorithm review evidence, hostile fixture evidence, and
  scale/property evidence exist.

**Warnings**

- Do not treat counters alone as complexity proof.
- Do not hide a full scan behind an API named `locate`.
- Do not allow fragmented free space to turn foreground append into unbounded
  search.

**Test requirements**

- Adversarial scale parity: increasing unrelated segments, pages, extents, and
  manifests does not change locate-by-reference counters beyond the declared
  locality bound.
- Adversarial denial/defer: fragmented free-space fixtures either remain within
  the declared candidate-search bound or produce typed defer/denial and
  fragmentation pressure evidence.
- Performance proof: every named S.1 complexity contract has foundational
  counter-backed receipt evidence plus algorithm review and scale/property
  evidence.

**Engineering decisions**

- S.1 does not need S.2 resident-memory enforcement, but it must prove its
  metadata access patterns will not force S.2 into unbounded scans.
- Full scans are admitted only through explicit scan APIs with scan counters.
- Complexity status `Debt` is not admitted for S.1 platform-grade closeout
  operations.

**Open questions**

- None.

### Phase 13: Prove The Physical Page/Segment/Extent Substrate

Phase 13 closes S.1 with the named suite, exact counter receipts, foundational
evidence materialization, and hostile shortcut lanes. This phase closes the
sequence only if the evidence is independently interpretable by downstream
Roadmap 2 work.

**Relevant subsystems**

- `worth-store-certification`
- `worth-store-readiness`
- `worth-store-physical-format`
- `worth-store-contracts`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `Physical page/segment/extent substrate test`
- `AcceptedHandoffReadiness`
- foundational canonical, diagnostic, profile, performance, provenance, support,
  and boundary receipt APIs
- `worth-proof` proof progression and certification vocabulary consumed through
  the existing S.0 proof adoption map

**Required build shape**

- This phase runs the `Physical page/segment/extent substrate test`.
- This phase writes representative authoritative and derived artifact records.
- This phase closes and reopens the store from persisted bytes.
- This phase scans and locates records by physical identifiers.
- This phase asserts page/frame headers, generation counters, root manifests,
  segment manifests, extent manifests, allocation classes, and free-space maps
  are internally consistent.
- This phase rejects stale-generation physical references.
- This phase disables or instruments full-store heap materialization and proves
  platform-grade lanes do not use it.
- This phase runs legacy-backend forbidden-claim lanes.
- This phase runs offline verifier comparison lanes.
- This phase emits the required S.1 evidence bundle with foundational
  canonical basis, diagnostic rows, profile/materialization posture,
  provenance, completed-boundary receipts, support truth, and counter-backed
  performance receipts.

**Warnings**

- Do not close S.1 on "records survived restart" if the survival proof uses a
  heap map, serde-loaded full object graph, SQLite row lookup without S.1
  physical structures, or backend-private file residue.
- Do not emit completed-boundary receipts for planned, attempted, or failed
  work.
- Do not allow exact counters to remain local Store structs without
  foundational performance receipt participation.

**Test requirements**

- Adversarial parity: runtime layout report, offline verifier report, and
  emitted evidence bundle canonicalize to the same S.1 physical layout identity
  across independent construction paths.
- Adversarial denial: stale references, legacy platform claims, invalid
  headers, backend residue guessing, and whole-store materialization all fail
  with typed denials before semantic decode or platform-grade promotion.
- Certification proof: all exact physical counters are attached to
  foundational counter-backed performance receipts, and all S.1 diagnostic rows
  materialize through foundational diagnostic/profile policy.

**Engineering decisions**

- Closeout evidence is a boundary artifact, not a log bundle.
- Foundational evidence surfaces make S.1 interpretable to later Roadmap 2
  sequences without giving them Store internal topology access.
- S.1 closure consumes S.0 handoff readiness and proof-vocabulary adoption
  rather than reopening source/readiness authority locally.

**Open questions**

- None.

## Workflow Surface

S.1 is not done because bytes survived restart.

It is only done when the workflow operates over:

- accepted S.0 handoff readiness rather than raw source or claim inputs
- explicit binary physical format law rather than serializer behavior
- sealed physical references rather than raw offsets, filenames, row ids, or
  semantic artifact ids
- header decode witnesses rather than raw byte slices
- framed record placement rather than domain-object scans
- segment and extent manifests rather than backend-private object maps
- root manifest discovery rather than filesystem residue
- platform facade operations rather than private storage helpers
- offline verifier evidence rather than live-runtime self-reporting
- foundational canonical basis, diagnostics, profiles, provenance, support
  truth, completed-boundary receipts, and counter-backed performance receipts
  at every exported S.1 evidence boundary
- S.2 readiness that carries exactly the physical proofs S.2 needs and no
  weaker substitute

## Replay Closure

Replaying S.1 over unchanged admitted inputs and deterministic placement policy
must preserve:

- physical format declaration identity
- binary format digest
- physical reference identity for unchanged placements
- header decode witness identity
- framed record placement identity
- segment manifest identity
- extent manifest identity
- root manifest identity
- free-space map identity where allocation history is unchanged
- platform facade layout report identity
- offline verifier layout report identity
- foundational evidence bundle identity
- counter snapshot identity for fixed workloads
- typed denial outcomes for hostile fixtures

If a placement policy intentionally permits nondeterministic placement, the
policy must be explicit, the canonical evidence must identify the placement
basis actually chosen, and replay parity must compare the admitted evidence
rather than relying on hidden backend state.

## Diagnostics Closure

Denials must localize whether failure occurred at:

- S.0 handoff readiness consumption
- physical vocabulary admission
- foundational adoption/freshness
- binary format declaration
- unsupported page size
- unsupported alignment class
- reserved-field policy
- unsupported forward-compatibility posture
- physical id construction
- stale generation validation
- page header decode
- frame header decode
- unknown page/frame/manifest kind
- header length mismatch
- payload length mismatch
- checksum-slot reservation misuse
- page-local slot lookup
- deleted/moved/free/reserved slot access
- extent-backed record reference admission
- segment manifest membership
- extent manifest membership
- root manifest discovery
- free-space generation reuse
- free-space candidate search bound
- backend residue discovery attempt
- whole-store materialization attempt
- legacy backend platform claim
- platform facade boundary
- offline verifier independence
- verifier/runtime comparison
- foundational diagnostic/profile materialization
- foundational performance receipt construction
- S.2 readiness construction

No denial may collapse into a generic physical format error, generic validation
failure, raw string error, generic I/O error, or semantic decode failure.

## Determinism Closure

S.1 must make the following stable:

- binary field order
- byte order
- integer widths
- header magic and version rows
- page-size class ordering
- alignment-class ordering
- physical id encoding
- generation encoding
- slot directory ordering
- frame ordering within a page
- segment manifest row ordering
- extent manifest row ordering
- allocation-class ordering
- free-space map row ordering
- root manifest row ordering
- manifest traversal report ordering
- verifier observation ordering
- physical counter row ordering
- foundational evidence row ordering

Benign differences in filesystem enumeration order, backend open order,
manifest construction order, report generation order, and terminal projection
formatting must not change accepted native evidence identity.

## Complexity And Counter Closure

S.1 must expose counters for:

- binary format declarations admitted
- binary format declarations rejected
- page headers decoded
- frame headers decoded
- unknown kinds rejected
- header length mismatches
- payload length mismatches
- physical references validated
- stale references rejected
- pages read
- pages written
- frames decoded
- records appended
- records located
- slot directory entries inspected
- segment manifests read
- segment entries inspected
- extent manifests read
- extent entries inspected
- root manifests read
- root manifest entries inspected
- manifest index probes
- allocation-class probes
- free-space class probes
- free-space candidate scans
- fragmentation pressure signals
- backend-residue attempts rejected
- whole-store materialization attempts
- logical decodes after invalid header
- offline verifier bytes read
- verifier/runtime mismatches
- foundational canonical basis rows
- foundational diagnostic rows
- foundational performance counter rows
- foundational receipt/provenance attachments
- S.2 readiness denials

Complexity evidence types:

- counters prove observed work for one execution
- algorithm review proves the stated asymptotic bound from code structure
- hostile fixtures prove known pathological inputs deny, defer, or remain
  bounded
- scale tests prove observed growth is consistent with the bound across input
  sizes
- property tests prove replay, ordering, and equivalence invariants across
  generated input permutations

Counters alone do not verify complexity. A contract is `Verified` only when all
required evidence types for that contract are present.

Named complexity contracts:

- `binary_format_admission`: `O(format_fields + reserved_fields)`
- `header_decode`: `O(1)`
- `physical_reference_validation`: `O(1)`
- `page_slot_locate`: `O(1)`
- `page_local_record_scan`: `O(slots_on_page)`
- `segment_manifest_lookup`: `O(log segment_entries)` or `O(1)` when backed by
  an admitted manifest index
- `extent_manifest_lookup`: `O(log extent_entries)` or `O(1)` when backed by an
  admitted manifest index
- `root_manifest_open`: `O(root_entries)`
- `manifest_index_probe`: `O(1)` or the explicitly declared index strategy
  bound
- `append_record_placement`: `O(candidate_free_space_classes + admitted_candidate_scan_bound)`
- `manifest_traversal`: `O(root_entries + segment_entries + extent_entries)`
- `offline_verifier_walk`:
  `O(root_entries + segment_entries + extent_entries + pages_walked + extents_walked)`
- `foundational_evidence_materialization`:
  `O(evidence_rows + diagnostic_rows + counter_rows + attachment_rows)`
- `s2_readiness_validation`: `O(required_physical_artifacts + freshness_inputs)`

Every required closeout lane must mark its complexity contract `Verified` with
counter evidence, algorithm review evidence, hostile fixture evidence, and
scale/property evidence where applicable. `Debt` is not admitted for binary
format admission, header decode, physical reference validation, slot locate,
manifest lookup, root manifest open, platform facade operations, offline
verification, foundational evidence materialization, or S.2 readiness
validation.

## Allowed Debt

- No debt is allowed merely because the robust physical implementation is
  larger than a restart demo.
- No debt may weaken physical reference authority, binary format law, header
  witness construction, page-local addressing, root manifest discovery,
  stale-generation denial, no-whole-store-materialization proof, offline
  verifier independence, foundational performance receipt participation at
  exported/certified boundaries, or S.2 readiness construction.
- No debt is allowed that lets raw bytes satisfy a logical decoder without a
  header witness.
- No debt is allowed that lets raw offsets, filenames, row ids, vector indexes,
  or semantic artifact ids substitute for physical references.
- No debt is allowed that lets backend-private residue satisfy reopen, locate,
  scan, or verifier discovery.
- No debt is allowed that leaves binary format details to serde, host
  endianness, Rust layout, or platform defaults.
- No debt is allowed that lets full-store materialization satisfy platform
  lanes.
- No debt is allowed that exports S.1 performance or boundedness claims without
  foundational counter-backed receipts.
- No debt is allowed that emits completed-boundary receipts for planned,
  attempted, failed, or synthetic work.
- No debt is allowed that leaves foundational adoption as prose-only
  references or local lookalike types.

Debt may exist only when it cannot weaken those boundaries and is explicitly
owned by a later sequence or non-platform-grade lane. Admitted examples:

- richer diagnostic materialization beyond the required typed denial rows
- optional support truth attachments beyond required exported/certified
  evidence boundaries
- non-required allocation strategies beyond the S.1 candidate-search contract
- additional page-size classes beyond the admitted baseline class
- advanced moved-slot behavior beyond typed denial or certified single-hop
  forwarding
- advanced free-space policy beyond bounded search, typed defer, typed denial,
  or maintenance signal
- additional verifier report richness beyond the required physical layout,
  denial, counter, and parity traces

Deferred by design:

- buffer pool, resident-memory budgets, eviction, dirty-page tracking, and
  zero-copy hot-path leases remain S.2
- full physical checksums, scrub, quarantine, and corruption localization remain
  S.3
- WAL, checkpoint, pageLSN, and recovery physics remain S.4
- physical latch/read-plan/reclaim isolation remains S.5
- hardware I/O QoS remains S.6
- blob chunk tree substrate remains S.7
- artifact-family layout/index strategy remains S.8

Deferred later work may not weaken S.1's production-grade implementation bar.
S.1 may reserve fields for integrity or recovery only because those behaviors
belong to later sequences; it may not defer robust format law, physical
reference authority, header admission, manifest discovery, verifier
independence, bounded metadata access, or evidence materialization.

## Milestone Done When

- S.1 consumes `AcceptedHandoffReadiness` and rejects weaker S.0 artifacts.
- production public contracts use physical domain vocabulary rather than phase
  or milestone provenance names.
- `PhysicalScenarioQualityHarness` exists as Roadmap 2 certification
  infrastructure, with scenario definitions, lowered plans, execution,
  drivers, observers, proof oracles, story transcripts, quality lanes, counter
  traces, denial traces, verifier parity traces, and adversarial fixture
  reports.
- binary physical format law fixes byte order, widths, magic/version fields,
  page-size classes, alignment, reserved fields, and forward compatibility.
- format admission, rejection, preservation, migration-reserved posture, and
  downgrade refusal are distinct outcomes.
- physical references are sealed, generation-bearing, and stale-reuse safe.
- generation ownership is tied to the smallest reusable physical cell each
  reference addresses.
- header decode witnesses are required before payload access.
- page-local record placement works through slot directories or an equivalent
  page-local addressing table.
- ordinary locate does not deserialize all records on a page or in the store.
- segment and extent manifests own physical membership.
- root manifest discovery is sufficient for reopen and offline verification.
- root publication ambiguity is typed-denied in S.1 rather than guessed or
  repaired as crash recovery.
- free-space reuse carries generation evidence.
- platform facade operations are the only platform-grade S.1 operation surface.
- legacy heap/file/SQLite paths are fenced by capability tier.
- offline verifier walks persisted bytes independently from live runtime state.
- verifier/runtime mismatch is reported through typed diagnostics and
  foundational comparison/support surfaces.
- Foundational canonical basis, diagnostics, profiles, provenance/support
  truth, boundary receipts, and performance receipts are used at exported S.1
  evidence boundaries.
- exact counters and complexity contracts are `Verified`.
- S.2 entry consumes `S2PhysicalSubstrateReadiness` and rejects weaker
  substitutes.
- no phase closes through a placeholder, toy, best-effort, happy-path-only,
  scalar-only, serde-first, in-memory-map-first, backend-residue-first, or
  convention-enforced lane.

## Must Ship

- physical id and reference model for pages, segments, extents, frames, slots,
  roots, epochs, and generations
- Roadmap 2 `PhysicalScenarioQualityHarness` with scenario definitions,
  lowered scenario plans, execution, drivers, observers, proof oracles,
  evidence bundles, story-readable lane grammar, counter expectations, denial
  expectations, runtime/verifier parity expectations, adversarial fixture
  adapters, and stable proof transcripts
- initial `lanes/physical_substrate` family plus reserved lane-family topology
  for `buffer_pool`, `integrity`, `wal_recovery`, `physical_isolation`,
  `io_qos`, `blob_chunks`, `layout_indexes`, `formal_model_alignment`,
  `operations_repair`, `security_tenant`, and `physical_certification`
- binary physical format law for byte order, integer widths, magic/version
  fields, page-size classes, alignment, reserved fields, and forward
  compatibility
- explicit format admission, rejection, preservation, migration-reserved
  posture, and downgrade-refusal rules
- fixed-size page substrate for ordinary framed records
- variable-size extent substrate for large framed records and future chunk
  families
- physical page and frame headers with kind, version, length, checksum slot,
  generation, publication state, and reserved recovery fields
- header decode witnesses required before payload access
- slot directories or equivalent page-local record addressing
- framed record views independent of serde domain object materialization
- segment manifests, extent manifests, root manifests, allocation classes, and
  free-space maps
- manifest traversal plans and physical layout reports
- S.1 platform-grade physical backend facade for append, read, scan, locate,
  root publication, and reopen
- stale-generation detection and typed stale-reference failures
- generation ownership rule for slot/allocation, extent/allocation, and root
  publication reuse domains
- typed root-publication ambiguity denial for S.1 clean-close/interposer
  scenarios, with recovery repair deferred to S.4
- legacy backend capability-tier classification and forbidden platform-grade
  claim reports
- S.1 offline verifier for root, segment, extent, page, frame, slot, and
  free-space structures
- exact counters for all S.1 physical access and shortcut-rejection claims
- verified complexity contracts for binary format admission, header decode,
  reference validation, slot locate, manifest lookup, root open, append
  placement, manifest traversal, offline verifier walk, foundational evidence
  materialization, and S.2 readiness
- foundational canonical basis and digest participation for layout reports,
  manifest traversal reports, verifier observations, counter bundles, and
  evidence bundles
- foundational diagnostic rows, profile/materialization posture,
  provenance/support truth, completed-boundary receipts, and counter-backed
  performance receipts at S.1 evidence boundaries
- compile-fail coverage preventing local stand-ins for foundational canonical,
  diagnostic, profile, performance, provenance, or receipt surfaces where S.1
  has adopted the real foundational API
- `S2PhysicalSubstrateReadiness` as the only admitted S.2 entry handoff
- machine-checkable S.1 certification bundle

## Must Preserve

- canonical commit envelopes remain semantic authority
- physical pages, frames, extents, manifests, and free-space maps remain byte
  substrate, not domain truth
- `worth-relational` remains owner of semantic MVCC, identity, transaction
  meaning, and truth validation
- existing semantic certification value from earlier Store milestones remains
  valid even when old backends are reclassified
- backend variation may change physical placement and cost, not artifact
  meaning
- physical publication state does not imply semantic commit visibility
- S.1 does not claim S.2 memory bounds, S.3 corruption localization, S.4
  recovery physics, S.5 read isolation, S.6 QoS, S.7 blob scale, or S.8 access
  strategy discipline
- `worth-foundational` supplies shared boundary meaning and proof-composable
  materialization vocabulary; it does not own Store physical byte authority,
  page layout, frame decode law, manifest authority, or durability semantics
- adopted foundational surfaces must be consumed through public facade or
  grouped public lanes, not deep internal modules or local compatibility
  lookalikes
- S.1 does not use binary format law, physical references, or manifests to
  smuggle semantic truth authority into Store physical placement

## Acceptance Evidence

S.1 is complete only when the store satisfies the Roadmap 2 named suite:

- `Physical page/segment/extent substrate test`

Required machine-checkable outputs:

- `physical_story_transcript`
- `physical_scenario_definition`
- `physical_scenario_plan`
- `physical_scenario_execution_report`
- `physical_proof_oracle_verdict`
- `scenario_counter_trace`
- `scenario_denial_trace`
- `runtime_verifier_parity_trace`
- `shortcut_rejection_trace`
- `fixture_adversary_report`
- `physical_layout_report`
- `artifact_digest`
- `failure_digest`
- `counter_snapshot`
- `resource_envelope_report`
- `hardware_assumption_report`
- `foundational_canonical_basis_bundle`
- `foundational_diagnostic_bundle`
- `foundational_profile_materialization_plan`
- `foundational_boundary_evidence_bundle`
- `foundational_counter_backed_performance_receipt`

Minimum certification matrix rows are grouped by proof domain. Each group maps
to at least one `PhysicalScenarioQualityHarness` quality lane and should become
its own certification module or fixture family.

**Physical story harness**

- `physical_story_transcript_replay`
  replays a representative authority story and proves transcript, counter
  trace, denial trace, and runtime/verifier parity trace identity across
  independent observers.
- `physical_scenario_plan_lowering_replay`
  lowers the same `PhysicalScenarioDefinition` twice and proves required
  capabilities, driver requirements, observer requirements, expected boundary
  crossings, artifact policy, counter expectations, and denial expectations
  are identical before execution.
- `roadmap_lane_family_extension_without_harness_fork`
  registers stub oracle families and driver requirements for S.2 buffer pool,
  S.4 WAL recovery, S.7 blob chunks, and S.12 physical certification without
  changing scenario definition, planning, execution, observer, oracle,
  transcript, or evidence-bundle architecture.
- `test_support_cannot_own_certification_meaning`
  attempts to place proof-lane meaning, oracle verdicts, or evidence bundle
  authority in `worth-store-test-support` and proves the compile or admission
  boundary rejects it; reusable fixtures may provide bytes, clocks, faults,
  workloads, and counters only.

**Physical references and generations**

- `single_page_authority_reopen`
  persists an authoritative record on one page, closes, reopens from bytes, and
  locates it by physical reference.
- `stale_generation_reference_rejected`
  attempts to use a physical reference after generation reuse and proves typed
  stale-reference rejection before logical decode.
- `free_space_generation_reuse_detectable`
  reuses free space with a new generation and proves old references do not
  silently resolve.

**Binary format**

- `binary_format_golden_bytes_replay`
  proves S.1 golden bytes decode identically through explicit byte order,
  field width, magic/version, page-size, and alignment rules.
- `binary_format_serializer_accident_rejected`
  attempts host-endian, serde-order-dependent, Rust-layout-dependent, or
  reserved-field-ambiguous format construction and proves typed rejection.
- `binary_format_migration_posture_declared`
  proves format admission, rejection, preservation, migration-reserved posture,
  and downgrade refusal are distinct machine-checkable outcomes.

**Page, frame, slot, and extent records**

- `multi_segment_authority_scan`
  persists records across multiple segments and proves manifest traversal, page
  scan, and record locate counters match expected breadth.
- `derived_record_non_authority`
  stores a representative derived record and proves its physical placement does
  not alter semantic artifact authority.
- `extent_backed_large_record`
  stores a large framed record through an extent reference without whole-store
  domain materialization.
- `unknown_frame_kind_rejected_before_decode`
  attempts to decode an unknown frame kind and proves logical decode is skipped.
- `length_mismatch_rejected_before_payload`
  corrupts or misdeclares frame length inside an S.1 lane and proves payload
  view construction is rejected before semantic interpretation.
- `slot_directory_locate_bounded`
  locates records by page-local slot without deserializing all records on the
  page.
- `moved_slot_bounded_or_denied`
  proves moved slots are either typed denial or certified single-hop physical
  forwarding with exact bounded counters; multi-hop forwarding is rejected.

**Segment, extent, root, and free-space manifests**

- `root_manifest_discovery`
  opens the store through the root manifest and discovers segment manifests,
  extent manifests, allocation classes, and free-space maps.
- `root_publication_ambiguity_denied`
  injects ambiguous or interrupted root publication under the S.1 interposer
  and proves typed denial rather than S.4-style recovery guessing.
- `locate_by_reference_ignores_unrelated_store_growth`
  increases unrelated segments, pages, extents, manifests, and records while
  proving locate-by-reference counters remain inside the declared locality
  bound.
- `fragmented_free_space_append_bounded_or_denied`
  creates fragmented free-space pressure and proves append placement remains
  within the declared candidate-search bound or produces typed defer/denial
  with fragmentation evidence.

**Platform facade and legacy fences**

- `legacy_heap_backend_platform_claim_rejected`
  attempts to satisfy S.1 through a heap-shaped backend and proves the platform
  claim is rejected typed.
- `legacy_file_sqlite_backend_tier_fenced`
  proves existing file/SQLite paths are classified as bootstrap,
  compatibility, or semantic-certification unless they expose the S.1 physical
  substrate.
- `whole_store_materialization_forbidden`
  disables or instruments full-store heap materialization and proves all
  platform-grade S.1 lanes still pass with zero forbidden materialization
  attempts.
- `backend_residue_guessing_forbidden`
  attempts reopen or locate through backend-local residue not represented in
  the root/segment/extent manifests and proves rejection.

**Offline verifier and runtime parity**

- `minimal_offline_verifier_manifest_smoke`
  walks persisted root, segment, extent, page, slot, and free-space bytes
  before facade parity and proves manifests are sufficient for an independent
  reader.
- `offline_verifier_layout_match`
  walks persisted bytes through the S.1 verifier and matches runtime physical
  layout reports for admitted structures.
- `offline_verifier_runtime_disagreement_reported`
  injects a controlled verifier/runtime mismatch and proves disagreement is
  surfaced as evidence rather than hidden.

**Complexity and counters**

- `physical_counter_bundle_exact`
  proves page reads, page writes, frame decodes, header decodes, record locates,
  manifest lookups, allocations, stale-generation rejections, and forbidden
  shortcut counters match exact expected values.
- `physical_complexity_contracts_verified`
  proves every required S.1 complexity contract has counter, algorithm review,
  hostile fixture, and scale/property evidence.

**Foundational evidence export**

- `foundational_adoption_canonical_parity`
  independently constructs S.1 layout/evidence adoption rows and proves they
  canonicalize and digest through foundational APIs to the same identity.
- `foundational_local_stand_in_rejected`
  attempts to satisfy S.1 canonical, diagnostic, profile, provenance, receipt,
  or performance evidence with Store-local lookalike types and proves compile
  failure or typed denial.
- `foundational_performance_receipt_required`
  attempts to export exact physical counters as a platform-grade performance
  claim without foundational counter-backed performance receipt participation
  and proves rejection.
- `foundational_diagnostic_profile_controls_richness`
  varies diagnostic richness/profile materialization and proves authoritative
  physical outcomes are unchanged while optional diagnostic/support rows are
  included or elided only at named boundaries.

**S.2 handoff**

- `s2_entry_rejects_weaker_physical_substrate_inputs`
  proves S.2 entry consumes `S2PhysicalSubstrateReadiness` and rejects raw
  pages, private backend handles, physical projections, or local evidence
  bundles.

Milestone-specific proof obligations:

- certification lanes are story-readable, replayable, and mapped to named
  quality lanes with stable transcripts
- scenario definitions lower into inspectable plans before execution, and
  executors do not re-decide capability, driver, observer, artifact policy,
  locality, counter, denial, or oracle strategy
- S.2 through S.12 certification work can add lane families, drivers,
  observers, and oracles without forking the harness architecture
- reusable test support remains mechanical fixture infrastructure and cannot
  own proof-lane meaning, oracle verdicts, or evidence-bundle authority
- page, frame, segment, extent, root manifest, and free-space structures are
  self-describing enough for S.1 verification
- binary physical format law is explicit enough for cross-platform persisted
  byte replay
- format evolution posture distinguishes admission, rejection, preservation,
  migration-reserved behavior, and downgrade refusal
- record location uses physical references and page-local addressing, not
  heap-domain scans
- physical references carry generation evidence sufficient to reject stale
  reuse
- moved-slot behavior is typed denial or certified single-hop physical
  forwarding with bounded counters
- root publication ambiguity is denied in S.1 and reserved for S.4 recovery
  physics rather than guessed
- logical decoders cannot consume bytes without header decode witnesses
- legacy backends cannot satisfy platform-grade S.1 evidence accidentally
- offline verifier output is independent enough to disagree with live runtime
  discovery and report that disagreement
- physical layout reports identify authoritative and derived record placement
  without making placement semantic authority
- foundational canonical basis, diagnostics, profiles, provenance/support
  truth, boundary receipts, and performance receipts are used where S.1
  evidence crosses Store boundaries
- no foundational artifact can promote itself into Store physical byte
  authority without a Store-owned physical witness
- no S.2 entrypoint accepts weaker authority than `S2PhysicalSubstrateReadiness`
- no certification lane relies on logs, same-run self-comparison, or successful
  completion as proof
- `physical_logical_decode_after_invalid_header_count` remains zero
- `physical_whole_store_materialization_attempt_count` remains zero in required
  platform-grade lanes
- `physical_legacy_backend_platform_claim_rejection_count` matches hostile
  legacy-claim lanes exactly

S.1 is not closed by "records survived restart" tests if survival is proven
through a heap map, serde-loaded full object graph, SQLite row lookup without
the S.1 physical substrate, or backend-private file residue.

## Architectural Notes

- The smart abstraction is not a generic `StorageRecord`. The smart
  abstraction is a physical proof chain: root manifest to segment or extent
  manifest to page/frame header to slot or extent record to framed payload.
- S.1 should use names that teach the physical model directly. Prefer
  `PhysicalRootManifest`, `SegmentMembershipRecord`, and `StalePhysicalReference`
  over broad terms such as `StorageMetadata` or `RecordInfo`.
- Physical ids should be boring and strict. If they become convenient aliases
  for semantic artifact identity, the authority boundary has already leaked.
- A page header checksum slot is not a checksum program. S.1 may reserve and
  serialize integrity fields so S.3 can enforce them, but it must not claim
  corruption localization yet.
- The platform-grade physical facade should be narrow. Later sequences should
  harden it rather than every subsystem inventing a private byte path.
- Existing backends should not be shamed out of existence. They are useful for
  bootstrap, compatibility, and semantic certification, but those are different
  claims from platform-grade physical database posture.

## Sequencing Notes

S.1 belongs immediately after S.0 because the system must first stop
overclaiming existing persistence, then define the physical substrate that can
eventually earn stronger claims.

- S.1 consumes S.0 capability-tier vocabulary and deferred physical-guarantee
  mapping.
- S.1 consumes `AcceptedHandoffReadiness` as the exclusive S.0 truth boundary.
- S.1 must close before S.2 can honestly bound memory residency, because S.2's
  buffer pool needs page identities, frame boundaries, and manifests to lease.
- S.2 must consume `S2PhysicalSubstrateReadiness` rather than raw physical
  internals, legacy backend handles, or S.1 evidence projections.
- S.1 must close before S.3 can honestly localize corruption, because S.3 needs
  physical artifact boundaries to attack and quarantine.
- S.1 must close before S.4 can honestly define pageLSN, checkpoint, and WAL
  recovery physics, because recovery needs stable physical roots and references.
- S.1 may be designed while late 13.x documentation closes, but platform-grade
  implementation belongs inside the Roadmap 2 gate before Milestone 14.

## Required Self-Check

- Does the sequence solve a real structural problem or just package work
  cosmetically?
  Yes. It creates the physical byte-addressed substrate without which Store is
  still a semantic persistence harness rather than a database.
- Is the adversarial constraint precise and load-bearing?
  Yes. Every phase prevents full-store materialization, stale physical
  references, serializer-accident format law, backend residue guessing,
  unbounded metadata scans, or legacy backend overclaiming.
- Does the roadmap justify this sequence now?
  Yes. Roadmap 2 places S.1 immediately after S.0 because physical page,
  segment, extent, manifest, and reference authority must exist before memory,
  integrity, recovery, isolation, I/O, blob, layout, operations, security, and
  certification work can be honest.
- Does the sequence preserve crate authority boundaries?
  Yes. It gives Store physical byte authority while preserving canonical commit
  envelopes and `worth-relational` semantics as truth authority.
- Does the sequence use `worth-foundational` as a real shared-vocabulary
  dependency without outsourcing Store authority?
  Yes. It requires foundational canonical basis, digest derivation,
  diagnostics, profiles, provenance, boundary receipts, support truth, and
  performance receipts at evidence boundaries, while reserving page, frame,
  segment, extent, manifest, reference, generation, and byte-survival authority
  for Store-owned types.
- Does the sequence define proof obligations, not just implementation tasks?
  Yes. It names the required suite, evidence outputs, matrix rows, exact
  counters, shortcut rejections, offline verifier behavior, scenario
  definition/plan/execution/oracle/transcript progression, and zero-count
  assertions.
- Are the phases carrying most of the real design information?
  Yes. The implementation shape lives in the thirteen ordered phases, with the
  top-level sections acting as closure laws and summary gates.
- Is each phase centered on one conceptual detail or boundary?
  Yes. Capability fences, Roadmap 2 scenario harness grammar, binary format law,
  physical identity/generations, header witnesses, page slots, extent-backed
  records, manifests, early verifier smoke, platform facade, runtime/verifier
  parity, scale/locality proof, and final closeout evidence are split by
  authority and proof transition.
- Does each phase contain at least 2 adversarial tests by default?
  Yes. Each phase has parity/replay-style proof and denial/drift/localization
  proof, with more tests where the failure surface is broader.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names physical ids, scenario definitions, scenario plans,
  execution reports, observers, drivers, proof oracles, story transcripts,
  binary format law, generation ownership, headers, slot directories, extent
  records, manifests, allocation classes, backend tiers, verifier paths,
  counters, complexity contracts, facades, directory skeletons, phases, and
  certification lanes.
- Does each phase say how to build the boundary, not just what the boundary is?
  Yes. Each phase states required build shape, source surfaces, prohibited
  substitutes, hostile tests, and the enforcement boundary made real by that
  phase.
- Does the sequence belong in this roadmap sequence, or is it out of order?
  Yes. S.1 follows S.0 claim reclassification and must precede memory,
  integrity, recovery, isolation, I/O, blob, layout, operations, security, and
  certification work.
