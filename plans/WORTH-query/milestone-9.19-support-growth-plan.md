# Geometric growth of actual Source support

## Boundary review

At HEAD58032cbcb4, scope_closure/projections::reserve_support grows exactly and
adds the entire new length quote to an additive original meter. Earlier support
producers are also not capacity-funded: affected::expression retains one element
per Vec push; affected::collect retains combined length before implicit extend;
scope_closure adopts those vectors and later appends individual inverse facts.
Changing only the final reserve helper would leave an unfunded initial-capacity
assumption and multiple competing ordinary growth paths.

Put one concrete DerivedRelationData support push/append/reserve helper under
its existing derived_relation owner. Every producer and merger uses it, including
scope closure's final per-row inverse support. Preserve all repeated before/after
and empty inverse support occurrences; this is allocation amortization, not
semantic deduplication. Empty-vector custody moves do not create a new payer.

SourceRead already carries original retention admission. Add a private temporary
peak operation to that same port. StructuralSource delegates to the actual
CustomInvariantReadBudget and CustomInvariantWorkMeter.check_scratch_peak;
ChangeSource/LifecycleSource forward it. PinnedSource must use its own original
reservation/current retained total; no fake/no-op default or new ledger.

For capacity C and requirement R>C, choose at least max(2C,R,4), with checked
layout arithmetic. Admit the capacity delta as persistent backing, then the OLD
capacity as extra temporary overlap before reallocating. The original meter's
peak operation reserves without increasing cumulative claimed_memory; its ticket
keeps the admitted high water until actual holders drop. This accounts for old
and new buffers together without quadratic historical capacity charges. Exact
reserve preserves the pinned native allocator layout contract; reject unexpected
reported capacity before appending rather than silently accepting unclaimed
backing. All refusal/cancellation remains sticky on the original budget.

Independent review confirms both physical views share the same original Source
stage budget, and requires initial affected support, merges and final closure
pushes to participate. No graph, Source or publication authority is introduced.

## Implementation plan

1. Add the concrete support helper and private SourceRead peak method. Implement
   the actual structural, pinned, change and lifecycle bridges; existing retained
   claims and exact typed refusal classification stay authoritative.
2. Cut over affected-expression pushes, affected forward/reverse merge, projection
   support append/push and final inverse-Source pushes. Delete reserve_support and
   per-element/full-length duplicate capacity quotes. No fallback grows implicitly.
3. Extend the existing real relational certification Source work-account court
   with a finite composed directory-to-item path and actual projection fanout.
   Author a real Native batch, including an unattached row with empty inverse
   support. Observe original payer bytes at the genuine provider boundary. Compare
   modest32/64 populations under a predeclared generous linear-growth envelope;
   do not fabricate a SourceRead or reconstruct an invoice from counters.
4. Exercise genuine finite-memory refusal and parent cancellation with unchanged
   selected Native head, released actual request ownership and fresh funded lawful
   retry. Preserve original Serial/leased/inherited work-account courts and derived
   before/after/empty-support semantics through existing targets.
5. Run the narrow original relational certification target and relevant public
   Source/managed family as justified by the substrate change. Keep default pinned
   compiler/cache/profile and no new private-owner build unless evidence requires
   private observations that public owners cannot provide. Run boundary/context,
   size/function/format/whitespace gates and independent all-three QA review.
6. Commit the verified repair with measured results; keep all other audit items
   and remaining milestone phases open. Do not repeat full geometry50k merely
   for unchanged fixture semantics unless a real regression concern requires it.

## Boundary refinement during cutover

The first narrow compile exposed a fifth private SourceRead implementation:
aggregate_identity::derived_source::StateSource. It borrows the existing
AggregateWork through RefCell and captures original funding failures for return
from with_views. Add its required peak forwarding and a checked temporary-peak
method on that actual AggregateWork/ExecutionResultMemory. Do not increment
claimed, shrink an existing larger peak, or create a replacement payer. Existing
retain may subsequently settle to retained backing only after allocation. This
required port cutover adds no new aggregate semantics or public authority.

## Measured verification

The real relational certification fanout authors32/64 contained self-linked
items plus one unattached self-linked item under a composed Source and direct
projection route. At the actual provider entry, the unchanged original payer
holds320,360/640,872 bytes. These are whole Native-preparation retention values,
not an exclusive support-vector invoice. The predeclared<3x doubling envelope
passes; exact authored issued-ID order and unattached-row exclusion also pass.

A controlled temporary regression changed the concrete helper to exact required
capacity and additive FULL-capacity claims. The same unchanged court measured
1,212,048/4,136,080 bytes and failed the<3x guard. The saved production helper was
then restored byte-for-byte and the real managed_source family passed6/6 in1.01s.
This discriminates the accounting pattern without replacing the original payer
or deriving memory from synthetic counters.

The128KiB case refuses before the participant with the original typed
PolicyMemoryExhausted{requested:130648, admitted:130600, ancestor:0}; it is not
claimed to pinpoint a particular realloc. The cancellation case cancels the
original parent after a genuine Source acquisition and refuses the next actual
acquisition. Both preserve the selected Native head and release failed request
ownership, then commit the identical authored batch on the SAME runtime and
registration using a fresh funded uncancelled request. Successful retained
observations release original/retry memory after their actual holders drop.
The existing before/after/empty-support proposal-audience court passes1/1.

Narrow builds use the unchanged Rust1.99.1/default sccache and target. Boundary,
agent context, dirty Rust400-line caps(16files), function scrutiny(14advisories,
zero scan errors), formatting and whitespace checks pass. The final restored public managed_index consumer family passes46/46 in18.22s
(excluding unchanged large50k); the independent reviewer gives unconditional clearance under qa-loop, qa-tests
and code-quality-qa. All other
audit repairs and remaining milestone phases remain open.

## Broader consumer baseline qualification

An accidentally broader public command selected143 tests(excluding unchanged
large50k) and passed141 in29.74s. It exposed two unrelated existing proposal
cleanup failures: partial_attempt_disposal_retries_actual_retired_cleanup_until_
its_reader_releases at disposal.rs117 with NestedStopped, and previous_head_
cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again at
previous.rs131 with cleanup not settled. Both exact tests were rerun individually
with every scoped Relational production change replaced temporarily by HEAD
58032cbcb4, and BOTH failed identically at the same line/cause. All changed
production files were then restored byte-for-byte. Their unchanged proposal host
has no managed Source/derived registration; the independent reviewer confirmed
this is existing repository debt, not part of this Source repair. The143 suite
is explicitly not green. Logs are query919-support-consumer.log and
query919-support-baseline-{partial,previous}.log outside the repository.

Both mandatory Cargo boundary/context commands also pass against the final scoped files.
