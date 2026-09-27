# Milestone 9.15: Managed Domain Computation, Proposed State, And Invariant Execution

**Status:** Complete through Phase 10.

## Goal

Make installed domain computation safe to prepare and evaluate before commit.
An installed operation can carry governed intermediate products, run under real
resource pressure, bind provider work to one attempt, retain the complete basis
of its decisions, construct proposed post-state, and execute installed
invariants against that state.

This milestone deliberately stops before public application authoring,
authentication and authorization, and compare-and-commit. Those capabilities
are governed by [Milestone 9.16](./milestone-9.16.md). Advanced access products
and realized footprints are governed by
[Milestone 9.19](./milestone-9.19.md); correlated paths and bulk conflict
partitioning by [Milestone 9.20](./milestone-9.20.md); governed decision
evidence by [Milestone 9.21](./milestone-9.21.md); and occurrence-safe reuse by
[Milestone 9.22](./milestone-9.22.md).

## Roadmap Placement

Milestone 9.14 established the installed operating world and the authority
progression that binds domain declarations to runtime execution. Milestone 9.15
extends that foundation through the last honest pre-commit state:

```text
installed operation
    -> admitted execution resources
    -> managed run
    -> provider-bound attempt
    -> basis-complete decision read-set
    -> proposed post-state
    -> executed invariant result
```

The milestone does not claim that evaluated state has committed. Only the
provider-bound compare-and-commit progression introduced in Milestone 9.16 may
cross that boundary.

## Governing Laws

This milestone is bound by the repository engineering constitution:

- `MENTALITY.md`: build the load-bearing authority and lifecycle before richer
  application surfaces.
- `arch_laws.md`: phase transitions are represented by types; public authority
  cannot be reconstructed from data or caller assertion.
- `composition_laws.md` and `domain_structure_laws.md`: physical ownership
  follows semantic ownership and no facade hides a second authority path.
- `perf_laws.md`: ordinary execution carries proof forward; reconstruction and
  certification costs stay out of warm paths.
- `testing_laws.md`: evidence crosses real boundaries and does not trust the
  implementation's own receipts as its oracle.
- `dx_laws.md`: a consumer sees a coherent progression rather than construction
  details for lower authorities.

## Adversarial Constraint

A long-running installed operation carries typed intermediate products, is
admitted against finite resources, performs actual incremental provider work,
yields under pressure, resumes only through the same managed run authority,
records both present and absent facts that affected its decision, constructs a
proposed graph state, and runs a real installed invariant over that exact
state.

Cancellation, exhaustion, provider failure, invariant denial, and cleanup may
occur at every legal boundary. No path may:

- turn a resource estimate into consumed-capacity proof;
- use a caller-created support snapshot to mint executable authority;
- fabricate a resume token or continue an abandoned run;
- confuse an artifact value with its production occurrence;
- erase negative or membership-sensitive decision facts;
- present declared invariant obligations as executed invariant results;
- mutate authoritative graph state while constructing proposed state; or
- present prepared or invariant-approved work as committed work.

## Product Decision Lock

1. Query owns installed contracts, admission, execution phase progression,
   managed handles, derived evidence, and the public meaning of outcomes.
2. Domain packages own artifact meaning, reproducibility, search and
   convergence semantics, counters, decision schemas, and invariant semantics.
3. Providers own physical allocation, reads, proposed-state mechanics, and
   invariant execution mechanics behind admitted Query contracts.
4. An artifact identity, a production or acquisition occurrence, and an
   independent-certification occurrence are distinct concepts.
5. Runtime-affine managed artifact handles are move-only authority. A public
   identifier, digest, schema, or receipt cannot reconstruct the handle.
6. Transformation and loss evidence is derived. It cannot admit work, publish
   truth, or resolve a governed decision.
7. Resource support is live consumable capacity with reservation and release,
   not a caller-provided static comparison record.
8. A yielded run retains only the authority and resources its lifecycle
   explicitly permits. Resume is an in-memory continuation of that run, not a
   durability or restart claim.
9. Provider execution plans and session identities are sealed products of the
   installed-operation progression. Public callers cannot assemble their
   ingredients into an attempt.
10. A decision read-set records every fact class whose truth influenced the
    decision, including relevant absence, predicates, membership, cardinality,
    ordering, traversal, artifact, and structural facts.
11. Proposed state is isolated, inspectable, and disposable. It is not
    authoritative state.
12. An invariant result proves actual execution over an exact proposed-state
    identity and provider session. A schema, declaration, selection, or receipt
    alone is not execution evidence.
13. Governance metadata is enforced where contracts are admitted and where
    evidence is disclosed. Restricted nested records cannot be weakened by a
    more permissive container.
14. Structural counters compose only when their installed units and aggregation
    laws are compatible.
15. All public progressions preserve the authority established by Milestone
    9.14; no compatibility lane may accept caller-constructed equivalents.

## Destination Topology

The completed implementation distributes responsibility across the authority
packages introduced by Milestone 9.13.2.

```text
worth-query-decl
    public domain declarations and consumer-safe types

worth-query-installation
    artifact, occurrence, reproducibility, search, convergence,
    transformation, counter, decision, and invariant contracts

worth-query-admission
    installed-operation-bound resource and execution admission

worth-query-execution
    managed artifacts, runs, provider sessions, read-sets,
    provisional attempts, proposed state, and invariant execution

worth-query-publication
    disclosure-governed derived evidence

worth-query-cert
    hostile evidence for the completed pre-commit progression

worth-query-host
    narrow host-facing composition over those authorities
```

No package may recover a later authority by importing representation types from
an earlier package. Audience facades expose legal use, not construction seams.

## Completed Phase Plan

### Phase 1: Installed Domain Artifact Contracts

**Requirement**

Install schema-versioned artifact families whose semantic identity and allowed
operations are declared by a domain package without placing payload meaning or
domain vocabulary in Query.

**Implementation boundary**

- Installation owns canonical artifact-family admission.
- Execution owns runtime-affine managed handles and lifecycle.
- Providers own allocation and payload access.
- Publication exposes only disclosure-governed evidence.

**Required proof**

- Unknown schemas, incompatible versions, and forged handles fail before use.
- Large payloads move across stages without an untyped blob or side store.
- Dropping, cancellation, and failure release owned provider resources once.

**Next trust established**

Later phases may carry a governed artifact without learning or reconstructing
its physical representation.

### Phase 2: Artifact Ownership, Occurrence, And Reproducibility

**Requirement**

Separate semantic equality from the facts that a product was produced,
acquired, or independently observed. Install the reproducibility contract that
governs when one occurrence may substitute for another.

**Implementation boundary**

- Domain installation declares exact, seeded, canonical-reduction,
  comparator/bound, distributional, or observational posture.
- Execution records occurrence lineage without treating equal content as the
  same event.
- Certification purpose is explicit; cache reuse cannot manufacture
  independent evidence.

**Required proof**

- Equal content does not collapse required independent occurrences.
- Byte-different results are equivalent only through the installed comparator
  or bound.
- Provider changes, reduction order, seeds, cache hits, and repeated
  observations cannot silently strengthen reproducibility.

**Next trust established**

Later execution and reuse can reason honestly about both product meaning and
how the product came to exist.

### Phase 3: Native Access, Search, Convergence, And Transformation Evidence

**Requirement**

Provide bounded native artifact access and governed evidence for candidate
searches, single-basis convergence, transformations, correspondence, and loss.

**Implementation boundary**

- Native access reports actual chunk, allocation, resident-memory, and work
  posture, including heap-owned scalar payloads.
- Candidate search exposes universe, termination, completeness, feasibility,
  comparison, incumbent, optimality, and cost posture.
- Convergence distinguishes progress, stability, feasibility, oscillation,
  exhaustion, cancellation, and indeterminacy.
- Transformation evidence retains source/output occurrence and correspondence
  cardinality without gaining admission or publication authority.

**Required proof**

- Variable-width scalar families are included in memory accounting.
- An incomplete or heuristic search cannot claim uniqueness or optimality.
- Stable, feasible, converged, oscillating, exhausted, and cancelled outcomes
  remain distinct.
- Loss and repair evidence cannot promote derived output to authoritative
  truth.

**Next trust established**

Managed execution may consume complex domain products while retaining honest
cost and epistemic posture.

### Phase 4: Structural Counters And Decision Evidence

**Requirement**

Install domain-defined structural counter and decision-record schemas with
enforced units, aggregation, classification, retention, and disclosure.

**Implementation boundary**

- Core operational evidence remains separate from policy-materialized sidecars.
- Counter aggregation requires installed unit compatibility or an explicit
  conversion law.
- Nested decision governance may only maintain or strengthen its containing
  artifact's disclosure posture.

**Required proof**

- Cycles, missing required counters, and unit-incompatible aggregation reject.
- Restricted decision records do not escape through permissive artifact
  containers.
- Redaction, retention, and classification are enforced rather than descriptive.

**Next trust established**

Later resource and invariant phases may publish meaningful evidence without
inventing domain counters or leaking governed decisions.

### Phase 5: Real Execution Resource Admission

**Requirement**

Admit work against live multi-axis capacity and reserve that capacity through a
managed lifecycle.

**Implementation boundary**

- Installed operation authority determines which resource contract is eligible.
- Admission owns reservation, queue/backpressure posture, and release.
- Execution consumes the admitted reservation; support records alone carry no
  authority.

**Required proof**

- Caller-constructed contracts or support snapshots cannot start attempts.
- Concurrent arrivals produce actual saturation and backpressure.
- Capacity is consumed, released once, and made available to later work.
- Denial and degradation are typed and cannot be treated as full admission.

**Next trust established**

The runtime may begin a managed run knowing its resource authority represents
real capacity rather than an estimate.

### Phase 6: Managed Run Lifecycle

**Requirement**

Advance admitted work through a cancellable, yielding, in-memory managed run
whose continuation cannot escape its authority or semantic basis.

**Subphases**

1. **6.1 — State and ownership:** establish run identity, admitted resources,
   safe points, and cleanup ownership.
2. **6.2 — Progress and cancellation:** derive progress and cancellation from
   actual incremental provider work.
3. **6.3 — Yield and continuation:** yield only at declared safe points and
   issue a sealed continuation for the same live run.
4. **6.4 — Readmission and exhaustion:** reacquire eligible capacity, preserve
   bounded work, and distinguish exhaustion from cancellation or degradation.
5. **6.5 — Cleanup and terminality:** make completion, abandonment, denial, and
   failure terminal; release every resource exactly once.

**Non-goal**

This is not restart, durable resume, checkpoint persistence, or recovery. It is
ordinary in-memory lifecycle control for a live runtime process.

**Required proof**

- Repeated yield/readmission does not duplicate work or widen authority.
- Cancellation works at every safe point and leaves no promotable continuation.
- A continuation from another runtime, run, operation, or basis fails.
- Exhaustion, cancellation, failure, and degradation remain distinct.

**Next trust established**

A provider-bound attempt can inherit one controlled execution lifecycle.

### Phase 7: Sealed Provider Plan And Session

**Requirement**

Bind provider work to one installed operation, admitted run, semantic basis,
and graph authority before reads or effects can begin.

**Implementation boundary**

- Query admission creates the sealed execution plan from installed authority.
- Execution opens a provider session from that plan and the managed run.
- The provider session binds reads, proposed state, invariant execution, and
  eventual terminal handling.

**Required proof**

- Public construction of plan ingredients opens no executable path.
- Cross-runtime, cross-operation, cross-basis, and cross-graph mix-and-match
  attempts fail.
- A closed or abandoned session cannot be reused.

**Next trust established**

Every decision fact and proposed-state operation can be tied to the provider
session that actually observed or produced it.

### Phase 8: Basis-Complete Decision Read-Sets

**Requirement**

Retain the complete semantic basis used by a domain decision, not merely the
rows returned to the caller.

**Required fact families**

- positive entity, relation, and aspect observations;
- relevant absence and predicate outcomes;
- membership, cardinality, and ordering facts;
- traversal and boundary facts;
- artifact and access-product dependencies where used; and
- structural counters or installed decision evidence that affected the choice.

**Required proof**

- Insertions into previously empty or negative space are represented.
- Membership and ordering changes cannot hide behind unchanged returned rows.
- Facts are session- and basis-bound and cannot be replayed into another
  attempt.
- Read-set completeness is tested with an independent mutation oracle.

**Next trust established**

The prepared attempt has enough causal evidence for a later commit phase to
distinguish relevant drift from unrelated change.

### Phase 9: Provisional Attempt And Proposed Post-State

**Requirement**

Construct, inspect, and discard a provider-backed proposed graph state without
mutating authoritative state.

**Implementation boundary**

- Query owns the attempt progression and proposed-state identity.
- The provider owns physical staging and exact proposed-state inspection.
- Domain invariant code receives only the installed, bounded inspection
  capability for the proposed state.

**Required proof**

- The pre-state remains unchanged before commit.
- Proposed state cannot be inspected outside its session and attempt.
- Abort, cancellation, or failure disposes all provisional resources.
- A proposal cannot be published, shared as truth, or passed off as a committed
  snapshot.

**Next trust established**

Installed invariants can evaluate the exact state that would be committed.

### Phase 10: Real Invariant Execution

**Requirement**

Execute installed blocking invariants over the exact proposed post-state and
return proof tied to the provider session, attempt, invariant contract, and
proposed-state identity.

**Implementation boundary**

- Domain packages define invariant meaning.
- Installation admits invariant contracts and their required inspection
  capabilities.
- Providers execute through the bounded proposed-state inspection surface.
- Query owns progression and the typed satisfied, violated, failed, cancelled,
  or indeterminate result.

**Required proof**

- Selecting or declaring an invariant cannot mint an execution result.
- A result for another proposal, session, basis, or invariant is unusable.
- Violations block later commit admission.
- Provider failure and indeterminacy cannot masquerade as satisfaction.
- Hostile invariants prove they see proposed state rather than stale pre-state
  or already-mutated authoritative state.

**Next trust established**

Milestone 9.16 may build compare-and-commit from a real evaluated proposal
without inventing invariant authority or reopening the pre-commit model.

## DX Target

The completed pre-commit progression should read approximately as follows. The
names are a semantic target, not a promise that every internal type is publicly
constructible.

```rust
let admitted = runtime
    .admit(installed_operation, execution_request)
    .await?;

let mut run = admitted.start().await?;
let session = run.open_provider_session(graph).await?;

let decision = session
    .evaluate(|facts, artifacts| domain_operation(facts, artifacts))
    .await?;

let proposal = session.propose(decision.effects()).await?;
let evaluated = proposal.run_required_invariants().await?;

match evaluated {
    EvaluatedProposal::Satisfied(prepared) => prepared,
    EvaluatedProposal::Violated(violations) => return Err(violations.into()),
    EvaluatedProposal::Indeterminate(reason) => return Err(reason.into()),
}
```

Nothing in this target implies commit. The `prepared` value is useful because
it carries the proof needed by Milestone 9.16, not because it can mutate state
on its own.

## Must Ship

- installed artifact, occurrence, reproducibility, search, convergence,
  transformation, structural-counter, decision, and invariant contracts;
- move-only runtime-affine managed artifact and continuation authority;
- native access with honest variable-width memory and work accounting;
- enforced counter units and nested disclosure governance;
- live resource reservation, saturation, backpressure, release, cancellation,
  yielding, readmission, exhaustion, degradation, and cleanup;
- sealed provider execution plans and sessions;
- basis-complete decision read-sets;
- isolated provisional attempts and exact proposed-state inspection;
- real installed invariant execution with typed terminal posture; and
- consumer and hostile certification evidence for every transition above.

## Must Preserve

- one installed operating-world root and the authority progression from
  Milestone 9.14;
- domain-agnostic Query grammar;
- provider ownership of physical state and mechanics;
- exact Foundational value meaning;
- ordinary versus certification/reconstruction cost separation;
- cert-only replay boundaries; and
- the rule that derived artifacts and evidence carry no authoritative truth or
  commit power.

## Explicit Non-Goals

The following are not incomplete 9.15 work:

- public typed aspect authoring;
- authentication, authorization, permission evaluation, or touched-graph
  policy;
- compare-and-commit or any committed mutation;
- ordinary application read, mutation, workflow, live, or HTTP facades;
- managed domain access products and membership maintenance;
- verified realized footprints;
- correlated heterogeneous path programs;
- conflict partitions and set-oriented bulk execution;
- decision attachments and incremental summaries;
- stage/subartifact reuse, eviction, and rebuild; and
- geometry, netlist, or research provider certification.

Their governing homes are Milestone 9.16 and Milestones 9.19 through 9.22.

## Acceptance Evidence

Milestone 9.15 is closed only if the implementation and its evidence ledger
jointly prove:

- every Phase 1–10 requirement has an implementation owner, independent
  evidence, and closed failure posture;
- authority-construction, phase-skipping, and mix-and-match probes fail;
- resource saturation is produced by concurrent demand rather than a fabricated
  insufficient snapshot;
- heap-owned native values participate in peak-memory accounting;
- decision governance and counter-unit compatibility are enforced;
- negative and membership-sensitive read facts survive into the read-set;
- proposed state remains isolated from authoritative state;
- selected invariants actually execute against the exact proposal;
- cancellation, failure, exhaustion, degradation, and invariant denial clean up
  without widening authority; and
- boundary checks, context checks, facade checks, residue scans, and relevant
  warm-path tests are green.

The closure evidence is maintained in the phase QA ledgers, including
[the Phase 2–5 ledger](./milestone-9.15-phases-2-5-qa-ledger.md), and in the
authority-local and consumer certification suites.

## Handoff

[Milestone 9.16](./milestone-9.16.md) consumes an invariant-evaluated proposal
and proves that a real authenticated asynchronous application can reach an
atomic outcome through the ordinary Query front door.

[Milestones 9.19 through 9.22](./milestone-9.19.md) consume that honest public
front door and add advanced access, set execution, governed decision evidence,
and reuse without reopening the authority model completed here. Milestones
9.17 and 9.18 first settle composite product-branch/history truth and tree-based
semantic correction.
