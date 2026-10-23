# Query composite correction recovery boundary

This review concerns public 9.18 Phase 2 only. It supplements the existing
composite effects plan; it does not change the milestone's acceptance criteria.

## Actual ownership and current gap

`prepare_operation` constructs the native source operation from the original
inverse input, transformed key and `WorthQueryCorrectionIntent`. The inverse
handler also prepares a conditional definition successor. At commit preparation,
that successor's identity becomes part of the complete native idempotency
binding. The retained source operation's earlier correction intent does not yet
carry that definition identity.

`provider/managed_application_recovery.rs` freshly admits the caller, binds the
operation, scope, preconditions, governed input and proposal, then compares the
whole result with `ManagedUnpublishedAttempt::matches_fresh_binding`. The absent
definition identity can therefore produce `BindingMismatch` before the explicit
`ConditionalDefinitionRequired` refusal. A future repair must take only that
descriptive definition identity from the exact retained attempt, after acquiring
its native guard, and preserve every other equality and fresh-authority check.
It must not copy the entire old idempotency binding over current admission.

There is a separate substantive recovery limitation. The retained Query attempt
records `ManagedUnpublishedPublicationMode::ConditionalDefinition`, and ordinary
managed adoption explicitly refuses that mode. Query's publication consumes the
move-only conditional change through Bridge and World's combined candidate.
World retains any applied Bridge installation extension in its existing attempt
custody, but no public World recovery operation completes a conditional
definition adoption. Its recovery service exposes Relational settlement and
`prepare_settled_relational_adoption`, which constructs a without-Signal attempt.
The latter also rejects retained evidence whose Signal component is Published.
Removing the Query refusal would publish incomplete meaning.

## Adequate existing evidence and missing Query proof

Reuse the verified World combined cancellation/race evidence and the successful
Query composite publication/replay evidence. The World cases establish real
component movement, whole-World visibility and retained owner resources; they do
not establish that an interrupted Query correction traverses its original
operation, exact inverse identity and mandatory cleanup handoffs correctly.

The required additional Query proof uses the original operation throughout:

1. Prepare the same recorded composite correction through `prepare_operation`.
   Inject the existing native durable-append failure after preparation and before
   its publication, using the established certification control.
2. Require the returned original source operation to have ProductUnpublished
   posture. Inspect its exact native recovery record and require real Relational
   effects with settlement owed, Signal untouched, and unchanged selected World.
   The ordinary query must still observe the complete predecessor.
3. Verify normal close returns the same owner with NativeObligationRetained.
   Retry through fresh authorization, settle only the original Relational result,
   and continue the original retained conditional publication to one coherent
   performed World successor.
4. Verify ordinary reads and conditional execution observe the recovered
   successor, exact keyed replay returns its receipt without new owner effects,
   and that same original operation reaches normal close.
5. Preserve negative cleanup controls: cancellation before any owner effect
   releases the prepared candidate; stale recovery and inspector-held refusal
   retain original custody; explicit cleanup retires the exact native record.

World's existing recovery owner must consume retained conditional custody and
complete exact Bridge activation under the current branch/product checks. The
Query source owner retains the correct definition identity and routes to that
operation. Neither a handler rerun nor without-Signal adoption is an acceptable
substitute.

The focused native reproduction now establishes the current loss directly:
after real Relational settlement failure, the prepared Bridge candidate count
falls from one to zero and its retained bytes from 4390 to zero while World
still holds ProductUnpublished custody. The cancellation-before-effects control
passes. The failing case completes exact native cleanup before its assertion,
so this evidence does not leave an outstanding fixture obligation.

## Required destination for performed combined recovery

Successful recovery belongs in the existing World recovery service and its
conditional-definition attempt custody. The bounded extension has two native
states to preserve, rather than one generic retry abstraction:

- **Before Signal movement:** retain the original prepared Bridge extension and
  its Signal definition publication operation in
  `ConditionalDefinitionAttemptCustody` before entering Relational execution.
  Currently they live in a consuming callback and disappear if Relational
  settlement stops first. After settling the exact Relational result, resume the
  retained Signal operation against its original admitted predecessor. Reuse
  settled Relational adoption so no Relational candidate or handler runs again.
  `execution_service.rs` admits the Signal publication before Relational work,
  so custody must preserve the resulting admitted permit across that early exit,
  not reconstruct a raw publication request. Existing generic
  `CompositePublicationIntent<S>::with_settled_relational_adoption` already
  supports a WithSignal intent; the component lowering can pair that settled
  Relational leg with Signal AdvanceExact.
- **After Signal movement:** retain the actual Signal advance binding, resulting
  admitted Signal basis and component result with the applied Bridge extension.
  A fresh World successor adopts both exact settled component results, completes
  the retained Bridge activation, and performs the ordinary product CAS. Neither
  owner runs again. Bridge registry commit remains after World Performed.
  `execution_service/signal.rs` removes the advance binding before invoking the
  activation callback. Move that exact capability into custody before fallible
  activation and retain it on refusal; validate before any consuming handoff
  that could otherwise lose the original capability.

The precise fallible activation handoff must return both capabilities. Change
Signal's existing consuming completion to
`Result<RetainedParts, (Self, SignalConditionalDefinitionAdvanceBinding,
SignalConditionalInstallationExtensionDenial)>`. Its private binding match
returns `Result<AdmittedSignalBranchBasis, Self>` so mismatch returns the same
binding, then the outer failure returns the same completion as well. There is no
replacement binding or clone. Bridge's existing method still borrows
`&mut BridgeAppliedConditionalInstallationExtension`; when it takes the Signal
completion, it must restore that exact completion into `applied.signal` on
failure and return
`Result<(), (SignalConditionalDefinitionAdvanceBinding, BridgeConditionalDenial)>`.
World restores that returned binding alongside the still-owned applied extension
in its recovery lease. Test mismatched root/sibling binding twice: both returned
completions must subsequently succeed with their original matching bindings.
This protects the two-capability refusal before extending World continuation.

The concrete missing service is an owner-issued preparation/continuation for a
retained conditional-definition record, alongside
`prepare_settled_relational_adoption`. Its input is the existing recovery handle
plus the fresh request controls and original Bridge assembly; its output uses
the existing performed/no-effect/product-unpublished conditional publication
outcomes. It must acquire the recovery catalog's existing exclusive update
guard, validate runtime/branch/incarnation and exact expected product head, and
return custody unchanged on refusal. It must never accept caller-supplied
component identifiers or reconstruct a candidate from authored values.

The concrete public entrance is
`RuntimeWorldPublicationPort::prepare_bridge_conditional_definition_adoption`.
It returns `PreparedConditionalDefinitionRecovery`, whose `planned_successor`
and `unpublished_recovery_handle` let Query reserve its output lineage,
idempotency claim and terminal before another component can move.
`execute_bridge_conditional_definition_adoption` consumes that prepared owner
and returns the existing conditional performed/no-effect/product-unpublished
outcome. Dropping preparation restores the same original catalog custody.
Actual execution transfers custody only to its exact reserved successor.

Retained catalog progress is deliberately a summary. Settled Signal adoption
therefore comes from the original `Arc<SignalBranchAdvanceOutcome>` in
`component_results.signal`, not from that summary. This distinction is covered
by the after-Signal native continuation, which must preserve both exact result
identities and issue no additional component-owner calls.

Use existing plan and reservation owners for the fresh successor. Extending the
private component-adoption representation to include retained Signal evidence
is necessary for the second state: `prepare_settled_relational_adoption` is
correctly restricted to Signal RetainExact. Carry original evidence and owned
resources through a move-only recovery lease; on cancellation, failed activation,
stale CAS or unwinding, restore the lease to the same catalog record or its exact
registered successor. Bridge's candidate payload keeps its original Bridge
reservation; moving its owner into World must not duplicate that charge. World
reserves its own additional custody bookkeeping before owner work through the
existing attempt metadata reservation. Busy inspectors, stale
product selection, unavailable owner evidence, foreign Bridge identity, request
stops and exhausted allocation remain typed refusals.

Query then extends its current managed recovery dispatch by publication mode.
It recovers only the missing descriptive conditional-definition identity from
the guarded original attempt before full fresh-binding equality. Conditional
mode calls the new native recovery service under the existing product activation
gate. Finalization uses `ManagedUnpublishedAttempt::complete` with the actual
recovered definition generation, then the unchanged recovered publication and
cleanup owners. The original source operation advances normally to completed
custody and closes; no replacement operation is minted.

The native service has separate preparation and execution. Preparation owns the
exclusive original recovery lease and exposes its reserved successor. Query
then prepares the original attempt's output lineage slot, reserves the provider's
exact successor mapping, and rebinds its unfilled terminal before execution can
move an owner. Dropping preparation restores the original native custody. Both
publication modes share those delivery reservations; conditional continuation
uses the existing activation gate and original Bridge assembly. The gate is
released before ordinary recovered publication and fresh idempotency resolution.
A freshly supplied conditional identity is compared, never overwritten; only an
absent identity is completed from the guarded retained attempt.

### Focused regression order

First reproduce the current limitation in the native definition-publication
recovery family, with the existing real interruption controls:

1. Fail durable Relational settlement before Signal is contacted. Prove the
   actual prepared conditional successor remains recoverable; then recover with
   one total Relational execution and one total Signal execution.
2. Cancel immediately after actual Signal movement. Both owner results remain
   retained and the selected World stays the predecessor. Fresh recovery must
   perform with zero further component contacts and make the exact retained
   Bridge definition visible only with the combined World successor.
3. Move the expected World before recovery and require a stale refusal with the
   original custody intact; retain an unrelated-successor guard and normal
   cleanup. Preserve an activation-failure twin and inspector-held/busy refusal
   when existing unchanged evidence does not already exercise the new lease.

Only after those native boundaries pass, add one Query correction owner journey
using the same durable-settlement interruption: prepare, publish to the actual
partial owner, refuse premature close, freshly advance that same owner to the
combined performed receipt, exact keyed replay, ordinary reads/conditional
execution, and normal close. Assert original causal source, definition
generation, both component postures and handler contact counts. Existing World
two-owner cancellation and race evidence remains complementary; it is not a
substitute for this Query journey.

### Restorative Signal publication permit handoff

The dedicated definition port currently consumes the admitted operation before
owner validation, output reservation and the cancellable advance. The output
owner then splits it into scope and mint before a canonical movement exists.
A refusal or unwinding before movement therefore destroys the original admitted
permit even when the installation callback was never entered.

Keep the exact operation in a mutable optional slot through the existing mutation
and output-retention owners. Borrow its scope for transaction admission; consume
it into its advance mint only after the branch carries canonical completion.
The dedicated public port returns the ordinary completion together with the
original admitted permit when it remains unconsumed. World restores that permit
to its current custody before inspecting the completion or resuming unwinding.
No new permit is issued and all owner, predecessor, cancellation and allocation
checks remain in place. The three existing healthy internal test callers pass
the same operation through this slot.

One focused Signal regression will pass a genuinely admitted publication through
cancelled-before-callback and callback-no-movement refusals, then successfully
publish its original paired installation request. It checks no basis movement
before success, actual callback counts and final one-use advance binding.

### Query custody during native outcome unwinding

Signal's existing outcome-construction fault can carry a genuine advanced result
and an unwind payload together. World records that outcome and resumes unwinding;
the active attempt's Drop retains its exact reserved successor. Native conditional
recovery then leaves its definition custody with that successor. Query's managed
recovery guard currently restores the original provider entry and removes its
staged successor mapping unconditionally on Drop. That loses the relationship
between the retained Query attempt and the actual World effects.

Reproduce this at the borrowed managed recovery boundary with the existing native
fault control. Inspect actual successor custody and the provider's exact recovery
mapping after the caught unwind, then continue and release the retained work. A
consumed source-operation wrapper cannot be reconstructed by catching a panic;
the ordinary same-owner lifecycle court remains a separate proof.

The focused borrowed fixture uses the ordinary combined `Change` operation and
its original input and key. A fresh ordinary mutation cannot fabricate the
recorded-correction causality component; the separate correction owner court
proves that lifecycle. Arm the existing Signal outcome-construction fault by
delegating its original owned control through Bridge and Query, with closed
certification callers. After a real durable-append partial, catch the exact
Signal fault during recovery, readmit the actual native successor, and recover
it with the same input and key. Require the combined receipt, unchanged handler
contacts and exact cleanup; a missing provider correlation must fail the proof.

Reconcile the existing managed guard with the exact pre-reserved successor's
native evidence before resuming unwinding. If World retained successor effects,
transfer the existing Query attempt and mapping there; if no successor exists,
preserve the original. Use the existing reservation/guard and native recovery
port, with no candidate replay, inferred partial outcome or replacement owner.
Preserve the normal typed-result handoff and test pre-effect restoration as its
negative counterpart. Runtime qualification follows the failing reproduction.

### Performed source completion identity and delivery

The original source-operation recovery reaches performed publication, then its
fresh keyed completion read reports `IntentDrift`. Its retained correction
intent predates the handler's conditional-definition binding. Carry only that
definition descriptor from the actual guarded performed attempt into the
existing recovery result. The publication-boundary completion reader may fill
an absent descriptor after fresh authorization; an explicit different value
still reaches full equality and refuses. Apply this only to actual performed
custody, including its retained completion state. Extract the existing performed
result into its own child module to preserve the file limit.

The court closes the original completed owner with `close_with_evidence` and
performs exact replay. Recovered idempotency receipts deliberately carry no
one-shot performed patch. Boundary inspection established that the canonical
Relational subscription already delivers the recovered publication to the
conditional clock. The court uses that existing path and proves the corrected
conditional wake after normal close and replay; no delivery carrier was added.

The focused native Signal installation-extension family passes all five cases
(`q918-phase2-signal-restorative-publication-native.log`). The original permit
survives cancellation, no-movement refusal and a caught caller unwind before
its paired request performs. Both mismatched genuine completion/binding pairs
remain usable with their original counterparts, and the existing exact-byte
and work-exhaustion controls still pass.

### Recovered source identity during fresh keyed resolution

The actual SourceCompletion correction court reaches `RecoveryPerformed`, then
its fresh keyed resolution returns `IntentDrift`. Managed recovery restores the
original conditional-definition component before comparing the complete binding,
but its performed result does not carry that descriptive component into the
source owner's next fresh-authorized read. The original recorded correction
intent predates the handler's prepared definition and cannot supply it.

Retain that one component on the existing native performed result, derived from
the exact recovery attempt that performed. Its publication-boundary method fills
only a missing component on the fresh keyed binding; an explicitly different
component remains different. Fresh request admission, authority, scope, input,
preconditions, and full idempotency comparison stay in their existing owners.
The source owner's Performed and retained-delivery states borrow their actual
performed result for this handoff. Other states have no such descriptor.

Move the existing performed result type and its lifecycle methods into the
dedicated `managed_application_recovery/performed.rs` child to preserve the
parent's size boundary. Update the existing source admission resolver and its
three state callers. The unchanged real combined correction court must progress
from the same original owner to its exact receipt and normal close; the recorded
fail-before is `q918-phase2-query-composite-recovery-exact.log`.

### Conditional execution after owning recovery

A recovered idempotency receipt deliberately has no consumable performed-change
witness. `close_with_evidence` transfers original completion custody, but cannot
manufacture a witness missing from the native result. The existing recovered
session owns the actual publication and its one-shot `take_fresh_delivery` cell.
ProgramSource extracts its full original carrier before pending publication;
OwningSource currently extracts nothing and returns only the recovered read and
cleanup custody.

The native conditional clock already consumes canonical Relational subscription
commits through `reconsider_authoritative_clock_work`, retaining their exact
cursor, pending invalidations and wake identities. That ordinary path observes
the actual recovered publication without needing another direct delivery
witness. No additional carrier is necessary. The source-only completion court
closes with its original evidence, performs exact replay, and observes the clock
through that subscription. The existing independent conditional predicate and
actual output value 99 must prove the recovered definition participates in
execution after the same-owner close. No receipt clone supplies delivery.

The current native example passes the original-owning correction recovery court
and the borrowed unwind court independently, then passes the full registered
example with the configured 64 MiB worker stack. The same-owner court reaches
exact causal replay and normal close, then the ordinary subscription produces
one eligible wake and actual secondary output 99. The unwind court catches the
real Signal fault, preserves the native successor's provider correlation,
completes without handler replay, drains the recovery catalog and closes the
runtime. Logs: `q918-phase2-query-composite-recovery-final.log`,
`q918-phase2-query-composite-unwind-final.log`, and
`q918-phase2-query-correction-all-final-stack.log`.
