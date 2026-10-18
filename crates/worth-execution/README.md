# Execution fixed payload custody

The composition root constructs the process execution authority once. Its
`charged_memory_bytes: Option<u64>` selects an optional ceiling for execution
payload reservations. `Some` preserves the selected value, including zero;
`None` keeps checked ledger arithmetic while omitting only that process ceiling.
Every request and ancestor retains its explicit finite budget. This does not
make any algorithm budget unbounded or intercept process allocations.

`ExecutionResourceLease::controlled_child(cancellation, deadline)` retains the
parent's exact request policy and uses the earlier parent/caller deadline.
It adds the supplied cancellation token to the existing ancestor constraints
without mutating the parent. A payload reservation owns its ledger custody;
dropping this child does not release storage still owned by an array or buffer.

Byte and typed array builders require the same explicit
`ExecutionAllocationPolicy::SystemAllocation` or `Execution(&lease)`. There is
no default, pool construction or fallback. A supplied process lease admits
physical storage; it grants no graph/model permission. The private fixed owner
checks `Layout::array::<T>(element_count)`, checks cancellation/deadline, admits
the exact payload-byte reservation, then attempts a fallible fresh Global
allocation using pinned allocator-api2 0.2.21. For non-zero-sized elements that
implementation requests that layout and records exactly that capacity. This
guarantee is scoped to the pinned fresh allocation, not generic/std Vec behavior.
Zero-sized types have allocator capacity `usize::MAX` but zero payload bytes;
the independently retained logical count still bounds initialization and seal.
Over-aligned zero-sized consuming iteration keeps the original Vec and uses
safe pop in either direction; such values have no distinct stored
representation. Non-zero-sized iteration uses the pinned owning Vec iterator.

`ExecutionByteBuffer::allocate(total, policy)` retains byte authoring: append
within the declared length, overwrite only written regions, and poll live
status between 64 KiB copy chunks. Sealing requires exact fill, shares the same
backing without shrink or reallocation, and exposes immutable bytes. Clones of
`ExecutionImmutableBytes` share one allocation and its noncloneable reservation.

`ExecutionArrayBuilder::<T>::allocate(element_count, policy)` moves each value
through `push` and seals only after exact fill and a live-status check. Push
consumes the input, including on refusal; a stop detected after the move may
leave an initialized prefix which cannot seal. Neither the builder nor
`ExecutionArray<T>` implements Clone, exposes mutable storage, accepts an
arbitrary Vec, or extracts a reservation. Sharing is explicit through
`Arc<ExecutionArray<T>>`; it neither clones T nor charges another allocation.
The immutable slice preserves authored order and T's own interior mutability.
Consuming iteration moves T without allocation, preserving the original
backing and reservation even after exhaustion until the iterator drops.

Elements and their allocation drop before the reservation. Custody can outlive
the originating lease or World; it grants no further permission to execute.
Moved-out array values own their separate nested resources. The physical owner
charges only the contiguous element layout, not allocations nested inside T.

Platform storage owners reserve newly owned nested payload with
`policy.reserve_memory(bytes)` **before** allocating or copying it. Execution
returns the genuine `ExecutionMemoryReservation`; explicit system allocation
returns `None`. The method performs no allocation or retrospective funding.
It uses the same live-status checks, typed refusal and payload quote as fixed
backing. The payload owner retains that reservation after its data fields so
the data drops first, and shared roots keep the complete owner alive. A cached
copy bound must cover every newly copied nested allocation; shared existing
owners keep their original custody.

`ExecutionImmutableBytes::from_external_bytes(Arc<Box<[u8]>>)` retains already
allocated external bytes without copying or retrospectively admitting them.
`charged_payload_bytes()` returns `None` for external/system storage and the
actual retained ticket amount for execution storage, including `Some(0)` for
empty or zero-sized payloads. Equality compares values, never admission mode
or allocation identity. The Query checkpoint capture policy remains a semantic
re-export of this one physical policy, with no independent behavior.

Allocator, Arc and ledger metadata, nested element heaps, native checkpoint
image/codec allocations, imported buffers, decoding tables, upstream domain
maps/sets, temporary provider collections and compression/transport remain
separate owners. Only callers that explicitly admit this contiguous backing
retain its charge. This is neither full heap/RSS accounting nor removal of
domain count gates.


## Native kernel cost receipts

Platform runtimes retain `ExecutionKernelReceipt` from
`MapKernelContext::record_kernel(&storage, kernel)`. The callback executes on
that same native context; its successful accepted checkpoints, not caller
supplied units, create the receipt. `ExecutionKernelReceiptStorage::new` takes
an existing genuine `ExecutionMemoryReservation`. Capture admits fresh backing
before allocation, including simultaneous old/new backing during growth. Clones
retain the original backing and funding; dropping the last receipt releases it.
The quoted retained bytes include the receipt and shared storage wrappers
conservatively, without a global deduplication promise.

`context.carry_kernel(&receipt)` consults the current native request. It uses the
cached cost when the whole receipt fits and descends the original accepted
checkpoint sequence when the ceiling falls inside it. Work, span and the first
checkpoint refusal therefore follow actual execution. Cancellation/deadline and
untaken nested/request stops remain the existing meter's decisions.

These receipts currently admit checkpoint-only kernels. A remaining-work or
absolute-cost question, nested outcome, completed-work accounting, or another
carried stage prevents receipt issuance. Such a kernel executes normally and
leaves no reusable retained state; a context-dependent result is never certified
by summing its checkpoints. A failed/panicking kernel also issues no receipt.

A receipt grants no input equality, graph currentness, installation identity,
output correspondence or publication authority. Equality compares only cost
sequences. Query's existing retained-computation owners must independently prove
which kernel inputs may be carried. The current map still dispatches all slots;
this cost owner is a prerequisite to sparse retained-map settlement, not that
capability or whole-computation locality.

## Retained native results

Platform runtime owners use `ExecutionMap::run_retained` when later execution
needs the actual successful value as well as its native meter capture. Query
applications continue to use the declaration and host facades; Query qualifies
their source, inputs, snapshot, installation, edition and authorization.

`ExecutionRetainedResult<R>` owns one accepted value, its original native
checkpoint capture when eligible, and its surviving backing grants. Native
evaluation checks the declared deep result bound once; retained custody consumes
that same checked bound. There is no constructor from an arbitrary value,
receipt, report, digest or success flag. The native map settles identities in
canonical order before issuing handles; a stopped suffix issues none.

```rust
use worth_execution::{
    ExecutionMap, ExecutionRequest, ExecutionRetainedResult, ExecutionWorkCeiling,
    MapKernelFailure, MapOutcome, RetainedMapDenial,
};

fn evaluate(
    map: &ExecutionMap<u64, u64>,
    request: ExecutionRequest<'_, '_>,
) -> Result<MapOutcome<ExecutionRetainedResult<u64>, ()>, RetainedMapDenial> {
    map.run_retained(request, ExecutionWorkCeiling::new(100), |input, context| {
        context.checkpoint(3)?;
        Ok::<_, MapKernelFailure<()>>(*input * 10)
    })
}
```

The request is borrowed for this call. Its lease, session and run limits are
not stored in a result. `value()` borrows the actual accepted value; cloning a
handle shares that value and its grants without cloning R or retaining a parent
batch. Each unrelated result can drop independently. The final handle releases
its value before its funding. R retains responsibility for its own interior
mutability and nested ownership contract.

### Carrying a value with a current input

After the application owner has independently qualified the original inputs,
use `RetainedMapInput::Carry` for that same partition identity and `Evaluate`
for a freshly gathered input. The native backend performs the carry itself.
This example assumes the prior result belongs to partition 1; partition 2 has
a new input. Planning and runtime refusals remain distinct.

```rust
use std::collections::BTreeMap;
use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionRequest, ExecutionRetainedResult,
    ExecutionWorkCeiling, KeylessPartition, MapKernelFailure, MapMemoryOverflow,
    MapOutcome, RetainedMapDenial, RetainedMapInput,
};
use worth_foundational::PartitionIdentity;

enum UpdateError {
    Plan(MapMemoryOverflow),
    Run(RetainedMapDenial),
}

fn update_two(
    prior: ExecutionRetainedResult<u64>,
    changed: u64,
    request: ExecutionRequest<'_, '_>,
) -> Result<MapOutcome<ExecutionRetainedResult<u64>, ()>, UpdateError> {
    let carried_bytes = prior.additional_charged_bytes();
    let plan = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([
        (PartitionIdentity::new(1), KeylessPartition {
            value: RetainedMapInput::Carry(prior),
            kernel_scratch_bytes: 0,
            max_result_bytes: carried_bytes,
        }),
        (PartitionIdentity::new(2), KeylessPartition {
            value: RetainedMapInput::Evaluate(changed),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }),
    ])).map_err(UpdateError::Plan)?;
    plan.run_retained_inputs(request, ExecutionWorkCeiling::new(100), None,
        |input, context| {
            context.checkpoint(3)?;
            Ok::<_, MapKernelFailure<()>>(*input * 10)
        },
    ).map_err(UpdateError::Run)
}
```

Carried inputs retain the same value cell. A fitting capture takes a real
checkpoint on the fresh request; a tight ceiling descends the original native
checkpoint prefix. The existing canonical settlement, speculative suffix,
first-stop and request latches decide the outcome. Cancellation/deadline
observed at the final boundary also latch on an invoking kernel, even if it
handles the returned error.

Context questions, nested stages or capture refusal can leave a successful
value without reusable native cost. `native_reuse_available()` describes that
capture only. An unqualified or differently keyed carried result returns
`RequiresExecution` before any new kernel runs. Gather the real current input
and evaluate it; a cached value cannot reconstruct that input. `try_carry` is
available inside a native kernel and returns `None` for this same fallback.

### Canonical reduction and inspection

`run_reduce_retained_holding` returns a `RetainedReduceOutcome`. Only a complete
canonical reduction returns its tree and genuine native partition results.
Pass the caller's input and tree reservations for atomic handoff, as with
`run_reduce_holding`; a retained tree still needs its tree reservation. A tree
refusal publishes no successful reduction. Map reports and reduction metrics
remain available from their actual completed stages, so observers do not ask
live kernels about remaining work and accidentally change capture eligibility.

Inspect `identity()`, `charged_work()`, `charged_span()` and the returned reports
for their respective native meanings. `has_same_native_cost` compares meter
captures only; it is not value equality or freshness. A cost receipt alone
cannot create a retained result. Content equality cannot replace the owning
runtime's source, authorization, edition or settlement checks.

These explicit-map APIs still admit and dispatch every supplied slot. They do
not avoid caller roster construction, update collection membership, or maintain
a retained reduction tree by delta.
Allocation bounds include live old/new overlap and do not promise global
deduplication, full heap/RSS accounting or persistence of execution handles.
Current-format recovery reconstructs native execution through its owning
runtime rather than decoding a fabricated result handle.

### Persistent native runs

A platform owner with independently qualified input changes can retain a
keyless native run. Cold issue uses the real full map; later attempts supply
only changed inputs and canonical removed identities. The owner must check
source, authorization, installation, edition, snapshot and settlement before
omitting an input. Native result custody establishes none of those facts.

```rust
use std::collections::BTreeMap;
use worth_execution::{ExecutionMap, ExecutionRequest, ExecutionWorkCeiling,
    KeylessPartition, MapKernelFailure, RetainedRunOutcome};
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

fn example(initial_request: ExecutionRequest<'_, '_>, fresh_request: ExecutionRequest<'_, '_>) {
let id = PartitionIdentity::new(1);
let initial = ExecutionMap::<_, ()>::from_keyless_partitions(BTreeMap::from([
    (id, KeylessPartition { value: 7_u64,
        kernel_scratch_bytes: 0, max_result_bytes: 0 }),
])).unwrap();
let first = initial.run_retained_run(initial_request, ExecutionWorkCeiling::new(100),
    |value, context| {
        context.checkpoint(3)?;
        Ok::<_, MapKernelFailure<()>>(*value * 10)
    }).unwrap();
if let RetainedRunOutcome::Complete { run: prior, .. } = first {
    // Obtain this request afresh from the current host call. No request is
    // stored in `prior`; this example only shows the borrowed call shape.
    let changes = BTreeMap::from([(id, KeylessPartition { value: 9_u64,
        kernel_scratch_bytes: 0, max_result_bytes: 0 })]);
    let removed = CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap();
    let next = prior.advance(fresh_request, ExecutionWorkCeiling::new(100),
        changes, removed, |value, context| {
            context.checkpoint(3)?;
            Ok::<_, MapKernelFailure<()>>(*value * 10)
        }).unwrap();
    match next {
        RetainedRunOutcome::Complete { run, .. } => {
            assert_eq!(*run.result(id).unwrap().value(), 90);
        }
        RetainedRunOutcome::Stopped { completed_prefix, boundary, .. } => {
            // Inspect accepted values; a prefix has no advance method.
            let _ = (completed_prefix.partition_count(), boundary);
        }
    }
}
}
```

Unchanged slots keep the exact funded native value cells. Cached work/span and
range summaries belong to the persistent native tree. Fitting ranges skip
individual dispatch; a tight range descends to native slot evaluation and its
original checkpoints. Changed inputs use the existing scheduler, evaluator
and canonical settlement, including request ownership of a discarded suffix.
`RetainedRunMetrics` reports actual supplied inputs, edit/range visits, copied
nodes and individually evaluated carried slots separately from logical work.

Each copied node owns its backing grant; old/new roots share unchanged nodes
without retaining a chain of parent runs. A fresh request re-admits the cached
incoming backing, and refusal borrows and preserves the prior run. Removal,
insertion and a move expressed as remove-plus-insert use this same path. No
request/session borrow survives the call. Context-sensitive results can be
inspected after completion, but an unqualified unchanged slot returns
`RequiresExecution` before a new kernel runs; gather current inputs and execute
them. Arbitrary declared access-key maps do not qualify for sparse keyless
advancement. Cold issue and recovery remain full; no execution handle is a
persisted recovery record. Query's roster, publication and membership cutover
is a separate downstream obligation.
