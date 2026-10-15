# Milestone 9.17.6.5: Observable Workflow Truth

> **Status:** Planned. Follows [9.17.6.4](./milestone-9.17.6.4.md) and consumes
> the existing workflow kernel, completed 9.17.7 inbound contract and 9.17.6.3
> envelope subscription. One current contract replaces the displaced surfaces
> and formats. Implementation and certification remain outstanding.

## Goal And Governing Decision

Make Query's own workflow truth sufficient to build a multi-actor application:
declare its policies, run it through the owning operation API, and discover,
observe and audit it without copying workflow state or supplying actor identity.

Framework grade means the lawful path is complete and the misleading path is
mechanically unavailable. AI-first means explicit authoring, typed lifecycle
states, one ownership model, narrow audience facades and compiled examples.
Instructions to remember a missing gate, supply an actor, reconstruct a private
identity or choose between retry styles do not satisfy this milestone.

Query keeps one workflow kernel and one capability decision pipeline. It gains
required canonical actor attribution, a positive recorded-actor requirement in
that pipeline, governed discovery and read-only observation, and owning adapters
for the remaining capability commands. Observation never becomes authority.

**No legacy or backward compatibility.** Replace affected contracts outright.
Do not preserve old bytes, optional attribution, omitted-policy meanings, API
aliases, fallback readers or a second ordinary retry style to avoid a break.
Keep genuinely distinct semantics distinct: observation and admission; proposer,
advancer, executor and approver; current logical workflow migration and format
migration; domain mutation and a kernel capability command.

## Placement And Scope

This is the successor to 9.17.6.4. Its owning adapters consume that milestone's
certified descriptor, fresh-admission, recovery and cleanup contracts. Declaration,
actor facts, disclosure and index work can be prepared beside 9.17.6.4 after their
shared boundaries are fixed; integration must consume the completed predecessor,
rather than implement a competing operation owner.

The milestone does not reopen the 9.17 umbrella or retroactively gate 9.17.7.
It covers process-local truth and supported current-format checkpoint restore.
Store still owns durable restart recovery. Current-format adoption, logical
workflow migration and fork continuation remain supported native operations.

Do not add a status table, actor store, history log, instance registry, scheduler,
polling loop, new observation stream or second permission evaluator. Existing
commits, workflow facts, capability admission and the envelope retain ownership.
New indexes and projections are derived and disposable.

## Grounding In The Existing Implementation

Paths below are relative to `workspaces/worth-query/crates/`.
`primary_graph/` abbreviates
`worth-query-execution/src/domain_computation/primary_graph/`. This is the
planning baseline; implementation must review changes landed by 9.17.6.4 before
editing a shared owner.

| Existing boundary | Actual guarantee or defect | Required change |
| --- | --- | --- |
| Publication `application_entry/retained_read.rs`, `request.rs` | `WorthQueryApplicationReadObservation`, `retain_read` and `at(..)` already bind reads to an exact retained basis. | Reuse them; no implicit-latest workflow read API. |
| `primary_graph/application_discovery/workflow.rs` | Definition discovery has indexed lookup and duplicate detection, but is public on the selected operation and calls broad `ensure_primary_indexes_for_basis`. | Govern the consumer entrance and make only the selected discovery indexes current. |
| `primary_graph/workflow/schema/indexes.rs` | Branch-scoped instance and proposal identity indexes already exist. | Add their lookup consumers and selected-index currency; use existing derived-index machinery for bounded enumeration. |
| `primary_graph/application_attempt/workflow_instance_observation.rs`, `step_budget.rs` | Observation purposes serve advance and close, interacting with admission allowance, deadlines and retained progress. | Add a truly read-only purpose with bounded reconstruction and no admission/progress side effects. |
| `primary_graph/workflow/instance/adoption_inventory.rs`; publication `programs/` | Adoption has an inventory; 9.17.6.4 Phase 0.4 supplies adoption authorization. That is distinct from permission to disclose instance identities. | Gate inventory reads, preparation denials and advanced exposure without duplicating adoption authorization. |
| `primary_graph/workflow/schema/proposal.rs`, `transition.rs`, `approval.rs` | Proposals and transitions lack actor fields; approvals already retain the admitted approver and authorization evidence. | Required canonical actor roles from the native admitted basis. |
| `primary_graph/application_attempt/workflow_proposal_program/` | Proposal identity/replay binds operation, input and source without a proposer contract. | Principal-bound proposal intent and replay refusal across principals. |
| Declaration `application_capability/composition.rs`; execution `domain_computation/authorization/capability_lowering.rs`, `capability_observation/decision_denial.rs` | Existing separation-of-duty and distinct-actor graph rules are prohibitions: a match denies; no matching actor relation does not deny. | Add a positive recorded-actor requirement to the existing composition/lowering/decision pipeline. Missing evidence must deny. |
| `primary_graph/application_attempt/elevation_lifecycle_facts.rs` | Native admission already authors requester/approver attribution. | Reuse its admitted-identity discipline, without copying its application-relation representation into a second workflow actor fact. |
| Declaration `application_program/workflow/vocabulary.rs`, `application_operation/binding/workflow_requirements.rs` | Workflow specs have no observation policy; guarded binding omission has an implicit approval meaning. | Explicit workflow observation/actor policies and explicit guarded evidence requirements. |
| `primary_graph/workflow/schema/version.rs` and workflow readers | Current protocol deliberately accepts earlier condition shapes; retained assessment includes compatibility interpretation of an absent program revision. | Version the changed workflow contract and remove compatibility-only interpretation in affected readers. |
| Publication capability delegation/revocation; Bank `estate_capability_admission/delegation_progression.rs` | Native commands still expose caller re-issue/replay handling outside the ordinary mutation owner. | Mandatory request-level command-owning adapters using 9.17.6.4 ownership mechanics. |

The current graph prohibition vocabulary cannot, by itself, enforce the required
positive presence of an exact private workflow actor. Declaring it sufficient or
adding a workflow-only check would both be dishonest. The extension below is
part of the destination, not a conflict deferred until implementation.

## Product Decision Lock

### 1. One Supported Contract, With Honest Refusal

Update affected declaration, installation, canonical identity, portable package,
workflow fact, checkpoint and facade contracts together. Required actor meaning
participates in the authoritative record and proposal replay identity. Change
protocol versions and canonical digests wherever interpretation changes; retaining
an old identity is not a constraint.

An artifact using a displaced affected contract is unsupported. Detect and refuse
it with a typed format/version outcome at the earliest owning load, installation,
restore or replay boundary, before interpreting it as current truth or producing
effects. A rejected restore/import does not partially install state. Leave the
source artifact intact; no destructive reset or automatic format migration.
Do not add an upgrade tool, dual writer, compatibility mode or actor backfill.

A malformed current-format record missing its required actor is corruption, not
a supported old record, an anonymous actor or a successful inequality comparison.
Genuinely optional lifecycle data stays optional: an instance awaiting a proposal
has no proposer yet, and a transition with no performed operation has no executor
role. Represent those states explicitly instead of inventing an unknown principal.

Remove old aliases, omitted guarded-requirement semantics and affected fallback
branches when the current producer lands. The old condition encoding and absent
revision branch are concrete cleanup targets where the replaced workflow protocol
owns them. This does not authorize deleting unrelated platform history support.
Logical migration between valid current workflow definitions remains a distinct
native operation; it is not backward-format compatibility.

### 2. Canonical Actor Roles, Authored Only By Admission

The admitted operation or command supplies actor identity. A public API accepts
no actor override, role callback or caller-authored authorization witness.
Business input and `StrategyCallerProvenance` are never attribution sources.

| Role | Canonical source | Required when |
| --- | --- | --- |
| Proposer | Principal of the admitted proposal command | A proposal is published. |
| Advancer | Principal whose admitted workflow command causes the transition | A transition is published, including native start/close/migration paths as applicable. |
| Executor | Principal of the admitted performed operation | A workflow transition records performance of that operation. |
| Approver | Principal of the admitted approval decision | An approval decision is published. |

Use typed role-bearing records/constructors and the existing complete entity
identity discipline (partition, slot, generation). Define each transition kind's
admitted source so system completion or adoption cannot invent a principal.
Deferred publication retains the original admitted actor: a later settler is
neither the original executor nor the proposer merely because it finishes work.

These canonical workflow fields are the one actor truth. Policy and audit
read them; any lookup index is derived. Do not additionally require applications
to declare and maintain proposer/advancer/executor relations. Do not store an
independent actor projection whose agreement is a convention or dual-write test.
Platform workflow kinds remain outside the application catalog; no raw-layout
query or newly public platform kind substitutes for the typed workflow facade.

A current-format carry, restore, migration or fork preserves attribution to the
original occurrence. It does not replace historical actors with the principal
performing adoption or reconstruction. Comparing a historical actor does not
re-authorize that historical principal; the acting principal still needs its own
fresh current admission.

Proposal intent binds the proposer. Replay matching the same native intent and
principal can recover its original result. A different principal presenting the
same key/input/source cannot recover or silently replace the first proposal:
return a typed mismatch/refusal without leaking it through a weaker read route.

### 3. Positive Recorded-Actor Requirements In The Capability Pipeline

Extend the existing capability actor composition with a typed requirement over
a native recorded-role source. The source identifies the precise workflow
proposal/transition occurrence being admitted, not every actor who ever touched
the subject. It is selected by Query from the admitted workflow binding.

An author declares the required role and comparison against the acting principal.
The installed program validates the source/role against the guarded operation.
There is no stringly typed platform path, application actor entity argument or
consumer-supplied observation accepted as evidence.

The requirement is **presence AND the declared actor comparison**, within the
same Query-to-Bridge/Signal authorization pipeline that already owns capability
rules. A missing, foreign, stale, wrongly scoped or malformed required source
cannot satisfy it. A role that has not occurred cannot satisfy a requirement for
that role either. The distinct-actor case compares the full entity identity.

This requires a principled typed clause/source extension, its canonical encoding,
installation validation and lowering. It does not require a new rule engine,
workflow permission service or post-admission side check. Existing graph-based
prohibitions retain their independent relational meaning; they are not a substitute
for positive actor presence. Share fact selection and role semantics between policy
and observation without making an observation an authorization input.

A matched actor comparison alone grants nothing. Current capability grant,
scope, revocation, program, branch and operation admission remain mandatory.
Prove separation of duty against another proposal on the same subject, another
instance, another branch and a reused entity slot with a different generation.

### 4. Explicit Authoring And Governed Disclosure

Every guarded operation binding explicitly chooses its workflow evidence
requirements, including occurrence-only and approval-requiring forms. Omission
is an authoring error. Remove the legacy omitted encoding rather than keep its
bytes under a new spelling.

Every workflow spec explicitly chooses disabled observation or a governed policy,
including its audit policy and actor requirements. Explicit `NotApplicable` is
valid where a distinction is genuinely inapplicable; an implicit permissive
default is not. Installed identity includes these declarations.

Observation, definition/instance discovery and enumeration enter through the
principal-bearing application request and existing disclosure machinery.
Instance access is subject-scoped by default. Enumeration across a definition
requires an explicitly declared definition-scoped read; a missing subject never
widens a subject-scoped grant. Definition discovery has an explicit declared scope
rather than borrowing an arbitrary instance's subject.

Authorize the principal and caller-supplied scope before existence-dependent
lookup. Undeclared/unauthorized readers receive the same typed refusal for present
and absent identities. After that gate, validate the located record against the
authorized scope; a record outside it is undisclosed, just as an absent record is.
Pages cannot smuggle records outside it. Historical fact selection does not
resurrect an expired or revoked disclosure grant: read admission is fresh even
when the selected data basis is old.

Actor/approver identities require the separately declared audit read. A reader
without it sees explicit withholding rather than misleading actor absence.
Redaction must still report incomplete custody, recovery or approval honestly.
Never disclose authorization snapshots or external identity credentials.

Apply the same inventory disclosure contract to the normal programs request,
adoption preparation denials and the advanced selected-operation route.
Unprivileged preparation errors must not embed instance inventories. Remove or
internalize ungated consumer entrances, including definition discovery; calling
through an advanced/host facade must not bypass the gate. Adoption authority
itself remains the existing 9.17.6.4 owner.

### 5. Exact-Basis Discovery And Observation, With No New Authority

All reads use `WorthQueryApplicationReadObservation`. Results identify the basis
whose truth they report; they never silently consult the current branch head.

Provide exact instance discovery and bounded definition/subject enumeration.
Exact discovery uses Query's canonical original start-intent material, including
the start-time definition/program identity where the existing identity requires
it. Supply a typed non-authoritative discovery intent from that owner; do not
make applications assemble private digest fields or recompute an old intent
using the currently installed revision. It may be retained as ordinary intent
data; it is not a serialized live ref, authority or required instance registry.
Enumeration permits reacquisition when no original intent was retained.

An adoption that carries an instance forward does not change its start identity.
A logical migration reports the successor through existing native relations,
with separate disclosure of that successor. The caller never manufactures the
successor identity. A fork's copied instance is historical on the fork; source
discovery remains a lawful locator for native fork continuation.

Discovery distinguishes live, historical and missing results. Only live discovery
yields a `PublishedWorkflowInstanceRef`; an observation of a pending proposal
can reacquire its `PublishedWorkflowProposalRef` after the same gates.
References stay opaque and nonserializable, with no public `from_parts`.
They locate native truth and authorize nothing.

Observation derives state, recorded definition/program revision, subject, awaited
node/need and bounded history of transitions, proposals and approval decisions.
Report actor roles only under audit disclosure. Proposed domain input values
remain domain data; report the existing input type/identity, not a new stored
payload. State derivation uses the existing bounded history reconstruction
contract; history pages do not require replaying an unbounded log.

A read-only observation purpose spends no workflow step budget, binds no workflow
admission deadline and does not install, advance or evict the warm progress
projection. Normal request cancellation and read/reconstruction resource budgets
still apply. Old-basis and fork reads cannot modify head admission progress.

A page continuation binds runtime/branch, exact basis, workflow identity,
authorized scope, filters and ordering. It is opaque, non-authoritative and cannot
be reused at another basis or with different selection parameters. Fresh read
admission applies on continuation; no page token preserves a revoked read grant.

Make only the selected lookup/enumeration indexes current. Ordinary requests must
not inherit workflow population work through all-index currency. An explicitly
budgeted cold rebuild may reconstruct derived indexes from truth; hot exact
lookup and bounded pages must not hide a population scan. Duplicate exact hits
are corruption, never arbitrary selection. Hot cost is the existing index lookup
plus bounded matching candidates or emitted page, not linear work over unrelated
population. Reconstruction work is separately identified and budgeted.

`advance` remains the authority-producing workflow decision. Observation cannot
convert to an operation/approval requirement, admission pin or executable handle.
Advancing and executing after a read re-derive current state and current permission.

An advance at a proposal slot returns a typed `AwaitingProposal` outcome naming
the node, operation and input type; it is not an unsupported-node denial.
Live observation uses the existing 9.17.6.3 envelope/touched-fact subscription.
There is no additional scheduler, poll contract or workflow stream.

### 6. One Owning Lifecycle For Capability Commands

Ship request-level owning adapters for delegation and revocation. Query retains
the original native command and its Query-scoped identity; the caller advances
the owner instead of resubmitting the command and interpreting `AlreadyCommitted`.

Reuse 9.17.6.4's descriptor, fresh-admission, exact recovery and independent cleanup
mechanics. Native capability command owners still decide authority and publication.
A command is not a fake domain mutation, workflow operation, domain action or
manufactured output. Common ownership mechanics belong to one owner; distinct
native admission and result semantics stay distinct.

Retries use fresh requests and preserve the original command across conflicts,
deferred settlement and recovery. A changed principal/program/scope cannot reuse
old admission. Ambiguous publication recovers the exact keyed native result,
without duplicating effects. Terminal cleanup remains independently advanceable.

Move Bank and public examples to these adapters. Remove displaced ordinary
consumer re-issue entrances and manual replay plumbing; keep implementation-native
execution behind its owning boundary. Documenting the second retry style does
not satisfy closure.

## Intended Caller Experience

The following is a target sketch, not a claim that these names exist today.
Final spellings follow the existing facade and are compiled in the reference
consumer. The original start intent is typed Query material, not a hand-built
workflow identity:

```rust,ignore
// Illustrative target: one request, declared policy, one exact retained basis.
let basis = request.retain_read(&phase)?;
let read = request.at(&basis);
let found = read.discover_workflow_instance(&workflow, &original_start)?;
let view = read.observe_workflow_instance(&found)?;
```

The real example matches typed live/historical/missing and denied/withheld
outcomes. It discovers in a later request and after current-format restore.
Execution uses the existing owning workflow adapter and its complete caller
loop; no new advance API is introduced just to resemble the sketch.
A capability command uses the same retain-and-advance lifecycle vocabulary while
retaining its native command semantics. No example supplies actor ids, writes
workflow status, reconstructs a ref or matches native replay receipts.

## Destination Topology And Owners

Paths below are destinations inside existing audience crates, not a second
framework. Create files only when their responsibility lands; split by owner
rather than collecting the milestone in a large miscellaneous module.

| Owner | Destination and responsibility |
| --- | --- |
| Declaration: `worth-query-declaration/src/application_program/workflow/` | `observation_policy.rs`, `actor_policy.rs` and existing vocabulary: explicit disclosure/role declarations. |
| Declaration: `application_capability/` | `recorded_actor.rs` with typed native sources/comparisons; extend existing composition and canonical encoding. |
| Declaration: `application_operation/binding/workflow_requirements.rs` | Explicit guarded evidence authoring; remove omitted legacy semantics and update macros. |
| Installation: `worth-query-installation/src/application_program/workflow/` | Role/policy validation and installation through the existing vocabulary owner; identities include meaning. |
| Execution: `primary_graph/workflow/` | `actor/record.rs`, `actor/selection.rs`: one canonical role record and exact source selection. Existing `schema/` owns required fields, indexes and protocol changes. |
| Execution: `domain_computation/authorization/` and its existing capability observation | Extend lowering and positive requirement decisions within the existing pipeline; no parallel evaluator. |
| Execution: `primary_graph/application_attempt/` | Existing proposal, transition, approval, settlement and observation readers publish/consume the new contract; private `workflow_observation/` owns read state, history and paging. |
| Execution: `primary_graph/application_discovery/workflow/` | Definition, instance and enumeration responsibilities over selected-index currency. The current `workflow.rs` becomes their narrow entry. |
| Publication: `worth-query-publication/src/application_entry/workflow/` | Discovery/observation entrances and typed outcomes, including `AwaitingProposal`, over existing retained reads and owning adapters. |
| Publication: `application_entry/capability_command/` | Delegation/revocation adapters and command-specific admission/recovery/close; delegate common lifecycle mechanics to the existing operation owner. Replace the flat ordinary command entrances. |
| Existing portable/package/checkpoint owners | Current-only encoding and early typed refusal; no compatibility package or migration service. |
| Certification and doc tooling | Focused workflow-truth courts, one compiled neutral mixed workflow example, and the existing boundary/doc checker owners. |

Audience facades export caller tasks, policies and outcomes; execution layout,
actor selectors, read purposes and progress retention stay private. Keep module
and function size within repository laws, with public entry modules explaining
their task and routing to domain owners. Do not create placeholder files or a
duplicate helper tree merely to match this table.

The concrete skeleton below uses **E** for an existing owner extended in place,
**N** for a new responsibility, **R** for a replaced entrance and **D** for removed
compatibility code. All crate paths are under `workspaces/worth-query/crates/`.

```text
worth-query-declaration/src/
  application_program/workflow/                  E: workflow meaning
    vocabulary.rs                               E: explicit policy contract
    observation_policy.rs                       N: read/audit scopes
    actor_policy.rs                             N: occurrence role bindings
  application_capability/
    composition.rs                              E: positive actor requirement
    recorded_actor.rs                           N: source/comparison vocabulary
    canonical_composition_components.rs          E: current canonical meaning
  application_operation/binding/
    workflow_requirements.rs                    R: explicit evidence; D: omission
worth-query-installation/src/
  application_program/workflow/
    actor_policy.rs                             N: source/role validation
    observation_policy.rs                       N: policy installation
    vocabulary/installation.rs                  E: install integration
    vocabulary/support_identity.rs              E: installed meaning
  application_capability/canonical_basis.rs      E: current contract basis
worth-query-execution/src/domain_computation/
  authorization/
    capability_lowering.rs                      E: same decision pipeline
    capability_observation/recorded_actor.rs     N: positive evidence/denials
  primary_graph/
    workflow/
      actor/{record,selection}.rs                N: canonical native roles
      schema/{proposal,transition,approval}.rs   E: required role facts
      schema/{version,indexes}.rs                E: current format/derived lookup
    application_attempt/
      workflow_observation/{state,history,paging}.rs  N: read-only derivation
      workflow_instance_observation.rs           E: explicit read purpose
      workflow_proposal_program/                 E: principal-bound intent
    application_discovery/
      workflow.rs                               R: narrow governed routing
      workflow/{definition,instance,enumeration}.rs   N/move: indexed readers
worth-query-publication/src/application_entry/
  retained_read.rs                              E: existing exact read basis
  workflow/{discovery,observation}.rs             N: request audience entrances
  workflow/operation/                           E: predecessor owning adapter
  capability_command/                           N: native command adapters
    delegation.rs
    revocation.rs
    operation.rs
    operation/{admission,recovery,close}.rs       command semantics, not a new engine
  capability_delegation.rs                       R: ordinary re-issue entrance
  capability_revocation.rs                       R: ordinary re-issue entrance
worth-query-certification/
  examples/observable_workflow.rs                N: compiled mixed consumer
  tests/workflow_truth/                          N: focused public-boundary courts
```

The structural axes are declaration meaning, installation validation, native
truth, authorization decisions and public lifecycle. The command adapter tree
contains command-specific semantics; shared lifecycle stays in the predecessor
operation owner. Read derivation depends on canonical facts, never the reverse;
authorization selects those facts privately and never consumes a public view.
Existing facade/dependency checks preserve those directions. M2.7.1 enters through
the certified audience facade without exposing or moving these private owners.
No uncommitted generic framework or versioned legacy subtree is reserved.

## Adversarial Courtroom

Use real Query composition roots, installed programs and public audience facades.
Internal corruption injection is permitted only to test the owning boundary.
Choose representative hostile cases and focused mutation probes; do not multiply
every test by every input dimension.

| Court | Hostile sequence | Required result |
| --- | --- | --- |
| Current contract / clean refusal | Load displaced affected package/checkpoint/fact versions, then separately restore a valid current-format checkpoint. | Typed unsupported outcome before partial install/effects; original bytes intact. Current restore retains discovery, actors and exact recovery. No fallback reader. |
| Required roles | Publish proposal, approval and each relevant transition kind; attempt malformed current records; supply misleading business actor input. | Required admitted roles are present and correct; malformed facts refuse; input never determines actor. Not-yet-performed roles remain typed lifecycle states. |
| Deferred attribution | A admits an operation, B settles/reconstructs it, C adopts the current program. | Original executor remains A; B/C cannot overwrite it. New commands retain their own distinct actors. |
| Actor policy | Proposer tries to approve; another currently authorized actor tries; required source is absent, foreign or from another occurrence/branch; an entity slot is reused. | Same actor denies; distinct authorized actor passes only with exact positive source evidence; missing/foreign/stale sources deny; generation matters. |
| Proposal replay | B submits A's exact key/input/source; A retries after ambiguous publication. | B receives typed refusal; A recovers exactly its own result with no second proposal/effect. |
| Reacquisition / adoption / migration | Drop refs, discover in another request and after restore; carry by adoption; migrate to a successor. | Original start intent still resolves correctly; bounded enumeration also works; successor follows native truth and disclosure; fresh native advance remains required. |
| Fork isolation | Discover source and copied instances on a fork; perform native fork continuation. | Copied instances are historical, cannot advance, and do not contaminate source progress; lawful source locator supports continuation. |
| Observation is not authority | Observe a pending step, change workflow truth or revoke a grant, then attempt execution. | No observation conversion supplies authority; fresh advance/admission sees changed truth and refuses or returns the correct new result. |
| Exact basis / paging | Read A, change head to B, continue A; attempt token reuse with B, another runtime, filters or scope; revoke read permission. | A stays A; mismatched continuations refuse; revoked disclosure cannot continue; neither read updates head progress or consumes workflow steps. |
| Disclosure / audit | Unauthorized reads of present/absent ids through every discovery/inventory/preparation/advanced route; subject reader tries definition enumeration; ordinary observer reads actors. | Indistinguishable typed unauthorized refusal, no leaked ids; no scope widening; actors explicitly withheld without hiding incomplete lifecycle state. |
| Awaiting proposal | Advance a valid proposal node through the owning workflow adapter. | Named awaiting outcome with operation/input type; no unsupported-node error or host status field. |
| Cost / rebuild | Increase unrelated workflow population; run exact lookup, bounded pages and an ordinary request; discard/rebuild indexes. | Hot work is local to selected lookup/page; no all-index population work on ordinary paths. Cold reconstruction is explicit and budgeted; rebuilt results are identical. |
| Owning kernel commands | Advance delegation/revocation through conflicts, changed admission, ambiguous commit, deferred settlement and terminal cleanup. | Query retains intent; fresh permission is required; exact recovery avoids duplicate effects; cleanup progresses independently; no caller re-issue loop. |
| Mechanical misuse | Omit guarded evidence/policies, construct/serialize a live ref, supply an actor or convert observation into authority. | Authoring/compile/facade checks reject these paths. The supported mixed consumer compiles without manual escape hatches. |

Mutation probes must expose the critical lies: removing positive actor presence,
selecting another occurrence, attributing a settler, ignoring the proposal principal,
bypassing disclosure, reading head for a retained basis, mutating warm progress
during a historical read, skipping selected-index currency or replaying a command
without current admission. Reuse tests for the same boundary rather than requiring
an exhaustive mutation matrix or repeated full-suite runs per edit.

## Ordered Delivery Phases

### Phase 1: Current Actor Contract And Explicit Policy

Land explicit guarded requirements and workflow policies, required canonical roles,
principal-bound proposal intent and the positive capability requirement through
declaration, installation, canonical encoding and native authorization. Update all
affected publication/read/settlement paths together with version checks and remove
the replaced compatibility code.

Prove current-format restore, intact predecessor refusal, required roles, deferred
attribution, exact actor policy and cross-principal replay. Freeze the shared
policy/actor/discovery-intent contracts before downstream implementation fans out.
This phase must not ship optional actors or an unenforced policy as a foundation.

### Phase 2: Governed Discovery And Reacquisition

Move definition discovery and all inventory routes behind principal-bearing
disclosure. Ship typed exact start-intent discovery, current indexed instance
lookup and bounded authorized enumeration. Return live locators or historical
results, including adoption/migration/fork semantics. Remove ungated bypasses.

Prove cross-request and current-checkpoint reacquisition, no existence leak,
scope discipline, fork isolation and selected-index locality/rebuildability.
Discovery can now supply lawful refs without any host registry.

### Phase 3: Read-Only Observation And Live Marking

Add the isolated read purpose, exact-basis state and paged history, audit withholding,
proposal-ref reacquisition and the typed awaiting-proposal outcome. Register
observation dependencies through the existing envelope subscription.

Prove read/admission separation, bounded reconstruction, basis-bound pages,
fresh disclosure on continuation and unchanged warm progress under old-basis/fork
reads. Use Phase 1's canonical actor reader; do not invent an audit-only source.

### Phase 4: Owning Commands, Consumer Deletion And Closure

Ship mandatory delegation/revocation owning adapters and delete Bank's manual
re-issue/replay plumbing. Complete the nine hardening obligations below, consume
the certified 9.17.6.4 workflow adapter and exercise the joined mixed consumer.
Remove closed known gaps and displaced API/docs/tooling forms.

Run the affected Query gate suites and repository-required boundary, facade,
agent-context and doc checks on the joined result. Correctness and structural
review must agree with the public proof. No legacy fallback or undocumented
alternate ordinary path is an acceptable stopping point.

Independent disclosure/index preparation and doc tooling can proceed after shared
contracts freeze; command adapters can proceed once 9.17.6.4's owner is certified.
Parallel workers must own disjoint responsibilities and integrate at these seams.

## Consumer Guidance Hardening: Nine Required Deliverables

1. **Explicit requirements.** Omission on a guarded binding is an authoring error.
   Occurrence-only and approval-requiring forms are explicit current encodings.
   Update macros, portable packages, examples and installed identities together.
2. **One owning command style.** Delegation/revocation use the required adapters,
   including Bank adoption and deletion of ordinary manual replay handling.
   There is no documentation-only alternative.
3. **Compiled mixed reference workflow.** Add a CI-run neutral example under
   `worth-query-certification/examples/`: occurrence-only data capture, a
   condition-gated approval-requiring operation, proposer/approver separation,
   `AwaitInbound` completion, later-request discovery, audit observation and the
   full owning caller loop including `AwaitingProposal`. Use the real inbound
   boundary; do not invent a fake remote service. Guide excerpts are marker-checked.
4. **Generated doc facts.** Render workflow node kinds, progress outcomes,
   validation denials and mutation outcomes from their source owners through the
   existing doc tooling; drift fails the check.
5. **Checked absence claims.** Consumer claims that a capability does not exist
   require a known-gap or facade-absence marker that the checker verifies.
6. **One prohibition list.** The capability index owns consumer prohibitions.
   Build guide, front door, host README and How WORTH Works link to it.
7. **Facade rustdoc routes.** `worth-query-decl` and `worth-query-host` crate
   docs link the build guide and capability index.
8. **One skill source.** `skills/` is authoritative; `.grok/skills` is generated
   from it and checked for drift, with no independently edited copy.
9. **Zero consumer fence debt.** Every document configured by
   `tools/boundary-check/config/consumer-docs.toml` has compiled, marker-checked
   or explicitly illustrative code under the existing policy. No silent debt
   remains in configured root docs, foundations, AI README or audience READMEs.

## Documentation And Closure

Deliver documentation with the capability it describes, in the existing documents.
Consumer guides describe the current lawful path and link to this milestone for
engineering rationale; they do not teach compatibility branches or private layout.

| Document / owner | Required deliverable |
| --- | --- |
| `docs/capability-index.md` | Discovery, observation, audit, actor policy and owning command task rows; canonical prohibition list; delete closed gap rows. |
| Query `docs/foundations/workflows.md` | Explicit policies, required roles, discovery intent, live/historical refs, exact observation, audit withholding, fork/migration and read-versus-authority contract. |
| `docs/build-an-application.md` | Compiled mixed journey, reacquisition after restore, complete owning workflow/command loops and named awaiting outcomes. |
| `docs/how-it-works.md` | One kernel truth, positive recorded-actor requirement in existing authorization, exact observation and no second owner. |
| `docs/glossary.md` | Observation, discovery intent, live locator, historical result and distinct admitted actor roles. |
| Query `docs/api.md`, host/decl READMEs and rustdoc | Audience-correct new entrances; no displaced public signatures or alternative ordinary retry recipe. |
| Existing doc/boundary tooling and skills | Generated facts, checked absence, marker-linked excerpts, one prohibition source, derived skill check and zero configured fence debt. |
| Roadmap | This destination, current-only cutover, dependencies and evidence remain consistent with the milestone. |

Closure requires every product decision, courtroom boundary and all nine hardening
items above, with the affected production, examples, macro/package and checkpoint
surfaces on one current contract. Accepted earlier evidence is reused unless its
boundary changed; verify coherent slices, not every small edit. No test-only
authority, consumer status/actor store or manual identity plumbing is allowed.

After public certification, proprietary M2.7.1 can pin the certified public
revision and consume these contracts. This milestone does not certify that
proprietary implementation or silently move private scope into the public plan.
