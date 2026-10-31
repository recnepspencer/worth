# Output-sized candidate collection

## Boundary review

At85c2a7a5c0, CandidateWriter owns a dense SDK Cell<bool> array for Completed
Sources and an im::OrdSet for Maintained Sources. The dense path allocates,
initializes and later scans Source.len even when a checked root is pruned. It
also validates owner/ordinal without rejecting entity:Some on Completed rows.
The authoritative SourceImageView.validate_slot already owns all shape,
provenance and membership checks. Canonical Slot Ord means Completed ordinal
order and Maintained full EntityId order. Neither candidates nor slots grant
Source reads, membership proof or disclosure; the existing receiver and original
parent selection consume the finished array.

Use one output-sized SDK ExecutionOrderedMap<Slot,()> in the writer, under its
existing original request and control. Existing access_strategy::ordered_storage
owns paid lookup/insert bounds and actual SDK cell-grant admission. Remove both
the dense bitmap and the independent im-set/SourceFactsAllocation/count path;
there must be one growth/validation/ordering path. Every emit validates the actual
borrowed Source image before probing the selected map, including duplicates.
Finish allocates exactly selected.len and iterates only those keys; no Source
enumeration is necessary. The physical ordered map drops its original grants
with its actual cells. No new ledger, test authority or provider capability.

Current public checked/exhaustive fixtures cannot directly test duplicate/reverse
emits because Query owns traversal; provider.query is bypassed. A legitimate
Candidates-profile consumer, sharing the existing authored catalog/schema/provider
files with its own compiled binding, can use real Source slots without weakening
checked coverage or forging coordinates. Existing publicBox remains the genuine
Completed checked-pruning receiver. Unavailable keys must not be invented; actual
negative controls may retain a genuine foreign Source slot from another completed
invocation. The existing private validate_slot remains authoritative for bounds.

## Implementation plan

1. Replace CandidateWriter storage with the SDK ordered map and actual Source
   image. Use existing paid lookup/insert before each map probe. Delete bitmap,
   independent sparse branches and redundant count. Preserve sticky controls.
2. Add a legitimate public Candidates-profile fixture using existing catalog
   authoring/schema modules. Its provider query emits real matching slots in
   reverse order and repeatedly, checks len/empty on the same writer, and can
   emit a genuinely retained slot from another Source invocation as a hostile
   foreign-owner control. No synthetic entity/Source owner IDs or mutable proof.
3. Check exact public query rows against ordinary Source scan sorted by actual
   full issued identity, repeated/equal-name results, empty output and genuine
   foreign-slot refusal with a fresh lawful query on the same ready index.
4. Run narrow new consumer courts, then actual public managed_index family
   excluding unchanged large50k. ExistingBox empty-root-pruning and maintained
   results protect both Source representations. Do not claim all Source admission
   or coverage work is output-sized: this repair is candidate accumulation only.
5. Run mandatory boundary/context, dirty caps/function scrutiny, rustfmt/diff and
   compiled-document drift as needed. Obtain independent qa-loop/qa-tests/code-
   quality-qa clearance and commit. Other audit repairs/milestone phases stay open.

### Verified empty-allocation seam

Independent review identified an existing real callback bracket: validate_probe
runs immediately before candidate allocation, and the first checked box summary
callback immediately after it and before frontier insertion. Coverage traversal
uses only scalar cursor state there. Add fixture-owned one-shot read-only callbacks
at these actual points and sample the original SerialMemoryBudget under its real
ExecutionWorkCeiling. For a genuine nonempty Completed box Source pruned at its
root, require no added retained bytes across this interval. The old dense array
must fail. This is interval live retention, not a whole-query complexity claim;
no new framework observer or authority door is needed.

### Shared consumer module placement

The first public-profile compile confirms Rust's #[path] file module resolution
uses the containing directory for boxes.rs's implicit children. Move that one
root declaration mechanically to boxes/mod.rs, within the same existing owner,
so both genuine compositions share its actual child directory without copying
schema/provider declarations or accidentally loading unrelated lifecycle code.
No child domain is reparented. The original module declaration still resolves
normally, while the new profile references the exact boxes/mod.rs.

The shared box courts use the existing lifecycle allocator/run harness. Expose
only those existing fixture utilities to the containing application-invariant
fixture, and provide Candidates-profile authentication against its actual Catalog.
There remains one actual global allocator. Fix the box provider's hard-coded
normal Catalog import to its real enclosing schema. Public row oracles use the
Parent's declared identity-field order; internal slot order follows actual map
Ord and is not inferred from public sorting.

### Falsified first memory witness and correction

The first ambient-work SerialMemoryBudget sample was [392,392] for BOTH new and
exact previous dense implementation; it does not establish candidate allocation.
Do not count that witness. The actual Host execution policy has no memory ceiling.
Use the SAME installed Host composition with a deliberately finite actual request
memory policy, and existing ExecutionMemoryReservation::available_in_scope(None)
at the same two callbacks. Require actual finite room observations and unchanged
room for sparse allocation; the exact dense regression MUST fail before accepting
this court. Remove the displaced ambient SerialRequest/factory exception rather
than retaining an unused separate payer. This is active original allocation-room
observation, not an ambient work invoice or reconstructed byte counter.

The existing scope-room observation is unavailable at the actual leased callback,
even with finite Host policy, because available_in_scope(None) cannot borrow the
lease held by Query. Do not add a public authority/observer API for this test.
Revert the experimental Host policy wrapper. Use the established single-allocator
isolated-allocation court harness instead: sample actual StatsAlloc snapshots
at those same two provider boundaries into a fixed stack-sized two-cell record.
Require zero physical allocations/bytes allocated across the sparse interval;
use a separate process to exclude other courts' traffic. Label this physical
allocation evidence, not original-payer retention. Require the exact old dense
implementation to fail the unchanged physical guard. Actual payer/CPU admission
continues to be owned by SDK ExecutionOrderedMap and the existing paid ordered
storage wrapper; no new ledger or authority lane is introduced.

## Implementation and measured evidence

CandidateWriter now uses one SDK ExecutionOrderedMap<Slot,()> for both Source
representations. Every emit validates the actual Source image, pays the existing
ordered probe/insert bounds, and relies on actual SDK cell-grant admission. The
bitmap, separate im tree/grant, redundant count and now-unused SourceRows.owner
forwarder are removed. Finish allocates selected.len and visits only those keys.
Source/proof admission, per-use region checks, ordinary refinement/disclosure and
Parent ordering stay with their existing owners.

The new genuine Candidates-profile court passes reverse real Source emissions,
three repeated emits per match, len/empty checks, equal-name distinct rows, actual
Parent identity-field order, empty output, a genuinely retained foreign Source
slot refusal and a fresh lawful query on the same Ready index. No Source slot or
physical entity identity is manufactured. It reuses the authored consumer modules
and existing physical lifecycle harness, with its own installed schema/principal.

The accepted memory-adjacent witness is PHYSICAL ALLOCATION, not payer retention:
the established subprocess-isolated global allocator observes zero allocations
and zero allocated bytes between actual probe-validation and first checked-root
summary callbacks for a genuine nonempty Completed box Source, with one root
summary and no Source matches. The exact HEAD85c2a7a5c0 dense candidates.rs control
fails BOTH compositions with22allocations and1,158allocated bytes in that same
unchanged bracket. The sparse production file is restored byte-for-byte. Initial
ambient-budget [392,392] and unavailable leased-room attempts are NOT acceptance
evidence; no experimental policy wrapper, factory exception or new SDK API remains.
The bracket proves allocation through first root prune; selected-key-only finish
is a separate structural code property, not a whole-query complexity measurement.

Final restored public managed_index family passes47/47 in18.90s, excluding
unchanged large50k; the genuine Candidates composition passes9/9 in0.65s.
Both mandatory Cargo boundary/context commands and compiled-document drift1/1
pass. The independent reviewer gives unconditional final clearance under qa-loop,
qa-tests and code-quality-qa.
Boundary/context, dirty400-line caps(12Rust files), function scrutiny(7advisories,
zero scan errors), formatting and whitespace checks pass. Other audit repairs,
churn, prefix-large qualification and remaining milestone phases stay open.
