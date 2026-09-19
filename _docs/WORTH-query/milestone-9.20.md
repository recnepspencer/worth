# Milestone 9.20: Correlated Paths And Conflict-Proof Set Execution

## Goal

Install typed heterogeneous path programs and execute large batches through
verified conflict partitions and real provider set operations. Correlated
reads remain one admitted graph-shaped plan, and bulk work remains set-oriented
rather than a host loop over scalar operations.

## Roadmap Placement

Milestone 9.19 supplies admitted access products, complete membership, exact
refinement, and verified footprints. This milestone then owns the independent
typed execution-language and bulk-strategy boundary; it is not a continuation
of the Milestone 9.17 composite-branch implementation phases.

## Adversarial Constraint

A provider lowers a typed path into per-binding child queries, omits a negative
dependency, widens traversal, reports host-side sorting as provider ordering,
partitions a batch with quadratic all-pairs planning, processes an unauthorized
member, and reports a scalar loop as one batch. An independent interpreter,
partition oracle, capability oracle, and structural counters must convict it.

## Product Decision Lock

1. Paths are installed typed semantic programs, not strings or opaque
   callbacks.
2. Steps carry entity/relation/aspect identity, direction, cardinality, scope,
   bounds, predicates, projections, and named typed bindings.
3. Provider lowering extends the existing admitted graph-read plan and cannot
   widen semantics, authority, ordering, disclosure, or lifecycle.
4. Correlated execution carries complete path, membership, ordering, and
   negative decision facts.
5. A domain-installed conflict relation or disjointness proof is required.
6. Partitions have canonical identity, complete coverage, uniqueness, and
   conflict freedom.
7. Bulk means provider set work. Host scalar loops do not satisfy the contract.
8. Planning and execution cost are accounted separately.
9. Each partition retains per-item capability, disclosure, purpose, recovery,
   and Milestone 9.18 aftermath posture.
10. Canonical reduction is required wherever provider execution order may vary.

## Destination Topology

```text
worth-query-installation/src/domain_computation/
    path_program/
        contract.rs
        step.rs
        binding.rs
        bounds.rs
    set_execution/
        conflict_contract.rs
        partition_contract.rs

worth-query-admission/src/domain_computation/
    path_plan.rs
    conflict_partition.rs
    set_strategy.rs

worth-query-execution/src/domain_computation/
    correlated_path/
        lowering.rs
        execution.rs
        receipt.rs
    set_execution/
        partitioning.rs
        provider_batch.rs
        reduction.rs

worth-query-certification/src/reference_domains/
    chip_netlist/
    geometry/
    bank_compliance/
worth-query-certification/fixtures/
    consumer_values/  # reuse/extend 9.17.4 and 9.19 value definitions
    consumer_entry/   # reuse/extend their installed contributions
```

Geometry and Bank reference-domain modules own correlated-execution court
orchestration, not duplicate schemas or bindings. Extend the preceding fixture
crates and real Bank entry; share compiled fixtures and the owning integration
target across cases. Chip/netlist adds its distinct domain meaning explicitly.

## Phase Plan

### Phase 1: Typed Correlated Programs

Establish schema-derived path steps, typed cross-step bindings, bounds,
canonical identity, equivalence, and early rejection of illegal composition.

### Phase 2: Admitted Provider Lowering

Lower the path once into the Milestone 9.10/9.19 admitted plan, carry complete
negative and ordering dependencies, and prove exact-zero caller-owned N+1 work.

### Phase 3: Conflict Proof And Canonical Partitioning

Install domain conflict meaning, verify partition coverage, uniqueness, and
conflict freedom, and choose scalar, partitioned, or bulk posture through a
governed strategy transition.

### Phase 4: Provider Set Execution

Execute real set operations with per-partition authority, cancellation,
failure, recovery, aftermath, canonical reduction, and exact structural work
counters.

### Phase 5: Reference Adoption And Certification

Chip/netlist pressures heterogeneous high-fan-out paths and partitions;
geometry pressures topology and batch effects; bank/compliance pressures
mixed-authority bulk work. Public facade, executable documentation, independent
interpreter, parity, slope, no-N+1, authorization, and residue evidence close
the milestone together.

## Performance Contract

- Path work scales with admitted candidate and traversal breadth plus declared
  dependency closure.
- Covered per-binding and per-result child lookups remain exactly zero.
- Partition planning exposes comparisons and must not hide O(n^2) all-pairs
  work behind faster execution.
- Provider contacts, batch width, bytes, conflicts, reductions, and retries are
  separately counted.
- Scalar fallback is explicit, admitted, and reported; it cannot satisfy a bulk
  capability claim.

## Acceptance Evidence

Milestone 9.20 closes when typed invalid paths fail before execution, legal
paths match an independent interpreter, bounds and negative facts hold,
partitions are complete and conflict-free, mixed-authority batches deny or
partition before processing, structural slopes meet their contracts, and all
three reference domains delete replaced local path/batch workarounds.

## Handoff

[Milestone 9.21](./milestone-9.21.md) may attach governed decisions and
summaries to exact path and set executions without becoming path, commit, or
correction authority.
