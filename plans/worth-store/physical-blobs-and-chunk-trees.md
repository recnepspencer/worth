# Physical Blobs And Chunk Trees

This document describes native ingest, protected reads, resume readmission,
explicit abort, checkpoint-driven abandonment, and failed-ingest residue
reclaim. It does not certify C.11 Phase 3 completion or claim published-generation
reclaim, dedupe, movement, or export/import integrations.

## Entry And Authority

Obtain `PhysicalBlobFacade` through `ServingPhysicalRuntime::blobs()`.
The facade borrows the serving runtime; it is not an independent catalog or
authority owner. An inspection-required runtime denies the accessor.

Build `AdmittedBlobScope` from an admitted Store security scope. A raw scope
fingerprint cannot grant access. `issue_object_id()` issues a Store-bound
identity after bounded selected-root collision inspection. Exhausting that
inspection denies admission rather than treating incomplete evidence as absence.

The compiled [ingest example](../../workspaces/worth-store/crates/worth-store/tests/physical_runtime_authority/blob_ingest_supported.rs)
shows the borrowed session and shutdown ordering. Its companion compiler tests
reject recovery allocations as ingest declarations, sessions escaping their
runtime, and runtime close while an ingest session remains live.

## Ingest And Publication

`BlobIngestDeclaration::new()` binds the issued object, fixed chunk size,
declared total bytes, admitted scope, checkpoint horizon, and mutation deadline.
Current chunk sizes are 64–256 KiB. The horizon is relative to Store's selected
durable checkpoint; callers cannot invent an absolute expiry checkpoint.

`begin_ingest()` admits the Store allocation and durably publishes a session
declaration before accepting chunk bytes. Each `push()` accepts at most the
admitted source window and never more than the declared remaining object bytes.
An oversized source frame is denied before chunk effects. One partial chunk
is retained between calls; chunk occurrences and tree nodes use ordinary C.5
mutation, WAL, and root-publication ownership.

`finish()` consumes the session, requires exactly the declared byte count,
finishes the tree, and publishes generation 1. It does not publish a generation
for an incomplete object. Each chunk's C.5 root progression is distinct from
the one final blob-generation publication.

The source window must be nonzero, at most 64 MiB, and smaller than the object.
The ingest residency contract is the window plus 5 MiB, including the caller's
source frame and simultaneously live buffers. The process-level test measures
actual peak allocation independently of the component charge observation.
Run that 8 MiB fresh-process/independent-observer journey in the ordinary
feature graph, from `workspaces/worth-store`:

```text
cargo test -p worth-store --test physical_blob_journeys --no-default-features --offline -j1
```

The certification feature retains per-allocation diagnostic events, which are
not part of the production ingest heap. Run its blob crash and recovery
journeys separately with:

```text
cargo test -p worth-store --test physical_blob_journeys --features certification-test-authority --offline -j1
```

The buffer-pool allocation trace has its own focused certification lane:

```text
cargo test -p worth-store-buffer-pool --features certification-test-authority allocation_events --offline -j1
```

The 8 MiB memory case is excluded only from the certification blob graph. The
ordinary and certification blob lanes keep the same Store mutation and C.8/C.9
boundaries.

## Interrupted Ingest And Resume

`resume_token()` returns a portable, untrusted reference to the selected
declaration. Save its `encode()` bytes with the caller's source position;
`BlobResumeToken::decode()` validates transport syntax, not authority.
`frontier()` reports completed chunk bytes, excluding buffered partial input.
`checkpoint()` persists that completed prefix without flushing partial input;
ordinary ingest also persists a frontier every 64 completed chunks.

After dropping the old handle, or recovering and reopening a killed writer's
Store through C.8, readmit through the same borrowed facade:

```rust,ignore
let mut ingest = blobs.resume_ingest(
    token, &scope, placement, window_bytes, deadline, resume_limits,
)?;
source.seek(ingest.frontier().bytes())?;
// Feed the remaining source bytes through push(), then finish().
```

Readmission authenticates the original declaration, scope, chunk rule, total,
and absolute checkpoint limit against a protected selected root. It rejects a
competing live handle, publication, selected abandonment, expired declaration, gaps, or conflicting
occurrence claims. It verifies and reuses selected chunks beyond the last
frontier record when a crash interrupted frontier publication. It rebuilds the
logical digest from selected bytes and validates existing tree nodes before
performing any resumed append. A new deadline never extends original expiry.

This is explicitly cold reconstruction, not constant-time token lookup.
`BlobResumeLimits` bounds scanned records and reserved claim metadata. Selected
claims are sorted in place; reconstruction rereads each selected chunk using
one bounded buffer and the protected Rebuild reader. Source-window residency
remains charged throughout, within the same window-plus-5-MiB ingest grant.
Capacity exhaustion denies readmission without abandoning durable custody.
`resume_observation()` separates selected scan rows/payload bytes from rehashed
chunk bytes, reused reconstruction nodes, and metadata capacity. Nodes retained
for a later `finish()` are not included in reconstruction's reused-node count.

The scheduled multilevel resume journey uses 4,097 64 KiB chunks: one full
leaf, one partial leaf, and an interior root. It kills the real writer after
the interior root is selected but before generation publication, then performs
fresh-process C.8 recovery, exact selected-record reuse checks, streamed byte
verification, and independent C.9 observation. It completes a physical
checkpoint every 64 chunks; these are separate from blob frontier records.
Build the recovery CLI before running this expensive, explicitly ignored case:

```text
cargo build -p worth-store-recovery-runtime --bin physical_store_recover --offline -j1
cargo test -p worth-store --no-default-features --test physical_blob_journeys blob_resume_multilevel::killed_interior_root_resumes_exact_selected_tree --offline -j1 -- --ignored --exact --nocapture
```

This journey uses the finite `c11-blob-multilevel-v1` recovery profile and is
not the separate 4 GiB acceptance lane or a Phase 3 completion certificate.

## Explicit Abort And Retained Custody

Dropping an ingest handle does not abort its durable session. After releasing
that handle, explicitly select abandonment through the borrowed facade:

```rust,ignore
let receipt = blobs.abort_ingest(
    token, &scope, placement, deadline, terminal_limits,
)?;
```

`BlobTerminalLimits` bounds the complete selected-record inspection. The token
and caller scope must authenticate the original declaration; another live or
inspecting same-session claim denies the attempt. Abort appends a versioned
declaration-bound terminal through the existing C.5/C.8 path and does not
rehash all chunk bytes. `BlobTerminalDisposition` distinguishes
`NewlyAbandoned` from a previously selected `AlreadyAbandoned` result. Resume
rejects either with `BlobResumeFailure::AlreadyAbandoned`, including after
recovery and reopen. An uncertain physical effect remains a typed
`BlobTerminalFailure::Append(BlobAppendFailure::Indeterminate(..))` requiring
C.8 reconciliation; it is not an effect-free abort denial.

The terminal record preserves every existing route and byte. Generic C.10
read leases therefore do not veto this append-only fate transition, but all
reader/recovery pins remain binding on subsequent reclaim and retirement.
Abort is not deletion and does not grant release authority for a published
generation. Failed-ingest record-dropping reclamation is a separate, bounded
operation described below.

The compiled ingest example also exercises the public abort signature.
Focused integration coverage is in `physical_blob_journeys::blob_abort`;
its independent observer must be rebuilt along with the C.8 recovery binary
before running those process boundaries.

## Checkpoint Expiry (Under Verification)

`expire_ingest(token, &scope, placement, deadline, terminal_limits)` uses the
same declaration authentication, bounded selected-record inspection, and
same-session claim exclusion as explicit abort. The checkpoint owner supplies
the completed durable checkpoint; the caller cannot supply its sequence.
An issued or partially published checkpoint is insufficient. With no completed
checkpoint strictly beyond the original declaration's maximum, the call returns
`BlobTerminalFailure::NotExpired` before append effects.

The expiry terminal carries a nonzero completed-checkpoint witness. Selected
fate requires `declaration.maximum < witness <= selected_completed_checkpoint`;
a later completed checkpoint may replace the original witness's checkpoint
artifact. Resume and repeated terminal calls validate the same relationship.
C.8 checks retained-WAL expiry against the exact selected declaration before
redo effects; the independent observer checks selected terminal custody against
the completed current checkpoint. Expiry, like abort, does not delete records.

C.8 cleanup must also preserve physical WAL continuation. A checkpoint cutoff
does not supply a replacement WAL segment/generation identity. Source admission
therefore retains the authenticated covered suffix connecting the binding
cutoff to the physical replay tail, or the exact terminal segment when no replay
tail remains. Missing, conflicting, interrupted, or discontinuous covered-suffix
evidence is denied before recovery effects. Ordinary reopen still rejects a
checkpoint whose cutoff lies outside retained WAL; it never invents an origin.

Run the focused lane after rebuilding both standalone recovery and observer
executables:

```text
cargo test -p worth-store --test physical_blob_journeys blob_expiry --offline -j1 -- --nocapture --test-threads=1
```

The pruning/reopen journey exposed C.8 cleanup deleting the final WAL
continuation segment. With continuation retention installed, the journey now
passes exact retained-byte checks, ordinary reopen, a new append at the expected
LSN, and repeated recovery. Final verification also covers the public source
join: a WAL tail admitted under an absent or different checkpoint basis cannot
be combined with the selected checkpoint. Independent review accepted this
expiry/cleanup correction after the focused journeys, public API boundaries,
and C.8 interruption families passed. Subsequent retention-accounting changes
require their own affected reruns; this is not a Phase 3 completion claim.

## Failed-Ingest Residue Reclaim

After explicit abort or completed-checkpoint expiry, use the borrowed facade to
admit a bounded reclaim batch. A selected declaration and Abandoned terminal,
the Store-bound token and scope, and the selected graph must agree. A live or
inspecting same-session claim and a published generation deny failed-ingest
reclaim; an existing protected reader can defer it. Neither an abort receipt nor
a caller-supplied list of record IDs is drop authority.

```rust,ignore
let mut receipt = blobs.reclaim(BlobReclaimRequest::abandoned(
    token, &scope, placement, deadline, reclaim_limits,
))?.wait()?;
let retirement = blobs.continue_reclaim_retirement(&mut receipt, retirement_budget)?;
```

`BlobReclaimLimits` bounds selected-root inspection and cumulative encoded
payload bytes actually inspected across declaration authentication, graph
selection, reservation/fate reconciliation, and metadata cleanup. It also
bounds one drop set to at most 1,024 record identities. These are not
logical-object-byte or total backend-I/O bounds; callers must budget for all
inspection phases, not assume a fixed number of scans. A batch selects
unreferenced graph roots before children;
subsequent bounded requests can progress remaining residue. Selected frontier
prefixes, tree edges, and generation publications protect their referenced
records. Shared or published records are not treated as failed-ingest residue.

Dropping or cancelling an admitted handle has `ProvenNoEffect` disposition.
`wait()` uses ordinary C.5 mutation, WAL, and root publication for its manifest,
reservation, and typed drop descriptor. Its receipt distinguishes
`ProvenNoEffect` from `Dropped`, reports exact displaced extents and remaining
payload records, and may report retirement still pending. A durable drop is not
itself proof that protected extents were released: C.10 retirement supplies that
credit. `BlobReclaimObservation` reports semantic inspection separately from
Store physical-media counters.

Continue pending retirement with a finite `BlobReclaimRetirementBudget` on the
same issuing runtime. A completed generic retirement can be credited exactly
once without repeating the selected drop. `Pending(ArenaIndexCapacity)` records
real allocator-index pressure; blindly repeating the same budget does not
remove it. A receipt cannot be resumed in a different or reopened runtime;
inspect selected custody through a fresh reclaim request instead.

A crash after manifest publication but before the drop retains identifiable
manifest/reservation custody. After C.8 reconciliation establishes that the
original drop had no durable effect, a subsequent bounded request can remove
that metadata through a root-only durable drop, without inventing a payload
drop. `BlobReclaimFailure::ManifestRetained` means the selected manifest is
durable and the descriptor is proved absent, so later bounded cleanup owns the
residue. `BlobReclaimFailure::Publication` with an indeterminate append cause
does not prove the descriptor absent; C.8 must reconcile it before another
attempt, never a blind retry. C.8 replays a durable descriptor before its root
publication, and the independent C.9 observer checks selected custody. This
path does not implement proof-consuming published-generation release,
cross-session dedupe, or movement.

## Protected Range Reads

`resolve_publication()` accepts a portable object/generation reference, but
resolves it only through a bounded scan of the selected C.5 root under the
admitted scope. `BlobReadLimits` bounds that scan. Scan exhaustion, missing
publication, conflicting publication, and scope mismatch are distinct denials.

Until the derived catalog is installed, this inspection consumes the layout
owner's maintenance `RebuildRead` declaration. Its allocations are charged to
maintenance, and media reads and metadata queries use securely admitted,
bounded `RepairScan` background work. Capacity denial occurs before read-work
submission. This is not a foreground full-scan fallback.

`read()` opens one requested range under the protected root captured by the
publication scan. `BlobReadSession::read_next()` authenticates only the tree
path and chunks needed for that range, retaining the current chunk for smaller
caller transfers. Record-read transfer chunks and persisted blob chunks are
different units.

Completing catalog selection drops the scan cursor and maintenance allocation.
The returned reader retains the same protected root, but tree/chunk traversal
uses ordinary read admission; it does not retain the catalog's rebuild lane.

`BlobReadObservation` separates logical tree-node/chunk loads and returned
bytes from C.5 physical-work counts. Physical work includes tree/root/chunk
read and artifact-metadata operations; publication-catalog scanning is a
separate bounded operation and is excluded. Reusing a retained chunk adds no
physical work. Reads retain their Store allocation and protected root until
the session is dropped, even after all requested bytes have been returned.

## Failure And Lifecycle

Append failures retain the underlying preparation, proven-no-effect, or
indeterminate physical fate. A failed chunk flush poisons the session against
further writes. Do not convert indeterminate failure into a blind retry.
Dropping an unfinished session releases live allocation, but does not roll
back its durable declaration or claimed chunks. Recovery and the independent
offline observer recognize that retained unfinished custody without inventing
a published generation or an orphan.

Durable terminal arbitration establishes abandonment, but neither explicit
abort nor a resume expiry denial alone authorizes deletion. Failed-ingest
residue reclaim requires the selected proof and bounded graph inspection above;
it never silently replaces or deletes unfinished custody. Published-generation
reclaim remains deferred. The complete destination contract remains the
[C.11 specification](physical-reconstruction-c11-layout-index-and-native-blob-adoption.md).
