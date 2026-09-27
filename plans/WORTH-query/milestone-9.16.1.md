# Milestone 9.16.1: Canonical Graph Obligation And Provider Session Convergence

> **Status:** Closed — Milestone 9.16 resumes at Runtime Hardening Phase 7.3
>
> **Historical posture:** Milestones 9.9, 9.10, 9.11, and 9.15 retain their
> recorded statuses. Milestones 9.9, 9.11, and 9.15 closure and Milestone
> 9.10's recorded Draft status are not reopened, revoked, or rewritten by this
> corrective milestone.
>
> **Closure ledger:**
> [milestone-9.16.1-closure-ledger.md](./milestone-9.16.1-closure-ledger.md)

## Goal

Establish one compiler-visible graph-obligation progression for every ordinary
Query read and mutation, then migrate each covered authority surface into that
progression without losing the behavior, lanes, outcomes, lifecycle, or cost
guarantees already implemented by the Query product.

Milestone 9.16.1 is a corrective interstitial milestone. It preserves the
useful meaning and lower-owner work produced by Milestones 9.9, 9.10, 9.11,
9.15, and 9.16 while supplying a stronger composition guarantee discovered
during Milestone 9.16 Runtime Phase 7.2. It does not revise the historical
closure of those milestones or phases.

The canonical progression is:

```text
typed query or operation declaration
    -> installed graph-obligation set
    -> touch- and intent-scoped obligation selection
    -> canonical graph-read requirements
    -> current support inventory and structural cost
    -> budget and live-capacity admission
    -> sealed branch-affine managed-run/provider-session plan
    -> exact lower-owner graph and policy work
    -> complete session-bound decision read-set
    -> read terminal
       OR proposed state -> installed invariants -> compare-and-commit
    -> terminal receipt and disclosure-governed publication
    -> exact resource release
```

Selection is not execution. Review is not authority. An envelope is not
authority. Application-query semantics may shape the work and results, but may
not create another graph-read plan. Every decision-bearing graph observation
occurs after the managed provider session begins.

## Roadmap Placement And Append-Only Rule

Milestone 9.16.1 consumes:

- the recorded guarantees and artifacts of Milestones 9.9, 9.10, 9.11, and
  9.15;
- Milestone 9.16 Runtime Phases 1-6;
- the installed capability meaning and lower-owner authorization work completed
  through Runtime Phase 7.2; and
- the currently useful uncommitted convergence work only where it conforms to
  this specification.

Milestone 9.16 pauses after Runtime Phase 7.2 while 9.16.1 is open. After
9.16.1 closes, Milestone 9.16 resumes at Runtime Phase 7.3. No earlier milestone
or closed phase changes status.

Future discoveries follow the same append-only rule:

1. A new guarantee receives a new phase or interstitial milestone.
2. A later discovery may supersede an implementation surface, but does not
   rewrite the historical status of the milestone that originally introduced
   it.
3. The new ledger records which prior artifacts remain valid inputs, which
   integrate unchanged, which exact semantic surfaces require migration, the
   parity required before cutover, and which authority-capable predecessors
   become removable only after that cutover.
4. Dependent unfinished work is blocked by the corrective milestone; completed
   rows are not changed to manufacture a different history.
5. A discovery whose required correction extends beyond the graph-obligation,
   graph-read-planning, or provider-session authority surfaces receives an
   explicit phase amendment or successor milestone before implementation; it
   is not implicit authority to restructure the rest of `worth-query`.

## Central Claim

For an installed application query or operation, there is exactly one legal
way to obtain executable graph authority and exactly one legal way to plan a
graph read. The compiler and package graph prevent a consumer from constructing
an admitted equivalent, bypassing the managed provider session, substituting a
parallel executor, or promoting inspection evidence into authority. This is an
authority-uniqueness claim, not a crate-deletion claim: the `worth-query`
monolith remains the product composition root and retains every feature whose
exact destination cutover has not yet been proved.

The claim is false if any public caller can:

- construct a requirement, inventory, budget result, admitted plan, provider
  call, decision read-set, execution row, or receipt that opens a later phase;
- invoke graph-read review as executable authority;
- execute an application query directly against raw Relational runtime state;
- report a selected invariant, validator, or policy obligation as executed
  without invoking its actual owner;
- begin authorization observation before the provider session exists;
- substitute a same-version snapshot, read set, proposal, or receipt from a
  different typed branch;
- enter proposed-state or invariant phases for a read-only query;
- invoke a manual invariant pack beside installed invariant progression;
- import an authority-capable predecessor alongside or through a destination
  package; or
- retain two public paths whose products are only superficially equivalent.

The claim does not forbid the monolith facade, orchestration, or feature
implementation from calling the destination authority through one-way
lowering. It forbids the caller-facing and internal product graph from having
two independently executable authorities for the same semantic decision.

## Current Boundary

### Preserve

The following work is directionally correct and remains the foundation:

- Relational owns exact graph truth, traversal, field and adjacency
  observations, negative facts, complete path witnesses, currentness, touched
  scope, and invariant mechanics.
- Query owns translation from Query declarations and installed authority into
  neutral Bridge requests. Runtime Bridge owns installed neutral correspondence,
  crossing admission, and lowering from Relational observations to Signal
  policy inputs and evidence.
- Signal owns installed policy evaluation, including required and prohibited
  composition and its typed evaluation evidence.
- Query installation owns typed application-query, operation, capability,
  decision-fact, resource, and provider contracts.
- Query admission owns real capacity reservation and denial before execution.
- Query execution owns managed runs, provider sessions, basis affinity,
  decision read-sets, proposed state, invariant progression, and terminal
  lifecycle.
- Phase 6 canonical identities and Phase 7.1 capability identities are derived
  only at installation or bounded fresh-admission seams and retained afterward.
- Phase 7.2 exact-grant witnesses, principal currentness, trusted time,
  request binding, and policy evidence remain required decision facts.

### Integrate or migrate prospectively

The following exact authority surfaces do not satisfy the new central claim.
Each is integrated when its owner and authority are already correct, or
migrated with explicit feature-and-evidence parity when authority currently
resides in the monolith:

- the monolith-owned graph-obligation authority tree;
- selection-backed executors that can mark an obligation executed without
  invoking its semantic owner;
- public graph-read planning inputs, proof-like requirement constructors,
  inventory construction, and plan review;
- the monolith admitted graph-read plan and executor;
- application-query-owned graph-read plan authority and direct Relational
  execution outside a managed provider session;
- authorization observation performed before the managed provider session;
- manual invariant-pack callbacks and no-op invariant composition paths;
- consumer-kit or reference-consumer proof that demonstrates local
  registration or execution rather than adoption of the public Query facade;
  and
- any export, alias, adapter, or wrapper that can independently mint or execute
  the superseded authority after its covered surface cuts over.

The implementation preserves the product behavior behind these surfaces. A
private algorithm, result shape, continuation, live lifecycle, diagnostic, or
consumer-facing facade remains where it is unless its own semantic ownership
requires migration. Re-exporting or wrapping old authority is not migration,
but one-way facade lowering into the sole destination authority is lawful and
is the preferred incremental integration seam.

### Authority uniqueness and parity-gated migration

The unit of migration is one named semantic authority surface, not a directory,
module family, crate, or historical milestone. Every migration row names:

- the predecessor guarantee and current production owner;
- the exact authority-capable entry and every real consumer lane;
- the destination proof type and owner;
- behavioral, denial, lifecycle, receipt, and warm-cost parity;
- the atomic cutover point; and
- the exact predecessor authority that becomes residue after cutover.

Before cutover, the predecessor remains the only executable authority. The
destination may be exercised as non-authoritative comparison evidence, but it
cannot execute effects or mint a terminal receipt. At cutover, all covered
consumers switch to the destination and the predecessor's ability to mint or
execute authority is sealed or removed in the same coherent slice. There is no
intermediate state in which both paths are legal authorities.

Parity is not representation equality. It proves the same supported feature
set, typed outcomes, consequential state, cancellation and cleanup, public
receipt meaning, and contractual work bounds through the real public facade.
If parity exposes missing destination behavior, the migration remains open and
the predecessor remains authoritative. The feature is not deleted to make the
new path appear complete.

Milestones 9.9, 9.10, and 9.11 are integration authorities for the semantics
they introduced, not blanket deletion instructions. Milestone 9.13.2's valid
package-boundary contract is likewise authority locality and one-way dependency
direction; its former whole-monolith deletion wording is not inherited by this
milestone. For each inherited surface, 9.16.1 must either consume its existing
proof and owner unchanged or migrate that exact surface with parity. Unrelated
monolith responsibilities are out of scope. A broader architectural defect is
recorded and scheduled explicitly rather than absorbed into this corrective
milestone.

## Adversarial Courtroom

The decisive proof uses the real `worth-query-host` composition root, the real
Query declaration, installation, admission, execution, and publication
packages, real Relational/Runtime Bridge/Signal owners, and the dynamic Bank
World fixture. An in-memory reenactment that replaces any disputed authority is
not end-to-end evidence.

The courtroom installs one application query and one mutation that share
authorization observations and graph dependencies. It then:

1. authenticates a dynamically created principal and binds an explicit
   purpose and typed input;
2. selects the installed obligations and admits graph work under a live
   capacity reservation bound to an explicit typed branch;
3. opens one managed provider session before the first decision-bearing graph
   observation;
4. resolves Relational graph witnesses, Runtime Bridge correspondence, and
   Signal policy evidence through their production facades;
5. runs a read to its read-only terminal without constructing proposed state;
6. runs a mutation from the same common progression through complete decision
   facts, proposed state, the real installed invariants, and provider
   compare-and-commit;
7. injects relevant negative-space drift and proves the mutation denies;
8. injects an unrelated mutation on the same branch and proves no false
   conflict, then presents an equal-version basis from another branch and
   proves it cannot satisfy the session;
9. revokes or replaces the exact authorization grant after admission and proves
   the old attempt cannot migrate to an equivalent grant or path;
10. cancels at every managed transition and proves the session, reservation,
    basis, buffers, and any proposal release exactly once;
11. repeats the query through current, continuation, historical, preview, and
    live lanes and proves they consume the same graph-read planning authority;
12. grows unrelated installed obligations, grants, graph population, result
    rows, and live consumers independently and verifies the declared structural
    counters; and
13. attempts every known public constructor, raw executor, copied proof,
    cross-runtime plan, cross-basis session, manual invariant hook, and retired-
    authority import from a hostile external consumer crate.

The scale court includes at least 4,096 unrelated grants or obligations and 32
live consumers. Increasing unrelated population may increase only the counters
whose declared index granule includes it. Ordinary planning, execution,
projection, commit, retry, and delivery perform exactly zero canonical-basis
preparation, canonical encoding/allocation, digest derivation, SHA-256
compression, and digest-text materialization.

The courtroom must independently observe:

- the exact selected obligation identities and owner routes;
- the admitted requirement, inventory, budget, and reservation evidence;
- the typed branch identity and single session identity on every graph call
  and decision fact;
- the actual lower-owner invocation and terminal outcome for every obligation;
- the read terminal or mutation proposal/invariant/commit terminal;
- authoritative graph state through a separate observation path;
- terminal receipts and disclosed results;
- capacity and managed-resource baselines after every terminal; and
- compile-fail and dependency-enforcement denial of every bypass.

A mutation probe that turns owner execution into selection-only success, moves
authorization observation before session start, makes a proof constructor
public, restores a direct graph-read executor, or skips an invariant must make
the evidence red.

## Product Decision Lock

### One installed obligation vocabulary

Every installed application query and operation owns one
`WorthQueryInstalledGraphObligationSet`. It is derived at installation from
typed declarations and installed contracts. Runtime callers cannot append,
remove, replace, or reinterpret entries.

Each installed entry identifies:

- its stable installed identity and schema/generation affinity;
- the query or operation that owns it;
- its semantic kind;
- its required lower owner or Query transition;
- its touch or read-selection basis;
- its canonical graph-read requirements;
- its support and resource contract;
- its effect posture: observational, policy, advisory, invariant, or mutating;
  and
- the exact evidence required before the entry may become terminal.

The vocabulary is shared; execution is not falsely unified. A Relational read,
a Bridge/Signal authorization evaluation, and a Relational invariant retain
different proof types, failure topology, and cost evidence.

### One selection and admission chain

Selection consumes installed authority plus the typed query or mutation intent.
It emits a sealed selected set. That set carries no execution authority.

Admission consumes the selected set, the current installed support inventory,
the canonical graph-read requirements, structural cost evidence, the installed
budget, and a real capacity reservation. Only that transition may mint the
sealed graph-work plan.

Public consumers may inspect non-authority views of requirements, costs,
budgets, support, and selected owner routes. Inspection views have no
constructor or round-trip conversion into a proof-bearing phase.

### Session before observation

The sealed graph-work plan can open exactly one managed provider session. The
session is bound to runtime, schema, installation generation, query or
operation, authenticated principal and access context where applicable,
typed branch, semantic basis, provider, managed run, and reservation. Branch
identity is part of authority affinity even while the only installed branch is
the ordinary branch. It is never inferred from `"main"`, a version number, a
snapshot handle, or adapter convention.

Snapshot and version identity are branch-qualified. Equal snapshot or version
ordinals on different branches are not equivalent and cannot substitute in a
read set, proposal, invariant execution, compare-and-commit, retry, receipt, or
publication transition. The session and every retained fact carry the typed
branch forward; later phases may inspect but may not replace it.

No decision-bearing graph read occurs during declaration, installation,
selection, static support matching, cost estimation, or capacity admission.
The provider session exists before Relational observation, authorization fact
collection, query execution, or proposal construction.

Every read product and decision fact is minted through the session. Raw rows,
snapshot handles, copied observation identities, or descriptive digests cannot
be promoted into session evidence.

### Exact owner boundaries

| Responsibility | Owner | Query may do | Query may not do |
|---|---|---|---|
| Graph truth, traversal, negative facts, path witnesses, exact touched scope | Relational | Request work through its facade and retain returned evidence | Reconstruct graph truth or infer touch authority |
| Query declaration translation and Query authority continuity | Query | Validate portable meaning, retain installed Query authority, and emit neutral Bridge requests | Delegate Query comparison or authority interpretation to Runtime Bridge |
| Installed neutral cross-runtime correspondence and lowering | Runtime Bridge | Select installed correspondence and consume sealed evidence | Import Query packages or recreate Query, Signal, or Relational meaning |
| Required/prohibited policy composition and decision evidence | Signal | Consume Bridge-retained policy evidence | Re-evaluate policy branches locally |
| Installed obligation composition, selection, admission, phase progression, lifecycle | Query | Compose the exact lower proofs into attempt authority | Treat lower observations or receipts as authority by themselves |
| Proposed state and invariant graph mechanics | Relational through the provider session | Sequence installed invariants and consume their verdicts | Substitute callbacks or selection rows for execution |
| Terminal envelope, disclosure, and explanation | Query Publication | Derive consumer material from terminal evidence | Execute work, widen disclosure, or mint authority |

Lower evidence constructors may remain public only when the value is
demonstrably non-authoritative and no public Query transition accepts it as a
substitute for evidence minted by the installed runtime. Query authority types
remain sealed and move-only.

### Read and mutation terminals

Read and mutation lanes share installation, selection, planning, admission,
session start, lower-owner observation, and complete decision read-set capture.
They branch only after the common session-bound read work is complete.

A read terminal:

- projects only from session-owned read products;
- publishes a governed read receipt;
- releases the session and every managed resource; and
- cannot construct proposed-state, invariant, or commit authority.

A mutation terminal:

- consumes the same session-bound decision facts;
- constructs a provider-backed proposed state;
- executes every selected blocking invariant through its actual owner;
- advances only an invariant-approved proposal to compare-and-commit;
- publishes the typed commit, denial, conflict, cancellation, or indeterminate
  outcome; and
- releases every resource exactly once.

### Application-query role

An installed application query retains canonical semantic identity, parameter
binding, result meaning, ordering, projection, continuation, history, preview,
and live semantics. Its private kernel may translate a sealed graph-work plan
into result-shaping mechanics.

It does not own another requirement, inventory, budget, admitted plan, session,
or graph execution authority. `review_graph_read_access` and caller-built
planning evidence are removed from the public facade. Application query code
cannot call raw Relational runtime state without the session-owned port.

### Receipts and envelopes

Receipts are terminal evidence. They are minted only from actual owner
completion plus the session terminal. A selected or dispatched row cannot be
reported as executed. An unsupported or unavailable owner produces a typed
denial before effects; it never produces synthetic success.

Receipts retain identity, basis, owner completion, decision-read-set posture,
resource and structural work, outcome, disclosure posture, and release
evidence. They grant no retry, commit, continuation, or execution authority.

### Performance and cryptographic posture

Canonicalization and SHA derivation occur only at installation or bounded fresh
admission when a new semantic identity is genuinely created. Retained typed
identities are carried through selection, support matching, session start,
observation, execution, projection, commit, retry, live delivery, and terminal
publication.

Named counters separate:

- installed entries considered;
- selector index probes and selected entries;
- requirement rows;
- inventory probes and supported rows;
- intrinsic and supported cost work;
- capacity reservations;
- provider calls and returned facts;
- traversed entities, relations, adjacency lists, and fields;
- decision facts retained and revalidated;
- invariant executions;
- projected rows, fields, and retained bytes;
- releases; and
- canonical preparation, encoded bytes, canonical allocation, digest
  derivation, SHA compression, and digest-text materialization by phase.

Warm work scales with selected obligations, admitted graph requirements,
actual traversal, returned/projected result shape, exact authorization witness,
and installed synchronous invariants. It does not scale with unrelated
obligations, grants, graph population, projections, diagnostics, or consumers.

Milestone 9.16.1 does not implement multiple branch heads or concurrent branch
writers. It does require that reservation and commit coordination remain
branch-scopeable: a temporary global coordinator may limit concurrency, but it
is not semantic authority and no public proof or contract may define global
serialization as required correctness.

## Compiler-Visible Progression

The exact private representation is an implementation decision, but the phase
topology is fixed:

```rust
InstalledGraphObligations
    -> SelectedGraphObligations
    -> AdmittedGraphWorkPlan
    -> ManagedGraphSession
    -> SessionDecisionReadSet
    -> ReadCompletion
       | ProposedGraphState
           -> InvariantApprovedProposal
           -> ProviderCommitOutcome
    -> PublishedGraphOutcome
```

Each arrow consumes the prior proof. No later type has a public constructor.
Read completion and proposed state are disjoint terminal branches. Inspection
types are not accepted by any arrow.

## Destination Directory And Module Skeleton

The labels below are normative: **retain** keeps correct ownership, **create**
establishes a destination, **integrate** consumes an existing owner unchanged,
**migrate** prepares a parity-backed successor while the predecessor remains
authoritative, **cut over** atomically transfers the covered consumers and
authority, **narrow** removes public construction or execution power, and
**retire** removes only the exact superseded authority after parity and cutover.

```text
workspaces/worth-query/crates/
  worth-query-decl/src/
    facade.rs                                      [retain declaration-only audience]

  worth-query-installation/src/
    graph_obligation/                              [create/complete]
      mod.rs                                       [facade only]
      contract.rs                                  [installed entry meaning]
      kind.rs                                      [semantic kind]
      owner.rs                                     [required owner/transition]
      installed_set.rs                             [sealed installed family]
      query_binding.rs                             [create]
      operation_binding.rs                         [create]
      selection_index.rs                           [move from monolith index]

  worth-query-admission/src/
    graph_obligation/                              [create]
      mod.rs                                       [facade only]
      selection.rs                                 [move actual selector]
      selected_set.rs                              [create sealed proof]
      support_admission.rs                         [create orchestration]
      capacity_admission.rs                        [create orchestration]
      admitted_plan.rs                             [create sealed proof]
      denial.rs                                    [typed phase denials]
    graph_read_access/                             [retain and narrow]
      requirement_*.rs                             [retain mechanics]
      graph_index_inventory/                       [retain mechanics]
      cost_model/                                  [retain mechanics]
      planning_input.rs                            [narrow construction]
      planning_derivation.rs                       [crate-private]
      plan_review.rs                               [replace public review]

  worth-query-execution/src/domain_computation/
    provider_session/                              [retain authority owner]
      graph_obligation/                            [create]
        session_start.rs                           [single opening]
        branch_affinity.rs                         [typed basis qualification]
        owner_execution.rs                         [exact owner routing]
        decision_read_set.rs                       [session capture]
        read_terminal.rs                           [read-only completion]
        mutation_progression.rs                    [proposal handoff]
        terminal_release.rs                        [exact cleanup]
      provisional_attempt/                        [retain]
        invariant_execution/                      [retain real execution]
    authorization/                                [retain and bind]
      admission.rs                                 [session-bound]
      bridge_observation.rs                        [session-bound]
      decision_facts.rs                            [session-bound]
      authorization_revalidation.rs                [retain exact evidence]
    primary_graph/application_query/               [retain semantic kernel]
      graph_read_plan_binding.rs                   [replace authority role]
      read_execution/                              [private session port only]
      basis/                                       [consume session basis]
      continuation/                                [consume terminal receipt]
      live/                                        [consume same sealed plan]

  worth-query-publication/src/
    graph_obligation/                              [create]
      mod.rs                                       [facade only]
      read_receipt.rs                              [terminal evidence]
      mutation_receipt.rs                          [terminal evidence]
      obligation_outcome.rs                        [owner completion]
      inspection.rs                                [non-authority views]

  worth-query-host/src/
    facade.rs                                      [narrow stable execution audience]

  worth-query-certification/
    tests/
      canonical_graph_progression/                 [create in one harness]
        public_consumer.rs
        authority_substitution.rs
        owner_execution.rs
        session_affinity.rs
        read_terminal.rs
        mutation_terminal.rs
        lifecycle.rs
        scale.rs
        residue.rs

  worth-query/src/runtime/mutation/graph_composition/
    obligation/                                    [migrate exact authority rows]
      ...                                           [retire only after every row cuts over]
  worth-query/src/runtime/graph_read_access/        [retain feature behavior]
    ...                                             [migrate authority roles per lane]
  worth-query/src/runtime/                          [retain product composition root]
    ...                                             [out of scope unless named by a row]

crates/
  worth-relational/src/authorization/              [retain]
  worth-runtime-bridge/src/authorization/          [retain]
  worth-signal/src/data/authorization/              [retain]
```

The stable axis is authority progression. Installation owns immutable meaning,
admission owns current eligibility and reservation, execution owns the live
session and effects, and publication owns non-authoritative terminal views.
`graph_read_access` remains the specialized planning mechanism under admission;
it does not become a second progression. `application_query` remains semantic
query execution under the session; it does not become a second authority.
`worth-query-decl` remains the declaration-only audience facade, while
`worth-query-host` exposes the ordinary execution entry and inspection views by
re-exporting destination facades; neither facade implements domain behavior or
authority transitions.

Forbidden destinations include a new `common`, `helpers`, or `compatibility`
module; an authority-capable wrapper around the monolith tree; a destination
package that imports `worth-query`; and any facade file that implements
planning, execution, policy, or receipt construction. A one-way monolith facade
adapter that lowers into the destination authority is allowed while it remains
the sole product path and grants no independent authority.

## Phase Plan

### Phase 1: Installed Obligation Authority And Owner Matrix

**What becomes true**

Every installed application query and operation owns one sealed graph-obligation
set whose entries identify their exact semantic kind, lower owner, selection
basis, graph-read requirements, resource posture, and required terminal
evidence.

**Consumes**

- typed query, operation, capability, invariant, and resource declarations;
- closed 9.9 touch-selection semantics;
- closed 9.10 graph-read requirement and cost semantics; and
- current Query installation authority.

**Establishes**

- destination-package ownership of the installed obligation vocabulary and
  selector index;
- query and operation binding without caller-authored strings or summaries;
- an exhaustive kind-to-owner matrix; and
- installation/encoded-byte budgets plus retained typed identities.

**Mechanically forbids**

- public construction or mutation of installed obligation sets;
- owner selection by strings, callbacks, or runtime type discovery;
- an installed query or operation with an unowned executable obligation; and
- warm canonicalization or hashing of retained obligation meaning.

**Closure evidence**

- installation twins for every semantic kind and owner route;
- unsupported combinations deny during installation;
- public compile failures for counterfeit installed sets plus exact foreign-
  runtime and cross-generation substitution denials;
- structural identity and budget evidence; and
- the existing Phase 7.1 capability and Bank installation suites remain green.

**Next trust**

Admission may select from one complete installed authority without importing
the monolith.

### Phase 2: Single Admission Spine And Session-Before-Observation

**What becomes true**

Every governed read and mutation uses one selection, requirement, inventory,
cost, budget, capacity, sealed-plan, and provider-session progression. The
session begins before the first decision-bearing graph read.

**Consumes**

- the Phase 1 installed obligation set;
- installed graph-read support and budgets;
- real execution-resource reservation;
- authenticated principal and access context where required; and
- an admitted semantic basis; and
- an explicit typed branch identity supplied by the installed runtime basis.

**Establishes**

- sealed `SelectedGraphObligations` and `AdmittedGraphWorkPlan` transitions;
- one managed session identity carried by every lower-owner call and fact;
- branch-qualified snapshot/version affinity carried without string or
  default-branch inference;
- exact lower-owner routing through Relational and Runtime Bridge/Signal; and
- read and mutation terminal branches after the shared decision read-set.

**Mechanically forbids**

- caller-built planning evidence or executable review;
- raw graph observation before session start;
- cross-runtime, cross-generation, cross-operation/query, cross-basis,
  cross-provider, or cross-principal plan/session substitution;
- same-version cross-branch substitution and hard-coded default-branch
  authority;
- selection-only execution success; and
- read-only construction of proposed-state or invariant authority.

**Closure evidence**

- one real Bank read and one real Bank mutation traverse the complete public
  host boundary;
- hostile public consumers fail to construct every proof-bearing phase;
- session identity is independently observed on every decision fact;
- a hostile lower-owner basis carrying another typed branch at the same version
  ordinal fails substitution while same-branch evidence continues lawfully;
- unsupported owners fail before effects; and
- denial and cancellation release reservation and session resources exactly
  once.

**Next trust**

All ordinary consumers can migrate without inventing another authority path.

### Phase 3: Application Query And Read-Lane Cutover

**What becomes true**

Current, continuation, historical, preview, and live application-query lanes
consume the Phase 2 plan and session. Application-query planning remains
semantic result shaping, not graph-read authority.

Migration proceeds one named lane family at a time. The existing lane remains
authoritative until the destination path demonstrates result, denial,
continuation or live lifecycle, receipt, cleanup, and warm-cost parity through
the public facade. Each lane cutover switches all covered callers and disables
the exact predecessor planning or execution authority in the same slice. An
uncut lane remains a declared predecessor responsibility, not a failure to
delete enough of the monolith.

**Consumes**

- the Phase 2 admitted plan and session;
- installed application-query identity and result contract;
- session-owned read products and complete read-set; and
- existing basis, continuation, live, projection, and buffer lifecycles.

**Establishes**

- one graph-read planning path across every query lane;
- read-only terminal receipts and exact resource release;
- private execution-kernel access only through the session-owned graph port;
  and
- identical query/result meaning across one-shot, continuation, history,
  preview, and live delivery.

**Mechanically forbids**

- application-query-owned admitted graph plans;
- direct raw Relational runtime execution;
- consumer-created continuation or live resources;
- proposed-state or invariant theater on read terminals; and
- public `review_graph_read_access` as a route to execution.

**Closure evidence**

- public consumer transcripts for every lane;
- cross-lane, cross-basis, cursor, continuation, and live-session hostility;
- negative-space, ordering, membership, and projection dependencies appear in
  the common read-set;
- cancellation and subscriber disposal return exact baselines; and
- direct-planner, direct-executor, and obsolete public-export residue is zero.

**Next trust**

Mutation can use the same read/session foundation without preserving a read
bypass.

### Phase 4: Authorization, Mutation, Invariant, And Commit Cutover

**What becomes true**

Capability and ordinary authorization observations occur inside the same
managed provider session whose complete decision facts support proposed state,
real installed invariant execution, and provider compare-and-commit.

Cutover is likewise per named authority family: authorization observation,
proposal construction, installed invariant execution, and provider commit.
Existing Relational, Runtime Bridge, Signal, and Query semantics integrate
unchanged when already correctly owned. A predecessor Query surface retires
only after the real Bank mutation and hostile twins prove that its destination
performs the same supported work and that bypassing either owner turns the
evidence red.

**Consumes**

- the Phase 2 session spine;
- preserved Relational exact observations and witnesses;
- Runtime Bridge correspondence and Signal decision evidence;
- Phase 7.2 principal, purpose, exact-request, trusted-time, and exact-grant
  facts;
- installed mutation preconditions; and
- Milestone 9.15 proposed-state and invariant progression.

**Establishes**

- one attempt-bound authorization composition owned by Query;
- complete positive, negative, traversal, membership, ordering, time,
  principal, grant, and policy decision facts;
- actual owner execution for every selected blocking invariant; and
- terminal outcomes derived from the provider session and compare-and-commit.

**Mechanically forbids**

- equivalent-grant or equivalent-path rescue of stale authority;
- policy evaluation or graph-truth reconstruction inside Query;
- manual invariant packs, no-op invariant defaults, and callback authority;
- execution rows minted from selection or dispatch alone; and
- receipts or idempotent disclosure before current authorization re-admission.

**Closure evidence**

- the Bank mutation courtroom covers relevant and unrelated drift, principal
  disablement, grant revocation/replacement, trusted-time expiry, response loss,
  idempotent retry, cancellation, invariant violation, and commit conflict;
- independent Relational and Bank oracles verify state and policy outcomes;
- mutating or skipping each selected invariant makes the suite red; and
- Phase 7.2 consumer proofs remain green through the canonical session path.

**Next trust**

Each exact predecessor authority can retire without losing a legal consumer or
product feature.

### Phase 5: Parity Closure, Exact Authority Retirement, And Publication

**What becomes true**

Destination packages own the covered canonical progression; every exact
authority-capable predecessor whose consumers have cut over is retired.
Unrelated monolith behavior and every not-yet-migrated feature remain intact.
Terminal publication describes actual executed work and grants no authority.

**Consumes**

- the complete read and mutation cutovers from Phases 3 and 4; and
- the semantic-surface migration, parity, and evidence ledgers.

**Establishes**

- destination publication receipts and inspection views;
- public facade and documentation agreement;
- permanent dependency, visibility, per-surface residue, and performance
  enforcement; and
- an explicit handoff back to Milestone 9.16 Runtime Phase 7.3.

**Mechanically forbids**

- importing the monolith from destination packages to recover authority;
- shims or aliases that can mint or execute a retired authority;
- public proof constructors and direct executors;
- two canonical graph-read planning products; and
- warm canonicalization, hashing, hidden scans, synthetic owner completion, or
  leaked managed resources.

**Closure evidence**

- every graph-obligation authority row has a named predecessor, destination,
  parity proof, cutover receipt, and exact residue result;
- the monolith obligation tree retires only after all of its authority rows
  have migrated, while unrelated `worth-query` runtime and public behavior
  remain present and green;
- manual invariant hooks retire only after installed owner execution proves
  parity for every supported invariant family;
- the hostile external-consumer crate proves exactly one obligation path and
  one graph-read planning path;
- compile-pass/fail evidence is consolidated into the fewest practical compiler
  sessions;
- scale twins cover every independent warm-work axis without Cartesian test
  multiplication;
- documentation examples compile against the real public facade;
- Clippy, line-cap, boundary-check, and generated-context checks pass; and
- every high or critical ledger finding is closed with root-cause and residue
  evidence; and
- the full Query monolith, public declarative journeys, Worth UI Query binding,
  and Bank consumer lanes remain green at each final cutover boundary.

**Next trust**

Milestone 9.16 resumes at Runtime Phase 7.3 and may rely on one canonical
obligation, graph-read planning, session, authorization, and receipt chain.

## Caller DX Target

The ordinary application caller names semantic intent, identity, purpose, and
controls. Query owns installation lookup, obligation selection, planning,
admission, session lifecycle, and lower-owner orchestration.

```rust
let activity = bank
    .query::<AccountActivity>(
        &authenticated_principal,
        AccountActivityInput { account, page },
        AccessPurpose::AccountServicing,
    )
    .await?;

let transfer = bank
    .execute::<TransferFunds>(
        &authenticated_principal,
        TransferFundsInput {
            from,
            to,
            amount: Money::usd_cents(12_500),
        },
        AccessPurpose::CustomerInitiatedTransfer,
        idempotency_key,
    )
    .await?;
```

The advanced caller may inspect a non-authoritative plan view before execution,
but cannot execute that view or reconstruct its sealed source:

```rust
let request = bank
    .queries()
    .prepare::<AccountActivity>(
        &authenticated_principal,
        AccountActivityInput { account, page },
        AccessPurpose::AccountServicing,
    )?;

let inspection = request.inspect();
assert!(inspection.resource_budget().is_bounded());

let activity = bank.execute(request).await?;
```

No caller imports Relational, Runtime Bridge, Signal, provider-session internals,
graph-read planning constructors, or monolith modules. No string key selects a
query, obligation, owner, policy, field, relation, or invariant.

## Documentation Deliverables

Implementation closure must:

- revise `workspaces/worth-query/crates/worth-query/docs/AI_README.md` so its
  canonical runtime map points to the single progression;
- create
  `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md`
  as the durable architectural explanation for declarations, installed
  obligations, graph-read planning, session lifecycle, owner execution,
  terminals, and inspection;
- revise `authoring/graph-touch-obligation-authority.md` to describe selection
  as non-executable installed meaning;
- revise `authoring/graph-read-access-planning.md` to describe the specialized
  planning mechanism inside the canonical admission spine;
- remove `authoring/graph-obligation-consumer-kit.md` if it documents local
  registration or execution authority, or rewrite it only if a real public
  consumer-facing inspection/adoption responsibility remains;
- revise `domain-capabilities/provider-sessions-and-decision-read-sets.md` and
  `domain-capabilities/provisional-state-and-invariant-execution.md` to show the
  common read/mutation progression, typed branch affinity, and the rule that
  snapshot/version identity is meaningful only inside that branch; and
- add the per-surface migration table to the canonical graph-progression
  document, including the inherited 9.9, 9.10, and 9.11 guarantee, current
  owner, destination owner, covered consumers, parity evidence, cutover, and
  exact retirement posture; and
- compile or run every public call sequence in the ordinary documentation gate.

The new canonical document owns the cross-surface mental model. The narrower
documents explain their specific authoring or lifecycle responsibility and may
not duplicate or contradict the progression.

## Must Ship

- one installed graph-obligation set per installed query or operation;
- one sealed selection, inventory, cost, budget, capacity, plan, and provider
  session chain;
- typed branch affinity on the plan, session, read set, proposal, invariant,
  commit, retry, receipt, and publication chain;
- one graph-read planning authority consumed by every application-query lane;
- session-bound Relational, Runtime Bridge, and Signal evidence;
- complete decision read-sets before read/mutation branching;
- honest read-only and mutation terminals;
- actual installed invariant execution before commit;
- receipts derived only from terminal owner evidence;
- exact lifecycle and warm-work counters;
- parity-backed cutover and retirement of every exact authority-capable
  predecessor covered by this milestone, without unrelated feature deletion;
- hostile consumer and residue proof sufficient to resume Milestone 9.16.

## Must Preserve

- all closed historical milestone statuses and closeout evidence;
- Relational ownership of graph truth and exact mechanics;
- Query ownership of Query-to-neutral translation, portable conditional
  comparison, and Query authority continuity;
- Runtime Bridge ownership of installed neutral correspondence, crossing
  admission, and Signal-facing lowering;
- Signal ownership of policy evaluation evidence;
- Query ownership of legal composition and compiler-visible phase progression;
- Foundational ownership of canonical encoding and SHA mechanics at bounded
  cold seams;
- Phase 6 query identity, basis, lane, result, buffer, continuation, live, and
  warm-path guarantees;
- branch-qualified basis identity, with no assumption that a version ordinal
  is globally unique;
- Phase 7.1 installed capability meaning and budgets;
- Phase 7.2 purpose, request binding, trusted time, exact grant/path witness,
  principal currentness, policy evidence, and revalidation guarantees;
- cert-only replay and the ordinary/reconstructive lane boundary; and
- typed denials, cancellation, conflict, partial, and indeterminate outcomes;
- the complete existing Query feature set and ordinary public facade until each
  exact semantic surface has a proved destination cutover.

## Explicit Non-Goals

- durability, restart resume, store integration, or checkpoint recovery;
- new Bank domain behavior;
- Runtime Phase 7.3 disclosure and noninterference implementation;
- Runtime Phase 7.4 delegation, Phase 7.5 conflict composition, Phase 7.6
  emergency elevation, or Phase 7.7 Bank estate cutover;
- Runtime Phase 8 aftermath, Runtime Phase 9 host-installed conditional
  operations, or Runtime Phase 10 public policy cutover;
- advanced Milestone 9.19 access products;
- multiple branch heads, per-branch version allocation, concurrent branch
  writers, branch creation, merge, rebase, or branch-local inversion;
- a generic cross-runtime workflow engine;
- crate-wide decomposition or deletion of the `worth-query` monolith; or
- preservation of source compatibility for an exact authority-capable API
  after its covered consumers have cut over. Stable product behavior and the
  ordinary facade remain required.

## Acceptance Evidence

Milestone 9.16.1 closes only when:

1. every requirement and high/critical finding in its closure ledger is closed;
2. one real Bank read and one real Bank mutation traverse the public host
   boundary and the same canonical session spine;
3. every application-query lane consumes the same graph-read planning
   authority;
4. authorization observations and decision facts are session-bound before use;
5. selected invariants execute through their actual owner before commit;
6. read-only work cannot enter proposal or invariant phases;
7. public construction, cross-affinity substitution, raw execution, manual
   invariant, and retired-authority import attempts fail mechanically;
8. equal-version evidence from another typed branch cannot satisfy any plan,
   session, read-set, proposal, invariant, commit, retry, receipt, or
   publication transition, and no ordinary authority derives branch meaning
   from the string `"main"`;
9. no destination package depends on the monolith and the monolith reaches
   covered destination authority only through one-way facade lowering;
10. every covered semantic surface has a complete parity and cutover row, every
    authority-capable predecessor for a completed row is absent, and unrelated
    monolith features remain present and green;
11. exact lifecycle baselines return after success, denial, conflict,
    cancellation, failure, and indeterminate outcomes;
12. scale-sensitive evidence proves unrelated population and consumer fan-out
    do not widen ordinary work or trigger warm canonical/SHA activity;
13. the new canonical documentation and AI README agree with the real facade;
14. the Query monolith, public declarative journeys, Worth UI Query binding,
    and Bank consumer suites prove that no supported feature disappeared during
    migration; and
15. the boundary checker, generated context checker, line-cap audit, targeted
    tests, workspace tests appropriate to the touched boundary, and strict
    Clippy all pass.

## Handoff Back To Milestone 9.16

After closure, Milestone 9.16 resumes at Runtime Hardening Phase 7.3. Phase 7.3
may trust that internal-computation and disclosure admission begin from one
session-bound access decision and one canonical graph-read plan. It may not
reintroduce a disclosure-specific selector, planner, raw graph executor,
receipt, or compatibility path.

The handoff is additive to the existing Milestone 9.16 front door. Runtime
Phase 7.3 consumes the same installed application-query identity, parameter
binding, basis, continuation, history, preview, live, result-shaping, and
publication behavior already established by Runtime Phase 6. Milestone 9.16.1
changes which authority proves graph planning and session progression; it does
not replace those feature contracts or require Phase 7.3 to reconstruct them.

The handoff record must enumerate the exact 9.9 obligation, 9.10 graph-read,
9.11 downstream-authority, 9.15 provider-session, and 9.16 Phase 6/7.1/7.2
guarantees that integrate unchanged, plus every exact surface migrated by
9.16.1. Any broader work discovered during that reconciliation is scheduled
explicitly and does not silently expand the prerequisite for Phase 7.3.

Runtime Phase 7.3 and every later 9.16 phase inherit the session's typed branch
unchanged. They may not default to `"main"`, treat version identity as global,
or introduce a disclosure-, recovery-, aftermath-, transport-, or
publication-local branch choice. Actual multiple-head and concurrent-writer
mechanics plus exact owner Relational/Signal component bases remain the
Milestone 9.17.1 handoff. Composite product-branch history/publication follows
in 9.17.2, and complete Query carriage/public-facade cutover follows in 9.17.3.

The 9.16 Runtime Phase 7 ledger retains its existing historical rows. The
9.16.1 closure ledger is the additional prerequisite for Phase 7.3 and all
later 9.16 phases.
