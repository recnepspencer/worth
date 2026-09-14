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

A definition is not a runtime installation or grant of action authority. An instance
is not a generic task runner. Static feature contracts, Query planning and managed
execution, Bridge/Signal eligibility and World publication remain the existing owners.

Complete one proposal -> two required assessments -> approval -> real application
journey in Phase 1. Then finish branching, bounded retries, definition revisions,
branch forks, program adoption and external effects. A graph editor UI is not required;
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
journey for non-geometric authority, exactly-once effect and external recovery.
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


### Exact definition shape

ApplicationWorkflowSpec is the installed vocabulary contract, containing allowed
operation/query/assessment/approval references, connection/result bindings, authoring
capability, maximum effect ceiling, policy and resource profiles. It does not embed
a user's current definition or instance state.

AuthoredWorkflowDefinition contains:

| Field family | Meaning |
| --- | --- |
| Identity and parent revision | Explicit workflow identity and observed predecessor; no last-writer-wins revision overwrite |
| Subjects and typed inputs | Domain/native identity bindings, units, required/optional posture and admitted scope |
| Nodes | Qualified node identity, installed vocabulary reference, typed arguments and declared result binding |
| Data connections | Exact output-to-input contract, occurrence mapping and conditional availability |
| Control connections | Successor on a typed result/condition; complete failure/terminal routing |
| Required evidence | Applicable inventory/coverage contract, accepted assessment source/version and pass policy |
| Effect/approval policy | Explicit automatic, proposal, approval and application boundaries; no inferred authority |
| Resource and progress policy | Graph/queue/work/retention/deadline limits, retry bounds and cancellation posture |

The definition's authored graph is finite. IDs locate nodes inside a validated
definition; they are not operation capabilities. The public Rust builder uses typed
references; transport/archive decoding returns only an untrusted draft with the same
structural validator. Existing Query expression machinery evaluates predicates.
No arbitrary Rust, string expression evaluator, dynamic service lookup or callback
receiving the application runtime can be installed through a node.

### Node semantics and complete control flow

| Node kind | Consumes | Produces and limitation |
| --- | --- | --- |
| Query/read | Installed query and current or explicit retained read contract | Disclosed typed observation; no mutation permit |
| Operation | Installed typed intent, inputs, required source expectation and current authority | Existing actual mutation terminal; performed and settlement are distinct |
| Assessment | Required subject/source contract and installed producer | Exact completed evidence, possibly failing; Pending is not success |
| Approval | Immutable proposal revision, evidence requirements and approval policy | Owner-published decision under current approver authority; no general capability |
| Condition | Declared typed inputs and pure Query predicate | A declared branch selection, not an effect or new observation |
| Evidence join | Required inventory and compatible completed evidence | Complete/failing/incomplete/stale result under declared policy |
| Bounded back edge | Explicit retry/revision reason, attempt bound and retained state | A new transition occurrence; cannot reuse changed intent under an old key |
| Terminal | Declared complete, rejected, cancelled or recovery-pending posture | Exact result and remaining owner custody, not discarded resources |

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
  -> bounded current vocabulary/support/source validation
  -> ValidatedWorkflowDefinition
  -> freshly admitted definition publication
  -> World-performed definition revision
  -> freshly admitted instance start
  -> owner-managed eligibility / step admission / operation publication
  -> typed result / evidence join / next transition / terminal custody
~~~

Declaration validation supplies structural facts. Primary-graph workflow admission
binds them to exact application/installation, branch/incarnation, program/component/
Bridge evidence, vocabulary revision, predecessor definition and consumed dependency
versions. Publication consumes that stronger result and rechecks affinity/currentness
at the actual owner boundary. A changed program or source cannot be fixed by retagging.
Two competing updates to the same predecessor cannot both become its current revision.

Publishing a definition is an ordinary governed model mutation. It reserves storage/
validation resources and runs its required actual candidate checks; its source and
idempotency semantics match 9.17.4. A validated draft is not already-published meaning.
A descriptive receipt does not construct a PublishedWorkflowDefinitionRef; the owner
projects that reference only from the actual performed publication.
An archive/import is data requiring the same admission.

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

Approval binds instance identity/incarnation, definition revision, proposal revision,
operation meaning, subjects/scope, required evidence and source footprint, approver,
purpose, expiry and policy. Approve/reject are declared operations; caller booleans,
role strings or edited database status fields cannot create an admitted approval.
The application step checks current authority/revocation and complete evidence again
at its owner handoff. A currently valid approval cannot override failed hard integrity.

Changing a proposal, consumed source, required inventory, applicable rule or semantic
operation contract invalidates affected evidence. Unrelated branch/source edits do
not. Compatible reuse is a checked dependency/contract equivalence result, never
the default because an ID or displayed value matches.

### Instance progression and linear effect custody

An instance binds its branch, immutable definition revision, subjects and logical
transition history. Authoritative instance records carry typed references to actual
step publications/outcomes; live managed resources remain in their existing owners.
Persisting a step name/status is not evidence that its effect happened.

Before invoking an operation, the owner admits/reserves its stable transition identity
and idempotency intent. A performed step's receipt and successor eligibility are
recorded through the existing execution/result-publication contract. If the effect
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
Only owner-issued transition results advance them. Cancellation is not rollback:
performed effects remain in the result, and recovery can outlive the cancelled run.

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
let draft = request.workflow_definitions(ReviewedGeometryChange)
    .draft(workflow_identity, definition_limits)?;
let draft = draft
    .operation(ProposeChange)?
    .assess(RequiredGeometryChecks)?
    .approve(GeometryReviewer)?
    .operation(ApplyApprovedChange)?;
let validated = draft.validate()?;
let outcome = request.publish_workflow(validated)
    .idempotency(definition_key).execute();
// Match the performed outcome to obtain its owner-published definition reference.
let started = request.start_workflow(&published_definition, subjects)
    .idempotency(instance_key).controls(instance_limits).execute();
// Observe/advance via managed owner progress with fresh request authority.
~~~

Typed condition/join/back-edge builder forms must expose result availability and
bounded recurrence. Public examples include a rejected branch and a proposal
revision, not only the linear successful form. The shipped signatures follow the
named authority transitions; private spelling may not remove a validation product.

The model exposes ordinary declared approve, reject, revise, cancel, inspect,
retire-definition and migrate-instance actions. Each takes the appropriate typed
identity/source expectation and current authority. It does not expose a
set_workflow_status operation, arbitrary node executor or deserialize-proof route.

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
  definition/{authored,canonical,inputs,identity}.rs          N immutable model meaning
  connection/{data,control,join,retry}.rs                     N distinct edge semantics
  validation/{binding,availability,termination,resources}.rs  N structural checks
worth-query-installation/src/application_program/workflow/
  {vocabulary,adapters,definition_contract}.rs                N/R concrete installed bindings
worth-query-execution/src/domain_computation/primary_graph/workflow/
  definition/{admission,publication,revision,retirement}.rs   N branch-local definition facts
  instance/{start,observation,transition,migration,retirement}.rs N model instance meaning
  proposal/{source,revision,application}.rs                   N operation proposal contract
  evidence/{inventory,assessment,approval,currentness}.rs     N owner-backed evidence meaning
  adoption/{inventory,dispositions}.rs                       N participant in 9.17.5
  recovery/{continuation,disposition}.rs                      N adaptation to actual custody
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
worth-query-package-archive/                                 E/R definition codecs, not authority
worth-query-decl/src/facade.rs worth-query-host/src/facade.rs E/R reexports
worth-query-certification/tests/application_graph/workflow/
  {definition,evidence,revision,branching,recovery,resources}.rs N grouped public proof
~~~

Authoring/validation is pure meaning; installation binds known implementations;
primary graph stores and mutates definition/instance/proposal facts through admitted
operations; managed_run owns execution lifecycle; proof/World/Signal retain their
authority. Model-step records are projections/references to canonical performed
facts, not another commit/dispatch ledger. No persisted live lease or arbitrary
status table may become an execution permit.

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
Implement finite typed definition construction, validation, branch-local publication,
instance start, proposal, two assessment requirements, approval and application.
Lower directly to existing managed/conditional entry and actual World effects.

Ship real authoring and instance command surfaces immediately. The first proof includes
assessment rejection then retry, wrong-source approval, duplicate wake, denied apply,
successful exact output and cleanup. No graph builder-only checkpoint.
The next phase trusts runtime-authored meaning producing actual admitted effects.

### Phase 2: Control flow, coverage and external recovery

Add typed conditions, joins and bounded revision/retry with complete outcome handling.
Close the separate-assessment/ABA/new-required-subject sequence. Bind Bank's real
approved payment workflow and HTTP/user-node/rail recovery; test lost response,
owner-unpublished and externally pending outcomes with independently observed effects.

Complete authority revocation at waits, effect ceiling denial and resource admission.
Remove any new client-authored workflow loops or manual receipt routing. Existing
static Bank semantics remain protected. The next phase trusts lawful branching
and honest partial effects, not just a successful linear workflow.

### Phase 3: Definition revisions, branches and program adoption

Ship definition retirement, A/B coexistence, explicit instance migration and fork
continuation admission. Implement the definition/instance participant in 9.17.5
adoption with exact state/evidence dispositions. Test compatible/incompatible
program changes while approval, delivery and external recovery are pending.

No global instance revision bump, copied approval or duplicate external execution.
An unaffected sibling progresses. The next phase trusts definition and program
evolution without losing meaning, authorization or custody.

### Phase 4: Lifecycle, public closure and qualification

Complete bounded observations, queued runs, retention, deadline/iteration accounting,
cancellation, close and retirement across every new state. Finish public codecs,
discovery, authoring/control-flow examples and real product adapters.

Compile-fail raw draft as published definition, approval description as execution
permit, omitted validation and cross-role value substitution, with valid counterparts.
Runtime cases supply genuinely issued foreign/stale evidence. Reuse owner affinity/
queue/recovery tests and inspect failures before broader reruns. Delete obsolete
workflow glue and any weaker new experimental entry. All dynamic behaviors close
here before 9.18; no unfinished workflow migration is handed to correction.

## Acceptance, Cost And Review

A user-authored definition must reach real proposal/assessment/approval/effect and
recovery through production entry. A static hardcoded workflow rerun with different
labels is insufficient. A declaration with an invalid unused branch must fail, while
a valid definition still requires fresh execution authority for the actual step.

Definition/instance branch affinity, exact source evidence, required inventory,
current approval, effect idempotency and cleanup must survive the decisive scenarios.
No copied receipt, raw ID, string node name or constructed lookalike can satisfy the
canonical owner handoff. Enforce the same typed publication return path used by
static graph features. Different live occurrences may share Rust types, so owner
affinity tests remain necessary.

Bounds cover definition nodes/edges/bytes, validation visits, admitted step work,
queued wakes, in-flight operations, retained evidence, history references, back-edge
iterations and total deadline. Count actual provider/World/dispatch contacts and
retained resources. A loop consumes one total budget; a yield does not reset it.
Independent instances/branches share immutable contracts without a whole-runtime lock.
Program/definition canonicalization is publication work, not repeated per step.

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
program/fork transition discards recovery. It must also answer the compiler omission
and state-versus-evidence question from 9.17.4. No proof ledger, test-for-test system
or new generic runtime is an acceptance substitute.

## Documentation And Successor Handoff

Extend Query's existing ordinary-application-front-door guide with typed dynamic
definitions, explicit control flow, publication/start, required evidence, approval,
A/B revisions and instance actions. Host/decl READMEs link to compiled examples;
AI_README locates definition, managed execution, branch adoption and effect owners.

Existing aftermath/resource/branch guides explain exact workflow cancellation,
partial outcomes, retention, migration and fork posture. CAD command references and
Bank public-consumer/process contracts document real authoring/action/recovery APIs,
including typed denial and fresh authorization. No parallel workflow guide or journal.

[9.18](./milestone-9.18.md) receives the completed static graph, program evolution
and dynamic workflow contracts. Correction must treat definition/instance changes as
governed semantic state and preserve exact performed-step history and external-effect
posture; it does not undo by setting status backwards or revive provisional APIs.

The [cross-runtime merge roadmap](../cross-runtime/merging-and-branching-roadmap.md)
later reconciles definition graphs, instance progress, evidence and effect provenance.
Merging records cannot duplicate dispatch or turn parent approvals into authorization
for a changed merged proposal. This milestone preserves that meaning but does not
implement multi-parent merge, rebase, collaboration or durable reconstruction.
