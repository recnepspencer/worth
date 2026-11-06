# Milestone 10.6: Shared Request Stops In Owner Refusals

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
First in the group's order: no prerequisite. [10.1](milestone-10.1.md) consumes its
stop carriage for the stopped owning handoff.

Owners currently re-declare the same request stops inside their own refusal enums
and forward them by hand. Make the execution stop a single carried value, so a new
stop or a new owner refusal changes one place, and a stop can never be relabelled
as a domain refusal or lose its inner cause.

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | One stop carriage shape in Execution and the owner-refusal convention. | Existing `KernelStop` and nested-stop latch. | Owner migration; 10.1 stopped outcome. | Freeze the carriage before owners migrate. |
| 2 | Native, World, Bridge and Query refusals carry the stop instead of re-declaring it. | Phase 1. | Removal of forwarding `map_err`. | Owners migrate independently; one owner per slice. |
| 3 | Courts assert the carried stop; displaced variants are deleted. | Phase 2 per owner. | 10.1 cutover; 10.7/10.8 refusals. | Follows each owner's Phase 2 slice. |

## Investigation

Measured on `worth-relational` over 30 days to 2026-11-03:
- **Refusal enums:** 186. 74 carry at least one shared stop (execution/work, memory, cancellation) as their own variant or wrapper.
- **Forwarding:** 2,664 `map_err` calls; most only re-wrap another owner's refusal.
- **Churn:** refusal lines are 7.3% of all changed production lines, the largest single category measured.

`worth-execution` already owns the stop meaning: `KernelStop` (`backend/meter.rs`), `KernelFailure<E>` (`Stop | Domain`), the latched `request_stop`/`request_refusal`, and `NestedComputationStop` (`backend/meter/nested_stop.rs`), which keeps a computation-owned failure distinct from its request's. Owners then mirror these as `WorkCeilingDenial`, `RelationalExecutionDenialCause`, `MapKernelStop`, `Cancelled`, `WorkExhausted` and similar per-owner variants.

## Phase 1: One carriage, not one universal error

Reuse `KernelStop` and the nested-stop latch; do not add a second stop vocabulary or a second interruption latch. The carriage separates three things that owners currently merge by convention:
- **Stopped:** the original request stopped. Its exact latched cause is retained.
- **Owner refusal:** the owner's domain answer. It stays owner-specific.
- **Computation failure:** a nested computation failed on its own terms (`NestedComputationStop`). It is not relabelled as a request stop.

Target shape, beside `KernelFailure`:

```rust
pub enum OwnerOutcome<T, R> {
    Done(T),
    Refused(R),
    Stopped(RequestStop),
}

pub struct RequestStop { /* the latched KernelStop plus its nested owner */ }

impl RequestStop {
    pub fn cause(&self) -> KernelStop;
    pub fn nested(&self) -> Option<NestedComputationStop>;
}
```

`RequestStop` is constructible only from the request's latch. No owner can mint one from a label, and no `From<RequestStop>` exists into an owner's domain refusal. Owner refusal enums lose their stop variants. A boundary that must merge two owners' refusals composes `OwnerOutcome<_, Either-shaped R>`; it never flattens the stop.

Architecture 12 forbids false unification: storage unavailability, residency and owner-specific capacity refusals stay with their owners. Only request interruption and request-scoped accounting stops are shared. Architecture 5/7 and DX 17 require the stop's machine-readable cause to survive every hop.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| No relabel | Stop the request inside a nested Native stage that also has a domain refusal path. | Caller observes `Stopped` with the original cause; the domain refusal is not taken. | Execution nested-stop tests plus one Native owner court, focused. |
| Computation versus request | A nested computation exceeds its own narrower ceiling while the request is healthy. | `Refused`/computation failure, never `Stopped`; the request stays usable. | Existing Execution nested tests. |
| No minting | Attempt to construct `RequestStop` or convert an owner refusal into it. | Compile failure at the intended boundary; lawful twin passes the latched stop through. | Consolidated compile-contract lane. |

## Phase 2: Owner migration

Migrate one owner per slice: Native relational, World lifecycle and publication, Bridge, then Query execution and publication. Each slice:
- returns `OwnerOutcome` (or the owner's existing outcome type with a `Stopped(RequestStop)` arm) from its public and crate-internal fallible entry points;
- deletes its re-declared stop variants and every `map_err` that only forwarded a stop;
- keeps every owner-specific refusal and its context unchanged.

This is mostly mechanical once Phase 1 freezes. It suits a single-worktree codemod swarm per owner. A slice that finds a stop being converted into a domain refusal, or the reverse, records it as a defect fix in its commit, not as a rename.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Forwarding removed | After an owner slice, count stop-forwarding `map_err` and stop variants in that owner. | Zero re-declared stop variants; forwarding count reduced to genuine owner translations. | Source search in the slice evidence. |
| Behaviour unchanged | Run the owner's existing cancellation, deadline and work-ceiling courts. | Same typed outcomes and counters as before the slice. | Owner courts, focused. |

## Phase 3: Courts and deletion

Courts that matched an owner's stop variant match `Stopped(stop)` and assert `stop.cause()`. Rendered-diagnostic pins that only restated the stop are replaced by the cause assertion. Delete displaced stop variants and conversion impls with the last caller; no compatibility alias.

Closure: a new `KernelStop` cause reaches every owner's callers without editing any owner refusal enum, which a compile-contract or source-search check demonstrates. Update the contributor error-handling guide.
