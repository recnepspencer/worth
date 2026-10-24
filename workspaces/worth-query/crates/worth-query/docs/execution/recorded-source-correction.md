# Correct a recorded source operation

A correction applies the recorded inverse or compensating action declared by
the application. Reapplication performs the original retained input again under
current authority. Select a performed source receipt, prepare the action against
the target branch, then publish the prepared candidate. Success appends a new
World commit while preserving the target's intervening disjoint edits. A
Relational-only correction retains the target's Signal basis; an authored
conditional-definition correction publishes its explicit Signal successor with
the corrected Relational state.

The [compiled numeric example](../../../worth-query-certification/examples/application_correction.rs)
starts with primary `5` and secondary `7`, changes primary to `11`, changes
secondary to `19`, and reverses the first operation. Readback becomes primary
`5` and secondary `19`. Its fork and authority courts use the same installed
application and ordinary request API.

## Declare the corrective action and disclosure query

Implement `decl::application_aftermath::ApplicationCorrectionBinding<Schema>`
and list it in the contribution's `corrections: [RecordedEdit]` declaration.

| Member | Application meaning |
| --- | --- |
| `Source` | The ordinary mutation binding whose performed occurrences can be selected. |
| `InverseIntent` | The ordinary intent that performs the correction. Its binding supplies the handler, principal mapping and admission mode. |
| `Disclosure` | The query determining whether the current caller can inspect the original subject on the target. |
| `source_definition()` | The source operation's external-effect policy. Access is declared by the source operation. |
| `mechanism()` | The recorded-inverse capture demand or compensation business postcondition, bound to the actual corrective operation. |
| `source_output_fields()` | Historical fields of declared created output roles used to derive compensation or a created-entity inverse. |
| `authored_identity()` | The authored lifecycle family, logical-key field and exact source/inverse output roles for create/delete inversion. |
| `completed_external_reconciliation()` | The exact reconciliation procedure authorizing business compensation of a completed external workflow operation. |
| `disclosure_parameters(original)` | Query parameters associated with the original governed input. |
| `derive_correction(original, material)` | The domain's deterministic action construction from the original input and its mechanism-specific material. |
| `inverse_key(key)` | Fallible conversion to the inverse binding's key type, returning `Result<Key, ApplicationValueEncodeDenial>`. Validation refusals use the ordinary mutation key-identity denial. |
| `visit(dispatch)` | Typed selection of conventional or capability admission for the inverse. |
| `visit_reapplication(dispatch)` | Enables the original source binding's conventional or capability admission for retained reapplication. |
| `reapplication_key(key)` | Converts a new reapplication command key into the original source binding's key type. |

The complete [numeric binding](../../../worth-query-certification/examples/application_correction_support/correction.rs)
captures primary and decodes it through `U64ApplicationValueBinding`. Its
`mechanism()` returns `DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(demand)`;
the capture width comes from the field binding's encoded representation. Native
derivation supplies `ApplicationCorrectionSourceMaterial::RecordedInverse(preimage)`.
The
[disclosure query](../../../worth-query-certification/examples/application_correction_support/query.rs)
addresses the original number and applies its current read-enabled predicate.
The [contribution](../../../worth-query-certification/examples/application_correction_support/schema.rs)
installs the association alongside the ordinary schema members.

## Select, prepare and publish

Import the runtime API from `worth_query_host::facade::application_entry`:

```rust,ignore
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationCorrectionPreparation,
    WorthQueryApplicationRequestExt,
    WorthQueryCorrectionHistoryLimits,
};

let request = runtime.request(&principal, &scope).on_branch(target);
let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
let source = request.corrections().select(&original_receipt, limits)?;
let preparation = request.corrections().reverse(&source)
    .idempotency(client_key)
    .limits(limits)
    .prepare()?;

match preparation {
    WorthQueryApplicationCorrectionPreparation::Prepared(candidate) => {
        handle_outcome(candidate.publish()?);
    }
    WorthQueryApplicationCorrectionPreparation::Settled(outcome) => {
        handle_outcome(outcome);
    }
    WorthQueryApplicationCorrectionPreparation::PublicationPending => {
        show_pending_publication();
    }
    WorthQueryApplicationCorrectionPreparation::ProductUnpublished(handle) => {
        retain_unpublished_recovery(handle);
    }
}
```

The handlers above are application callbacks. The compiled example provides a
complete caller with actual readback assertions.

History limits control ancestry traversal and disclosure-result work. The
example permits one result because its query addresses one unique number.
Choose bounds from the application's history and query contracts.

Selection requires current disclosure plus retained native ancestry and source
material. A copied receipt identifies the same performed source in its owning
runtime. Keep the returned source selection alive for the correction and its
retries: it owns the source's history pin while operation owners can close
normally. Each preparation observes the target again and checks current authority.

`inspect(limits)` reports semantic compatibility. Preparation additionally
derives and captures the inverse input, resolves its scope, admits the current
caller and runs the installed handler to build a candidate. State changes when
`publish()` succeeds. Dropping the candidate releases its preparation.

## Select a source after closing its operation

Capture the completed receipt's `correction_source()` locator, then close
the completed operation normally. A later request can select that locator:

```rust,ignore
let locator = receipt.correction_source();
// Close the completed operation through its owning handle.
let request = runtime.request(&current_principal, &scope);
let source = request.corrections().select_recorded(&locator, limits)?;
let preparation = request.corrections()
    .reverse(&source)
    .idempotency("correct-original-transfer")
    .limits(limits)
    .prepare()?;
```

`WorthQueryRecordedCorrectionSource` contains the World commit identity and
full Relational receipt. Its `coordinates()` method returns the serializable
`WorthQueryCorrectionSourceCoordinates`. Store these coordinates with the
application's history entry. A transport can define its own scalar wire type
and construct native coordinates with
`WorthQueryCorrectionSourceCoordinates::from_untrusted_fields`, supplying the
World owner and commit, Relational branch and commit, version, and parent list.
Then call `request.corrections().select_coordinates(&coordinates, limits)`.
Selection resolves the actual retained completed evidence, compares its World
identity and checks ancestry and current disclosure on the target branch. Keep
the complete coordinates together when serializing them.

Normal operation close releases its custody while the native history owner
governs source retention. Selection returns a typed refusal when that evidence
is unavailable or the current caller cannot disclose the original subject.
The inverse then receives its own current admission checks during preparation.
For a sequence of requests that must keep the source available, retain a source
selection through the sequence and release it when that history is no longer
needed. Serialized coordinates locate an occurrence; its live selection owns
retention. After the final selection is released, subsequent publications can
retire that optional source history.

The [Bank accounting court](../../../../../worth-query-bank-world/crates/bank-server/tests/ordinary_mutations/recorded_correction.rs)
shows closed source operations, later transfers, a freshly authenticated teller,
original-journal compensation and replay through this entry.

## Keep the corrective action's delivery owner

Choose preparation according to the action's installed delivery. Reversal uses
the inverse action; reapplication uses the original source action:

| Delivery | Preparation |
| --- | --- |
| Mutation publication with its ordinary receipt | `prepare()` |
| An installed action with SourceCompletion or RetainedCommit delivery | `prepare_operation::<Correction, Program, Action>(&application)` |
| An installed action with a fixed required output root | `prepare_performed::<Correction, Program, Root>(&application)` |

Bank exposes its installed journal compensation through this public entrance:

```rust,ignore
let preparation = runtime.prepare_send_money_correction(
    &teller,
    &scope,
    &completed_source.coordinates(),
    client_key,
    limits,
)?;
```

This returns `WorthQueryOperationCorrectionPreparation`. Publish a `Prepared`
candidate to receive `WorthQuerySourceOnlyAdvance`; a `Continued` result supplies
that existing action's progress directly. Store the returned active, completed
or terminal owner before replying to the client. Advance it with a fresh request,
read through its declared delivery and close it through its ordinary lifecycle.
A refusal returns its owner so the caller can continue or close it later.

Source-only correction owners retain `WorthQueryCapturedMutationInput` with the
selected binding and original encoded input. This supports actions that share
one input codec across several mutation bindings. Use the completed owner's
`original_input()` borrow when the application needs to inspect the input;
advance and close the owner through its ordinary action methods.

`IdempotencyIntentDrift` identifies reuse of the key for different correction
meaning. `PublicationPending` and `ProductUnpublished` preserve the outstanding
native publication obligation. Equivalent retries resolve the original action
and its performed source. The Bank
[correction entry](../../../../../worth-query-bank-world/crates/bank-server/src/correction.rs)
shows selection of received coordinates and the typed action entrance.

## Reapply an explicit historical alternative

Enable reapplication on the same `ApplicationCorrectionBinding` by implementing
`visit_reapplication` and `reapplication_key`. The declaration chooses the
original source binding's admission mode:

```rust,ignore
fn visit_reapplication<V: ApplicationReapplicationVisitor<NumericSchema>>(
    dispatch: ApplicationReapplicationDispatch<NumericSchema, Self, V>,
) -> V::Output {
    dispatch.conventional()
}

fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
    Ok(format!("numeric-reapplication:{key}"))
}
```

For a capability-governed source, use `dispatch.capability::<Capability>()`.
Installation records the association with the original source binding. At
preparation, Query takes its retained encoded input and resolves the source
action and delivery in the current program. The ordinary handler evaluates
that input against current state and current authority.

Read alternatives through the same branch-bound request. `maximum_entries`
sets the page width independently of the ancestry and disclosure limits:

```rust,ignore
let source = request.corrections().select(&source_receipt, limits)?;
let mut page = request.corrections()
    .alternatives(&source, limits, maximum_entries)?;
let mut alternatives = page.entries().to_vec();
while let Some(continuation) = page.continuation() {
    let next = request.corrections()
        .continue_alternatives(&source, limits, maximum_entries, continuation)?;
    alternatives.extend_from_slice(next.entries());
    page = next;
}
let chosen = choose_alternative(&alternatives);
let selected = page.select(chosen, limits)?;
let preparation = request.corrections().reapply(&selected)
    .idempotency(client_key)
    .limits(limits)
    .prepare_operation::<RecordedEdit, Program, Action>(&application)?;
```

`choose_alternative` is the application's history chooser and returns a reference
to one of the listed entries. Each entry identifies an exact performed reversal.
Complete the continuation before selection: the complete traversal establishes
which reversal is currently eligible to be consumed. `AlternativeResolutionIncomplete`
means further pages remain. Each continuation belongs to the exact native index
generation and target observation. A stale or unavailable continuation returns
a typed refusal; obtain a new page under the current request.

Publish or continue `WorthQueryOperationCorrectionPreparation` and close its
ordinary source owner as described above. For an original action with fixed
required outputs, use `prepare_performed::<Correction, Program, Root>` and
complete its output owner. Both entries use the original retained input.

An ordinary edit preserves historical alternatives. For example, source S is
reversed by U, reapplied by R and reversed again by V. The chooser still lists
U and V. Choosing U now repeats S's original meaning and consumes V, the active
reversal on this branch. The committed relation records the chosen U and the
consumed V separately. Another command must wait for a new reversal before
reapplying again. Reverse the newly performed occurrence to open that next
cycle.

A fork created before consumption may independently reapply its inherited
alternative. A fork created after consumption inherits the consumed relation.
This follows the selected branch's actual ancestry. Reusing a command key
resolves its original selected and consumed relationships, including after a
later reversal; the retry leaves that later reversal available.

Retain the selected alternative through preparation and retries. Historical
entries remain useful for display when optional source material expires.
Selection reports unavailable semantic material or the native ancestry owner's
history-expiry refusal, according to which required evidence expired. Current
disclosure and fresh mutation authority apply to every new selection and
preparation. A changed source action or delivery produces a typed action or
program meaning refusal so the application can explain the required support.

### Carry a chooser across requests

A server can retain the native page while the client displays its scalar history
entries. Consume the request-bound wrapper with `into_page()`, then associate
that page with the server's existing session or continuation owner. Share the
page with `Arc` when an in-flight request and the session both need it. Replacing
or expiring the session entry releases its reference to the page.

Keep source or alternative selections, or completed operation owners with retained
commit delivery, alive for every occurrence the chooser needs. Release selections
and close completed operation owners normally after the final selection and retry.
Their native observations keep source history available while requests traverse
and select historical entries.

Each entry's `source()` coordinates identify the original occurrence operated on
by that historical reversal. Send these coordinates with the client's selection.
After traversal completes, a fresh authenticated request resolves that selection
against the retained final page:

```rust,ignore
let completed_page = page.into_page();
// A later request supplies its own principal and scope.
let fresh_request = runtime.request(&current_principal, &fresh_scope)
    .on_branch(target);
let selected = fresh_request.corrections()
    .select_alternative_coordinates(&completed_page, &chosen_source, limits)?;
let preparation = fresh_request.corrections().reapply(&selected)
    .idempotency(client_key)
    .limits(limits)
    .prepare_operation::<RecordedEdit, Program, Action>(&application)?;
```

Selection checks the page's exact target observation, resolves the chosen
occurrence through native history, and checks current disclosure for its source.
When the target has advanced, refresh the chooser and submit the chosen source
with the same command key for an equivalent retry. Keep that key stable through
response loss so ordinary idempotency resolves the recorded command.

For paginated requests, retain each returned page's native continuation until
the next page succeeds. Continue it through a fresh request's
`continue_alternatives` method with the selected traversal source. The final page
carries the completed lineage resolution used to select an entry displayed on
any earlier page. Session ownership and expiry belong to the server's ordinary
request lifecycle; action publication returns its own owner, which the server
retains through advancement and normal close.

The [tree journey](../../../worth-query-certification/examples/application_correction_support/tree_reapplication.rs)
shows multi-page selection across fresh request scopes, older-alternative choice,
exact retries and normal source close. The [required-output journey](../../../worth-query-certification/examples/application_correction_support/required_reapplication.rs)
shows the source publication and subsequent output settlement separately.

## Compensate a performed operation

Compensation applies a new forward operation to neutralize an earlier effect.
Its handler evaluates the current target and preserves later lawful work. For
example, a balance of `100` receives `+30`, then `+7`; compensating the first
credit debits `30` from `137`, leaving `107`.

Declare the compensation through the same `corrections: [Binding]` contribution
clause and ordinary selection/preparation/publication entry. The binding names
the actual source mutation, corrective intent and disclosure query. Its
`mechanism()` derives the compensating operation slot from that concrete intent:

```rust,ignore
fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
    DeclaredCorrectionMechanism::compensation_binding::<Self>(
        DeclaredAftermathPostcondition::BusinessPostcondition {
            identity: "original-credit-neutralized".to_owned(),
        },
    ).expect("the compensation declares a business postcondition")
}

fn derive_correction(
    original: &Credit,
    material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
) -> Result<Debit, ApplicationCorrectionDerivationDenial> {
    match material {
        ApplicationCorrectionSourceMaterial::Compensation(_) => Ok(Debit {
            key: original.key.clone(),
            amount: original.amount,
        }),
        _ => Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning),
    }
}
```

Use either `BusinessPostcondition` or `InvariantRestored` to name the operation's
business guarantee. Installation resolves its exact mutation binding, handler,
input and disclosure meaning and publishes `Compensatable` posture. Recorded
inversion publishes `Reversible` posture and receives its demanded pre-image.

The original occurrence supplies the mechanism and original admitted input.
Current disclosure controls access to that material; the compensating handler's
fresh ordinary reads and admission determine current eligibility. Preparation
retains the exact target through publication. Equivalent retries resolve the
original correction command and receipt, including its causal source link.

The [compiled credit/debit binding](../../../worth-query-certification/examples/application_correction_support/compensation_movement.rs)
and [compensation court](../../../worth-query-certification/examples/application_correction_support/compensation_courts.rs)
show current-balance arithmetic, intervening work, replay and current authority.

## Correct an operation performed inside a workflow

Select the performed operation's receipt in the same way as an ordinary source.
The runtime retains its originating workflow instance, transition and operation
receipt. Preparation resolves that exact native settlement on the selected
target and preserves the instance's current continuation. A live successor keeps
its progress; a completed or cancelled instance keeps its terminal disposition.

Continue an unsettled operation through its original owner first. A pending
operation in external-owner custody returns `WorkflowOperationInOwnerCustody`
during correction preparation. Once the original operation settles, its
authored decision dependencies still determine correction compatibility.
Workflow authority consumed by that successful settlement is accounted for
separately from the source's authored reads. The corrective operation receives
fresh disclosure and ordinary admission on the current target.

The inverse handler writes the domain subject covered by the binding. Native
candidate validation preserves the original workflow instance and its custody.
The [document retention example](../../../worth-query-certification/tests/application_graph/workflow_approval/correction.rs)
shows an approved retention edit corrected after completion, during a live
successor and after cancellation.

## Compensate a completed external workflow operation

Declare the source's real external effect and inbound protocol in
`source_definition()`. Bind a separate business compensation through
`mechanism()` and name the original external owner's reconciliation procedure:

```rust,ignore
fn completed_external_reconciliation() -> Option<DeclaredReconciliationProcedure> {
    Some(DeclaredReconciliationProcedure::new(
        "confirm-approved-payment-settlement",
    ).expect("the payment settlement procedure has a stable identity"))
}
```

Installation binds this procedure to the correction's source, inverse and
disclosure meaning. Preparation requires the canonical authenticated inbound
completion for the exact original dispatch, publication and product incarnation,
plus the originating workflow operation's native settlement. Keep advancing and
accepting the original owner until these obligations settle, then prepare the
compensation through a fresh request.

Bank's `ReverseApprovedPayment` reads the journal created by the approved payment
and derives an ordinary `ReverseJournal` intent. Current teller admission and
balanced posting invariants govern the new compensating journal. The original
payment remains committed, the remote settlement remains completed and the
workflow keeps its existing continuation. Exact retry returns the same
compensating occurrence. Call
`runtime.prepare_approved_payment_correction(&teller, &scope, &coordinates, key, limits)`
to retain the installed journal reversal's delivery; publish or continue its
returned owner and close it under a fresh request. The
[binding](../../../../../worth-query-bank-world/crates/bank-domain/src/schema/approved_payment_correction.rs)
shows the declaration and original journal projection; the
[owning journey](../../../../../worth-query-bank-world/crates/bank-server/tests/approved_payment_workflow/owner_custody/correction.rs)
shows original rail custody, settlement, compensation and replay.

## Read original fields of a created output

A source can create an entity whose identifier is allocated during execution.
Declare an exactly-one Create output role on the source mutation and bind the
actual created entity with `writer.create_output::<Role>(&entity)`. The
correction can then request fields of that role from the original commit.

For each required field, construct a typed witness from the source binding,
its declared role and the entity's field reference:

```rust,ignore
fn issued_key() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Key::reference())
}

fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<NumericSchema>> {
    vec![issued_key()]
}

fn derive_correction(
    original: &Issue,
    material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
) -> Result<Record, ApplicationCorrectionDerivationDenial> {
    let ApplicationCorrectionSourceMaterial::Compensation(fields) = material else {
        return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
    };
    let encoded = fields.field(&issued_key())
        .map_err(|_| ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning)?;
    let key = StringApplicationValueBinding::decode(encoded)
        .map_err(|_| ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning)?;
    // Construct the domain's corrective intent using the original key.
    record_issued_output(original, key)
}
```

`source_output_fields()` is the correction's complete manifest. Installation
checks the actual source binding, role, entity and field meaning, including the
value codec, unit and coordinate frame. Use the field's ordinary value binding
to decode its borrowed representation inside `derive_correction`.

The material view follows the original role's full generational entity identity
into that same commit's canonical field patch. It returns the original value
after later edits or retirement of the live entity. Current disclosure of the
source subject runs first; the corrective action then receives its own fresh
ordinary admission. A refused material read terminates preparation even when
the domain callback catches its marker error.

The [complete binding](../../../worth-query-certification/examples/application_correction_support/source_output_correction.rs)
requests both the created identifier and original payload. Its
[ordinary handlers](../../../worth-query-certification/examples/application_correction_support/source_output_handlers.rs)
bind two actual created roles, and its
[runtime court](../../../worth-query-certification/examples/application_correction_support/source_output_courts.rs)
reads back the published output, changes its payload, retires it and corrects
from the original committed values. Exact replay preserves the original
correction receipt and World head.

## Restore an authored entity with a new identity

An authored entity has two useful identities: its domain logical key and its
native generational entity identity. A recorded deletion restores the authored
values under the same logical key by creating a successor entity. The successor
receives a new native identity. References to the retired entity continue to
identify that retired occurrence. The corrective operation's created output
identifies the successor for subsequent authored relationships.

Declare one family for the logical-key field used by both lifecycle directions:

```rust,ignore
pub struct NumberFamily;
impl ApplicationAuthoredCorrectionFamily<NumericSchema> for NumberFamily {
    const IDENTITY: &'static str = "numeric.authored-number.v1";
    type Entity = Number;

    fn logical_key() -> ApplicationAuthoredLogicalKey<NumericSchema, Number> {
        ApplicationAuthoredLogicalKey::from_field(Key::reference())
    }
}
```

The family binds the entity and the complete field meaning, including its value
codec, unit and coordinate frame. Use an exactly-one Create role on the creating
operation and an exactly-one Retire role on the deleting operation. Their
handlers bind the actual entities through `create_output` and `retire_output`.

For a creation's inverse, return this association from `authored_identity()`:

```rust,ignore
Some(ApplicationAuthoredCorrection::retire_created::<
    Self, NumberFamily, Created, Retired,
>())
```

Use `DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()`.
Its derivation receives `ApplicationCorrectionSourceMaterial::CreatedEntity`.
Declare any original created fields needed by the domain inverse in
`source_output_fields()` and decode them with their ordinary bindings. At native
candidate validation, the retired role and actual delete must both name the
original created entity's full generational identity.

For a deletion's inverse, return the opposite association:

```rust,ignore
Some(ApplicationAuthoredCorrection::restore_retired::<
    Self, NumberFamily, Retired, Created,
>())
```

Use `recorded_inverse_binding::<Self>(demand)` and capture the logical key plus
the authored fields needed to rebuild the entity. Derive the restoring intent
from `RecordedInverse(preimage)`. The ordinary handler allocates the successor,
initializes those values and binds its Create output. Native validation compares
that actual allocation and initialized key with the original retired role and
its captured key before publication. A mismatched key, unrelated deletion or
substituted existing entity produces `CorrectionIdentityMismatch`.

Choose a disclosure scope that remains meaningful after retirement, such as the
owning document or collection. The caller's current disclosure and the inverse's
current admission still govern restoration. Use the corrective receipt's created
output to identify the successor when establishing a new authored relationship.
For relations declared with `same_context_unbounded_retain_dangling`, retirement
preserves the original endpoints for audit and changes the relation's lifecycle
to `RetainedDanglingForAudit`. Ordinary live adjacency reads omit that relation.
A consumer that requires it refuses the missing live relationship, even when a
successor with the same logical key is available through a fresh query.

The [complete family and correction bindings](../../../worth-query-certification/examples/application_correction_support/authored_identity_correction.rs)
show both directions. The [ordinary lifecycle handlers](../../../worth-query-certification/examples/application_correction_support/authored_identity_handlers.rs)
perform the actual allocation and deletion; the
[reference consumer](../../../worth-query-certification/examples/application_correction_support/authored_identity_reference.rs)
follows the authored relation while live and compares its endpoint's complete
identity with the original created output. After retirement and restoration, it
requires the exact missing-relationship refusal and an unchanged product head. The
[lifecycle court](../../../worth-query-certification/examples/application_correction_support/authored_identity_courts.rs)
checks restored values, the new native identity, independent edits, exact retry
and mismatched candidates.

## Correct an authored conditional definition

Keep the authored parameters in ordinary application fields. Declare a recorded
inverse that captures those parameters and derives the corrective intent from
their original values. Use the same ordinary handler for the authored edit and
its correction, so both prepare their effects through current admission.

In the handler's decision, build the replacement predicate provider from the
admitted input and tracked reads, then call
`reader.prepare_conditional_definition::<Conditional, Node, Clock, Provider>(provider)`.
`Conditional` is the installed contribution binding; its `Installed` type is
`WorthQueryConditionalClockHandle<Schema, Node, Clock>`. Installation supplies
that exact clock handle. The method prepares the successor against the decision's
retained product and returns `WorthQueryAdmittedApplicationConditionalDefinition`.
Carry that permit in the decision value.

In `build_candidate`, write the authored fields through the ordinary
`CandidateWriter` and pass the permit to
`writer.advance_conditional_definition(decision.definition)`. Publishing the
candidate joins the Relational effects and Signal definition successor in one
World commit. The receipt identifies both component publications and the
conditional definition generation. A moved target returns the ordinary stale
outcome; prepare again against the current target to retry.

The [complete ordinary handler](../../../worth-query-certification/examples/application_correction_support/composite_definition_handler.rs)
and [correction binding](../../../worth-query-certification/examples/application_correction_support/composite_definition_model.rs)
show these two stages. Their [runtime example](../../../worth-query-certification/examples/application_correction_support/composite_definition_court.rs)
changes an authored parameter from `5` to `23`, changes a separate field to
`19`, then corrects the original parameter to `5`. Readback preserves `19`.
The replacement predicate then permits an actual due conditional operation.
Equivalent correction retries return the original receipt and World head.

Bind the conditional's semantic output version through
`bind_host_output_version_provider` when installing its predicate. The example
uses the temporal operation's actual output value, `99`, as its output identity
under exact comparison.

Advance the installed conditional owner through its ordinary clock observation.
Its canonical Relational subscription supplies performed changes, including
changes completed during recovery, so the corrected definition evaluates current
dependencies. Open the clock port during host advancement, finish that phase,
then call the port's `observe` method with the execution request.

A fresh receipt can also transfer its owned change through
`deliver_relational_change_to_conditional`. When a completed source operation
owns that receipt, `close_with_evidence` releases its cleanup and returns the
original completion evidence for delivery. Historical receipt copies support
comparison and exact command lookup.
For an action with source delivery, use
`prepare_operation::<Correction, Program, Action>(&application)` and retain the
owner returned by publication. An interruption can leave Relational settlement
pending before the Signal definition runs, or leave both component results
waiting for World publication. In either case, advance that same owner with a
fresh request. Recovery continues its retained definition and component results;
the performed receipt identifies the combined successor and its actual
definition generation. World observations expose the last complete published
pair until that successor is published.

Keep the owner while recovery or delivery remains pending. A close refusal
returns it with the outstanding native obligation. After recovery, consume the
action's declared delivery and close the returned owner normally. Reuse the
original client key when checking the command after a lost response.

## Required outputs and recovery

When the inverse action delivers a required output root, use the typed performed
entrance. The installed action must declare fixed delivery to that exact root,
and its binding must implement the root connection's required-source contract.
The [numeric output program](../../../worth-query-certification/examples/application_correction_support/output_program.rs)
declares a calculation root whose producer computes `primary * 2 + 1`.

```rust,ignore
use worth_query_host::facade::application_entry::WorthQueryRequiredCorrectionPreparation;

let preparation = request.corrections().reverse(&source)
    .idempotency(client_key)
    .limits(limits)
    .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)?;

match preparation {
    WorthQueryRequiredCorrectionPreparation::Prepared(candidate) => {
        handle_performed_outcome(candidate.publish()?);
    }
    WorthQueryRequiredCorrectionPreparation::Settled(outcome) => {
        handle_performed_outcome(outcome);
    }
    WorthQueryRequiredCorrectionPreparation::PublicationPending => {
        show_pending_publication();
    }
    WorthQueryRequiredCorrectionPreparation::ProductUnpublished(handle) => {
        observe_pending_recovery(handle);
    }
}
```

Preparation derives output demand from the captured inverse input. Publication
returns `WorthQueryApplicationPerformedMutationOutcome`, the ordinary required
source outcome. Its variants determine which obligation the caller owns:

| Outcome | Continue with |
| --- | --- |
| `Performed(source)` | `source.start_required_outputs(&application, &fresh_request, controls)`, then settle the returned output handle. |
| `ProductUnpublished(recovery)` | Retain the owner and resume its source publication through fresh admission. |
| `RequiredOutputDenied(failure)` | Retain the committed source and retry required-output retention through fresh admission. |
| `CleanupRequired { cleanup, outcome }` | Retain the independent cleanup obligation and handle the enclosed source outcome. |
| `NotPerformed(outcome)` | Handle the ordinary mutation outcome, including exact replay. |
| `Blocked(owner)` | Retain its native custody and inspect its recovery posture. |

A published source and its completed outputs are separate milestones. In the
numeric example, correcting primary from `11` to `5` publishes authored state
while the stored calculation remains `23`. Settling the original source's output
owner regenerates the calculation to `11`. Secondary remains `19` throughout.

An equivalent retry reports the original command's status while the caller keeps
the original output or recovery owner. `AlreadyCommitted` supplies the same source
receipt; the original owner supplies the one-use continuation.

For an unpublished correction, resume with the original owner and a fresh request:

```rust,ignore
let progress = recovery.recover_recorded_correction(&application, &fresh_request)?;
// When progress is Performed, promotion transfers the original output custody.
let recovered = recovery.promote_recorded_correction(&application, &fresh_request)
    .map_err(|(denial, owner)| retain_failed_recovery(denial, owner))?;
let (outputs, initial_cause, performed, prior_cleanup) = recovered.into_parts();
```

The owner retains the captured inverse input and the key produced by
`Correction::inverse_key`. Recovery freshly checks principal, scope, current
authority and source identity, then continues the existing native attempt.
Cancellation or admission refusal leaves the owner available for another fresh
request. Promotion failures return the complete owner alongside the denial.

`performed` preserves the publication, idempotency-read and cleanup results.
Inspect these results and discharge their cleanup obligations alongside
`prior_cleanup`. The returned `outputs` preparation starts the original required
root under fresh request authority. A recovered correction receipt retains its
causal link to the original source.

For a committed source whose retention was refused, call
`failure.retry_recorded_correction_retention(&application, &fresh_request)`.
Success returns the original performed-source owner. Refusal returns the denial
and the complete failure owner for another attempt.

## Compatibility and current authority

Recorded-inverse eligibility compares the original operation's complete semantic dependencies
against the target. Own writes are compared with their committed after-state.
A later primary edit conflicts with restoring primary; a secondary edit is
preserved. A changed decision dependency also matters: changing a ceiling read
by the source handler can refuse correction while primary still equals `11`.

The caller needs current disclosure and current admission for the inverse.
Another authorized actor can correct the source after the original actor's
mapping is disabled.

An occurrence can be corrected after ordinary adoption to a program with
equivalent installed meaning. The runtime compares the retained source program
with the selected current program and resolves the complete original inverse
action in that program. The correction runs under the current program.
Relational-only correction retains the current Signal basis; an authored
conditional-definition correction supplies its explicit successor. Preparation keeps both program
interpretations alive through publication or recovery.

`InverseAction` distinguishes a missing, ambiguous or changed inverse action.
`ProgramMeaningChanged` carries the installed comparison requirements when
the programs differ. `ProgramComparison` preserves a comparison refusal.
The compiled example exercises adoption, reversal, exact retry and an adoption
that makes an already prepared correction stale.

A prepared candidate retains an exact target. A later head change returns the
native stale outcome. Prepare again against the current target when the
application chooses to retry. Keep the same client key for the same command.

## Outcomes and retries

`WorthQueryApplicationCorrectionOutcome` carries ordinary mutation outcomes:

| Outcome | Caller action |
| --- | --- |
| `Committed { receipt, result }` | Consume the published receipt and inverse result. |
| `AlreadyCommitted(receipt)` | Consume the original correction receipt from an equivalent retry. |
| `PreviouslyCommitted(history)` | Observe current state through a fresh query; historical evidence identifies the earlier commit. |
| `IdempotencyIntentDrift` | Report that the key names a different correction intent. |
| `DomainDenied(value)` | Interpret the inverse handler's domain refusal. |
| `Cancelled` or `DeadlineExceeded` | Handle the request stop. |
| `Commit(custody)` | Inspect and retain the native uncommitted outcome, including unpublished or unresolved effects. |

Domain values use `WorthQueryCorrectionValue`. `binding()` names the actual
inverse binding and `downcast_ref::<Value>()` reads its typed value. The
inverse result becomes available with the committed outcome.

Keys belong to the installed application, schema and caller. Reusing a key
for another source, inverse action or target is intent drift. An equivalent
retry freshly admits the retained performed inverse input and resolves the
existing command. It skips inverse derivation and first-use after-state
eligibility.

The inverse keeps its own declared correction contract. An irreversible
inverse still retains its actual input for equivalent retry; its receipt
reports that contract when selected as a potential new correction source.

Preparation errors preserve their owning boundary: disclosure, ancestry,
semantic comparison, derivation, input capture, principal resolution or
ordinary admission. `SourceDiverged` reports changed source meaning.
`RequiredOutputReconciliation` identifies a target requiring output
reconciliation.

The committed receipt's `aftermath_causality()` exposes its actual parent and
`recorded_source()`, which identifies the original World and Relational
commits. On a fork, the child belongs to the fork while its inherited parent's
receipt can still name the original branch.

## Related guidance

- [Ordinary application entry](../foundations/ordinary-application-front-door.md)
- [External effects and recovery](application-aftermath-and-recovery.md)
