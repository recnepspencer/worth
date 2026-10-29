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
ordinary source-to-producer-to-query journey. Reviewer043 cleared that prerequisite
in commit `8396646380`; broader Phase9 qualification remains open.

### Source-only inspection

The four focused source-only runtime cases pass in the reviewer043-cleared
prerequisite `8396646380`; whole-Phase9 qualification remains open. A fresh
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
checks and consuming terminal as query inspection. A completeness
correction now returns the exact existing session proof from a successful fresh
currentness check directly into terminal construction. Intermediate checks discard
their proofs; the terminal still revalidates independently at use. The preceding
check-and-stored-proof flow was logically guarded; no bypass was demonstrated.
The proof remains owner-issued for its exact session, plan, basis and target and
is consumed with the actual basis release. It is not permanent authorization.
InspectionTerminal and CompletedInspection retain the same consuming progression,
request cancellation/expiry checks, separate source/query policies and original
payer. Reviewer056 cleared this correction; five focused Execution courts and
eight application courts pass alongside the ordinary Operand authoring repair.
Final evidence review and commit remain pending.
Query-owned source reporting
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
The independently admitted connection below passed focused qualification and
reviewer049 final review, committed as `21674b9dac`.
Absent-field joins, actual suppression and rich materialization remain unfinished.

### Independently admitted source and query connection

The same Inspection lifecycle admits an actual source and one original query
under different installed bindings. Its focused runtime, affected doctests and
normal gates and reviewer049 final review passed in `21674b9dac`. The earlier
qualified paired path remains available below.

```rust,ignore
let report = fresh.inspect::<CalculatedOnlyBinding>(&after)
    .why_observed()
    .caused_by::<EditDisclosure>(&source_receipt)
    .reference_only()
    .execute_governed(
        query_metadata, query_diagnostics,
        source_metadata, source_diagnostics,
    )?;
let connection = report.producer_connection();
assert_eq!(connection.source_commit(), source_receipt.commit_reference());
report.close();
```

`EditDisclosure` is the source's actual `ApplicationSourceDisclosureBinding`.
The source and query each supply their own original complete binding and captured
parameters; their parameter values need not equal each other. The source query's
admitted diagnostic fields must include the actual input being explained. That
input need not appear in the observed query. Query diagnostics must independently
admit the original output field and every predicate/influence used by that query.
Public metadata uses `execute_with_diagnostics(query_diagnostics,
source_diagnostics)`; it cannot admit a governed metadata contract.

Current query READ, metadata and diagnostics precede source-disclosure lookup.
The original source scope is independently re-resolved and its own READ, all six
metadata facets and diagnostic contract are admitted before source facts or
output links are examined. Both policies bind to their distinct immutable targets
in one fresh Inspection session. The session rechecks both READ decisions and
both sets of capabilities before publication. This bounded connection requires
the same full subject, current principal and original branch occurrence; another
subject or incarnation refuses. Original binding, parameter, runtime and schema
identity cannot be supplied by a descriptor, a retained source-only report or an
output query's projection.

The producer's genuine completed handler read and original Native field revision
must intersect the exact canonical source write on the admitted original full
source subject. A projected entity kind does not authorize another entity's input. Installed coordinates resolve
both original descriptors without changing their authority. The output still
requires the actual full-generation role, producer canonical write and the
query-observed Native revision. An empty output query, same-value source rewrite,
plain head read or stable reuse supplies no replacement evidence. This remains a
present-field struct/field-set path: absence and whole-aspect scalar support are
not added here.

The connected report carries its actual source/producer/role and original
readonly Native backing through the existing Bridge report and consuming
terminal. It retains neither mutable runtime authority nor a Native reader or
parent fact body. `with_allocation_policy` uses the fresh request's actual lease;
original evidence keeps its original payer. Closing the report releases the
fresh reservation. Source-only inspection continues to report the actual source
without inventing a producer.

Five new grouped application cases cover the Calculated-only public positive,
four independently admitted capability roles, wrong original bindings and owners,
equal rewrites, genuine no-match query parameters, current revocation,
cancellation/expiry and real lease release. All 12 selected application cases
(five new and seven affected) and seven Execution admission/currentness controls
pass from actual source-owned artifacts. The affected Execution/Host doctests
pass 233/233; guidance/context generation and boundary/context checks pass with
verified source-owned tools and no generated content change. Application acquisition
r1 ended101 on ambiguous observation imports and an uninferred closure parameter;
two explicit fixture annotations fixed those compiler joins without changing
assertions or production behavior. The failed receipt is preserved. Reviewer049 cleared those joins and final
evidence, committed as `21674b9dac`. Prior
31-case/316-doctest evidence remains attached to prerequisite `8396646380`. Actual suppression,
rich materialization, optional evidence expiry, private handoff and Phase9
completion remain open.

### Canonical absence evidence (bounded qualification)

The terminal pre-correction candidate has 17/19 runtime passes: Native2,
application9 and Execution6. Its two new public absence journeys fail at original
source publication with `Commit(Aborted)`, before producer/read/connection checks.
Those original failed receipts remain preserved. The repair below now passes
eight application cases, including both original absence journeys, and five
Execution currentness/terminal/funding courts. Expected output and strict
source/producer/role assertions remain unchanged. This establishes the bounded
optional-field absence connection; final review and commit remain pending.

The implementation keeps completed present,
field-absent and whole-aspect-absent handler observations distinct in the existing
pre-normalization performed-field capture. The original NodeSource query footprint
still supplies its actual Native revision. Only `Some(revision)` with the source
commit's exact version can connect; `None` is unavailable, never revision zero or
proof of absence. Full-generation subject, independently admitted source/query
policies, canonical producer publication, output role and observed output revision
remain required. Output retirement is still closed; an empty result is no substitute.

Native's `PublishedAuthoritativePatch::field_effect` interprets one original
canonical operation against the actual installed aspect contract and binding.
A field set or present whole struct/scalar set is distinct from a field clear,
a whole struct replacement omitting an optional field, and whole-aspect removal.
Scalar correspondence requires the binding's exact `EntityField`; aspect names
and field positions cannot supply it. Mismatched contract/binding, unsupported
shape, conflicting writes and paths below Native's top-level granularity remain
unavailable. Query consumes that interpretation through its installed coordinate
rather than duplicating a present-value search. Equal bytes do not establish
revision equality or which source created it.

The existing numeric court adds an optional arithmetic operand to the real source
query and producer handler. The producer validates its actual optional read before
publishing Calculated. New grouped cases exercise present -> cleared -> present,
absence ABA and no-op clear against actual original receipts and independently
admitted Calculated-only observations. Existing expected rows now explicitly say
`operand: None`; old assertions are retained. Both absence cases now pass actual
source publication, producer optional read, source-created absence revision,
canonical output role and query revision, ABA/no-op and terminal/release checks.
The no-op court checks stable reuse and does not claim Signal suppression.

Native owner fixtures separately exercise scalar whole set/removal/recreation,
exact physical binding, struct replacement/field clear, and genuine absence
revisions. The installed optional-aspect court checks that its actual completed
absence observation survives immutable field-only capture. These are owner proofs,
not a public whole-aspect-removal journey: CandidateWriter currently has no
whole-aspect-clear operation or declared authority. That public authoring/connection
journey remains open. A required field beneath an optional aspect is not silently
reinterpreted as a cleared optional field. Query-installed scalar end-to-end
qualification also remains open; the scalar owner fixture alone does not close it.
No new report policy or admission door is added. Original payers, severed parent
facts, no Native pin, currentness and the consuming terminal are preserved.

The observed source failure is `Commit(Aborted)`. Source review of the receiving
owner identifies a concrete incompatible contract in that failed candidate:
Operand edits used EditBinding, whose RecordedEdit inverse demands a
read-and-mutated Primary pre-image. The retention owner requires that exact demanded field to be mutated;
an Operand-only write cannot satisfy it. The runtime receipt does not expose
that internal denial, so MissingDemandedField is not claimed as dynamically
observed or the sole cause. Neither a no-op Primary write nor weakened inverse
retention is an acceptable repair.

The correction gives [Operand authoring](../../../worth-query-certification/examples/application_correction_support/operand_edit.rs)
a typed OperandEdit input, EditOperand operation, OperandBinding and its own
ApplicationSourceDisclosureBinding. The program-required, no-aftermath operation reads
Key, Ceiling and Operand and writes only Operand; the plain mutation lane refuses
ApplicationProgramRequired before publishing a source. NumericContribution installs
its real handler; OutputProgram declares its fixed-output action and maps the
original source key into the existing required calculation demand. The ordinary
performed lane captures source disclosure from that binding's actual original
input, without an invented inverse or Primary pre-image. Original input identity,
source occurrence and parameters remain sealed by the existing admission/capture
owners. Primary's RecordedEdit inverse is unchanged, and Operand no longer has
an enum path or read/write authority under EditBinding.

Both original Primary and Operand source journeys start and settle the same real
required-output owner. The two absence courts retain 15/11/18/11, exact source
occurrence, output-role, ABA/no-op and reader-release expectations; they now name
Operand's disclosure. The positive is paired with refusal of Primary's disclosure
for the genuine Operand receipt, before cause lookup, and a typed refusal of the
plain lane that omits its program delivery. Independent source/query
READ, metadata and diagnostics are unchanged. No Native or Query production
change accompanies this authoring correction. Reviewer056 cleared its Source.
Application r1 ended101 on an ambiguous observation glob import; one explicit
application-entry import resolved it without changing semantics or assertions.
Fresh r2 acquisition and all eight selected cases pass; the failed r1 receipt remains.

The pre-correction candidate's affected doctests pass 257/257 (Native24,
Execution/Host233). Its source-owned guidance/context generation and
boundary/context checks pass with no generated delta. Those receipts remain
attached to their actual compiled snapshot, alongside all 17 successful cases.
Unchanged Native2, leaf-copy/supplier/absence evidence and doctests carry. The
new authoring and proof-return work is covered by application8 and Execution5,
rather than replaying the old portfolio. Native production, owner fixtures,
disclosure policy and public doctest contracts are unchanged.
Native acquisition r1 and application acquisition r1 compiler failures are kept.
The narrow joins pass the real request into fork helpers, assert successful index
registration, and add three explicit absent Operand row literals. Application r2
then failed all eleven cases on a stale conditional fixture contract; aligning
its optional Operand declaration with the installed schema preserves its
Primary-only mask, predicate and admission. Application r3 passes nine existing
cases; r4 adds only a strict failure diagnostic and retries the two failed cases.
All failed receipts remain preserved. Two actual source-owned artifacts now
pass application8 (two absence cases, three original causal controls,
required-output correction, source-only cancellation/expiry and independent
governance) and Execution5 (observed/comparison currentness, fresh capability,
final cancellation, report refusal and original funding). Both artifacts report
Fresh=false with actual UUID, executable hash and unique exact lists. Rust1.99.1,
six jobs and 64MiB stack use the isolated Phase9 target. Verified source-owned
guidance/context generation and boundary/context checks pass without generated
content changes. No broad doctest or unrelated campaign ran. Reviewer056 cleared
the authoring/proof Source; the post056 import and final evidence await final
persistent review. Prior proof remains attached to `21674b9dac`
and `8396646380`; none of these results certify Phase9.

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
path above. The independent source/query connection is qualified for present
fields at `21674b9dac`; the canonical-absence increment above remains blocked.
Public absent-field connections, richer materialization and full Phase9
qualification remain open. The [ordinary source court](../../../worth-query-certification/tests/application_graph/correction/causal_inspection.rs)
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
