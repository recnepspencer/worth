# Execution workers and retained custody

Use `ExecutionAuthorityConfig::for_workers(width)` to construct the process's
execution service, and `ExecutionRequestPolicy::for_workers(posture, determinism,
width)` to choose request placement. Construct the service at process startup
and share its `Arc` with each application and reopened session. Handle
`ConstructionDenial` at that startup boundary. The
[application composition guide](../../docs/build-an-application.md#one-process-execution-service)
shows the complete installation sequence.

Serial placement executes on the calling thread. Automatic placement uses the
process pool when the request admits it. Child requests inherit the parent's
worker width, cancellation and deadline; a narrower child can reserve a subset
of that concurrency.

Use `ExecutionRequest::run_native` to execute a native pattern with the request's
lease. Use `run_native_with_checkpoint` when the callback needs
`KernelContext::poll` between dependent stages. Poll at safe interruption points,
then propagate the returned failure to the caller. Request handoff checks the
original controls before returning a successful value. Pattern settlement
retains canonical result order.

Both methods return the callback's value and an `ExecutionPhysicalReport`.
Inspect that report for observed placement and worker activity. Handle admission,
interruption and allocation failures through their typed result at the call that
owns them.

For algorithms that need work measurement or an explicit work/memory bound,
construct the policy with `ExecutionRequestPolicy::new(...,
ExecutionBudget::new(...))` and use `ExecutionRequest::run(ceiling, ...)`.
`policy.accounting()` exposes that contract; `policy.max_workers()` gives the
admitted width. Bounds supplied by the process or an ancestor apply to its
descendants. `ExecutionReport::accounted_cost()` returns the measured work/span
when the run has this contract. Use the physical report to inspect placement.

## Fixed payload custody

Fixed payload builders allocate a declared contiguous layout and keep its
storage alive through the returned owner. For accounted execution,
`charged_memory_bytes: Option<u64>` configures a process payload ceiling:
`Some(bytes)` enforces that value, including zero, and `None` leaves this
particular ceiling unspecified. Ancestor bounds also apply.

`ExecutionResourceLease::controlled_child(cancellation, deadline)` retains the
parent's exact request policy and uses the earlier parent/caller deadline.
It adds the supplied cancellation token to the existing ancestor constraints
without mutating the parent. A payload reservation owns its ledger custody;
dropping this child does not release storage still owned by an array or buffer.

Pass `ExecutionAllocationPolicy::SystemAllocation` or `Execution(&lease)` to
byte and typed array builders. The builder checks
`Layout::array::<T>(element_count)` and the request's cancellation/deadline,
then attempts a fallible allocation. Accounted leases also admit the exact
payload-byte reservation. For non-zero-sized elements, the fresh allocation
requests that layout and records exactly that capacity.
Zero-sized types have allocator capacity `usize::MAX` but zero payload bytes;
the independently retained logical count still bounds initialization and seal.
Over-aligned zero-sized consuming iteration keeps the original Vec and uses
safe pop in either direction; such values have no distinct stored
representation. Non-zero-sized iteration uses the pinned owning Vec iterator.

`ExecutionByteBuffer::allocate(total, policy)` retains byte authoring: append
within the declared length, overwrite only written regions, and poll live
status between 64 KiB copy chunks. Sealing requires exact fill, shares the same
backing without shrink or reallocation, and exposes immutable bytes. Clones of
`ExecutionImmutableBytes` share one allocation and any original noncloneable
reservation.

`ExecutionArrayBuilder::<T>::allocate(element_count, policy)` moves each value
through `push` and seals only after exact fill and a live-status check. Push
consumes the input, including on refusal; a stop detected after the move may
leave an initialized prefix which cannot seal. Share a completed array through
`Arc<ExecutionArray<T>>`; clones share the original elements and allocation.
The immutable slice preserves authored order and T's own interior mutability.
Consuming iteration moves T without allocation, preserving the original
backing and any reservation even after exhaustion until the iterator drops.

Elements and their allocation drop before any reservation. Storage can outlive
the originating lease or World. Supply a current request for later execution.
Moved-out array values own their separate nested resources. When accounted, the
physical owner charges only the contiguous element layout, not allocations
nested inside T.

Platform storage owners call `policy.reserve_memory(bytes)` before allocating
or copying newly owned nested payload. Handle its typed admission failure and
retain the returned `Option<ExecutionMemoryReservation>` with the payload.
`Some` holds an accounted reservation. An `Execution(&lease)` policy also checks
the request's live controls. The payload owner places its reservation after its data fields so
the data drops first, and shared roots keep the complete owner alive. A cached
copy bound must cover every newly copied nested allocation; shared existing
owners keep their original custody.

`ExecutionImmutableBytes::from_external_bytes(Arc<Box<[u8]>>)` retains already
allocated external bytes without copying or retrospectively admitting them.
`charged_payload_bytes()` returns `None` for worker-only, external and system
storage. Accounted execution storage reports its retained reservation amount,
including `Some(0)` for empty or zero-sized payloads. Equality compares values.
Query checkpoint capture uses this same allocation policy.

Allocator, Arc and ledger metadata, nested element heaps, native checkpoint
image/codec allocations, imported buffers, decoding tables, upstream domain
maps/sets, temporary provider collections and compression/transport have
separate allocation owners. Use the contiguous-backing reservation to measure
the payload managed by these builders; use a process memory profiler for RSS.


## Explicitly accounted kernel cost receipts

Platform runtimes retain `ExecutionKernelReceipt` from
`MapKernelContext::record_kernel(&storage, kernel)`. The callback executes on
that same native context; its successful accepted checkpoints, not caller
supplied units, create the receipt. `ExecutionKernelReceiptStorage::new` takes
an existing genuine `ExecutionMemoryReservation`. Capture admits fresh backing
before allocation, including simultaneous old/new backing during growth. Clones
retain the original backing and funding; dropping the last receipt releases it.
The retained-byte measurement includes the receipt and shared storage wrappers.

`context.carry_kernel(&receipt)` consults the current native request. It uses the
cached cost when the whole receipt fits and descends the original accepted
checkpoint sequence when the ceiling falls inside it. Work, span and the first
checkpoint refusal therefore follow actual execution. Cancellation/deadline and
untaken nested/request stops remain the existing meter's decisions.

Use receipts for checkpoint-only kernels. Kernels that inspect remaining work,
query absolute cost, join nested outcomes or carry another stage require fresh
execution. Receipt issuance requires successful kernel completion.

Receipt equality compares cost sequences. Before carrying a receipt, the
application's retained-computation owner checks current input identity,
installation and source dependencies. That owner chooses the inputs to carry
and retains publication authority.

## Retained native results

Construct native partitions and call `run_retained_run` to retain per-partition
values, or `run_reduce_retained_native` to retain a canonical reduction.
`advance_native` reuses immutable cells whose inputs the caller has qualified
and evaluates changed inputs on a fresh request. Supply the current request
for each advancement and validate input identity and freshness before reuse.
An accounted run also exposes its measured cost through `accounted_cost()`.

The following lower-level examples use explicit accounted payload contracts.

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
    map.run_retained(request, ExecutionWorkCeiling::new(Some(100)), |input, context| {
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
    plan.run_retained_inputs(request, ExecutionWorkCeiling::new(Some(100)), None,
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
canonical reduction returns its tree and genuine native custody in `run`.
Keyless maps receive `Some(ExecutionRetainedRun)` from their actual native
settlement. Access-key maps still complete their reduction but receive `None`;
their declarations require full checking on later calls. There is no conversion
from the public reduction DTO, an arbitrary result vector or a report to a run.
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

Native runs retain immutable value cells and structural range summaries.
Advancement carries unchanged ranges and evaluates changed inputs in canonical
order under the current request's controls. `RetainedRunMetrics` reports supplied
inputs, edit/range visits, copied nodes and individually evaluated carried slots.
Use these counts to inspect the maintenance performed by an advancement.

The accounted example above retains cached costs and backing reservations.
A fitting accounted range uses its cost summary; a tight
ceiling descends to the original slot checkpoints. Its fresh request re-admits
incoming backing, and each copied node owns its new grant. Both modes share
unchanged nodes without retaining a chain of parent runs, and refusal preserves
the prior run. Express removal and insertion through their respective change
collections; express a move as remove-plus-insert. Returned runs own their
backing. Create a fresh request for each call.

Choose keyless partitions for this advancement API. An unchanged slot whose
inputs the caller has yet to qualify returns `RequiresExecution`; gather its
current inputs and execute them. Cold issue and recovery evaluate the full input.
For persistence, capture the application's checkpoint and restore through its
normal installation API. Query's partitioned computation retains
the run issued by successful reduction and advances it with a fresh descendant
request, changed inputs and actual removals. Its source, installation, input,
edition and read closure qualify reuse. Query also maintains provider/session,
effects and source-publication metadata. Membership and key changes require full
preparation; membership changes and explicit tree rebuilds construct a full
shape plan.

## Funded ordered storage

`ExecutionOrderedMap<K, V>` stores platform-owner metadata in canonical `K: Ord`
order. `Clone` shares the root without allocation. Native callers use
`insert_cloned_native_in(&key, &value, policy)` and `remove_in(&key, policy)`;
the native insertion requires `Clone`. Edits check the current request and
construct the new path with checked, fallible allocation. A refused edit leaves
the previous root usable.

For explicitly accounted payload bounds, `insert_cloned_in` requires
`Clone + ChargedBytes` and admits the new key/value heap bound before copying.
Accounted entries and path nodes retain their actual grants through final drop;
shared views retain those owners without predecessor-root chains. Use
`accounted_bytes()` for the optional reachable-storage information. A populated
worker-only map returns `None`; an empty map returns `Some(0)`.
Use `retained_bytes()` when the map has an accounted storage contract.
`get`, `first_at_or_after` and canonical
`iter`/`keys`/`values` borrow that backing. `rank_of(&key)` and
`entry_at(rank)` navigate cached subtree counts. The application retains source,
session and installation checks around access to these values.

`KeyedPartitioner::upsert_in`, `remove_in` and explicit full `kept_in` use this
same storage with the current allocation policy. Edits update membership and
structural counts. Accounted changed groups reserve their copies before
allocation; unchanged groups retain their original reservations.

`ReductionTree::checked_full_build_cost_after_edits(&removed, &inserted, policy)`
predicts the work/span of a cold full build from an accounted tree's shape.
It allocates temporary changed-path metadata during the Cartesian edit traversal.
Handle `ReductionShapeCostDenial` for allocation, interruption, invalid identities,
arithmetic overflow or unavailable accounting. `checked_full_build_cost()` returns
an optional measurement for the existing shape. Use the normal reduction methods
to compute the result after selecting the execution policy.
