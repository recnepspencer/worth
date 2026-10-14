# Milestone 9.17.6.5: Observable Workflow Truth

> **Status:** Planned. Successor to [9.17.6.4](./milestone-9.17.6.4.md). This
> milestone lets the owner of workflow truth be observed. It composes existing
> Query read, discovery, disclosure and authorization owners. It adds no store,
> no registry, no history log and no second authority. Implementation and
> certification remain outstanding.

## Why This Milestone Exists

WORTH's first principle is that truth has exactly one owner, and that other
runtimes "may observe it, derive from it, or ask its owner to change it". The
workflow kernel violates the observation half of that rule today:

- It retains instances, transitions, proposals and approvals as platform entity
  kinds lowered above the application catalog
  (`worth-query-execution/src/domain_computation/primary_graph/schema_layout/platform_entity_lowering.rs`).
  No consumer can name or query them.
- A caller sees workflow state only through the typed outcome its own request
  happened to return. There is no read of an instance's current node, its
  transitions, its proposals or its approvals.
- `PublishedWorkflowInstanceRef` and `PublishedWorkflowProposalRef` have no public
  constructor and no serialization, and there is no instance discovery. After a
  checkpoint restore, or in any later request or process, a caller has no lawful
  handle on an instance that already exists.
- Proposal and transition records do not retain the acting principal. A
  separation-of-duty rule cannot see who proposed, so consumers put a
  caller-asserted actor field in business input instead.

Every consumer that runs a multi-actor workflow is therefore forced to keep a
second copy of kernel truth (a status table, an instance registry, an audit log,
an actor column), which is exactly the defect WORTH exists to prevent. The
platform manufactures the violation. This is a correctness defect against the
platform's own principles, not a missing convenience, and this milestone treats
it that way.

The rationale for having no instance-status query was sound: a status projection
must never stand in for transition admission
([9.17.6](./milestone-9.17.6.md): "Neither compiled meaning nor instance status
can execute an operation"; "a stale projection cannot authorize a step"). That
rule stays. Principle 10 ("reporting is not authority") already permits a
read-only, derived observation that authorizes nothing. The missing piece is the
observation, not a relaxation of authority.

## Roadmap Placement And Dependencies

9.17.6.5 consumes the completed 9.17.6 workflow kernel, 9.17.7 inbound
occurrences and the 9.17.6.3 envelope subscription. Phases 1 to 3 do not depend
on 9.17.6.4 and may start beside it when capacity allows. Where both milestones
touch `worth-query-publication/src/application_entry/workflow/`, the later slice
rebases on the earlier one; neither duplicates the other's owner. Phase 4
consumes 9.17.6.4's certified workflow adapter and source-only lane.

This milestone does not reopen completed milestones, does not gate 9.17.7 or the
9.17 umbrella, and does not claim durable persistence (Store owns restart
recovery). It makes the existing process-local and checkpoint-restored truth
observable and re-acquirable.

## Forbidden Implementations

These are the mistakes this milestone exists to make impossible. Each is
plausible, each would ship a second source of truth or a second authority, and
each must be refused by compile-fail evidence, a court, or a facade/boundary
check, not by review alone.

| Forbidden implementation | Why it is wrong | Required evidence that it is absent |
| --- | --- | --- |
| A workflow status field, status table, history log, event stream or instance registry, kernel-side or host-side | A second copy of truth (Principle 1). Status already exists as transitions; history already exists as commits. | Topology review of every new file; no new platform kind or aspect except the required actor fields in Phase 3; destroying every derived index and rebuilding it yields identical observations. |
| An observation, or any value derived from it, accepted by `advance`, `approve`, `for_workflow_operation` or any execution entrance in place of `RequiredWorkflowOperation`, `RequiredWorkflowApproval` or transition admission | Observation would become authority (Principle 10). `PublishedWorkflowInstanceRef` / `PublishedWorkflowProposalRef` stay what they are today: non-authoritative locators that advance, approval, migration and fork continuation already accept and re-verify against truth. | Compile-fail examples: no observation type converts into or substitutes for `RequiredWorkflowOperation`, `RequiredWorkflowApproval` or transition admission. Advance continues to re-derive state through `select_current_transition`; approval keeps re-binding a proposal ref to the latest proposal transition and refusing a changed one. |
| `Serialize`, `Deserialize`, `from_parts` or any public constructor on `PublishedWorkflowInstanceRef` / `PublishedWorkflowProposalRef` | A forged or stale locator. Re-acquisition must re-resolve against truth. | Facade snapshot shows no constructor; compile-fail on construction; the only sources are a landed receipt or governed discovery at an exact observation. |
| An observation without an exact basis ("latest status") | Reports must name the truth they describe. | Every observation result carries the `WorthQueryApplicationReadObservation` it was read at; no entrance reads an implicit current world. |
| Making platform workflow kinds nameable in the application catalog, or a raw query over their layout | Breaks the catalog collision guarantee and freezes internal layout as public contract. | Observation is a typed workflow API. `platform_entity_lowering.rs` keeps kinds above the catalog. |
| Discovery by scanning unrelated population | Cost grows with unrelated truth; violates hot-path honesty. | Discovery uses indexes made current for the basis (the `discover_workflow_definition` pattern); cost courts hold with unrelated instance population increased 1x/10x/100x. |
| Ungated disclosure of instance existence, actors or approvers | Self-description grants no disclosure authority (Principle 10). | Every read goes through Query's disclosure contract; courts deny an undeclared reader; approver/actor identities need a separately declared audit read. |
| Recording an actor from business input, `StrategyCallerProvenance` or the settling caller | Caller-asserted identity is forgeable; it is the pattern Principles 3 and 4 forbid. | The actor is taken only from the admitted operation's `operation_scope_binding().principal()`; a court submits a mismatched actor field in input and proves it is ignored. |
| Any legacy or compatibility path: optional actor fields, an "unknown actor" state, an old-format decoder, a migration or conversion of predecessor records, a compatibility flag, or an alias kept for old callers | Clean cutover is the product decision (as in 9.17.6.4). A legacy path is a second contract, and an absent actor would fail open on separation of duty. | Actor fields are required. A predecessor checkpoint, installation or record without them is refused with a typed unsupported-contract outcome before readmission, original bytes preserved. Court over a pre-milestone checkpoint; enforcement turns red if a fallback is reintroduced. |
| A new rule evaluator, a second authorization path or a workflow-specific permission check for separation of duty | A second way to express policy. | Separation of duty uses the existing capability composition rules over Query-authored actor relations; no new rule clause kind. |
| Exposing `WorkflowInstanceProgressRetention`, a progress basis, or any cached projection as truth | Derived state promoted to truth (Principle 2). | No facade export; observations are read from kernel truth through the observation readers. |
| Exposing a fork's copied instance as a live, advanceable ref | Forks hold history, not callable authority (9.17.6). | Discovery on a fork returns copied instances as historical; advancing them is refused; court on a forked branch. |
| A new polling loop, wake, callback or scheduler to keep an observation fresh | The caller pumps; there is no background scheduler. | Live observation rides the existing 9.17.6.3 envelope subscription; no new stream type. |

## Current Boundary: What Exists And What Is Missing

Paths are relative to `workspaces/worth-query/crates/`; `primary_graph/` means
`worth-query-execution/src/domain_computation/primary_graph/`.

| Existing owner | Present guarantee | Gap this milestone closes |
| --- | --- | --- |
| `worth-query-publication/src/application_entry/retained_read.rs` (`WorthQueryApplicationReadObservation`), `request.rs` `retain_read` / `at(..)` | The one exact-basis read binder: branch identity and selected commit, lease-backed, not constructible. | Becomes the only basis for workflow observation and discovery. |
| `primary_graph/application_discovery/workflow.rs` (`discover_workflow_definition`) | Indexed lookup, `LIMIT 2` corruption detection, index currency for the basis, a non-authoritative ref exact for one occurrence. | The pattern instance discovery must mirror. Definitions shipped; instances, planned in 9.17.6, did not. |
| `primary_graph/workflow/schema/indexes.rs` | `instance-identity` and `proposal-identity` branch-scoped indexes are registered. | They have no lookup callers and are not in the index-currency set; discovery must make them current for the basis. |
| `primary_graph/application_attempt/workflow_instance_observation.rs` and its history reader | Advance re-verifies every ref field against truth; an ended-history basis already exists. | Add a read-only observation purpose that reads, reports and authorizes nothing. |
| `primary_graph/workflow/instance/adoption_inventory.rs` (`read_live_instances`) | Bounded enumeration of live instances for adoption. | It is a scan, not an index, and its public entrance is ungated (below). Enumeration for consumers needs an index. |
| `worth-query-publication/src/application_entry/programs/request.rs`, `programs/preparation.rs` (`workflow_inventory`) | Lists live instance ids on a branch for adoption. | **Disclosure defect:** the request stores `_principal` and never uses it (as does `programs/branch_set.rs`), so the inventory is not gated by any capability. The same inventory also leaks through `prepare`'s `WorkflowDispositionRequired { inventory }` / `WorkflowInventoryChanged { inventory }` denials, and through the public `WorthQuerySelectedProductOperation::workflow_adoption_inventory`, which a consumer reaches with no principal via `with_host_advancement` and `on_branch(..).select(..)` (`worth_query_host::facade::primary_graph`). Fixed in Phase 1 for every one of these routes. |
| `primary_graph/workflow/schema/approval.rs`, `primary_graph/application_attempt/workflow_transition_program/approval_decision/identity.rs` | Approvals retain the approver as a partition/slot/generation triple plus an authorization snapshot, failing closed when absent. | The pattern for recording proposal and transition actors. |
| `primary_graph/workflow/schema/proposal.rs`, `transition.rs` | Proposal and transition truth, no actor. | Required actor fields (Phase 3), clean cutover. |
| `primary_graph/application_attempt/workflow_proposal_program.rs` | Proposal replay matches operation, input and source only. | **Replay defect:** a different principal submitting identical input replays the first principal's proposal. Fixed in Phase 3. |
| `primary_graph/application_attempt/elevation_lifecycle_facts.rs`; `worth-query-declaration/src/application_capability/composition.rs` | Query authors requester/approver relations from the admitted principal into relation kinds the application declares and binds (`ApplicationCapabilityElevationDefinition::requester()` / `approver()`); `separation_of_duty` and `distinct_actor` rules are prohibition paths from the principal that traverse them. | The precedent for workflow actor relations. |
| `primary_graph/workflow/instance/transition/admission/selection.rs` | An advance at a proposal slot returns `PreparationDenied(WorkflowTransitionNodeUnsupported)` with only a node path. | A typed awaiting-proposal outcome (Phase 2). |
| `primary_graph/application_installation/program/checkpoint.rs` (`in_memory_rostered_program_from_checkpoint`) | Workflow facts survive restore as relational truth. | No caller can resume a restored instance; no court proves it. |

## Product Decision Lock

### Clean Cutover

This milestone follows the clean-cutover decision recorded in
[9.17.6.4](./milestone-9.17.6.4.md): no legacy support and no backward
compatibility. Breaking changes are permitted. Every consumer, fixture, example
and document moves to the current contract in the phase that changes it, and the
displaced implementation is deleted in that phase. No deprecated alias, forwarding
wrapper, optional-field-for-old-records, old-format decoder, version fallback,
automatic conversion, "unknown" placeholder, compatibility flag or grace period.
A predecessor definition, installation, archive or checkpoint that the new
contract cannot read is refused with a typed unsupported-contract outcome before
effects or readmission, with its original bytes preserved. Current-contract
recovery, idempotency, exact observation and custody remain required; they are
runtime truth, not compatibility.

### Observation Is Derived, Exact And Authorizes Nothing

Workflow observation is a typed, read-only API on the existing application
request. It reads kernel truth at one exact `WorthQueryApplicationReadObservation`
through the existing observation readers with a new read-only purpose. Every
result carries the observation it was read at. It returns facts, never handles
that execute: no `RequiredWorkflowOperation`, no `RequiredWorkflowApproval`, no
issued authority, no transition admission.

`advance` stays the only way to learn what may happen next with authority. An
observation that says an instance awaits an operation is a report of truth at a
basis; it is not permission and it may be stale by the time the caller advances.
Advance re-derives the transition through `select_current_transition` as today.

An instance observation reports, at its basis: identity, definition and program
revision recorded in truth (never the revision carried in a ref), subject, live
or ended state, the node it awaits and what that node needs (proposal, operation,
approval, inbound completion, condition, terminal), and bounded, paged history of
transitions, proposals (operation, input type, input identity, node, actor) and
approvals (decision, node, approver). The proposed input value is not retained and
is not reported; it remains domain data.

The read-only purpose spends no step budget, binds no deadline, admits nothing
and never installs, advances or evicts the warm progress projection
(`WorkflowInstanceProgressRetention`, which today's observation readers update)
from a basis other than the branch head admission would use. An observation at
an older basis or on a fork reads truth and leaves the projection untouched.
Deriving the awaited node is bounded by the existing history reconstruction
budget, as advance is; only the reported history is paged. A history or
enumeration page continuation is bound to the observation it was read at;
continuing it at another basis is refused with a typed denial, never silently
re-based, and a continuation is not a ref.

### No Second Store

The milestone adds no status field, status table, history log, event stream,
instance registry or cached projection exposed as truth. Indexes added or made
current for discovery are derived, branch-scoped and rebuildable from kernel
truth, and are proven rebuildable. The only new durable truth is Phase 3's
admitted actor attribution (see "Actors Are Admitted" for the one-fact rule).

### Discovery Re-Acquires Locators From Truth

Instance discovery mirrors `discover_workflow_definition`: an indexed lookup made
current for the basis, `LIMIT 2` corruption detection, and a ref that is exact for
one branch occurrence and grants no authority. Two lookups are provided:

- **Exact:** by the start intent the caller already owns (definition, subject and
  start key), which determines the instance identity, through the existing
  `instance-identity` index. The identity
  (`workflow_instance_program/intent_identity.rs`) also binds the definition
  entity, its content identity and structure, and the program revision the
  instance started under. Program adoption later changes the instance's recorded
  revision but not its identity, so exact discovery takes the start-time
  revision as part of the intent and never recomputes it from the current
  program; a court discovers an instance after adoption carried it forward.
- **Bounded enumeration:** paged instances of one definition, optionally of one
  subject, through a derived branch-scoped index made current for the basis,
  using an existing `DerivedIndexKind` (for example `RelatedEntityOrdering` over
  the existing instance-to-definition relation, the mechanism continuation
  ordering already uses), not a new index engine.

Results distinguish live instances (which yield a `PublishedWorkflowInstanceRef`)
from ended or cancelled instances (which yield history only). An instance a
migration ended reports its successor through the existing
`instance_migrated_from` relation, subject to the same disclosure, so
re-acquisition follows a migration instead of losing the instance; a successor's
identity, which also binds its source and fork, is never recomputed by the
caller. A fork's copied instances are historical on the fork. Fork continuation
and migration already take a `PublishedWorkflowInstanceRef`; discovery must leave
them a lawful source (the source branch's discovered ref, which they re-verify)
and must not add a constructor to serve them. Proposal refs are re-acquired from
an instance observation of the awaited proposal, never constructed by the caller.
A cross-process caller re-acquires by discovery; it never persists or forges a ref.

### Disclosure Is Governed

Observation, discovery and enumeration go through Query's existing disclosure
contract. The workflow spec declares the capability that may observe its
instances, scoped to the instance subject by default. Actor and approver identities
require a separately declared audit read; without it they are omitted, and the
observation states that they were withheld. Redaction may hide detail; it never
hides that custody, recovery or approval is incomplete. Only entity ids are
reported; the authorization principal snapshot and external identity material
are never exposed through a read.

The existing adoption inventory moves under the same gate. Its request's principal
is used, not stored and ignored. The gate covers every route that yields the
inventory or instance ids, including `prepare`'s inventory-carrying denials and
the public `WorthQuerySelectedProductOperation::workflow_adoption_inventory`,
which consumers reach without a principal. New discovery, enumeration and
observation entrances live on the principal-bearing application request, never as
public methods on `WorthQuerySelectedProductOperation`, even though
`discover_workflow_definition` sits there today. Definition discovery gains the
same principal-bearing, disclosure-gated entrance in Phase 1; its ungated
selected-operation method stops being a consumer route.

Authorizing program adoption itself (`adopt`, `prepare`, `publish` and branch-set
adoption on the programs request) is not this milestone's work: it is
[9.17.6.4 Phase 0.4](./milestone-9.17.6.4.md#phase-0-truth-guards). This
milestone consumes that authorization and adds only the disclosure gate for the
inventory routes it does not cover.

A refused read is decided from the principal and the caller-supplied scope before
any instance is read, and is the same typed refusal whether or not the instance
exists. Declaring the observation capability is optional on the workflow spec:
absent, no principal may observe or discover its instances. There is no default
that admits every principal.

### Actors Are Admitted, Recorded And Never Asserted

Proposal and transition records gain required actor fields: the admitted principal
of the operation that wrote them, as the partition/slot/generation triple used by
approvals, taken from `operation_scope_binding().principal()`. Deferred owner
settlement records the actor carried in the admitted operation's settlement basis,
not the caller that settles it.

This is a clean cutover. The actor fields are part of the current workflow
contract, not optional additions for old records. Bump the workflow contract
version so predecessor instances, checkpoints and installations are recognised and
refused with a typed unsupported-contract outcome before readmission, with their
original bytes preserved; they are never decoded under a fallback, defaulted to an
"unknown" actor, migrated or converted. Do not add a new kind inside the full
reserved workflow kind block; extend the existing proposal and transition aspects.
Proposal and transition identity digests may include the actor where that is the
honest identity.

Proposal replay matches the principal: identical input from another principal is
refused with a typed denial rather than replaying the first principal's proposal.
Including the proposer in the proposal's identity is an acceptable way to make
that structural.

### Separation Of Duty Uses Existing Rules

Following the elevation lifecycle precedent, Query authors workflow actor relations
(proposer, advancer, executor, approver of an occurrence) from admitted principals.
The existing `separation_of_duty` and `distinct_actor` capability composition rules
traverse them unchanged. No new rule clause kind, rule evaluator or workflow-only
permission check is added. Recorded actors are historical attribution compared by
entity id; they never re-authorize the recorded principal.

Three constraints follow from how those rules work, and Phase 3 resolves each
before it writes code:

- **Endpoints are application-declared.** The rules are paths from the principal
  over application-catalog relations, and platform workflow kinds stay
  unnameable. As with elevation, the workflow spec binds application-declared
  relation kinds that Query authors; the binding must still identify the one
  proposal or occurrence being decided, not every past proposal on the subject.
- **One fact, not two.** An actor relation and an actor field describe the same
  admitted principal. Either the relation is the record (the elevation precedent
  keeps no separate field) or both are written by the same owner in the same
  commit from the same principal, and a court proves they never disagree.
- **Absence must be impossible, not permissive.** `separation_of_duty` and
  `distinct_actor` are prohibitions: a matched path refuses, and a missing
  relation matches nothing, so a missing actor relation would pass them. Every
  current-contract occurrence therefore has its actor relation, written in the
  same commit as the fact it describes, and a court proves no current-contract
  path can commit an occurrence without one. Predecessor records never reach
  these rules because clean cutover refuses them. If the existing vocabulary
  cannot express a needed rule, stop and surface the conflict rather than add an
  evaluator or a workflow-only check.

### Awaiting A Proposal Is A Named State

An advance at a proposal slot returns a typed awaiting-proposal outcome naming the
node path, the operation and the input type, alongside the existing `Awaiting*`
family. `WorkflowTransitionNodeUnsupported` remains for genuinely unsupported
nodes.

### Live Observation Uses The Existing Envelope

A live instance observation is marked through the 9.17.6.3 envelope subscription:
it registers the instance facts it consumed as an ordinary consumer of that one
Query-owned subscription and is marked by the touched graph like any other. That
milestone's rule that "workflow coverage needs no marking" concerns admission
pins and stays unchanged; it does not mean observations are already covered. No
new stream, wake or polling contract.

## Consumer Guidance Hardening

These items were deferred from the 2026-10-13 documentation review. Each removes a
place where a careful consumer can still build a second way.

1. **Explicit workflow requirements.** Omitting `workflow_requirements` on a
   guarded binding currently means "exactly one approval", the opposite of what a
   reader expects. Make omission a compile error for guarded bindings and remove
   the implicit default and its encoding (clean cutover). A guarded binding
   declares `new()` or `requiring_approval::<Cap>()`; if a "one approval whose
   capability the definition chooses" meaning is still needed, it gets its own
   explicit named constructor. Predecessor installed contracts and archives that
   carry the implicit default are refused with a typed unsupported-contract
   outcome, never reinterpreted.
2. **One retry style for capability lifecycle commands.**
   `execute_capability_delegation_in_program` and its revocation sibling retry by
   re-issuing and matching `AlreadyCommitted`
   (`workspaces/worth-query-bank-world/crates/bank-server/src/estate_capability_admission/delegation_progression.rs`).
   9.17.6.4 places them outside `mutate()`, so its owning operation does not cover
   them as delivered. Give them a request-level owner that reuses 9.17.6.4's
   descriptor and fresh-admission model (Query owns the original command and its
   Query-scoped key; the caller never re-issues), or record in this plan, before
   Phase 4 closes, why they are kernel commands with a single documented retry
   contract. They may not remain an undocumented second style, and no third
   retry shape may be introduced for them.
3. **Mixed reference workflow.** A compiled, CI-run example in
   `worth-query-certification/examples/` using a neutral domain (for example a
   controlled document or a lab sample): unsigned data capture through an
   occurrence-only guarded operation; a condition-gated signature on a binding
   declared with `requiring_approval`; separation of duty between proposer and
   approver through recorded actors; an `AwaitInbound` external completion;
   discovery of the instance in a second request; observation of its history and
   approvers; and the complete caller loop, including the awaiting-proposal
   outcome. Its excerpts in the guides are marker-checked.
4. **Generated doc facts.** Enumerations that drifted before (workflow node kinds,
   `WorkflowProgressOutcome`, workflow validation denial kinds, mutation outcomes)
   are rendered into the docs from source by the existing doc tooling, and the
   check fails when they differ.
5. **Absence claims are checked.** A consumer-doc sentence that says a capability
   does not exist carries a marker naming either a known-gap row or a symbol whose
   absence the checker verifies against the facade snapshot. Unmarked absence
   claims fail the doc check.
6. **One prohibition list.** The four divergent "do not" lists (build guide §8,
   front door anti-patterns, host README "Host code must not", How WORTH Works
   §17) collapse into the capability index; the others link to it.
7. **Facade rustdoc routes to the guides.** The `worth-query-decl` and
   `worth-query-host` crate docs link the build guide and capability index.
8. **Agent skill copies stay in sync.** `.grok/skills` is derived from `skills/`
   and checked, or removed.
9. **Fence debt reaches zero for consumer docs.** The shrink-only fence debt
   seeded by 9.17.6.4 Phase 0 is zero for every document in the checker's
   configuration (`tools/boundary-check/config/consumer-docs.toml`), including
   `docs/*.md`, Query `docs/foundations/`, Query `docs/AI_README.md` and the
   `worth-query-host` and `worth-query-decl` READMEs. No configured document
   keeps silent debt.

## Adversarial Courtroom

Use the real Query composition root and installed public consumer programs. Each
court runs through the public facade.

| Court | Hostile sequence | Required outcome |
| --- | --- | --- |
| Cross-request re-acquisition | Start an instance; drop every in-memory value; in a new request, discover it by start intent and by paged enumeration; advance it. | The discovered ref advances exactly as the original would. A forged or altered start intent finds nothing; no ref can be constructed. |
| Checkpoint restore | Run to `AwaitingOperation`, capture a checkpoint, restore, discover, observe, advance and execute. | Observation after restore equals observation before at the corresponding basis; execution proceeds; no duplicate effect. |
| Observation is not authority | Observe "awaits operation X"; revoke the approver's grant or advance the instance elsewhere; then try to execute using anything derived from the observation. | Nothing derived from the observation reaches execution (compile-fail); a fresh advance re-derives and refuses or reissues correctly. |
| Exact basis | Observe at basis A; commit a transition; observe at basis B. | Each observation reports its own basis; A is unchanged when re-read at A. |
| Disclosure | A principal without the declared observation capability attempts discovery (for an existing and an absent instance), enumeration, observation, the adoption inventory, adoption `prepare` without dispositions, and the selected-operation inventory route; a principal with observation but not audit reads actors. | The same typed refusal for the first in every case, existing or absent; no route yields an instance id; actor and approver identities withheld and the withholding stated for the second; incomplete custody still visible. |
| Fork isolation | Fork a branch with a live instance; discover on both branches. | The fork reports a historical copy, not an advanceable ref; the source branch is unaffected. |
| Actor attribution | Propose as A with an input field naming B; advance as C; approve as D; execute as E. | Recorded proposer A, advancer C, approver D, executor E; the input field is ignored for attribution. |
| Separation of duty | Declare a distinct-actor rule between proposer and approver; approve as the proposer; then as another principal. | Denied, then allowed. No current-contract path commits an occurrence without its actor relation. |
| Clean cutover | Restore a pre-milestone checkpoint and import a pre-milestone archive whose workflow records lack actors or whose guarded bindings use the implicit approval default. | Typed unsupported-contract refusal before readmission or effects; original bytes preserved; no fallback decode, default actor, migration or conversion. Current-contract restore and import succeed. |
| Cross-principal replay | Principal B submits A's exact proposal input and key shape. | Typed refusal; A's proposal is not returned to B. |
| Awaiting proposal | Advance at a proposal slot. | Typed awaiting-proposal outcome naming operation and input type; no denial. |
| Cost | Increase unrelated instances 1x/10x/100x and one instance's history length; run an ordinary non-workflow request beside them. | Exact discovery cost independent of unrelated population; enumeration and reported-history cost bounded by the page size requested; state derivation bounded by the existing history reconstruction budget; the ordinary request's index-currency cost does not grow with workflow population. |
| Rebuildability | Destroy every derived index used by discovery and rebuild it. | Identical discovery and observation results. |

Mutation probes must turn these courts red when: discovery skips index currency;
observation reads an implicit current world; a ref gains a public constructor;
the actor is read from input; an occurrence commits without its actor relation; a
predecessor record is readmitted instead of refused; replay
ignores the principal; the disclosure gate is bypassed on any inventory route; a
refusal differs for an existing and an absent instance; an observation at an
older basis updates the progress projection.

## Ordered Phase Plan

### Phase 1: Governed Discovery And Re-Acquisition

Gate the existing adoption inventory with its request principal, on every route
named in Current Boundary. Declare the
observation capability on the workflow spec and route discovery through Query's
disclosure contract. Make the instance and proposal identity indexes current for
the basis. Ship exact and enumerated instance discovery returning live refs or
history. Prove cross-request, checkpoint-restore and fork courts. The next phase
may trust lawful re-acquisition.

### Phase 2: Instance Observation

Add the read-only observation purpose to the existing observation readers and the
typed observation API at an exact basis: state, awaited node and its needs, and
paged history of transitions, proposals and approvals. Add the awaiting-proposal
outcome. Wire live observation to the envelope subscription. Prove observation is
not authority, exact basis, disclosure, cost and rebuildability.

### Phase 3: Admitted Actor Attribution

First settle the three separation-of-duty constraints (endpoints, one fact,
absence impossible) against the existing rule vocabulary, or stop and surface
the conflict. Record required proposer, advancer and executor actors from admitted
principals, including deferred owner settlement, and bump the workflow contract
version. Make proposal replay
principal-aware. Author
workflow actor relations so existing separation-of-duty rules apply. Report actors
through the audit read. Prove attribution, separation of duty, cross-principal
replay, and typed refusal of every predecessor checkpoint, installation and
archive. Update every fixture, example and consumer to the current contract in
this phase and delete what it displaces; no legacy path remains.

### Phase 4: Consumer Guidance Hardening And Closure

Deliver the nine hardening items, consuming 9.17.6.4's certified owning entry for
item 2 and its workflow adapter for the reference example. Remove every closed
row from the known-gap tables in `docs/capability-index.md` and Query
`docs/foundations/workflows.md`. Run facade, boundary, doc and affected Query
gate suites. Both correctness and structure reviews clear the joined result.

## Documentation Deliverables

Documentation lands with the phase that ships each capability, in the existing
documents. Marker-checked or doctested examples only, per 9.17.6.4 Phase 0.

| Document | Required change |
| --- | --- |
| `docs/capability-index.md` | Task rows for discovering, observing and auditing workflow instances and for separation of duty over workflow actors; delete each known-gap row as it closes. |
| Query `docs/foundations/workflows.md` | Observation, discovery, disclosure and actor sections; the observation API and its no-authority rule; delete closed known-gap rows. |
| `docs/build-an-application.md` §6 | Re-acquire, observe and audit in the run sequence; replace the "Observing an instance. There is no instance-status query." paragraph with the observation API and its no-authority rule; the awaiting-proposal outcome in the outcome table. |
| `docs/how-it-works.md` §13 | Observation as derived truth at a basis; actor attribution; why advance remains the only authority. |
| `docs/glossary.md` | Workflow observation, instance discovery, actor attribution. |
| `worth-query-host/README.md`, `docs/api.md` | New facade names and their audience. |

## Must Ship And Closure

- A consumer can re-acquire any live instance it may see, in any request and after
  checkpoint restore, without holding prior in-memory values, and cannot construct
  or forge a ref.
- A consumer can observe an instance's state, awaited need and history at an exact
  basis, under declared disclosure, and nothing it observes can authorize anything.
- Proposals and transitions record admitted actors; separation of duty over
  workflow actors uses the existing rules; no occurrence lacks its actor, and no
  legacy decoder, default or migration path exists for predecessor records.
- The adoption-inventory disclosure defect and the cross-principal proposal replay
  defect are closed with courts.
- Every forbidden implementation above is absent, each by its stated evidence.
- No new store, registry, log, stream, scheduler or authority path exists.
- The nine hardening items are delivered or, for item 2 only, recorded with a
  single documented retry contract.
- The known-gap tables list none of the gaps this milestone closes.
- Final joined correctness and structure review, facade/boundary/doc checks and
  affected Query gate suites all agree.
