# Checkpoint allocation custody

Status: final-frame and native encoded-byte payload custody are verified,
including performed-transition repair. The checkpoint total-fact quota is removed
and verified. Broader allocation work remains open.

The final-frame slice admits that allocation under an explicit caller policy and
shares immutable backing through native-region handoff. Its charge stays live
until the last enclosing Query/native owner drops. The native extension applies
the same explicit policy to the earlier encoded Relational byte backing.

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

Relational's `native_checkpoint(policy)` returns the runtime-only
`RelationalNativeCheckpointCaptureDenial::{Durability, Allocation}`. Existing
serialized durability errors and recovery records retain their meaning. Query
moves either variant into its existing capture denial without reconstructing a
cause from serializer or I/O text.

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

## Native encoded-byte extension

The [native capture owner](../../crates/worth-relational/src/durability/authority/checkpointing.rs)
checks the supplied policy before capturing one immutable checkpoint image.
The [encoder](../../crates/worth-relational/src/durability/log/native_file_codec/checkpoint_encoding.rs)
counts the actual MessagePack serialization of that borrowed image using checked
length arithmetic, then emits the same grammar into one fixed
`ExecutionByteBuffer`. Counting and emission poll the live selected policy.
The [writer adapter](../../crates/worth-relational/src/durability/log/native_file_codec/checkpoint_encoding/writer.rs)
retains the original allocation denial before returning an I/O stop to the
serializer. Sealing transfers the immutable backing directly into
`RelationalNativeCheckpoint`; no final native Vec boxing or payload copy remains.
Runtime section metadata describes the emitted bytes, and wire meaning is
unchanged for System and leased capture.

Query forwards the same policy through ordinary capture, performed transition
publication and consuming repair. Native backing of size N remains live while
the final Query frame of size F is admitted and written. A leased attempt must
therefore admit the actual N + F coexistence. Query drops the native backing
after copying it into the final frame, before hashing and sealing; the resulting
checkpoint retains F. A final-frame refusal releases N while retaining the
acknowledged repair capsule and the exact current refusal. Neither policy gains
authority to skip native recovery checks or rerun authoring.

Captured-image construction, partition-alias planning and serializer metadata
heaps remain separate. Two serialization passes do not establish whole-checkpoint
heap custody or constant work, and imported buffers remain uncharged.

The process memory cap becomes Option<u64> only at its three comparisons.
Some preserves every existing value, including zero. None removes the process
comparison while retaining checked ledger arithmetic, workers and all finite
lease/ancestor/algorithm budgets. No sentinel or default allowance is introduced.

## Integration and evidence

The original final-frame inventory contains 54 public invocation occurrences
(53 external sites) and 16 private consumers. The sole private production caller is House
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

The native extension migrates three Query production seams and seventeen native
test calls, plus the direct encoder wire oracle. Its focused proof uses the
existing process authority to check exact native payload refusal, cancellation,
shared clone charge beyond the borrowed child lifetime, last-owner release,
fresh native restore and a subsequent ordinary edit. The transition proof checks
the actual N-only final-frame refusal followed by N + F repair, retaining author
count one. Existing independent wire and section-size assertions remain. These
native allocation and wire proofs passed: two actual native owner tests and eight
Host checkpoint-transition tests, including N + F repair and cancellation. The
Host run passed in 1.74 seconds with an invocation-only Relational opt-level 0
after the usual dependency opt-level 2 build encountered a compiler LLVM OOM.
Runtime assertions and allocation policies were unchanged; this is no runtime
performance claim. Query all-target metadata passed under the usual
profile before the three-file fact-quota follow-on. These results do not
replace the historical results below.

Six path lockfiles change only allocator-api2's exact dependency edge/package.
The UI compile-contract fixture also repairs its stale WORTH dependency closure.
Ten implicit UI dependencies now name their existing UI workspace explicitly,
following Runtime and Text; this preserves that owner's MSRV rather than copying
its package policy into the fixture or root. Its refreshed lock retains every
preexisting registry version/checksum; additional registry packages already belong
to the parent UI lock. Root serializes locked offline Cargo checks, boundary
snapshots, context, facade-consumer checks and scoped line caps. No existing
registry package is upgraded.

Captured-image construction, alias/serializer temporary heaps, imported bytes,
accepted-output/source-fact copies, decoded vectors, metadata, House
compression/transport and downstream allocations remain separate. This slice
does not establish whole-heap accounting, the 100,000/million-record axes,
whole K6 completion or repair of the original allocation abort.

Actual verification: 59 runtime/doc tests passed across fixed byte backing, process
policy, finite ancestors, native regions, Query wire/capture, transition repair and
producer reopen. The external topology consumer checked successfully. All seven
locked offline metadata checks, boundary snapshot/check, agent context and scoped
Rust line caps passed. The final diff check passed after correcting two inserted
manifest line endings. These results do not close the separate scope listed above.

## Checkpoint fact framing follow-on

The implemented follow-on deletes the separate 65,536-total-fact quota from
[fact encoding and decoding](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_checkpoint/facts.rs)
and the [producer/performed-output merge](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_checkpoint/capture/output_facts.rs).
There is no replacement total count quota or optional count control.

The u32 wire count remains a checked representation constraint. Decoding must
still reject a count unsupported by the actual remaining input and validate every
fact and complete payload framing. Existing payload-byte, text/value-byte,
per-selection membership/comparison and actual work controls remain independent.
Checked merge arithmetic, request liveness and selected allocation custody remain;
removing the total quota does not admit malformed input or establish whole-heap
custody, 100k/million success or the original allocation-abort cause.

Seven existing fact-wire tests passed in 0.04 seconds, including the literal
65,537-fact complete round trip and forged-payload refusal. The existing leased
checkpoint frame/clone/native-handoff/reopen proof passed in 0.22 seconds; all
eight Host transition tests passed again in 1.66 seconds. The two native
allocation/wire proofs passed unchanged earlier. Boundary and context checks,
scoped formatting, Rust line caps and composition checks passed across 25 Rust
files. Function review retained 21 advisories with zero hard failures and reviewed
the actual changed bodies. The compiler-profile exception above remains explicit;
these scoped results establish neither whole-heap custody nor K6 scale completion.
