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
The report establishes one original Query reference. Producer-chain explanations,
Changed comparisons, diagnostic fields and richer materialization are unavailable
through this Host entrance.

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

## Retained Workspace Diagnostics

The older `WorthQueryWorkspace` diagnostic builder remains a separate audience.
Its `CausalInspection::for_observation` takes a diagnostic observation and an
actual matching `ScopedInspectionBasis`; it supplies no application READ or
metadata permission. Its retained Route reporting and support/planning contracts
remain available to that audience. Both generic legacy Native-envelope and new
Query-observation report admission stay closed: an old diagnostic basis cannot
bypass the fresh application issuer, even with reference-only policy.

Use the ordinary Host receipt entrance above for application queries. Optional
Native materialization and performed-source/producer-chain inspection are
unavailable through this Host entrance. The existing Native backing alone cannot
open those reads.
