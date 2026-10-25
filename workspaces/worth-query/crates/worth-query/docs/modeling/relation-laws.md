# Relation Laws

## What This Feature Is

A relation law is declared graph meaning enforced whenever an application
commits a change. Declaring `.acyclic()` means that following that relation from
source to target must never return to the starting entity. Query lowers the
declaration into Relational's native commit validation.

Paired symmetry requires the reverse of each surviving directed edge on the
same relation. Scoped uniqueness limits physical records per ordered pair or
across both directions of an unordered pair.

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

Law meaning participates in canonical schema and program identity. Changing
the law is a schema/program change; reusing the same names does not authorize
an installed application to acquire different meaning on restore.

## How It Executes

1. Author the relation and completed schema, then validate the program with
   `ApplicationProgramAuthoring::begin().with_schema(&schema).validated_program()`.
2. Install through the ordinary program installation path.
3. Authenticate a request and run its declared operation. The handler reads
   through `DecisionReader` and writes through `CandidateWriter` as usual.
4. Native commit validation checks the relation's proposed edges. A law
   violation refuses publication and leaves committed graph truth unchanged.

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
native uniqueness scope and physical count. Diagnostic `ContractId`, `KindId`,
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
edge uniqueness. Other native symmetry modes and the reachability-minimum and
partition-isolation families are not exposed by these builders. They do not
provide general traversal, orbit or component query operations.
The closing-edge diagnosis is not an enumeration of every cycle or a repair
plan. Native evaluation has no application-authored depth limit; it remains
subject to the original execution request's resource and interruption policy.

## Related Docs

- [Application front door](../foundations/ordinary-application-front-door.md)
- [Programs and adoption](../foundations/programs-and-adoption.md)
- [Invariants and read observations](../foundations/invariants-and-read-observations.md)
- [Internal canonical-query schema validation](./schema-validation.md)
