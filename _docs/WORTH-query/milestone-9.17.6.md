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
After that kernel is usable, finish first-class inbound external occurrences and
asynchronous effect recovery in the final phase. A graph editor UI is not required;
the public typed authoring API and real product command surface are required.

Closure includes safe A/B definition coexistence, fresh approval/source validation,
independent required inventory, exact step idempotency, bounded execution, partial
effects, branch/program currentness, instance migration and every resource lifecycle.
All new behavior consumes the lower proofs from 9.17.4/9.17.5. No duplicated runtime,
string-node authority, reimplemented admission or parallel publication path.

## Current Boundary And Reuse

- Query managed_run/workflow_graph_execution.rs owns active workflow execution,
  provider step progression and completed/paused outcomes.
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

Definition compilation lowers to those installed operation/managed execution
contracts. New primary_graph/workflow code owns branch-local authored-definition,
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
- Remove an assessment supplier while its obligation remains applicable. Completion
  becomes unmet, not vacuously passing. Required executable ports cannot be left
  unbound at program adoption.
- Exercise rejection, bounded revision/retry, cancellation before and after effects,
  and exact recovery. Separately committed steps are not one rollback transaction.
- Adopt an incompatible program with a running instance, retained reader and
  unpublished effect. Deny missing dispositions, then accept owner-validated
  migration/retirement and recovery custody on exact branch coverage. Old
  candidates cannot publish after activation.

Use real CAD proposal/change and required geometry assessment contracts from
9.17.4 for the first endpoint. Add Bank's real approved business-payment/process
journey for non-geometric authority and exactly-once effect; its asynchronous inbound
recovery closes in the final phase.
No test-only callback substitutes for apply or approval authority.

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
Exhaust definition graph, prepared candidate, queued run, retained evidence and
iteration limits independently at their owner boundaries. Do not manufacture a
giant fixture that exhausts all limits together.

### Final inbound occurrence and asynchronous effect court

After kernel closure, the final phase supplies the inbound counterpart to the existing
outbox. A host receives an authenticated Bank rail or CAD/solver callback; Query admits
one bounded immutable occurrence, correlates it to the exact effect/transition, and
the instance owner consumes it through World publication. Raw payloads and identifiers
carry no completion authority.

Lose the synchronous response after the external effect, then deliver duplicate,
reordered and post-cancellation callbacks. Require one occurrence and one lawful
consumption, retain blocked external truth, and never reopen a cancelled instance.
Unknown/foreign correlation, invalid authentication/protocol/payload and exhausted
retention deny before workflow effects. Observe the external owner, occurrence,
correlation/consumption and World result independently. A shared-truth callback,
mutable status or direct resume command fails this court. Store remains the owner of
restart-safe rediscovery; this proof states process-local durability.

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

Instances bind immutable definition revision, exact subjects/inputs, proposal source
expectations, step receipts and required evidence. New instances select the current
revision. Existing ones stay pinned until explicit admitted migration/cancellation.
Pins are bounded custody, not retained permission. Historical definition revisions
may support existing instances under the selected branch's admitted program and
current host support/security. A revision that is historical on one branch may still
be current on another; neither an archive nor a copied receipt makes it executable.

Signal owns wakes/eligibility; Query freshly admits executing steps; World publishes
effects. Approval binds proposal identity, dependency versions, workflow revision,
scope, approver and expiry. Relevant source changes invalidate it. Migration must
revalidate affected evidence, never relabel approvals. Conditions consume admitted
observations with explicit current/retained posture.

Step idempotency binds instance, definition revision, transition occurrence and intent.
Replay resumes owner results, not another effect; changed intent conflicts. Definitions
declare finite graph size, work/retention budgets, deadlines, queues, retry bounds and
explicit terminal/partial-effect continuations.


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
operation/query/assessment/approval/external-input references, connection/result
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
reserved resources. Existing managed execution consumes it once and returns the real
owner outcome. A compiled plan, status projection, node ID, wake or prior transition
receipt cannot construct or substitute for this admission.

Compilation is definition-publication/cold reconstruction work. Ordinary wakes select
precompiled transition meaning and perform fresh admission; they do not expand
components, canonicalize definitions, rediscover ports, traverse unrelated nodes or
re-decide evidence requirements. Live facts that cannot be compiled remain explicit
inputs to transition admission.

### Node semantics and complete control flow

| Node kind | Consumes | Produces and limitation |
| --- | --- | --- |
| Query/read | Installed query and current or explicit retained read contract | Disclosed typed observation; no mutation permit |
| Operation | Installed typed intent, inputs, required source expectation and current authority | Existing actual mutation terminal; performed and settlement are distinct |
| Assessment | Required subject/source contract and installed producer | Exact completed evidence, possibly failing; Pending is not success |
| Approval | Immutable proposal revision, evidence requirements and approval policy | Owner-published decision under current approver authority; no general capability |
| External input wait | Installed inbound protocol and exact effect/transition correlation requirement | A committed correlated occurrence; receipt or payload alone cannot complete the step |
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
  -> managed execution / operation publication
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
authored-source dependency versions, result posture and performed publication.
Its native source footprint excludes irrelevant output publications, permitting
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
the same principal and proposal are visible.

Changing a proposal, consumed source, required inventory, applicable rule or semantic
operation contract invalidates affected evidence. Unrelated branch/source edits do
not. Compatible reuse is a checked dependency/contract equivalence result, never
the default because an ID or displayed value matches.

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

Step identity binds application/branch/instance incarnation, definition revision,
node and logical transition occurrence plus intent equivalence. Transport retry and
duplicate Signal wake reuse that transition's result. An authored loop creates a new
transition only after the prior result is settled or has a declared carried recovery
posture; it cannot escape a key conflict by inventing a fresh attempt ID.

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

### Inbound external occurrences

The final phase adds an inbound counterpart to the outbox, not a broker, scheduler or
workflow-local queue. Installed `ApplicationExternalInputBinding` declares protocol,
correlation, authenticated source, bounded payload and decoding. Host adapters own
transport; they cannot declare completion or select a transition.

Inbound progression is compiler-visible and owner-issued:

~~~text
ReceivedExternalEnvelope
  -> AuthenticatedExternalEnvelope
  -> AdmittedExternalOccurrence
  -> CorrelatedExternalOccurrence
  -> ConsumedExternalOccurrence
~~~

Admission commits one immutable occurrence before separate consumption. Deduplication
binds protocol, authenticated source, external message identity and correlation family;
correlation additionally binds the exact effect, branch, instance/definition and
transition. Bounded custody retains blocked, reordered or late occurrences.

Dispatch, acknowledgement, remote completion, inbound observation and consumption stay
distinct. A lost response remains indeterminate until an installed external-owner
callback or reconciliation proves completion; redispatch is safe only under the
installed remote-idempotency contract. Occurrences and typed relations, never mutable
status, are authoritative. Signal only wakes; Store later owns restart discovery.
Serialized envelopes and live leases carry no authority.

### Branches, revision retirement and migration

Definitions A/B coexist as immutable facts; one selected current definition per
workflow identity governs new starts on that branch. Retirement prevents new starts
without deleting history or live instance custody. Existing instances remain pinned
within bounded retention until completed, cancelled or explicitly migrated.

A branch fork preserves historical definitions/instance records under exact component
meaning. It does not clone executable approval, live leases or dispatch permission.
Continuing work there requires explicit admitted new-instance/fork disposition with
new branch identity, current authority and verified prior-effect references. The
default is historical inspection, not automatic execution. Cannot reconcile a prior
effect safely: deny continuation with its typed requirement, rather than replay it
under a fresh key.

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
// Observe/advance via managed owner progress with fresh request authority.
~~~

The primitive builder can construct this exact graph without the macro, and a bounded
command adapter can construct it without Rust syntax. Public equivalence tests compare
their canonical expanded definition rather than token streams or display layout. Typed
condition/join/back-edge forms expose result availability and bounded recurrence.
Public examples include component reuse, a rejected branch and a proposal revision,
not only the linear successful form. The shipped signatures follow the named authority
transitions; authoring accepts no request or authority, and private spelling may not
remove validation or publication preparation.

The model exposes ordinary declared approve, reject, revise, cancel, inspect,
retire-definition and migrate-instance actions. Each takes the appropriate typed
identity/source expectation and current authority. It does not expose a
set_workflow_status operation, arbitrary node executor or deserialize-proof route.
The final phase adds declared `await_external` nodes backed only by installed inbound
protocol references; it does not expose arbitrary callback URLs or caller-selected
correlation targets.

Error results name workflow/definition/instance/node and the violated binding,
source, authority, budget or transition contract. Disclosure may redact detail,
never erase the fact that execution or recovery is incomplete. Observation resource
close releases interest; it does not itself cancel the instance or its mandatory work.

## Destination Topology

Paths are under workspaces/worth-query/crates unless qualified.
E existing; R extend existing owner; N new semantic responsibility.
No new scheduler, generalized workflow runtime, queue registry or expression compiler.

~~~text
worth-query-declaration/src/application_program/workflow/
  vocabulary/{definition,operations,assessments,approvals}.rs  N installed semantic references
  authoring/{builder,component,expansion,macro_surface}.rs      N one graph-shaped authoring model
  definition/{authored,canonical,inputs,identity}.rs          N immutable model meaning
  connection/{data,control,join,retry}.rs                     N distinct edge semantics
  validation/{binding,availability,termination,resources}.rs  N structural checks
worth-query-declaration/src/application_schema/external_input/
  {binding,protocol,correlation}.rs                           N final-phase inbound meaning
worth-query-installation/src/application_program/workflow/
  {vocabulary,adapters,definition_contract}.rs                N/R concrete installed bindings
worth-query-installation/src/application_schema/external_input/
  {installed_contract,source_authentication}.rs               N final-phase installed inbound contract
worth-query-execution/src/domain_computation/primary_graph/workflow/
  definition/{preparation,publication,revision,retirement}.rs N branch-local definition facts
  definition/compilation/{lowering,plan,reconstruction}.rs   N rebuildable execution meaning
  instance/{start,observation,migration,retirement}.rs        N model instance meaning
  instance/transition/{selection,admission,settlement}.rs     N one-step authority and result
  proposal/{source,revision,application}.rs                   N operation proposal contract
  evidence/{inventory,assessment,approval,currentness}.rs     N owner-backed evidence meaning
  adoption/{inventory,dispositions}.rs                       N participant in 9.17.5
  recovery/{continuation,disposition}.rs                      N adaptation to actual custody
  external_input/{admission,correlation,consumption,custody}.rs N final-phase inbound occurrences
worth-query-execution/src/domain_computation/
  managed_run/workflow_*                                     E/R sole run/step/yield owner
  provider_session/readmission/workflow.rs                   E/R fresh continuation admission
  artifact_owner/{workflow_authority,frozen_workflow_authority}.rs E/R canonical carriers
  convergence_epoch/workflow_cleanup.rs                      E/R actual resource cleanup
worth-query-execution/src/domain_computation/primary_graph/
  conditional_operation/ application_output_demand/           E/R eligibility and exact closure
  product_operation/program_adoption/                        E/R existing branch adoption
  application_discovery/                                    E/R governed definitions/instances
worth-query-publication/src/application_entry/
  workflow/{definition,instance,actions,progress}.rs           N public builders/results
  external_input/{receive,outcome}.rs                          N final-phase host entry/result
worth-query-package-archive/                                 E/R definition codecs, not authority
worth-query-decl/src/facade.rs worth-query-host/src/facade.rs E/R reexports
worth-query-certification/tests/application_graph/workflow/
  {authoring,definition,evidence,revision,branching,recovery,resources}.rs N grouped public proof
  external_input/{admission,correlation,consumption}.rs        N final-phase protocol proof
~~~

Authoring/validation is pure meaning; installation binds known implementations;
primary graph stores and mutates definition/instance/proposal facts through admitted
operations; managed_run owns execution lifecycle; proof/World/Signal retain their
authority. Model-step records are projections/references to canonical performed
facts, not another commit/dispatch ledger. No persisted live lease or arbitrary
status table may become an execution permit.

`authoring` owns pure construction and deterministic component expansion only;
`definition` owns canonical model meaning and `validation` owns its structural proof.
The macro is a thin facade over `authoring::builder`, never a second validator.
Execution `definition/compilation` owns discardable plans derived from published truth;
`instance/transition/admission` alone binds current authority and resources for one
step. Neither may absorb the other's lifecycle or become a current-definition store.
`external_input` is structurally separate because protocol authentication, occurrence
custody and transport-facing failure behavior differ from workflow definition and
instance progression. Host-specific HTTP, broker and rail adapters remain in their
product/server owners and depend only on the publication/host facade.

CAD application composition binds the real reviewed geometry workflow; Bank's
application_definition/workflows binds approved business-payment and estate nodes.
Their domain handlers and numerical algorithms remain unchanged in ownership.
Real Bank HTTP/user-node/process adapters expose the new declared authoring/instance
actions with current authentication. No product-specific runner duplicates Query.

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
`AdmittedWorkflowTransition` for every executed node. Lower directly to existing
managed/conditional entry and actual World effects. Prove builder/macro/command-adapter
semantic equivalence on this endpoint.

Ship real authoring and instance command surfaces immediately. The first proof includes
assessment rejection then retry, wrong-source approval, duplicate wake, denied apply,
successful exact output and cleanup. No graph builder-only checkpoint.
Collect one approval/evidence item before its consuming node and prove its exact signing
intent, age and reuse policy are enforced at collection and consumption.
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
evidence and stale or differently purposed authentication cannot sign.

Complete authority revocation at waits, effect ceiling denial and resource admission.
Remove any new client-authored workflow loops or manual receipt routing. Existing
static Bank semantics remain protected. The next phase trusts lawful branching
and honest partial effects, not just a successful linear workflow. Delayed external
callbacks remain explicitly unsupported until the final phase rather than entering
through a temporary `resume` or status mutation.

### Phase 3: Definition revisions, branches and program adoption

Ship definition retirement, A/B coexistence, explicit instance migration and fork
continuation admission. Implement the definition/instance participant in 9.17.5
adoption with exact state/evidence dispositions. Test compatible/incompatible
program changes while approval, delivery and performed-effect recovery are pending.

No global instance revision bump, copied approval or duplicate external execution.
An unaffected sibling progresses. The next phase trusts definition and program
evolution without losing meaning, authorization or custody.

### Phase 4: Workflow-kernel lifecycle and public closure

Complete bounded observations, queued runs, retention, deadline/iteration accounting,
cancellation, close and retirement across every new state. Finish public codecs,
discovery, authoring/control-flow examples and real product adapters.

Destroy and reconstruct compiled definitions and current-status projections while
instances are ready, waiting and recovery-pending. Prove identical next-transition
meaning, zero authority from reconstructed projections, and no authored-graph
rediscovery in the ordinary wake path.

Compile-fail raw/validated definition as publication-ready, published/compiled
definition as transition authority, approval description as execution permit and
cross-role value substitution, with valid counterparts.
Runtime cases supply genuinely issued foreign/stale evidence. Reuse owner affinity/
queue/recovery tests and inspect failures before broader reruns. Delete obsolete
workflow glue and any weaker new experimental entry. At this boundary the complete
workflow kernel is usable for local effects and existing outbound-effect postures;
there is no provisional inbound callback API for callers to migrate later.

### Phase 5: Inbound external occurrences and final qualification

Add installed external-input protocols and the received -> authenticated -> admitted
-> correlated -> consumed phase progression. Commit immutable bounded inbox
occurrences before separate workflow consumption, reuse exact outbox correlation and
external-owner idempotency, and expose host entry/results without transport ownership.

Complete Bank's real HTTP/user-node/rail callback recovery and one CAD/solver-style
asynchronous callback. Exercise lost synchronous response, duplicate/reordered and
late delivery, invalid source/protocol/payload, unknown and foreign correlation,
blocked consumption, cancellation, retention exhaustion and exact cleanup. Count
actual admission, World, wake, dispatch and consumption contacts. State the
process-local durability posture and preserve Store's later restart authority.

Delete any direct resume/status/message-routing substitute. Run final public facade,
protocol, lifecycle, warning-free and repository enforcement checks. All dynamic and
external behaviors close here before 9.18; no unfinished workflow migration or inbox
authority is handed to correction.

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
reach managed execution without `AdmittedWorkflowTransition`; valid counterparts cross
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
queued wakes, in-flight operations, retained evidence, history references, back-edge
iterations and total deadline. Final-phase inbound bounds cover envelope bytes,
authentication/decoding work, admitted and unconsumed occurrences, pending correlation,
duplicate contacts and retention. Count actual provider/World/dispatch/inbound/
consumption contacts and retained resources. A loop consumes one total budget; a yield
or duplicate delivery does not reset it.
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
ownership, and whether raw inbound data can advance or complete an instance. It must
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
definition projection, per-transition admission, managed execution, branch adoption,
outbound effect and inbound occurrence owners.

Existing aftermath/resource/branch guides explain exact workflow cancellation,
partial outcomes, retention, migration and fork posture. CAD command references and
Bank public-consumer/process contracts document real authoring/action/recovery APIs,
including typed denial, fresh authorization, external-input authentication,
deduplication, correlation, consumption and process-local durability. Product/server
docs name their HTTP/rail adapters without assigning them message meaning or workflow
authority. No parallel workflow guide or journal.

[9.18](./milestone-9.18.md) receives the completed static graph, program evolution
and dynamic workflow contracts. Correction must treat definition/instance changes as
governed semantic state and preserve exact performed-step history and external-effect
posture; it does not undo by setting status backwards or revive provisional APIs.

The [cross-runtime merge roadmap](../cross-runtime/merging-and-branching-roadmap.md)
later reconciles definition graphs, instance progress, evidence and effect provenance.
Merging records cannot duplicate dispatch or turn parent approvals into authorization
for a changed merged proposal. This milestone preserves that meaning but does not
implement multi-parent merge, rebase, collaboration or durable reconstruction.
