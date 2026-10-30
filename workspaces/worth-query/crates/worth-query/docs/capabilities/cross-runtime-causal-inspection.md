# Cross-Runtime Causal Inspection

Application code imports `worth-query-decl` and `worth-query-host`. Start with the
[ordinary application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Inspect an actual completed query observation or performed source receipt through
a fresh Host request. The original supplies evidence, never current permission.
The request independently admits current READ and the binding's inspection metadata.
Diagnostic comparison and causal connection additionally require the applicable
current diagnostic contracts. Reports grant no mutation, correction or replay authority.

## Fresh Application Inspection

Keep the original observation before dropping the query result:

```rust,ignore
let original = result.receipt().inspect().observation()
    .ok_or(ApplicationError::ObservationUnavailable)?.clone();
drop(result);
let report = fresh.inspect::<ReadBinding>(&original)
    .why_observed().reference_only().execute()?;
assert_eq!(report.reason(), CausalInspectionReason::ObservedResult);
report.close();
original.close();
```

`ReadBinding` and the application error are application-defined. Governed metadata
uses `execute_governed(current_metadata_capability)`. One original establishes
`ObservedResult`, not `ChangedResult`.

The report borrows its original and owns its fresh request's report reservation.
Closing it releases that reservation. Observations share their immutable backing
and original payer; close or drop the last clone to release it. Neither retains
a mutable runtime or a Native snapshot reader.

### Original Query Binding Custody

Execute an ordinary bound query to retain its actual installed binding, scope and
admitted parameters. Two bindings of the same query remain distinct. A foreign
runtime, schema, binding, branch incarnation or full-generation subject cannot be
substituted using a matching name, digest or current value. An unbound query cannot
acquire an original binding later.

### Original Query Row Evidence

Observations retain actual completed row footprints and separate result-set
selection meaning. Empty results retain selection evidence and an empty row roster.
Set membership does not stand in for projected reads. These footprints supply
neither business rows nor current READ or a producer connection.

### Original Disclosed Projection Custody

Request complete original payload custody on each query execution to be compared:

```rust,ignore
let result = request.query(Read { key })
    .retain_projection_for_inspection()
    .execute()?;
```

Governed payload queries use their ordinary `execute_governed` form. Custody retains
the Native field/relation tree and original effective slot-disclosure decisions;
it does not reconstruct a DTO from later state. Complete empty results are supported.
Unbound, partial, continuation, orbit and component capture refuse rather than
publishing incomplete comparison evidence.

### Optional Query Inspection Metadata Meaning

Declare `ApplicationQueryInspectionDefinition<Schema, Binding>` against the complete
ordinary query binding and register it with `application_query_inspection`.
Metadata is separate from payload disclosure. The closed facets are observation
identity, evidence links, counts, availability, explanation and diagnostic fields.
A governed contract covers each facet once: counts require `Count` influence and
the other facets require `Explanation`. Public metadata is explicit.

Reference-only observation admits five reference facets before evidence lookup.
Diagnostic requests also admit the diagnostic facet and current `DiagnosticMask`
for projected fields and observable predicate, ordering and relation influences.
Another binding's metadata or public payload visibility supplies no such permission.

## Paired Original Comparison

Retain both complete original projections, then compare through a fresh request:

```rust,ignore
let report = fresh.inspect::<ReadBinding>(&after)
    .why_changed(&before).reference_only()
    .execute_governed(current_metadata, current_diagnostics)?;
let reason = report.reason(); // ChangedResult or UnchangedResult
report.close();
```

Public metadata uses `execute_with_diagnostic(current_diagnostics)`. Diagnostic
permission remains mandatory. Original binding, parameters, subject, shape, order
and disclosure interpretation must agree. Comparison uses actual Native cell meaning,
optional presence and ordered nested rows. Revisions and producer contacts cannot
substitute for output equality. Missing bodies, withheld slots, changed disclosure
or unsupported shapes refuse rather than publishing a partial `UnchangedResult`.

### Source-only inspection

Inspect a performed source without a query result:

```rust,ignore
let report = fresh.inspect_source::<SourceDisclosure>(&source_receipt)
    .why_performed().reference_only()
    .execute_governed(current_source_metadata)?;
let source = report.source_commit();
report.close();
```

Implement `ApplicationSourceDisclosureBinding<Schema>` and declare
`inspection_disclosure SourceDisclosure,` on the mutation binding. Execution captures
the source's genuine scope and disclosure parameters. Correction metadata and later
business-input reconstruction are not substitutes.

The source's declared query supplies independent current READ and metadata; it is
not executed to reconstruct the source and need not project the authored field.
Public source metadata uses `execute()`. Source-only reports return no diagnostic
fields or producer causes. Their consuming terminal freshly revalidates the actual
session, policies, request and released basis.

### Source/producer connection

For matching source/query disclosure bindings, ask for the performed connection:

```rust,ignore
let report = fresh.inspect::<ReadBinding>(&after)
    .why_changed(&before).caused_by(&source_receipt).reference_only()
    .execute_governed(current_metadata, current_diagnostics)?;
let connection = report.producer_connection();
report.close();
```

A genuine completed handler read must match the source-created Native revision on
the exact full-generation record and installed physical field. The producer's
canonical publication, declared output role and original query-observed revision
must also intersect. Binding compatibility, equal values, a plain head query and
stable reuse cannot manufacture these facts.

### Independently admitted source and query connection

The observed query can omit the input when both originals have their own bindings:

```rust,ignore
let report = fresh.inspect::<CalculatedOnlyBinding>(&after)
    .why_observed().caused_by::<SourceDisclosure>(&source_receipt)
    .reference_only().execute_governed(
        query_metadata, query_diagnostics,
        source_metadata, source_diagnostics,
    )?;
let connection = report.producer_connection();
report.close();
```

Public metadata uses `execute_with_diagnostics(query_diagnostics, source_diagnostics)`.
Each original retains its complete binding and parameters; their values need not
equal each other. Source diagnostics admit the input; query diagnostics independently
admit the output and observable influences. Both current READ and policy sets are
checked in one session before publication. Each member retains its own exact
full-generation subject; both belong to the same original branch occurrence.

The independent route rejects foreign issuer/installed association and source-subject
affinity before connection admission. Its public denial can be coarse `Admission`
with no connection projection. The paired route's later connection validator can
return exact `ForeignSource` for a sibling source. A missing connection projection
is not success and supplies no protected original-admission detail.

### Distinct Original Source And Query Subjects

An ordinary nested query rooted at B can observe A's produced field through an
authored reference while the performed source and producer belong to A. Use the
same `caused_by::<SourceDisclosure>` call and supply query capabilities for B and
source capabilities for A. Each member is freshly resolved, admitted and checked
again before the consuming terminal. A capability for one subject cannot be used
for the other, even when both records have the same entity kind and principal.

The [compiled distinct-subject journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/distinct_subjects.rs)
asserts both root B and nested A keys, the independently expected result and exact
source/producer receipts. A same-kind sibling's nested value cannot supply the
output join. Its [lifecycle courts](../../../worth-query-certification/examples/application_correction_support/causal_inspection/distinct_subjects/lifecycle.rs)
withdraw either real grant, replace B under the same selector with a new Native
generation, and cancel inspection without exposing cause details or retaining a
reader. An original observation remains descriptive evidence through those
refusals; a replacement identity grants no use of the original subject.

This qualified shape is a nested observation of A from B. Applications still
need actual performed input, canonical output and declared role correspondence;
a reference alone cannot establish a producer or authorize another subject.

### Retained Output After Retirement

An original query rooted at a surviving source A can retain its nested observation
of output B. Later lawful retirement of B does not replace the original observed
field or its declared output correspondence. Inspect that original under fresh
source/query admission. A replacement under B's old selector has a different full
generation, even when it has equal field values; a current query cannot borrow the
original supplier.

The [compiled retirement journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/retired_outputs.rs)
uses a real installed producer reading A and publishing B through a Preserve role.
It retains B's actual Native identity and independently expected value, retires B
through an ordinary action, and inspects the original before and after equal-value
replacement. Current grant withdrawal still refuses before cause discovery and
all holders release their readers. This supports inspection of a pre-retirement
original. An empty post-retirement query does not establish a Retire publication
connection; that unsupported shape continues to refuse.

### Original Settled Producer Custody

Read through the genuine settled observation to preserve its supplier association.
A later head query cannot infer it from equal values. Committed publication and
settled read remain distinct. Immutable custody shares the actual publication,
performed inputs, output correspondence and optional Native observation, without
retaining a commit receipt, Ready cell or operational snapshot reader.
An unsettled own-publication continuation may carry one publication candidate
across its admitted source occurrence. It preserves that candidate only when
the successor keeps the full-generation output roster without canonical output
mutation. A changed output resets it, and the successor's own eligible Ready
publication can become the new candidate. This is deliberately conservative for
mixed field origins: there is no per-field history lookup. A connection still
requires the exact observed Native field revision, installed contract, declared
role and canonical field effect; candidate retention alone proves none of them.

### Canonical Absence Evidence

Only a completed absence read and the particular canonical clear/removal that
created its Native absence revision can establish an absent-input connection.
Missing versions, equal `None`, no-op clears and an old absence after
remove/recreate/remove cannot substitute. Field absence, aspect absence, present
values and retired output roles have distinct meanings. Unsupported nested shapes
refuse. Scalar whole-aspect correspondence is never inferred from a field name or
from an aspect containing one field.

### Explicit Scalar Aspect Storage

Use `scalar_aspect_field` to bind an aspect's complete scalar value to one exact
Required field. Declare its parent separately:

```rust,ignore
let schema = schema
    .optional_aspect(Number::reference(), ScalarFacts::reference())
    .scalar_aspect_field(Number::reference(), Value::reference());
```

The declared field supplies its scalar family and executable value binding.
Installation requires exactly that field, Required presence and a primitive scalar
codec. Extra children, optional child presence and structured canonical carriers
refuse with `InvalidAspectShape`. Ordinary `field` declarations keep Struct storage,
including when the aspect has only one field. Storage interpretation participates
in schema identity and value-meaning predecessor comparison.

Use ordinary `DecisionReader::field` and `CandidateWriter::write_field` for the
bound scalar. Writes lower to the installed whole scalar contract. For a Required
child beneath an Optional parent, declare and project an
`ApplicationQueryOptionalAspectFieldRef`; genuine parent absence yields `None`.
An unrelated field name or nested path grants no scalar read or write.

Keep an ordinary producer-input query separate from governed inspection bindings.
For a connected scalar input, declare the bound field as `u64` with
`U64ApplicationValueBinding`. Have the ordinary producer read it and publish its
declared Struct output role on the same full-generation subject. Observe that
output through the settled query, then independently admit the retained source
and query bindings with the inspection call above.
The source receipt, completed producer read, canonical scalar write and original
query output revision must still connect. Equal bytes and no-op writes cannot
supply missing revision or effect evidence. An ABA rewrite does not make a later
occurrence identical to an earlier one. Read, metadata and
diagnostic admission remain independent for source and query subjects. Scalar
storage grants no parent-removal authority or retired-output interpretation.

### Scalar Producer Outputs

Declare a `u64` output with `scalar_aspect_field` and
`U64ApplicationValueBinding`, then use the ordinary producer's completed reads,
`CandidateWriter::write_field` and declared output role. Keep the producer-input
query limited to its actual inputs; projecting the produced output there would
make it part of the source identity.

Build readiness from the completed declaration and the actual output field:

```rust,ignore
let contract = WorthQueryOutputReadinessContractBuilder::for_field(
    declaration, Total::reference(), operation_identity, "total-ready",
    canonical_query, projection_role, strategy, "total-readiness",
)?
.field_dependency(
    declaration, Total::reference(),
    WorthQueryReadinessFieldUse::DeliveryAndEligibility,
)?
.build_typed::<Domain, Operation, Family>()?;
```

The builder retains the declared physical binding and seals it against the
installed catalog. Scalar readiness uses the whole Scalar projection and its
`WholeAspectSet` event. Struct child readiness keeps its field projection and
`FieldSet` event. Do not construct a raw field mask for a Scalar contract or
substitute an aspect name for its bound field.

Bind the resulting node to the producer with its declared delivery ordinal.
Read the Scalar output through the settled observation, using
`ApplicationQueryOptionalAspectFieldRef` for a Required child under an Optional
parent. The source-disclosure and output-query bindings still require independent
current admissions. The connection must find the queried Scalar field's own
Native revision and effect in the producer publication. A changed Struct sibling
or equal Scalar value supplies no missing output evidence. When that revision and
effect intersection is missing, the connection refuses `OutputUnconnected`.

`connection.producer_commit()` identifies the current settled producer publication.
Use `connection.output_publication_commit()` for the publication that actually
created the queried field revision. They may differ when an unsettled producer
has to reobserve its own publication. Only immutable evidence carried through that
same installed producer's selected own-publication continuation is eligible. It
must independently join the original source occurrence and completed input read,
the current output role and full-generation entity, the installed field/contract,
and the observed Native revision and canonical effect. An unrelated source change
or equal output cannot acquire that earlier publication's cause. Current settlement
and fresh admissions still govern use; this descriptive connection owns no reader.

On an inspection denial, `producer_connection_refusal()` gives the coarse
connection refusal. For `OutputUnconnected`, `producer_output_refusal()` also
returns a `WorthQueryProducerOutputConnectionDenial` after current diagnostic
admission. It distinguishes output kind/entity affinity, incomplete observation,
missing or absent Native revision, a revision not created by the selected producer
publication, unavailable installed field, contract revision mismatch, and the
canonical effect (unchanged, cleared, unsupported or ambiguous).

The detail describes the first failed condition after the longest satisfied join
prefix among disclosed candidates. Equally deep failures retain original
role/footprint order; it does not identify every failed candidate. No field names,
entity IDs, versions or business values are returned. Earlier admission, request
and source/input refusals return `None` from `producer_output_refusal()`. For
example, `NativeRevisionNotCreatedByProducer` says the observed field revision
differs from the selected producer publication; it does not identify a different
creator or authorize substituting another receipt. Preserve the refusal and the
original operation's cleanup custody.

The [compiled Scalar-output journey](../../../worth-query-certification/examples/application_correction_support/scalar_input/output.rs)
uses an Optional Scalar with a Required `u64` child and ordinary Preserve output
roles. It exercises changed values, ABA, no-op, independent governance and an
equal Scalar beside a changed Struct sibling through the same Decl/Host route.

### Public Optional Struct Removal

Declare `OperationRemovesAspects<Operation>` with
`worth_query_operation_removes_aspects!`, register `operation_remove_aspect`, and
declare every installed child decision read. `CandidateWriter::remove_aspect`
requires the ordinary completed full-generation target and complete read closure.
A field-write marker grants no parent removal. Unread siblings, Required parents,
wrong aspects, foreign backing and mixing removal with child writes in either order
refuse. Recreate the parent with all required fields.

For example, an Optional `InputFacts` parent with Required `Bias` and `Sibling`
children needs both declared reads and distinct parent-removal authority:

```rust,ignore
worth_query_operation_reads!(EditInputAspect => [Key, Ceiling, Bias, Sibling]);
worth_query_operation_removes_aspects!(EditInputAspect => [InputFacts]);
let schema = schema
    .operation_read_field(EditInputAspect::reference(), Bias::reference())
    .operation_read_field(EditInputAspect::reference(), Sibling::reference())
    .operation_remove_aspect(EditInputAspect::reference(), InputFacts::reference());
```

In the decision phase, complete each child read through `DecisionReader::field`
and propagate a denied or unavailable read. Pass the resulting mutation target
through the ordinary candidate writer's `projected_entity`, then call
`writer.remove_aspect(&target, InputFacts::reference())`. The marker and declaration
alone do not replace those completed reads. A no-op removal cannot create another
absence revision.

Use `ApplicationQueryOptionalAspectFieldRef`, `optional_aspect_field` and
`disclose_optional_aspect_field_by` for a Required child beneath an Optional parent.
Only actual parent absence yields `None`. A present parent missing its Required child
or an unavailable read is a denial. This does not implement
`OptionalApplicationFieldValue` for a Required child.

Register each parent-optional slot in the result shape with
`.optional_aspect_field(...)` and in the disclosure contract with
`.disclose_optional_aspect_field_by(...)`. Project each actual child separately:

```rust,ignore
let bias = row.optional_aspect_field(bias())?;
let sibling = row.optional_aspect_field(sibling())?;
```

Use an ordinary data query for a required producer's input port. A separately
governed inspection query keeps its own metadata and diagnostic admission; it
cannot be substituted for ordinary input admission. Declare the actual input
dependencies rather than adding an unused produced output to that input query.
The producer's completed decision reads and canonical publication still establish
any later causal connection.

### Performed Producer Output Evaluation

A producer may opt into exact output comparison for one Required standard `u64`
field on its exactly-one Preserve role:

```rust,ignore
fn output_equivalence(
) -> Option<WorthQueryProducerOutputEquivalence<Schema, Self::OutputRole>> {
    Some(WorthQueryProducerOutputEquivalence::exact_u64(Calculated::reference()))
}
```

The installed field seal binds the role, complete Native contract and physical
binding. The completed handler must actually read the output field before its
write. Query joins that read to the exact committed Native field revision and
readiness observation, then retains the actual Signal baseline/candidate
comparison before Ready. A missing, conflicting, absent or differently bound
artifact cannot stand in for that comparison.

Inspect the completed evaluation with independent query and source admissions:

```rust,ignore
let report = fresh.inspect::<OutputQuery>(&original)
    .why_observed().evaluated_by::<SourceDisclosure>(&source)
    .reference_only()
    .execute_governed(query_metadata, query_diagnostics,
                      source_metadata, source_diagnostics)?;
let evaluation = report.producer_evaluation();
let decision = evaluation.decision(); // Changed or Equivalent, from actual Signal evidence
```

`output_role()`, `output_field()` and `observed_output_revision()` describe only
that admitted field. Equivalent output retains the Native revision it actually
observed. Use `.caused_by(...)` when the desired claim is creation of an observed
revision; its exact canonical publication requirement still applies.

`same_evaluation_as` compares the retained owner-issued comparison occurrence.
Reopening an existing performed Ready result can retain its original evaluation;
this does not assert another handler execution or comparison. Stable/restored
suppliers cannot issue a performed-evaluation report. The compiled
[Stable input-cutoff journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/output_equivalence/reuse.rs)
checks actual `StableReused`, no live receipt and no Native publication. The
[checkpoint-restoration journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/output_equivalence/restoration.rs)
checks actual `RecoveredPerformed` without recreating the original in-memory
comparison. Both return `NotPerformed` after fresh independent admission; recovered
historical performance supplies no new computed evidence. A source receipt must still
join the producer's actual completed input: an old equal-output occurrence or a
same-shaped sibling cannot substitute for it.

The [compiled parity journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/output_equivalence/journey.rs)
qualifies source `2 -> 4` with output `0` and actual equivalence, followed by source
`4 -> 5` with output `1` and actual change. Independent permissions, original output
projection and exact field revisions gate disclosure. Cancellation, either subject's
grant withdrawal and insufficient request funding refuse the optional explanation;
closing it releases its request funding without retaining another Native reader.
`producer_evaluation_refusal()` exposes a typed refusal only after the required
fresh admission. It exposes no before/after business values.

A late interruption after comparison preserves its original pair in the Delivered
checkpoint. Fresh request admission can resume readiness without another handler,
comparison or Native publication. The [public cancellation/retry journey](../../../worth-query-certification/examples/application_correction_support/causal_inspection/output_equivalence/retry.rs)
qualifies this behavior; reopening the completed Ready result retains that same
evaluation occurrence. A retained comparison supplies no permission to inspect it.

### Native Bridge Assembly Boundary

Use Host's fresh issuer for application receipt inspection. Native backing,
descriptors and digests supply no admission. The generic diagnostic/legacy envelope
path refuses `NativeInspectionAdmissionRequired`; reference-only policy cannot
bypass that fence. Application reports expose no raw Native cells or proof constructor.

## Retained Workspace Diagnostics

The workspace diagnostic audience uses `CausalInspection::for_observation` with an
actual matching `ScopedInspectionBasis`. That basis supplies no application READ,
metadata or diagnostic permission. Use Host for application receipts; a retained
workspace Route cannot bypass its admission.

## Refusal, Cancellation And Retry

Read typed `kind()`, `resource_refusal()`, `canonical_refusal()` and
`producer_connection_refusal()` projections according to the failed boundary.
Missing bodies are not an empty successful explanation. An empty query result
cannot establish output retirement; an unverified supplier cannot establish a cause.

Revocation, cancellation and request expiry are checked before protected lookup and
again at the consuming terminal. An immutable proof is not permanent authorization.
Retain genuine originals and use a fresh request and capabilities for a lawful retry.
Closing a report or owned denial releases its own reservation; it neither undoes the
business operation nor discards another owner's original evidence.


## Rich original query materialization

A new request can materialize an original complete, retained projection after fresh
READ, inspection metadata and Diagnostic field/relation/influence admission:

```rust,ignore
let detail = WorthQueryApplicationMaterializationRequest::family(
    WorthQueryApplicationEvidenceFamily::QueryInspection,
);
let rich = fresh.inspect::<Binding>(&original).why_observed()
    .materialize(detail)
    .execute_governed(current_metadata, current_diagnostic)?;
for row in rich.materialization().original_query().into_iter().flat_map(|p| p.rows()) {
    for field in row.fields() {
        // Borrowed declared output name, canonical result path, and original Native value.
        consume(field.output_name(), field.result_path(), field.value());
    }
}
rich.close();
```

Public inspection metadata uses `execute_with_diagnostic(current_diagnostic)`.
A reference-only report never grants a subsequent rich read. The new request
uses the same inspection session and consuming publication terminal as the
reference path. It shares the original disclosed cells and their original owner;
it does not run a query against the current head. Rows include their original
ordered nested relations. Names come from the exact retained compiled slot
contract. Internal working nodes, predicate sources and operational handles are
not part of this view. The fresh request pays for the new report; close and refusal
release its existing reservation.

A retained empty projection is `Provided` with zero rows. An original captured
without `retain_projection_for_inspection()` reports `OriginalProjectionNotRetained`.
Current diagnostic policy admits before this availability lookup, even for empty
results. Unrequested families report `NotRequested`; a requested family outside
this selected original-query mode reports
`RequestedButNotProvidedBySelectedObservationMode`. This is a selection outcome,
not a Signal retention or expiry verdict. Owner/family references are the actual
freshly assembled Query envelope bindings. The rich report has a distinct versioned
identity bound to the original, policy, requested families and actual projection
outcome. Original-query materialization currently uses the existing
`OfflineInterpretableArtifact` policy. Rich connected-source and performed-evaluation
views remain subsequent Phase 9.9 work.

Cancellation after private projection preparation refuses before any public rich
artifact, without changing business output. Previously admitted descriptive
results remain usable after later grant revocation; every new request rechecks
current access. The older unadmitted generic materialization door continues to
refuse application observation envelopes with `NativeInspectionAdmissionRequired`.
