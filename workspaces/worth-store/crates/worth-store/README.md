# worth-store

This is the public facade and physical runtime composition owner for the
rebuilt Worth Store workspace. It owns live Store lifecycle and the adapters
that join lower physical owners to a serving Store. Physical format, lower
policy, and platform authority remain in their specialized crates.

## Native Physical Blobs

`ServingPhysicalRuntime::blobs()` exposes Store-owned ingest, explicit bounded
resume readmission, and protected range reads. Store publishes the session
declaration before chunk records and publishes a generation only after its
authenticated tree is complete. Blob
bytes are ordinary C.5 records packed into extent arenas; the mechanism crate
cannot persist bytes or issue Store allocations.

Ingest and read sessions borrow the serving runtime and retain their Store
allocation until completion or drop. A failed append preserves its physical
no-effect or indeterminate fate; dropping a session does not erase durable
unfinished custody. Current publication lookup is an explicitly bounded scan,
not an index lookup. Resume authenticates its untrusted token against a
protected selected root, rehashes the durable chunk prefix, and preserves the
original checkpoint expiry. A denial does not abandon or delete durable custody.

See [physical-blobs-and-chunk-trees.md](../../../../plans/worth-store/physical-blobs-and-chunk-trees.md)
for the implemented caller contract, limits, and later-phase boundaries.

## Protected Physical Reads

`ServingPhysicalRuntime::records()` captures a protected physical root or
returns a typed admission denial. Readers, sessions, and scans retain that
root's protection; root observations cannot manufacture a reader.

`try_records_for_checkpoint(original_request)` makes one nonwaiting attempt
under the same current-root, reclaim and protection-registry order. Busy,
stopped and protection refusals install no reader. Retain the accepted reader
before consulting cancellation again; no fallible request check follows lease
installation. Existing reader construction still allocates its read-source Arc;
this is no hard wall-clock or allocation-free guarantee.

`AnchoredRecords::open_payload_from` and `OrderedJournal::open_payload_from`
read through that exact protected reader. `describe_entry_from` provides an
original-request-funded journal header description without consuming the full
payload; this is metadata, not verified payload evidence. Their streams retain the original
request, copy at most 64 KiB per call, and latch any refusal. Header coordinates
are descriptive until verified EOF. Anchor EOF validates the complete anchor
record, member table and payload; journal EOF validates the target payload and,
for the selected head, its full record hash. Older journal linkage uses the
selected physical manifest's immutable identities and validated frames, not a
transitive journal hash chain. Metadata is admitted before growth; payload
storage belongs to the caller. Perform payload I/O outside logical reservations.
After verified anchor EOF, `into_verified_membership` moves the original table,
its grant and its physical session into `AnchoredRecordReadMembership` without
copying or releasing a lease. Before EOF or after refusal it returns the same
stream. `validate_read_membership` loans the existing membership only for the
same owner, exact protected root and live protection; its registry check is
nonwaiting. Keep this carrier and its reader through selected-section consumers.
Disposal follows ordinary unbudgeted physical protection release, not a Home
close or complete semantic-checkpoint proof.

`physical_runtime::stability` owns the live chunk guard, security-scope
propagation adapter, and `StablePhysicalReadExecution`. Its
`StablePhysicalReadReceipt` carries actual execution counters plus lower plan
completion. The lower `PhysicalReadPlanCompletionReceipt` alone proves no
byte access and cannot be promoted to a byte-read receipt.

Read-protection disposition adapters are compiled only for unit tests until a
C.11 owner consumes them; no resident owner issues dispositions yet.

See [plans/worth-store/bounded-physical-record-access.md](../../../../plans/worth-store/bounded-physical-record-access.md)
for admission limits, borrowed-byte lifetime, shutdown, and adapter boundaries.

## Physical Durability And Checkpoints

`physical_runtime` exposes the ordinary Store-owned durability contract.
Callers submit through `ServingPhysicalRuntime::record_submission()`, receive a
typed preparation posture, and start or execute only `PreparedPhysicalMutation`.
The terminal fate is completed, proven no effect, or indeterminate. Only the
completed typestate can become `PhysicalMutationAcknowledgment`.

The acknowledgment is physical truth under one qualified backend profile. It
is not semantic commit, Query truth, or transaction authority. Dropping a
mutation handle abandons observation only; explicit cancellation cannot turn a
possibly effectful mutation into a safe retry. `close_plan()` drains Store-owned
mutation and checkpoint work before Signal, residency, and media shutdown.

Managed fuzzy checkpoints are submitted through
`ServingPhysicalRuntime::checkpoints()`. A completed checkpoint carries its
exact contiguous retained WAL tail, idempotency-binding compaction, and lawful
WAL reclamation observation. Checkpoint existence alone grants no deletion
authority.

## Rewrite, Retirement And Scheduled Maintenance

Record-preserving rewrites are ordinary managed mutations prepared through
`PhysicalRecordSubmission` (`rewrite_selected_inline_segment`,
`rewrite_selected_inline_pages` and planned inline-artifact rewrites). The sole
publication owner admits pending publication and retained-storage growth before
the first WAL byte; refusals are the typed no-effect causes `ScopeConflict`,
`SourceChanged` and `RetentionPressure`.
`ServingPhysicalRuntime::retire_displaced_segment()` is the only deletion path
for displaced data, and it returns `PhysicalRetirementDenial` when a live
reader, a pending publication or an unfinished stage blocks it.

Every physical effect lowers exhaustively into exact conflict keys, and only
the instance executor performs scheduled maintenance effects; the repository
boundary checker enforces both. Store schedules checkpoint, scrub and WAL
reclamation as background work through the one I/O scheduler. It has no
automatic compaction producer: rewrite and retirement are caller-invoked.

See [plans/worth-store/physical-durability-and-checkpoints.md](../../../../plans/worth-store/physical-durability-and-checkpoints.md)
for the caller and operator contract.

## Physical Residency Failures

`PhysicalRecordResidencyFailure` is the Store-owned facade for a lower physical
residency denial. `kind()` supplies a broad policy class; `reason()` supplies
the exact causal reason and retains small actionable fields such as bounded-load
limits or candidate cardinality. Callers use the reason to distinguish a bad
declaration or payload from live identity contention, pin/dirty posture,
writeback-claim contention, receipt mismatch, lifecycle closure, or pressure.

The reason vocabulary carries no pool control, retry authority, Signal handle,
scheduler receipt, backend authority, `worth-proof` witness, Foundational
value, or semantic residency claim. Pressure follows its dedicated
`PhysicalRecordPressureEvidence` path, and a terminated frame load follows its
typed terminal projection.
