# Cross-Runtime Causal Inspection

Application code uses `worth-query-decl` and `worth-query-host`; start with the
[application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Use completed query observations to retain the original binding, parameters,
source footprints and optional disclosed projection after business rows drop.
These owners retain evidence without retaining a Native reader or granting a new
read permission. Close an observation or drop its last clone to release its custody.

An ordinary completed bound query can supply its actual opaque observation through
`result.receipt().inspect().observation()`. A fresh Host request admits a reference
report for that exact installed binding with
`request.inspect::<Binding>(original).why_observed().reference_only().execute()`.
Governed metadata additionally requires `execute_governed(actual_capability)`.
The fixed Observed report establishes one original Query reference. A paired
original request separately admits diagnostic comparison of complete retained
output trees, as described [below](#paired-original-comparison). Producer-chain
assembly and richer field materialization remain unfinished. The bounded
[source/producer connection](#sourceproducer-connection) below passes its focused
ordinary source-to-producer-to-query journey. Final evidence review and broader
Phase9 qualification remain open.

### Source-only inspection

The four focused source-only runtime cases pass; final review and whole-Phase9
qualification remain open. A fresh
Host request can select an actual committed source independently of a completed
query, paired comparison or projected input field:

```rust,ignore
let report = fresh.inspect_source::<DeclaredSourceDisclosure>(&source_receipt)
    .why_performed()
    .reference_only()
    .execute_governed(current_source_metadata)?;
let actual_source = report.source_commit();
report.close();
```

`DeclaredSourceDisclosure` implements the existing
`ApplicationSourceDisclosureBinding<Schema>`. Its original source, scope and
captured disclosure parameters resolve against the actual installed declaration
and original schema/package binding. The request selects the current occurrence
of that source's original branch and freshly resolves the full subject and
principal. Missing original disclosure or foreign structural affinity refuses;
no fallback reads a journal, business input or another query observation.

The source's declared disclosure query supplies current READ and its independent
five reference-metadata facets. It need not project the source's authored field,
and it is not executed to reconstruct the source. Public business payload or a
different binding's metadata cannot admit this report. Public source metadata
uses `execute()`; governed metadata requires its own actual current capability.
Diagnostic fields are never examined or returned by this mode, so no
DiagnosticMask permission is borrowed or implied.

The distinct source target enters the same Inspection session, final currentness
checks and consuming terminal as query inspection. Query-owned source reporting
uses the common funded Bridge assembler with a distinct performed-source shape;
it claims no retained Native evaluation or producer. Its report retains the
actual immutable canonical source/disclosure backing and original funding,
without the original receipt, business input, Native reader or parent fact body.
The source receipt can drop after execution. `close()` releases the fresh report's
reservation; cancellation or request expiry preserves the actual owned refusal.
A strongly retained immutable source is not labelled expired merely because its
request expires. Optional evidence retirement/expiry remains a later obligation.

Source-only inspection does not widen the paired source/producer join below.

The earlier connected candidate failed with `SourceInputUnconnected`; that
receipt remains historical evidence. The qualified coordinate repair resolves
both the original Planned handler descriptor and Authoritative source-revision
descriptor through the same installed entity-kind layout and Native contract
binding. Their original authority and paths remain unchanged. Matching that
physical field does not establish a cause without the actual performed read,
full entity generation, exact source-created revision, query-observed revision
and canonical source/output writes. The complete positive journey and extended
internal genuine-producer test now pass. Review033's guaranteed whole-locator
mismatch was repaired; the first predicate of the earlier failure was never
observed, so that finding alone does not establish its sole cause.
Independently bound source-plus-query connection, absent-field joins, actual
suppression and rich materialization remain unfinished.

### Source/producer connection

An explicitly supplied actual source receipt can be checked while comparing two
originals:

```rust,ignore
let report = fresh.inspect::<CalculatedReadBinding>(&after)
    .why_changed(&before)
    .caused_by(&source_receipt)
    .reference_only()
    .execute_with_diagnostic(actual_diagnostic_capability)?;
let connection = report.producer_connection();
report.close();
```

The named binding and capability are application-defined. Governed metadata
continues to require `execute_governed(metadata, diagnostic)`. `caused_by` borrows
a candidate; it reads no source evidence before fresh READ, all six metadata
facets and diagnostic admission. The source must declare its original
`inspection_disclosure` against this exact complete query binding, scope and
parameters. In this bounded lane source and query use that same installed
metadata contract and current scope READ; metadata and diagnostic capabilities
remain independent and are checked before link availability. It does not provide
a general source-only or cross-binding admission path.

This first connection lane requires the query to project the source input field
and the producer's present output field. It joins the original canonical source
write to the producer's actual handler read and pre-effect native field revision,
then joins the
producer's canonical write and declared output role to the query's original
native revision. Entity generation is part of both identities. An authored
same-value write need not create that revision and cannot substitute its own
occurrence for the one actually observed. Values and reporting digests never
establish a connection. A later read B may carry the
original producer publication A; they remain separate coordinates.

The canonical field lookup currently interprets struct sets and field sets only.
Scalar whole-aspect values, missing fields and unsupported paths cannot establish
a connection here and refuse with `SourceInputUnconnected` or `OutputUnconnected`.
The AspectPresence restoration repair does not expand this inspection lookup;
its Native presence coordinate serves the separate correction owner.

Before output normalization or publication, the producer copies only completed
field reads and original field revisions, using its original request funding.
This copy excludes Native publication pins and all other operational facts.
Later owners share those immutable copies; rebased postconditions remain separate.
The connection retains only immutable
source, input, output-role and Native readiness backing with their original
custody. It retains no receipt, mutable lineage record or Native reader; the
source candidate can drop after execution. Its report identity binds the source,
producer, role and actual readiness observation under the fresh Inspection
terminal and request ticket.

`producer_connection_refusal()` preserves typed foreign, unavailable and
unconnected causes. A plain head query cannot infer a supplier. Stable reuse and
restoration do not manufacture a performed producer. `UnchangedResult` remains
an output comparison; the actual readiness decision does not by itself establish
equivalent-output suppression. Source-only admission has the separately exercised
path above; it does not widen the connected journey's same-binding contract.
Independently bound source/query connections, missing/absent field connections,
richer materialization and full Phase9 qualification remain open. The [ordinary source court](../../../worth-query-certification/tests/application_graph/correction/causal_inspection.rs)
uses an installed arithmetic producer and a declared Calculated query through
the two facades. Retained governed reads use the existing Host selected-query
admission and one-shot execution owners; ordinary `execute_governed` rejects a
retained request. The source-only court uses the declared source-completion
program, without a fixed-output delivery obligation. Coordinate qualification
passes all six selected retry cases: three causal application cases and three
Execution field-copy/custody/supplier/reuse cases. The positive performs Primary
5-to-11, one producer contact and query `(11, 23)`, and asserts the actual source,
producer and declared output role. Together with 25 unaffected prior runtime
passes, the selected union is 31/31. The 316 unchanged doctest passes carry from
the prior candidate; they were not rerun. Guidance/context generation and normal
boundary/context checks pass in the isolated Phase9 target with six Cargo jobs.
Final review remains required; these results do not establish Phase9 closure.

Output-readiness delivery also privately retains the actual Native conditional
observation. Its immutable Bridge backing preserves the real Signal decision,
semantic observations, admitted snapshot identity and Signal-issued invalidation
observation, without keeping the operational snapshot reader or reentry seed.
The unique performed invalidation receipt still transfers only once; the
read-only observer shares its original funded backing and grants no execution or
disclosure authority. This observer supplies evidence, not a producer-to-row explanation. See
[Native observation custody](../../../../../../crates/worth-runtime-bridge/README.md#read-only-native-observation-custody)
for ownership, funding and release details.

### Native Bridge Assembly Boundary

The lower Bridge owner can assemble an envelope from its actual readonly
conditional observation. It verifies sealed-owner affinity before projecting
Native evidence, preserves the original observation backing until the last
envelope closes, and derives an invalidation reference only when that original
backing contains an actual performed invalidation. Operational reentry's lack of
a new performed receipt does not erase the original causal observation.

Route-based diagnostics require an actual retained Route;
Native conditional evidence cannot be reconstructed from that route or from
reporting identities. Its summary binds both the causal anchor and the distinct
expected Query observation, rather than treating those different identities as
equal. A summary is descriptive and cannot substitute fresh read authority.

The generic diagnostic inspection basis cannot admit Native envelope
materialization: it returns typed `NativeInspectionAdmissionRequired`. Retained
Native evidence cannot substitute for current READ, metadata or DiagnosticMask
admission.

### Original Query Binding Custody

An ordinary installed query binding carries the actual immutable catalog binding
into its query, including after `into_query`. Completed row and result-set sources
share that interpretation with their original admitted parameters in one capsule.
Cloning or retyping a source preserves it; retaining the application query
observation does not acquire a Native reader. No descriptor or parameter values
are copied for this binding handoff.

Two bindings can name the same query while differing in input, result, scope or
principal meaning. A matching query identity cannot substitute for the original
catalog binding. Queries issued through the raw certification owner retain no
binding, and inspection cannot infer one. The carrier stays opaque: it
exposes neither protected metadata nor permission through the application receipt.
The [Native query court](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_observation/declared_query/tests.rs)
executes two actual same-query bindings and an unbound query, preserving their
distinct custody after operational query bases have closed.

### Original Query Row Evidence

Query completion retains the exact immutable source meaning for every completed
row, once per invocation alongside the original binding and admitted parameters.
Each row source and the result-set source share that capsule. The opaque query
observation therefore preserves the original field and adjacency reads after
business rows and operational query bases have dropped. This capture copies no
business rows or footprint facts and retains no Native reader.

The roster owns fixed pointer storage admitted by the original execution request;
cloning a receipt shares that same backing and reservation. A failed capture
returns the original request refusal or a typed physical allocation refusal,
including its known byte quote. It never publishes incomplete observation evidence.
The [native custody court](../../../worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/retained_parameters/tests.rs)
checks actual backing funding, meaning sharing and release at the last receipt.

Result-set membership remains a separate meaning. Its selection footprint is not
a union of projected row reads: many rows retain their own exact field footprints,
and an empty result has an empty row roster with its original selection evidence.
The [many and empty query court](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_observation/declared_query/tests.rs)
provides examples of both through query admission and completion. Reading retained
footprints requires independent metadata admission; capture grants no permission
and establishes no producer-to-row association.

### Original Settled Producer Custody

An actual output settlement attaches one opaque readonly producer capsule to
its retained read. A query issued through that exact read shares the capsule with
its original parameters and row evidence. Retained, shared and batch query routes
preserve the same supplier; a plain head query cannot infer it from equal source
meaning. A different actual query basis drops the association.

The capsule distinguishes the producer's committed publication A from the settled
read B, which can follow an unrelated legal publication. It retains the original
committed envelope, exact settlement identity, output correspondence, optional
source-disclosure meaning and genuine Native conditional observation. It shares
only original immutable backings: no commit receipt, Ready cell, operational
Product observation or Native reader is retained by the query observation.
Its concrete allocation, including Arc header alignment, is admitted once by the
original execution request; clones share that reservation.

Stable input reuse carries its real stable settlement identity without fabricating
Native execution. Checkpoint restoration carries its original restored identity
without manufacturing an observer. The
[performed and later-basis court](../../../worth-query-execution/src/domain_computation/primary_graph/output_lineage/ownership_oracle/producer_observation.rs),
[installed input-cutoff court](../../../worth-query-execution/src/domain_computation/primary_graph/output_lineage/ownership_oracle/optional_aspects/producer_observation.rs)
and [checkpoint court](../../../worth-query-execution/src/domain_computation/primary_graph/output_lineage/ownership_oracle/optional_aspects/retained_recovery.rs)
exercise those actual production routes.

This is supplier custody, not proof that a result row or field was produced by
that supplier. Fresh metadata admission must precede even checking its availability.
Connecting a cause still requires exact entity generation, field, performed
revision and declared output-role overlap. A matching entity, aspect or value
alone cannot establish that association.

### Original Disclosed Projection Custody

Ordinary observations retain source footprints by default. A caller can explicitly
request the actual complete Native disclosed field/relation tree as well:

```rust,ignore
let result = request.query(intent)
    .retain_projection_for_inspection()
    .execute()?;
```

The one-shot wrapper also delegates `execute_governed` and `execute_approved` to
their existing admission owners. It exposes no subscription or continuation door.
The original Vec is moved once into an immutable shared backing after disclosure
stripping; the ordinary application decoder borrows those same nodes. Its clear
drops only its share. Original row, result-set and opaque observations share the
same backing and original-request reservation, without retaining a Native reader
or the application DTO. The effective decision for every declared slot, including
withheld slots, shares its original compiled slot key in funded fixed storage;
the retained binding supplies classification and required-disclosure meaning.
Withheld fields therefore remain distinct from disclosed absent values.

Bound empty results retain a genuine empty tree. Unbound queries and orbit or
component payloads represented outside the tree return typed
`ProjectionInspectionUnavailable`; an actual truncated or continuation tree
returns `ProjectionInspectionIncomplete`. Failed decoding or session completion
publishes no completed observation. The
[original projection courts](../../../worth-query-execution/src/domain_computation/primary_graph/application_query/projection/retained_projection/tests.rs)
use actual query completion and field publications, including an equal-value
rewrite, and check final-clone release alongside unchanged ordinary decoding.

A completed read establishes an observation, not a changed result. The captured
payload describes Native cell values/presence, ordered rows and nesting. Revision
changes, producer contacts and equal source identities do not establish payload
equality or a change. Private dependency provenance and arbitrary application DTO
comparison are outside this retained meaning.

Capture grants no inspection permission. Current READ, metadata and DiagnosticMask
admission must precede availability checks or body traversal. An observation
without this explicit body cannot recover it by rerunning the source. Closing the
observation releases its share of capture custody; the original reservation ends
when its final owner drops.

### Optional Query Inspection Metadata Meaning

The declaration facade exposes
`ApplicationQueryInspectionDefinition<Schema, QueryBinding>::new(metadata_contract)`
and the schema builder's `application_query_inspection(definition)`. This attaches
optional metadata meaning to the existing complete query binding; it requires no
additional inspection marker. See the
[typed declaration and installation court](../../../worth-query-installation/src/application_principal_binding_tests/inspection.rs).

`ApplicationQueryInspectionMetadataRef<Schema, QueryBinding>` selects one of six
closed facets: `ObservationIdentity`, `EvidenceLinks`, `EvidenceCounts`,
`EvidenceAvailability`, `Explanation` and `DiagnosticFields`. A governed contract
uses `disclose_inspection_metadata_by` for every facet exactly once. Counts need
`Count` influence and the other facets need `Explanation`. Each rule belongs to
the full binding, including its input, query, result, scope and principal meaning;
matching names cannot transfer a rule to another binding. Explicit public metadata
uses `ApplicationQueryDisclosureContract::public()` without rules.

Installation checks every definition against its actual compiled binding before
resolving metadata capabilities. Governed metadata must name an installed
capability; an absent capability returns `InvalidInspectionDefinition`. A valid
optional definition installs the independent metadata obligation for fresh Host
inspection. Definitions remain optional for ordinary query execution; a binding
without one cannot admit this inspection lane.

Payload disclosure grants no metadata access. A diagnostic-fields metadata rule
grants no field values: those require the separately admitted current subject's
diagnostic mask. Portable transport supports member45/v1 and selector8/v1 while
preserving unextended schema bytes and identity.

An application mutation may independently declare its original source-disclosure
meaning with `ApplicationSourceDisclosureBinding<Schema>` and the mutation macro's
`inspection_disclosure Binding,` clause. The binding names the actual source and
installed disclosure query, and maps the original typed input to query parameters.
Installation checks that query's scope and principal interpretation against the
source. `ApplicationQueryBindingReference` is the portable descriptive association;
it grants no permission and must resolve against the actual installed inventory.

Original source capture retains the genuine scope and admitted disclosure
parameters on the source receipt's existing owner. Accounted requests, including
an actual accounted ancestor, fund framework retention through their original
execution request; an opaque application
parameter mapper's returned heap is admitted after materialization, before
framework retention. Unaccounted serial requests keep their native posture and
original control checkpoints, without a fabricated accounting reservation.
Inspection alone retains neither the business input nor a correction contract.
Correction keeps its additional input/inverse custody where required. These
captures retain evidence only. They grant neither a new query read nor metadata
disclosure, and they do not establish a producer-to-row cause.

## Fresh Application Inspection

Use the same ordinary request and its actual bound query result:

```rust,ignore
let result = request.query(intent).execute()?;
let original = result.receipt().inspect().observation()
    .ok_or(ApplicationError::ObservationUnavailable)?;
let explanation = request.inspect::<InstalledQueryBinding>(original)
    .why_observed()
    .reference_only()
    .execute()?;
// A descriptive original observation identity and this fresh report identity.
let observation_id = explanation.observation_identity();
let report_id = explanation.report_identity();
explanation.close();
```

The binding and application error above are application-defined. The
[Host example](../../../worth-query-host/tests/application_runtime_surface/inspection.rs)
uses an actual typed query, installed principal binding, admitted authentication
adapter, ordinary publication receipt and this exact request entrance. Its
[declaration](../../../worth-query-host/tests/application_runtime_surface/inspection_model.rs)
registers the query binding and its explicit public inspection metadata together.
For governed metadata, supply the actual currently admitted capability to
`execute_governed`; it is revalidated independently of the original payload
contract. The [capability example](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_inspection/tests.rs)
shows admission, revocation and public metadata as distinct cases.

`observation()` returns `None` when the completed lane supplied no retained
observation. The borrowed capsule grants no permission and exposes no payload,
links or counts. To drop business rows first, retain a clone of the actual receipt
and borrow its observation afterward. The opaque observation shares its original
immutable source capsules and their original custody. Cloning the Publication
receipt also copies owned descriptive strings and basis projections; those copies
are separate from shared observation storage. The borrowing getter allocates
nothing. An opaque observation retains no operational reader.

Fresh admission checks the original runtime, schema and exact installed binding
before selecting its current Product branch occurrence. It resolves the current
principal and original scope selector, checks the full subject identity and
current READ authorization, then independently admits `ObservationIdentity`,
`EvidenceLinks`, `EvidenceCounts`, `EvidenceAvailability` and `Explanation` before
looking up protected target metadata. Public payload visibility cannot supply
metadata admission. This fixed report never exposes `DiagnosticFields`.

The [subject identity courts](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_inspection/tests/scope_replacement.rs)
exercise real replacement and same-slot reuse after Native reclamation. A new,
readable generation with the same selector cannot inherit the old observation:
fresh inspection checks the original full subject identity before its READ gate.
The reclaiming court supplies the actual Native profile before installation and
uses ordinary canonical deletion, maintenance and creation without allocator hooks.

A single observation yields `ObservedResult`. `lower_evidence_posture()` is
`MissingOptional`: lower causal connections are unestablished. It does not claim
that no producer exists. Matching values, field revisions, Route identities or
producer contacts cannot establish Changed or a producer-to-row cause.

The real Inspection read session checks current authority before report creation
and before its consuming terminal. Fresh Product, issued identity and basis guards
end before the descriptive result publishes. Cancellation is checked again at
publication. The same original execution reservation covers preparation, the
report, completion and any owned refusal; `close()` or drop releases it. The
[funding courts](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_inspection/admission/materialization/tests.rs)
exercise genuine report-memory refusal, while the Host journey checks reader
release and cancellation on a reused request. `SystemAllocation` delegates to the
installed World's execution policy; it does not promise unaccounted work.

Inspection denial provides a coarse `kind()` without protected metadata. Borrow
its actual `resource_refusal()`, `execution_stop()`, `request_interruption()` or
`canonical_refusal()` when present. These preserve their original typed causes
without formatting protected diagnostics or cloning owner evidence. The denial
keeps its original reservation until it drops.

## Paired Original Comparison

Retain the complete original disclosed trees when executing both ordinary queries
with `retain_projection_for_inspection()`. The governed and approved one-shot
wrappers preserve their ordinary capability admission. Comparison cannot recover
an uncaptured body by rerunning the query.

```rust,ignore
// Both results came from ordinary executions with retained original projections.
let before = before_result.receipt().inspect().observation()
    .ok_or(ApplicationError::ObservationUnavailable)?;
let after = after_result.receipt().inspect().observation()
    .ok_or(ApplicationError::ObservationUnavailable)?;
let comparison = request.inspect::<InstalledQueryBinding>(after)
    .why_changed(before)
    .reference_only()
    .execute_with_diagnostic(current_diagnostic_capability)?;
let reason = comparison.reason(); // ChangedResult or UnchangedResult
let before_report = comparison.before_report_identity();
let after_report = comparison.after_report_identity();
let paired_report = comparison.report_identity();
comparison.close();
```

The binding, error, results and actual capability above belong to the application.
The [complete Host journey](../../../worth-query-host/tests/application_runtime_surface/comparison.rs)
executes two genuine governed queries, borrows their published receipt observations,
and independently admits the current diagnostic capability through Decl/Host.
Its [declaration](../../../worth-query-host/tests/application_runtime_surface/comparison_model.rs)
and [ordinary grant](../../../worth-query-host/tests/application_runtime_surface/inspection_capability.rs)
install the real query, metadata and capability meanings together.

Public inspection metadata uses `execute_with_diagnostic`. Governed metadata uses
`execute_governed(current_metadata_capability, current_diagnostic_capability)`;
these are separate actual admissions. The fresh Inspection session checks current
READ and all six metadata facets, including `DiagnosticFields`, then admits the
current subject's actual DiagnosticMask and `Explanation` influence for every
relevant result field and internal predicate/ordering field. Relation presence
requires its own declared slot rule and `Explanation`. Public payload visibility
has no diagnostic contract and refuses this entrance.

Runtime, schema, actual installed binding, branch occurrence and full subject
generation can reject foreign originals before protected lookup. Cross-original
parameter comparison, body availability, disclosure decisions and cell traversal
occur only after current READ, metadata and diagnostic admission. Both original
parameters and complete effective slot decisions must agree with the current
interpretation. A withheld original cannot establish complete output equality.

The supported shape is a complete one-shot Native field/relation tree. Comparison
uses actual Native cell meaning, optional absence/presence and ordered nested
rows. Equal authored rewrites remain `UnchangedResult`; revisions, source IDs,
predicate provenance and producer contact do not enter output equality.
Root-path, orbit, component, continuation and live shapes return a typed
unsupported refusal.
Missing capture, changed disclosure interpretation or a partial original refuses
rather than publishing a partial `UnchangedResult`.

The result borrows both originals and owns two genuine Query-reference reports
plus a descriptive Query parent bound to those originals, report identities and
reason. One actual Inspection terminal releases current readers before publication;
the original request payer funds preparation, comparison stack, reports and owned
refusals. Close or drop releases that custody. No Native cells or producer-to-row
cause are exposed. The [Native courts](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_inspection/tests/comparison.rs)
exercise changed/equal writes, optional clears, nested order, policy-gated empty
results, actual diagnostic-capability revocation before missing-body lookup, and
foreign parameter/binding inputs.

## Retained Workspace Diagnostics

The older `WorthQueryWorkspace` diagnostic builder remains a separate audience.
Its `CausalInspection::for_observation` takes a diagnostic observation and an
actual matching `ScopedInspectionBasis`; it supplies no application READ or
metadata permission. Its retained Route reporting and support/planning contracts
remain available to that audience. Both generic legacy Native-envelope and new
Query-observation report admission stay closed: an old diagnostic basis cannot
bypass the fresh application issuer, even with reference-only policy.

Use the ordinary Host receipt entrance above for application queries. Optional Native materialization remains unavailable through this Host entrance.
The source-only path and paired connection candidate above retain their stated limits. The existing Native backing alone cannot
open those reads.
