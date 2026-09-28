# Milestone 3.15 Phase 2 Implementation Plan

## Governing Sources and Precedence

This plan implements, but does not redefine:

1. [`milestone-3.15.md`](milestone-3.15.md), especially the proposal compiler,
   publication, support, evidence, and ordered-phase contracts;
2. [`worth_ui_roadmap.md`](worth_ui_roadmap.md), which places production
   runtime services before appearance-state consumption and future undo/redo;
3. `workspaces/worth-ui/AI_README.md` and the repository engineering
   constitution under `docs/coding-guidelines/`; and
4. the closed Phase 1 compiler-visible request basis, family inventory,
   dependency stages, proposal census, occupancy vocabulary, and private
   destination topology.

If implementation requires the compiler to own family state, publish facts,
issue effects, reconstruct authority, switch over families to implement family
behavior, or expose public service declarations early, stop and repair the
plan or parent specification. No compatibility lane or generic payload bag is
permitted.

## Objective

Turn the Phase 1 proposal vocabulary into one bounded, session-owned proposal
lifecycle exercised against recorded typed fixture proposals. Phase 2 must
preflight one coherent proposal set, reserve exact budgets and occupancy,
enforce the fixed semantic dependency order, produce an inert batch of
owner-issued fact/work references for the existing publication boundary, and
release every compiler-owned resource after typed acceptance or rejection and
owner acknowledgement.

The compiler still does not implement portal, focus, motion, command, scroll,
or selection behavior. It cannot publish or issue effects. Phase 3 may consume
the lifecycle only after Phase 2 proves that independent family owners have
one lawful, zero-residue staging path.

## Current Readiness Boundary

Phase 1 provides the correct vocabulary but intentionally no production
lifecycle:

- `UiServiceRequestCoherence` binds application generation, semantic and host
  surfaces, binding, presentation, causal root, cancellation identity, and
  resource-budget identity;
- `UiServiceProposalCandidate` carries typed family participation and bounded
  demand;
- `UiServiceProposalStage::ORDER` fixes the seven semantic stages;
- `UiServiceProposalCompiler` currently records only proposal admission;
- `UiServiceProposalOccupancyLease` has no issuer, table, conflict policy, or
  release path;
- census operations are isolated counters rather than one atomic lifecycle;
- stage receipts currently prove only proposal identity and stage, so they do
  not prove owner, family participation, scope, or exact staged references;
- publication receipt and owner acknowledgement constructors are test-only and
  not connected to a terminal lifecycle; and
- no compiler state is installed under the runtime session owner.

Phase 2 closes those gaps using recorded, typed fixture owners. It does not
mount the real service families, create product controls, or claim `RS-01`,
`RS-07`, or `RS-10` complete.

## Destination Authority

| Responsibility | Owner | Phase 2 boundary |
|---|---|---|
| Common causal and coherence basis | `runtime/session/service_proposal/request_basis.rs` | carried, sealed, and compared; never reconstructed from diagnostics |
| Proposal lifecycle and bounded tables | runtime-session `UiServiceProposalCompiler` | exact active proposals, occupancy, cancellation posture, receipts, and terminal release only |
| Family state and unpublished successors | each family owner | represented in Phase 2 by sealed recorded fixture witnesses; never copied into compiler state |
| Fixed dependency order and cycle law | `compiler/dependency.rs` | semantic stage graph, not family behavior or dynamic dispatch |
| Budget admission | `compiler/preflight.rs` plus named budget records | atomic reservation before owner staging; no partial reservation |
| Occupancy/conflict policy | `occupancy.rs` | exact application/surface/family/scope key and bounded typed outcomes; never arrival-time queueing |
| Candidate facts and mounted work | existing owner-ranked fact/mounted contracts | compiler batch carries sealed references only; it does not mint facts or mounted frames |
| Atomic publication | existing application/mounted publication owner | Phase 2 uses a typed recorded publication port; compiler cannot invoke lower-level mutation paths |
| Owner commit/discard | participating family owners | exact publication receipt is returned to each owner, which emits one terminal acknowledgement |
| Physical work and settlement | existing mounted presentation/host-truth owners | absent from Phase 2 compiler interfaces and state |
| Inspection | compact census/current posture | reads produced lifecycle evidence; cannot mutate or retain family payloads |

The lawful Phase 2 transaction is:

```text
typed recorded owner proposals
  -> coherent-set validation
  -> atomic budget and occupancy reservation
  -> fixed dependency-stage witnesses
  -> sealed owner fact/work references
  -> inert staged publication batch
  -> existing publication accept/reject receipt
  -> exact receipt presented to every participating owner
  -> one terminal acknowledgement per owner
  -> occupancy, cancellation, receipt, and proposal release
  -> exact-zero proposal census
```

No compiler method returns an application generation, mounted frame authority,
host effect, family successor, or family-owned semantic receipt.

## Data and Lifecycle Decisions

### Recorded typed fixtures

Phase 2 certification uses production compiler types with test-only recorded
fixture issuers. A fixture records the family, owner/scope identity, declared
axes, sealed stage completion, owner-produced fact references, and ordinary
mounted-work references. It cannot carry `dyn Any`, bytes, strings as identity,
callbacks, family successor state, or publication authority.

The fixture issuer is not exposed by the runtime facade and is compiled only
for unit/certification construction. Production lifecycle code cannot depend
on fixture modules.

### Coherent proposal set

One compilation transaction has one `UiServiceRequestCoherence`. Every
candidate must match all eight axes before any resource changes. Participation
must be non-empty, duplicate-free, installed in the runtime service support
snapshot, and exactly equal to the set of recorded family proposals. An owner
cannot add an undeclared family, scope, fact reference, or mounted-work
reference after preflight.

### Budget shape

Use named limits and concrete reservation counts for proposals, participating
families, requirements, staged fact references, mounted-work references,
occupancy leases, stage receipts, cancellation records, and owner
acknowledgements. Checked arithmetic happens before mutation. A denial leaves
the prior compiler state and census unchanged.

### Occupancy and cancellation

Occupancy is keyed by exact application generation, semantic surface, family,
and family-declared scope identity. Each recorded family policy chooses one of
the closed dispositions: occupied, superseded, coalesced, or
cancelled-before-effect. No queue or implicit last-arrival winner exists.

Cancellation is a compiler-owned posture under the carried cancellation
identity, not a replay log or undo record. Before-publication cancellation
discards staged references through owner acknowledgement and performs no
semantic effect. A publication-terminal proposal cannot be cancelled back into
an earlier phase.

### Stage witnesses and cycle law

Each stage witness binds proposal identity, exact stage, issuer role, and the
preflighted participant set. Family-owned staging includes exactly one sealed
witness per participating family. Successor assembly, focus/reveal, and motion
stages use their named issuer roles without embedding algorithms.

The fixed graph is validated independently from input order. Duplicate edges,
back edges, skipped stages, repeated focus-reveal refinement, family-created
cycles, foreign proposal receipts, and receipts from nonparticipants are typed
denials. Phase 2 permits at most one focus-reveal refinement marker and never
calls scroll or focus behavior.

### Publication and settlement

`UiServiceProposalStagedBatch` contains coherence, proposal identity, exact
participants, bounded owner-issued fact/work references, and a digest over
their canonical order. It is inert: no method can publish or issue effects.

The existing publication boundary is represented in Phase 2 by a narrow typed
port that consumes the staged batch and returns one proposal-bound accept or
reject receipt. The compiler then enters settlement and accepts exactly one
acknowledgement from every participating owner. An acknowledgement binds the
same proposal, family, owner scope, and publication disposition. Only complete
settlement releases occupancy and returns the census to zero.

Shutdown terminalizes every before-effect proposal with typed abandonment or
cancellation, rejects shutdown while a publication result is awaiting required
owner acknowledgement unless the recorded port supplies a typed terminal
owner outcome, and proves exact-zero residue. Silence is never success.

## Implementation Batches

### Batch A: coherent-set preflight and atomic budgets

1. Split `compiler/proposal.rs` into named proposal-set, demand, and recorded
   reference responsibilities as needed to remain below the line cap.
2. Add typed per-family recorded proposals and exact participant-set
   validation without exposing real family behavior.
3. Replace aggregate `requirements`/`staged_outputs` folklore with named,
   checked demand counts while preserving bounded compact representation.
4. Validate coherence, installed family support, declared-axis scope, and all
   limits before mutating compiler state.
5. Prove empty, duplicate, unsupported, stale, mixed-coherence, overflow, and
   partial-preflight mutants leave the census and occupancy unchanged.

Batch A closes only admission. It does not create leases or stage witnesses.

### Batch B: occupancy, conflict, cancellation, and session ownership

1. Implement the exact occupancy key, bounded table, slot generation, and
   move-only lease issuance in `occupancy.rs` with named conflict policy input.
2. Add a bounded cancellation registry and typed before-effect lifecycle
   posture under the carried cancellation identity.
3. Make one compiler instance runtime-session owned and initialize it at
   runtime launch without allocating family owner state or performing
   per-frame work.
4. Make proposal, occupancy, cancellation, and budget reservation changes
   atomic and census-coupled.
5. Prove occupied, superseded, coalesced, cancelled-before-effect, stale lease,
   ABA slot generation, capacity, and zero-unused-family cases.

Batch B may reserve and cancel admitted work. It still cannot stage owner facts
or call publication.

### Batch C: fixed DAG and sealed owner staging

1. Replace freely constructible test stage receipts with sealed issuer-role
   witnesses bound to exact proposal identity and participants.
2. Implement fixed-order progression through validation, family staging,
   successor assembly, focus/reveal, and motion derivation, stopping before
   publication submission.
3. Add bounded canonical owner fact references and mounted-work references;
   validate that every reference belongs to a declared participant and exact
   owner scope.
4. Enforce one reveal refinement, no family-to-family invocation, no family
   switch implementing behavior, and no dynamic payload bag.
5. Prove cycles, duplicate/back/skipped stages, foreign owners, scope widening,
   undeclared facts/work, ambiguity, and order-randomization mutants.

Batch C emits one inert `UiServiceProposalStagedBatch`. It does not install a
publication implementation.

### Batch D: publication receipt, owner settlement, and teardown

1. Add the narrow existing-publication port and proposal-bound accept/reject
   receipt without exposing application or mounted publication authority.
2. Add settlement state that presents the exact receipt to each participating
   recorded owner and accepts one exact terminal acknowledgement per owner.
3. Release receipts, cancellation records, occupancy, budgets, and proposal
   state only after complete acknowledgement.
4. Implement typed before-effect shutdown/cancellation and exact-zero closure;
   preserve explicit awaiting-acknowledgement posture rather than treating
   silence as success.
5. Prove accepted and rejected paths, duplicate/foreign/mismatched
   acknowledgements, cancellation races, shutdown at every phase, publication
   port failure, and exact-zero census after every terminal route.

Batch D closes the Phase 2 lifecycle but not later family semantics.

### Batch E: adversarial evidence and handoff closure

1. Add focused runtime boundary tests using production compiler paths and
   independently ordered recorded fixture inputs.
2. Add structural source checks that the compiler cannot import or name host
   effect settlement, publication mutation, family state, replay, Query, a
   generic payload bag, or a family-behavior switch.
3. Add scale-shaped bounded-work tests for 64 independent neighborhoods and
   assert `proposal_requirements_visited` follows admitted proposals while
   `unrelated_neighborhoods_touched` remains zero. Pair that behavioral proof
   with a compile-visible assertion that the index key is the complete active
   application generation plus semantic surface, so a session-only bucket
   cannot keep the test green by filtering full keys inside one vector.
4. Add runtime launch/teardown evidence proving an unused compiler allocates no
   family state and all proposal resources return to zero.
5. Update only durable internal architecture/runtime-subsystem documentation
   needed to describe the now-real compiler lifecycle. Keep public service
   facade and Pulse behavior claims reserved for later phases.

Batch E closes Phase 2 only after all three Grok QA reviews, corrections, and a
Sol-high gate are clear.

## Test and Review Strategy

Focused evidence must include:

- unit tests beside each named owner for checked construction and exhaustive
  lifecycle transitions;
- runtime boundary tests for stale coherence, cycles, scope widening, bounded
  conflicts, family-switch enforcement, publication non-authority, and
  zero-resource teardown;
- an independent small model for occupancy and terminal census, implemented in
  test code without calling production conflict or settlement reducers;
- order-permutation cases proving canonical results do not depend on fixture
  input order;
- compile/source enforcement for sealed witnesses and forbidden dependencies;
- affected all-target checks, focused runtime tests, host/headless/native tests
  only where contracts change, and the full runtime suite at the phase gate;
- dirty line-cap, formatting, `git diff --check`, boundary-check, and generated
  agent-context checks.

After implementation, keep three persistent Cursor Agent sessions on
`cursor-grok-4.6-high`, each with a distinct skeptical remit:

1. `qa-loop`: specification, lifecycle, stale/denial, and integration defects;
2. `qa-tests`: oracle independence, mutation value, boundary honesty, and
   proportional proof; and
3. `code-quality-qa`: responsibility placement, naming, future insertion,
   authority topology, and file composition.

Correct all material findings, resume the same sessions for re-review, then
run one `gpt-5.6-sol` high-reasoning read-only gate. Phase 3 does not begin
until Sol returns clear.

## Phase 2 Completion Contract

Phase 2 is complete only when:

- one runtime-session compiler owns the bounded proposal lifecycle and no
  family behavior;
- recorded typed fixture proposals traverse one coherent, fixed, atomic path;
- stale, cyclic, ambiguous, unsupported, widened, partial, and over-budget
  proposals deny before effect without residue;
- occupancy never becomes an implicit queue and every conflict outcome is
  typed and bounded;
- the staged batch carries only sealed owner fact/work references;
- compiler code cannot publish, mint a mounted frame, issue/settle host work,
  retain family successors, or reconstruct authority;
- accept/reject settlement reaches every exact participating owner and only
  complete acknowledgement releases occupancy;
- cancellation and shutdown have typed terminal outcomes and exact-zero
  proposal census;
- unused families allocate no owner state and cause no per-frame work;
- no public service facade, DSL grammar, fake Pulse capability, undo/redo, or
  `provisional_aftermath` surface is introduced; and
- focused tests, full runtime proof, three Grok QA gates, Sol-high, and all
  constitutional checks are green on the final tree.

The Phase 3 handoff is deliberately narrow: focus and portal owners may trust
one lawful compiler lifecycle, but they must still implement and prove their
own state, routing, placement, restoration, native mechanics, and Pulse product
behavior.
