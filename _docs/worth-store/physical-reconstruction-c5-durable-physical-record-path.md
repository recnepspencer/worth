# Worth Store Physical Reconstruction C.5: Durable Physical Record Path

## Goal

Make ordinary physical records live in real page, segment, extent, manifest,
and bootstrap files reached through the C.4 media owner. A successful
publication must survive loss of the writer process, and a fresh process must
discover and read it from the Store root without caller-supplied heap state.

## Why This Milestone Exists

C.4 made filesystem effects real but intentionally exposed no record store.
C.5 is the first usable physical-data slice: it replaces the in-memory format
model as the production path, establishes the on-disk authority that C.6 will
buffer and C.7 will protect with WAL ordering, and prevents every later
milestone from integrating against a second pretend database.

## Governing Summaries

- `MENTALITY.md` protects foundations that survive the adversarial case. Its
  strongest constraint here is to build bounded fresh-process discovery and
  publication before adding convenient record features on a heap model.
- `arch_laws.md` protects compiler-visible authority, phase progression, and
  autonomous ownership. Its strongest constraint here is that only a real
  media-backed open may produce the record-serving runtime, while reads,
  writes, publication, and observation remain separately borrowable.
- `composition_laws.md` protects named semantic responsibilities. Its strongest
  constraint here is that codecs, allocation, publication, lookup, scanning,
  diagnostics, and test orchestration cannot collapse into one record-store
  function or a generic helper module.
- `domain_structure_laws.md` protects truth-source and boundary topology. Its
  strongest constraint here is that physical format owns byte meaning, the
  backend owns OS effects, Store owns orchestration and current physical truth,
  and the offline observer owns no mutation or runtime admission authority.
- `perf_laws.md` protects bounded work and cost-visible APIs. Its strongest
  constraint here is that reopen, locate, append, and scan breadth scale with
  the touched artifact path rather than total store size, with structural
  counters capable of falsifying every claim.
- `dx_laws.md` protects organized truth at the call site. Its strongest
  constraint here is that disk boundaries, bounded reads, scan lifecycle,
  publication uncertainty, and operation cost remain explicit without forcing
  callers to speak files, offsets, manifest grammar, or backend handles.
- `worth-proof` protects proof-bearing progression law. C.5 uses its checked
  outcome topology only for the consuming media-owned-to-serving transition
  and external-locator readmission; it does not use Proof as a runtime, storage
  engine, operation-outcome vocabulary, receipt vocabulary, or substitute for
  concrete Store authority.
- `worth-foundational` protects shared boundary meaning. C.5 lowers already
  authoritative Store facts into canonical comparison, counter-backed
  performance artifacts only at explicit certification methods; it never
  replaces Store-owned record ids, manifests, publication results, or
  runtime-local hot state. C.5 adds no generic diagnostic, support, lineage, or
  boundary-evidence vocabulary.
- `physical-foundation-reconstruction-roadmap.md` protects one real Part I
  platform. Its strongest constraint here is that page, segment, extent, and
  manifest records must survive a real process boundary before buffer, WAL,
  recovery, integrity, isolation, layout, or blob work receives closure credit.

## Adversarial Constraint

> Starting from an absent Store root, publish a deterministic record world
> whose aggregate persisted page-frame bytes are at least 64 times the maximum admitted C.5
> transfer width, whose root and segment manifests span multiple blocks, and
> which contains inline, page-spanning, segment-crossing, and extent-backed
> placements. Kill the writer after a completed publication without normal
> runtime close. A fresh process, given only the root and admitted
> configuration, must open with work independent of total record count, locate
> records in adversarial order, and scan through a bounded cursor without ever
> constructing a complete layout or store-sized `Vec`. At every interrupted
> publication seam, current truth must remain the prior published root, become
> the fully published new root, or return a typed indeterminate outcome; it may
> never be assembled by directory guessing, stale-manifest fallback, replay
> state, or surviving heap objects.

The reference complexity contracts are:

- open/reopen: `O(bootstrap_blocks + root_header_blocks)` work and memory,
  independent of total page, segment, extent, and record count
- locate: `O(manifest_depth + payload_chunks)` reads with no unrelated page
  scan
- append batch: `O(input_bytes + touched_pages + manifest_path_updates)` work,
  with publication barriers amortized across the batch
- scan: `O(records_returned + frames_traversed)` work and no materialization
  wider than the caller's batch plus one admitted frame/chunk

## Product Decision Lock

1. A C.5 record is one immutable, semantically uninterpreted byte sequence.
   Duplicate payloads are legal. Zero-length payloads are legal and receive a
   real frame. The format declares one exact maximum record length. Store does
   not infer keys, kinds, schemas, transactions, dedupe, or domain identity from
   payload bytes.
2. C.5 is append-only at its public record surface. Update, delete, reclaim,
   compaction, semantic transactions, MVCC visibility, and Query integration
   are not smuggled into this milestone.
3. One batch is the primary write cardinality. A one-record convenience may
   lower into that path, but no public scalar loop may create one publication
   and barrier sequence per record by accident.
4. Stable record identity and current placement are different Store types.
   `PhysicalRecordId` is a Store-scoped opaque allocation-epoch plus batch-
   ordinal identity, stable for the life of the Store record and never reused.
   Candidate ids remain private until publication; abandoned candidates do not
   recycle their allocation epoch.
   `CurrentPhysicalRecordPlacement` is a private root-scoped page/slot or extent
   location with generation. `ExternalPhysicalRecordLocator` is a weak
   serialized Store-id plus record-id form that must be readmitted. Physical
   placement, path, offset, generation, digest, and display identity are never
   accepted as stable record identity.
5. Data pages use a slotted-page grammar whose slot-directory growth does not
   rebase existing payload offsets. Within one batch, records pack into candidate
   pages. Across batches, adding to a partially filled current page produces a
   copy-on-write successor page generation and atomically remaps the affected
   stable record ids; no currently reachable page byte is mutated in place.
6. `PublishedRecordBatch` means every record in the batch is reachable from one
   fully published successor root under the admitted C.4 profile. Fresh reopen
   may observe the complete prior root or the complete successor root after an
   interrupted cutover, never a visible subset of the batch. It is not a
   semantic transaction commit, WAL receipt, checkpoint receipt, or C.7
   physical acknowledgment.
7. Publication has one locked progression:
   `CandidateDataWritten -> DataSynchronized -> ManifestsSynchronized ->
   CatalogCandidateSynchronized -> CatalogReplaced -> NamespaceSynchronized ->
   PublishedRecordBatch`. Each step consumes the exact prior typed state. The
   catalog is one staged, file-synchronized, atomic-replace, parent-directory-
   synchronized artifact; alternative dual-slot or list-and-elect protocols do
   not satisfy C.5.
8. `IndeterminateRecordPublication` describes caller knowledge after a possible
   catalog cutover. It is not safely retryable, is not permission to delete
   candidate artifacts, and is not a recovery handle. `close()` releases owned
   resources only; it never publishes, synchronizes, or strengthens unfinished
   work.
9. Initialization and opening are distinct consuming transitions.
   `initialize_record_store` requires a proven-absent record family and refuses
   any conflicting current or staged artifact. `open_record_store` requires an
   authoritative existing catalog and never creates one. No production
   `create_or_open` path exists.
10. The bootstrap catalog is the only ordinary current-root discovery source.
    Directory listing may diagnose residue; it may not elect current truth,
    iterate candidate roots until one decodes, or silently fall back to the
    newest-looking manifest.
11. Configuration has three explicit axes. `PhysicalRecordFormatDeclaration`
    contains Store-wide compatibility law such as format version, page size,
   byte order, field widths, root-catalog protocol, and mandatory CRC32C frame
   integrity. `PhysicalRecordPlacementPolicy` contains evolvable append choices
    such as target segment width, extent threshold, page-fill policy, and
    manifest node capacity; each produced artifact records the policy facts
    required to interpret its layout. `PhysicalRecordAccessPolicy` contains
    per-open transfer, scratch, scan, append-record-count, and aggregate append-
    byte limits. Placement/access changes do not masquerade as format
    incompatibility. An ordinary append preserves the current manifest fanout;
    changing fanout requires an explicitly named reconstructive append and may
    not hide a whole-tree rebuild.
12. Reopen takes only the Store root already owned by the runtime, one format
    expectation, and one independently admitted access policy. Placement policy
    is required only for new appends. `PersistedPhysicalLayout`, replay
    artifacts, decoded records, expected manifests, and writer-returned catalogs
    are not inputs.
13. Physical format owns deterministic bytes and structural validation. The
    backend owns handles, transfers, barriers, and publication effects. Store
    owns record identity, allocation and placement decisions, current-placement
    mapping, publication orchestration, current-root authority, and the public
    record facade.
14. Every C.5 authority artifact carries mandatory CRC32C coverage over its
    header (with the checksum field excluded), identity/generation fields, and
    complete framed payload. Bootstrap, manifest, page, and extent bytes with a
    mismatch are rejected before semantic use. C.9 later owns complete coverage analysis,
    corruption localization, scrub, quarantine, disagreement, and repair law.
14a. CRC32C is accidental-corruption and torn-write detection, not message
    authentication. C.3-C.5 do not provide confidentiality, authorization, or
    integrity against an administrator, compromised service account, kernel,
    storage appliance, or other actor already authorized to rewrite Store
    bytes. Such mutation must be detected where structurally visible and fail
    closed, but cryptographic authenticity and hostile-local-writer isolation
    are not claims of this milestone.
15. C.5 has one in-process mutation authority. Appends require exclusive mutable
    access and cannot overlap live locate or scan sessions. Immutable readers may
    coexist only while borrowing the same current-root basis. C.5 does not expose
    a concurrent serving process beside the writer; the offline observer is not
    a serving reader.
16. C.5 uses bounded direct media access. C.6 replaces only frame loading and
    candidate-frame residency. C.6 may own pins and dirty candidate state, but
    Store retains current-root publication authority and consumes a
    `CandidateFrameSet` through the same publication progression. The residency
    session owns each encoded frame for the duration of Store's physical write
    callback; it is not a notification hook over a write that bypassed the seam.
17. The C.5 transient-I/O budget bounds operation scratch, transfer width,
    cursor materialization, and bootstrap work. It is not a buffer-pool
    residency or total-process-memory guarantee.
18. Recognized versions are supported only when executable decode and
    compatibility behavior exists. Unknown, future, malformed, and merely named
    versions fail with typed outcomes; the runtime does not guess or migrate.
    Physical scan order is canonical encoded `PhysicalRecordId` order in the
    current root's routing tree; it carries no semantic or Query ordering claim.
19. Worth Proof is used only for media-owned-to-serving admission and explicit
    external-locator readmission. Append, locate, scan, partial-effect, and
    indeterminate outcomes are Store-owned types. Worth Foundational is used
    only for explicit canonical topology comparison and counter-backed
    performance receipts after Store truth exists.
20. Existing physical-format mechanisms are substrate, not presumptive design.
    Whole-manifest vectors, linear membership lookup, offset-rebasing page
    append, offline-codec production open, candidate-root iteration, heap/replay
    paths, and generic evidence machinery are replaced or quarantined from the
    production path.
21. C.5 adds three expensive end-to-end scenario families, not a Cartesian
    product of page sizes, record sizes, fault seams, versions, and process
    roles. Focused owner tests cover grammar; the three courtrooms prove joined
    behavior.

## Capability Boundary And Explicit Non-Goals

C.5 closes:

- real creation and fresh-process opening of the physical record store
- real batch append, direct locate, bounded scan, and extent streaming
- stable physical record ids, root-scoped placement, locator readmission, and
  generation validation
- bounded bootstrap and manifest traversal
- direct copy-on-write publication through C.4 media
- exact current-root, artifact, range, allocation, and operation observations

C.5 does not close:

- bounded shared residency, frame pinning, eviction, dirty writeback, or cache
  coherence (`C.6`)
- WAL-before-data ordering, group commit, checkpointing, or durable transaction
  acknowledgment (`C.7`)
- recovery from arbitrary mid-operation process death (`C.8`)
- complete corruption localization and quarantine (`C.9`)
- concurrent stable reads during rewrite/reclaim or scheduled maintenance
  (`C.10`)
- B-tree/LSM adoption, blob storage, semantic records, MVCC, Query, or branch
  behavior (`C.11` and Part II)

## Authoritative Artifact Graph

C.5 has one authority graph. Every production open, locate, append, and scan
must follow it in this direction:

1. the admitted namespace identity establishes the Store identity
2. `bootstrap.catalog` selects exactly one current root manifest and format
   identity
3. the current root manifest tree names the complete reachable artifact
   closure and routes each `PhysicalRecordId` to its current placement
4. segment manifests establish page membership and page generation
5. extent manifests establish extent membership and extent generation
6. the free-space manifest establishes allocatable ranges for the successor
   root
7. admitted page or extent frames establish the immutable record bytes

A directory listing establishes residue only. It cannot select a Store, root,
manifest, page, extent, generation, or record. Production open uses a bounded
production catalog decoder and follows only the selected root. The offline
observer may share immutable field widths, tags, and golden bytes with the
physical-format crate, but it owns a separately implemented decoder, traversal,
and current-root decision path. Production must replace the current
`OfflineManifestCodec` bootstrap dependency, whole-manifest vectors, candidate
root loops, and linear membership lookup; wrapping those paths does not satisfy
this milestone.

## Logical Vertical Slices

- **Slice A — establish real current truth:** Phases 1 through 3 freeze the
  format and configuration, create bounded bootstrap authority, then complete
  one inline record through append, publication, process loss, reopen, and
  locate. `ServingPhysicalRuntime` does not exist before this slice is real.
- **Slice B — make placement scale:** Phases 4 through 7 add batch page packing,
  segment rollover, extent streaming, stable generations, and free-space truth
  without changing the publication authority.
- **Slice C — make access and failure honest:** Phases 8 through 11 make
  manifests scale, expose bounded scans, preserve partial/indeterminate effects,
  and close version/readmission denials.
- **Slice D — make it usable and governable:** Phases 12 through 14 freeze
  lifecycle and C.6 seams, lower narrow canonical/performance evidence, and
  close the
  milestone through three production-path courtrooms plus mechanical
  anti-substitution gates.

## Intentional DX Target

The common path must read as physical intent while keeping I/O and bounded-work
boundaries visible:

```rust
let media_owned = admitted_runtime
    .try_admit_filesystem_media(FilesystemMediaAdmission::production(
        FilesystemAccessPosture::ReadWrite,
    ))
    .into_success()?;

let format = PhysicalRecordFormatDeclaration::builder()
    .format_version(PhysicalRecordFormatVersion::V1)
    .page_size(PageSize::KiB16)
    .root_catalog_protocol(RootCatalogProtocol::AtomicReplaceV1)
    .mandatory_integrity(FrameIntegrity::Crc32cV1)
    .finish()?;

let placement = PhysicalRecordPlacementPolicy::builder()
    .target_segment_pages(256)
    .extent_threshold(RecordBytes::new(8 * 1024)?)
    .page_fill(PageFillPolicy::ReuseByCopyOnWrite)
    .manifest_node_capacity(ManifestNodeCapacity::new(64)?)
    .finish()?;

let access = PhysicalRecordAccessPolicy::builder()
    .maximum_transfer_width(TransferBytes::new(64 * 1024)?)
    .operation_scratch_ceiling(TransferBytes::new(128 * 1024)?)
    .scan_batch_limit(RecordCount::new(256)?)
    .finish()?;

let mut serving = media_owned
    .initialize_record_store(PhysicalRecordInitialization::new(
        format,
        placement.clone(),
        access,
    ))
    .into_success()?;

let published = serving.records_mut().append_batch(
    RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()])?,
    placement,
)?;

let mut record = serving.records().open(
    published.record_id(0)?,
    RecordReadLimits::new(TransferBytes::new(64 * 1024)?)?,
)?;
while let Some(chunk) = record.read_next(&mut caller_scratch)? {
    consume(chunk);
}

let mut scan = serving.records().scan(
    RecordScanRequest::from_start().with_batch_limit(RecordCount::new(128)?),
)?;
while let Some(batch) = scan.read_next_into(&mut scan_scratch)? {
    consume_batch(batch);
}

let closed = serving.close();

// A later process opens; it never creates.
let serving = media_owned
    .open_record_store(PhysicalRecordOpen::new(
        PhysicalRecordFormatExpectation::version(PhysicalRecordFormatVersion::V1),
        access,
    ))
    .into_success()?;
```

The exact spelling may be refined before implementation only to improve
semantic precision. The following properties are locked:

- media admission and record-serving admission are separate consuming steps
- initialization and open are distinct consuming operations; initialization
  requires proven absence and open never creates
- format declaration, placement policy, and access policy are separate objects
  with Store-wide, append-time, and per-open compatibility rules respectively
- batch append is primary and crosses an obvious disk-publication boundary
- access admission bounds both record count and aggregate declared payload
  bytes before any append producer is consumed
- every append explicitly admits its placement policy; open does not pretend
  that policy is Store-wide compatibility law
- append returns stable `PhysicalRecordId` values, never physical placements;
  weak external locators require explicit Store readmission
- record and scan reads use explicit limits and caller-controlled bounded
  buffers or equivalent bounded sessions
- partial and indeterminate publication are typed outcomes, never generic I/O
  strings
- `close` releases owned resources and authority; it is not a flush, commit, or
  additional durability boundary
- no common-path method mentions file paths, byte offsets, manifests, replay,
  `PersistedPhysicalLayout`, Foundational artifacts, or backend handles

## Phase Plan

### Phase 1: Persisted Meaning And Configuration Admission

Freeze the byte-level and configuration decisions that every later C.5
artifact consumes. This phase produces validated Store-owned format, placement,
and access meaning;
it does not create `ServingPhysicalRuntime` or write a production artifact.

**Relevant subsystems**

- `worth-store-physical-format` binary declaration, framing, header, page,
  extent, manifest, bootstrap, record-id, placement, and generation vocabulary
- Store-owned format declaration, placement policy, access policy, and their
  distinct admission denials
- deterministic codec golden fixtures and independent decode fixtures

**Relevant APIs**

- `PhysicalRecordFormatDeclaration` and its builder
- `PhysicalRecordPlacementPolicy` and its builder
- `PhysicalRecordAccessPolicy` and its builder
- sealed `AdmittedPhysicalRecordFormat`, `AdmittedRecordPlacementPolicy`, and
  `AdmittedRecordAccessPolicy`
- existing `PhysicalFormatDeclaration`, `PhysicalPageSizeClass`,
  `PhysicalFormatVersion`, `PhysicalFrameHeader`, and framing types
- explicit format, placement-policy, and access-policy admission denials

**Warnings**

- Do not make format choices from runtime data after the first artifact exists.
  Page size, field widths, byte order, header grammar, alignment, reserved
  fields, manifest block width, and identity grammar are persisted contracts.
- Do not advertise old-version readability because a version enum exists.
  Executable decode behavior and a golden corpus must exist first.
- Do not introduce a generic config map, positional booleans, raw byte counts,
  or independent format declarations in Store and physical-format. Do not bind
  operational transfer, scratch, or scan policy into persisted format identity.
- Segment targets, extent thresholds, page-fill choices, and manifest node
  capacity are evolvable placement policy, not Store-wide format compatibility.
  Each artifact records the policy facts needed to interpret its own layout.
- `StoreNamespaceVersion`, `PhysicalRecordFormatVersion`, per-artifact schema
  versions, placement-policy identity, and root generation are distinct types.
  A generic `Version` or one number reused across these meanings is forbidden.
- C.9 owns comprehensive localization, scrub, quarantine, and repair. C.5 still
  requires checksum-protected frames for bootstrap, manifest, page, and extent
  authority and rejects torn checksums, malformed lengths, illegal frame kinds,
  impossible slot bounds, reserved fields, and unsupported versions before use.

**Test requirements**

- `current_format_golden_corpus_is_bit_exact`: encode every C.5 artifact family
  and compare exact bytes and independent decode meaning, including boundary
  field widths and empty/full slot directories.
- `format_placement_and_access_rules_do_not_collapse`:
  reject impossible page/manifest/extent/segment geometry and independently
  reject scan, scratch, and transfer limits that cannot bound an operation,
  while media counters remain exactly zero. Reopen accepts placement-policy and
  access-policy drift that does not change persisted format meaning, and rejects
  actual format drift.
- `unknown_and_future_versions_do_not_reach_payload_decode`: demonstrate typed
  localization at the format declaration rather than a later generic record
  failure.

**Engineering decisions**

- Preserve one canonical Store-wide format declaration and derive every
  artifact codec from it. Store configuration admits that declaration; it does
  not restate it. Append separately admits placement policy; each open admits
  access policy.
- Use semantic unit types for pages, bytes, record counts, segment pages, and
  transfer limits. Persist raw integers only inside the codec boundary.
- Store accepted format identity in bootstrap authority so reopen can
  compare caller expectation before opening record files. Per-open access
  policy and append placement policy are not part of format compatibility.
- Make checksum coverage mandatory in version 1 for every authority-bearing
  frame. Reserve C.9 for richer diagnosis and repair, not first detection.

**Open questions**

- None. Any newly discovered persisted field must be added to the format
  declaration and golden corpus before code depending on it lands.

### Phase 2: Bounded Bootstrap And Current-Root Authority

Build distinct real initialize/open substrates that can establish an empty
Store and find
its current root with constant bounded work. The public serving phase remains
unavailable until Phase 3 completes an actual record path.

**Relevant subsystems**

- physical-format bootstrap catalog, root locator, and format identity codecs
- C.4 namespace roles, staged publication, file/directory barriers, and media
  counters
- Store-owned bootstrap admission and current-root selection

**Relevant APIs**

- private `PhysicalRecordBootstrapOwner`
- `BootstrapCatalogReadLimits`
- `CurrentRootCatalogEntry` and `CurrentRootCatalogGeneration`
- `PhysicalRecordBootstrapOutcome`
- `PhysicalRecordInitialization` and `PhysicalRecordOpen`
- distinct `RecordStoreInitializationOutcome` and `RecordStoreOpenOutcome`
- C.4 `QualifiedFilesystemMedia` operations through the Store-owned media field

**Warnings**

- The bootstrap catalog is authority; directory enumeration is observation.
  “Choose the newest manifest” is forbidden even when it appears to recover a
  test fixture.
- Creating an empty store is a real physical publication. A post-effect failure
  cannot return a reusable pristine `MediaOwnedPhysicalRuntime` as if nothing
  happened.
- Initialization requires proven absence of the entire record family and
  rejects any conflicting catalog, manifest, data, extent, free-space, or
  staging residue. Open requires one authoritative catalog and never creates.
  No production `create_or_open` method or fallback branch may exist.
- The bootstrap reader may read fixed headers and the selected catalog slot or
  equivalent bounded object. It may not deserialize a complete manifest tree.
- The catalog protocol is locked: write one staged candidate, synchronize the
  file, atomically replace `bootstrap.catalog`, then synchronize its parent
  namespace. Dual-slot and list-and-elect alternatives are forbidden.

**Test requirements**

- `empty_bootstrap_create_and_reopen_converge`: create an absent root through
  C.4 media, discard the bootstrap owner, reopen from root/configuration only,
  and prove identical Store/format/current-root identity with exact bounded
  reads and no record artifacts.
- `initialize_and_open_never_substitute_for_each_other`: open against an absent
  record family fails without writes; initialize against any existing or
  ambiguous residue fails without electing or deleting it; successful
  initialization publishes exactly one empty current root.
- `namespace_residue_cannot_elect_current_truth`: place valid-looking older,
  newer, staged, duplicate, and foreign manifests around a valid catalog; open
  must use only the catalog-selected root and report residue separately.
- `incomplete_bootstrap_publication_never_returns_reusable_authority`: inject
  failure after each bootstrap effect and prove either pre-effect denial with
  returned media authority or terminal inspection-required posture.

**Engineering decisions**

- Bootstrap state contains the Store identity, smallest sufficient current-root
  locator, format identity, and frame checksum. Segment, page, extent, and
  free-space collections remain outside it.
- Bootstrap reads use exact offset/length operations and explicit limits.
- Bootstrap creation and replacement reuse C.4 staged publication and barrier
  primitives rather than a second file helper.
- Keep the bootstrap owner private to the record-serving admission path so no
  downstream caller can replace current-root selection.
- Production bootstrap decoding is a bounded production decoder; it must not
  call `OfflineManifestCodec`, loop over candidate manifests, or share the
  offline observer's traversal/current-selection implementation.

**Open questions**

- None.

### Phase 3: First Real Inline Record And Serving Admission

Complete the first end-to-end physical record slice: consume media ownership,
append and frame one inline record, publish its page and manifest chain, return
one stable record id, lose the writer process, reopen, and locate the exact
payload. Only this phase introduces the public record-serving runtime.

**Relevant subsystems**

- Store physical-runtime record-serving admission and lifecycle
- physical-format page records, slot directory, references, manifests, and
  bootstrap codecs
- C.4 positioned transfers, synchronization, staged publication, atomic
  replacement, and directory synchronization
- `worth-proof` checked progression topology

**Relevant APIs**

- `MediaOwnedPhysicalRuntime::initialize_record_store(...)`
- `MediaOwnedPhysicalRuntime::open_record_store(...)`
- `PhysicalRecordInitialization` and `PhysicalRecordOpen`
- `RecordServingAdmissionOutcome = worth_proof::ProofOutcome<...>`
- move-only `ServingPhysicalRuntime`
- `ServingPhysicalRuntime::records()` and `records_mut()`
- `PhysicalRecordStore::append_batch(...)`
- `PhysicalRecordStore::append_batch_reconstructing_manifest_capacity(...)`
- `PhysicalRecordStore::open(...)`
- `PublishedRecordBatch` and `PhysicalRecordId`

**Warnings**

- Do not expose `ServingPhysicalRuntime` from a constructor, test authority,
  memory implementation, replay artifact, supplied layout, or format witness.
- Do not mint a public `PhysicalRecordId` until the selected root contains its
  manifest membership and the current-root publication sequence completes.
- A successful result requires the exact progression
  `CandidateDataWritten -> DataSynchronized -> ManifestsSynchronized ->
  CatalogCandidateSynchronized -> CatalogReplaced -> NamespaceSynchronized ->
  PublishedRecordBatch`. No step may be skipped, reordered, or reconstructed
  from counters after the fact.
- This direct publication is not WAL-protected transaction commit. Name the
  result `Published`, not `Committed`, `Acknowledged`, or `Recovered`.
- The record reader may own one bounded frame/chunk scratch region. It may not
  return or retain the complete Store layout.
- A batch is visibility-atomic: a fresh open observes every record through one
  successor root or observes none through the prior root. There is no published
  per-record prefix.

**Test requirements**

- `one_inline_record_survives_writer_loss`: publish through the ordinary
  facade, report completion to the parent, terminate the writer without
  `close`, reopen in a fresh process from root/configuration only, and locate
  the exact record through an `ExternalPhysicalRecordLocator` containing only
  Store identity plus the returned stable record id.
- `publication_barrier_omission_is_observable`: inject the omitted-barrier
  fault at the production media boundary; the production trace and fresh
  observer must fail the exact publication assertion.
- `non_authority_values_cannot_admit_record_serving`: extend the existing
  cache-sharing compiler suite to reject `PersistedPhysicalLayout`, replay,
  format witness, digest, Foundational artifact, raw backend handle, and copied
  identity as serving admission inputs.

**Engineering decisions**

- The Store runtime owns a concrete private record-store subsystem containing
  the media owner, admitted format and policies, current-root state, allocation frontier,
  and counters. Public read and write facades borrow only the fields they need.
- The media-owned-to-serving progression preserves success, denied, deferred,
  stale, rebind-required, and failed/inspection-required categories. Concrete
  Store authority seals minting; public generic `AuthorityMarker` bounds are
  forbidden.
- The first publication already uses the final C.5 copy-on-write ordering. Later
  phases widen artifact topology; they do not replace a shortcut publication.
- Close and abort consume `ServingPhysicalRuntime`, close every owned handle,
  and return the existing terminal lifecycle families without weakening C.4
  mutation ownership. They never publish, synchronize, or strengthen a prior
  append outcome.

**Open questions**

- None.

### Phase 4: Batch Append And Page Packing

Make batch cardinality real by packing multiple framed records into slotted
pages and publishing one coherent root change, while preserving per-record
identity and exact placement outcomes.

**Relevant subsystems**

- Store batch admission, placement planning, allocation frontier, and
  publication orchestration
- physical-format record framing, slot directory, page header, and placement
  witnesses
- C.4 range writes and barrier counters

**Relevant APIs**

- `RecordAppendBatch` and `RecordAppendBatchBuilder`
- private `AdmittedRecordAppendBatch` and `LoweredRecordPlacementPlan`
- `PublishedRecordBatch::record_ids()` and private placement observations
- `RecordPlacementClass::{InlinePage, ExtentBacked}`

**Warnings**

- Batch input order is semantic and must map deterministically to publication
  order and returned record ids. Hash-map iteration is not an ordering basis.
- Reject invalid record sizes, total batch breadth, and impossible placement
  before allocating page images or touching media.
- Do not externally loop a scalar append path. The planner derives touched
  pages once, the executor writes them once, and manifest publication occurs
  once for the batch.
- Slot offsets, padding, and free space are format facts. Record payload code
  must not recompute them independently.
- The page grammar grows its slot directory and payload region toward each
  other. Adding a slot never rebases the payload offsets of existing slots.
  Rebuilding a page-wide `Vec` and shifting old offsets is forbidden.
- Reusing a partially filled published page across batches is copy-on-write:
  write a successor page generation and atomically remap every affected stable
  record id in the successor root. Never mutate a currently reachable page.

**Test requirements**

- `batch_packing_matches_independent_page_oracle`: for a deterministic mixed
  batch, compare exact slot order, offsets, padding, remaining space,
  record ids, payload bytes, and counters against an independent placement
  oracle and golden page decoder.
- `cross_batch_page_reuse_preserves_identity_without_rebasing`: append into a
  partial page across two publications; prove old payload offsets remain stable,
  the page generation/placement changes by copy-on-write, all old
  `PhysicalRecordId` values remain stable, and fresh reopen reads both batches
  only through the complete successor root.
- `invalid_batch_is_rejected_before_construction`: duplicate caller ids where
  applicable, empty batches, over-limit records, aggregate overflow, and
  impossible page geometry must produce typed denials with zero allocation and
  media-effect counters.
- `scalar_convenience_has_batch_parity`: if a one-record convenience ships,
  prove it lowers into the same planner and produces byte/counter parity with a
  one-element batch.

**Engineering decisions**

- The public batch object owns or borrows payloads under one explicit lifetime;
  it cannot hide per-record store-sized copies.
- Placement planning is pure and inspectable. Execution accepts only the
  lowered plan and never re-decides inline versus extent strategy.
- One batch publishes one immutable current-root successor even when it touches
  several pages.
- Return one ordered record-id vector with an explicit one-to-one input index
  mapping rather than a generic result bag.
- A page-sized candidate buffer is permitted and counted. Store-sized candidate
  materialization and hidden whole-page copies beyond the declared planner and
  encoder passes are forbidden.

**Open questions**

- None.

### Phase 5: Segment Rollover And Segmented Lookup

Extend the working page path across deterministic segment boundaries without
changing record identity or current-root publication.

**Relevant subsystems**

- Store segment allocation frontier and rollover planner
- physical-format segment header, page membership, and segment-manifest codecs
- record locate path and manifest membership validation

**Relevant APIs**

- `PhysicalSegmentId`, `PhysicalPageId`, and `SegmentPageManifestEntry`
- private `SegmentAllocationPlan` and `PublishedSegmentMembership`
- `RecordReadObservation` with touched-segment and touched-page counters

**Warnings**

- Segment rollover is determined by the append's admitted placement policy, not by
  ambient file size or an implementation-specific buffer threshold.
- Segment identity, page identity, and namespace name are distinct facts. A
  filename is never accepted as authority for the ids encoded inside a frame.
- Locate must use the record-id routing and manifest path. It may not scan segment
  files until matching bytes happen to appear.
- An incomplete successor segment is unpublished residue, not an empty current
  segment and not evidence that the prior root is damaged.

**Test requirements**

- `rollover_world_reopens_with_exact_segment_membership`: force at least three
  segment transitions with unevenly packed pages, reopen fresh, locate records
  in reverse and seeded-random order, and assert exact segment/page/slot
  placements plus zero unrelated segment scans.
- `segment_filename_and_header_disagreement_is_denied`: present a valid-looking
  file under the wrong segment role/name and prove typed identity mismatch
  before record decode.
- `incomplete_successor_segment_cannot_enter_current_root`: interrupt the first
  write and the publication of a new segment; fresh open must retain prior
  truth or return the exact indeterminate posture, never list-and-adopt it.

**Engineering decisions**

- Segment files contain a bounded sequence of fixed-size page frames. Segment
  manifests contain membership and generation facts, not complete page bytes.
- Rollover planning happens once per admitted batch and produces a deterministic
  ordered set of touched segments.
- Segment creation and segment-manifest publication reuse C.4 namespace roles
  and durability primitives; the backend gains no segment semantics.
- Locate reads only the manifest path and page frame named by the current
  root's `PhysicalRecordId` routing entry.
- Segment target width is not format compatibility. Each segment artifact
  records its actual page capacity and generation; a later append may use a new
  placement policy without making old segments unreadable.

**Open questions**

- None.

### Phase 6: Extent-Backed Records And Bounded Streaming

Add the large-record placement path so records wider than inline page capacity
are persisted and read in bounded chunks rather than whole-record or
whole-store materialization.

**Relevant subsystems**

- Store placement lowering and extent allocation
- physical-format extent frame, extent manifest, and record-membership codecs
- bounded media range writer and reader sessions

**Relevant APIs**

- `ExtentBackedRecordPlacement`, `ExtentMembership`, and `PhysicalExtentId`
- `RecordWriteSource` or equivalent bounded producer abstraction
- `RecordReadSession::read_next(&mut [u8])`
- typed `RecordStreamFailure` retaining completed byte range

**Warnings**

- An extent is a physical record placement, not a native blob/chunk tree. C.11
  still owns content addressing, dedupe, range trees, and blob lifecycle.
- The append path must not call `collect`, `read_to_end`, `to_vec`, or an
  equivalent whole-record helper for extent payloads.
- Caller-provided producers must declare exact length before effects so layout,
  limits, and framing can be admitted. Length drift during streaming is a typed
  partial-effect failure.
- Extent data must synchronize before any manifest that makes it reachable.
- Extent threshold is append placement policy, not Store-wide format identity.
  Zero-length records are legal real page frames and never become extents merely
  because a policy threshold is zero.

**Test requirements**

- `extent_record_larger_than_transfer_width_streams_and_reopens`: publish one
  deterministic extent-backed record at least 17 transfer widths long, kill
  the writer after publication, reopen fresh, stream it through differently
  sized caller buffers, and prove byte parity with peak scratch and transfer
  width bounded independently of record length.
- `truncated_or_overlong_extent_source_preserves_completed_range`: make a
  producer stop early and then exceed its declared length; both outcomes must
  retain exact attempted/completed ranges, leave no reachable record, and never
  mint a public record id.
- `whole_extent_materialization_mutant_fails_allocation_slope`: replace bounded
  streaming with one whole-payload allocation; the scale courtroom must fail
  at the operation-allocation boundary rather than only by elapsed time.

**Engineering decisions**

- Extent writes and reads operate on caller or runtime scratch bounded by the
  admitted maximum transfer width.
- Extent manifest membership records exact logical length, physical ranges,
  generation, framing identity, mandatory checksum coverage, and the placement
  policy fact needed for bounded read validation.
- A publication may mix inline and extent placements while retaining one
  ordered batch identity and one current-root successor.
- Operation results expose bytes requested, bytes completed, transfer count,
  peak transfer width, explicit copy count, and copied bytes without
  materializing a rich support report.

**Open questions**

- None.

### Phase 7: Record Identity Readmission, Placement Generations, And Free-Space Truth

Make stable record identity honest across process and trust boundaries, keep
current placement private, reject stale placement generations before payload
use, and persist the allocation facts later
reclamation can extend without pretending C.5 already performs reclaim.

**Relevant subsystems**

- Store stable record identity, current placement routing, exported locator,
  and readmission
- physical-format page, segment, slot, extent, and root generation cells
- physical-format free-space inventory and allocation classes
- `worth-proof` external-locator weakening/readmission progression only

**Relevant APIs**

- stable `PhysicalRecordId`
- private, root-scoped `CurrentPhysicalRecordPlacement`
- weaker `ExternalPhysicalRecordLocator`
- `PhysicalRecordStore::readmit_locator(...)`
- `StalePhysicalRecordPlacement` and `PhysicalLocatorReadmissionDenial`
- `FreeSpaceManifestEntry` and `AllocationClassManifestEntry`

**Warnings**

- Serialization preserves representation, not current authority. A deserialized
  locator containing Store identity plus record id must be readmitted against
  current Store identity and record membership. Current placement and its
  generation are resolved only from the selected root.
- Digest, display label, namespace path, record ordinal, raw page/extent ids,
  and serialized placements are not substitutes for stable record identity.
- C.5 records never-before-allocated and currently unallocated ranges. It does
  not expose delete/reclaim/reuse operations merely to make a generation test
  convenient.
- Reuse authority later added by C.10/C.11 must consume and increment these
  generation cells rather than creating a parallel reuse counter.

**Test requirements**

- `exported_locator_readmits_to_same_record_in_fresh_process`: export a locator
  from a published record id, cross a real process boundary, readmit it through
  the reopened Store, and prove record-id/payload parity without accepting the
  serialized form directly as current authority.
- `forged_store_root_and_generation_locators_are_rejected`: alter each authority
  field independently inside one table-driven owner test and prove exact denial
  localization with zero payload reads. This is one focused family, not one
  integration binary per field.
- `free_space_inventory_rebuilds_allocation_frontier`: compare persisted
  allocation entries to an independent walk and prove overlap, duplicate range,
  impossible class, and reused-without-generation-change inputs fail closed.
- `copy_on_write_placement_change_preserves_record_id`: publish an append that
  replaces a partial page, prove the old and new roots resolve the same stable
  record id to different placement generations, and prove a caller can neither
  export nor submit either private placement as identity.

**Engineering decisions**

- Stable record ids are sealed Store-owned values. Export produces a weak
  Store-id-plus-record-id locator suitable for storage or process transfer;
  current placement never crosses the facade.
- Readmission is point-local and manifest-driven; it never requires scanning
  all records or reconstructing a complete placement index.
- Generation validation occurs before reading a payload frame and contributes
  exact checked/rejected counters.
- Free-space facts are authoritative physical metadata. Convenience summaries
  are derived and destroyable.
- Worth Proof is used only for the external-locator weakening/readmission
  boundary in this phase. Stale placement and locator denials remain concrete
  Store enums.

**Open questions**

- None.

### Phase 8: Scalable Manifest Fanout And Root Discovery

Replace any whole-manifest vector or flat linear lookup with immutable,
fixed-fanout manifest blocks so roots, segments, extents, and free-space state
can grow without changing bootstrap or record APIs.

**Relevant subsystems**

- physical-format root, segment, extent, and free-space manifest block codecs
- Store record-id routing planner and bounded manifest scratch
- offline manifest walker with an authority path distinct from runtime open

**Relevant APIs**

- `PhysicalRootManifest` header and manifest-root reference
- private `ManifestSearchPath` and `ManifestPublicationPlan`
- `ManifestReadLimits`
- `ManifestDiscoveryCounterSnapshot`

**Warnings**

- A manifest-specific fixed-fanout search structure is not C.11's general
  B-tree/LSM feature. Its only responsibility is current physical membership.
- The root header and search path may be bounded; the total manifest need not
  fit in memory. A type named `PhysicalRootManifest` must not secretly own a
  `Vec` of every entry on the production path.
- Canonical order is persisted format law and must be proven during block
  construction, not rediscovered by sorting every reopen.
- Offline traversal may walk the full manifest for verification, but its API
  must look reconstructive and expose its different cost.
- The current root tree is the complete reachable closure: it owns record-id to
  current-placement routing plus segment, extent, and free-space manifest roots.
  Segment manifests own page membership/generation; extent manifests own extent
  membership/generation; the free-space manifest owns allocatable ranges.
- The existing `PhysicalRootManifest` entry vectors, linear lookup, production
  `OfflineManifestCodec` import, and candidate-manifest bootstrap loop must be
  replaced. Adapters around them are not an implementation of this phase.

**Test requirements**

- `multi_block_manifest_lookup_has_logarithmic_path_and_exact_parity`: force at
  least three levels or the maximum supported nontrivial fanout shape, compare
  runtime lookup to an independent full offline walk, and assert exact blocks,
  comparisons, bytes, and zero unrelated payload reads.
- `unsorted_duplicate_and_cross_root_entries_fail_before_membership`: corrupt
  focused format-owner inputs and prove canonical-order, uniqueness, and
  Store/root-scope denials remain distinct.
- `whole_manifest_materialization_mutant_fails_scale_slope`: substitute a
  complete manifest `Vec` on reopen or locate; the bounded-scale courtroom must
  fail exact block/allocation predicates as total manifest width grows.

**Engineering decisions**

- Use immutable manifest blocks with explicit fanout, generation, parent/root
  binding, covered key range, and next/child references as required by the
  selected structure.
- Bootstrap names one current root header. The root header names only bounded
  manifest roots and format identity, never all entries.
- Runtime point lookup and offline full walk share stable byte declarations but
  not decoders, traversal policy, current-root selection, caches, or result
  authority. Both must independently accept the same golden corpus.
- Manifest publication remains copy-on-write and joins the same Phase 3
  current-root cutover.

**Open questions**

- None.

### Phase 9: Bounded Physical Scan Sessions

Expose deterministic physical-order scanning through an explicit bounded
session whose root basis, cursor lifetime, batch breadth, and resume posture
are visible to the caller.

**Relevant subsystems**

- Store record-reader facade and scan-session lifecycle
- manifest range traversal and page/extent record iteration
- cursor export/readmission and scan counters

**Relevant APIs**

- `RecordScanRequest`
- `PhysicalRecordScanSession<'runtime>`
- `RecordScanBatch<'scratch>` or an equivalent caller-buffer view
- weaker `ExternalRecordScanCursor`
- `RecordScanOutcome` and `RecordScanCounterSnapshot`

**Warnings**

- C.5 scan order is canonical encoded `PhysicalRecordId` order in the current
  root's routing tree and yields stable ids. It may coincide with insertion
  order for a particular allocation policy, but it is not Query, key, semantic,
  branch, or MVCC order.
- A scan session borrows one current-root observation basis. The type system
  must prevent the same runtime from mutating that basis while the session is
  live. One in-process mutation authority owns append exclusively; immutable
  readers may coexist only while they share that same current root. A writer
  cannot overlap locate or scan, and no second serving process may coexist with
  the writer. Cross-root stable-read coordination remains C.10.
- The cursor carries a position, not authority. Exported cursors require Store
  identity, root-generation, format, and record-id readmission on resume.
- `collect_all`, an iterator that silently allocates records, or a convenience
  returning `Vec<Record>` for an unbounded scan is forbidden.

**Test requirements**

- `scan_batch_widths_converge_to_one_physical_sequence`: scan the same
  multi-segment/extent world with small, uneven, and maximum admitted batches;
  concatenate only in the independent oracle and prove identical ordered
  record ids/payloads with exact per-batch frame and byte accounting.
- `stale_foreign_and_out_of_range_cursors_fail_before_payload_read`: alter root,
  Store, format, generation, and position basis in one table-driven owner test;
  each must localize distinctly with zero payload bytes read.
- `live_scan_borrow_prevents_mutation`: extend the consolidated compiler suite
  so a live scan session cannot coexist with a mutable record facade from the
  same runtime.

**Engineering decisions**

- Scans are pull-based sessions. Caller buffers and batch limits provide
  backpressure; no producer thread, channel, or unbounded queue is introduced.
- Each batch result includes start/end cursor, records returned, frames
  traversed, bytes read, manifest blocks, and completion posture.
- End-of-scan is a typed completed posture distinct from an empty intermediate
  batch or failed read.
- Point locate and scan share decoding law but retain different access and cost
  surfaces.
- The separately linked offline observer is reconstructive tooling, not a
  serving reader, and grants no exception to the single-serving-process rule.

**Open questions**

- None.

### Phase 10: Publication Failure Topology And Residue Honesty

Drive every C.5 artifact family through the real C.4 fault boundary and freeze
what the runtime may claim before effect, after a known unpublished effect, and
after an uncertain current-root cutover.

**Relevant subsystems**

- Store publication orchestrator and operation outcomes
- C.4 storage-boundary fault schedule, partial-effect context, and counters
- staged/current namespace roles and residue observation
- fresh-process open and offline classification

**Relevant APIs**

- `RecordPublicationOutcome`
- `PublishedRecordBatch`
- `UnpublishedRecordBatchFailure`
- `UnpublishedRecordEffectFate::{DeniedBeforeEffect, EffectPossible}`
- `IndeterminateRecordPublication`
- `RecordPublicationRecoveryLocator` as descriptive C.8 handoff only
- private `CandidateDataWritten`, `DataSynchronized`,
  `ManifestsSynchronized`, `CatalogCandidateSynchronized`, `CatalogReplaced`,
  and `NamespaceSynchronized` states

**Warnings**

- A failed API call is not proof that no bytes landed. Preserve attempted and
  completed ranges, published artifact steps, barrier state, and possible-root
  posture.
- Every unpublished failure carries one structural effect-fate classification.
  `DeniedBeforeEffect` preserves reusable mutation authority; `EffectPossible`
  seals it as inspection-required. Cause labels are diagnostic and must never
  substitute for this authority decision.
- The first effect-possible unpublished failure ends mutation for that serving
  runtime. Residue observations may accumulate categories for diagnosis, but
  C.5 does not permit a second publication whose artifacts could make the
  abandoned frontier ambiguous.
- Candidate-frame validation proves the publication transition that was
  actually in progress. It is not a synthetic publication stage and may not
  erase data-write, manifest-sync, or catalog-sync progress.
- Do not simulate rollback by deleting candidate files after an indeterminate
  namespace result. C.5 may remove only residue whose non-reachability is
  proven from the current catalog; otherwise it preserves and reports it.
- An indeterminate result is not a C.8 recovery handle and cannot retry itself.
  It is not safely retryable or safely deletable. It is sufficient structured
  context for later recovery to inspect.
- Fault tests must interpose at the same media methods production uses. Private
  manifest mutation and test-owned file writes do not prove this boundary.

**Test requirements**

- `publication_cutover_never_invents_current_truth`: one table-driven courtroom
  injects short data write, extent truncation, manifest write failure, data
  sync failure, post-manifest/pre-catalog death, and post-catalog/pre-directory-
  sync death. A fresh runtime and separate offline observer must classify each
  world as exact prior root, exact new root, or the declared indeterminate
  posture with matching completed-effect context. A root exposing only a
  subset of the batch is always invalid.
- `known_unpublished_residue_is_not_current_or_silently_deleted`: prove staged
  pages, extents, and manifests remain excluded from locate/scan and appear in
  typed residue observation until an authorized later cleanup policy acts.
- `premature_identity_subset_and_success_mutants_fail_causally`: separately mint
  a record id before root publication, expose a batch subset, and return
  `Published` before the final required barrier; the courtroom must fail
  identity-authority, batch-atomicity, and durability
  predicates at different causal boundaries.
- `close_never_strengthens_publication_outcome`: after each non-success posture,
  close the runtime and prove no additional write, sync, catalog replacement,
  or publication claim occurs.

**Engineering decisions**

- Operation failure topology is Store-owned and family-specific. Append,
  locate, scan, partial-effect, and indeterminate outcomes are concrete Store
  enums, not Worth Proof vocabulary.
- Each publication step consumes a typed prior step and produces the exact next
  step; later steps cannot be called with weaker or reconstructed values.
- Diagnostics are derived from the operation outcome and media counters. They
  cannot influence whether a root becomes current.
- Keep all production and faulted execution in one parameterized publication
  path. Certification supplies a schedule, not an alternate executor.

**Open questions**

- None.

### Phase 11: Format Compatibility And Trust-Boundary Readmission

Close serving admission against persisted format drift and make every
cross-process artifact regain authority from the reopened Store rather than
from representation.

**Relevant subsystems**

- Store record-serving admission outcome
- physical-format version and forward-compatibility declarations
- Store identity, format identity, backend-profile basis, locator/cursor
  readmission
- `worth-proof` stale, rebind-required, denied, deferred, and failure topology

**Relevant APIs**

- `PhysicalRecordInitialization` and `PhysicalRecordOpen`
- `RecordServingAdmissionOutcome`
- `UnsupportedPhysicalRecordFormat`
- `PhysicalRecordFormatMismatch`
- `RecordServingAdmissionStale` and `RecordServingAdmissionRebindRequired`

**Warnings**

- “Readable header” does not mean “supported Store.” Serving admission requires
  an executable compatible path for every current artifact family.
- A caller-declared format expectation may narrow acceptance; it may not
  override persisted format or authorize a migration. Open does not accept an
  append placement policy.
- Bridged Store identity, exported locator, digest, canonical basis, and prior
  qualification report remain descriptive until current Store owners readmit
  them.
- Migration belongs in an explicit later program. C.5 must not mutate old
  versions in place during open.

**Test requirements**

- `current_version_reopens_and_every_unimplemented_version_fails_typed`: use
  exact golden artifacts for the current version plus older, future, malformed,
  and recognized-but-unimplemented versions; only executable support opens a
  serving runtime.
- `catalog_selected_stale_manifest_fails_at_generation_admission`: a catalog
  that names an otherwise valid root manifest under the wrong generation must
  fail at root-generation admission; the runtime may not fall back to another
  manifest found by listing.
- `format_policy_and_profile_drift_preserve_outcome_category`: page size,
  byte order, field widths, integrity law, root-catalog protocol, format
  identity, Store identity, and backend-basis drift resolve to the intended
  denied, stale, rebind-required, or failed category without filesystem
  guessing. Changing manifest node capacity, extent threshold, segment target,
  page-fill policy, or a valid per-open limit must not masquerade as format
  drift; old artifacts remain readable from their recorded facts.
- `bridged_boundary_values_open_no_door`: compiler and runtime checks prove
  exported locators, bridged identity, Foundational canonical artifacts,
  digests, reports, and copied admission outcomes cannot construct a serving
  runtime or bypass current readmission.

**Engineering decisions**

- Record-serving admission reuses the existing `worth-proof` progression and
  `ProofOutcome` family rather than creating C.5-local witness/outcome kernels.
- Concrete Store authorities mint successful admission and stable record ids.
  Generic marker bounds are not exposed on governed public surfaces.
- Unsupported and incompatible versions fail before manifest/payload traversal
  beyond the bounded declaration needed to classify them.
- Read-only compatibility is not advertised unless its runtime phase, allowed
  methods, and fresh-process tests are complete; an ordinary serving runtime
  is never secretly weakened.

**Open questions**

- None.

### Phase 12: Serving Lifecycle, Autonomous Owners, And C.6 Handoff

Freeze the production topology and public lifecycle so C.6 can replace direct
frame access with bounded residency without rewriting format, publication,
record identity, placement, or record semantics.

**Relevant subsystems**

- Store physical-runtime composition root and record-serving owner
- read facade, write facade, publication owner, bootstrap/root observation,
  managed-resource lifecycle, and shutdown
- frame-load, candidate-frame-set, and candidate-publication contracts for C.6

**Relevant APIs**

- move-only `ServingPhysicalRuntime`
- borrowed `PhysicalRecordReader<'runtime>` and
  `PhysicalRecordWriter<'runtime>` facades
- private `FrameLoadPort`, `CandidateFrameSet`, and
  `CandidateFramePublicationPort`
- `PhysicalRecordObserver`
- phase-typed `PhysicalMediaObserver<MediaOwnedObservationPhase>` and
  `PhysicalMediaObserver<RecordServingObservationPhase>`
- `ServingShutdownOutcome<ClosedRuntime | AbortedRuntime>`

**Warnings**

- Do not expose a generic subsystem registry or raw media accessor to make C.6
  integration easier. C.6 must replace narrow frame responsibilities, not reach
  through Store into backend internals.
- Read handles may clone only immutable observation state. They may not clone
  root-selection, allocation, publication, media-mutation, or lifecycle
  authority.
- Adding the record owner must break every incomplete construction, close,
  abort, observer, and later transition site at compile time.
- Shared lifecycle state enters generation-advancing `Terminating` before any
  record owner, media owner, or lease begins teardown. Only completed teardown
  may advance it to `Closed` or `Aborted`.
- Serving-frame reads, including admission-time catalog/root/free-space loads,
  require `FrameLoadPort`. Bootstrap/control mutation uses a distinct
  capability with no serving read fallback.
- C.5 operation scratch is scoped to one call or session and must be released
  deterministically. It is not a hidden cache or embryonic buffer pool.
- C.6 may own residency, pins, eviction, and dirty candidate state. It may not
  publish the current root: Store retains the exact Phase 10 publication
  progression and accepts only a complete `CandidateFrameSet`.

**Test requirements**

- `record_owner_propagates_through_every_lifecycle_boundary`: construction,
  observation, normal close, abort, panic/drop, child death, and re-admission
  reconcile exact owner/handle/resource counters and leave no live authority.
- `reader_writer_and_frame_authority_cannot_escape`: consolidated compiler
  fixtures prove readers cannot mutate, scan batches cannot outlive scratch,
  record ids and locators cannot mint writers, and direct frame/media seams are inaccessible
  outside their owner.
- `direct_frame_port_replacement_preserves_record_contract`: a narrow owner
  conformance test drives the same load/candidate/publication contract through
  the direct implementation and a counting wrapper, proving the future C.6
  seam owns residency but cannot select or publish current truth.
- `serving_concurrency_contract_is_enforced`: compile and process tests prove
  one in-process writer is exclusive with locate/scan, immutable readers share
  only one current root, and a second serving process cannot open beside the
  writer under the C.4 mutation lease.

**Engineering decisions**

- The rough production target is intentionally shallow and responsibility
  named:

  ```text
  worth-store/src/physical_runtime/
    record_serving/
      mod.rs                 # public facade aggregation only
      access_policy.rs       # per-open transfer, scratch, and scan limits
      identity.rs            # stable ids and weak external locators
      admission/             # bootstrap/open transition and typed denials
      planning/              # placement, reuse, allocation, and free-space plans
      publication/           # append, typed progression, outcomes, and residue
      access/                # locate, scan, readmission, and routing readers
      residency/             # capability-split artifact I/O and frame ports
      lifecycle/             # serving owner, observation, shutdown, termination
      evidence/              # explicit canonical and performance lowering
  ```

  The groups are semantic edit destinations, not file-count buckets. Direct
  artifact-tree access remains private to `residency/`; initialization and
  publication receive write/control capabilities, while serving receives a
  mandatory mediated-read capability.

- Existing physical-format homes remain the target rather than a parallel C.5
  tree:

  ```text
  worth-store-physical-format/src/
    binary_format/  bootstrap/  page_record/  extent_record/
    manifest/       record_identity/  placement/  record_framing/
    offline_walk/
  ```

  The existing location-shaped `reference/` module is replaced by the separate
  identity and placement homes; it is not retained as a compatibility facade.

- Add a child directory only when one of these files develops multiple
  independently testable responsibilities. Do not create `helpers`, `common`,
  `utils`, `c5`, `phases`, or a second `physical_record` universe.
- Reader and writer facades borrow disjoint owner fields wherever possible.
  A method may not borrow the whole runtime merely for convenience.
- The C.6 handoff names `load frame under limits`, `hold candidate frame set`,
  and `submit candidate frames for Store publication`. It does not expose
  backend file handles, pre-decoded whole layouts, or root publication.

**Open questions**

- None.

### Phase 13: Canonical Topology And Counter-Backed Performance Evidence

Lower completed Store-owned facts into Worth Foundational only for canonical
topology comparison and counter-backed performance receipts. C.5 does not add
generic diagnostic, support, lineage, or boundary-evidence vocabularies.

**Relevant subsystems**

- Store root/format/record-id/publication observations
- physical-format canonical basis preparation
- Store operation counters and C.4 media counters
- Worth Foundational canonical comparison and counter-backed performance lanes

**Relevant APIs**

- Store-owned `PhysicalRecordPublicationSummary`
- Store-owned `PhysicalRecordAccessSummary`
- `lower_record_publication_canonical_basis(...)`
- `lower_record_operation_performance_receipt(declared_contract, observed_summary)`
- explicit certification methods on an observer or completed outcome

**Warnings**

- A Foundational canonical or performance artifact describes a fact Store
  already owns. It never becomes the source of current root, record membership,
  publication completion, or runtime admission.
- Use Foundational counter-backed receipts only after real execution rows
  reconcile with declared counter specs. A descriptive performance claim is
  not executed proof.
- Declared counter values and observed execution rows are separate inputs.
  Deriving both from one post-execution counter array is a self-confirming
  receipt and is forbidden.
- Do not materialize canonical exports or performance receipts on append,
  locate, or scan; they exist only behind an explicit certification call.
- Do not lower runtime-local hot types into generic aspect state. C.5 stores
  opaque physical record payloads and owns no Foundational aspect semantics.

**Test requirements**

- `runtime_and_offline_topology_have_canonical_parity`: independently lower the
  reopened Store observation and full offline walk into the same declared
  canonical basis; equivalent worlds compare equal and a one-field topology
  divergence localizes without digest-only comparison.
- `counter_receipt_rejects_missing_duplicate_and_mismatched_rows`: every
  append, locate, scan, manifest, allocation, record identity, transfer, and barrier
  spec must have exactly one executed row; weakening or widening a row fails
  receipt construction.
- `foundational_outputs_cannot_promote_or_execute`: compile/runtime denials prove
  canonical topology and performance receipts cannot open a Store, read payload
  bytes, mint record ids, or publish a root.

**Engineering decisions**

- Keep the strongest Store type until the explicit boundary-lowering call.
- Canonical meaning includes Store identity, format identity, current root
  generation, ordered artifact membership, stable record ids, and
  publication basis; it excludes process identity, timestamps, paths that are
  presentation-only, and nondeterministic iteration order.
- Performance receipts declare included and excluded work. Hot record access
  and offline full traversal are different claims.
- Foundational lowering remains an explicit certification surface if
  ordinary product execution has no cross-crate need for it.

**Open questions**

- None.

### Phase 14: Direct Production Verification And Closure

Close C.5 with three difficult production-path scenarios and focused
dependency/compiler gates that make heap, replay, broad
materialization, and alternate writer substitution mechanically visible.

**Relevant subsystems**

- one Worth Store physical-record journey product with child-process roles
- one separately linked offline physical observer/verifier executable
- existing cache-sharing compiler-boundary product
- C.1 direct runner, smoke/CI/release lanes, and feedback budgets
- dependency, feature, and authority anti-substitution gates

**Relevant APIs**

- ordinary `worth_store::physical_runtime` facade only for writer/reopener roles
- stable physical-format declarations only for the offline observer
- C.4 media fault schedule and production interposer
- exact direct counter snapshots

**Warnings**

- These are three scenario families, not three tests per phase and not a matrix
  generator. Each scenario combines several causal assertions around one
  responsibility.
- The offline process may depend on stable physical-format decode vocabulary;
  it must use its own decoder, traversal, and current-root decision path and
  must not depend on Store runtime open, production bootstrap/manifest decoders,
  current-root selection, caches, record facades, or writer-returned decoded
  state.
- Expected records, placement classes, process schedule, artifact grammar, and
  fault outcomes are fixed before the verifier runs.
- A scenario that succeeds only under certification feature authority must
  still drive the ordinary production facade and production media methods.

**Test requirements**

- **Courtroom A — `record_world_survives_fresh_processes`:** start absent; use
  a small admitted page size and deterministic mixed batches to force at least
  three segments, multi-block manifests, inline records, exact-page-boundary
  records, and an extent record at least 17 transfer widths long. Persisted page
  bytes must be at least 64 transfer widths. After the writer reports a complete
  publication, terminate it without `close`. A distinct reopener receives only
  root/configuration and externally stored locators, readmits them, locates in
  adversarial order, and performs bounded scans. A third offline process walks
  the files. Include a cross-batch append into a partially filled page and prove
  stable record ids survive its placement-generation change without offset
  rebasing. Runtime, independent oracle, and offline meaning, record ids,
  artifact topology, bytes, barriers, and bounded-work counters must reconcile.
- **Courtroom B — `publication_cutover_never_invents_current_truth`:** create a
  separately rooted world for each of the small set of causally distinct seams
  fixed in Phase 10. Kill the writer at the actual production interposer, then
  compare fresh reopen and offline observation. The only allowed results are
  the exact prior root, exact fully published successor, or the specified
  indeterminate classification. No staged artifact becomes current, no old
  manifest is guessed when the current catalog is unavailable, and completed
  byte/barrier prefixes match exactly. No world exposes a subset of one batch,
  and closing after interruption never adds a sync or strengthens the result.
- **Courtroom C — `bounded_scale_identity_format_and_policy_courtroom`:** grow record
  and manifest counts geometrically while keeping transfer and scan limits
  fixed. Assert the declared open, locate, and scan slopes through exact
  structural counters and allocation instrumentation. In the same fixed worlds,
  prove locator readmission, private placement-generation denial, old residue
  exclusion, current-catalog removal refusal, catalog-selected stale-manifest
  rejection, checksum rejection, and unsupported-format localization. Change
  placement and access policies without format drift, then change actual format
  law and require denial. Runtime and offline
  observation must converge for every valid world and disagree visibly for
  every injected invalid world.

**Engineering decisions**

- Add at most one new integration test target,
  `physical_record_journeys`, containing the three scenario families and their
  child-process roles. Keep `test = false` on any standalone observer binary.
- Extend an existing consolidated physical compiler-boundary runner. Do not add
  a new trybuild runner, per-fixture Cargo project, nested Cargo invocation, or
  target directory.
- Owner checks run codec, planner, allocation, record-identity, and outcome tests
  without launching the courtrooms. Developer smoke runs one compact
  deterministic specimen of each responsibility through one package build.
  CI runs all three medium scenarios; release verification widens seeds,
  counts, and fault repetitions without creating new test binaries.
- Mechanical gates reject production dependency/use of
  `InMemoryPhysicalFormatModel`, `InMemoryPhysicalFormatReplayArtifact`,
  `PersistedPhysicalLayout`, test authority, direct `std::fs` record writes,
  raw qualified-media extraction, whole-store collection helpers, and alternate
  current-root election.
- Mechanical gates also reject a public location-shaped record identity,
  production `create_or_open`, production `OfflineManifestCodec` use, candidate
  manifest loops, flat root-manifest entry vectors, linear record membership
  lookup, and C.6-owned catalog replacement.
- Test support is scenario-owned. It may generate input bytes and independent
  expectations, launch processes, schedule faults, and parse evidence. It may
  not write Store artifacts, choose runtime current truth, or decode through
  the runtime on behalf of the offline observer.

**Open questions**

- None.

## Must Ship

- one consuming real-media-to-record-serving admission with sealed checked
  outcome topology, explicit initialize/open paths, and no heap/replay
  construction path
- version-1 canonical checksum-protected page, segment, extent, bootstrap,
  manifest, record-id routing, generation, allocation, and free-space bytes
  with golden parity
- one append-and-publish current-root authority using the exact seven-state C.4
  transfer/barrier progression
- batch append, inline/page packing, segment rollover, extent streaming, point
  locate, bounded scan, stable record-id export, and current Store readmission
- separate Store-wide format, evolvable append placement, and per-open access
  configuration axes
- one explicit authority graph from namespace identity through catalog, root
  routing, sub-manifests, and frames; directory listings are residue only
- bounded bootstrap and manifest-specific fixed-fanout lookup whose ordinary
  work does not scale with total Store width
- Store-owned partial, unpublished, indeterminate, format, stale, rebind,
  locator, placement, and cursor outcomes
- exact operation, artifact, record-id, allocation, copy, manifest, transfer,
  publication, barrier, scan, and lifecycle counters
- narrow C.6 `FrameLoadPort`, `CandidateFrameSet`, and
  `CandidateFramePublicationPort` seams that leave publication in Store
- explicit canonical-topology and counter-performance lowering after Store-owned truth exists
- three direct production scenarios and focused anti-substitution gates

## Must Preserve

- `worth-store` remains the only public composition/runtime authority.
- `worth-store-physical-format` owns bytes and validation but does not open
  media, elect current roots, or promote runtime phases.
- `worth-store-physical-backend` owns real filesystem effects but does not know
  record, page, segment, extent, manifest, or current-root meaning.
- C.4 Store identity, root confinement, process mutation ownership, fault
  interposition, barrier capability, lifecycle, and exact media counters remain
  the only physical effect path.
- Stable `PhysicalRecordId`, private root-scoped placement, and weak serialized
  Store-id-plus-record-id locators remain different types; locators require
  explicit readmission.
- Authoritative current manifests/free-space state remain distinct from
  residue, offline observations, caches, and Foundational evidence.
- Query, Relational, Signal, Runtime Bridge, semantic MVCC, aspect meaning, and
  runtime branch authority remain absent from Part I.
- C.6 may change residency mechanics; C.7 may change durable transaction
  ordering; neither may require a new physical record format or parallel record
  facade.
- Expensive certification remains outside owner feedback and developer smoke,
  without weakening the production-path assertions it eventually runs.

## Non-Fake Acceptance Setup

### Production Subject

- Public facade: `worth_store::physical_runtime`, specifically the existing
  admission path through `AdmittedPhysicalRuntime`,
  explicit `MediaOwnedPhysicalRuntime::initialize_record_store` or
  `MediaOwnedPhysicalRuntime::open_record_store`, and the resulting
  `ServingPhysicalRuntime` record facade. No production `create_or_open` exists.
- Production call path: `worth-store` orchestration ->
  `worth-store-physical-format` byte grammar ->
  `worth-store-physical-backend` C.4 media effects. `worth-proof` participates
  only in media-owned-to-serving admission and external locator readmission.
  `worth-foundational` participates only in explicit canonical-topology or
  counter-performance lowering.
- Process products: one `physical_record_journeys` integration product acting
  as parent, writer, and reopener roles, plus a separately linked
  `physical_store_offline_observer` executable owned by
  `worth-store-offline-verifier`; its dependency graph cannot depend on or
  construct the `worth-store` runtime crate.
- Expected ordinary artifact families beneath the admitted root:

  ```text
  namespace/identity
  namespace/mutation.lock
  families/records/bootstrap.catalog
  families/records/roots/root-<generation>.manifest
  families/records/segments/segment-<id>-<generation>.pages
  families/records/segments/segment-<id>-<generation>.manifest
  families/records/extents/extent-<id>-<generation>.data
  families/records/extents/extent-<id>-<generation>.manifest
  families/records/free-space/free-space-<generation>.manifest
  staging/records/<publication-identity>/...
  ```

  Phase 1 may refine separators and fixed-width encodings, but these semantic
  roles, authority distinctions, and typed namespace ownership are locked.
  Staging paths are absent after a clean control run and appear only as exactly
  classified residue in fault worlds.

### Initial World

- Start from an absent real temporary Store root on the host filesystem and
  initialize explicitly. Every reopen uses only the open path.
- Use persisted format version `1`, `PhysicalPageSizeClass::KiB16`, atomic-
  replace catalog protocol, and mandatory checksum integrity. Separately use a
  placement policy with a 32-page segment target, an 8 KiB extent threshold,
  and manifest node capacity small enough
  that the seeded workload creates at least three levels or the maximum
  supported nontrivial depth. Separately admit a per-open 64 KiB maximum
  transfer width, 128 KiB operation-scratch ceiling, and 17-record scan batch
  limit.
- Use deterministic workload seed `0xC5C5_0000_0000_0001` and record bytes
  generated independently from `(record_ordinal, declared_length, seed)`.
- Use the real filesystem production read/write profile admitted by C.4. The
  control has no faults. Fault worlds each declare one C.4 schedule before the
  writer starts.
- Before action, no `families/records` artifact, replay object, persisted
  layout, expected manifest, decoded record, or test-created Store file exists.

### Execution Topology

- The parent passes only root, format expectation, per-open access policy, seed,
  role, and fault schedule identity to reopeners. Only initializing/appending
  writers receive placement policy.
- The writer uses only the public runtime facade. Courtroom A kills it after a
  production publication-complete yieldpoint and before normal close.
- Courtroom B kills a fresh writer at each named production boundary. No seam
  is simulated by returning an error while the process survives when process
  death is the claimed condition.
- The reopener is a fresh process and receives only root, format expectation,
  access policy, plus
  weak external locators where a readmission test requires them. It receives no
  runtime identity, media owner, current root, manifest objects, pages, extent
  bytes, decoded values, or expected state from the writer.
- Every run records process identity, runtime identity, Store identity, media
  owner identity, format identity, placement-policy identity where applicable,
  current-root generation, publication
  identity, and each authority transition.

### Independent Observation

- The offline observer opens ordinary OS files read-only and uses stable
  field/tag declarations plus its own bounded decoder, traversal policy, and
  catalog-selection logic.
- It does not link Store runtime open, current-root selection, publication,
  record facade, recovery, caches, or test-oracle state.
- Expected semantic records and allowed fault outcomes are generated and sealed
  by the parent before the reopener and observer run.
- Runtime point/scan results, offline topology, and the independent record model
  compare through declared canonical meaning; digest equality alone never
  decides parity.

### Assertions

- exact record bytes, ordered stable record ids, Store/record-id locator
  readmission, root-scoped placement generations, placement class,
  page/slot/extent membership, and physical scan sequence
- exact artifact path set, lengths, frame boundaries, manifest membership,
  current-root selection, staged residue, and absence of out-of-root effects
- exact media effects and barrier sequence required by the admitted C.4 profile
- exact batch all-or-none visibility and exact seven-state publication
  progression; `close` adds no durability effect
- exact or weakest-sufficient structural counters, including zero unrelated
  segment scans, zero directory-based root elections, zero payload reads on
  denied locators, zero whole-store materializations, zero hidden whole-
  record copies, and zero test-owned physical writes
- open, locate, append, scan, manifest, transfer, and operation-allocation
  bounds at their named boundaries; persisted record bytes exceed the C.5
  scratch ceiling by at least 32 times and aggregate persisted page-frame bytes
  exceed transfer width by at least 64 times
- typed failure localization for partial writes, unavailable current catalog,
  torn/checksum-invalid authority frames, stale/foreign locator, unsupported
  format, format drift, and indeterminate
  publication

### Mechanical Anti-Substitution Gates

- dependency/source gates deny production reachability of
  `InMemoryPhysicalFormatModel`, `InMemoryPhysicalFormatReplayArtifact`,
  `PersistedPhysicalLayout`, and test-authority constructors
- production Store code cannot use direct `std::fs` record writes or extract a
  raw backend handle around the C.4 media owner
- ordinary open accepts no layout/replay/catalog/page/manifest argument and no
  directory-listing current-root strategy
- ordinary open cannot initialize; initialization requires proven absence and
  cannot adopt or erase residue
- direct allocation and counter assertions reject complete-store/manifest/
  extent collection on the ordinary path
- source/dependency gates reject public location-shaped record identities,
  production `OfflineManifestCodec` reachability, candidate-manifest loops,
  flat root-entry vectors, linear membership lookup, and C.6 catalog writes
- the offline observer dependency graph cannot import `worth-store` runtime or
  invoke Store open/record APIs, and cannot call production bootstrap/manifest
  decoders or current-root selection
- no new trybuild runner, per-case target directory, or nested Cargo behavioral
  test is introduced
- a Foundational artifact, digest, report, copied identity, format witness, or
  weak external locator satisfies no serving/publication authority signature

### Evidence And Rerun

- Each scenario directly asserts runtime outcomes, independent observations,
  artifact membership, record placement, fault behavior, and counters. Failure
  output names the seed, process role, schedule, and relevant identity needed
  to reproduce the case.
- Scenario output is disposable diagnostics; it is not a certificate or input
  to another test.
- Bind rerun commands to the C.1 lanes: owner check for local codec/planner
  work, one compact developer-smoke selector, full `physical_record_journeys`
  for CI certification, and widened deterministic campaigns for release.

## Acceptance Evidence

- golden byte corpus and independent current-format decode parity
- consuming serving-admission compiler and runtime evidence
- exact real artifact topology and current-root publication trace
- fresh-process record/scan/record-id parity
- bounded bootstrap, locate, scan, extent-streaming, and manifest-slope counters
- typed format, policy, stale, rebind, partial, and indeterminate outcomes
- Store-owned counter snapshots plus optional explicit Foundational
  counter-backed/canonical boundary artifacts
- offline/runtime canonical comparison with visible disagreement
- C.1 smoke-budget, boundary-check, agent-context, strict lint,
  warning-free, and source-cap results

## Sequencing Notes

### Canonical residency and writeback amendment

- `PhysicalResidencyPool` is the sole ordinary residency architecture.
  Historical S.2 models are certification-only behind the explicit legacy
  feature and are forbidden from the ordinary Store dependency graph.
- Store owns the join between one dirty frame claim, one scheduler plan, and
  one exact backend range write. Callers cannot provide a generic completion,
  raw pool, raw claim, backend execution witness, receipt, or artifact target.
- The declaration binds the complete `StoreSecurityScopeIdentity`; matching
  tenant, key class, and authenticity alone cannot substitute a different
  physical witness, key-version posture, or custody posture.
- Writeback admission revalidates the exact pool incarnation and scheduler
  plan before effects. A frame becomes clean only after the backend receipt
  then revalidates Store, frame coordinate, byte length, and digest and the
  scheduler accepts the completion created by that I/O.
- `BufferedWriteCompleted` means the OS accepted the exact write and flush debt
  remains. A durability-requiring plan completes only after the required file
  synchronization and returns `FileDataSynchronized`.
- Pre-effect denial leaves the claim dirty and retryable. Partial or
  indeterminate I/O revokes serving health and requires inspection. A completed
  physical write with rejected scheduler completion remains dirty but is safe
  to repeat because the exact range write is idempotent. A post-write residency
  invariant failure retains both the physical receipt and terminal denial.

- Phases are implemented in order. No batch skips the first real-record slice
  in favor of broad format vocabulary or evidence infrastructure.
- Phases 1 through 3 form the minimum implementation batch because the public
  serving phase is forbidden until one record survives a process boundary.
- Phases 4 through 7 scale placement and record-identity truth on the same
  publication path.
- Phases 8 through 11 close scalable access and failure/readmission behavior.
- Phases 12 through 14 freeze the C.6 seam, narrow Foundational evidence, and
  joined production proof.
- C.6 begins only after all ordinary C.5 record access uses the named
  `FrameLoadPort`/candidate-frame seams and the three C.5 scenarios are green.
- C.7 may replace C.5 direct publication orchestration with WAL-governed
  progression only by consuming the same artifacts and outcomes; it may not
  preserve a shadow direct acknowledgment path.
- Any missing lower capability discovered during implementation expands C.5
  into its owning crate. It is not replaced by a Store-local helper, public raw
  seam, fake backend, or permanent-looking debt marker.
