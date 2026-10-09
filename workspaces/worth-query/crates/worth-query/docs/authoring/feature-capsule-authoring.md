# Feature Capsule Authoring

An application feature capsule states one feature's installed meaning and lowers
directly into Query's canonical `ApplicationFeatureSpec`. It is the ordinary
place to associate typed actions, ports, derived truth, managed computation,
requirements, correspondence, and external input. It does not install handlers,
publish results, or create a second registry.

The compact form and the builder form produce the same canonical spec:

```rust
use worth_query_decl::facade::worth_query_feature_spec;

let capsule = worth_query_feature_spec!(
    root(ProductSchema, PricingFeature);
    provides(PricingOutput);
    mutation_with_locality_and_change(
        RepriceBinding,
        ProductNeighborhood,
        RebuildPrice
    );
    derived_artifact(CurrentPriceArtifact);
    managed_computation(PriceComputation);
);
```

Use `root(Schema, Feature)` for the root composition and
`at(Schema, Instance, Feature)` when the same feature meaning participates in a
named composition instance. The macro only removes builder plumbing. Its typed
members call the same builder methods and its result enters the same program
validation and installation path.

## Ports and connections

A feature implements `ApplicationFeature<Schema>` and declares its typed input
shape. Output ports are added with `provides(Output)`. Connections remain in the
program's typed connection inventory because they express a relationship between
two feature occurrences, rather than meaning owned by either capsule alone.

Required inputs must be connected by a compatible exported output. Validation
denies missing ports, incompatible value bindings, cross-instance connections
that are not exported, duplicate identities, and connections to an undeclared
feature. Declaration order and matching strings cannot create a connection.

## Scope and actions

Mutation and query bindings retain scope authority. A mutation binding identifies
its typed scope field, value extractor, principal binding, source expectation,
finite candidate limits, and output posture. `worth_query_mutation_binding!`
supports both concrete schemas and generic consumer schemas; the generic form
keeps the typed source expectation and schema-provided principal contract in the
same declaration.

Add the binding to a capsule with `mutation(Binding)` or, when derived truth is
affected, with:

```rust
mutation_with_locality_and_change(Binding, NeighborhoodLocality, ChangeShape);
```

Query rejects a field from another schema, a value extractor with the wrong
value type, a foreign principal contract, an undeclared operation, or a mutation
whose locality/change meaning does not satisfy a governed artifact. Broad scope
is valid only when the actual dependency closure is broad.

## Correspondence, requirements, and external input

Repeated product rows use the binding's typed action correspondence. The
installed correspondence derives initial state and exact row targets and
preserves `Unchanged`, `Set`, and `Clear`. It is not a client payload mapping.

An evaluated requirement is the one governed result used for both guidance and
submission enforcement. A separate UI predicate cannot grant callability.
External input resolution captures the selected values, provider revision, and
provenance before candidate validation; revision mismatch or unavailable input
is denied before effects.

The production consumer proofs live under
`worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/`.
The correspondence and external-input modules show real entry preparation and
denials without a test-only registration path.

## Derived artifacts and locality

`derived_artifact(Artifact)` attaches an already declared artifact contract.
That contract owns dependencies, producer family, locality, retention,
succession, reuse, stopped outcome, and resource ceilings. The capsule does not
restate those facts. `derived_collection(Collection)` does the same for a
governed collection.

When governance is `Required`, every connected output target needs complete
derived meaning. Query denies missing dependencies, a mismatched occurrence or
change shape, foreign settlement, resource excess, and invented reuse. Domain
code still owns artifact payloads and numerical algorithms.

## Managed computation and speculation

`managed_computation(Computation)` associates a typed computation declaration
with the feature that owns its result. The declaration identifies its input
basis, output artifact, computation partition key, reuse, stopped outcome,
execution posture, determinism contract, and ceilings. Installation then
requires the matching runtime owner.

A computation partition key is `Send + Sync` and serializable. Two keys are the
same key exactly when their canonical encodings are equal: the encoding is the
one that identifies structured operation inputs, and a key needs no ordering or
equality of its own. A partition's identity is derived from that encoding, and
partitions are ordered by partition identity, so their order is deterministic
and not chosen by the author. A `Deterministic` computation declares
`ApplicationSingleComputationPartition`, and a `DeterministicPartitioned` one
declares its own key. Program validation denies either mismatch. The
determinism contract defaults to `DeterminismContract::CanonicalBitwise`, and
like every other declared member it is part of the program revision.

A `Deterministic` computation installs a `WorthQueryManagedComputationOwner`
with `computation`. A `DeterministicPartitioned` one installs a
`WorthQueryPartitionedComputationOwner` with `partitioned_computation`: the
owner names each input item by a stable identity and a key, computes one
partition, declares its reducer, and completes the reduced result. Query
derives the partition identities, gathers each partition's items in item
identity order, runs every partition, and reduces the results over the
canonical tree, so the result and its charged work do not depend on the order
of the input. Installation refuses an owner binding that does not serve the
declared execution, and refuses a `ContractEquivalent` declaration.

Branch and preview work uses the same validated program and typed bindings as
ordinary work. Speculative execution does not create alternate feature meaning
or publication authority. Acceptance publishes admitted results; cancellation
and rejection release exact runtime resources without committing speculative
state.

## Validation and diagnostic inspection

Build through the canonical program boundary:

```rust
let program = ApplicationProgramAuthoring::<ProductSchema, ProductProgram>::begin()
    .validated_program()?;
let manifest = program.normalized_manifest();
for record in manifest.records() {
    println!("{record}");
}
```

The normalized manifest is deterministic diagnostic evidence. It describes
features, ports, actions, connections, rules, artifacts, collections, and
computations. It carries no runtime handle or authority; deleting a rendered
manifest cannot change installation or execution.

For a complete production entry, read
`worth-query-certification/examples/ordinary_product_workflow.rs` and its
`product_workflow_support/program.rs`. `tests/program_example_authority.rs`
shows that a lookalike program cannot borrow another program's authority. These
examples compile against the public facade and execute through the host entry.

## Authoring rule

An ordinary mutation may attach an external source contract with
`mutation_with_external_input::<Binding, Provider>()` or the matching
`mutation_with_external_input(Binding, Provider);` capsule form. This requires no
evaluated requirement or output correspondence. The installed provider slot
captures the provider's resolved values and revision; admission revalidates that
revision. The caller must pass those captured values into the ordinary operation.
The declaration does not itself replace the mutation input or prove the external
source stayed current through graph publication.

Add meaning at its owner: schema members in the schema declaration, operation
and scope in the binding, feature membership in the capsule, cross-feature
relationships in the program, and runtime implementations in the contribution
setup. A capsule replaces recursive application-wide membership lists; it must
not be accompanied by a global action registry, provider switch, copied identity
table, manual invalidation map, or alternate installer.
