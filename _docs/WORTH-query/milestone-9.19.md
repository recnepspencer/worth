# Milestone 9.19: Managed Advanced Access And Verified Footprints

## Goal

Add installed-query-bound search and provider-backed access products whose
lifecycle, candidate completeness, negative membership, and realized semantic
footprint are independently proved. The capability must extend the Milestone
9.10 graph-read planner and the Milestone 9.16 ordinary front door rather than
creating a specialist query language or authorization path.

## Roadmap Placement

The completed Milestone 9.17 umbrella and Milestone 9.18 settle exact composite
product-world truth and tree-based semantic aftermath before advanced
computation begins. This milestone is the first advanced vertical slice after
that branch/history foundation and remains independently governed by its access
product and verified-footprint authority.

```text
installed application query
    -> admitted graph-read requirements and strategy
    -> managed provider access product
    -> complete candidates and negative membership
    -> domain-owned exact refinement
    -> verified realized footprint
    -> ordinary execution, commit, and tree-based aftermath
```

## Adversarial Constraint

A provider omits candidates after insertion, deletion, motion, capability
change, and rebuild; leaks a protected candidate through ranking, count, cursor,
or timing-class metadata; understates memory; widens a footprint; and reports
returned rows as complete dependency evidence. Independent full-scan and
resource oracles must expose every lie.

## Product Decision Lock

1. Access products are derived managed resources, never authoritative truth.
2. Every strategy realizes an existing Milestone 9.10 requirement, inventory,
   budget, admitted plan, and receipt contract.
3. Search retains the installed query's identity, parameters, root, basis,
   ordering, result shape, capability, purpose, disclosure, continuation,
   recovery, and aftermath meaning.
4. Candidate production and exact refinement are distinct contracts.
5. Completeness includes positive, negative, boundary, absence, cardinality,
   authorization, disclosure, and conflict-sensitive membership.
6. Protected facts may influence internal computation only under separately
   admitted noninterference and disclosure posture.
7. Created, ready, maintaining, stale, evicted, rebuilding, disposed, and
   failed are explicit lifecycle states.
8. A realized footprint is verified from actual execution and dependency
   evidence. Caller or provider assertion grants no authority.
9. Declared authority is an upper bound. A verified footprint may narrow but
   never widen it.
10. Verified footprints feed dependency, invalidation, conflict, reuse,
    publication, commit, and Milestone 9.18 correction applicability.

## Destination Topology

```text
worth-query-installation/src/application_query/access_strategy/
    contract.rs
    graph_read_requirement_binding.rs
    search.rs
    spatial.rs
    membership.rs

worth-query-execution/src/domain_computation/application_query/access_strategy/
    admission.rs
    managed_product.rs
    candidate_delivery.rs
    exact_refinement.rs
    coverage.rs
    realized_footprint.rs
    continuation.rs

worth-query-publication/src/application_query/
    search_result.rs
    access_product_evidence.rs
    verified_footprint.rs

worth-query-certification/src/reference_domains/
    bank_compliance/
    geometry/
worth-query-certification/fixtures/
    consumer_values/  # extend 9.17.4's Query-free values
    consumer_entry/   # extend 9.17.4's explicit entry contributions
```

Reference-domain modules orchestrate advanced courts over the 9.17.4 fixture
crates and Bank's real entry where applicable. They do not redeclare those
schemas or value/binding identities. Cases share the owning integration target
and compiled fixture dependencies rather than creating a binary per scenario.

## Phase Plan

### Phase 1: Managed Access Products

Install typed strategy declarations, graph-read-plan binding, provider
admission, resource accounting, move-only handles, lifecycle transitions, and
cleanup under cancellation, yield, failure, rebuild, and disposal.

### Phase 2: Coverage And Membership Completeness

Prove exact covered scope and basis, candidate completeness or explicit
best-effort posture, negative membership, invalidation, maintenance/rebuild
policy, and protected-fact noninterference.

### Phase 3: Verified Realized Footprints

Derive the actual influencing subset from execution, read-set, membership,
effect, and artifact evidence; mint `VerifiedRealizedFootprint` only after
Query verification; consume it in conflict and commit progression.

### Phase 4: Bank And Geometry Adoption

Replace host-local bank search and geometry spatial-index workarounds through
the public Query facade. Bank pressures capability/disclosure/ranking/cursor
meaning; geometry pressures candidate completeness, motion, exact refinement,
and narrow footprints.

### Phase 5: Facade, Documentation, And Hostile Certification

Publish access-product, search, membership, lifecycle, and footprint guidance
with executable examples. Independent full-scan, memory, mutation,
noninterference, receipt, no-N+1, lifecycle, facade, and residue courts must
close before the feature is advertised.

## Performance Contract

- Build, lookup, refinement, maintenance, rebuild, eviction, disposal, and
  resident-memory work have separate counters.
- Search universe width, provider candidates, disclosure-eligible candidates,
  ranked candidates, and delivered results remain distinct.
- Candidate count is distinct from exact-refinement count and realized
  footprint width.
- Ordinary execution imports no full-scan certification cost.
- Covered per-result neighbor lookups remain exactly zero.

## Must Preserve

- Milestone 9.10 planning and no-N+1 authority;
- Milestone 9.16 capability, purpose, disclosure, continuation, and recovery;
- Milestones 9.17 and 9.18 composite branch and correction semantics;
- provider ownership of physical mechanics; and
- domain ownership of search meaning, exact predicates, and tolerances.

## Acceptance Evidence

Milestone 9.19 closes only when bank and geometry use the real public facade,
local replacements are deleted, membership remains complete across lifecycle
change, footprints narrow without omitting influence, protected candidates do
not leak, provider work is independently counted, and alternate access-product
implementations converge on the same semantics.

## Handoff

[Milestone 9.20](./milestone-9.20.md) consumes admitted access strategies,
complete membership, and verified footprints to execute correlated paths and
large conflict-safe sets without reopening candidate or authority meaning.
