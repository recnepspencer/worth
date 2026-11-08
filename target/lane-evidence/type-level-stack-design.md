# Original-funded type storage (revised design, not implemented)

H10 requires the ordinary Windows dev example mains on their genuine 1 MiB stack.
Other frame ceilings are C2. Protected Native/World files remain untouched.

## Findings accepted from follow-up review

The initial complete design is REJECTED: three SignalRuntime boxes allocate
on ordinary Bridge lowering; the exclusivity reservation is not heap funding;
the early FixedOperation denial does not reach the conditional slot. Withdraw
runtime boxes. Keep the already-funded partition box. No product edit exists.

## Causal owner fix

LoweredBridgeAsyncSourceDeclaration is 3160 bytes (body 2992), embedded in the
4432-byte AdmittedBridgeAsyncRequestIdentity and 5440-byte bound basis. Query
FixedOperation cleanup/denials carry these. Put its immutable body in a private
Arc payload at the definition. Keep identity, counters and per-instance digest
on the outer value; live runtime/attempt custody remains outside unchanged.

The payload owns body, original template digest and optional original
ExecutionMemoryReservation. Debug/Eq exclude the ticket and preserve meaning.
Operational managed/atomic factories require a private reservation obtained
from their actual original ExecutionRequest::reserve_retained_memory before
allocating. Map refusal through existing ExecutionDenied(BridgeExecutionDenial),
retaining the exact cause. Quote Arc payload/control block plus newly retained
fixed-length template digest Arc<str>, using checked layouts and SHA-256 output
length plus the existing prefix. Ticket lives until the final Arc drops.
Native-only None is the existing request contract, never a fresh grant/fallback.
Nearest analogue: recorded_baseline/json.rs reserves before allocating.

Existing public lower() and owned-async installation construction remain cold
owner routes using the same semantic lowerer; only operational factory requires
the original reservation. No new public door or authority. Split lowering storage
and construction into named responsibilities, <=400 lines.

Readmission already owns the original lowering. Re-instantiate its shared body
for the fresh intent instead of rebuilding/allocating a new body. Store the
original template digest so it never hashes the previous instance digest.
Result must equal fresh lowering of the same contract for the same fresh intent.
Keep all runtime, operation, source-profile, attempt, Signal and queue checks.
No readmission request/grant is invented.

Box SignalConditionalExecutionSlot.partition at its definition and the two
constructors: execution_kernel.rs and evaluation_reuse/readmission.rs. Initial
partition admission already reserves capacity<Self>(1) before construction.
Both storage/slot reservations remain inside it; no duplicate charge.

## Expected evidence

Lowered declaration ~176 bytes (guard <=256), bound basis ~2456, slot pointer
size (guard <=64). SignalRuntime stays15552; other ceilings are C2. Update
metadata expressions to the real field types. Genuine SerialRequest and
SerialMemoryBudget tests prove original charge retention until last body share,
refusal without backing, and canonical template re-instantiation equivalence.

Finish all callers/constructors before checks. Three implementation rounds:
one check/reached workspace, owned focused builds/runs of changed tests, type
guards, actual example mains, consumer-root and Bank parents. Audit exclusions.
No stack override; verify unchanged 1048576 PE reserve. Reuse unchanged Store
measurements. Rerun seven-binary frame census. Record risk before extra rounds.
