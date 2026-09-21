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

## Required Public Experience

These target Rust examples compile in the existing host/decl audience tests when
their phase ships. Definitions and operations use typed installed references.

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

Paths are under workspaces/worth-query/crates unless qualified.
E existing; R extend existing owner; N new semantic responsibility.
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
worth-query-admission/src/authentication_event/
  {intent,issuance,clock}.rs                                  N purpose/age/reuse proof in named clock
worth-query-execution/src/domain_computation/primary_graph/workflow/
  schema/{relations,version}.rs                              N Query-contributed fact schema
  definition/{preparation,publication,revision,retirement}.rs N branch-local definition facts
  definition/compilation/{lowering,plan,reconstruction}.rs   N rebuildable execution meaning
  instance/{start,observation,migration,retirement}.rs        N model instance meaning
  instance/transition/{selection,admission,settlement}.rs     N one-step authority and result
  instance/status_projection.rs                              N discardable current-status view
  proposal/{source,revision,application}.rs                   N operation proposal contract
  evidence/{inventory,assessment,approval,currentness}.rs     N owner-backed evidence meaning
  adoption/{inventory,dispositions}.rs                       N participant in 9.17.5
  recovery/{continuation,disposition}.rs                      N adaptation to actual custody
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
worth-query-package-archive/src/workflow_definition/
  {codec,compatibility}.rs                                   N draft codec only, never live facts
worth-query-decl/src/facade.rs worth-query-host/src/facade.rs E/R reexports
worth-query-certification/tests/application_graph/workflow/
  {authoring,definition,evidence,revision,branching,recovery,resources}.rs N grouped public proof
worth-proprietary/crates/worth-cad-entry/src/application/workflow/
  {vocabulary,reviewed_geometry}.rs                            N proprietary declarations
worth-proprietary/crates/worthy-house-application/src/workflow/
  {composition,actions,program_adoption}.rs                    N real product integration
worth-proprietary/crates/worthy-house-certification/tests/journeys/workflow.rs
                                                               N cross-repository court
worth-proprietary/docs/house/query-platform.md                 R consumer contract and operation
~~~

Authoring/validation is pure meaning; installation binds known implementations;
primary graph stores and mutates definition/instance/proposal facts through admitted
operations; existing operation, output-demand, managed-read and change-delivery owners
execute their node families; proof/World/Signal retain authority. Model-step records
are projections/references to canonical performed
facts, not another commit/dispatch ledger. No persisted live lease or arbitrary
status table may become an execution permit.

`authoring` owns pure construction and deterministic component expansion only;
`definition` owns canonical model meaning and `validation` owns its structural proof.
The macro is a thin facade over `authoring::builder`, never a second validator.
Execution `definition/compilation` owns discardable plans derived from published truth;
`instance/transition/admission` alone binds current authority and resources for one
step. Neither may absorb the other's lifecycle or become a current-definition store.
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

### Phase 2: Control flow, coverage and effect custody

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

### Phase 3: Definition revisions, branches and program adoption

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

### Phase 4: Workflow-kernel lifecycle and public closure

Complete bounded observations, pending notifications, retention, deadline/iteration accounting,
cancellation, close and retirement across every new state. Finish public codecs,
discovery, authoring/control-flow examples and real product adapters.

The proprietary slice closes observation, cancellation, retention, resource
exhaustion, cold reconstruction, command/action exposure and operator diagnostics
through the same product composition used in Phases 1-3, not a new closure fixture.

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

## Documentation And Successor Handoff

Extend Query's existing ordinary-application-front-door guide with typed dynamic
definitions, primitive builder/macro/component/command authoring equivalence, explicit
control flow, publication/start, required evidence, approval, A/B revisions and
instance actions. Evidence examples cover early collection, per-subject coverage,
Back/navigation, selective invalidation and signing-intent authentication policy.
They show CAD model graphs as workflow values rather than
one workflow node per geometric primitive. Host/decl READMEs link to compiled examples;
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
