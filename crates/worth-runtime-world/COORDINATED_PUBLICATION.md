# Coordinated publication contract

The frozen compiler-visible progression is:

```text
CompositePublicationIntent
  -> ResolvedExpectedProductHead
  -> AdmittedCompositeRuntimeWorldBasis
  -> LoweredOwnerComponentPlan
  -> ReservedCompositePublicationAttempt
  -> OwnerExecutionSettlement
  -> CompositePublicationReady
  -> RuntimeWorldPublicationOutcome
```

The terminal outcome has exactly three cases:

- `PerformedCompositePublication` proves the reserved commit won the exact
  product-head compare-and-publish.
- `NoEffectCompositePublication` proves that no component owner and no product
  reference moved.
- `ProductUnpublishedOwnerEffects` records named owner movement when the
  product reference did not move and retains the recovery obligation.

Every `ReservedCompositePublicationAttempt` carries the complete expected
observation, predecessor basis, specialized Relational and Signal plans,
reserved commit/history/recovery/pin capacity, cancellation/deadline policy,
canonical owner order, and exact progress. It is linear and cannot be cloned.

The order is fixed as Relational preparation, exact product-head recheck,
Relational publication and settlement, exact recheck, Signal operation, exact
recheck, and final product publication. World reference guards do not span
companion preparation, payload destruction, settlement, Signal work or final
product publication. The optional current-read condition below guards only
Native's already-prepared memory cutover. Unchanged components are represented by `RetainExact`
and do not imply a latest lookup or owner contact.

The public builder installs the managed owner and exposes typed ports. The
public prepared types distinguish execution with and without Signal; internal
progression types cannot be constructed or skipped by consumers. Branch creation uses an
explicit reuse/fork plan per component and checks cancellation before effects
and at source-guarded installation. A performed fork denied by cancellation
remains retained with its exact effects.

The internal execution and reference boundaries support concurrent mixed plans:
a parked Signal owner call cannot block the independent Relational leg, and
only one of their product CAS operations can win. A losing attempt retains its
actual owner effect without calling the sibling or retrying product movement.

## Require a current control input

When a write targets one branch but depends on current control state in another,
attach the actual owner-issued `ProductBranchObservation` to its intent:

```rust
let mut intent = CompositePublicationIntent::with_signal(Some(change))
    .with_prepared_relational_candidate(candidate);
intent.require_current_read(current_control.clone())?;
let prepared = publication.prepare_with_signal(target, intent, &cancellation, None)?;
```

Use `without_signal(change)` with `prepare_without_signal` when Signal is retained.
The write target and payload stay unchanged. World checks the exact control
reference under its cell guard while Native checks the corresponding admitted
source at the same cutover. A changed control input returns
`NoEffectCause::CurrentReadDependency`, distinct from a stale write target.
The condition grants no permission to perform a new action or Signal operation.

Attach the requirement once. Signal-only and settled-adoption plans refuse it.
The supported profile requires a committed Native source root and a memory-only
publication port; an initial Empty source or attached durable medium refuses
before effects. After genuine Native performance, this condition is discharged.
Settlement and original recovery use that performed evidence even if control
state changes later. The original write target's expected head still applies.

The final branch write lock compares the complete expected observation before
populating the preallocated history slot and transferring the bound component
pin pair. A stale comparison does neither. The final cell swap records canonical performed facts in the preallocated
history envelope before readers can see the moved head or old protection can
drop. A caller lost at that boundary can recover the original linear delivery
from the live owner. Recovery and the normal return read the same immutable
facts and cannot deliver a consumed publication again.

Ordinary reservation, settlement, readiness, and publication carry one caller
capability to a preinstalled owner record. Relational identity is recorded
before settlement consumes its performed capability; settled Relational
progress is retained before entering Signal. Dropping or unwinding an affected
phase abandons that capability without constructing history, acquiring pins,
or executing recovery. Explicit retained terminals acquire their returned
catalog view under the same lock that converts reservation accounting.

Branch creation carries the same custody across its real forks and finalization.
The registry binds a destination witness before effects and records actual
insertion under the source guard. A refused cell stays in the resource lease.
Post-insertion unwind releases attempt admission without inventing an unpublished
record, even if retirement and name reuse precede the caller's Drop.

## Cancellation and unwind

The World token is checked before effects and at named progression boundaries.
The final clock read precedes the final atomic cancellation check; once the CAS
installs, late cancellation cannot reclassify a performed publication. A stale
product head after an owner effect is a retained partial, never rollback.

Signal's owner-issued `SignalBranchAdvanceCompletion` carries its canonical
advanced result across a post-movement unwind. World records that exact result
in the preinstalled attempt custody before resuming the original panic payload.
An unwind before movement creates no invented owner effect. No counter, latest
lookup or reconstructed descriptor substitutes for that carried result.

`recovery_port().continue_effects(effects)` continues an already-performed
Relational settlement when necessary, or exposes cleanup-only obligations.
It never calls an unperformed Signal sibling or a product CAS. To publish the
original settled Relational successor explicitly, call
`recovery_port().prepare_settled_relational_adoption(&effects, &cancellation, deadline)`
and execute the returned `WithoutSignal` preparation. This preserves the original
expected product head; it does not rebase a lost write target.

To surround only the Native cutover with caller-owned liveness guards, use
`execute_without_signal_with_memory_cutover`,
`execute_with_signal_with_memory_cutover`, or
`publish_bridge_conditional_definition_with_memory_cutover`. These require a
fresh Native publication with a current-read requirement. Native preparation
finishes before the callback starts. The callback receives a borrowed,
single-use `PreparedWorldRelationalCutover`; consume it with `perform()` or
`perform_before(deadline)`. The latter adds a narrowing cutoff to the original
Native operation, checked after reservation waits before position assignment.

The callback starts before World acquires the current control cell guard. Only
the token's perform operation holds that guard; Signal, settlement, activation
and final product movement follow after the callback returns. Returning without
performing yields `CutoverNotPerformed`. This entrance grants no additional
permission and cannot be used for Signal-only work or settled adoption.

Inspect both parts of the returned `(Option<CallerResult>, outcome)`. `None`
means the callback was never entered. A caller error returned after `perform()`
cannot erase an actual Native effect or change the publication outcome. If the
callback panics after performing, World records that genuine effect in the
original recovery handle before resuming the panic. Inspect and continue that
handle to repair the exact Native settlement and release the retained
obligations. Identity-only recovery carries a settlement receipt; explicit
adoption requires the full settlement result and refuses a receipt alone.

Recovery retains exclusive custody of the record while settlement runs. If
settlement unwinds, the record regains its exact carried progress and component
results before it becomes available again. A consumed settlement route leaves
only its owner-issued identity and basis for explicit repair, never a recreated
performed capability. Cleanup remains denied while settlement is still owed;
retry continues that settlement without rerunning component publication.
