# Main-thread layout batch (design, not implemented)

The ordinary Windows examples require a genuine 1 MiB main stack in the
committed dev profile. Delivery-track H10 requires those entries to pass;
closing every frame/future ceiling remains C2. No Native storage/checkpoint/root
or World/Lifecycle/transition file is in this implementation batch.

## Measured boundary

Root's three retained owner units passed once, Query's three passed once, and
Store's two passed once. They print 1,787 concrete size observations, including
field types, without constructing runtime values or authorities. Scalar generic
instances are labeled by their actual compiler type name. They are not claims
about every application instantiation. The real consumer wrapper/return sizes
and private generic layout diagnostic are still being collected; the final
inventory must preserve that distinction.

The measured common carriers are:

| Carrier | Dev unit bytes | Dominant inline storage |
|---|---:|---|
| SignalRuntime<(),(),(),()> | 15,552 | checkpoint 4,000; event bus 4,048; telemetry 3,856 |
| BridgeOwnedSignalRuntime | 17,728 | optional SignalRuntime under a mutex: 15,568 |
| BridgeSealedRuntimeAssembly | 20,432 | owned Signal runtime assembly |
| BridgeAssemblyRetirement | 39,376 | prepared and stopped Signal allocations, each 15,552 |
| SignalConditionalExecutionSlot | 11,480 | complete evaluation partition |
| SignalConditionalEvaluationState | 11,520 | optional slot, even while absent |
| SignalConditionalEvaluationAdmission | 11,928 | mutex/state: 11,528 |
| BridgeConditionalEvaluationSession | 11,960 | admitted Signal evaluation |

Query also carries a 69,136-byte admitted legacy read intent and a 25,792-byte
running application commit. Store's recovered core is 65,632 bytes. These are
reported; they are not all selected for representation changes in this batch.

## Representation changes

1. At `SignalRuntime`'s definition in
   `crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs`,
   make `checkpoint`, `event_bus` and `telemetry` owned boxes. Keep all public
   methods and the runtime's ownership shape. Do not add Deref, a public backing
   type, a second runtime wrapper, or a new phase proof.

   These are three fixed installation-lifetime allocations. Allocate them while
   assembling the runtime, before it enters service, alongside the existing
   owned graph box. They do not allocate on an application mutation or during
   close. Snapshot/capture continues to copy the same telemetry value and clone
   the same checkpoint state; restoring updates the existing boxed object.
   Never replace a runtime box with a fresh allocation on restore/reset.

   Trace/update owned construction, snapshot projection, branch transfer and
   telemetry value access. Borrowed field calls use Box's built-in projection;
   keep independent field borrows and exact destruction order. Prepared/stopped
   Signal owners still carry the same runtime, so their inline size shrinks
   automatically without a new retirement protocol.

2. At `SignalConditionalExecutionSlot`'s definition in
   `crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs`,
   make `partition` an owned box. Construct it only after the existing original
   admission completes, in `execution_kernel.rs` and
   `evaluation_reuse/readmission.rs`.

   Funding already exists: `SignalEvaluationPartition::initial_heap_charge`
   includes `Charge::capacity::<Self>(1)`. Execution-basis partition admission
   reserves that storage (plus observation and reservation backing) and its
   original work before constructing the partition. The canonical partition's
   storage and slot reservations remain inside the boxed partition until its
   owned destruction. Do not add a grant or a second reservation; do not alter
   charged partition visits, unwind release, retry or denial outcomes.

   The admission itself ceases reserving space inline for a not-yet-created
   partition. Any footprint assertion must use the owner's actual current
   reported footprint, retaining its original claim.

## Expected drops and invariants

- SignalRuntime: approximately 15,552 -> 3,672 bytes (guard <=4 KiB).
- Each inline runtime occurrence in Bridge/Query shrinks by approximately
  11,880 bytes. Composite-enum maxima will be remeasured, not inferred as fact.
- SignalConditionalExecutionSlot: 11,480 -> pointer width (guard <=64 bytes).
- Evaluation state/admission: approximately 48/456 bytes; Bridge session ~488.
- RuntimeTelemetry remains Copy, with unchanged independent value semantics.
- No new public type, permission, control door, branch root, feature or macro.
- No new mutable sharing. Boxes retain unique ownership and field destruction
  order. Live custody remains inside the same native owners on every refusal.
- No changed serialized meaning or checkpoint format; this is live storage.

## Verification plan

Before the first check, finish every reached constructor/value transfer and
inspect the complete diff. Tighten the two measured owner ceilings and update
field-size observations to their actual new declarations.

One coherent verification round checks each touched workspace once, then builds
only the selected census targets, real consumer entries and Bank parents through
the owned build/nextest runners. Audit all selected names before execution:
no ignored test, query-scale, exhaustive oracle or scenario. Run the three
ordinary examples as their real mains with RUST_MIN_STACK absent, consumer-root,
the two Bank parents and the retained type-size tests. Confirm the PE stack
reservation stays 1,048,576 bytes for all three examples, and rerun the existing
whole-frame census on the final artifacts. Unchanged independent Store physical
inputs may reuse their green census evidence; do not rebuild them without a
changed input or unresolved concern.

Three implementation rounds are available. Before any additional round, record
the remaining risk and selected failed names in integration-progress.md.
