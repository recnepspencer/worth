# Feature Capsule Authoring

An application feature capsule states one feature's installed meaning and lowers
directly into Query's canonical `ApplicationFeatureSpec`. It is the ordinary
place to associate typed actions, ports, derived truth, managed computation,
requirements, correspondence, and external input. It does not install handlers,
publish results, or create a second registry.

The compact form and the builder form produce the same canonical spec. An
ordinary action begins with `.action::<Binding, Delivery>()`; its real delivery
selects source completion, retained commit, fixed output or discovered output.
Attach the applicable metadata, then call `.finish_action()` to append one
complete declaration and resume feature authoring. The unfinished action has
neither feature `finish()` nor `provides()` and cannot expose mutable meaning.

The [neutral consumer declaration](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/composed_action.rs)
keeps locality/change, a rule and an external provider on its actual fixed
source action. The surrounding source module supplies its typed imports and
other feature members:

<!-- compiled: composed_action.rs -->
```rust
pub(crate) fn source_feature_spec() -> ApplicationFeatureSpec {
    features::source_builder()
        .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>()
        .locality_and_change::<SourceLocality, SourceChange>()
        .evaluated_requirement::<CatalogInputRequirement>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
        .finish()
}
```

Use `root(Schema, Feature)` for the root composition and
`at(Schema, Instance, Feature)` for a named composition occurrence. The macro's
block grammar is:

```text
action(Binding, Delivery) {
    locality_and_change(Scope, Shape);
    correspondence(Correspondence);
    evaluated_requirement(Rule);
    external_input(Provider);
};
native_action(Binding) {
    // The same optional metadata clauses; no delivery or output-source override.
};
```

Each optional clause is consuming and once-only. All subsets are legal under
the existing trait bounds; empty blocks work. Metadata order preserves canonical
meaning. Equivalent authoring preserves the program revision; changing actual
delivery or metadata contracts changes canonical meaning through the existing
identity owner. Unsupported or repeated clauses are rejected, and each block delegates
to one typed builder and one `finish_action()`. Non-mutation members such as
`provides`, `derived_artifact`, `derived_collection`, `managed_computation` and
`conditional_operation` retain their semantics. Root-only `.operation::<Op>()`
remains a builder capability.

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

Mutation and query bindings declare scope meaning. A mutation binding identifies
its typed scope field, value extractor, principal binding, source expectation,
finite candidate limits, and output posture. `worth_query_mutation_binding!`
supports both concrete schemas and generic consumer schemas; the generic form
keeps the typed source expectation and schema-provided principal contract in the
same declaration.

The feature parent owns Schema, Instance and Feature; the action entry selects
Binding and Delivery once. `.locality_and_change::<Scope, Shape>()` attaches the
paired contract. `.correspondence::<C>()` requires correspondence for that exact
binding, and `.evaluated_requirement::<Rule>()` / `.external_input::<Provider>()`
require the binding's operation type. Callers supply neither state flags nor
raw type identities. Delivery derives required-output-source meaning; an
ordinary source-only declaration cannot erase a real required-output obligation.
The actual owning constructor refuses that mismatch.

Use `.native_action::<Binding>()` only for a present specialized native
responsibility, such as native staging, authority/phase certification or
explicit branch/host control. Ordinary product mutations and retries require
real delivery, including when a deliberate native caller also consumes the
same occurrence. Native required-output membership is derived from actual
program roots; there is no output-source toggle or second mutation spelling.

The five-part `ApplicationActionReference<Schema, Instance, Feature, Binding,
Delivery>` is a sealed selector. `ApplicationAction::selector()` does not expose
or manufacture full declaration meaning. Installation resolves the complete
actual action and later progression compares all metadata against that installed
meaning. The existing definition validator rejects duplicate binding TypeIds
and duplicate portable `(instance, feature, binding identity)` keys while
preserving separate legal composition occurrences.

Query rejects a field from another schema, a value extractor with the wrong
value type, a foreign principal contract, an undeclared operation, or a mutation
whose locality/change meaning does not satisfy a governed artifact. Broad scope
is valid only when the actual dependency closure is broad.

## Correspondence, requirements, and external input

Repeated product rows use the binding's typed action correspondence. The
installed correspondence derives initial state and exact row targets and
preserves `Unchanged`, `Set`, and `Clear`. It is not a client payload mapping.

Correspondence capability lookup is binding-type scoped. Evaluated-rule and
external-provider lookup is scoped to operation plus rule/provider type across
declared actions. Delivery/locality/change remain exact-occurrence meaning;
this composition does not introduce occurrence-isolated permission or a new
policy engine.

The installed evaluator returns one result for guidance and explicit submission
enforcement. Installing the clause does not run it or turn rule evaluation into a native
authorization gate. Rule-without-provider and provider-without-rule are
both legal. External input resolution captures values, revision and provenance;
its actual admission revalidates the revision before those accepted values enter
the original source action. Declaration alone does not prove an external source
stayed current through later graph publication.

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

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
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

An ordinary action composes `.external_input::<Provider>()` on its
`.action::<Binding, Delivery>()` progression before `.finish_action()`.
It requires neither an evaluated requirement nor correspondence. Resolve and
admit the external input through the installed provider, then use its accepted
values in the actual source intent. Keep every contract on this same action;
adding another feature action is not a metadata attachment.

Add meaning at its owner: schema members in the schema declaration, operation
and scope in the binding, feature membership in the capsule, cross-feature
relationships in the program, and runtime implementations in the contribution
setup. A capsule replaces recursive application-wide membership lists; it must
not be accompanied by a global action registry, provider switch, copied identity
table, manual invalidation map, or alternate installer.
