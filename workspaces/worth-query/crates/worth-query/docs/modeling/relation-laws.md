# Relation Laws

## What This Feature Is

A relation law is declared graph meaning enforced whenever an application
commits a change. Declaring `.acyclic()` means that following that relation from
source to target must never return to the starting entity. Query lowers the
declaration into Relational's native commit validation.

Paired symmetry requires the reverse of each surviving directed edge on the
same relation. Scoped uniqueness limits physical records per ordered pair or
across both directions of an unordered pair. Partition isolation requires both
endpoints of every surviving edge to belong to the same native partition.
A reachability minimum obligates every entity of the relation's source kind to
reach enough distinct entities of selected target kinds.

## Why You Use It

Use acyclicity for dependency edges, revision ancestry or a feature history.
Every writer must preserve the same rule, including operations that add several
edges in one commit. The application declares the rule once alongside the
relation's endpoint, cardinality and deletion policies.

## Stable Entry Points

Import `ApplicationRelationIntegrity` and `ApplicationRelationRef` through
`worth_query_decl::facade::application_schema`. Call the const `.acyclic()`
builder on the integrity value passed to your relation reference, then register
that reference through the schema contribution's ordinary `.relation(..)`.

The same const builder exposes `.symmetric()` and
`.unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge)` or
`.unique(ApplicationRelationUniquenessScope::NormalizedSymmetricEdge)`.
Use `.same_partition_endpoints()` to declare endpoint partition isolation, and
`.reaches_at_least(minimum, kinds)` for a distinct reachable-target minimum.
`kinds` is a static slice of authored entity names. Obtain a name from your typed
entity reference with `.name()`, or use its declared name directly. The completed
schema rejects zero minima, empty kind sets and unknown entity names.

Use `.orbit()` on a stored homogeneous relation to require exactly one
successor and one predecessor for every live member of that kind. The completed
schema rejects different source and target kinds. The law remains conjunctive
with the relation's endpoint, cardinality and deletion policies.

For refusals, use `worth_query_host::facade::primary_graph`:
`WorthQueryApplicationCommitDenial::relation_law()` returns an optional borrowed
`WorthQueryRelationLawDenial`. Bootstrap publication refusals expose the same
view on `WorthQueryPrimaryGraphInstallationDenial`.

## Core Mental Model

The declaration belongs to the relation. A handler supplies candidate graph
changes; it does not decide whether those changes satisfy the law. Relational
checks the complete proposed graph, including every edge added by the same
candidate and the edges already present in its basis.

The existing integrity constructors enable none of these laws. Each builder
preserves the other laws and endpoint, cardinality and deletion policies. Permitting self
endpoints does not exempt a self-edge from acyclicity: a self-edge is a cycle.

For paired symmetry, a permitted self-edge is its own reverse. For distinct
endpoints, both directed records must survive. Add or remove both in one
candidate. The reverse must satisfy the same relation's endpoint contract, so
use homogeneous endpoints such as `Node → Node`. The builder does not declare
another inverse relation or create a reverse edge for you.

Directed uniqueness permits one `A → B` record and one `B → A` record. Normalized
uniqueness counts them together and refuses that pair. Consequently, pair
`.symmetric()` with directed uniqueness when both inverse records should exist.

Partition isolation compares the endpoints, not the partition storing the
relation record. Ordinary `CandidateWriter::create_entity` places new entities
in the mutation's issued partition. To create beside an existing projected
entity, use `CandidateWriter::create_entity_in_context(&context, entity, key)`.
The context must come from the operation's genuine decision and candidate
authority. A key or diagnostic partition number cannot substitute for it.

A reachability minimum follows this same relation from source to target. It
counts each reachable target once, however many records or paths reach it. The
starting source never counts itself, even through a cycle or a self-edge.
Every visible source-kind entity is obligated, including a newly created or
isolated source. A target of another kind does not acquire that obligation.
Deleting a source removes its obligation; deleting a target removes it from
the reachable set, even when an audit edge is retained.

A nonempty finite `Node → Node` graph cannot combine acyclicity with a positive
minimum: acyclicity leaves a sink, and the sink has no reachable target. Use a
relation such as `Node → Leaf` when terminal targets should not be obligated
sources. Bootstrap publication must also provide a lawful graph; isolated
sources cannot be published with the intention of linking them later.

Minimum kind names form a normalized set for schema and program identity:
ordering and duplicate names do not change the meaning. Adding a different
kind or changing the minimum does.

Law meaning participates in canonical schema and program identity. Changing
the law is a schema/program change; reusing the same names does not authorize
an installed application to acquire different meaning on restore.

An orbit is a permutation, so initial publication must include every member's
successor and predecessor. An isolated new member refuses; creating it with a
self-edge is lawful when the independently declared endpoint policy allows
self-edges. Duplicate physical edges violate the exactly-one constraint even
when they connect the same pair. Rewire all affected successors in one mutation
to split or merge cycles. These changes preserve the permutation law.

The stored orbit law also enables platform-owned committed aggregate identity
and lineage records. Birth identity depends on the definition, commit occurrence
and least member at birth; removing that member does not rename a whole orbit.
On a split, the part containing the least surviving old member keeps its identity.
On a merge, the participant with the least current anchor keeps its identity.
Mixed rewiring conceptually splits first, then compares the incoming old fragments.
If the winning fragment does not contain its old identity's least surviving member,
the result gets a newly minted final identity; it cannot take the keeper part's
identity. Split and merge events name the actual final identities and commit
occurrence. Applications cannot write these private records.

Installed invariant planner/context views expose `orbit(&binding, &member)` and
`orbit_of(&binding, &member)` for the actual declared orbit relation. The orbit
retains Native identity truth and complete business topology; `members()` yields
successor order from its current least member, `len()` reports its size, and
`identity()` returns its persistent typed name. Both methods admit every member
and physical successor edge through the ordinary prepared scope. A name's bytes
grant no read or write access.

The compiled [topology invariant](../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_invariant/preparation.rs)
uses ordered orbit windows and processes each affected orbit once. Its former
successor lookup is deleted. Application read observations describe business
walk/field acquisitions; private Native identity preparation is outside them.
Ordinary projection readers also expose `reader.orbit(Next::reference(), &member)`
and `orbit_of(..)`. The result retains actual Native aggregate and complete walk
owners from the selected snapshot. `members()`, `member(index)`, `relations()`,
`identity()` and `len()` borrow that materialized result without graph reads.
Operation readers require `OperationReads` for the relation and admit every
returned member through the original projection's realized scope. These are
inspection reads; they do not record publication decision dependencies. The
compiled [projection examples](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_orbits/projection.rs)
exercise ordinary and operation readers, including a retained basis after an edit.
Handler `DecisionReader::orbit` / `orbit_of` and computation readers record two
original Native facts: persistent identity with complete canonical membership,
and the complete successor walk with physical edges. Lower-level admitted
operation readers call `decision_orbit` / `decision_orbit_of`. Both values remain
bound to their original admission, installed contracts and realized scope; a
selector alone cannot reconstruct them at sealing. Fresh comparison detects
identity changes even when membership is restored by later edits. Incidental
stored lineage is outside orbit-answer equality. The compiled
[handler](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_orbits/decision/handler.rs)
and [currentness court](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_orbits/decision/currentness.rs)
exercise this path. Aggregate inputs currently require full verification and a
fresh computation; live fact bodies cannot be reconstructed from checkpoints.

A declared `Many` query can select one complete orbit with
`builder.orbit(ApplicationQueryOrbitRef::new("loop", NextFin::reference()))`.
The query's root and scope must be the same entity kind. Its returned rows stay
in successor order from the Native anchor. `result.orbit(loop_slot())` returns
the exact declared slot's `identity()`, `len()` and borrowed `members()`; the
getter performs no graph read and survives conversion into a disclosed result
or output-demand source. The selector has schema, query, slot, relation and
entity marker types; see the compiled [query declaration and result court](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_orbits/declared_queries.rs).

Whole-orbit metadata is one disclosure slot. A governed contract must provide
`orbit_by(slot, disclosure_value, influence)` for identity, length, membership
and order together; member fields keep their own disclosure rules. An omitted
orbit slot refuses before group selection. Filtering, root-path unions,
explicit ordering, continuation and live lanes cannot turn a complete orbit
into a partial answer. Row and result-set sources retain the original Native
aggregate and complete walk; portable record grammar 17 preserves this meaning
and refuses older record grammars.

Lineage reads and Proprietary adoption remain unfinished in Phase 4. Generic Native branch reconciliation has
separate existing execution limitations and retains its typed refusal.
The [installed orbit court](../../../worth-query-certification/tests/application_graph/relation_laws/orbit.rs)
exercises native refusals and atomic lawful mutations through the ordinary
authenticated program.

An orbit's cardinality refusal exposes
`WorthQueryRelationLawDenial::Cardinality` with the original `member`,
`boundary`, `count`, `limit` and `contract_id` (`relation:orbit`).
`RelationCardinalityBoundary` is available through the host's
`application_invariants` facade. Another independent policy may refuse first
and retains its original cause, such as `SelfEdge` for a forbidden singleton.

## How It Executes

1. Author the relation and completed schema, then validate the program with
   `ApplicationProgramAuthoring::begin().with_schema(&schema).validated_program()`.
2. Install through the ordinary program installation path.
3. Authenticate a request and run its declared operation. The handler reads
   through `DecisionReader` and writes through `CandidateWriter` as usual.
4. Native commit validation checks the relation's proposed edges. A law
   violation refuses publication and leaves committed graph truth unchanged.

At each execution point, native checks run and reduce before an installed
application invariant prepares its custom scope. A native failure that blocks
that point prevents custom preparation, so a declared acyclicity law can protect
a commit-boundary custom rule that follows the same relation. Successful native
checks remain in the combined result alongside custom checks. Native advisory
evidence permits custom preparation; a harness audit collects both stages.

The guarantee applies at the same execution point. A publication-only law does
not establish a precondition for an earlier callback. Administrative admission,
including refusal of an unchecked custom kernel under a request admitted for
parallel execution,
still precedes native work.

Cancellation and resource stops keep their execution causes. An interrupted
check does not become a successful law verdict or a domain law violation.

## Small Example

The associated integrity constant in the compiled graph example enables
acyclicity while retaining the existing same-context endpoint and audit
deletion policies:

<!-- compiled: relation_law_declaration.rs -->
```rust
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().acyclic();
```

Its relation reference passes this value as the final argument of
`ApplicationRelationRef::from_schema_identifiers("Precedes", "Node", "Node", integrity)`.
The schema contribution registers `Precedes` from `Node` to `Node`.

To require a physical reverse record and forbid same-direction duplicates,
the compiled pair example uses this declaration:

<!-- compiled: relation_pair_declaration.rs -->
```rust
const PAIRED_UNIQUE: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .symmetric()
        .unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
```

Register that integrity value on the same relation reference. Deleting only
`A → B` then refuses because `B → A` has lost its reverse; deleting both in one
candidate succeeds. With normalized uniqueness instead, proposing both
directions refuses even when each direction has only one record.

To isolate endpoint partitions while allowing the endpoint contract to reach
that law, the compiled partition example declares:

<!-- compiled: relation_partition_declaration.rs -->
```rust
const EXPLICIT_CONTEXTS: ApplicationRelationIntegrity = ApplicationRelationIntegrity::new(
    ApplicationRelationEndpoints::new(true, ApplicationRelationCrossContextPolicy::AllowExplicit),
    ApplicationRelationCardinality::unbounded(),
    ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
);
const PARTITION_LOCAL: ApplicationRelationIntegrity = EXPLICIT_CONTEXTS.same_partition_endpoints();
```

Register `PARTITION_LOCAL` on the relation. Linking an existing node to a new
node created in an issued partition refuses when their partitions differ.
Creating the new node in that existing node's context allows the same edge.
The earlier endpoint policy can also forbid cross-context edges independently;
its refusal keeps that earlier cause.

To require at least two distinct reachable nodes, the compiled minimum example
uses the authored entity name:

<!-- compiled: relation_minimum_declaration.rs -->
```rust
const MINIMUM_TWO: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .reaches_at_least(2, &["Node"]);
```

Register this integrity on `Node → Node`. A three-node directed cycle satisfies
it: each source reaches the other two nodes. A two-node cycle reaches only one
other node per source, regardless of duplicate records or repeated walks.

For `Node → Leaf`, declare the same minimum with `["Leaf"]`. Replacing two old
links with one new Leaf refuses the entire proposal; replacing them with two
distinct new Leaves can commit. A live Principal elsewhere in the graph cannot
satisfy the minimum merely because its kind is also listed: it must actually be
reachable through this relation. The complete proposed record state is judged
before publication, including target creation and removal in the same mutation.

## Real Example

For a feature history, let `Precedes` connect one node to its next node.
After `A → B` commits, proposing `B → A` refuses. Proposing both `A → B` and
`B → A` together also refuses; neither edge becomes committed. A lawful next
step `B → C` can still commit.

This compiled request method runs that history edit through the ordinary
authenticated program entry. `EdgeEdit` is the operation's structured input
(a serial, added source/target node keys and removal pairs); `Edit<L>` is its declared
mutation intent. The enclosing application wrapper owns the installed runtime.
The operation declares reads of `NodeKey` and links of `Precedes`. Its handler
resolves those keys through its decision reader and links the resulting
projected entities through its candidate writer.

For removals it first reads the actual complete outgoing adjacency, then uses
`CandidateWriter::unlink` for observed matches. It retains absence reads while
skipping absent removal effects and coalesces repeated removal commands. Those
steps normalize the command; physical edge multiplicity remains intact.

<!-- compiled: relation_law_request.rs -->
```rust
    pub fn apply_edit_in_scope(
        &self,
        input: EdgeEdit,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<GraphDenial, Edited>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let serial = input.serial;
        let principal =
            authenticate_operator(self.application.installed_schema(), &request_scope());
        self.application
            .runtime()
            .request(&principal, scope)
            .on_branch(self.application.current_world())
            .advanced()
            .mutate(Edit::<L> {
                input,
                marker: PhantomData,
            })
            .without_source()
            .idempotency(&serial)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
    }
```

The authentication helper uses the installed principal mapping and a fresh
authentication scope. The supplied `scope` remains the original execution
scope, so a cancelled caller cannot publish through this wrapper. No handler
cycle walk or custom invariant is needed for the declared law.

## How It Relates To Other Features

Endpoint and cardinality contracts still apply independently. A relation that
also forbids self endpoints may refuse a self-edge at that earlier contract.
Use an [installed application invariant](../foundations/invariants-and-read-observations.md)
for domain conditions beyond the declared law vocabulary. Ordinary query and
decision reads retain their own authorization and tracking rules.

## Inspection And Debugging

On an uncommitted mutation, inspect the commit denial's `relation_law()`.
The `Acyclicity` variant borrows the native `contract_id`, relation `KindId`,
and `source`/`target` `EntityReference` values for the candidate edge that
closes the cycle. References can name existing entities or entities created
by the same candidate. They describe the original violation and grant no read
or mutation authority. Keep the owning denial alive while borrowing them.

`Symmetry` identifies the surviving edge without its required reverse and
carries the native symmetry mode. `Uniqueness` carries the duplicate pair,
native uniqueness scope and physical count. `PartitionIsolation` borrows the
original source and target references whose native partitions differ.
`ConnectivityMinimum` borrows the original source and reports
`reachable_target_count` and `minimum_reachable_targets`. An unpublished source
retains its creation reference. These counts describe the violated law, not
read-cost observations. The same typed view is available when bootstrap
publication refuses. Diagnostic `ContractId`, `KindId`,
`EntityReference`, `SymmetryMode` and `UniquenessScope` are nameable through
`host::application_invariants`; none grants runtime authority.

Law checks do not replace other admission rules. In particular, two create or
endpoint-update commands with identical partition, relation kind and ordered
endpoints conflict at native proposal identity admission, before law
evaluation. Such a refusal keeps that earlier cause and need not carry a
relation-law view. Distinct physical records already present plus a newly
proposed duplicate still reach the installed uniqueness law.

The view is optional: other refusal families, including execution stops, have
their existing typed causes. Match future law variants with a fallback; the
law-denial enum is non-exhaustive. Do not parse diagnostic strings to recover
identities or assume a particular edge wins when several close cycles.

## Anti-Patterns

- Implementing a second cycle checker in every operation handler.
- Checking only the committed graph when judging a multi-edge candidate.
- Treating a diagnostic entity identity as an authorized runtime handle.
- Treating cancellation as proof that a candidate satisfies the law.
- Changing relation policy while assuming unchanged schema/program names make
  persisted application meaning compatible.

## Current Limits

The ordinary application builders expose source-to-target acyclicity with no
allowed cycle class, same-relation paired symmetry, and directed or normalized
edge uniqueness, same-partition endpoints and distinct reachable-target minima.
Other native symmetry modes are not exposed by these builders. They do not
provide component query operations. Use projection readers for complete walks
and orbit inspections as described above.
The closing-edge diagnosis is not an enumeration of every cycle or a repair
plan. Native evaluation has no application-authored depth limit; it remains
subject to the original execution request's resource and interruption policy.

## Related Docs

- [Application front door](../foundations/ordinary-application-front-door.md)
- [Programs and adoption](../foundations/programs-and-adoption.md)
- [Invariants and read observations](../foundations/invariants-and-read-observations.md)
- [Internal canonical-query schema validation](./schema-validation.md)
