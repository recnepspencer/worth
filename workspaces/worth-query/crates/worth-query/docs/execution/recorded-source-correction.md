# Correct a recorded source operation

A recorded correction restores authored state through an inverse declared by
the application. Select a performed source receipt, prepare the inverse against
the target branch, then publish the prepared candidate. Success appends a new
World commit while preserving the target's intervening disjoint edits and exact
Signal basis.

The [compiled numeric example](../../../worth-query-certification/examples/application_correction.rs)
starts with primary `5` and secondary `7`, changes primary to `11`, changes
secondary to `19`, and reverses the first operation. Readback becomes primary
`5` and secondary `19`. Its fork and authority courts use the same installed
application and ordinary request API.

## Declare the inverse and disclosure query

Implement `decl::application_aftermath::ApplicationCorrectionBinding<Schema>`
and list it in the contribution's `corrections: [RecordedEdit]` declaration.

| Member | Application meaning |
| --- | --- |
| `Source` | The ordinary mutation binding whose performed occurrences can be selected. |
| `InverseIntent` | The ordinary intent that performs the correction. Its binding supplies the handler, principal mapping and admission mode. |
| `Disclosure` | The query determining whether the current caller can inspect the original subject on the target. |
| `source_definition()` | The source operation's external-effect policy. Access is declared by the source operation. |
| `preimage_demand()` | The exact fields needed by the inverse and their encoded storage demand. |
| `disclosure_parameters(original)` | Query parameters associated with the original governed input. |
| `derive_inverse(original, preimage)` | The domain's deterministic inverse construction. |
| `inverse_key(key)` | Conversion of the client command key to the inverse binding's key type. |
| `visit(dispatch)` | Typed selection of conventional or capability admission for the inverse. |

The complete [numeric binding](../../../worth-query-certification/examples/application_correction_support/correction.rs)
captures primary and decodes it through `U64ApplicationValueBinding`. Its
capture width comes from that binding's encoded representation. The
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

## Compatibility and current authority

Eligibility compares the original operation's complete semantic dependencies
against the target. Own writes are compared with their committed after-state.
A later primary edit conflicts with restoring primary; a secondary edit is
preserved. A changed decision dependency also matters: changing a ceiling read
by the source handler can refuse correction while primary still equals `11`.

The caller needs current disclosure and current admission for the inverse.
Another authorized actor can correct the source after the original actor's
mapping is disabled.

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
