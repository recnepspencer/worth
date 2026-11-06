# Milestone 10.1: Original-Request Execution And Ownership Handoffs

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).

Preserve an original runtime owner and every produced native obligation when
opening, executing or handing back one stage is refused. Extend existing
Execution/World/Query boundaries; do not replace their scheduling or publication.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | One owner-preserving stage runner through real Query execution. | Existing request/phase contracts. | Reliable adapter handoff. | Land shared result semantics before adapter cutover. |
| 2 | Native/Signal/World effects preserve their exact outcomes across stops. | Phase 1 contract and existing native ports. | Recovery/disposal without false no-effect. | Owner adapters can be implemented separately. |
| 3 | All covered owning Query entries use the runner. | Phases 1–2. | 10.2–10.5. | Coordinate with accepted 9.x source; no implementation fork. |

## Phase 1: Preserve owners through the final request check

`ExecutionRequest::run_native_with_checkpoint` may discard an ordinary computed
value after final polling fails. That remains correct for discardable reads.
An effect-bearing stage needs a different owning handoff: callback output must
be stored outside the fallible execution return before final polling.

Target API below is a proposed platform-only Query publication boundary. Existing
`with_application_advancement` remains for scoped observations/discardable work.

```rust
pub enum WorthQueryOwnedAdvancement<O, V> {
    Completed { owner: O, value: V },
    Stopped {
        owner: O,
        produced: Option<V>,
        cause: WorthQueryAdvancementDenial,
    },
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn with_owned_application_advancement<O: WorthQueryAdvancementOwner, V>(
        &self,
        request: &WorthQueryRequestScope,
        owner: O,
        stage: impl for<'p> FnOnce(&mut O, WorthQueryAdvancementPhase<'p>) -> V,
    ) -> WorthQueryOwnedAdvancement<O, V>;
}
```

The implementation stores both the owner and a returned value before the
original final poll. Opening refusal returns `produced: None`; a stop after the
body returns retains `Some(value)` and the mutated owner. `produced` is internal
custody, not permission to disclose a cancelled query result. Only the sealed
owning adapter may interpret it. It is not exported through ordinary Host.
`WorthQueryAdvancementOwner` is a sealed framework owner association with exact
runtime affinity and native unwind/disposal custody, not an application marker
trait or an implementation for arbitrary `()`. Its implementations bind existing
native/operation owners. No stopped outcome may substitute a fresh request or suppress the original
latched cause. Distinguish a concrete inner denial from a generic nested stop.
The stop half of `WorthQueryAdvancementDenial` is [10.6](milestone-10.6.md)'s
carried `RequestStop`; owner denials stay separate and are never merged into it.

Unwind remains a programmer-defect path: preserve registered native effects,
restore borrowed custody and resume the original panic. Do not invent a handler
result, report success or serialize arbitrary Rust values. A shared guard must
retain the input owner during unwinding; its disposal follows 10.3. No guarantee
depends on an adapter catching a panic and rebuilding ownership from an ID.

| Transition | Granting owner/input | Output and permitted use | Refusal posture |
|---|---|---|---|
| Open stage | Installed runtime + original request + owned `O`. | Nonescaping runtime-affine `WorthQueryAdvancementPhase`. | Return original owner; callback not entered. |
| Execute/handoff | Same phase and borrowed owner. | Owned advancement retaining actual produced state. | Stop cannot discard owner/output or clear request latch. |
| Unwind | Existing native active-custody guards. | Registered native residuals, then original unwind. | No synthesized completion or rollback. |

Place the shared wrapper beside `request_execution/advancement.rs`; Execution
remains Query-agnostic. Architecture 9/16 require exhaustive result construction
and exact phase carriage; Architecture 22 requires stopped ownership to survive.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Opening refusal | Cancel the original scope before opening; lawful twin opens it. | Original move-only owner returned; zero handler contacts. | Query Execution owner test, focused. |
| Final handoff | Return a real native owner, then cancel at the existing final-handoff seam. | Exact owner and produced state survive; typed stop, no successful disclosure, no empty-`Option` panic. | Query public operation/consumer journey, CI. |
| Original controls | Exercise serial/leased parent cancellation, earlier deadline and a swallowed nested stop. | Same parent constraints and stop remain effective; unrelated sibling stays usable. | Existing Execution request tests plus Query seam, focused/CI. |
| Unwind | Panic after a genuine owner effect at the actual owner handoff. | Native record remains inspectable/recoverable; original panic retained. | World custody and Query owner tests, CI. |

Close only after the real stopped-handoff journey works. Add a contributor
example contrasting discardable computation with an owning effect stage.

## Phase 2: Preserve Native and Signal effect cutoffs

Use existing `PreparedRelationalCommitCandidate`, `RelationalPublicationOutcome`,
`SignalOwnerMovementPermit`, `ReservedCompositePublicationAttempt`,
`OwnerExecutionOutcome` and `ActiveAttemptCustody`. No generic execution receipt
may mint one of these proofs. World retains the authoritative orchestration
between the Native and Signal owners.

Target owner call sequence continues to consume its actual prepared candidate:

```rust
fn publish_original(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    port: &RelationalPublicationPort,
    prepared: PreparedRelationalCommitCandidate,
) -> RelationalPublicationOutcome {
    port.compare_and_publish(execution, prepared)
}
```

`RelationalPublicationPort` is the existing receiver in Native's
`mvcc/publication/port.rs`; the target is its consuming call, not a new public wrapper. World stores
the returned outcome before checking another owner or final delivery. Request
interruption and World/Signal cancellation remain different inputs; the World
source still owns propagation to Signal. A performed movement wins over late
cancellation. Current-read and product-head guards run at their real Native/CAS
boundary, not solely when a plan was created. Recovery cannot execute another
business handler without the existing native no-effect/retry proof.

Architecture 12 forbids collapsing owner-specific cancellation cutoffs; 14/22
forbid relabeling effects as rollback; 16 requires prepared native proofs.
Extend `current_read_publication`, `execution_service` and existing Native
execution adapters. No peer imports or new cross-runtime state machine.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Before effect | Cancel a genuinely prepared Native/Signal operation before its cutoff. | Typed no-effect; actual owner head unchanged; lawful twin moves it. | Native/Signal existing owner courts, focused. |
| Between effects | Publish Native, cancel before Signal; retry original recovery. | Native effect retained, Signal untouched, product unpublished; original recovery resolves without handler replay. | World between-owner court, CI. |
| After movement | Cancel after Signal or product movement and lose delivery. | Actual performed occurrence retained; no false aborted outcome or duplicate movement. | World cutoff/custody courts, CI. |
| Currentness | Move the original read dependency immediately before Native publication. | Real cutover refuses before the prohibited effect; cleanup retains its owner. | Native/World current-read court, CI. |

Close with unchanged existing cutoff, currentness and custody evidence plus
the new owning handoff cases. Preserve the existing explanatory API docs.

## Phase 3: Cut over owning Query entries

Migrate fixed, discovered, source-only and workflow operation advancement,
construction/retention, output handoff, delivery-close and candidate-adoption
entries where an owner currently crosses a fallible callback. Reuse the
existing public `advance`/`close` outcomes; 10.2 refines their common protocol.
Read-only callbacks receive a proposed `WorthQueryObservationPhase` through
`with_application_observation(request, body)`. It borrows the same admitted
request but exposes only observation ports. There is no public conversion to
`WorthQueryAdvancementPhase`. An owned advancement may lend its observation
view for existing reads; the inverse conversion is forbidden. This closes a
real bypass: merely calling an old callback "read-only" does not stop it from
using the full effect-capable phase.

Narrow `with_application_advancement` to the shared runner's implementation
boundary after callers migrate. It cannot remain an exported alternate way to
mint an effect-capable phase. Reads that trigger producers use the owning demand
pipeline for those producer effects; they do not gain mutation through an
observation token. Architecture 1 requires this read/write capability distinction.

The target caller uses the original operation with a fresh ordinary request:

```rust
fn advance_original<S, I, P, A, M>(
    operation: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> WorthQuerySourceOnlyAdvance<
    WorthQuerySourceOnlyOperation<S, I, P, A, M>,
    WorthQueryCompletedSourceOperation<S, I, P, A, M>,
    WorthQueryTerminalSourceOperation<S, I, P, A, M>,
>;
```

This is the existing source-owner shape with its existing schema/action bounds;
the implementation changes the handoff, not original key/input or fresh admission.
Runtime/branch mismatch remains an early typed refusal. Remove local `take()`
plus opening-only restoration patterns once their lane uses the common runner.
Adding a new owning entry must require the owned protocol through visibility
and sealed adapter construction, not a list of approved source filenames.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Family cutover | Stop after a real source/fixed/discovered result is produced, then retry. | Same result/roots/recovery retained; handler effect occurs once. | Existing Query consumer fixture, CI. |
| Workflow affinity | Supply wrong runtime/branch, then restore a lawful request. | No effect on foreign receiver; original issued workflow authority preserved. | Query workflow consumer court, CI. |
| No bypass | Attempt an ordinary effect adapter without the owning protocol; lawful adapter counterpart. | Compiler/visibility or installation rejects at the intended boundary. | Existing consolidated compile-contract lane. |
| Observation-only phase | Pass an issued observation phase to a mutation/adoption port, then use a lawful owned advancement. | Read still works; effect call fails compilation for the missing phase capability. | Same consolidated compile-contract lane. |
| Cost | Advance one owner amid growing unrelated retained population. | No registry-wide scan or new pool; generic handoff adds bounded owner-local work. | Existing cost court, scheduled only if expensive. |

Update contributor guidance and the existing compiled operation examples. Run
affected owner/public courts, formatting, line limits, boundary-check and
agent-context on implementation. No Cargo build is required for this brief.
