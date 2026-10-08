# Checkpoint allocation custody

Status: implemented and verified for checkpoint payload custody; broader allocation work remains open.

The published baseline reserves one final frame and shares immutable checkpoint
backing through native-region handoff without execution-ledger custody. This
slice admits that actual allocation under an explicit caller policy and keeps
its charge until the last enclosing Query/native owner drops.

## Authority and public contracts

worth-execution owns one opaque fresh byte buffer and immutable shared backing.
An explicit allocation policy selects SystemAllocation or a caller-borrowed
ExecutionResourceLease. Neither policy has a Default. SystemAllocation preserves
ordinary authority-free composition with checked sizing and fallible allocation;
it grants no ledger admission. Execution reserves the checked payload layout
before allocation and never falls back after a refusal.

Query reexports this physical policy for its existing capture, capture-with-sizes,
transition installation and consuming repair methods. Every caller selects it
explicitly; there are no parallel legacy capture methods. Physical policy does
not enter graph, program, installation, source or idempotency identities or the
checkpoint wire grammar. Ordinary restore without a transition encodes no frame.

Query owns a typed capture denial containing either its existing native/framing
DurabilityError or the exact lower allocation denial. Allocation refusals retain
layout, lease, cancellation, deadline and allocator distinctions. A requested
payload quote is diagnostic information, never caller-provided authority.

Recovery keeps its existing unpublished graph and Deferred/Acknowledged native
settlement custody. Each repair receives a fresh explicit policy; it retains no
borrowed lease and never reruns authoring. Early stop leaves native settlement
untouched. Refusal after acknowledgment retains that authority and the current
typed capture cause. A later noncapture settlement failure clears an obsolete
capture cause. No refused capsule exposes a World.

## Allocation and immutable lifetime

Use pinned allocator-api2 0.2.21's concrete fresh Global Vec path: validate its
exact u8 Layout, check live stop policy, reserve the requested payload amount,
then fallibly allocate. No std Vec capacity inference, postallocation admission,
existing-buffer growth, shrink or boxed-slice conversion supplies this proof.
Allocator and Arc headers and ledger bookkeeping remain outside the charge.

The builder checks capacity before copying, polls stop policy during chunked
emission and allows only bounded overwrite of already-written checksum bytes.
Query uses one frame grammar for both modes. Sizing checks each accepted row and
role; SHA updates also poll live stop policy. Seal requires exact consumption in
release mode. Frozen owners expose only immutable bytes; clones share one backing
and one nonclone reservation. Payload deallocation precedes ticket release on
success, refusal and final drop. Native-region ownership retains the entire
enclosing frame charge, even if only a small region is read.

Moved external Arc<Box<[u8]>> buffers remain explicitly uncharged. Allocation
mode and custody metadata do not change byte-value equality or recovery authority.

The process memory cap becomes Option<u64> only at its three comparisons.
Some preserves every existing value, including zero. None removes the process
comparison while retaining checked ledger arithmetic, workers and all finite
lease/ancestor/algorithm budgets. No sentinel or default allowance is introduced.

## Integration and evidence

The reviewed inventory contains 54 public invocation occurrences (53 external
sites) and 16 private consumers. The sole private production caller is House
session checkpoint capture; its ordinary policy remains SystemAllocation.
Both repositories cut over existing callers rather than retaining aliases.

Lower proofs exercise actual capacity/pointer stability, bounded append/overwrite/
seal, stop policy, concurrent backing charges, clones outliving leases and final
release. Separate process-fenced test binaries cover optional process policy and
Some(0); existing finite ancestor tests stay intact. A real Query capture proves
leased custody through native-region decode and readmission. Genuine zero-memory
lease refusal proves no fallback. A real performed transition proves retained
acknowledged repair custody, exact refusal and author count one, followed by an
explicit successful repair and ordinary reopen. Existing producer reopen/source
change and historical grammar tests protect SystemAllocation and wire semantics.

Six path lockfiles change only allocator-api2's exact dependency edge/package.
The UI compile-contract fixture also repairs its stale WORTH dependency closure.
Ten implicit UI dependencies now name their existing UI workspace explicitly,
following Runtime and Text; this preserves that owner's MSRV rather than copying
its package policy into the fixture or root. Its refreshed lock retains every
preexisting registry version/checksum; additional registry packages already belong
to the parent UI lock. Root serializes locked offline Cargo checks, boundary
snapshots, context, facade-consumer checks and scoped line caps. No existing
registry package is upgraded.

Native image encoding, imported bytes, accepted-output/source-fact copies, decoded
vectors, metadata, House compression/transport and downstream allocations remain
separate. This slice closes neither whole-heap accounting nor ordinary count
eligibility, the 100,000/million-record axes or the original allocation abort.

Actual verification: 59 runtime/doc tests passed across fixed byte backing, process
policy, finite ancestors, native regions, Query wire/capture, transition repair and
producer reopen. The external topology consumer checked successfully. All seven
locked offline metadata checks, boundary snapshot/check, agent context and scoped
Rust line caps passed. The final diff check passed after correcting two inserted
manifest line endings. These results do not close the separate scope listed above.
