# Explain Original Query Results and Source Operations

Use causal inspection to answer an audit or debugging question about work that
already happened: what a query returned, whether two results differ, which source
operation supplied a produced value, and whether a producer changed its output.
Inspection uses the saved originals, so a later write cannot replace the evidence
you are explaining. Each inspection checks the caller's current permission.

Application declarations use `worth-query-decl`. Requests and results use
`worth-query-host::facade::{application_entry, causal_inspection}`. Start with
[application requests](../foundations/ordinary-application-front-door.md) if you
have not installed an application and authenticated a caller yet.

| You want to… | Start with |
| --- | --- |
| Identify one original query result | [Fresh application inspection](#fresh-application-inspection) |
| Compare what two queries returned | [Paired original comparison](#paired-original-comparison) |
| Inspect an authored operation without a query | [Source-only inspection](#source-only-inspection) |
| Connect an authored operation to produced output | [Source/producer connection](#sourceproducer-connection) |
| Distinguish changed output from equivalent output | [Performed producer output evaluation](#performed-producer-output-evaluation) |
| Read the saved query values or source effects | [Materialize original values](#rich-original-query-materialization) |
| Check retained detail and handle its loss | [Evidence availability](#original-evidence-availability) |

In the examples, `Read`, `ReadBinding`, `SourceDisclosure`, capability names and
`AppError` are types supplied by your application. `runtime` is your installed
application runtime; a program installation exposes it through
`application.runtime()`. `external_principal` and `scope` identify the current
caller and request. Use these Host imports for the examples:

```rust,ignore
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    application_contribution::WorthQueryProducerOutputEquivalence,
    causal_inspection::{
        CausalInspectionReason,
        WorthQueryApplicationEvidenceFamily as Family,
        WorthQueryApplicationMaterializationRequest as Detail,
        WorthQueryConditionalHistoryOutcome,
    },
    primary_graph::WorthQueryPrincipalResolutionMode,
};
```

## Prepare the inspection contracts

### Optional Query Inspection Metadata Meaning

Declare `ApplicationQueryInspectionDefinition<Schema, ReadBinding>` against the
query's complete binding and register it with `application_query_inspection`.
This determines who may inspect identities, evidence links, counts, availability,
explanations and diagnostic fields. It is separate from the query's business-data
disclosure: permission to read a result does not automatically permit an explanation.

A public metadata contract is explicit. For governed metadata, disclose the
inspection facets through the intended capability. Counts require `Count`
influence; the other metadata facets require `Explanation`. Comparison and rich
values also need the declared diagnostic field, relation and influence permissions.

For source inspection, implement `ApplicationSourceDisclosureBinding<Schema>` and
add `inspection_disclosure SourceDisclosure,` to the mutation binding. The source
receipt then retains that binding's original scope and parameters. The disclosure
query defines inspection permissions; inspection does not rerun it to reconstruct
the operation.

The [query binding declaration](../../../worth-query-certification/examples/application_correction_support/causal_inspection/query.rs),
[source-disclosure declaration](../../../worth-query-certification/examples/application_correction_support/causal_inspection.rs)
and [metadata contract](../../../worth-query-certification/examples/application_correction_support/causal_inspection/independent.rs)
show the binding and facet declarations together.

## Keep the evidence you will inspect

| Object | How to obtain it | What to use it for |
| --- | --- | --- |
| Authored source receipt | Keep the performed receipt returned by the original action or mutation | Pass it to `inspect_source`, `caused_by` or `evaluated_by` |
| Settled output observation | Keep `settlement.observation()` from the output demand | Select the produced result's read basis |
| Original query observation | Clone `result.receipt().inspect().observation()` before dropping the result | Pass it to `inspect::<ReadBinding>` |
| Current capability access | Admit the declared inspection capability for the current caller and subject | Supply metadata or diagnostic permission for a new inspection |

### Original Disclosed Projection Custody

Opt into saved values on each query that you intend to compare or materialize:

```rust,ignore
let request = runtime.request(&external_principal, &scope);
let result = request.query(Read { key })
    .retain_projection_for_inspection()
    .execute()?;
let original = result.receipt().inspect().observation()
    .ok_or(AppError::ObservationUnavailable)?.clone();
drop(result);
```

A governed payload query uses its ordinary
`execute_governed(Capability::reference(), Operation::reference(), input)` form.
Save both originals when comparing two executions. Captured empty results are
valid empty projections. Without the opt-in, you can inspect the original's
reference, but its values are unavailable for comparison or materialization.

Complete one-shot field/relation projections support capture. Unbound, partial,
continuation, orbit and component capture return a typed refusal. Handle that
refusal when executing the query rather than assuming values were saved.

### Original Settled Producer Custody

When you need a producer explanation, read through its actual settlement:

```rust,ignore
let result = request.at(settlement.observation())
    .query(Read { key })
    .retain_projection_for_inspection()
    .execute_governed(ReadCapability::reference(), ReadOperation::reference(), read_input)?;
let original = result.receipt().inspect().observation()
    .ok_or(AppError::ObservationUnavailable)?.clone();
drop(result);
```

A plain `request.query(...)` reads the selected head and does not infer the
settled producer from equal values. Keep the authored source receipt separately:
`settlement.application_commit_receipt()` identifies the producer publication and
is not automatically the authored operation that changed its input.
[Output demands](../foundations/ordinary-application-front-door.md)
explain how to obtain and settle the output owner.

## Admit current inspection access

Inspection's `execute_governed` consumes already-admitted
`WorthQueryAdmittedApplicationCapabilityAccess` values. This differs from a
payload query's refs-and-input execution call. Admit access using the Host's
selected branch and the capability declared by the inspection contract:

```rust,ignore
let metadata = runtime.with_host_advancement(|phase| -> Result<_, AppError> {
    let selected = runtime.on_branch(runtime.current_world()).select(&phase)?;
    let binding = runtime.installed_schema()
        .principal_binding(ActorBinding::reference())?;
    let principal = selected.resolve_authenticated_principal(
        &phase, &binding, &external_principal, &scope,
        WorthQueryPrincipalResolutionMode::Ordinary,
    )?;
    let capability = runtime.installed_schema()
        .capability(Inspect::reference(), InspectEvidence::reference())?;
    Ok(selected.admit_capability_access(
        &phase, &principal, &capability, inspection_input, &scope,
    )?)
})??;
```

`inspection_input` selects the actual subject and purpose required by your
capability contract. `AppError` maps the typed setup/admission denials into your
application's error handling. Repeat admission for each metadata and diagnostic
argument. Use the source subject for source access and the query subject for
query access; do not clone or borrow one side's permission for the other.
The [subject-specific access helper](../../../worth-query-certification/examples/application_correction_support/causal_inspection/distinct_subjects.rs)
shows this selection in application code.

## Fresh Application Inspection

Start a new request and inspect the saved original:

```rust,ignore
let fresh = runtime.request(&external_principal, &scope);
let report = fresh.inspect::<ReadBinding>(&original)
    .why_observed().reference_only().execute_governed(metadata)?;
assert_eq!(report.reason(), CausalInspectionReason::ObservedResult);
let availability = report.evidence_availability();
report.close();
```

For public inspection metadata, use `execute()` instead. One original establishes
`ObservedResult`. It does not establish that anything changed or that a particular
operation caused the result. `observation_identity()` correlates the original;
`report_identity()` identifies this inspection. Treat both as descriptive IDs.

## Paired Original Comparison

Use two saved complete projections to compare their returned values:

```rust,ignore
let report = fresh.inspect::<ReadBinding>(&after)
    .why_changed(&before).reference_only()
    .execute_governed(metadata, diagnostics)?;
match report.reason() {
    CausalInspectionReason::ChangedResult => show_changed(),
    CausalInspectionReason::UnchangedResult => show_unchanged(),
    _ => unreachable!(),
}
report.close();
```

Public metadata uses `execute_with_diagnostic(diagnostics)`. The bindings,
parameters, subject, result shape and disclosure meaning must be compatible.
Comparison includes optional presence and ordered nested rows. A new revision or
producer execution alone does not mean the returned values changed. Missing or
withheld values refuse comparison rather than producing a partial unchanged result.

## Source-only inspection

Inspect an authored operation even when no query result was retained:

```rust,ignore
let report = fresh.inspect_source::<SourceDisclosure>(&source_receipt)
    .why_performed().reference_only()
    .execute_governed(source_metadata)?;
let commit = report.source_commit();
report.close();
```

Public source metadata uses `execute()`. This report identifies the original
performed source. It does not include business values or claim a producer cause.

## Source/producer connection

Use a source receipt and a query read from the producer settlement to explain
creation of an observed output revision:

```rust,ignore
let report = fresh.inspect::<ReadBinding>(&original).why_observed()
    .caused_by::<SourceDisclosure>(&source_receipt).reference_only()
    .execute_governed(query_metadata, query_diagnostics,
        source_metadata, source_diagnostics)?;
let connection = report.producer_connection();
report.close();
```

The arguments are query metadata, query diagnostics, source metadata, then source
diagnostics. Public metadata uses
`execute_with_diagnostics(query_diagnostics, source_diagnostics)`.
The source must be the operation whose recorded change was actually read by the
producer; its declared output must match what the original query observed. An
older equal write, no-op write or unrelated producer cannot supply that cause.

`producer_commit()` identifies the settled producer publication.
`output_publication_commit()` identifies the publication that created the
observed field revision. These can differ when a producer reobserves its own
publication. Use the returned IDs; do not infer either from the current head.

### Independently admitted source and query connection

The output query may omit the authored input. Give source and query their own
inspection bindings and admit their contracts independently. Their parameter
values can differ; each must still describe its own original subject.

### Distinct Original Source And Query Subjects

A query rooted at B may observe A's produced field through an authored reference.
Admit query access for B and source access for A. A grant for one record does not
authorize the other, even if both have the same entity kind. A replacement under
the same selector is a different generation and cannot stand in for the original.

### Retained Output After Retirement

If the original query's subject still admits inspection, it can explain a nested
output captured before that output was retired. An equal replacement does not
replace the original evidence. An empty query made after retirement does not by
itself establish a retirement cause.

## Performed Producer Output Evaluation

Use `evaluated_by` to distinguish a producer that changed its output from one that
computed an equivalent output. The producer opts in for one Required standard
`u64` field on its exactly-one Preserve role. Add this method to its
`WorthQueryApplicationProducerBinding<Schema>` implementation:

```rust,ignore
fn output_equivalence(
) -> Option<WorthQueryProducerOutputEquivalence<Schema, Self::OutputRole>> {
    Some(WorthQueryProducerOutputEquivalence::exact_u64(Calculated::reference()))
}
```

The handler must complete the declared read of that output field before writing
it. Then inspect the original evaluation:

```rust,ignore
let report = fresh.inspect::<ReadBinding>(&original).why_observed()
    .evaluated_by::<SourceDisclosure>(&source_receipt).reference_only()
    .execute_governed(query_metadata, query_diagnostics,
        source_metadata, source_diagnostics)?;
let evaluation = report.producer_evaluation();
let decision = evaluation.decision(); // Changed or Equivalent
report.close();
```

For a parity producer, input `2 -> 4` can compute output `0` again: Equivalent.
Input `4 -> 5` computes output `1`: Changed. Equivalent preserves the observed
output revision, so it cannot claim a new creation through `caused_by`.
`output_role()`, `output_field()` and `observed_output_revision()` identify the
admitted field; `same_evaluation_as` compares evaluation occurrences.

Stable input reuse and checkpoint-restored output cannot supply a new performed
comparison; inspection returns `NotPerformed`. A retry that resumes the same
completed evaluation can report that original occurrence without another compute.

## Rich original query materialization

Select `QueryInspection` to read the saved values:

```rust,ignore
let detail = Detail::family(Family::QueryInspection);
let rich = fresh.inspect::<ReadBinding>(&original).why_observed()
    .materialize(detail).execute_governed(metadata, diagnostics)?;
for row in rich.materialization().original_query().into_iter().flat_map(|p| p.rows()) {
    for field in row.fields() {
        consume(field.output_name(), field.result_path(), field.value());
    }
}
rich.close();
```

Public metadata uses `execute_with_diagnostic(diagnostics)`. The view reads the
original disclosed fields and ordered nested relations, even if the current head
has changed. A captured empty projection is Provided with zero rows. An original
without saved values reports `OriginalProjectionNotRetained`.

## Rich connected source and producer evidence

Add source effects and the verified producer explanation to a connected request:

```rust,ignore
let detail = Detail::family(Family::QueryInspection)
    .with_source_effects().with_producer_evidence();
let rich = fresh.inspect::<ReadBinding>(&original).why_observed()
    .evaluated_by::<SourceDisclosure>(&source_receipt).materialize(detail)
    .execute_governed(query_metadata, query_diagnostics,
        source_metadata, source_diagnostics)?;
let body = rich.materialization();
for field in body.source_effects().into_iter().flat_map(|s| s.fields()) {
    consume(field.output_name(), field.effect(), field.value());
}
rich.close();
```

For a `caused_by` result, read `rich.materialization().producer_connection()`;
for an `evaluated_by` result, read `rich.materialization().producer_evaluation()`.
Both return an `Option`. Select `.with_producer_evidence()` to include that
explanation in the materialized body. Unrequested detail remains NotRequested.

## Rich source-only inspection

Select source effects without requiring a completed query:

```rust,ignore
let rich = fresh.inspect_source::<SourceDisclosure>(&source_receipt)
    .why_performed().materialize(Detail::source_effects())
    .execute_governed(source_metadata, source_diagnostics)?;
drop(source_receipt);
for field in rich.materialization().source_effects().into_iter().flat_map(|s| s.fields()) {
    consume(field.output_name(), field.effect(), field.value());
}
rich.close();
```

This result owns its readonly source view and can outlive the source receipt.
Public metadata uses `execute_with_diagnostic(source_diagnostics)`.
Source effects support the disclosure query's direct-root fields on the original
source subject. Other shapes report `UnsupportedSourceProjectionShape`.

| Effect | Value returned |
| --- | --- |
| Present | The value recorded by that source write |
| Unchanged | None; the field was not changed by that source |
| FieldCleared | None; the source cleared the field |
| AspectCleared | None; the source removed the parent aspect |

The view describes the source's recorded effects, not a query against later
state. Query projection, producer evidence and conditional history are not
provided by source-only mode.

## Original evidence availability

Reference, connected, evaluated and rich query reports expose
`evidence_availability()`. Comparisons expose before/after availability separately.
The values are snapshots taken during that inspection; getters do not check again.

| Accessor | Meaning |
| --- | --- |
| `projection()` | Retained or NotRetained; an intentionally saved empty result is Retained |
| `supplier()` | CommittedOutput, StableInputReuse, CheckpointRestoration or NotRetained |
| `native_observation()` | Whether the original conditional observation is retained |
| `conditional_history()` | Optional history's recorded availability, or None when no conditional observation was captured |

NotRetained does not mean no producer ran or that evidence expired. An original's
immutable evidence has no TTL. For materialized detail, distinguish NotRequested,
OriginalProjectionNotRetained, unsupported source shape, and
RequestedButNotProvidedBySelectedObservationMode. Use the outcome accessors before
reading an optional body; a missing body is not a complete empty explanation.

## Optional conditional execution history

Use history when you need the supplier's conditional/readiness decision, attempt
or output versions. It describes readiness separately from Changed/Equivalent
output comparison. Select it on a connected rich request:

```rust,ignore
let detail = Detail::conditional_history()
    .with_source_effects().with_producer_evidence();
let report = fresh.inspect::<ReadBinding>(&original).why_observed()
    .evaluated_by::<SourceDisclosure>(&source_receipt).materialize(detail)
    .execute_governed(query_metadata, query_diagnostics,
        source_metadata, source_diagnostics)?;
match report.materialization().conditional_history_outcome() {
    WorthQueryConditionalHistoryOutcome::Provided => {
        let history = report.materialization().conditional_history().unwrap();
        consume(history.attempt(), history.decision_class());
    }
    WorthQueryConditionalHistoryOutcome::OwnerUnavailable(cause) => show_loss(cause),
    other => show_selection_outcome(other),
}
report.close();
```

History has bounded retention. A new execution can evict detail between an
availability report and a later materialization request. That later request
rechecks the original's history and can report EvictedByRetention while the source,
query values and performed evaluation remain usable. OmittedByTier, CaptureDenied
and OwnerClosed describe different owner outcomes. Minimal diagnostics tier alone
does not imply omission; the installed record-retention policy decides that.
A previously materialized readonly history snapshot remains usable after eviction.
Plain query and source-only requests cannot provide connected history.

## Refusal, Cancellation And Retry

Handle `kind()` first. Depending on the failed operation, inspect
`resource_refusal()`, `canonical_refusal()`, `producer_connection_refusal()`,
`producer_output_refusal()` or `producer_evaluation_refusal()` for more detail.
Early permission refusal can intentionally expose no protected cause detail.
For an output connection, detail can distinguish an unrelated entity/field,
missing revision, wrong contract or an output not created by that publication.
It does not identify an alternative creator.

Revoked access, cancellation and deadline expiry refuse the new request. They do
not expire an immutable original or undo a completed source operation. Retain the
originals and use a new authorized request and newly admitted capability access
for a retry. Close reports to release their request funding. Close or drop the
last original holder when you no longer need its evidence; closing one clone
leaves its sibling valid. Original query reports borrow that original, while the
source-only materialized result owns its source view.

## Declare source shapes that can be explained

### Canonical Absence Evidence

An absent-input connection needs a completed absence read and the specific clear
or parent removal that created that absence. Repeated no-op clears and equal
`None` values cannot replace that occurrence. Keep the original source receipt
through removal, recreation and a later removal.

### Explicit Scalar Aspect Storage

Bind a whole scalar aspect explicitly:

```rust,ignore
let schema = schema
    .optional_aspect(Number::reference(), ScalarFacts::reference())
    .scalar_aspect_field(Number::reference(), Value::reference());
```

Use one Required field with a primitive scalar codec such as
`U64ApplicationValueBinding`. Extra children, optional child presence or structured
carriers refuse with `InvalidAspectShape`. A normal `field` declaration uses Struct
storage, even with one child. Read and write the bound scalar through ordinary
`DecisionReader::field` and `CandidateWriter::write_field`.

### Scalar Producer Outputs

Use the scalar field's own declared output role and build readiness with
`WorthQueryOutputReadinessContractBuilder::for_field`, then its declared
`field_dependency` and `build_typed`. This preserves the Scalar field's delivery
and eligibility meaning. Read through the output settlement and independently
admit the source/query inspection contracts. A changed Struct sibling cannot
supply the Scalar output's missing effect or revision; handle `OutputUnconnected`.

### Public Optional Struct Removal

Declare parent removal with `worth_query_operation_removes_aspects!`, register
`operation_remove_aspect`, and declare every child decision read:

```rust,ignore
worth_query_operation_reads!(EditInputAspect => [Key, Ceiling, Bias, Sibling]);
worth_query_operation_removes_aspects!(EditInputAspect => [InputFacts]);
let schema = schema
    .operation_read_field(EditInputAspect::reference(), Bias::reference())
    .operation_read_field(EditInputAspect::reference(), Sibling::reference())
    .operation_remove_aspect(EditInputAspect::reference(), InputFacts::reference());
```

Complete the child reads before `writer.remove_aspect(&target, InputFacts::reference())`.
A field-write marker does not authorize parent removal. Recreate a removed parent
with all required fields; mixing removal and child writes refuses.

For a Required child under an Optional parent, use
`ApplicationQueryOptionalAspectFieldRef`, `optional_aspect_field` and
`disclose_optional_aspect_field_by`. Project each child with
`row.optional_aspect_field(slot)?`. Only actual parent absence returns None;
a present parent missing Required content is a denial.
