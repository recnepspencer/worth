# Milestone 9.17.6: Dynamic Workflow Authoring And Execution

> **Status:** Planned successor to [9.17.5](./milestone-9.17.5.md).
> [9.17.4](./milestone-9.17.4.md) supplies typed graph authoring, ordinary execution,
> publication return paths and migrated existing consumers. 9.17.5 supplies exact
> branch-local program adoption and custody disposition. Neither waits for this
> milestone's new user-authored definition/instance product.

## Goal, Entry And Completion

An authorized user composes a bounded workflow from installed operations, queries,
conditions, assessments and approval contracts; validates and publishes its definition
as branch-local data; starts an instance; and observes real admitted effects,
rejection, revision, recovery and cleanup through the same application graph.

One canonical `AuthoredWorkflowDefinition` is the authoring product. The typed Rust
graph builder, `worth_query_workflow!` declarative surface, reusable workflow
components and UI/API/AI authoring commands all lower into that same bounded data and
the same validator. Friendly syntax cannot mint validation, publication or execution
authority, and no authoring surface owns a parallel graph or execution path.

Published definition truth, compiled execution meaning and authority for one concrete
transition are distinct. A published definition derives a rebuildable
`CompiledWorkflowDefinition`; each actual step requires a fresh owner-issued
`AdmittedWorkflowTransition`. Neither compiled meaning nor instance status can execute
an operation, and execution never reinterprets the authored graph to recover decisions
already fixed by compilation.

A definition is not a runtime installation or grant of action authority. An instance
is not a generic task runner. Static feature contracts, Query planning and managed
execution, Bridge/Signal eligibility and World publication remain the existing owners.

Complete one proposal -> two required assessments -> approval -> real application
journey in Phase 1 through the graph-shaped public authoring surface. Then finish
branching, bounded retries, definition revisions, branch forks and program adoption.
A graph editor UI is not required; the public typed authoring API and real product
command surface are required.

From Phase 2, geometry-scale execution and concise kernel authoring are completion
requirements. Phase 2 establishes their foundation before the later control-flow,
evolution and closure work builds on it. These additions do not reopen Phase 1;
corrections to inherited implementation belong to Phase 2 or the affected later phase.
Phases 2.1-2.6 close measured House build/edit/checkpoint/reopen bottlenecks before Phase 3 relies on geometry-scale execution.

Closure includes safe A/B definition coexistence, fresh approval/source validation,
independent required inventory, exact step idempotency, bounded execution, partial
effects, branch/program currentness, instance migration and every resource lifecycle.
All new behavior consumes the lower proofs from 9.17.4/9.17.5. No duplicated runtime,
string-node authority, reimplemented admission or parallel publication path.

## Current Boundary And Reuse

- Query managed_run/workflow_graph_execution.rs owns provider-backed read-graph
  execution, provider step progression and completed/paused outcomes; it is not an
  authored-application workflow executor.
- managed_run/workflow_yield_* and provider_session/readmission/workflow.rs own
  suspended/yielded resource custody and fresh provider/session readmission.
- artifact_owner workflow authority and convergence_epoch workflow cleanup own
  existing artifact/progression responsibilities.
- Primary graph conditional_operation, performed relational change delivery and
  application_output_demand route real changes, eligibility and exact settlement.
- 9.17.4 binds existing fixed domain workflows into ordinary application meaning.
  It does not establish runtime-authored application definition and instance storage.
- 9.17.5 supplies branch-affine program compatibility, prepared adoption, live-object
  dispositions and exact recovery. This milestone adds its new definition/instance
  participants to that boundary, not a competing adoption engine.

Definition compilation lowers to installed mutation, output-demand, managed-read and
change-delivery contracts. New primary_graph/workflow code owns branch-local definition,
instance and proposal/approval meaning; it does not own another queue, worker
scheduler, provider session or authoritative runtime graph.

Reuse follows semantic ownership and demonstrated cost, not representation uniformity.
Relational already supplies bounded kind-specific adjacency/frontier reads, planned
traversal and derived indexes; workflow observation already uses native adjacency.
Select the narrowest fitting access contract, preserving exact snapshot/branch basis,
negative observations and pre-allocation work bounds. A generic traversal packet is
not automatically preferable to a bounded frontier read. Local authoring indexes and
compiled transition tables are permitted; another truth, admission or scheduling
owner is not. Do not require publishing a temporary graph just to validate pure input.

## Decisive Production Journey

### Authored definition to real application

Through the installed public entry, author and admit a workflow that prepares a
proposal, obtains two required assessments, requests approval and applies a declared
operation. Its definition is data composed from installed capabilities.

Author the same semantic graph twice: once through the typed primitive builder and
once through `worth_query_workflow!` with one reusable `RequiredGeometryReview`
component. Both must lower to the same canonical definition identity, expanded node
occurrence paths, typed connections, resource contract and validation result. A
UI/API-style command authoring adapter builds the same definition without calling the
macro or private builder mechanics. Changing a component occurrence, port binding or
retry bound changes canonical meaning; changing only authoring syntax does not.

- Start an instance at definition A. Publish B while A awaits approval. New
  instances use B; the old instance stays bound to A. Change a proposal dependency:
  prior evidence cannot approve/apply the changed proposal. Duplicate wakes and
  retries cannot perform a step twice.
- Revoke the approver or remove a required installed capability. Deny before
  effects. An unauthorized node hidden behind a condition cannot evade definition
  contract validation; actual action authority is still freshly checked at execution.
- Revoke the starter, advance as another authorized principal and attribute the effect
  to that actor. An unauthorized pumper observes `AwaitingActor`; a wake without a
  request causes zero provider/World contacts. Ordinary mutation cannot bypass a
  workflow-authority port, and a definition cannot route around its approval source.
- Remove an assessment supplier while its obligation remains applicable. Completion
  becomes unmet, not vacuously passing. Required executable ports cannot be left
  unbound at program adoption.
- Exercise rejection, bounded revision/retry, cancellation before and after effects,
  and exact recovery. Separately committed steps are not one rollback transaction.
- Adopt an incompatible program with a running instance, retained reader and
  unpublished effect. Deny missing dispositions, then accept owner-validated
  migration/retirement and recovery custody on exact branch coverage. Old
  candidates cannot publish after activation.

Use the real `worth-proprietary` CAD consumer for the first endpoint:
`worth-cad-entry` declares the reviewed-geometry vocabulary,
`worthy-house-application` composes and runs it through the ordinary Query host
facade, and `worthy-house-certification` owns the cross-repository product court.
Do not substitute an in-repository CAD facsimile. Public WORTH remains independent
of proprietary crates; the proprietary workspace consumes a pinned candidate public
revision through public facades. Add Bank's real approved business-payment/process
journey for non-geometric authority and exactly-once outbound effect. No test-only
callback substitutes for apply or approval authority.

The proprietary court also exercises the completed 9.17.5 boundary: start a reviewed-
geometry instance under P0, leave it awaiting evidence or approval, prepare P1, and
require the workflow participant's exact carry, migrate, retire or cancel disposition
before adoption can publish. After P1 activation, complete the lawful disposition and
observe P1 geometry/output; a sibling branch remains callable under P0. A stale P0
transition, copied approval or missing live-instance inventory must fail. This is one
consumer journey across the two contracts, not a reopening of 9.17.5.

### Exact evidence under separate publications

Publish a proposal for source S and required inventory {geometry, cross-feature check}.
Accept the first assessment, reject the second, and observe incomplete readiness.
Retry the second under S; its exact evidence settles the inventory. Advance unrelated
model data between acceptances: valid source-local evidence still combines.
Change a consumed source relation away and back before approval: native ABA makes
prior evidence stale even when values compare equal.

Add a newly applicable required subject before the join. Old coverage cannot count
as complete. Remove the installed supplier under an admitted program change:
requirements remain, definition/instance dispositions apply, and missing evidence
never becomes a passing empty set. Finish a failing assessment: settlement may complete,
but the approval/application guard must reject if the declared policy requires passing.

The oracle states required subjects from the independent authored requirement model,
not by enumerating whatever assessment outputs currently exist. Inspect actual
publication results, source versions and product output; log text alone cannot prove it.
With one, ten and one hundred waiting instances bound to a source, editing it performs
zero workflow contacts; later admission pays only for its bound dependencies.

Collect one required signature as soon as its declared subject is ready, before the
control-flow point that consumes it. Advance, navigate Back and reach that point again:
unchanged compatible evidence remains available. Edit one covered subject in a
multi-subject proposal and invalidate only its affected coverage; an unrelated edit
preserves the independent evidence. A new transition occurrence alone does not stale
evidence. Fresh authentication is bound to the exact signing intent and its declared
age/reuse policy; stale or differently purposed authentication cannot sign.

### Definitions, forks and program changes

Start instance I on definition A in branch X. Publish definition B on X. New instance J
uses B; I stays bound to A. Branch Y retains its selected definition/program basis.
I's approval does not authorize J or a Y instance even with equal visible subject IDs.

An explicit definition retirement prevents new starts but preserves admitted old
instances until their owner-approved disposition. Branch fork preserves historical
facts, not executable grants. A fresh admitted instance/fork continuation on Y uses
a new identity and verified effect/proposal disposition; copied approvals and step
receipts alone cannot open execution or repeat previously performed effects.

While I waits, use 9.17.5 to change X's program. An incompatible operation/rule/output
change blocks without instance disposition. Compatible carriage freshly binds I to
current supported contracts. Migration revalidates every affected proposal/evidence
binding; it never renames old proof to the new revision. Y still progresses.
Start J between adoption prepare/publish: the head fence makes adoption stale and
re-preparation inventories J. Fork Y while I waits, then adopt on Y: copied I is
historical there and creates no live disposition, while X still requires one.

### Effect and resource boundaries

For Bank, lose the response after a step's performed publication and retry the
same transition. Observe one posting/rail dispatch through the actual process root,
not two logical events with one hidden retry. Exercise ProductUnpublished and
SettlementDeferred independently. Source fact publication and external settlement
are separately observable; no workflow rollback erases a performed external effect.

Deliver duplicate/reordered wakes; revoke authority while waiting; cancel before
execution, after owner effect, and after performed publication. Close the last user
interest and retire the branch while recovery remains. Dispose no required custody.
With two observers, closing one releases its own interest, not the other's run.
After performed publication, drop every handle plus compiled/status projections and
exhaust replay retention; retry as another authorized actor and require one effect plus
the correct successor. Race cancel between admission/publication: cancel-first makes
the step stale before effect, effect-first makes cancellation report that effect.
Exhaust definition graph, prepared candidate, pending notification, retained evidence and
iteration limits independently at their owner boundaries. Do not manufacture a
giant fixture that exhausts all limits together.

### Geometry scale and ordinary authoring (Phase 2 onward)

Through the real proprietary composition, author a short solver chain and the reviewed
geometry component through the common public surface. Change one local geometry region,
settle its actual output, and inspect geometry plus performed effects independently of
workflow status. Increase unrelated model population from 1k to 10k to 100k native
records while holding the edited region, active steps and dependency coverage fixed.
No geometry-to-workflow-node expansion or unrelated evidence refresh may appear.

Independently qualify authored sparse graphs at 100, 1k and 10k expanded nodes with
proportional edges, including long chains, branching/joins, repeated components and
bounded retries as those forms ship. Vary settled history at 100, 1k and 10k occurrences
and unrelated instances at 1, 10 and 100 while holding one warm advancement fixed.
Construction may use production-valid bulk setup; the measured action uses the public
entry and real owners. Denial of every large valid case is not successful qualification.

Warm advancement performs zero whole-definition reconstruction and zero historical
prefix replay. Unrelated population growth adds zero unrelated records visited or
provider contacts; bounded index lookup depth and declared storage granules are allowed.
Grow actual dependency coverage separately: its necessary reads and a broad solver's
real work are charged, never hidden as workflow overhead or claimed constant-time.
Reject exhausted work/memory budgets before oversized allocation or effects. Destroy
derived plans/progress, reconstruct with a separately charged cold budget, and recover
the same next action without repeating a performed effect. Repeat locality checks after
the fork/adoption scenarios when Phase 4 ships.

Use existing owner counters and scale harnesses; add only missing boundary observations.
Known repeated scans can be removed directly; a preliminary benchmark project is not a
prerequisite. Acceptance records cold/warm timings, p50/p95, peak and retained bytes,
work counts, hardware/runtime configuration and workload shape. Fix reproducible lane
budgets before qualification; do not widen them merely to accept a regression. Small
counter cases belong in focused tests, full scales in scheduled/product qualification.
These sizes establish a minimum tested envelope, not a claim about every geometry
algorithm or unlimited model size. The proof must expose whole-graph-per-step scans,
history replay, per-element orchestration and cache reuse across foreign publications.

## Definition Language And Ownership

### Definition and instance ownership

Installed application programs, domain occurrences, workflow definitions and running
instances are distinct typed objects/lifetimes. Inspection connects them without a
single untyped mutable registry. Component definition/occurrence upgrades use this
vocabulary without requiring a new CAD component engine here.

ApplicationWorkflowSpec defines the installed vocabulary and contract.
AuthoredWorkflowDefinition is bounded data composed from typed operation, query,
condition, assessment, approval and transition references. Validation checks every
branch, values/units, inputs, effect ceilings, result/evidence contracts, reachability,
terminal coverage, retries and versions. Static Rust connections fail at compilation
where expressible; user-authored connections require typed runtime validation using
the same canonical rules. Unknown operations and arbitrary code bodies are invalid.

ValidatedWorkflowDefinition carries substantive validation/dependency results. Only
an owner-admitted published revision can start an instance. Publishing requires current
authoring authority; it grants no action authority. Definitions are versioned model
data under the selected branch's installed program. Editing one does not reinstall
the application or invoke 9.17.5 program adoption.

Definitions and instances are branch-local model facts under World/Relational;
A/B publication in the decisive journey occurs on the same branch. An instance binds
its branch identity/incarnation as well as definition revision. Sibling branches do
not discover each other's current definition or approve each other's instances.
Branch forks retain historical facts, not callable instance/approval authority;
continuing work there requires an admitted new instance identity and fresh evidence.
A definition imported to another branch is validated/published there under current
contracts. Program adoption inventories running instances in its explicitly selected
branch scope; sibling instances retain their own program and evidence. Removing an operation requires affected definitions to be revised or retired
and running instances to have lawful dispositions before activation; historical data
and performed recovery remain, without callable removed operations.

Definition identity has three non-interchangeable axes: workflow identity names the
lineage, canonical content identity hashes version-tagged expanded meaning, and the
published revision occurrence names one performed commit. Branch-current selection is
a separate fact. Pins and retirement bind the occurrence; compiled plans key by
content plus installed support so equal meaning may be shared without aliasing A-B-A
publication or branch currentness.

Instances bind immutable definition revision, exact subjects/inputs, proposal source
expectations, step receipts and required evidence. New instances select the current
revision. Existing ones stay pinned until explicit admitted migration/cancellation.
Pins are bounded custody, not retained permission. Historical definition revisions
may support existing instances under the selected branch's admitted program and
current host support/security. A revision that is historical on one branch may still
be current on another; neither an archive nor a copied receipt makes it executable.

Progression is caller-pumped through a live authenticated request. Signal wakes only
make readiness observable; without a request provider and World contacts remain zero.
Every node uses the advancing request's principal, never the author, starter or an
ambient service principal. A host may pump with its own authenticated principal; this
milestone installs no run-as grant, background sweeper or per-definition registration.
An unauthorized next actor yields `AwaitingActor` without flattening the node outcome.
Deadlines, expiry and authentication age use one installed named clock domain and are
checked at admission.

Approval binds proposal identity, dependency versions, workflow revision, scope,
approver, purpose and expiry. Migration revalidates affected evidence, never relabels
approval. Conditions consume admitted observations with explicit current/retained
posture. Query freshly admits steps and World publishes effects.

Transition occurrence identity is derived from instance/incarnation, definition
occurrence, expanded node path and the settlement-derived back-edge iteration vector;
it is never allocated from process memory. Its owner-derived step key is
principal-independent. Caller keys scope declared action intent inside that occurrence.
Definitions bound graph size, work/retention, deadlines, notifications and retries.


### One canonical authoring model

The primitive typed builder creates qualified node occurrences, typed data/control
connections, joins, terminals and bounded back edges in
`AuthoredWorkflowDefinition`. `worth_query_workflow!` is concise Rust syntax over that
builder and returns the same unvalidated phase. UI/API/AI commands use installed
vocabulary and typed port descriptors to produce the same data; none has private
validation or execution rules.

`AuthoredWorkflowComponent` is a finite fragment with typed public ports. It expands
deterministically before validation; expanded identities include the complete
component occurrence path and retain authored-clause provenance. It owns no runtime
instance, scheduler, lease or independently current revision. Canonical identity uses
the expanded semantic graph, ignoring syntax, UI layout, labels and irrelevant order.
Equivalent builder, macro and command input must produce identical canonical material.

Components express process reuse such as `RequiredGeometryReview`. CAD model graphs
remain separate: walls, NURBS points, faces, fillets and assemblies are CAD values and
feature occurrences, not workflow nodes. Static port misuse fails compilation where
knowable; dynamic data receives equivalent validation. Live branch, support, capacity,
source and authority facts remain runtime admissions producing stronger phases.


### Exact definition shape

ApplicationWorkflowSpec is the installed vocabulary contract, containing allowed
operation/query/assessment/approval/provider-input references, connection/result
bindings, authoring capability, maximum effect ceiling, policy and resource profiles.
It does not embed a user's current definition or instance state.

AuthoredWorkflowDefinition contains:

| Field family | Meaning |
| --- | --- |
| Identity and parent revision | Explicit workflow identity and observed predecessor; no last-writer-wins revision overwrite |
| Subjects and typed inputs | Domain/native identity bindings, units, required/optional posture and admitted scope |
| Component occurrences | Reusable fragment identity, qualified occurrence path, typed public bindings and deterministic expansion provenance |
| Nodes | Qualified node identity, installed vocabulary reference, typed arguments and declared result binding |
| Data connections | Exact output-to-input contract, occurrence mapping and conditional availability |
| Control connections | Successor on a typed result/condition; complete failure/terminal routing |
| Required evidence | Applicable inventory/coverage contract, accepted assessment source/version and pass policy |
| Effect/approval policy | Explicit automatic, proposal, approval and application boundaries; no inferred authority |
| Resource and progress policy | Graph/queue/work/retention/deadline limits, retry bounds and cancellation posture |

The definition's authored and fully expanded graphs are finite and separately bounded.
From Phase 2, component occurrences, nesting depth and retained expansion provenance
have explicit bounds independent of node/edge counts and canonical bytes. Empty or
small fragments cannot evade occurrence/provenance accounting. Expansion checks
cumulative capacity before allocation and carries totals instead of rescanning all
earlier occurrences. Dynamic authored input receives the same limits as Rust input.
IDs locate nodes inside a validated definition; they are not operation capabilities.
The public Rust builder and macro use typed references; command/transport/archive
decoding returns only an untrusted draft with the same structural validator. Existing
Query expression machinery evaluates predicates.
No arbitrary Rust, string expression evaluator, dynamic service lookup or callback
receiving the application runtime can be installed through a node.

### Published truth, compiled meaning and transition authority

`PublishedWorkflowDefinition` is authoritative branch-local model truth.
`CompiledWorkflowDefinition` is a discardable projection bound to its exact definition
revision and supported program/vocabulary contracts. It fixes expanded occurrences,
typed port/data bindings, transition/result tables, evidence requirements, dependency
sets, effect contracts and structural resource ceilings. Equivalent instances share it;
destroying it and recompiling from published truth must produce identical meaning.
It carries no principal, live branch currentness, resource reservation or execution
authority.

`AdmittedWorkflowTransition` is the sole workflow-issued permit for one concrete step.
Its owner binds instance/incarnation, definition and program revision, transition
occurrence, exact inputs/source expectations/evidence, current principal authority and
reserved resources. The selected existing node owner consumes it once and returns its
real outcome. A compiled plan, status projection, node ID, wake or prior transition
receipt cannot construct or substitute for this admission.

| Phase value | Visibility and mint | Consumer and authority posture |
| --- | --- | --- |
| `ValidatedWorkflowDefinition` | public; pure canonical validator | publication preparation; grants nothing |
| `PreparedWorkflowDefinitionPublication` | public opaque; live application owner using concrete authoring authority | publication only, under exact affinity/currentness |
| `PublishedWorkflowDefinitionRef` | public opaque; performed publication or governed discovery | compilation/start selection; grants nothing |
| `CompiledWorkflowDefinition` | crate-private; compiler from published truth and support | transition selection; grants nothing |
| `AdmittedWorkflowTransition` | crate-private; instance owner using the installed operation's concrete `worth-proof` authority plus current principal/resources | exactly one existing owner execution |

No phase has `from_identity`, a proof codec or public fields. Compile-fail evidence
pairs forged construction with valid owner-issued use.

Compilation is definition-publication/cold reconstruction work. Ordinary wakes select
precompiled transition meaning and perform fresh admission; they do not expand
components, canonicalize definitions, rediscover ports, traverse unrelated nodes or
re-decide evidence requirements. Live facts that cannot be compiled remain explicit
inputs to transition admission.

Phase 2 separates shareable semantic tables from publication-specific bindings.
Semantic reuse keys include canonical content and exact supported program/vocabulary
meaning. Concrete node/connection IDs remain bound to their runtime/application and
published occurrence; equal content never imports another publication's IDs or
branch authority. The publication owner enforces immutability of definition members
through every write surface. A stored content hash alone cannot certify member truth.
Fresh admission checks the selected occurrence, supported contracts and live branch
posture without re-reading immutable members. Retirement/adoption can deny execution
without changing or discarding shared immutable meaning.

Compiled entries have an owner, byte budget and release/eviction policy; they never
evict authoritative revision pins. A real miss enters explicitly budgeted cold
compilation and reports that work. Warm reuse must remain possible across repeated
requests and equivalent instances; recreating request-local caches does not qualify.

### Geometry execution cost contract (Phase 2 onward)

Let V/E be expanded workflow nodes/connections, P retained authored provenance, H
settled instance history, D dependencies actually checked and G geometry population.
These are separate axes. CAD features and primitives remain domain data; bulk inputs,
queries and operations do not imply one workflow step, Signal node or subscription
per element. Dynamic locality uses existing aspect/record/partition contracts.

| Lane | Required work and resource posture |
| --- | --- |
| Authoring/expansion/validation | Ordinary identity, binding, coverage and adjacency passes are O(V + E + P), allowing ordered-index/sort logarithms; no per-node full-edge scans or repeated whole-prefix expansion. Availability/dominance must use a separately stated algorithmic bound, avoid quadratic retained all-node sets, and pass the 10k sparse cases within admitted visits/bytes. |
| Publication/cold compilation | Read only the exact definition and necessary support; indexed membership checks and bounded bulk/frontier reads preserve complete inventory, including absence. Charge canonicalization and compilation here. |
| Warm selection/progression | Precompiled local dispatch and relevant edges, plus D and fresh admission; no V/E/H scans, repeated canonical serialization or topology validation. Reuse canonical identity material already established for this purpose. |
| Domain execution | Charge the actual affected geometry, query traversal, numerical solver and effect scope separately; whole-model work requires the operation's explicit admitted contract. |
| Reconstruction/migration/inspection | Separate work/memory budget and visible posture; never smuggle history replay, diagnostic expansion or migration into an ordinary warm step. |

Data structures and access strategies remain implementation choices within these
contracts. Related-entity/index layouts, bounded traversal, dense compiled tables and
native references are judged by authority, locality and total cost. Do not require a
new generic graph framework or migrate unrelated schemas merely for consistency.
Count allocations, retained plans/progress, dependency reads and scheduler/World
contacts at their owners. No unbounded queue or background rebuild hides synchronous
cost. Independent instances share immutable meaning without a whole-runtime lock.

Phases 2.1-2.6 impose native costs without House caches or raised limits: linear Query step preparation, preadmitted Relational index work,
liveness-bounded generations, shared candidate reads, consumed-dependency reuse, changed-component writes, incremental views and bounded
capture/restore. No cache decides currentness; ABA, foreign meaning, unknown coverage and copied receipts deny.

### Node semantics and complete control flow

| Node kind | Consumes | Produces and limitation |
| --- | --- | --- |
| Query/read | Installed query and current or explicit retained read contract | Disclosed typed observation; no mutation permit |
| Operation | Installed typed intent, inputs, source expectation, current authority and any declared workflow-authority port | Existing mutation terminal; performed and settlement are distinct |
| Assessment | Required subject/source contract and installed producer | Exact completed evidence, possibly failing; Pending is not success |
| Approval | Immutable proposal revision, evidence requirements and approval policy | Owner-issued unforgeable authority port for its exact covered operation; no general capability |
| Condition | Declared typed inputs and pure Query predicate | A declared branch selection, not an effect or new observation |
| Evidence join | Required inventory and compatible completed evidence | Complete/failing/incomplete/stale result under declared policy |
| Bounded back edge | Explicit retry/revision reason, attempt bound and retained state | A new transition occurrence; cannot reuse changed intent under an old key |
| Terminal | Declared complete, rejected, cancelled or recovery-pending posture | Exact result and remaining owner custody, not discarded resources |

Subject assignment, evidence collection and evidence consumption are separate facts.
Evidence may be collected before its consuming control-flow point once the declared
subject and dependencies are ready. Navigation or a new transition occurrence does
not itself revise that subject or invalidate compatible evidence; only a changed
covered dependency, contract, policy or explicit expiry does. Multi-subject proposals
track coverage per subject and dependency scope rather than treating the proposal as
one all-or-nothing evidence key.

Every executable path has declared typed successors for relevant results. Values
from one conditional arm are unavailable in another unless a typed join supplies
them. Validation rejects reads of unproduced results, conflicting node identities,
dangling edges, missing terminal handling, unbounded cycles and illegal port bindings.
An unreachable node is rejected rather than hidden from permission/effect checks.

The author may declare eligible independent work, but concurrent mutation requires
existing admitted footprint/disjointness and resource evidence. Source order or node
identity cannot decide overlapping write legality. A join's execution policy is
explicit: all required results, a declared conditional subset, or a typed failure.
No default race-to-first rule silently weakens requirements.

An operation declaring a workflow-authority port cannot run through ordinary mutation
entry without an owner-issued value; that entry returns typed
`RequiresWorkflowTransition` before handler work. Validation rejects an unbound port
or a non-approval producer. Operations without such a port remain directly callable:
placing them after approval is process order, not security.

Retry of an infrastructure delivery is not a new domain operation. A declared
revision/back-edge that changes proposal or input is a new transition occurrence
and intent; it must clear/revalidate affected evidence and obtain fresh admission.
Deadline and work/iteration budgets cannot reset indefinitely on back edges.

### Validation and publication protocol

~~~text
untrusted authored draft
  -> canonical definition structure and complete binding checks
  -> ValidatedWorkflowDefinition
  -> bounded current vocabulary/support/source publication preparation
  -> PreparedWorkflowDefinitionPublication
  -> World-performed definition revision
  -> PublishedWorkflowDefinition
  -> CompiledWorkflowDefinition (rebuildable, non-authoritative)
  -> freshly admitted instance start
  -> owner-managed eligibility / AdmittedWorkflowTransition
  -> selected node owner / actual publication
  -> typed result / evidence join / next transition / terminal custody
~~~

Pure validation supplies canonical structural/semantic facts without a request,
principal, branch or live authority. Publication preparation binds them to exact
application/installation, branch/incarnation, program/component/Bridge evidence,
vocabulary revision, predecessor definition, dependencies and reserved resources.
Publication consumes that stronger phase and rechecks affinity/currentness at the owner
boundary. A changed program or source cannot be fixed by retagging. Two competing
updates to the same predecessor cannot both become its current revision.

Publishing a definition is an ordinary governed model mutation. It reserves storage/
validation resources and runs its required actual candidate checks; its source and
idempotency semantics match 9.17.4. A validated draft is not already-published meaning.
A descriptive receipt does not construct a PublishedWorkflowDefinitionRef; the owner
projects that reference only from the actual performed publication.
An archive/import is data requiring the same validation and preparation. Compilation
consumes only the performed published definition plus exact installed support; it
cannot make an unperformed definition current or callable.

Installation provides concrete adapters for the installed vocabulary once. Dynamic
publication does not add a new operation implementation, mutate the program support
registry or select a provider by a user string. Definition-authoring capability
restricts what may be composed; operation capability is checked freshly when run.
A structurally valid draft can still fail publication or execution authority.

### Proposal, assessment and approval evidence

A proposal is immutable authored data naming the intended installed operation,
typed inputs, expected source footprint, affected subjects and definition/transition
provenance. It may include explanatory predicted effects, but never a serialized
executable candidate or preparation proof retained for later unconditional use.
Applying it prepares the actual candidate under fresh admission and source checks.

Assessment evidence binds producer/contract version, required subject, exact
authored-source dependency versions, result posture, performed publication and output
content identity. The evidence fact is authoritative; the assessment output is a
rebuildable projection and may be evicted without changing what was accepted. Its
native source footprint excludes irrelevant output publications, permitting
separate assessments to settle without requiring identical latest World heads.
Equal values are insufficient after native ABA. Required coverage is computed
from the authored inventory/applicability, not supplied by the evidence rows.

Approval binds instance identity/incarnation, definition revision, proposal identity,
declared covered subject/dependency revisions, operation meaning, required evidence and
source footprint, approver, purpose, expiry and policy. A policy that approves the
whole proposal explicitly includes its complete revision in that coverage; narrower
independent reviews do not acquire that coupling accidentally. Approve/reject are
declared operations; caller booleans, role strings or edited database status fields
cannot create an admitted approval.
The application step checks current authority/revocation and complete evidence again
at its owner handoff. A currently valid approval cannot override failed hard integrity.

Evidence collection is admitted when its declared subject is ready; it need not wait
for the workflow cursor to stand at the consuming node. Collection records assignment,
subject/dependency coverage and evidence fact separately from later consumption.
Back/navigation and creation of a later transition occurrence preserve compatible
evidence because neither fact alone revises the subject. A multi-subject proposal
records coverage per subject and dependency scope so a relevant edit invalidates the
changed subject without discarding independent reviews or preserving approval for the
changed part.

Fresh-authentication evidence binds the exact signing intent, subject coverage,
principal and declared maximum age/reuse policy. Authentication for another intent,
an expired challenge or reuse beyond policy cannot authorize the signature even when
the same principal and proposal are visible. The admission-owned authentication event,
not a caller flag or principal expiry, records purpose and issuance in the named clock.

Evidence staleness is never an eagerly stored flag. Admission compares bound native
dependency and contract versions; changes make only affected coverage stale. An
ordinary source commit performs zero workflow invalidation work. Unrelated edits
preserve coverage. Reuse is a checked equivalence result, never an ID/value match.

From Phase 2, coverage also retains negative and set-completeness dependencies:
"no conflicting member" and "all required members" must notice matching insertions,
removals and ABA even when no previously returned entity changed. Use native query,
adjacency or indexed-selection evidence at the smallest sound scope. A wake or touched
set may narrow delivery; it cannot replace admission's authoritative currentness proof.
No source commit scans waiting workflows to maintain an eager stale flag.

### Instance progression and linear effect custody

An instance binds its branch, immutable definition revision, subjects and logical
transition history. Authoritative instance records carry typed references to actual
step publications/outcomes; live managed resources remain in their existing owners.
Persisting a step name/status is not evidence that its effect happened.

Before invoking any executable node, the owner derives `AdmittedWorkflowTransition`
from compiled meaning plus current instance, authority, sources, evidence and resource
admission. The managed executor accepts that phase rather than a definition/node pair.
A performed step's receipt and successor eligibility are recorded through the existing
execution/result-publication contract. If the effect
performs before the instance projection advances, the operation's terminal/recovery
custody remains authoritative; resumption resolves that exact outcome rather than
executing another effect. No owner-unpublished step is recorded as performed merely
because its private candidate or outbox exists.

For a local mutation, transition settlement and the domain mutation are staged in one
Relational candidate with the exact instance head in its source expectation. Effectful
transitions serialize on that head. Cancellation after admission makes the candidate
stale before effect; cancellation after publication reports the performed effect.
Successor selection reads settlement, not replay retention or an in-memory handle.

Step identity binds application/branch/instance incarnation, definition occurrence,
node, logical transition occurrence and intent equivalence. Transport retry, duplicate
wake and another authorized principal reuse the result. A loop creates a new occurrence
only after settlement or declared recovery; it cannot invent a fresh attempt ID.

Instance observable states distinguish Ready, Running, WaitingForEvidence,
WaitingForApproval, Completed, Rejected, Cancelled and RecoveryPending, refined by
existing owner outcomes. These describe application meaning, not a second scheduler.
Only owner-issued transition results advance them. Current status/node is a rebuildable
projection: destroying it must leave exact legal next actions reconstructible from the
published definition, compiled meaning, transition occurrences and owner results.
Cancellation is not rollback: performed effects remain in the result, and recovery can
outlive the cancelled run.

From Phase 2, the instance owner maintains a discardable progress projection carrying
the settled head, occurrence/iteration counters and relevant join progress. Update it
from exact owner results with continuity checks; gaps, duplicates or a foreign basis
cannot advance it. Warm selection neither replays H records nor recounts each retry
from a history prefix. Admission still fences the authoritative head, so a stale
projection cannot authorize a step. Loss enters bounded reconstruction from retained
authoritative settlement/effect facts, independently of disposable replay receipts.
Retention must preserve sufficient truth for that reconstruction; cancellation and
recovery custody cannot be evicted with the projection. Phase 4 adds migration/fork
rebinding through the same owner rather than trusting a copied projection.

The typed publication return path from 9.17.4 drives dependent eligibility and evidence.
The workflow author does not forward receipts, choose dependency ordinals or parse
event strings. Pure evaluation creates no authoritative publication. Effectful steps
return through the same admitted candidate/invariant/World path as ordinary actions.

### Branches, revision retirement and migration

Definitions A/B coexist as immutable facts; one selected current definition per
workflow identity governs new starts on that branch. Retirement prevents new starts
without deleting history or live instance custody. Existing instances remain pinned
within bounded retention until completed, cancelled or explicitly migrated.

A branch fork preserves historical definitions/instance records under exact component
meaning. It does not clone executable approval, live leases or dispatch permission.
Continuing work there requires explicit admitted new-instance/fork disposition with
new branch identity, current authority and verified prior-effect references. The
default is historical inspection, not automatic execution. If a prior effect cannot
be reconciled safely, deny continuation with its typed requirement rather than replay
it under a fresh key.

Definition migration prepares an exact source instance, source/target definition,
node/result mapping, subject/source compatibility and effect/evidence dispositions.
Required action: preserve verified compatible completed effects as history, invalidate
changed proposal/approval evidence, re-evaluate affected joins, and establish a lawful
next transition. No mapper can turn an unperformed step into completed or forget an
external effect. Publish through the owner under exact instance currentness.

Program adoption uses 9.17.5. The instance owner supplies bounded inventory and
typed dispositions to its existing adoption preparation. Compatible carriage obtains
fresh execution evidence; incompatible definitions retire/migrate and uneffected
instances cancel. Performed obligations remain occurrence-bound recovery.
Do not re-open ordinary operations from a historical program to finish cleanup.
Unaffected sibling programs/instances remain usable.

This milestone additively extends adoption preparation with a live-participant port.
`WorkflowVocabulary` joins the semantic families; published definitions retain their
dependency facts so inventory never reads compiled plans. The port inventories exact
live definition/instance occurrences within `maximum_selection_work`; exhaustion
denies without truncation. The owner derives legal dispositions, while callers choose
only among legal alternatives. Cancel/retire/carry facts stage in the adoption
candidate; mapped instance migration publishes earlier. A definition replacement
depending on target meaning publishes after activation, so the intentional gap denies
new starts. Fork-copied instances from another incarnation are historical, not live.

The disposition law, as delivered. A definition covered by the target vocabulary
with unchanged node dependencies may carry or retire; otherwise it only retires.
A compatible instance that has performed nothing may carry or cancel; a compatible
performed instance only carries; an incompatible unperformed instance only cancels.
An incompatible performed instance, or any instance whose latest approval has not
been followed by its receipted guarded operation, has no adoption disposition: it
is settled or recovered under the source, or migrated, and then re-inventoried.
Carry rebinds the exact instance to the target in place. Its pinned definition
keeps the revision it was published under and records the carriage beside it;
execution reads the latest carriage, never a rewritten publication revision.
Retire drops only the current-definition relation; cancel records the cancelled
state and drops live membership, so every later request is refused as cancelled. Choices are
bound to the inventory digest, and preparation refuses them with the fresh inventory
when owner truth moved. Assessment evidence records the program revision it was
collected under and is never reused under another, so a carried instance collects
fresh evidence before any approval. A fork's copied current definition governs its
new starts and is decided on that fork like any other.

The migration law, as delivered. Migration never rewrites an instance: it ends the
exact source as migrated and starts a successor on the current target definition of
the same lineage, resumed at a named target node. The node mapping is identity by
path. The successor carries only receipted effects, linked as prior-effect history;
proposals, evidence, joins and approvals are re-established by running the nodes that
produce them. Admission refuses as unmapped another workflow's definition, a resume
node missing from the target, a resume that could run a node before a proposal or
evidence it consumes where a fresh instance could not (a retry loop back to the
producer never counts), a completed source, and any performed effect without a
same-path, same-kind target operation, or whose operation meaning the successor could
reach again under any path. An approval not yet followed by its receipted operation is
refused as unsettled until it settles under the source. The request is start-shaped:
it runs under the workflow's start capability, which therefore also authorizes ending
the source and discarding its unconsumed proposal, evidence and decisions. Like a
start, a target that is no longer current comes back stale. It replays exactly, and
its identity binds the source occurrence; any other request for a migrated source,
including another migration, is refused as migrated. Only a new successor may link to
an existing instance or transition, so native writers can neither forge nor erase that
history. Adoption inventories a successor that inherits effects as performed, offering
only carry, and never inventories the migrated source; inherited effects never settle
the successor's own approvals.

The fork continuation law, as delivered. A fork copies an instance's history, never
its execution: the copy keeps the occurrence it was started on and stays historical on
the fork. Work continues there only as a fork continuation, the migration law applied
to the fork's copy and issued on the fork. It ends only that copy, and the instance on
its own branch is untouched and still needs its own decision there. The successor
then runs on the fork like any instance: its proposal, evidence, approval and effect
are the fork's own, and none reaches the source's branch. The target may be the
source's own definition, or any definition current on the fork, including one the
fork copied; like a migration, a target not current on the fork comes back
stale, and a definition published on another branch after the fork is not the fork's
to continue under. A continuation on the instance's own branch is refused as an
affinity mismatch, since migration is the only way to move an instance forward there,
and so is one naming a third branch, such as a sibling fork's instance or definition.
Copied receipts carry as prior-effect history, so the fork never repeats an effect the
source performed, and a copied approval still awaiting its operation is refused as
unsettled until the operation settles under the source on its own branch. The
continuation's identity binds the fork occurrence, so it replays exactly and a second
continuation of the same copy is refused as migrated. Program adoption reads the
selected branch's own root, so a fork's inventory lists its successors and never the
copies it holds or its parent's later writes. A copied definition is named on the
fork with `held_on`, which grants nothing: the fork's own truth decides its new starts,
its successor and its retirement exactly as for a definition the fork published, and
the branch it was copied from keeps its own. A definition the fork never held, such as
a sibling's successor, cannot compile or retire there, and a successor over it is stale.

Adoption races and identity, as delivered. Preparation binds the exact workflow
inventory, so an instance started between preparation and publication makes the
publication stale; re-preparing with the old choices is refused with the fresh
inventory, which lists the new instance for its own disposition. A fork taken while an
instance waits adopts without deciding that instance, since the fork holds only its
historical copy; the instance keeps its disposition on its own branch and still
progresses there. Equal-content publications are distinct definitions, and their
instances stay distinct through a carry: each progresses and completes on its own. A
progression prepared under the source program cannot commit after adoption, and the
carried instance progresses only through the target program's binding.

## Required Public Experience

These target Rust examples compile in the existing host/decl audience tests when
their phase ships. Definitions and operations use typed installed references.

### Common path from Phase 2

Ship an ordinary surface over the same builder and publication/execution owners. A
kernel developer declares capabilities, connects typed values and starts a workflow
without naming preparation phases, source-expectation products, occurrence IDs,
assessment demand settlement or transition admission. The following is the target
interaction; public examples must adopt the final consistent Rust spelling at delivery.

~~~rust,ignore
let request = application.request(&principal, &scope).on_branch(branch);
let draft = workflow::<HouseDesign>(workflow_identity, |w| {
    let structure = w.operation::<StructuralSolver>("structure")?;
    let electrical = w.operation::<ElectricalSolver>("electrical")?;
    w.connect(structure.output(), electrical.input())?;
    w.sequence((structure, electrical))
})?;

let publication = request.workflow(draft).publish()
    .idempotency(definition_key).execute();
// Match the typed performed outcome to obtain `published`.
let started = published.start(&request, subjects)
    .idempotency(instance_key).execute();
// Match the typed start outcome to obtain `instance`.
let progress = instance.run(&request).execute();
~~~

Authoring remains pure. `sequence` declares completed-order flow and an explicit
inspectable fail-closed policy for unconnected outcomes; pending work and partial or
indeterminate effects retain typed wait/recovery custody. It never guesses data or
authority connections. Typed bindings may be inferred only when uniquely determined
by declared contracts; ambiguity requires a named port. Reusable components hide
repeated review wiring while retaining public ports, provenance and canonical identity.
Publication convenience runs validation/preparation internally and yields a callable
handle only from the performed outcome. Handles carry selection, not retained permission.
Principal, branch, subjects, intended approval/effects and retry/idempotency semantics
remain explicit or are carried by the existing typed request. Installed finite profiles
may supply ordinary resource defaults; callers can narrow them or explicitly select
another admitted profile. No unbounded default or generated key changes retry meaning.

`run` pumps through the existing managed owners with the supplied live request until
completion, a wait, cancellation or its admitted budget. It returns typed progress/
custody and preserves nested denials and partial/indeterminate effects. It does not
wait indefinitely for a human, invent a scheduler or require callers to poll `settle`
a fixed number of times. Advanced publication, budgets, retained work and recovery
remain accessible through the same owner path. Product adapters expose semantic
start/assess/approve/apply actions without Debug-string error erasure or receipt routing.

Phase 2 compiles both a short solver chain and a reusable reviewed-geometry example
through this surface and uses it in the real proprietary adapter. Phase 3 adds the
rejected/revised branch and Bank adapter. Compare canonical meaning, typed outcomes,
authority denial and effects with the advanced path; a second validator cannot pass.

### Explicit graph and advanced lifecycle path

~~~rust,ignore
let request = application.request(&principal, &scope).on_branch(branch);

let draft = worth_query_workflow! {
    spec: ReviewedGeometryChange;
    identity: workflow_identity;
    limits: definition_limits;

    nodes {
        propose: operation ProposeChange;
        checks: component RequiredGeometryReview;
        review: approve GeometryReviewer;
        revise: operation ReviseProposal;
        apply: operation ApplyApprovedChange;
        completed: terminal Applied;
        rejected: terminal Rejected;
    }

    connect {
        propose.proposal -> checks.proposal;
        propose.proposal -> review.subject;
        checks.evidence -> review.evidence;
        propose.proposal -> apply.change;
        review.approval -> apply.authority;
    }

    flow {
        start -> propose -> checks -> review;
        review.approved -> apply -> completed;
        review.rejected -> revise;
        revise.completed -> propose retry 2;
        revise.exhausted -> rejected;
    }
}?;

let validated = draft.validate()?;
let prepared = request.prepare_workflow_publication(validated)?;
let outcome = request.publish_workflow(prepared)
    .idempotency(definition_key).execute();
// Match the performed outcome to obtain its owner-published definition reference.
let started = request.start_workflow(&published_definition, subjects)
    .idempotency(instance_key).controls(instance_limits).execute();
let progress = request.advance_workflow(&started.instance())
    .step_budget(step_budget).execute();
match progress {
    WorthQueryWorkflowProgress::AwaitingActor(next) => show_required_actor(next),
    WorthQueryWorkflowProgress::Settled(outcome) => render_node_outcome(outcome),
    other => render_progress(other),
}
~~~

The primitive builder can construct this exact graph without the macro, and a bounded
command adapter can construct it without Rust syntax. Public equivalence tests compare
their canonical expanded definition rather than token streams or display layout. Typed
condition/join/back-edge forms expose result availability and bounded recurrence.
Public examples include component reuse, a rejected branch and a proposal revision,
not only the linear successful form. The shipped signatures follow the named authority
transitions; authoring accepts no request or authority, and private spelling may not
remove validation or publication preparation.

Later sessions obtain definition references through governed branch discovery; a ref
grants no authority, and starting from a superseded occurrence returns a typed stale
denial naming the current occurrence. The model exposes approve, reject, revise, cancel, inspect,
retire-definition and migrate-instance actions. Each takes the appropriate typed
identity/source expectation and current authority. It does not expose a
set_workflow_status operation, arbitrary node executor or deserialize-proof route.
The declaration vocabulary reserves no executable inbound node. 9.17.7 adds
`await_inbound` only after its external-effect owner exists; until then validation
returns an unsupported-vocabulary denial rather than a provisional resume path.

Each entry has typed performed, denied, no-effect, unpublished/recovery, partial,
cancelled and indeterminate postures as applicable; nested node outcomes remain
unflattened. Errors name workflow/definition/instance/node and the violated binding,
source, authority, budget or transition contract. Disclosure may redact detail,
never erase the fact that execution or recovery is incomplete. Observation resource
close releases interest; it does not itself cancel the instance or its mandatory work.

## Destination Topology

Paths are under workspaces/worth-query/crates unless qualified. E existing; R extend existing owner; N new semantic responsibility.
No new scheduler, generalized workflow runtime, queue registry or expression compiler.

~~~text
worth-query-declaration/src/application_program/workflow/
  vocabulary/{definition,operations,assessments,approvals,authority_ports}.rs N semantic references
  authoring/{builder,component,expansion,macro_surface}.rs      N one graph-shaped authoring model
  definition/{authored,canonical,inputs,identity}.rs          N immutable model meaning
  connection/{data,control,join,retry}.rs                     N distinct edge semantics
  validation/{binding,availability,termination,resources}.rs  N structural checks
worth-query-installation/src/application_program/workflow/
  {vocabulary,adapters,definition_contract}.rs                N/R concrete installed bindings
worth-query-declaration/src/application_query/{dependency_equivalence,derived_view}.rs N Phases 2.4-2.5 public contracts
worth-query-admission/src/authentication_event/
  {intent,issuance,clock}.rs                                  N purpose/age/reuse proof in named clock
worth-query-execution/src/domain_computation/primary_graph/workflow/
  schema/{relations,version}.rs                              N Query-contributed fact schema
  definition/{preparation,publication,revision,retirement}.rs N branch-local definition facts
  definition/compilation/{lowering,plan,reconstruction}.rs   N rebuildable execution meaning
  definition/compilation/{reuse,publication_binding}.rs     N Phase 2 semantic reuse and exact bindings
  instance/{start,observation,migration,retirement}.rs        N model instance meaning
  instance/transition/{selection,admission,settlement}.rs     N one-step authority and result
  instance/status_projection.rs                              N discardable current-status view
  instance/progression/{projection,continuity,reconstruction}.rs N Phase 2 incremental progress
  proposal/{source,revision,application}.rs                   N operation proposal contract
  evidence/{inventory,assessment,approval,currentness}.rs     N owner-backed evidence meaning
  adoption/{inventory,dispositions}.rs                       N participant in 9.17.5
  recovery/{continuation,disposition}.rs                      N adaptation to actual custody
worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_binding/effect_accumulator/expected_steps.rs N 2.1 ordered proof
worth-query-execution/src/domain_computation/primary_graph/{index_maintenance_budget.rs,provider/application_attempt_state/commit_preparation/relational_commit/commit_execution.rs} E 2.1 pre-effect index gate
crates/worth-relational/src/indexes/authority/maintenance/{entry_edits,work}.rs E/R bounded charge
crates/worth-relational/src/runtime/state/subsystems/indexing/generation_catalog/{scope,retention}.rs E/N 2.2 liveness
crates/worth-relational/src/durability/{derived_index_artifacts,log/persisted_checkpoint}.rs E/R shared checkpoint encoding
crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore.rs E/R Phase 2.6 restoration
crates/worth-relational/src/validation/engine/input_preparation/{plan,materialization}.rs N Phase 2.3 shared candidate reads
worth-query-execution/src/domain_computation/primary_graph/output_reuse/{dependency,selection}.rs N Phase 2.4 output equivalence
worth-query-execution/src/domain_computation/primary_graph/application_query/derived_view/{dependency,refresh}.rs N Phase 2.5 managed views
worth-query-execution/src/domain_computation/
  managed_run/workflow_*                                     E/R read-node provider-run owner only
  provider_session/readmission/workflow.rs                   E/R fresh continuation admission
  artifact_owner/{workflow_authority,frozen_workflow_authority}.rs E/R canonical carriers
  convergence_epoch/workflow_cleanup.rs                      E/R actual resource cleanup
worth-query-execution/src/domain_computation/primary_graph/
  application_attempt/product_operation/{operation,transaction}.rs E/R mutation/atomic settlement
  conditional_operation/ application_output_demand/           E/R eligibility and exact closure
  product_operation/program_adoption/                        E/R live-participant port and branch adoption
  application_discovery/                                    E/R governed definitions/instances
worth-query-publication/src/application_entry/
  workflow/{definition,instance,actions,progress}.rs           N public builders/results
  workflow/ordinary/{publication,run}.rs                     N Phase 2 common-path composition
worth-query-package-archive/src/workflow_definition/
  {codec,compatibility}.rs                                   N draft codec only, never live facts
worth-query-decl/src/facade.rs worth-query-host/src/facade.rs E/R reexports
worth-query-certification/tests/application_graph/workflow/
  {authoring,definition,evidence,revision,branching,recovery,resources}.rs N grouped public proof
  {scaling,ordinary_api}.rs                                 N Phase 2 cost and caller contracts
worth-proprietary/crates/worth-cad-entry/src/application/workflow/
  {vocabulary,reviewed_geometry}.rs                            N proprietary declarations
worth-proprietary/crates/worthy-house-application/src/workflow/
  {composition,actions,program_adoption}.rs                    N real product integration
worth-proprietary/crates/worthy-house-declarations/src/application/framing/revision/{handler,candidate}.rs E/R 2.4 delta authoring
worth-proprietary/crates/worthy-house-application/src/session/model_thread/scene_projection/{dependency,refresh}.rs N 2.5 scene
worth-proprietary/crates/worthy-house-certification/tests/journeys/workflow.rs
                                                               N cross-repository court
worth-proprietary/docs/house/query-platform.md                 R consumer contract and operation
~~~

Authoring/validation is pure meaning; installation binds implementations; primary graph stores definition/instance/proposal facts through admitted
operations. Existing operation, output-demand, managed-read and change-delivery owners execute nodes; proof/World/Signal retain authority.
Model-step records reference performed facts, not another ledger; no persisted live lease or arbitrary status table permits execution.

`authoring` owns pure construction/expansion, `definition` canonical meaning, and `validation` structural proof; the macro only fronts the builder.
Execution `definition/compilation` derives discardable plans; `instance/transition/admission` binds current authority/resources for one step.
Neither absorbs the other's lifecycle or stores current definitions. Phase 2 `compilation/reuse` manages immutable semantics, `publication_binding`
exact concrete references, and `instance/progression` derived progress/cold rebuild. `workflow/ordinary` composes owners; facades reexport only.
No cached constructor grants authority; existing parents gain these roles without a second model graph, traversal service or currentness registry.
Operation/approval nodes lower to mutation, assessment to output demand, read to
managed run and notification to change delivery. “Transition” never aliases a managed
provider step. Capability elevation/mandatory-review workflows remain separate.

Facts use versioned Query-contributed native relations; current status is derived.
Archives carry definition drafts only: instances, approvals and live authority never
import. `worth_query_workflow!` is `macro_rules!` in declaration, not a hidden
proc-macro crate. The 9.17.4 successor sketch is superseded by this populated tree.
`primary_graph/workflow` alone mutates facts; `application_discovery` exposes governed
read selection and mints non-authoritative refs, never currentness or start authority.

The `worth-proprietary` CAD application composition binds the real reviewed geometry
workflow; Bank creates application_definition/workflows for approved business-payment
and estate nodes.
Their domain handlers and numerical algorithms remain unchanged in ownership.
Real Bank HTTP/user-node/process adapters expose the new declared authoring/instance
actions with current authentication. No product-specific runner duplicates Query.

`worth-proprietary` is the standing reference consumer from Phase 1 onward, not a
terminal smoke test. Every phase closes its corresponding proprietary workflow slice
through the pinned public facades before the next phase relies on it. Query-local
owner/certification tests remain the faster diagnostic proof and the portable public
contract; neither evidence lane substitutes for the other.

All files remain within 400 lines absent a separate exemption. Reuse the existing
public graph integration target and managed-run owner tests. Do not create a crate
or compiler session for every negative input.

## Ordered Phases

### Phase 1: Authored definition to approved real effect

Use 9.17.4 operation/rule/output contracts and 9.17.5 selected program binding.
Implement the canonical finite authored definition, primitive typed graph builder,
`worth_query_workflow!`, deterministic component expansion, validation, branch-local
publication, instance start, proposal, two assessment requirements, approval and
application. Derive one rebuildable compiled definition and require an
`AdmittedWorkflowTransition` for every executed node. Phase 1 supports operation,
assessment, approval, evidence-join and terminal nodes, lowered to their existing
owners and actual World effects. Prove builder/macro/command-adapter equivalence.

Ship real authoring and instance command surfaces immediately. The first proof includes
assessment rejection then retry, wrong-source approval, duplicate wake, denied apply,
successful exact output and cleanup. No graph builder-only checkpoint.
Collect evidence before its consuming node. Destroy compiled/status projections and
all handles, rebuild identical next actions from facts, then retry as another principal
after replay retention is exhausted: one effect and correct successor remain. Race
cancel between admission/publication in both orders. Compile-fail every shipped phase
substitution with valid counterparts and pin canonical identity through closure.
In the same phase, `worth-proprietary` authors, publishes, starts and completes the
minimal reviewed-geometry definition through `worthy-house-application`, including
real assessment rejection/retry, approval authority and geometry publication.
The next phase trusts runtime-authored meaning producing actual admitted effects.

### Phase 2: Geometry-scale foundation and ordinary authoring

Consume Phase 1's published definitions, transition admission and real geometry effect. Preserve its evidence and carry unfinished work forward.
Correct inherited validation/expansion scans, ordinary full compilation and history replay. Establish the cost contract, independent component/
provenance bounds, exact compiled reuse, incremental progress and complete positive/negative dependencies. Select bounded native access/index/
managed-resource contracts where they fit; new private mechanics meet the same locality and authority guarantees.

Ship the common authoring/publication/start/run surface and compiled solver-chain/review-component examples. Route the proprietary adapter through
it with typed denials, explicit authority, resource/cancellation custody and managed progression; keep advanced control, not consumer settlement.

Close 10k authored nodes, fixed-local-edit slopes, history-independent warm selection, foreign binding denial, bounded exhaustion and destroyed-
projection reconstruction; keep source/authority revocation and duplicate-effect parity. Later phases extend this scale/API proof as semantics
arrive, never defer regressions to closure. Phase 2.1 trusts bounded execution and an ordinary API without exported internals.

### Phase 2.1: Linear effects and admissible index publication

Query carries one immutable ordered, exactly deduplicated effect-step summary from registration through overlay, invariants and commit: no growing
linear deduplication or rebuild. Relational budgets exact index deltas before World publication; post-commit finalization does no unadmitted
ordinary work. Failure retains a typed recoverable performed obligation, not “producer unavailable.” Prove the first real 802-solid scene under
ordinary limits, pre-effect exhaustion, retry and generation parity.

### Phase 2.2: Index lifetime and bounded checkpoints

Retain index generations reachable from branch/history roots or active readers; reclaim the rest. Shared/delta checkpoint payloads retain exact
version/digest readmission without repeating full maps. Prove pinned old readers and siblings select exact generations; under repeated House edits,
capture fits the existing 1 GiB cap. Report index, envelopes, partitions, outputs, peak and retained bytes separately. Corrupt or absent artifacts
deny or rebuild within a finite cold budget, never answer stale.

### Phase 2.3: Shared candidate inputs, independent verdicts

Derive overlapping topology/field preparation from installed rule declarations and materialize once per exact candidate/observation basis. Each
rule retains independent validation and denial; developers manage no sibling caches or adjacency maps. Changed topology, absence and foreign basis
invalidate sharing. Count reads, gathers, adjacency and rule cost; real CAD rules plus an overlapping rule expose duplicate work without suppressing
an independent failure.

### Phase 2.4: Exact output reuse and House delta cutover

Installed query/subfield bounds plus tracked positive, absent, negative and complete-set reads define consumed dependencies. Query compares native
versions and declared equivalence to retain unchanged outputs automatically; changed/untracked inputs get fresh admission. ABA, foreign meaning,
incomplete coverage and copied receipts deny. House revisions emit/write only affected components, including multi-joist commands; untouched bands
have zero producer/invariant/index contacts. Prove changed-band correctness and 1k/10k/100k unrelated-population locality through real facades.

### Phase 2.5: Incremental managed scene views

The House scene uses a Query-managed derived view over exact definition, occurrence and topology dependencies. Publication refreshes affected
entries; unchanged geometry requires zero body reads before reuse. Framework-owned scene/tessellation/subscription state has finite retention,
disposal and cold reconstruction, never currentness authority. Prove services, edits, discarded views and foreign bases through real House; warm scene
cost follows affected entries rather than all 802 solids.

### Phase 2.6: Restore and production journey closure

Complete compatible House capture/reopen, not generic Query Save/Open. Remove avoidable decoded clones and duplicate partition/root/scene rebuild without weakening verification.
The real 802-solid build -> services -> edit -> capture -> reopen court requires output/evidence parity, zero replayed effects and no stale view within existing byte/resource bounds.
Without a certified scene snapshot, cold restore performs exactly one occurrence/BodySet pair per entry, one discovery per frame band, one index entry per frame member and one lookup per penetration; warm unchanged entries require zero body reads.
Native restore counts root, mirror, history and index work without duplicate partition rebuild. These workload-derived work bounds, not wall-clock time, gate performance on any machine.
Report release p50/p95 as diagnostics with reads, work, bytes, contacts, peak/retained memory and pinned hardware by source/band/rule/index/World/output/scene/checkpoint/restore. Timeout is a hang guard, not a latency gate. Vary size, history and readers independently; prove typed exhaustion before Phase 3 trusts it.

### Phase 3: Control flow, coverage and effect custody

Add typed conditions, joins and bounded revision/retry with complete outcome handling.
Close the separate-assessment/ABA/new-required-subject sequence. Bind Bank's approved
payment definition to its existing operations and performed-publication-gated outbound
effect contracts without introducing the new inbox. Exercise owner-unpublished,
dispatch-pending and indeterminate existing postures through their current custody.

Advance and navigate Back without revising the evidence subject, then consume the
still-compatible early evidence. In one multi-subject proposal, change a reviewed
subject and preserve an independent review for an unrelated subject while invalidating
only the affected coverage. Prove a new transition occurrence alone does not invalidate
evidence and stale or differently purposed authentication cannot sign. Bind purpose,
issue time, age and reuse through the installed authentication-event owner and clock.

Complete authority revocation at waits, effect ceiling denial and resource admission.
Remove any new client-authored workflow loops or manual receipt routing. Existing
static Bank semantics remain protected. The next phase trusts lawful branching
and honest partial effects, not just a successful linear workflow. Inbound callbacks
remain unsupported until 9.17.7 rather than entering through `resume` or status.
The proprietary slice exercises condition/join/Back behavior, early geometry evidence,
selective multi-subject invalidation and cancellation around a real geometry effect;
Bank remains the distinct outbound-custody court.
Extend the Phase 2 common/advanced parity and scale cases to conditions, joins, Back,
retries and multi-subject evidence. Matching insertions must stale negative coverage;
unrelated edits must preserve independent assessments. Retry counts and join updates
use incremental progress. Migrate Bank's new workflow adapter off fixed polling and
stringified denials while preserving its existing outbound-custody owner.

### Phase 4: Definition revisions, branches and program adoption

Ship definition retirement, A/B coexistence, explicit instance migration and fork
continuation admission. Implement the definition/instance participant in 9.17.5
adoption with exact state/evidence dispositions. Test compatible/incompatible
program changes while approval, delivery and performed-effect recovery are pending.
Start an instance between adoption prepare/publish and require stale re-preparation;
fork while an instance waits and prove only its exact live incarnation is inventoried.

Run that adoption court through `worth-proprietary`: P0 reviewed geometry waits,
P1 adoption consumes the workflow owner's exact live inventory/disposition, the
lawful instance outcome observes P1 meaning, and a sibling continues under P0.
The public certification fixture proves platform portability but cannot replace this
consumer gate or duplicate proprietary geometry semantics inside WORTH.

No global instance revision bump, copied approval or duplicate external execution.
An unaffected sibling progresses. The next phase trusts definition and program
evolution without losing meaning, authorization or custody.
Prove semantic sharing does not alias concrete IDs across equal-content publications,
runtime instances or forks; a stale compiled binding/progress projection cannot survive
adoption as authority. Repeat the fixed-local-work scale case on an unaffected sibling.

Status: the sibling courts found version-addressed reads that answered from whichever
branch published last. A branch's program is now read on its own root, and authorization
path evaluation reads its basis's own adjacency, so a sibling's approval dependencies no
longer change when its parent adopts. Invariant and aggregate projection reads are
branch-rooted the same way, so Phase 4 is closed. Ordinary House edits and
performed-output actions on an adopted branch still run through the installed program;
selected-program execution for those lanes moves to Phase 5.
Bank approval now runs only through the workflow. A payment initiated at runtime grants
its approval workflow to each approver on the source account inside the initiation
candidate itself, up to eight approvers. Before this fix, only bootstrapped payments
could be approved.

### Phase 5: Workflow-kernel lifecycle and public closure

Complete bounded observations, pending notifications, retention, deadline/iteration accounting,
cancellation, close and retirement across every new state. Finish public codecs,
discovery, authoring/control-flow examples and real product adapters. Ordinary
product edits and performed-output actions on an adopted branch execute under that
branch's selected program, and a target that is no longer current is a typed denial.
The commit receipt Bank hands its callers is ordinary only once slice 5.6 lands:
until the Bank adapter replaces it, the public `application_program()` accessor
exposes the program runtime, which can redeem a receipt for recovery authority.

The proprietary slice closes observation, cancellation, retention, resource
exhaustion, cold reconstruction, command/action exposure and operator diagnostics
through the same product composition used in Phases 1-4, not a new closure fixture.

Runtime cases supply issued foreign/stale evidence. Deny publication/start before any
pinned-revision capacity effect; never evict pins. Count cold compile misses and require
zero on warm progression. Delete only 9.17.6 scaffolding, manual receipt routing and
weaker experimental entries; existing static product workflows remain until explicitly
migrated. Enforce that managed_run and conditional_operation never import
primary_graph/workflow. The complete local/outbound kernel closes with no provisional
inbound API; 9.17.7 enters through the external-effect owner.

Closure requires a reproducible `worth-proprietary` dependency pin to the candidate
public revision and green focused `worthy-house-application` plus
`worthy-house-certification` workflow/adoption evidence. A green Query-local CAD-shaped
fixture alone cannot close the milestone.

Status: ordinary product edits and performed-output actions on an adopted branch now
execute under that branch's selected program. `execute_performed_in_selected_program`
and `execute_performed_discovered_in_selected_program` join
`execute_in_selected_program`. Each selects the request's exact branch and presents
that program at the source commit, so the source and its outputs settle under the
successor. An installed-program lane on an adopted branch is refused before any
effect with the typed `ProgramNotActiveOnOccurrence` commit denial, and so is a
revision retired from the host after selection. An adoption that lands between
selection and commit is refused by the same commit-time check; that interleaving is
argued from the check's position, not driven by a test. A retried key replays its
recorded outcome before any commit, and no refusal claims its key. Every House edit
lane uses the selected lanes. The House journey `adopted_branch_edits` proves plain,
required-output and discovered-output edits under P1, the typed refusal of the P0
lanes on that branch, and a sibling still editing under P0. The consumer journey
proves a foreign runtime and an undeclared root are refused before publication
without claiming their keys. The selected owner's own `owns_output_root` and
`owns_output_source` refusal is argued, not tested: reaching it needs a branch whose
adopted program drops a root the host still declares, and adopting such a successor
needs a migration, which may not remove a produced output.

Status: `boundary-check` enforces the import law as a configured source-owner
isolation in `road1.toml`. `managed_run` and `conditional_operation` may not name a
path through `primary_graph::workflow`, whether written as a `crate::` path or
resolved from `self::`, `super::` or an inline module. They also may not name any
type, visible function, constant, static or exported macro that the workflow kernel
or a workflow application lane (`application_attempt/workflow_*`) declares, nor call
a visible method either adds to a type declared elsewhere, so a re-export through a
parent facade is caught too. A value restricted to a module binds only guarded code
inside that module. Test-only items, and every file of a `#[cfg(test)]` module,
bind nothing; a file's name alone never makes it a test. Bare names are not traced,
so a crate-relative glob import may not reach outside the guarded roots, where a
facade could re-export kernel values. A path or glob written through a name a
`use` binds, renamed or not, resolves through that binding, so an alias reaches only
what its target reaches.

Status: an explicit cancellation ends a live instance where it stands, through
`prepare_workflow_instance_cancellation` on the workflow entry, authorized by the
start capability on the instance's own branch. It is not rollback. The instance
records the cancelled state and this cancellation's identity and drops its live
membership, and the outcome names every node whose effect it, or a source it was
migrated from, performed; each remains performed. The same key replays its recorded
outcome, including after the branch adopts a new program, and any other request for a
cancelled instance is refused as cancelled. A completed instance has nothing left to
cancel and is refused as completed. A step admitted before the cancellation goes
stale before its effect; a cancellation prepared before an effect lands goes stale
without claiming its key, and the same key then cancels afresh and reports that
effect. Cancelling one instance leaves its siblings running, and a request on
another branch than the instance's own is refused as an affinity mismatch, so a
fork's successor and its source end independently. A request without the start
capability is refused without claiming its key, and a performed history that cannot
be read within its retained bounds is refused as unavailable. An instance that has
used its whole retained-transition capacity cannot yet be cancelled; slice 5.3b closes
that with its budgets.

## Acceptance, Cost And Review

A user-authored definition must reach real proposal/assessment/approval/effect and
recovery through production entry. A static hardcoded workflow rerun with different
labels is insufficient. A declaration with an invalid unused branch must fail, while
a valid definition still requires fresh execution authority for the actual step.
The primitive builder, macro and bounded command adapter must lower equivalent meaning
to one canonical expanded definition; a syntax-specific registry, validator or runtime
path fails acceptance. Component reuse is proven through distinct qualified
occurrences, explicit public ports and independent inspection provenance.

Compile-fail evidence proves a raw/validated definition cannot publish without
prepared live affinity and that neither published nor compiled definition meaning can
reach any node owner without `AdmittedWorkflowTransition`; valid counterparts cross
the intended phases. Runtime evidence destroys compiled/status projections, rebuilds
them from authority and obtains the same next actions while copied plans, statuses,
wakes and prior transition receipts open no door.

Definition/instance branch affinity, exact source evidence, required inventory,
current approval, effect idempotency and cleanup must survive the decisive scenarios.
No copied receipt, raw ID, string node name or constructed lookalike can satisfy the
canonical owner handoff. Enforce the same typed publication return path used by
static graph features. Different live occurrences may share Rust types, so owner
affinity tests remain necessary.

Evidence acceptance distinguishes assignment, collection and consumption; proves
early collection, Back/navigation preservation, transition-occurrence neutrality,
per-subject selective invalidation and exact fresh-authentication intent/age/reuse.
An implementation that keys validity only by current node, whole proposal revision or
latest transition cannot pass.

Bounds cover definition nodes/edges/bytes, validation visits, admitted step work,
pending notifications, in-flight operations, retained evidence, history references,
revision pins, back-edge iterations and total deadline. Count provider, World,
dispatch and settlement contacts plus retained resources. A loop consumes one total
budget; a yield or duplicate delivery does not reset it.
Independent instances/branches share immutable contracts without a whole-runtime lock.
Program/definition canonicalization is publication work, not repeated per step.
Ordinary transition counters distinguish compiled-plan selection, live admission,
provider execution and settlement; graph expansion, canonicalization and whole-definition
traversal remain zero after warm compilation.
From Phase 2, the geometry cost contract and common-path examples gate every affected
phase. Qualify definition size, actual dependency coverage, unrelated geometry,
instance population and history independently; report numerical solver cost separately.
The 10k valid sparse definitions must fit a documented finite qualification profile.
No lower-runtime benchmark alone certifies workflow overhead. Bound total retained
compiled/progress state and per-run queues; stop/release tests must observe reclamation
without losing authoritative pins, history or recovery obligations. Inspectable cost
and typed failures are preserved on both ordinary and advanced public paths.

In-memory interruption/recovery does not claim durable restart. Native project
Save/Open and Store recovery belong to their governing milestones. This spec preserves
versioned definition, source and effect meaning for them; it does not serialize live
authority to pretend persistence already exists.

Use existing focused/public/process tests and scheduled qualification as appropriate.
Do not regenerate identity/protocol goldens for unchanged meaning. Reuse 9.17.4/9.17.5
evidence until a changed boundary invalidates it; do not run all three entire suites
after every edit. Required builds are warning-free without hiding production code.

Before boundary-relevant phase completion run affected owner/public/process checks,
formatting, dirty line caps, boundary-check and agent-context:

~~~text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
~~~

Review asks whether a definition grants authority, a status field substitutes for a
performed effect, a changed proposal reuses approval, a loop escapes bounds, or a
program/fork transition discards recovery. It also asks whether any authoring surface
can bypass canonical validation, whether component expansion creates hidden runtime
ownership, whether ordinary mutation can bypass an authority port, and whether a wake
can execute without an authenticated request. It must
answer the compiler omission and state-versus-evidence question from 9.17.4. No proof
ledger, test-for-test system or new generic runtime is an acceptance substitute.
Review also rejects per-step topology/history scans, unsound content-only concrete
binding reuse, incomplete negative dependencies, unbudgeted cold fallback, and a
friendly API that silently widens authority or drops partial effects. Infrastructure
uniformity alone neither proves scaling nor justifies unrelated schema migration.

## Documentation And Successor Handoff

Extend Query's existing ordinary-application-front-door guide with typed dynamic
definitions, primitive builder/macro/component/command authoring equivalence, explicit
control flow, publication/start, required evidence, approval, A/B revisions and
instance actions. Evidence examples cover early collection, per-subject coverage,
Back/navigation, selective invalidation and signing-intent authentication policy.
They show CAD model graphs as workflow values rather than
one workflow node per geometric primitive. Host/decl READMEs link to compiled examples.
From Phase 2, lead the front-door guide with the short solver chain and reviewed
component through the common surface, then show advanced publication/recovery controls.
Phases 2.1-2.6 explain effect/index admission, recovery, retention, shared inputs, exact reuse, component deltas, views and capture/reopen in
that guide and `worth-proprietary/docs/house/query-platform.md`.
Document finite defaults, typed waits/partial outcomes, cold versus warm cost and the
tested geometry/workflow scale envelope. Keep examples compiled against real facades;
Bank and proprietary adapters must not teach internal demand-settlement loops.
AI_README locates definition/component expansion, publication preparation, compiled
definition projection, per-transition admission, node-owner lowering, branch adoption
and outbound-effect owners.

Existing aftermath/resource/branch guides explain exact workflow cancellation,
partial outcomes, retention, migration and fork posture. CAD command references and
Bank public-consumer/process contracts document real authoring/action/recovery APIs,
including typed denial, fresh authorization and outbound recovery. No parallel
workflow guide or journal.

Revise `worth-proprietary/docs/house/query-platform.md` for the reviewed-geometry
definition, author/approve/advance actions, P0-to-P1 live-instance disposition,
operator-visible recovery and the exact public WORTH dependency pin used by the
certification run.

[9.17.7](./milestone-9.17.7.md) adds inbound occurrences under the external-effect
aftermath owner, then [9.18](./milestone-9.18.md) receives the completed graph,
program-evolution, workflow and inbound-effect contracts. Correction treats
definition/instance changes as governed semantic state and preserves exact performed
history; it never sets status backwards or revives provisional APIs.

The [cross-runtime merge roadmap](../cross-runtime/merging-and-branching-roadmap.md)
later reconciles definition graphs, instance progress, evidence and effect provenance.
Merging records cannot duplicate dispatch or turn parent approvals into authorization
for a changed merged proposal. This milestone preserves that meaning but does not
implement multi-parent merge, rebase, collaboration or durable reconstruction.
