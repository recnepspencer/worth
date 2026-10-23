# Correct a recorded source operation

A correction applies the recorded inverse or compensating action declared by
the application. Select a performed source receipt, prepare the action against
the target branch, then publish the prepared candidate. Success appends a new
World commit while preserving the target's intervening disjoint edits and exact
Signal basis.

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
| `source_output_fields()` | Historical fields of declared created output roles required by compensation. |
| `disclosure_parameters(original)` | Query parameters associated with the original governed input. |
| `derive_correction(original, material)` | The domain's deterministic action construction from the original input and its mechanism-specific material. |
| `inverse_key(key)` | Conversion of the client command key to the inverse binding's key type. |
| `visit(dispatch)` | Typed selection of conventional or capability admission for the inverse. |

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
runtime. Each preparation observes the target again.

`inspect(limits)` reports semantic compatibility. Preparation additionally
derives and captures the inverse input, resolves its scope, admits the current
caller and runs the installed handler to build a candidate. State changes when
`publish()` succeeds. Dropping the candidate releases its preparation.

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

The [compiled recovery journey](../../../worth-query-certification/examples/application_correction_support/required_output_recovery.rs)
shows publication recovery, explicit cleanup and native output settlement.

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
action in that program. The correction runs under the current program and
retains its exact Signal definition basis. Preparation keeps both program
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
