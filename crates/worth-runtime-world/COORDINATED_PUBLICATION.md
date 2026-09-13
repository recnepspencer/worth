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
recheck, and final product publication. No Runtime World lock may be held over
a component-owner call. Unchanged components are represented by `RetainExact`
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
It never calls an unperformed Signal sibling or a product CAS. Automatic adoption
of an owner-local successor is outside 9.17.2.

Recovery retains exclusive custody of the record while settlement runs. If
settlement unwinds, the record regains its exact carried progress and component
results before it becomes available again. A consumed settlement route leaves
only its owner-issued identity and basis for explicit repair, never a recreated
performed capability. Cleanup remains denied while settlement is still owed;
retry continues that settlement without rerunning component publication.
