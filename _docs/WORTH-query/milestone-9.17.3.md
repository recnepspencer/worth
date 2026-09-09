# Milestone 9.17.3: Query Product-Branch Carriage, Facade, And Certification

> **Product posture:** This milestone completes the public in-memory Query
> product-branch workflow. Persistence and restart recovery remain Worth Store
> integration work.

> **Implementation status:** Phase 1 is certified. The sealed Signal owner,
> shared Query/Bridge/World root, exact source binding, bounded evaluation
> admission, and complete conditional service table are installed. Later phases
> have not started. The architecture and acceptance requirements below remain
> binding.

## Goal And Roadmap Placement

Carry the exact Runtime World product-branch basis and performed composite
publication from Milestone 9.17.2 through every Query phase that consumes or
projects product currentness. Publish one coherent Query host facade for branch
creation, selection, mutation, reads, history, live observation, inspection,
aftermath, and existing-outbox eligibility. Delete Relational-only product
identity and ambient Signal selection.

Milestones 9.17.1 through 9.17.1.2 own the exact component bases, owner
services, and branch-local progress contracts. Milestone 9.17.2 owns composite
history and product-head movement in `worth-runtime-world`. Query owns admission,
phase carriage, public DX, and projection; it cannot restamp lower authority.
Closing this milestone closes the 9.17 umbrella and unlocks tree-based semantic
undo/redo in Milestone 9.18.

## Central Claim

Every public Query operation on a product branch resolves one exact
Runtime World-admitted composite basis before planning, carries it without
rediscovery through execution, and constructs committed Query outcomes only
from performed Runtime World composite publication. One-shot, history, live,
inspection, aftermath, and
outbox eligibility all report the same product branch, composite commit, and
owner component bases.

The claim is false if:

- Query derives product identity from a Relational branch or commit;
- Signal basis is selected from ambient current state;
- a later phase re-resolves latest instead of consuming carried proof;
- a plan, session, proposal, invariant result, effect, or terminal from another
  composite basis can be paired successfully;
- owner-local performed work becomes a committed Query outcome before Runtime
  World
  product publication;
- history or live views infer product currentness from component history;
- an outbox fact becomes dispatch-eligible when its exact composite publication
  failed or lost the product-head race;
- Query owns another branch registry or calls private owner mechanics; or
- a physical runtime, recovery barrier, or persistent dispatch coordinator is
  introduced in this milestone.

## Ownership Lock

| Responsibility | Owner |
| --- | --- |
| Component truth, bases, branches, and owner-local publication | Relational and Signal |
| Installed Relational-to-Signal semantic correspondence | Base Runtime Bridge |
| Composite commits, product branches, currentness, and coordinated publication | `worth-runtime-world` composition owner |
| Product-branch intent, admission, carried execution affinity, public outcomes, and Query projections | Query |
| Existing outbox payload, correlation, and idempotency identity | Query aftermath meaning co-committed in the Relational component |
| Eligibility to attempt an external effect in the live runtime | Query after consuming exact performed composite publication |
| External exactly-once consequence | External owner through stable idempotency behavior |
| Persistence, restart-safe discovery, durable claims, recovery, and physical reads | Worth Store integration |

`worth-query-host` remains an audience facade. It reexports public Query
capabilities but implements no product history, branch selection, publication,
or dispatch policy.

## Current Boundary

After 9.17.2, the Runtime World facade can resolve a product branch to an exact composite basis,
coordinate owner work, publish a new composite commit, and issue performed
authority. Query still has branch-bearing surfaces that may use Relational
identity, ambient Signal context, or weaker descriptors. The migration must
cover the complete causal chain:

Query also still owns its unit-typed Signal runtime inside
`BridgeOwnedSignalRuntime`, whose ordinary paths use construction-only Signal
graph access. That root cannot simply issue 9.17.1.2 services and continue on
the old lane. This milestone must refactor or replace that composition root
once so Runtime Bridge and Runtime World operate against one owner-compatible
sealed Signal state. The three frozen branch ports do not expose Bridge's
conditional graph execution. Conditional execution remains required: Phase 1
must supply a concrete Signal-owned conditional execution service on the same
sealed owner state before carriage spreads to other consumers. A consumer
adapter, second Signal graph, or simultaneous legacy graph access after sealing
is forbidden.

```text
declaration -> normalization -> admission -> planning -> access plan
    -> provider session -> observations/read set -> proposal -> invariants
    -> effect lowering -> Runtime World publication -> terminal -> public projections
```

No post-terminal surface may retain the old identity lane after cutover.

## Adversarial Courtroom

The cumulative Supply Chain world creates two product branches sharing one
Signal basis and diverging in Relational history. It executes Relational-only,
Signal-only, and combined operations while:

- racing the same product head;
- blocking an unrelated branch writer;
- making each carried binding axis stale between adjacent Query phases;
- substituting equal-version component and composite artifacts;
- cancelling before owner preparation, between owner calls, before Runtime World
  publication, and after performed publication;
- forcing partial owner work and product-unpublished-owner-effects posture;
- opening one-shot, live, history, inspection, and aftermath observations at
  adversarial times; and
- creating an outbox occurrence in an owner-local Relational result whose
  composite publication subsequently fails.

An independent product-world oracle must observe exact phase affinity, one
winner, no half-current projection, correct component reuse, independent branch
progress, and no dispatch eligibility for the failed composite publication.
Dropping a carriage field, resolving latest, promoting owner evidence, bypassing
Runtime World, deriving identity from Relational, or enabling outbox work from row
existence must turn the court red.

### Scenario contract

The following scenario families are the acceptance contract, not a required
number of test functions. Combine cases sharing setup and failure semantics;
retain separate observations for setup, action, outcome, and teardown.

### Causally installed Supply Chain world

Build through Query declaration and host entry surfaces with real application
schema, principal/capability admission, provider sessions, Relational services,
Bridge correspondence, Signal owner services, and Runtime World. The ordinary
caller imports only declaration and host facades. Lower-owner inspection may
serve as a certification sidecar; it cannot manufacture the journey's authority.

Use cargo quantities `grain = 2`, `steel = 5`, and a declared grain-only
projection `grain_load = 2 * grain`. A grain limit condition is independently
defined as `grain <= 6`. The oracle uses these arithmetic rules and an immutable
model of branch-to-occurrence selection; it never calls production lowering,
impact classification, history resolution, or affinity comparison to calculate
expected answers. Real owner-issued identities are recorded as opaque labels,
not guessed or recreated from model integers.

Create A and B by forking Relational and retaining the same exact Signal basis.
Exercise all four explicit retain/fork component combinations in the creation
family. Equal bases may identify distinct commit occurrences; neither the model
nor the production comparator may collapse those occurrences.

### Required hostile scenarios

| Scenario and hostile sequence | Required observations and defect exposed |
| --- | --- |
| **One sealed graph, real delivery.** Warm both branch projections and hold a pinned pre-edit A read. On A, commit grain 3, deliver its committed patch through installed Bridge correspondence, and execute the installed conditional through Signal's owner service. Read B, A, B in that order, then the pinned pre-edit A basis. Commit steel 9 as the nonmatching twin. | The alternating loads are 4, 6, 4; pinned A remains 4. Conditional and invalidation evidence binds each exact admitted source observation as well as the selected Signal basis and installation. Grain delivery contacts the matching semantic target; steel causes no grain-target invalidation. Actual values and receipts must come from the same sealed graph. This rejects a second graph, global Bridge baselines, Signal cache reuse across different source worlds, and manually dirtied-node reenactment. |
| **Repeatable selection versus changing currentness.** Open a read and a prepared mutation at A's C0; publish C1 before either continues. | The retained read returns C0 values and C0 identity; the mutation receives a typed stale outcome before effects. A fresh selection returns C1. No later read phase resolves latest. Component movement alone cannot relabel a product observation. |
| **Adjacent-phase substitution.** Obtain two genuine admitted operations and present B's session, observation/read set, proposal, invariant receipt, or terminal to A's next boundary wherever pairing is expressible. Repeat with equal version ordinals, another owner, another installation generation, and another attempt on the same head. Change one axis at a time. | Private minting and illegal phase transitions are compiler-denied. Dynamic mismatches return the precise affinity denial before the next provider/owner/transport call. The valid twin reaches that call. Copying a descriptor, string, digest, or equal-valued basis cannot pass. |
| **All component effect postures.** Perform a Relational-only operation, a Signal-only declared state/definition operation, and a combined operation. Include an operation retaining a non-current component basis. | Each successful operation installs one exact composite occurrence. The unchanged component basis is identical and receives zero mutation contacts. Combined work records the actual results from both owners. Observational computation does not invent a Signal publication. |
| **Same-head race with a surviving loser effect.** Prepare Relational-only and Signal-only work from the same C0. Park after the first owner has moved; allow the competing attempt to win the product CAS, then release the parked attempt. | Exactly one performed product publication; the losing moved owner returns ProductUnpublished with its real result and recovery obligations. No missing sibling is executed, no product retry occurs, and all current projections select the winner's tuple. Run the reverse owner ordering where it reaches a distinct failure boundary. |
| **Independent branch progress.** Park A inside a Signal owner call. Complete B's Relational-only operation before releasing A; separately test independent Signal branches. | B completes while the pause is held, using deterministic rendezvous rather than sleeps. Shared exact Signal basis may correctly serialize changing work on that component, but cannot impose a global Query/Bridge/World lock on unrelated work. |
| **Cancellation and unwind.** Cancel before preparation, after Relational movement, before product CAS, and after performed publication. Unwind inside Signal execution and again while explicitly continuing retained Relational settlement. | Pre-effect cancellation has no owner movement. Post-owner/pre-CAS cancellation retains exact effects. Post-CAS cancellation remains performed. Recovery unwind retains exact progress/results, cleanup is denied while settlement is owed, and retry settles only. Dropping the caller cannot fabricate NoEffect, consume the result twice, or lose the retained obligation. |
| **Outbox loser survives.** Co-commit an actual outbox row during the losing Relational leg of a combined operation. Prove the row exists through the exact owner result, then attempt initial dispatch and redispatch. Present a different winner's receipt and later publish a descendant containing the old row. | Transport contacts remain zero for the original losing occurrence in every case. The failure is composite eligibility, not a missing row, missing transport, or unrelated admission denial. A genuine performed twin with fresh effect authority dispatches the exact payload and stable idempotency identity. Row inheritance does not retroactively authorize the loser. |
| **Complete footprint and aftermath.** Request a Signal-only domain change with an external effect; exhaust Relational/outbox capacity before preparation. In the admitted twin, dispatch and record workflow-visible aftermath. | The lowered plan is combined before effects. Exhaustion moves neither component and calls no transport. The admitted operation co-commits the outbox with its domain work. Workflow-visible dispatch aftermath creates a fresh admitted composite occurrence, never a sideband Relational write or reuse of consumed publication authority. |
| **Projection fan-out under partial progress.** Open one-shot, retained historical, live, inspection, and aftermath observers before, during, and after a parked partial operation and successful product CAS. Consume the performed delivery once and derive their projections. | Every result reports one complete canonical tuple. Historical reads remain historical; current views never publish the partial tuple. Grain changes preserve the existing declared patch granule; steel produces no grain patch. Slow live consumers obey existing backpressure and cannot block authoritative commit through synchronous fan-out. |
| **Lifecycle and bounded retention.** Hold a read, live subscription, prepared attempt, recovery view, and unconsumed delivery while retiring the branch and closing the owner. Exhaust each relevant installed budget in turn, then release eligible resources. | New work is fenced; held evidence retains its documented truth and typed next actions. No live protection is pruned to admit work. Cleanup releases exact obligations, not unrelated component leases. Repeated create/drop cycles plateau at installed capacity. Empty recovery pages may have a continuation because vacancies consume the work bound. |
| **Re-entry and reinstatement.** Queue conditional/managed-clock/async work, replace the installation or select a different product occurrence, then deliver the delayed work. Reconstruct a portable Query description and try to reuse its old live authority. | Re-entry validates the exact retained binding and returns typed stale/readmission posture where required; it never selects ambient Signal or latest product state. Fresh reconstruction admission is required. Late installation uses the owner service and cannot reopen construction-only graph access. |

The sealed-graph scenario keeps the relevant evaluation slots retained with
capacity for all selected bindings. After warming B, the third B read in the
B/A/B sequence must hit its retained derived slot: one slot-reuse hit, zero
source data reloads and zero recomputation for that unchanged projection.
Required constant-time admission/currentness checks remain separately counted.
This convicts clear-on-switch implementations that return correct values by
reloading everything. A separate cold/miss twin verifies exact-source recompute
and typed capacity denial, so the warm case does not accidentally exercise a miss.

The outbox-loser scenario must settle the actual retained loser record and then
use World's permitted fresh-admission route to reference its settled Relational
basis in a later performed product publication. This is a new operation, not a
recovery method that adopts a successor. The original row must exist and initial
dispatch and redispatch must both remain ineligible before and after that fresh
publication. A new effect occurrence follows fresh admission and idempotency
rules; no test silently assigns it the loser's occurrence identity.

The service-authority compiler family denies obtaining the conditional service
from the ordinary three-port bundle, forging its issuer/evaluation binding, and
constructing a committed receipt or committed recovery binding from a retained
partial. Its runtime counterpart supplies a genuine lowering with Relational
dependencies and no admitted source: the exact missing-source denial occurs
before observation, compute, or invalidation. The valid admitted-source twin
executes. Independently opened equal-version snapshots are denied at composite
admission, rather than passing because their descriptor strings compare equal.

The installation scenario starts with A and B retaining definition generation
D0, installs a conditional on A through the public product path, and observes A
at D1 while B and pinned A remain D0. A wrong-generation lowering is denied.
Exercise a losing post-owner/pre-CAS installation too: its Signal definition
successor remains retained partial evidence, and no Query registry or B read
exposes it as installed product meaning. The independent-progress scenario must
enter both operations through shared host handles; a lower-owner-only race does
not satisfy it.

The ordinary court runs a bounded seeded sequence containing fork/select,
read, the three effect postures, retirement, and explicit cleanup, checking the
independent model after each step. CI fixes the seed and bounds; the scheduled
lane broadens seeds and populations. On failure, report the seed and minimal
failing operation prefix, not a persisted proof ledger. Targeted negative twins
and mutation probes are reserved for ambiguous high-risk observations such as
outbox eligibility and affinity omission. Tests cannot guarantee arbitrary code
is correct; closure requires these falsifiable claims and review of residual risk.

## Product Decision Lock

1. Query product-branch identity is the exact Runtime World product branch, not a
   Relational alias.
2. Query resolves one exact composite basis before planning and carries it
   through the entire operation.
3. Every phase consumes the exact private-minted predecessor and preserves all
   binding axes needed by later phases.
4. Same-looking plans, sessions, attempts, proposals, or receipts from different
   composite heads are non-substitutable.
5. Lower runtimes are reached only through their audience facades and exact
   Runtime World-owned progression.
6. A Query committed terminal consumes performed composite publication. Owner-
   local candidate or performed evidence is insufficient.
7. Product-unpublished permits settlement, cleanup, inspection, or a fresh
   Query-admitted operation referencing the record without deriving authority;
   every outcome keeps typed next actions.
8. One-shot, history, live, inspection, preview where supported, and aftermath
   project the same canonical product-world identity.
9. Existing Query live views retain their exact patch granularity. Product-
   branch carriage changes the basis attached to patches; it does not create a
   second streaming abstraction.
10. An outbox occurrence is eligible only through the performed composite
    publication of the same operation/publication attempt that co-committed it.
    Its Relational basis appearing in another performed publication, including
    fresh adoption after settlement, grants no eligibility to the original loser.
11. External-effect plans include the Relational outbox footprint before owner
    preparation; a Signal-only domain change with an external effect becomes a
    combined component operation.
12. Workflow-visible dispatch aftermath re-enters as a new ordinary composite
    operation; no sideband Relational mutation creates hidden product history.
13. Query does not persist dispatch attempts, leases, outcomes, or retry state
    here. Live-process dispatch retains existing bounded lifecycle and stable
    idempotency semantics only.
14. Public branch creation exposes explicit exact component retain/fork intent
    because only the caller can choose that product meaning.
15. Raw component ids, generic authority markers, internal Bridge or Runtime
    World handles, and compatibility aliases are absent from the ordinary
    public facade.
16. All application state remains memory-resident. A process loss loses the
    current world; no restart guarantee is implied.

## Architectural Destination And Authority

### One owner-compatible Signal root

Replace the ordinary raw-graph execution in
`crates/worth-runtime-bridge/src/conditional_execution/{contract,execution}.rs`.
Bridge keeps correspondence, lowering, provider interpretation, and its derived
observation baselines. Signal alone owns graph state, exact branch admission,
execution-cell synchronization, and conditional execution evidence. Query's
composition root retains the required owners and installs World once; no owner
is hidden in an optional legacy execution lane after cutover.

The new Signal-owned conditional service accepts an owner-issued selected Signal
basis, installed conditional contract, and an admitted evaluation binding for the
exact source observation used by the provider request. A Signal basis alone is
insufficient when product branches share it but select different Relational truth.
Bridge derives the source binding from its actual admitted source observation;
rendered snapshot strings cannot manufacture this admission. The shared boundary
vocabulary and concrete owner-sealed proof contracts carry this meaning without
making Signal import Query, Runtime World, or their product branch types. It performs
observation-session admission, conditional execution, and invalidation observation
as one named owner operation. Its result carries the strongest actual decision,
observation, invalidation, movement, and cancellation posture; a computed value
or diagnostic graph identifier is insufficient. Bridge consumes this evidence
without restamping it. No service exposes `&mut SignalGraph`, a general graph
callback, or a second branch registry to Bridge or Query.

Signal issues this conditional service separately from the frozen three-port
`SignalOwnerServicePorts` bundle. Issuance requires the exact aspect-lowering
owner already claimed by Bridge on that graph. Bridge retains the resulting
capability privately; Query and World receive no conditional-port accessor or
issuer capability. Possessing ordinary Signal basis/mutation/lifecycle ports
does not admit conditional execution or grant source-interpretation authority.
The capability remains weak with respect to the same Signal owner lifecycle.

When conditional execution runs inside an already admitted Signal owner
operation, including World's Signal transaction callback, it borrows that
operation's exact owner-issued execution scope. It validates selected-basis and
evaluation affinity without reacquiring the branch cell. Standalone observation
admits its scope once through Signal. Nested work cannot mint publication
authority, start another owner mutation, or expose graph access. The combined
operation court must exercise this nested service route and complete without
reentrant locking; a standalone-only conditional test is insufficient.

Bridge alone constructs the execution evaluation binding from its actual admitted
snapshot context, selected Signal observation, and installed lowering, using the
concrete Signal-issued admission capability. Source presence is a typed choice:
`NoRelationalSource` for a lowering whose declared dependency closure requires
none, or `AdmittedRelationalSource` carrying the exact admitted source evidence.
The latter is mandatory for Relational dependencies. Missing-source execution is
a typed pre-execution denial, never `None`, a fabricated default snapshot, or a
shared source-less cache slot. Public constructors accepting arbitrary source
identities, and Query access to the binding mint, are forbidden. Shared substrate
vocabulary carries the boundary contract without a Signal dependency on Bridge.

Freeze the service's operation table in Phase 1: conditional execution, committed
patch delivery, retained-decision re-entry, owned async sources, managed-clock
wake execution, installation extension, and successor reconstitution. Initial
graph construction remains pre-seal. Runtime-supported installation extensions
use an explicit Signal-owned installation operation with versioned installed
evidence; they do not reopen `graph_mut()` or silently discard existing support.
Post-seal conditional installation is an authoritative Signal definition change,
not derived evaluation. It creates a Signal definition/basis successor through
World publication, with the corresponding versioned Bridge installation evidence.
The selected product branch sees the extension only after Performed. Other
products retaining the old Signal basis keep its old definitions and installed
meaning; sharing that basis is not consent to a global installation update.
Each retain/fork posture preserves the selected definition generation unless an
explicit admitted definition-changing operation advances it. An incompatible
lowering is denied before effects; explicit fork intent provides independent
future mutation when the shared component's current reference has advanced.

On failed composite publication, any performed definition change remains a
retained owner effect, not an installed Query product generation. Existing
observations and delayed work retain their original generation; reconstitution
requires explicit readmission and cannot publish it by updating a registry.
Pure derived evaluation of already-installed definitions creates no owner commit.
Old-generation lowerings and delayed work require typed re-entry/readmission.

Bridge baselines, retained conditional observations, and Signal derived-result
reuse are governed by the same exact source-observation, selected Signal basis,
and installation evaluation affinity. Node, record, or Signal-branch identity
alone is not an equivalence key. Signal owns its evaluation-slot/cache isolation
and invalidation; Bridge owns its source interpretation and retained baselines.
The returned evidence carries that evaluation binding so a consumer cannot pair
the right value with another source world. Switching between two live evaluation
bindings must preserve each one's truth and use bounded owner-managed retention,
not global cache clearing or a whole-graph copy per request. These derived slots
do not mint authoritative Signal commits. Reuse requires declared exact binding
equivalence and dependency validity; an old pinned source remains readable after
another source world advances.
Mutable Bridge bookkeeping may not remain borrowed across unrelated owner work.
The service uses Signal's existing branch execution cells and lifecycle gates;
adding a whole-owner mutex would defeat the independence claim. Every required
operation above must have a legal service route before Phase 1 closes.

### Independently callable Query operation paths

The host product-branch path through Query execution and Bridge conditional
execution must be callable through shared `&self` capabilities. A parked owner
call cannot retain `&mut WorthQueryRuntime`, `&mut BridgeOwnedSignalRuntime`, or
a mutex covering either whole root. Mutable provider/compute context belongs to
the individual operation; ownership and phase progression isolate that state.
This is a required Phase 1 root cutover, not a concurrency test below the public
root. It does not require changing unrelated APIs that never enter this path.

The operation path may read immutable admitted snapshots of domain/schema,
operation/provider, conditional-lowering, and installed routing registries.
Installation extensions construct a new version and publish its admitted binding
through the definition/publication lifecycle, rather than holding registry writes
over execution. Root session labels are descriptive membership, never another
product registry. Managed sessions, attempts, Bridge baselines, subscriptions,
and projection state use their existing semantic owners with per-operation,
per-evaluation-binding, or per-artifact synchronization. Registry lookup and
admission release their locks after acquiring the required managed lease and
before any provider, component-owner, or transport call.

Effect/consumer indexes derive the existing delivery routing and stay outside
the authoritative product CAS. Queued live delivery uses bounded handles;
diagnostics use sidecars; counters use operation-local or bounded owner updates.
None requires a borrowed root map across a parked operation. Construction must
assemble these capabilities exhaustively, so adding the World or evaluation
owner cannot leave an alternative exclusively borrowed execution root in use.

### Proof placement and carriage

Apply the repository's three placement questions before declaring a new type:
legality/progression vocabulary belongs in `worth-proof`; portable boundary
meaning belongs in `worth-foundational`; live retention, counters, clocks, and
Drop obligations belong to the owning runtime. Reuse the existing proof binding
and progression contracts rather than hand-rolling an authority substrate.
`Binding::ensure_matches` is comparison, not a transferable authorization grant.
Governed methods accept concrete owner-sealed proof specializations, never a
caller-selected `AuthorityMarker` or unconstrained generic capability.

Query execution already owns move-only `WorthQueryProviderSessionAffinity` in
`domain_computation/provider_session/protocol/session_affinity.rs`. Extend that
chain with the live World observation; do not install another session protocol.
The admitted operation holds existing Query authorization plus exact World
selection. Query owns the legal composition of these inputs, never World
currentness itself. The lease and its proof move together through phases.
Cloning a read observation is permitted only for a named managed observer;
publication and dispatch consumption remain linear.

| Product | Constructor and proof | Consumer and authority limit |
| --- | --- | --- |
| Normalized branch intent | Query declaration normalization; explicit selected branch and retain/fork meaning | Admission input only; no live owner authority |
| Resolved product observation | World observation facade; exact owner, branch incarnation, reference generation, commit occurrence, admitted Relational/Signal/Bridge bases and retention | Query admission and pinned reads; cannot publish by itself |
| Query-admitted composite operation | Query execution composes the World observation with existing admitted principal, capability, purpose, disclosure, schema, provider, and operation evidence using concrete proof contracts | Planning and one operation's phase chain; no widening of existing admission |
| Provider session through invariant result | Existing protocol consumes its exact predecessor, retaining run/plan/session/attempt affinity and actual read/invariant evidence | Next phase only; a matching string or copied report cannot replace a predecessor |
| Committed Query artifact | Terminal owner consumes the linear World Performed delivery and the matching operation state | Immutable receipt/history/live/aftermath projections; no second product commit |
| Composite dispatch admission | Query consumes/binds exact performed publication, original outbox occurrence and existing fresh external-effect authority | One bounded live dispatch attempt under existing retry rules; no restart discovery or authorization of inherited loser rows |

`worth-query-execution` may depend on `worth-runtime-world` through its facade.
`worth-query-admission` remains upstream: it must not import execution to obtain
a live carrier. Existing policy admission supplies its sealed output; execution
performs the live composite binding before planning. `worth-query-publication`
consumes execution's committed product, and host reexports those capabilities
without behavior. Declaration and portable package encoding do not acquire live
World or component-owner dependencies. Add only causally required manifest and
boundary-rule changes; no cycle or facade alias may route around this direction.

The binding includes World owner, product branch/incarnation/generation/commit,
exact component admission identities, Query runtime and installation, schema,
operation/admission identity, provider generation, run, plan, session, and attempt.
These axes are declared once at their owning boundaries and carried, not copied
into parallel mutable records. Portable 9.16.2 descriptions and existing rendered
plan-contract identities remain descriptive. Decode/reconstruction crosses fresh
admission and never restores the live World observation lease.

Resolve World selection before opening the provider's Relational snapshot. Open
that snapshot from the observation's admitted exact Relational basis, then seal
it into the same Query composite admission and provider affinity. An operation
presenting an independently opened snapshot is denied at composite admission;
equal versions, equal rendered plan identities, and a later current snapshot
cannot repair its missing provenance. Existing preselected snapshot paths must
be cut over, not simply supplemented with a World descriptor.

Pinned reads may lawfully finish at an older admitted occurrence. A mutation's
expected currentness is checked at the required pre-effect/World comparison
boundaries. Retaining exact history is not itself a reason to deny a read or
refresh it to latest. Fresh security or provider admission remains required where
the existing trust/lifecycle contract demands it; carriage does not disable it.

### Publication, dispatch, and resource custody

Effect lowering fixes component posture and the complete synchronous footprint
before preparation. Include declared domain writes, idempotency and outbox rows,
invariants, and required retention. An external effect adds Relational work even
when the domain change affects only Signal. No outbox preparation or capacity
admission may first appear after a component has moved.

World remains the sole coordinator. Query supplies admitted plans and preserves
NoEffect, Performed, and ProductUnpublished in typed terminals. The terminal
owner consumes Performed exactly once into a canonical committed artifact;
downstream projections borrow/share that artifact under their existing bounded
lifecycles instead of independently consuming or reconstructing the witness.
Post-Performed cancellation, projection failure, or caller loss cannot undo the
committed fact. Projection repair consumes retained committed evidence and never
re-executes the application mutation.

The committed Query receipt constructor consumes only the matching Performed
composite delivery. ProductUnpublished is a different terminal carrying World's
own retained-effects handle and exact partial facts, exposing inspection,
settlement, and eligible cleanup. It is never a committed receipt plus a flag
and cannot enter committed aftermath or redispatch constructors. A fresh
Query-admitted operation may separately reference settled evidence where World
permits it; the retained terminal itself grants no adoption or retry authority.

The committed artifact retains the owner-issued composite commit occurrence,
World publication attempt identity, and exact Query operation/outbox occurrence
binding. The attempt axis is required in committed recovery bindings and is
consumed by composite dispatch admission on initial dispatch and redispatch.
Carry it through preparation, owner effects, Performed, and lost-delivery
recovery; if an owner facade lacks this projection, extend that canonical
artifact rather than reconstructing it from a digest or caller correlation.
These are live-world occurrence identities, not durable restart capabilities.
9.18 obtains source occurrence/causality from retained canonical evidence and
World history, without introducing a Query history store.

Dispatch binds the original operation/outbox occurrence, its containing exact
Relational basis, and that operation's performed composite publication. Payload,
correlation, and stable idempotency identity retain their existing meaning.
Both initial dispatch and redispatch enter the same eligibility boundary. A
surviving owner-local row, matching payload, another winner's receipt, or a later
performed descendant containing that row supplies no missing authority for the
original loser. Losing publication is not an invitation to scan history for a
different qualifying commit. Fresh application intent may declare a new effect
under the existing idempotency contract, without relabeling the old occurrence.

This intentionally allows a fresh admitted operation to make settled domain
writes product-visible while the original losing operation's external effect
remains ineligible. A new effect requires its own fresh admission and occurrence
under the existing idempotency semantics. Adoption does not rewrite causality or
retroactively turn a ProductUnpublished terminal into Performed.

Transport acceptance/completion remains external truth. The installed transport
port is the sole external effect boundary; no network call runs under an owner
or product-reference lock. Workflow-visible dispatch aftermath is a new ordinary
Query-admitted World operation with explicit causality. It cannot mutate a
Relational row behind product history. No persistent dispatcher, lease journal,
restart barrier, or exactly-once external-completion promise is added.

Every live selection, session, queued continuation, subscription, dispatch
attempt, retained partial, and delivery has a framework owner, installed bound,
and Drop/close disposition. Query admission accounts for its own incremental
resources before effects and reuses World retention rather than shadowing it.
Releasing a caller must preserve any owner-performed obligations. Recovery
settles or cleans up only; it never completes a missing sibling or adopts a
product-unpublished successor. Generated inspection rows remain non-authorizing.

### Transition surface

```text
ProductBranchWorkflowIntent
    -> NormalizedProductBranchIntent
    -> RuntimeWorldResolvedProductBranchBasis
    -> QueryAdmittedCompositeBasis
    -> ProductBranchExecutionPlan
    -> ProductBranchProviderSession
    -> ProductBranchProposal
    -> ProductBranchInvariantResult
    -> RuntimeWorldPublicationOutcome
         PerformedCompositePublication -> QueryCommittedProductBranchTerminal
             -> receipt / history / live / aftermath / dispatch eligibility
         NoEffect / ProductUnpublished -> typed Query terminal
```

Compiler evidence denies raw composite-basis construction, unresolved planning,
unbound phase pairing, phase skipping, duplicate performed-witness use, direct
owner publication, and committed projection from a weaker artifact. Dynamic
heads share Rust types: exact cross-head/owner/attempt mismatches also require
typed runtime checks at the earliest pairing boundary. A compile-fail test must
not pretend that Rust statically distinguishes arbitrary runtime identities.

## Destination Topology

```text
E = existing, C = create, R = replace/cut over, S = committed successor only

crates/worth-signal/src/branch/owner_services/
    conditional_execution/                  C: concrete owner service family
        port.rs                             exact selected-basis entry
        issuance.rs                         claimed lowering-owner-only capability
        request.rs                          installed operation inputs
        evaluation_binding.rs               exact admitted source/installation affinity
        completion.rs                       strongest owner-issued evidence
    owner/conditional_execution/            C: same-cell implementation
        execution.rs                        observation/execution completion
        installation.rs                     lawful installed-contract extension
        evaluation_reuse.rs                 source-bound derived reuse and retention
    service_ports.rs                        E: unchanged three-port World audience

crates/worth-runtime-bridge/src/conditional_execution/
    contract.rs                             R: sealed services, no raw graph lane
    execution.rs                            R: interpret and consume owner evidence
    service_binding.rs                      C: private issuer/conditional-port custody
    observation_baseline/                   C: exact branch/basis/installation key
        binding.rs
        retention.rs
    owned_installation.rs                   R: lower to owner installation operation
    owned_async.rs                          R: preserve selected-basis re-entry
    managed_time/                           E/R: exact wake/lifecycle bindings
    successor_reconstitution.rs             R: explicit readmission lifecycle

workspaces/worth-query/crates/worth-query-declaration/src/branch/
    intent.rs                               C: pure product workflow meaning
    selection.rs                            C: explicit selection intent
    creation.rs                             C: explicit retain/fork posture

workspaces/worth-query/crates/worth-query-execution/src/
    basis/product_branch/                   C: live Query/World composition
        admission.rs                        compose existing Query and World proof
        observation.rs                      retained selection and release
        readmission.rs                      explicit trust/lifecycle crossing
        denial.rs                           typed failed binding/selection
    domain_computation/
        execution_runtime/product_world/    C: owner lifetime and required assembly
            installation.rs
            lifecycle.rs
            operation_ports.rs              independently callable admitted operations
        provider_session/protocol/
            session_affinity.rs             E/R: one inseparable live affinity
            plan_contract.rs                E/R: carry proof, descriptors stay descriptive
        provider_session/provisional_attempt/
            proposal_basis.rs               E/R: exact proposal/session binding
            proposed_state.rs               E/R: actual provisional state evidence
            invariant_execution/            E/R: preserve strongest invariant result
            commit_progression.rs           E/R: lower into World progression
        primary_graph/application_query/
            basis/                          E/R: current/pinned/history/preview cutover
            one_shot.rs                     E/R: exact product observation
            live.rs                         E/R: existing subscription protocol
        primary_graph/application_attempt/
            provider_execution/             E/R: retire direct owner commit/dispatch gate
            compare_and_commit/
                commit_receipt/             E/R: canonical committed product artifact
                commit_outcome.rs           E/R: all World terminal postures
        application_aftermath/
            performed_product_publication.rs C: bind committed operation occurrence
            composite_dispatch_admission.rs C: original and retry eligibility
            reversal/                       S: 9.18 fresh reversal progression
            reapplication/                  S: 9.18 fresh reapplication progression
            correction_history/             S: 9.18 source causality and alternatives

workspaces/worth-query/crates/worth-query/src/runtime/
    runtime_root_state.rs                   E/R: required one-world composition
    product_branch/                         C: ordinary caller orchestration
        selection.rs
        creation.rs
        operation_ports.rs                  shared host access, no root-wide borrow
    installed_live_routing.rs               E/R: existing granules, composite affinity

workspaces/worth-query/crates/worth-query-publication/src/runtime_world/
    receipt.rs                              C: committed artifact projection
    aftermath.rs                            C: typed product-world aftermath view
    live_delivery.rs                        C: identity on existing patch protocol

workspaces/worth-query/crates/worth-query-host/src/facade.rs E: reexports only
workspaces/worth-query/crates/worth-query-decl/src/facade.rs E: reexports only

workspaces/worth-query/crates/worth-query-certification/
    tests/runtime_world_branching.rs        C: one intentional integration target
    tests/runtime_world_branching/
        world.rs                            C: production-valid construction/teardown
        oracle.rs                           C: independent arithmetic/occurrence model
        sealed_signal.rs                    C: delivery/conditional causality
        phase_affinity.rs                   C: genuine substitution families
        publication.rs                      C: component postures and terminal fan-out
        concurrency.rs                      C: deterministic races/independence
        outbox_eligibility.rs                C: loser row, descendant, retry
        lifecycle.rs                        C: loss/capacity/close/continuations
        projections.rs                      C: exact identity and patch precision
        model.rs                            C: bounded seeded action sequences
        cost.rs                             C: independent scale axes
```

Existing package decomposition follows the Milestone 9.13.2 authority graph.
Forbidden placement includes a Query branch registry, generic helper/bucket
modules, direct component storage access, physical-runtime composition,
persistent recovery modules, adapter-owned dispatch admission, or facade files
implementing behavior.

The tree strengthens existing protocol and primary-graph owners; it does not
create a parallel generic planning/proposal/invariant pipeline. New leaf names
may be refined for the actual subordinate operation, but their authority and
parent axes are fixed. Any required move is part of the owning cutover, with the
old implementation deleted rather than retained as an alias.

Signal's service family owns execution mechanics, Bridge owns semantic
interpretation and derived baselines, execution owns live Query progression,
and publication owns derived audience views. Public entry remains the existing
audience facade route. Private visibility and explicit facade exports enforce
these boundaries; generated context and dependency checks enforce crate direction.
No code/test/support file gains an implicit exemption from the 400-line cap.

Committed growth is additive. 9.18 adds installed correction meaning under
installation, fresh correction admission under admission, and correction
execution under the existing aftermath owner. Its exact source operation,
source commit, target head, and per-component posture feed this same World
publication/terminal path. It does not relocate the session protocol, public
facade, dispatch gate, or history authority. Store integration later consumes
versioned descriptive artifacts at its own persistence boundary, never serialized
live leases. These successor locations do not require empty files now. No
multi-parent DAG, generic plug-in registry, universal phase framework, or storage
adapter is built speculatively in this milestone.

## Ordered Phase Plan

### Phase 1: Composite Basis And Carriage Inventory

Inventory every branch-bearing Query type, constructor, transition, facade,
lower-runtime request, receipt, history/live/preview/inspection surface, and
fixture. Install the private admitted composite-basis carrier and compiler
denials before broad migration. Freeze the one-graph
`BridgeOwnedSignalRuntime` cutover seam and its exact service ownership before
parallel carriage work begins.

The inventory must trace producers and consumers in both `worth-query`'s
runtime root and `worth-query-execution`'s primary-graph/protocol roots, including
conditional, async, managed-clock, portable reconstruction, and retry paths.
Record the replacement owner and deletion condition for each old authority path
in the implementation plan; do not add a permanent machine-readable ledger.

Build the Signal-owned service, partition Bridge baseline state, and complete
Signal derived-reuse affinity and required root/service assembly in this phase.
Demonstrate one real branch-bound
read and Bridge-delivered conditional operation through the sealed composition
root. The evidence must include actual conditional/invalidation receipts and
independent branch progress. A type skeleton, graph-id equality, or facade that
still delegates to `graph_mut()` is not completion. Installation/reconstitution
and every operation in the frozen service table must remain expressible without
a future graph escape hatch.

Freeze shared host/Query/Bridge operation entry, the lowering-owner-only service
issuer, source-present/source-free admission variants, definition-generation
publication, and evaluation-slot budgets here. The real public-root progress
test, warm-slot reuse test, missing-source denial, and late-installation isolation
are foundation evidence; none may be deferred until the facade closure phase.

The next phase may trust one live selected World observation, one Signal owner,
and a fixed service ownership contract. Broad carriage migration must not begin
until that boundary and the proof/package placement pass review.

**Certification (2026-09-08): complete.** The public host journey selects and
reads a product branch, performs a World publication, delivers the committed
patch through Bridge, executes its conditional, proves sibling progress and a
retained historical read, and releases its resources. The same frozen root
expresses retained-decision re-entry, owned async supersession, managed-clock
wake, post-seal installation, and successor reconstitution without graph
escape. Exact-source denial, definition-generation isolation, bounded slot
admission/reuse, and terminal retry affinity have positive and hostile twins.
The retained B/A/B court reports one Signal-owner slot reuse, zero source
admissions, and zero recomputation on the third B execution. Its cold twin
performs one recomputation, then holds that live session while a second admission
receives the typed capacity denial. Hostile generation courts reject both a D0
lowering at D1 and a D1 lowering at retained D0, while the exact retained D0
sibling remains executable. Application publication and Query test installation
require their evaluation resources explicitly. The Phase 1 owner, certification,
and UI-binding suites pass together with the dirty Rust line-cap, boundary
topology, and generated-context gates.

### Phase 2: Planning Through Invariant Carriage

Carry the exact basis through normalization, admission, planning, access plans,
provider sessions, observations, read sets, proposals, and invariant execution.
Remove default-main, derived identity, ambient Signal, and current-head relookup.

Extend the existing session affinity and strengthen signatures at actual pairing
sites. Eliminate optional composite fields and constructors that accept only
rendered basis identities. Preserve 9.16 authorization, graph-read/touch closure,
provider generation, invariant execution, and portable fresh-readmission rules.
Use exact retained component reads; do not convert a repeatable read into a
current-head lookup. Each migrated consumer loses its old mint/lookup route in
the same slice.

The phase closes on the adjacent-phase substitution family, repeatable-read
versus stale-mutation family, and real provider/invariant tests with valid twins.
Counters show one product resolution and no downstream latest lookup or basis
rehash. Effect lowering may then trust the strongest exact invariant result.

### Phase 3: Effect, Publication, And Terminal Carriage

Lower Query effects into exact Runtime World component plans, consume only 9.17.2
publication progression, and construct committed terminals only from performed
composite transitions. Preserve every losing, no-effect,
product-unpublished-owner-effects, and cancellation outcome.

Replace direct Query orchestration of Relational commit/settlement with World
coordination; keep provider-local candidate preparation behind its lawful owner
surface. Carry cancellation and deadline controls rather than sampling a new
policy in each phase. Reserve the complete footprint, including outbox and
retention, before effects. Construct the canonical committed Query artifact at
the single terminal boundary and install the mandatory performed-publication
precondition for dispatch now; no intermediate revision may dispatch from the
old Relational-only gate while Phase 4 is pending.

Prove all component postures, same-head losers, unrelated branch progress,
pre-/post-effect cancellation, unwind custody, and single-consumption fan-out.
The next phase consumes committed or explicitly partial artifacts, never raw
owner results that it must reinterpret as product truth.

### Phase 4: Projection And Existing-Outbox Cutover

Cut receipts, history, live views, inspection, preview where supported,
aftermath, and live-process outbox eligibility to canonical composite identity.
Prove exact patch granularity remains unchanged and failed composite publication
cannot dispatch its owner-local outbox.

Migrate both initial dispatch and redispatch, retained recovery bindings,
workflow-visible dispatch aftermath, conditional continuations, and supported
preview/live paths. Preserve their existing admission, backpressure, disclosure,
and lifecycle contracts. No latest lookup or history scan may repair missing
carriage. Descriptive inspection may expose partial owner facts, but cannot
present them as current product state or operational permission.

Close on the surviving loser-row/descendant/winner-substitution family, complete
outbox footprint and fresh aftermath publication, projection fan-out, delayed
re-entry, and lifecycle capacity scenarios. The public facade may then expose
one coherent product workflow with no hidden weaker post-terminal consumer.

### Phase 5: Public Facade And Legacy Deletion

Publish product branch selection, explicit component reuse/fork creation,
mutation, reads, history, live, inspection, and aftermath through declaration
and host facades. Delete the Relational-only/ambient-Signal lane with its last
consumer; no compatibility authority remains.

Compile the caller DX below from declaration/host imports, including all four
creation postures, bounded history/recovery inspection, explicit close, and
typed outcome handling. Reuse existing surface owners and reexport namespaces;
host adds no registry, dispatch policy, or publication behavior. Required World
budgets, owner roots, clock, and lifecycle wiring are exhaustive construction
inputs, not optional defaults that retain the former root.

Audit the Phase 1 consumer inventory to zero surviving authority routes. Remove
obsolete constructors, tests, aliases, and docs with their last consumer. Raw
component ids may remain descriptive where honest, but cannot select a product,
admit a session, produce a committed terminal, or authorize dispatch. Compiler
denials and dependency/facade enforcement keep those routes closed. The next
phase validates the completed public product, not an alternative cert-only root.

### Phase 6: Documentation And Cumulative Certification

Compile the ordinary and advanced public examples, run the cumulative 9.17
court through the real Query composition root, and close facade, dependency,
line-cap, formatting, boundary, generated-context, counter, and residue proof.

Run the cumulative model and cost scenarios through the same public root as the
examples. Execute the affected existing authorization, provider, conditional,
projection, aftermath, and retry portfolios so 9.17.3 does not narrow previously
accepted behavior. Resolve material defects in the complete scoped change;
report untouched unrelated debt without absorbing it into this milestone.
No phase is complete merely because its new tests pass while an old ordinary
authority path remains usable. Review approves a coherent slice before the
next one begins; repeat only the evidence invalidated by subsequent changes.

## Caller DX Target

```rust
let branch = app
    .branches()
    .fork(app.current_world())
    .components(|components| {
        components
            .fork_relational()
            .reuse_exact_signal_basis()
    })
    .create()?;

let committed = app
    .on_branch(branch)
    .transaction()
    .apply(admitted_change)
    .commit()?
    .require_committed()?;

assert_eq!(committed.product_branch(), branch.id());
```

The caller chooses product intent and component retain/fork posture. Query
carries exact owner identities and the Runtime World owner orchestrates
publication; callers never wire component runtimes or manufacture bases.

This is the required API design target, not a claim that the methods exist
before implementation. The public example must also show explicit NoEffect,
ProductUnpublished, cancellation, and cleanup handling; `require_committed()`
is a convenience that preserves the typed noncommitted outcome, not an unwrap
that drops recovery custody. `current_world()` obtains a managed explicit
observation at the entry boundary, never ambient authority for later phases.
Advanced examples select exact history and component retain/fork posture through
Query types. They cannot require imports of internal owner services.

## Performance And Resource Contract

- Basis admission and carriage are O(1) in fixed component and binding axes.
- Later phases do not rediscover or rehash a basis already carried within the
  same trust boundary.
- A single-component operation mutates only the changing owner. Exact-basis
  observation and derived delivery/evaluation contacts are separately bounded
  from mutation, correspondence validation, and World publication work.
- One-shot, history, live, inspection, and aftermath never scan histories to
  infer currentness.
- Existing live patch computation remains bounded by semantic delta and the
  query's declared delivery granule.
- Diagnostic richness and certification identity remain sidecar/cold work.
- Counters distinguish basis resolution/readmission, phase carriage, freshness
  checks, owner contacts, composite publication, projections, outbox admission,
  fallback use, and retained resources.

### Measurable ordinary-path bounds

For an admitted ordinary attempt, the product-selection counter is exactly one;
downstream latest-product resolutions, basis reconstruction, history scans for
currentness, and fallback authority routes are zero. A pinned read consumes its
existing admitted observation with zero new latest resolutions. Freshness checks
have named trust/currentness boundaries and a count fixed by the chosen path,
not the number of retained branches, history entries, or subscribers. Do not
claim that World retention or hash lookup counters measure physical allocation
or hash collisions.

For one successful publication, the product CAS succeeds exactly once and the
carried reserved history entry is installed once. An unchanged component has
zero mutation contacts. The protocol may make additional named preparation,
settlement, or observation contacts on the changing owner; expose those
separately rather than calling the whole sequence one contact. A known pre-effect
denial has zero mutation and transport contacts. Every ineligible outbox path has
zero transport contacts. Projections add no product resolutions or commits.

Zero mutation contacts on a retained component does not mean zero derived
contacts. A Relational-only publication may deliver a committed patch into the
retained Signal cell and evaluate an installed projection. Count Bridge delivery,
Signal observation/evaluation, invalidation, source data loads, derived-slot
hits/misses, and authoritative component mutation separately. Derived delivery
does not advance the retained Signal basis or justify claiming Signal execution
where only direct Bridge truth was consumed.

Signal evaluation-slot count and retained-byte budgets are required, exhaustive
construction inputs alongside World and Query resource budgets. An idle derived
slot may be evicted within policy; an actively retained evaluation binding may
not be silently cleared or repurposed. A miss recomputes from the exact admitted
source and installed definition under the declared work budget, never latest.
Evaluation required by a synchronous mutation footprint reserves its slot/work
budget before authoritative effects; denial then prevents the mutation. Standalone
observation admits its evaluation budget before computing. Delivery or projection
after Performed instead retains the committed artifact and follows the existing
typed projection/backpressure/retry disposition on exhaustion. It cannot
reclassify commitment, rerun publication, or promise a pre-effect denial after
the effects already happened. The capacity court must exercise both the
synchronous pre-effect denial and post-Performed projection-exhaustion posture.
Expose slot admission,
release, eviction, hit/miss, source-load, and recomputation costs. Cold recompute
is an explicit cost posture, not permission to evict every slot on each switch.

Measure independent axes B (product branches), H (retained history), U (distinct
component pins), A (active attempts), P (retained partials), O (observations),
L (live consumers), and W (independent writers). Use lawful populations at
1/8/64 for B/H/U/A/P/O/L and 1/4/16 for W; report unavoidable covariation and
actual populated values. Hold the selected semantic edit fixed, then separately
vary touched records at 1/8/64 to distinguish legitimate delta cost. Logical
World charges and installed Query queue/resource limits are explicit fixture
inputs sized to admit the largest profile, with separate one-below/exact-bound
exhaustion cases. Never label a sparse or rejected setup a scale proof.

Carriage and currentness work remain fixed in population axes; patch work scales
with matching semantic delta and declared delivery granule. Authoritative commit
cannot scale with L: delivery uses the existing bounded consumer scheduling and
backpressure lane. Retained Query state is bounded by installed live resources,
not lifetime attempt count. Histories remain explicit bounded traversal; recovery
uses the World opaque position cursor and counts examined vacancies.

Scheduled timings report environment, cold/warm posture, three fresh repetitions,
variance and p50/p95/p99 without machine-specific pass thresholds. Structural
assertions are the ordinary regression gate; timings reveal uncounted work and
do not upgrade expected/amortized hashing to worst-case constant time. Retain
only named correctness/cost facts in the ordinary result; diagnostic-rich and
reconstructive work remain separate lanes.

## Documentation Deliverables

Paths below are relative to
`workspaces/worth-query/crates/worth-query/docs/` unless explicitly rooted.

| Continuing audience and authoritative document | Required change and check |
| --- | --- |
| AI agents and contributors: `AI_README.md` | Replace the Relational-only mutation spine and pending-9.17.3 statements only when cutover is real. Explain World selection, carried proof, canonical terminals, owner boundaries, and forbidden shortcuts. Check every authority claim against the facade and constructor inventory. |
| Application authors: `foundations/ordinary-application-front-door.md` | Teach the actual declaration/host construction, branch-bound read/mutation, explicit controls, all terminal cases, and cleanup. Extract or include snippets from the executable ordinary example. |
| Advanced branch users: `foundations/branches-and-previews.md` | Explain product versus component identity, all four retain/fork postures, exact history, retirement/name reuse, repeatable reads, and the supported preview boundary. Include migration away from Relational selection and ambient Signal rather than creating a competing milestone guide. |
| Live consumers: `runtime-surfaces/granular-live-invalidation.md` | Document canonical composite identity on existing patch granules, no partial-current delivery, delayed work, backpressure, and disposal. Validate against the real matching/nonmatching and slow-consumer court. |
| Effect and recovery integrators: `execution/application-aftermath-and-recovery.md` | Explain Performed versus ProductUnpublished, original-occurrence dispatch eligibility, retry, inherited loser rows, late cancellation, settlement-only recovery, and fresh composite aftermath writes. Link the executable failure/recovery example. |
| Observers/operators: `capabilities/inspection.md` and `capabilities/historical-diff-and-basis.md` | Explain exact selected versus current truth, bounded ancestry and recovery cursors, empty-page continuation, protected resources, and descriptive non-authority. Verify examples against the public inspection surface. |
| Lower-runtime contributors: root `crates/worth-runtime-bridge/API_OVERVIEW.md` and `REFERENCE_MAP.md`, plus affected Signal facade docs | Replace raw-graph conditional ownership with the sealed service operation/lifecycle contract and exact evidence returned. No doc may teach reopening graph access. |
| All consumers: docs `README.md`, Query workspace `README.md`, and public rustdoc | Point to the continuing guides and ordinary/advanced executable examples. State memory residency, process-loss behavior, and that restart durability belongs to Store. No generated `AGENT_CONTEXT.md` is hand-edited. |

Add ordinary and advanced examples under the Query certification package using
the real declaration/host audience. Register intentional executable targets in
Cargo so package/CI runs execute them; a snippet that only compiles as dead code
does not certify a lifecycle journey. Remove obsolete paths/snippets during their
own cutover and validate relative links. The milestone specification remains
implementation authority, not a replacement for these public guides.

## Verification And QA Closure

Use one intentional `runtime_world_branching` integration target for the
cumulative scenario modules, plus existing owner tests for local contracts and
one consolidated compiler family for public minting/linearity/phase guarantees.
Each compiler denial has a valid counterpart and fails at the intended public
boundary. A fake provider or transport proves behavior against that substitute;
the court must still cross the real Query/Bridge/Signal/World boundaries. A
recording transport is appropriate for eligibility and stable request identity,
but proves no external service completion or exactly-once behavior.

Ordinary CI runs bounded scenarios, deterministic race/cancellation controls,
compiler fixtures and examples. Feature-gated operation-control tests use the
same production transitions and cannot mint authority or bypass admission.
Scheduled CI runs the broader model and all scale profiles with explicit harness
bounds and retained failure output. Do not add a new integration binary per
scenario, process-wide allocation instrumentation to ordinary tests, test-proof
receipts, source fingerprints, or progressive verification ledgers.

Run affected package tests directly with the Query workspace manifest for
declaration, admission, execution, publication, host, and the main runtime;
compiling them as dependencies is insufficient. Run the certification target
and compiler/examples after the relevant facade cutover, and affected Signal,
Bridge, and World owner/feature suites for the new service boundary. Run portable
package/replay tests when their reconstruction carriage changes, in their cold
certification lane. Repair any inherited Signal batch-read defect if the new
ordinary Query service depends on that behavior; substituting a narrower test
path cannot certify an unchanged ordinary batch API.

Before boundary-relevant completion, run from the repository root:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
python scripts/quality/scrutinize_rust_functions.py --dirty .
```

Run formatting, scoped strict lint in affected configurations, and the affected
workspace/facade guards. Review function advisories for semantic cohesion rather
than splitting mechanically. Scope line-cap enforcement to the complete dirty
set, including staged and untracked files. Required checks may not be bypassed;
untouched dependency debt is reported distinctly from a scoped failure.

QA must specifically attack one-graph ownership, proof weakening at phase joins,
loser outbox eligibility, fan-out from one consumed Performed artifact, and
resource survival across unwind/close. A valid positive twin must reach the
disputed owner or transport boundary so unrelated rejection cannot produce green.
Review the actual index/lock/allocation paths in addition to counters. Completion
requires appropriate passing evidence and no known material defect in the causal
scope, not a promise that test enumeration proves all possible behavior.

## Must Preserve

- every 9.16 authorization, provider-session, invariant, aftermath, and
  publication guarantee;
- every 9.16.2 portable package and fresh-readmission guarantee;
- every 9.17.1 component-authority and independent-progress guarantee;
- every 9.17.1.1 and 9.17.1.2 owner-service, retention, lifecycle, and
  independent-branch progress guarantee;
- every 9.17.2 explicit-bootstrap, composite-history, retention, and
  no-half-publication guarantee;
- Query as audience/admission facade rather than history/currentness owner;
- exact existing outbox payload and idempotency identity;
- existing Query live-view patch granularity and backpressure policy;
- diagnostic noninterference and certification-only replay.

## Explicit Non-Goals

- persistence, PostgreSQL, physical runtime composition, restart recovery,
  durable dispatch claims/outcomes, paging, or database fetches;
- semantic undo/redo, merge, rebase, multi-parent history, or tags;
- offline synchronization or distributed publication; and
- a public lower-runtime orchestration API.

## Acceptance And Handoff

Milestone 9.17.3 closes when the real public Query composition root proves exact
composite carriage through every phase and projection; shared Signal basis with
divergent Relational histories; Relational-only, Signal-only, and combined
operations; independent progress; one-winner same-head races; typed partial and
cancellation outcomes; no half-current observation; existing-outbox eligibility
only after performed composite publication; unchanged live patch precision;
compiler-denied minting/phase skipping and runtime-denied dynamic cross-basis
pairing; executable docs;
exact counters; dependency/facade enforcement; and zero legacy identity lanes.

Milestone 9.18 receives exact Query-selected product branches and composite
heads, immutable single-parent composite commits, owner-observed component
bases, Runtime World coordinated publication, retention, public history/inspection/
aftermath, typed outcome carriage, and performed-publication-gated outbox
eligibility. It may add freshly admitted correction semantics but may not create
a second history owner or introduce persistence as part of undo/redo.
