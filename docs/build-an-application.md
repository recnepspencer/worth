# Build an Application

> **The application story, end to end:** declare the program and its
> features, install the application graph, run it, author and run workflows,
> and adopt a new program on a branch. Code examples link to runnable fixtures;
> illustrative blocks identify the parts you supply in your own application.

This is the centerpiece guide for building on WORTH. Other public documents
explain *why* the platform works the way it does
([Philosophy](philosophy.md)) and *what* each runtime guarantees
([How WORTH Works](how-it-works.md)). This one shows *which names you call, in
which order*.

For a new feature, use the complete feature definition below. It keeps the
feature's schema members, actions, required lifecycle roles and runtime slots
together; the SDK derives their projections. The lower-level builders later in
this guide explain the underlying contracts and support specialized composition.

| Your next task | Start here | What you supply |
|---|---|---|
| Choose durable facts, names and links | [Native representations](native-representations.md), then [source subgraphs](source-subgraphs.md) | Independently mutable fields, entities, typed relations and an explicit read closure; canonical atoms need a recorded reason |
| Explain a query result or an authored operation | [Inspect original evidence](#inspect-original-source-and-query-evidence) | Saved source/query receipts and current inspection permissions |
| Correct a recorded source operation | [Recorded source correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md), including [consecutive original sources](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#reverse-consecutive-original-sources) and the [compiled consecutive-undo journeys](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/consecutive_undo.rs) | A source receipt, current principal, target branch and client key; declare the inverse and disclosure query in the contribution. Canonical authored writes require exact later source/reversal pairs on the selected ancestry. Original and target support must agree under compatible installed meaning and full-generation identity, including complete aspect content, presence and live adjacency. Relation-dependent fields remain separate dependencies. Retain endpoint custody; handle typed unavailable evidence and stale-target refusals through the returned owners. |
| Add an ordinary feature | [Complete feature definitions](#complete-feature-definitions) | Typed members, actual handlers and domain configuration |
| Declare operation access | [Operation access](#21-declare-operation-access-once) | Reads, effects and preconditions in one declaration |
| Compute from tracked inputs | [Managed computation](#36-declare-and-install-a-partitioned-managed-computation) | Tracked gathering, pure numerical data and original-owner completion |
| Share execution across applications and reopened sessions | [One process execution service](#one-process-execution-service) | One process-owned `Arc<ExecutionAuthority>`, an explicit worker policy and the original requests |
| Install or reopen an application | [Build identity](#application-build-identity), then [installation](#41-one-program) | One immutable identity from the build pipeline, configuration and installed resources |
| Submit, recover and close an action | [Owning actions](#53-execute-through-the-program) | Original input, source and key; fresh authorized requests for progression |

An application imports exactly two crates, `worth-query-decl` and
`worth-query-host`. Their facades are made of modules, so import from the
module that owns each name:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Declare meaning: features, program, workflow authoring.
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::{application_operation, application_query, application_schema};

// Install and run.
use worth_query_host::facade::{
    admission::authenticated_principal,   // principals and request scopes
    application_entry,                    // requests, mutations, adoption, workflow lifecycle
    application_installation,             // in_memory_program, rosters, limits
    domain,                               // installed schema, workflow vocabulary installation
    primary_graph,                        // handlers, seeding, inspection types
    runtime,                              // resource profiles
};
```

| Module | Names you will look for there |
|---|---|
| `â€¦decl::facade::application_program` | `ApplicationFeature`, `ApplicationFeatureSpec`, `ApplicationProgramDefinition`, `ApplicationProgramAuthoring`, `ValidatedApplicationProgram`, ports and connections, `ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder` |
| `â€¦decl::facade::application_operation` | `ApplicationMutationBinding`, `ApplicationMutationIntent`, `ApplicationCapabilityMutationBinding` |
| `â€¦host::facade::application_installation` | `in_memory_program`, `in_memory_rostered_program`, `WorthQueryInMemoryApplicationLimits`, `WorthQueryProgramApplicationRuntime`, `WorthQueryProgramOwner` (a trait: import it to call `owned_revision()`) |
| `â€¦host::facade::application_entry` | `WorthQueryApplicationRequestExt`, `WorthQueryOperationBuilder`, `WorthQueryOperationDescription`, `WorthQueryOperationAdvance`, `WorthQuerySettledFixedOperation`, `WorthQueryTerminalFixedOperation`, `WorthQueryWorkflowOperation`, `WorthQueryApplicationMutationOutcome`, `WorthQueryBranchAdoptionPublicationOutcome`, `Workflow*Outcome`, `Published*Ref` |
| `â€¦host::facade::application_discovery` | `WorthQueryWorkflowDefinitionDiscovery`, `WorthQueryWorkflowInstanceRead`, `WorthQueryWorkflowInstancePage`, `WorthQueryWorkflowHistoryPage`, `WorthQueryWorkflowHistoryMark` |
| `â€¦host::facade::domain` | `WorthQueryApplicationWorkflowSpecInstallation`, `WorthQueryApplicationWorkflowResourceCeiling`, `WorthQueryInstalledApplicationSchema` |
| `â€¦host::facade::primary_graph` | `OperationHandler`, `DecisionReader`, `CandidateWriter`, `WorthQuerySelectedProgramInspection` |
| `â€¦host::facade::admission` | `authenticated_principal::{admit_authentication_adapter, WorthQueryRequestScope}`, `authentication_event::install_authentication_event_owner` |

`worth_query_host::facade::declaration` exposes the same declaration modules,
so a host crate can reach `declaration::application_program::*` without a
second import. Most excerpts below do that. The host facade re-exports only
some declaration macros. `worth_query_mutation_binding!`,
`worth_query_query_binding!`, `worth_query_capability!`,
`worth_query_policy!`, and `worth_query_feature_spec!` are only on
`worth_query_decl::facade`.

**Before you conclude a capability is missing,** follow the procedure in the
[Capability Index](capability-index.md).

---

## Contents

- [0. The whole journey on one screen](#0-the-whole-journey-on-one-screen)
- [1. The model: six things, kept apart](#1-the-model-six-things-kept-apart)
- [2. Declare the schema and its contributions](#2-declare-the-schema-and-its-contributions)
- [3. Declare features and the program](#3-declare-features-and-the-program)
- [4. Install the application graph](#4-install-the-application-graph)
- [5. Run the program](#5-run-the-program)
- [6. Author, publish, and run workflows](#6-author-publish-and-run-workflows)
- [7. Adopt a new program on a branch](#7-adopt-a-new-program-on-a-branch)
- [8. Application execution checklist](#8-application-execution-checklist)
- [9. Use the consumer vocabulary](#9-use-the-consumer-vocabulary)
- [10. Where to go deeper](#10-where-to-go-deeper)

---

## 0. The whole journey on one screen

| Step | What you do | The call | Section |
|---|---|---|---|
| 1 | Declare the schema and its contributions | `worth_query_application!`, `worth_query_application_contribution!`, `impl WorthQueryApplicationContribution` | [Â§2](#2-declare-the-schema-and-its-contributions) |
| 2 | Declare features and the program | `worth_query_application_contribution! { pub feature ... }`, its `ApplicationFeatureDefinition` projections, `impl ApplicationProgramDefinition` | [Complete definitions](#complete-feature-definitions), [Â§3](#3-declare-features-and-the-program) |
| 3 | Validate the program | `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Â§3.5](#35-validate) |
| 4 | Install the application graph | `application_installation::in_memory_program(..)` or `in_memory_rostered_program(..)` | [Â§4](#4-install-the-application-graph) |
| 5 | Run program-owned actions | `request.mutate(intent)â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)` | [Â§5](#5-run-the-program) |
| 6 | Install the workflow vocabulary | `WorthQueryApplicationWorkflowSpecInstallation::begin(..)â€¦finish()`, then `retain_workflow_spec(..)` | [Â§6.2](#62-install-the-workflow-vocabulary-once) |
| 7 | Author and validate a workflow | `ApplicationWorkflowDefinitionBuilder::<Spec>::new(..)`, nodes, connections, `finish()?.validate()?` | [Â§6.3](#63-author-a-definition) |
| 8 | Publish, start, and advance it | `bind_definition`, `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_advance` | [Â§6.6](#66-publish-discover-start-advance) |
| 9 | Adopt a new program on one branch | `.on_branch(b).programs()`, `compare(&target)`, `adopt(&requirements)?`, `prepare(max)`, `programs.publish(prepared)` | [Â§7](#7-adopt-a-new-program-on-a-branch) |

The running example is the **Bank reference application**
(`workspaces/worth-query-bank-world`). It is the only complete application
in the repository that uses nothing but the two facades. Banking is
incidental: every step below has the same shape for documents, orders,
tickets, or anything else. Where Bank does not exercise a feature, the guide
uses the document-retention certification model or a runnable example.

---

## 1. The model: six things, kept apart

| Thing | What it is | Where it lives | Rust name |
|---|---|---|---|
| **Schema** | The shape of the application graph: entities, aspects, fields, relations, operations, queries | Declared once, installed once | `worth_query_application!`, `ApplicationSchemaDeclaration` |
| **Contribution** | One named slice of the schema, plus the host setup it needs (handlers, invariants, conditional nodes) | Declared and configured at install | `worth_query_application_contribution!`, `WorthQueryApplicationContribution` |
| **Feature** | A named unit of application meaning that *owns* actions (mutations, operations) and may have typed input and output ports | Part of a program | `ApplicationFeature`, `ApplicationFeatureSpec` |
| **Program** | A content-addressed revision of the application's authored meaning: its contributions, features, connections, output graph, and rules | Validated before install; rostered on a host | `ApplicationProgramDefinition`, `ValidatedApplicationProgram`, `ApplicationProgramRevision` |
| **Workflow** | A branch-local, versioned graph of governed steps, authored against a vocabulary the host installed | Data on a branch, published at run time | `ApplicationWorkflowSpec`, `ValidatedWorkflowDefinition` |
| **Activation** | Which program revision one branch runs | One record per branch | Read through `.programs().inspect()` |

Three separations matter most:

1. **Declaration grants nothing.** A validated program or a validated workflow
   definition carries no install, publication, or execution authority. The
   host turns meaning into a running application.
2. **Programs are static; workflows are data.** A program is compiled Rust,
   validated and installed by the host. A workflow definition is published on
   a branch at run time, even when it was written in Rust and ships with the
   application.
3. **Revision, support, and activation are three different facts.** The
   revision says what the program means. Support says this host can run it
   (it is rostered). Activation says a given branch runs it. Changing
   activation is *adoption* ([Â§7](#7-adopt-a-new-program-on-a-branch)).

Terms: [Glossary](glossary.md) â€” see **Program**, **Feature**,
**Contribution**, **Adoption**, **Workflow**, **Installation**.

---

## 2. Declare the schema and its contributions

A schema is composed from contributions. Each contribution declares its own
entities, fields, operations, queries, and invariants.

For feature members, use the [complete definition](#complete-feature-definitions)
to supply declarations and runtime projections. Use the contribution layer below
for contribution-wide invariants, host setup and standalone conditionals.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-testkit-certification/src/certification/program_example_authority/product_workflow_support/schema.rs (abridged)
worth_query_application! {
    pub TemporalHostSchema {
        owner: "temporal_host_courtroom",
        version: (1, 0),
        contributions: [TemporalHostContribution],
    }
}

worth_query_application_contribution! {
    pub contribution TemporalHostContribution in TemporalHostSchema {
        identity: "temporal_host_courtroom.example.v1",
        members: |schema| {
            let schema = schema
                .entity(TemporalIntent::reference())
                .aspect(TemporalIntent::reference(), IntentFacts::reference())
                .field(TemporalIntent::reference(), IntentRevisionField::reference())
                // ... more fields, relations, a principal binding ...
                // ... operations use declared_operation with their typed access (Â§2.1) ...
                .invariant(integrity::definition());
            super::application_entry::declare(schema)
        }
    }
}
```

The host side of the same contribution registers what runs: handlers,
invariants, and conditional nodes.
For a required Initial/Preserve producer feature, use the single
[required-role declaration](#required-initial-and-preserve-output-roles) and its
`contracts.feature` / `setup.feature` projections. The temporal example below
is a standalone conditional, not a paired output feature.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/conditional_contribution.rs
impl WorthQueryApplicationContribution<TemporalHostSchema> for TemporalHostContribution {
    type Configuration = TemporalContributionConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<TemporalConditional>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            TemporalIntegrity::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            integrity::resolve_rule,
        )?;
        setup.handler::<super::application_entry::AmendTemporalBinding, _>(
            super::application_entry::AmendTemporalHandler,
        )?;
        setup.conditional::<TemporalConditional>(configuration)
    }
}
```

- `contracts` defaults to `Ok(())` for contributions without required runtime
  contracts. A required pair must project its contracts with `contracts.feature`.
- A contribution with nothing to configure uses `type Configuration = ();`.
  Bank has three such contributions and installs them with the configuration
  tuple `((), (), ())`.
- A missing handler fails installation before any initial state is written.

The schema and contribution APIs are covered in depth by the
[`worth-query-host` README](../workspaces/worth-query/crates/worth-query-host/README.md)
and the [`worth-query-decl` README](../workspaces/worth-query/crates/worth-query-decl/README.md).
### Add optional aspects to a shared entity

The model fragment declares `Feature` once. Independent features can extend it
without making the model depend on those features:

<!-- illustrative -->
```rust,ignore
use worth_query_decl::facade::application_schema::fragment::*;

impl Entity for Feature {
    const NAME: &'static str = "feature";
    type Aspects = (Identity,);
    type Relations = ();
}
impl Fragment for Naming {
    const NAME: &'static str = "naming";
    type Bindings = FeatureBindings;
    type Members = (Extend<Feature, (Option<Label>,), (WithAspects<Related, (Option<Weight>,)>,)>,);
}
impl Fragment for Coloring {
    const NAME: &'static str = "coloring";
    type Bindings = FeatureBindings;
    type Members = (Extend<Feature, (Option<Color>,)>,);
}
// The root includes Model (whose Members contains Feature), Naming and Coloring.
let label = aspect(Naming, Feature, Label);
let color = aspect(Coloring, Feature, Color);
let related = relation::<App, _, _, _, _, _>(Naming, Feature, Related);
```

`Extend` accepts only `Option<A>` and `Option<Exclusive<G>>` (including nested
tuples). The entity must be declared by another fragment in the same schema;
member order does not matter. Duplicate member names are refused with both paths.
The third parameter adds outgoing relations with the usual payloads and policies, with no positive source minimum and positive target/pair minimums only for targets declared by the extending fragment or its nested fragments.
Only the extending fragment, or its nested tuple path, can select its additions.
Selections use the ordinary fields and `View` reads/writes; optional additions
leave creation by the model complete. The executable counterpart is the
[extension court](../workspaces/worth-query/crates/worth-query-court-decl-host-native-fragments/tests/worth_query_court_decl_host_native_fragments/native_fragments/extensions.rs).

### 2.1 Declare operation access once

For a native handler on a concrete application schema, put its complete access set in
`worth_query_operation!`'s `access` clause. It generates both the Rust
capabilities and the installed schema projection. In this small example,
`Schema`, `Input`, `Record`, `Value` and `Children` are already declared typed
schema members:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_decl::facade::{application_schema::*, worth_query_operation};

worth_query_operation!(pub Edit for Schema, input Input;
    access {
        read_entities: [Record];
        read_fields: [Value];
        read_relations: [Children];
        writes: [Value];
        expects_version: [Value];
        links: [Children];
        unlinks: [Children];
    }
);

let schema = schema.declared_operation::<Edit>(
    Edit::reference().definition().no_external_effect().no_aftermath().finish(),
);
```

Use `read_entities`, `read_fields` and `read_relations` for decisions; `writes`,
`creates`, `deletes`, `links`, `unlinks` and `emits` for effects; and
`expects_version` / `expects_fact` for source preconditions. Empty or unused
clauses can be omitted. Declare every actual access, including related members;
the macro does not inspect handler bodies. Effect/aftermath policy and mutation
binding installation remain explicit at their existing owners.

Edit the access clause when adding a field or relation. The generated Rust
capabilities and schema registration follow that declaration; native admission
checks the handler's actual reads and effects against it.

For a [declarative operation body](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md),
the body derives its installed footprint from the authored statements. See the
[schema and access examples](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/declared_operation/tests.rs)
for projections and schema validation. The
[`ApplicationDeclaredOperation` reference](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/declared_operation.rs)
includes a passing capability example and an undeclared-member compile refusal.

---


### Inline opaque values

`ScalarAspectType::InlineBytes` carries the complete canonical representation of
one atomic domain value. It can be decoded by a pure observer receiving the
scalar, without borrowing Native content custody. The generic certification
counter's conditional predicate is an actual consumer of this contract; the
ratio binding is a standalone atomic example and the broken binary binding is
a refusal fixture. See the [full ownership review and inventory](inline-atomic-content-boundary.md).

Use Native ContentRef for ordered identity-free numerical data, including
splines at every size. Its digest requires the installed format and real Native
resolver authority. The 16 MiB content frame limit refuses at admission before
effects; it never redirects the value to InlineBytes. Representation follows
meaning and authority, never a size threshold.

The typed content door is `content(entity, content_field(Field::at("Field"), Format))`.
It returns `Result<Option<AdmittedContent<Format>>, ContentReadDenial>` on
`DecisionReader`, `WorthQueryApplicationInvariantProjectionReader`,
`WorthQueryApplicationOperationInvariantProjectionReader`,
`WorthQueryComputationReader`, and `WorthQueryApplicationInvariantReadView`.
For the invariant view, pass its own `WorthQueryApplicationInvariantEntity`;
other readers use their resolved `WorthQueryInvariantEntityIdentity`.
`payload()` borrows the admitted immutable bytes. An absent optional aspect or
field returns `None`; a missing required field, missing content, mismatched
format or invalid frame is a typed refusal. The installed field remains the
dependency, so replacing its content identity invalidates consumers. Each read
uses its reader's original request and work allowance. Refusals remain sticky
through projection and invariant completion even if a callback catches them.
The Host `primary_graph`, `application_invariants` and
`application_contribution` facades export the content custody and denial types.

The immutable
`InlineBytes` carrier admits at most **4,294,967,290 payload bytes**
(`u32::MAX - 5`). The existing native scalar frame uses a 32-bit body length;
one tag byte and four payload-length bytes occupy the remaining five bytes.
This is a framing limit, not a recommended allocation or an execution grant.
`InlineBytes::admit_length` and `try_new` return `InlineBytesLengthDenial`
above that limit, before the value can enter a mutation. Wire readers check
the declared length before copying its payload, and serde preserves the same
invariant. Binding-specific representation bounds and the original request's
work, cancellation and memory limits still apply.

The scalar carries no domain ordering, comparison or indexing hooks. Byte
identity serves digests and change detection. The owner remains responsible
for canonical encoding, domain admission, and terminal rendering. Independently
editable objects and graph references belong in entities and relations.

Choose a representation by ownership and change granularity:

| Meaning | Representation |
|---|---|
| Independently named or editable object | Entity with stable semantic identity |
| Link to another object | Typed relation to its actual entity |
| Fact with an existing scalar meaning | Native scalar field |
| Complete canonical atomic value used by pure observations | Versioned domain-value binding; one native scalar atom |
| Ordered identity-free numerical dataset | Format-bound Native ContentRef at every size |

Declare a binary atom with `worth_query_value_binding!` and `binary: OwnerDelegate`,
plus its intrinsic `unit:` and `frame:` when applicable. The delegate implements
`BinaryValueDelegate` through `worth_query_decl::facade::application_schema`;
its associated value needs no Query or Serde implementation. The
[compiled entry binding](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/binding.rs)
adopts a [pure Ratio owner](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_values/src/ratio.rs).
Its nine-byte maximum is its complete versioned representation, rather than
an arbitrary payload policy. `worth_query_decl::facade::typed` also exposes
`InlineBytes` and `InlineBytesLengthDenial` for scalar-boundary input.

The owner normalizes authored values through its constructor and encodes only
canonical bytes. Incoming bytes must already be canonical: admission refuses
alternate spellings, malformed data and unsupported versions. Every binary decode
checks that encoding the admitted result reproduces the input exactly; a corrupt
owner result returns `ValueCodecDenial::InternalInconsistency`. Size refusals retain
the binding identity and precede mutation effects. Decoding and the independent
byte comparison share the original request's work, memory and cancellation loan.
Ordinary writes also pass through this installed admitted decoder and
canonical comparison after effect expansion and before provider lowering or
publication. This single commit-preparation seal covers binary, canonical domain
and delegated canonical domain bindings, including entity creation, field updates,
optional-field patches and relation creation. An owner whose encoder emits bytes
its reader rejects receives `DomainValueRejected`; `domain_value_decode_denial()`
retains the binding identity and codec cause. Request resource refusals retain
their original execution cause. Either refusal leaves the selected commit unchanged.
There is no replacement request or representation-only decoder fallback. Declare
fields with `no_equality`; domain equality and ordering remain with the owner.
Render terminal values through the owner's reader/renderer, as the Ratio example does.

Use these bindings on fields in an existing struct aspect. The explicit
`scalar_aspect_field` declaration currently requires a codec-free scalar binding;
it refuses a domain codec. This capability adds no aspect shape form.
The [correction and recovery courts](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain.rs)
exercise the opaque field through ordinary owners. Their literal encodings and
the production decode/re-encode check certify stored byte identity across recovery.
Whole model checkpoint equality separately checks that admitted reads have no
effects within either runtime; runtime identity metadata legitimately rebinds.

Canonical text can explicitly claim the same leaf posture with
`canonical: FINITE_BOUNDS, domain_value,` and optional `unit:` and `frame:`.
Binary atoms use `worth.domain-binary.v1`; canonical atoms use
`worth.domain-canonical-json.v1`. Existing structured canonical artifacts retain
`worth.canonical-json.v1` and their existing complete grammar. Those artifacts
make no domain-leaf claim and cannot satisfy a domain-atom contract. An immutable
operation artifact's established structured protocol is distinct from a leaf value;
neither posture licenses hiding independently editable objects or graph links.

Every explicit domain binding must have a source-guard registration, including
canonical and manual bindings. Register its actual Rust source, binding name
(including inline module names), and the owner's key/name/object types:

```toml
[[binding]]
source = "app/src/value_binding.rs"
binding = "RatioBinding"
forbidden_types = ["app::graph::NodeKey", "app::names::StableName"]
posture = "domain"
reason = "Reference-free numerical ratio with a canonical representation."
```

The guard follows Cargo library and binary roots, actual modules, imports, re-exports, aliases,
all fields and enum variants, and concrete generic arguments. Platform entity and
content references are always refused. Unknown types, ambiguous imports, transforming
attributes, generated bindings and unsupported local scopes refuse a proof. The
guard does not infer reference-free meaning from an innocent type name. Keep the
owner's forbidden list complete; a primitive string or integer is not permission
to encode a graph link. In WORTH this runs as part of `boundary-check`. An external
consumer runs the same checker from its Git and Cargo workspace root, with its own registry:

```text
cargo run --manifest-path path/to/worth/tools/boundary-check/Cargo.toml -- domain-values --root . --registry domain-values.toml
```

A repository whose bindings live in a separate consumer workspace can list its actual
Git-owned `Cargo.toml` paths in the registry's optional `workspace_manifests` array.
This extends Cargo metadata discovery; module and type names still come from the
actual library roots and Rust declarations. It does not register generated context
workspaces or supply alternate type roots.


Consumer sources must be in Git's tracked or unignored source inventory. The
registry path is relative to the consumer root, or absolute. Consumer metadata
uses `cargo metadata --no-deps --offline`; the checker never compiles consumers.
Manual bindings returning `ApplicationCanonicalValueContract::of` or `delegated`
also require registration. The guard walks their actual associated `Value`, not
the constructor's codec or grammar type. Every entry requires a `posture` and a
nonempty `reason`: `domain` and `exception` both receive the reference walk;
`diagnostic` exempts owner diagnostic payloads from that walk. Diagnostics still
require a real, uniquely registered source binding. Unmarked structured artifact
macros retain their existing separate protocol posture.
Unresolved external opaque types require a source representation the guard can
inspect; they receive no automatic exemption. Installation independently refuses
a mismatch between the declared domain posture, carrier and executable binding.

### Canonical nested values and derived readiness

Choose [native fields, entities and typed relations](native-representations.md)
for durable application structure first. A nested Rust value does not make its
members independently readable or patchable. The canonical binding below is for
an intentional identity-free, reference-free atom, an opaque owner-admitted leaf
or a named document boundary. Record each durable exception's owner, policy,
reason and reference guard; keep independently edited objects and links native.
See [source subgraphs](source-subgraphs.md) for partial edits and prior state.

An application value owner can derive
`worth_query_decl::facade::application_schema::WorthValue` to emit
Serde serialization, portable wire shape, and a checked deserialization seed
from one definition. Use those generated serialization implementations. Supported
grammar includes concrete named/unit/newtype structs, external unit/newtype/named
variants, internally tagged unit/named variants, 8â€“64 bit integers, booleans,
strings, checked `usize` indices on a `u64` wire, fixed arrays, `Vec`, `Option`,
`Box` and exact two-element tuple payloads. Literal
`serde(rename = "...")`, enum `rename_all = "snake_case"`, and container
`deny_unknown_fields` are supported. Other Serde behavior and containers fail
compilation. An opaque domain owner supplies its own bounded `CanonicalValue`
seed and admitted construction, without a generic deserializer fallback.

Recursive types describe a finite grammar with lexical binders/references;
they do not claim finite type nesting. `ValueShapeDefinition::inspect()` reports
`ValueShapeDepth::Recursive` or an actual finite extent. Each recursion cycle
must cross a structural level. Structs, enum envelopes, sequences and pairs
descend an explicit depth policy when one exists; transparent newtypes and boxes
do not. Ordinary canonical bindings require no guessed byte, depth or collection
ceilings. JSON serialization and deserialization use stack growth; recursive
application-owned `Drop` and `Debug` remain the value owner's responsibility.
Static grammar functions allocate no descriptor during decoding. Their closure
inspection and all nested seeds use the same admission callback. The installed
catalogue refuses an unadmitted child even when another variant is selected.

An internally tagged canonical object writes its tag first, followed by fields
in declaration order. The bounded reader requires that tag before selecting a
payload; it never buffers an arbitrary object. Unknown, duplicate, missing or
conflicting fields/tags and incorrect pair arity refuse. Noncanonical ordering
does not become an alternative encoding. These native rules do not change an
external pack owner's declared input format.
Nullable optional payloads, such as `Option<Option<T>>` or `Option<()>`, are
refused because Serde's `null` would erase the distinction between absence and
a present null payload. Use an externally tagged enum for that meaning.

For that whole-value atom, the existing entry uses
`worth_query_decl::facade::worth_query_value_binding!` with `scalar: String`,
`canonical,` and an optional `validate: domain_validator` returning
`Result<(), &'static str>`. Domain validation runs on encode and decode. Fixed
arrays and pairs retain their actual type cardinality. Encoding checks
structure before emitting bytes; decoding uses generated seeds and requires the
exact canonical JSON form. There are no omitted fields or permissive defaults.
The installed field's `canonical_contract()` exposes its shape, bounds and wire
protocol. Nested field, enum, rename and bound changes participate in portable
schema identity; semantic aspect revisions remain explicit. An existing domain
representation policy may use `canonical: DOMAIN_BOUNDS`; absent policies use
`ValueDecodeBounds::unrestricted()`, without sentinel ceilings. Known codec
failures retain their exact `ValueCodecDenial` in
`Canonical { binding_identity, cause }`; native cancellation retains its original
request's stop at the owning handoff.

This canonical grammar remains one native String field. Inner members have no
native read/patch, touch, conflict or invalidation path, and serialization emits
the complete atom. Current structured canonical bindings install in Struct
fields; `ApplicationSchemaDeclarationBuilder::scalar_aspect_field` refuses that
carrier. Fine native field reads also do not imply independent sibling-write
transactions. See the [actual grain contract](../plans/worth-store/storage-foundation-aspect-native-gate.md#current-value-grain-contract).

The [canonical binding example](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical_tests.rs)
derives a nondefault `Span`, validates positive length in both directions and
distinguishes grammar refusal from explicit policy refusal:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// The linked owner test compiles the complete binding.
worth_query_decl::facade::worth_query_value_binding!(SpanBinding for Span {
    identity: "tests.span.v1",
    scalar: String,
    canonical,
    validate: positive,
});
```

A field whose concrete value belongs elsewhere can use
`#[worth_value(delegate = DomainDelegate)]`. Its existing `ValueSeedDelegate`
must describe the exact serialized wire and finish through the domain constructor.
For an already sealed body/original-text document, use
`native_envelope: DomainEnvelope` instead of repeating scalar bridge methods.
The pure envelope implements `CanonicalDocumentCodec` with associated `Document`,
domain-owned `BOUNDS`, original `encoded` borrow and strict `read`/`read_admitted`.
The generated Entry bridge forwards those readers and copies original text
fallibly. It never reconstructs a body/text pair or adds Query to the pure owner.

Opaque implementations describe their wire using the `SHAPE_BODY` constant and
sealed typed `ValueShapeEdge` links. A transparent delegate uses
`ValueShapeEdge::value::<Wire>()`; it does not copy Rust identities into the
portable shape. `SUPPORTS_ADMISSION` declares that implementation's own seed;
installation checks support across the complete value shape.
Constructor normalization and retained memory remain owner contracts. A pair
seed introduces no blanket tuple residency or shared-pointer accounting rule.

`installed_schema.value_meaning()` derives a descriptive aspect/field roster and
checks it against the installed executable bindings and native contracts. Field
codec digests use the same emitter as schema identity, including wire shape,
bounds, presence, binding, unit and frame. Native contracts carry those digests
beside the explicit semantic aspect revision. `compare_predecessor` refuses a
changed field meaning without an advanced revision; a golden or this comparison
does not establish authentic predecessor or migration authority.

Ordinary native recovery refuses changed codec meaning before exposing the
recovered graph. Advancing the aspect revision alone does not authorize adoption.
The explicit checkpoint transition below can retire a complete producer-created
output whose present fields remain covered by its authenticated native
Create/Preserve publication succession. Membership includes unpatched companions
and executed no-data completions. New dynamic members require actual allocations;
retired members require actual native retirement, and every prior live member
must remain accounted for. An intervening ordinary overwrite, including equal
bytes, revokes that producer ownership. Native
codec retirement records its complete governed disposition before publication;
target production starts a new ownership generation on that same route.
Ordinary deletion and producer Retire alone do not authorize a fresh Birth. Native
recovery preserves the same physical carrier and unchanged authored records,
retires the old output before adopting target contracts and indexes, and leaves
reconstruction to the installed target producer. The
[native codec journey](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_transition/successor/adoption.rs)
uses nonempty authored source, incompatible V1/V2 output codecs, real producer
publication, a source edit through Preserve, and current-format reopen.

For derived readiness, `package_contract` receives the completed schema declaration
and returns `Result<_, WorthQueryOutputReadinessContractDenial>`. Start
`WorthQueryOutputReadinessContractBuilder::for_field(declaration,
PrimaryField::reference::<Schema>(), ...)`, add typed `field_dependency` entries
and call `build_typed`. Each resolves the complete declared aspect, including
whole-aspect absence, required/optional siblings and codec digests. The exact
concrete value binding and full aspect are checked against installed catalogs
before configuration or initial-state effects.

`Delivery`, `Eligibility` and `DeliveryAndEligibility` express distinct uses.
All remain native watched inputs; eligibility selects evaluation triggers.
Retain the generated contract during installation and use
`contract.delivery_ordinal(installed, node, Field::reference())` for an explicitly
declared delivery field. It selects the genuine canonical ordinal and refuses
eligibility-only fields or a different node. Consume it with
`into_package_contract(binding.portable().clone())`. The
[typed readiness examples](../workspaces/worth-query/crates/worth-query-court-execution-primary-graph-application-contribution/tests/worth_query_court_execution_primary_graph_application_contribution/domain_computation/primary_graph/application_contribution/conditional/output_readiness_contract/typed/tests/mod.rs)
exercise full optional aspects, nested wire changes, concrete binding mismatch
and reordered delivery/eligibility inputs.

The executable [nested-value model](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_value/model.rs)
and [native checkpoint journey](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_value.rs)
exercise nondefault values through the entry binding, installed catalog,
authenticated read and current-format native recovery. The
[compiler fixtures](../crates/worth-court-foundational-expressions/tests/worth_court_foundational_expressions/compile_fail/foundational/value_shape/compiler/mod.rs)
cover supported and refused grammar.

Read canonical fields with `reader.field(...)` or the installed query's row
reader. The installed codec validates canonical grammar, representation bounds,
binding identity and domain values under the original request's controls.
For an opaque canonical value, implement `deserialize_admitted` and declare
`SUPPORTS_ADMISSION` before installing its binding.

Propagate a decoder or request-control refusal from the projection. The first
refusal remains attached to the read, so catching its error still prevents
`complete` from sealing facts. Retry with a fresh authorized request.
`begin_application_read_attempt` shares that state across repeated observations;
managed collection-pair reads share it until their atomic view decision.
Continuation and live delivery retain it through finalization and recheck the
request after application projection returns. See the
[canonical read examples](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_admission.rs),
[read-attempt example](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_admission/ordinary_attempt.rs)
and [paired-read example](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_admission/paired.rs).

For several reads from a retained invariant snapshot, use
`snapshot.read_fields(&execution, |read| { /* read.field(&identity, field) */ })`.
The closure borrows the existing native pin and shares decode state across its
field reads. A field refusal or late cancellation discards the closure's output.
`snapshot.field(&execution, &identity, field)` remains a single completed read.
See the [retained snapshot example](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_admission/retained_snapshot.rs)
for fresh-request retry at the original version after a source edit.

Representation bounds, request grants, resident retained-size accounting and
derived-codec transition permission remain separate contracts. Package archive
export of a canonical field currently refuses
with `UnsupportedCanonicalValueContract` before emitting its payload; native
application checkpoints use their existing owner and support the journey above.

## 3. Declare features and the program

A **program** is a type that implements `ApplicationProgramDefinition<Schema>`.
It names its contributions, lists its features, and declares its output graph
and its rules. Everything is typed: a connection between two ports whose
values differ does not compile.

All names in this section are on
`worth_query_decl::facade::application_program`, which is also reachable as
`worth_query_host::facade::declaration::application_program`.

### 3.1 Declare a feature

A feature is a marker type with a stable identity and a list of input ports.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/composition.rs
pub struct BankAccountsFeature;
pub struct BankPaymentsFeature;

impl ApplicationFeature<BankSchema> for BankAccountsFeature {
    type Inputs = ApplicationFeatureInputLeaf;   // no input ports
    const IDENTITY: &'static str = "worth.bank.feature.accounts.v1";
}

impl ApplicationFeature<BankSchema> for BankPaymentsFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.bank.feature.payments.v1";
}
```

`ApplicationFeature` also has `const MAJOR: u16 = 1` and `const MINOR: u16 = 0`,
which you may override.

### Complete feature definitions

Use `worth_query_host::facade::worth_query_application_contribution!` with its
`pub feature` form for an ordinary installed feature. Keep genuine schema
members in its `schema` clause, source actions in `action` members and paired
production in `required_pair` members. A source-only feature declares its actions;
a producing feature can also declare its Initial/Preserve pair. The same
definition provides:

| Projection | Owner and use |
|---|---|
| Schema members and operation access | `ApplicationFeatureDefinition::declare_members` contributes the feature's declared members to its schema |
| Authored program meaning | `ApplicationFeatureDefinition::feature_specs` supplies the feature specifications |
| Delivery/output roots | `ApplicationFeatureDefinition::Outputs` supplies roots derived from the declared deliveries |
| Required runtime contracts | `WorthQueryFeatureDefinition::contracts` projects requirements into contribution installation |
| Runtime configuration | `WorthQueryFeatureDefinition::Configuration` and `configure` carry actual handlers and role runtimes into the existing setup owner |

Compose definitions as typed tuples when appropriate. The application still
chooses its contributions, feature membership, connections and domain
configuration explicitly. Supply the handlers and readiness predicates as
runtime values, then execute actions through authorized application requests.
Keep domain and numerical types in their own crates.

The required-role example below is a complete feature definition with real
Initial and Preserve producers. Its linked executable includes schema/program
composition, configuration, installation, an authored edit and identity-preserving
publication. For an action member, compare the consumer's
[source-adjustment definition](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/source_adjustment/feature.rs),
whose delivery names its actual dependent output tree.
Use the lower-level `ApplicationFeatureSpec` builders for specialized composition
and for inspecting the meaning supplied by a definition.

### Required Initial and Preserve output roles

Use one Host-side alias for a producer feature requiring both lifecycle roles.
The ordinary Topology consumer binds its existing producers and readiness
owners this way:

<!-- compiled: final_output/feature.rs -->
```rust
pub type PlanarFinalFeatureRoles<Schema, Feature = PlanarFinalOutputFeature> =
    WorthQueryRequiredFeaturePair<
        Schema,
        Feature,
        PlanarFinalOutputProducer<Schema>,
        PlanarFinalOutputReadiness<Schema>,
        PlanarFinalPreserveProducer<Schema>,
        PlanarFinalPreserveReadiness<Schema>,
    >;
```

Select a complete feature definition through the Host facade's existing
`worth_query_application_contribution!` macro. The same typed members supply
schema operations and access, feature declarations, runtime slots, contracts and
output roots. The runnable consumer uses this definition:

<!-- compiled: final_output/definition.rs -->
```rust
worth_query_host::facade::worth_query_application_contribution! {
    pub feature PlanarFinalFeatureDefinition<Schema: TopologySchemaBinding>
        for PlanarFinalOutputFeature => CompletePlanarFinalRuntime {
        schema: |schema| { schema };
        provides(crate::PlanarFinalBodyOutput);
        required_pair lifecycle(
            PlanarFinalFeatureRoles<Schema>, FinalPlanarMutationHandler, FinalPlanarPreserveHandler
        ) {
            initial_operation: PublishFinalPlanarOutput::reference::<Schema>().definition()
                .no_external_effect().no_aftermath().finish();
            preserve_operation: PreserveFinalPlanarOutput::reference::<Schema>().definition()
                .no_external_effect().no_aftermath().finish();
        };
    }
}
```

Query definitions in a feature's schema block remain unfinished builders:
`schema.application_query(query_definition())`. Composition builds the query and
retains its feature owner, query name and typed cause on refusal; callers do not
unwrap a query builder. Correction `mechanism()` callbacks likewise construct
retention demands directly, without `expect`; association checks the complete demand.

`ApplicationFeatureDefinition<Schema>` exposes the owner-scoped `declare_members` entry, an infallible `members` callback, `feature_specs`
and its actual typed `Outputs`. `WorthQueryFeatureDefinition<Schema>` supplies
`Configuration`, `contracts` and `configure` through the existing installation
owners. A tuple composes these definitions, so an application selects each family
once. The `schema` clause owns genuine entities, fields and queries; action and
role operations are generated from their members.

An ordinary member uses `action name(Binding, Delivery, HandlerType)` with its
explicit completed `operation:` definition and the normal action clauses.
When another member already declares an action's operation and mutation
binding, use `action_ref name(Binding, Delivery, HandlerType) { ... };` with the
normal action clauses and no `operation:` clause. The feature retains its action,
delivery roots and handler configuration while that member remains the sole
operation owner. The owner may be a correction or an ordinary member. A missing
operation or binding returns `MissingReferencedMutation` at schema composition,
naming both; declaring either twice still returns `DuplicateMember`.

```rust,ignore
action_ref create(CreateBinding<Schema>, SourceDelivery<Schema>, CreateHandler) {};
```

The generated runtime constructor accepts actual handler values in action order,
then two-role runtimes in pair order. Supply each runtime's domain configuration
explicitly.
Each role remains `WorthQueryFeatureRoleRuntime::new(handler, provider,
readiness_configuration)` inside `WorthQueryFeatureRuntime::new(initial, preserve)`.
Readiness predicates and `bind_output_readiness` remain with their existing owners.

Declare operation access once with `worth_query_operation!(... for Schema:
SchemaBinding, input Input; access { ... })`. Both Rust capabilities and schema
access come from those categories. Where a sealed capability family already owns
inherited access, the optional `inherited_access: |schema, operation| { ... }`
continuation calls that capability family's access declaration.
Delivery roots come from each actual delivery mode. Identical concrete roots
are visited once before flattening; distinct roots still undergo the normal
duplicate-connection checks.

Program meaning retains both requirements independently of those callbacks.
Omitting contracts and registration together is refused. Configured `None`
roles, handlers, providers or readiness values preserve those requirements and
return the original typed installation cause before initial state or output
publication. Output-family `SUPPORTED` lists describe applicability; they do
not declare mandatory installation roles.

The runnable [required-feature example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/examples/required_feature.rs)
uses the complete public schema/program/configuration/install path, publishes
an Initial value of 3, edits authored source, and preserves the original entity
with value 11. Run it from the repository root:

```text
cargo run --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-topology-entry --example required_feature
```

`DecisionReader::prior_output_preserved_or_created` selects the latest publication
of its declared Create/Preserve bindings in one installed output family, exact
source partition and selected ancestry. The selected typed role must still be
a live full-generation entity. A stale or missing member is a denial; it never
falls back to an older binding. Ordinary `prior_output` keeps its exact-binding
historical meaning. Query owns source-native-version commitment, completed
dependencies and current publication custody; a separate application command
nonce is not a replacement for that contract.

### 3.2 Attach actions to features

`ApplicationFeatureSpec` records which feature owns which actions. Begin a
feature with `root::<Schema, Feature>()` or
`at::<Schema, Instance, Feature>()`, then begin each ordinary action with
`.action::<Binding, Delivery>()`. Delivery is mandatory and names the real
source-completion, retained-commit, fixed-output or discovered-output contract.
Attach the action's optional metadata before `.finish_action()` returns the
feature parent. Only then can you add another action or finish the feature.

The neutral consumer's actual fixed source action retains locality/change,
evaluated requirement and external-input metadata alongside its real output
delivery. This excerpt uses the surrounding imports and source-feature members
from [the compiled declaration owner](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/composed_action.rs):

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

| On the feature parent | Result |
|---|---|
| `action::<Binding, Delivery>()` | An unfinished ordinary action with explicit delivery |
| `native_action::<Binding>()` | An unfinished deliberately specialized native action, without delivery |
| `conditional_native_action::<Binding>()` | A concrete mutation binding owned only by its installed conditional, without ordinary delivery |
| `operation::<Op>()` | A declared operation on the root-composition builder |
| `conditional_operation::<Op>()` and `conditional_operation_with_requirement_and_external_input` | A conditional operation |
| `provides::<Port>()` | An output port this feature provides |
| `derived_artifact`, `managed_computation`, `derived_collection` | Derived outputs the feature produces |
| `finish()` | The complete `ApplicationFeatureSpec` |

| On the unfinished action | Contract |
|---|---|
| `locality_and_change::<Scope, Shape>()` | The paired locality and change shape |
| `correspondence::<Correspondence>()` | Correspondence for this mutation binding |
| `evaluated_requirement::<Rule>()` | An evaluated rule for the binding's operation |
| `external_input::<Provider>()` | An external-input provider for the binding's operation |
| `finish_action()` | Append exactly one complete declaration and return its feature parent |

Each optional clause consumes its unconfigured state; setting it twice is a
compiler error. All subsets are legal under their existing trait bounds,
including a rule without a provider and a provider without a rule. No incomplete
action is appended, and the action builder has no feature `finish()` or
`provides()` method. Ordinary delivery derives required-output-source meaning;
there is no output-source override. A source-only declaration cannot erase a
binding's actual required-output obligation: owning construction refuses it.

`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>`
selects this exact occurrence. Its sealed `ApplicationAction::selector()`
returns selection facts, not a complete declaration. Installation resolves the
complete actual action, including every metadata contract; fresh progression
compares that full installed meaning. Duplicate binding types and duplicate
portable `(instance, feature, binding identity)` keys are rejected. Distinct
legal composition occurrences remain distinct.

Correspondence lookup is binding-scoped. Rule/provider capabilities are scoped
to operation plus rule/provider type across declared actions; they do not become
occurrence-isolated permission. Installing a clause neither evaluates a rule nor
admits an external revision. Use the installed evaluator's result for guidance
and submission enforcement; native authorization remains with its existing owner.
Resolve external input, admit its captured revision and pass the accepted values
into the original mutation intent.

If admission refusal must be resumable, use
`WorthQueryCapturedExternalInput::admit_preserving(&provider)`. Its error returns
the exact `(original_capture, provider_denial)`; retain that capture and retry
validation against it. Selection, values, revision and provenance remain unchanged.
Resolving the provider again produces a different observation, not recovery of
the original one. The existing `admit` is the one-shot form: it intentionally
discards the capture on refusal. Neither form grants mutation permission.

The `worth_query_feature_spec!` macro on the decl facade lowers
`action(Binding, Delivery) { ... };`, `native_action(Binding) { ... };` and
`conditional_native_action(Binding) { ... };`
through that same builder. Clauses are `locality_and_change(Scope, Shape);`,
`correspondence(Correspondence);`, `evaluated_requirement(Rule);` and
`external_input(Provider);`. Empty blocks are valid. Clause order preserves
canonical meaning; unsupported or repeated clauses fail rather than disappear.
See [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md).

Use `native_action` only for a present specialized native responsibility, such
as staging, authority/phase certification or explicit branch/host control.
Ordinary product mutations and retries use explicit owning delivery, including
when a native caller also consumes that occurrence. Native required-output
membership comes from actual declared roots; there is no native output toggle.

When an installed conditional owns a mutation binding that requires program
membership, declare that binding with `.conditional_native_action::<Binding>()`
and `.finish_action()`, or the equivalent capsule clause:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
worth_query_feature_spec!(root(ProductSchema, LoadFeature);
    conditional_native_action(ResolveLoadBinding) {};
)
```

The binding supplies its operation, input, identity and workflow requirements.
The constructor declares conditional ownership and no ordinary delivery.
`conditional_operation(Op)` alone declares no mutation binding membership;
adding a separate `native_action(Binding)` conflicts with conditional ownership.
Install the real matching conditional, handler and producer in the contribution.
The exact declared and installed conditional operation sets must still agree.

### 3.3 Declare the program

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/application_definition/composition.rs
pub(crate) type BankRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<BankSchema, BankPaymentsFeature, BankPostingIntegrity>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

impl ApplicationProgramDefinition<BankSchema> for BankApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs()
    }
}
```

| Item | Meaning |
|---|---|
| `type Contributions` | A tuple of the contributions this program installs, in order. It fixes the type of the `configuration` argument at install. |
| `type Outputs` | The program's output graph, wrapped in `ApplicationProgramOutputs<..>`. Use `ApplicationNoOutputGraph` when there is none. |
| `type Rules` | A type-level list of rules: `ApplicationRuleList<ApplicationRuleAt<Ref, Point>, Tail>`, ended by `ApplicationRuleLeaf`. |
| `const IDENTITY` | The program's stable identity. |
| `const DERIVED_ARTIFACT_GOVERNANCE` | Optional. `ApplicationDerivedArtifactGovernance::{Compatible, Required}`; defaults to compatible. |
| `fn feature_specs()` | The features and the actions each one owns. |

**Rules.** A rule reference is one of:

- `ApplicationLocalRuleRef<Schema, Feature, Invariant>`: owned by one feature;
- `ApplicationSharedRuleRef<Schema, Invariant>`: owned by no single feature;
- `ApplicationLocalRuleInstanceRef` / `ApplicationSharedRuleInstanceRef`: the
  same, inside a composition instance.

The execution point is `ApplicationCommitBoundary`,
`ApplicationMutationSensitive`, or `ApplicationSnapshotPublication`. The rule's
handler is still registered in the contribution's `configure`, with
`setup.invariant(..)` ([Â§2](#2-declare-the-schema-and-its-contributions)). A
program selects which declared rules govern its branches.

### 3.4 Ports, connections, and the output graph

Features can pass typed values to each other. A feature *provides* an output
port; another feature *requires* an input port; a connection binds the two;
the program's output graph says which connections produce settled outputs.

For a generated child, keep its source and its realization in **separate
records**, joined by a declared typed relation. The parent produces and owns the
child's source record; the child produces its own realization and any next-level
sources. A declared relation names each source's immediate parent. A parent's output
witness covers the whole source entity, including absent aspects: a child writing
its realization onto that source invalidates the parent's witness. Corrections
to a generated source therefore go through its owning producer, from that
producer's input. See the [generic split-record courts](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/dependent_dispatch/split_courts.rs)
for installed roots, sealed dependent dispatch and current delivery.

The block below is a **shape with illustrative names**. The compiled
counterpart is the consumer fixture at
`workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/application_program.rs`
(features and ports) and
`.../fixtures/consumer_entry/consumer_root/src/application_program/roots.rs`
(the output graph). That fixture takes every Query name from the two facades.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Shape. Illustrative names; see the fixture above for compiled code.
pub struct DraftFeature;
pub struct SummaryFeature;
pub struct DraftOutput;
pub struct DraftInput;
pub struct DraftToSummary;

impl ApplicationFeature<MySchema> for DraftFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "example.draft-feature.v1";
}
impl ApplicationFeature<MySchema> for SummaryFeature {
    type Inputs = ApplicationFeatureInputList<DraftInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "example.summary-feature.v1";
}

impl ApplicationOutputPort<MySchema, DraftFeature> for DraftOutput {
    type Value = DraftValueBinding;            // an ApplicationStructuredValueBinding
    const IDENTITY: &'static str = "draft";
}
impl ApplicationInputPort<MySchema, SummaryFeature> for DraftInput {
    type Value = DraftValueBinding;            // must equal the output's Value
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl ApplicationOccurrenceConnectionBinding<MySchema, DraftFeature, SummaryFeature>
    for DraftToSummary {}
impl ApplicationConnectionIdentity for DraftToSummary {
    const IDENTITY: &'static str = "example.draft-to-summary.v1";
}

type DraftConnection = ApplicationConnectionRef<
    MySchema, DraftFeature, DraftOutput, SummaryFeature, DraftInput, DraftToSummary,
>;

// The output graph: a root connection and its dependents.
pub type MyOutputRoot = ApplicationOutputGraph<DraftConnection, ApplicationOutputLeaf>;

// In the program:  type Outputs = ApplicationProgramOutputs<MyOutputRoot>;
```

- `ApplicationConnectionRef` only exists when the target port's `Value`
  equals the source port's `Value`. A mismatch is a **compile error**, not a
  validation denial.
- Dependents chain with `ApplicationOutputEdge<Connection, Dependents>`.
  Several roots are a tuple: `ApplicationProgramOutputs<(RootA, RootB)>`.
- **Composition instances.** `trait ApplicationCompositionInstance { const PATH: &'static str; }`
  names a place in the program; `ApplicationRootComposition` is `"root"`. A
  feature placed with `ApplicationFeatureSpec::at::<S, Instance, F>()` lives
  there. A connection that crosses instances uses `ApplicationConnectionInstanceRef`
  and must set `const EXPORTS_ACROSS_INSTANCES: bool = true` on its binding.
  A PATH does not allocate a live occurrence; it names a position in meaning.
- The host side of an output connection implements
  `WorthQueryApplicationRequiredOutputConnection<Schema>` (on
  `worth_query_host::facade::primary_graph`), which turns the source mutation's
  input into an output demand.

### 3.5 Validate

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/application_definition/composition.rs
pub(crate) fn validated_bank_application() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplication>,
    ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplication>::begin().validated_program()
}
```

- The result is a `ValidatedApplicationProgram<Schema, Program>`. Its fields
  are private. Read it back with `identity()`, `revision()`, `features()`,
  `actions()`, `connections()`, `rules()`, `semantic_description()`, and
  `normalized_manifest()`.
- `revision()` is the content-addressed `ApplicationProgramRevision`: the same
  meaning always gives the same revision.
- Validation uses `ApplicationProgramDefinition::CANONICAL_REVISION_BUDGET`,
  a finite `CanonicalDigestWorkBudget` available from the same program facade.
  Its default remains 4096 basis entries and 1 MiB encoded bytes. Large authored
  programs may override that associated constant; exceeding either ceiling is
  a typed `CanonicalRevisionBudgetExceeded` denial. The budget governs cold
  declaration work, is not part of the manifest, and does not change the revision
  when the accepted program meaning is identical. It grants no runtime resource
  or publication permission.
- A denial is an `ApplicationProgramValidationDenial` with `kind()` and
  `subject()`. Kinds include `DuplicateFeature`, `DuplicateAction`,
  `DanglingFeature`, `MissingRequiredInput`, `UndeclaredOutput`,
  `UnexportedCrossInstanceConnection`, `CyclicConnection`, `DuplicateRule`,
  `IncompleteActionChangeContract`, and `UngovernedDerivedOutput`.

### 3.6 Declare and install a partitioned managed computation

A managed computation declares its input, output artifact, partition key,
execution posture and determinism contract. Its execution uses the caller's
request for cancellation, deadlines, placement and allocation controls.
Add the computation
with `managed_computation::<Computation>()` beside its `derived_artifact` in the
feature specification. Install its owner with
`setup.partitioned_computation::<Feature, Computation, _>(owner)` and retain the
returned handle in the handler that uses it.

The owner separates reads from computation. `partitions`, `partition_key`, and
`gather` receive an operation-scoped reader on the caller thread. They can read
only what the operation declares. `compute_partition` receives gathered data
and a checkpoint, with no graph reader. `partitions`, `partition_key`, `gather`, and
`compute_partition` must be pure in the declared input, partition key, and
facts read through the reader: no ambient clocks, mutable globals, changing
owner state, or effects. A completed producer run keeps its partition state.
The next run of the same installed producer, on an equal input value, reuses
an unchanged partition without gathering or computing it again. An impure
function can therefore yield a stale reused result. `complete` receives the
reduced result back on the caller thread. Preparation precedes kernel dispatch; completion
follows successful computation and reduction.

In the executable examples below, `run_inventory` and `run_peak` call
`installed.decision_plan(reader, input, complete)`. Query owns preparation,
dispatch, reduction, owner completion and native failure conversion. The callback
receives only the owner's completed value and returns a `HandlerResult`.
The plan carries compute inputs, not a live reader. Use this entrance when a producer's computation can run beside
another required producer: Query dispatches compute and invokes completion in
canonical member order. Override `OperationHandler::prepare_decision` to return
this plan. Keep graph reads in preparation and return the decision from
completion; the ordinary candidate-building and commit steps follow on the
owner. A handler that does not compute can keep its `decide` implementation:
the default `prepare_decision` wraps it in an immediate plan.

An installed single-kernel computation uses the same owner lifecycle through
`installed.decision_plan(input, complete)`. Its owner declares `Prepared`,
`Computed`, `Output` and `Stopped`, and implements `prepare`, `compute` and
`complete`. `Prepared` and `Computed` are opaque domain data; the caller cannot
construct a completed SDK value or bypass the owner's validation. A genuine
domain outcome such as an unavailable engineering response belongs in the
owner's output when it is a successful domain settlement.

Use `checkpoint.poll()?` at a supported kernel safe point to check the original
request without supplying a work invoice. The existing `advance(work)` is for
owners that deliberately charge request work. A stop remains final even when a
child ignores it and returns apparent success. Query checks before and after
compute and at the owner handoff. A synchronous numerical kernel still has its
own cancellation contract; polling before it does not make it preemptible.

`HandlerFailure<Domain>` carries explicit `Domain`, `Execution` and `Interrupted`
failures. `HandlerResult::from_result` and `into_result` convert that same typed
topology; `map_domain` changes only the domain case. Native allocation, read,
panic and nested-execution causes retain typed execution failures; cancellation
and deadline remain interruptions. There is no blanket arbitrary-error-to-domain
conversion. A genuinely infallible domain can use `Infallible`.

Partition reuse is distinct from clean reuse of an entire output and from
input cutoff. It applies only to the runs of a producer. A producer is a
`WorthQueryApplicationProducerBinding`; its `Operation` is the mutation binding
the producer runs. For a required Initial/Preserve feature, the
[required pair](#required-initial-and-preserve-output-roles) projects its
contracts and registration. Installation decides
retention once: it compares the owner's `Operation` type with the `Operation`
of each declared producer's mutation binding. When a declared producer runs that
operation, a run made under that producer leaves its partition state on the
record its attempt publishes. When no declared producer runs it, a run is never
retained: no input or item digest is taken, and every run gathers and computes
every partition.

A retained computation still gathers and computes every partition when any of
these holds:

- No output record is published at the run's address: the first run.
- The operation is invoked outside a producer invocation, or an earlier
  partitioned computation in the same handler already took the kept state. A
  handler that runs several partitioned computations in one attempt keeps none
  of them.
- The installed owner differs, the installed producer edition differs, or the
  input value's canonical encoding differs. A reinstalled owner with the same
  types is a different installation.
- Observing the kept facts could exceed an explicitly supplied request work
  ceiling, or its checked observation bound cannot be represented.
- The selected record holds no computation state. A run that stopped before it
  completed, whose charge could not be measured, or whose reservation was
  refused leaves none. A previous attempt that ran no partitioned computation
  leaves none. Checkpoint restoration and republication carry none. A record
  whose successor took over its state holds none.

Otherwise the run names and keys again only what changed, and gathers and
computes again only the partitions whose members or facts changed. A partition
is carried only when every fact its owner calls read in the previous run has
the same content at this attempt's snapshot; a fact that cannot be observed
counts as changed. When the exact retained settlement and selected native
snapshot preserve complete change evidence, Query compares only the native
owner's dirty computation facts. Ordinary output verification can clear its
marks without discarding that computation frontier. History gaps, missing
registrations, full re-establishment, and superseded own-write evidence retain
full validation; an installation or edition change cannot reuse the old state.
The completion consumes carried-fact validation only at the same pinned reader
snapshot, after the ordinary declared read closure and preconditions pass.
Applications supply no dirty list or freshness declaration.

Call the same installed computation on subsequent attempts. Query retains its
successful partition results, validates the tracked facts against the selected
snapshot and recomputes changed partitions. Source registration and publication
use that attempt's current authorization and source evidence.

Keep item identities and partition keys stable across attempts. Membership or
key changes trigger full preparation; recovery, missing history or an incompatible
installation also requires reconstruction. The runtime determines reuse from
tracked reads and retained state. Continue to poll the request during computation
and handle cancellation, deadlines and allocator failures on every attempt.

The example groups warehouse quantities by storage zone and sums them. Stable
item IDs identify inventory lines; the zone key defines partition membership.
Neither depends on worker assignment. The generic schema, feature, artifact,
and operation parameters stand for your application's declarations. The
`Reuse` marker names declaration vocabulary; partition reuse depends on the
state kept from the previous run and the purity contract above. Both examples
in this section are compiled as written: the consumer fixture builds and runs
`workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/warehouse_inventory.rs`
and `.../sensor_telemetry.rs`, and a test fails when a block here and its file
differ.

The executable examples are exercised by
[`compiled_examples_warehouse_inventory_total` and `compiled_examples_sensor_telemetry_peak`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/compiled_examples.rs).
They assert a warehouse sum of `13.0` and a sensor maximum of `850` through a
host mutation.

<!-- compiled: warehouse_inventory.rs -->
```rust
use serde::Serialize;
use std::convert::Infallible;
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_operation::ApplicationMutationBinding, application_program::*,
        application_schema::ApplicationSchema,
    },
    primary_graph::{
        DecisionReader, HandlerResult, WorthQueryDecisionPlan,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

#[derive(Clone, Serialize)]
pub struct InventoryLine {
    id: u64,
    zone: u64,
    quantity: f64,
}
impl ApplicationComputationPartition for InventoryLine {
    const IDENTITY: &'static str = "inventory.line.v1";
}

pub struct InventoryInput;
impl ApplicationComputationInput for InventoryInput {
    type Value = Vec<InventoryLine>;
    const IDENTITY: &'static str = "inventory.input.v1";
}
#[derive(Serialize)]
pub struct Zone(u64);
impl ApplicationComputationPartition for Zone {
    const IDENTITY: &'static str = "inventory.zone.v1";
}
pub struct InventoryReuse;
impl ApplicationComputationReuse for InventoryReuse {
    const IDENTITY: &'static str = "inventory.reuse.v1";
}
pub struct InventoryStopped;
impl ApplicationComputationStopped for InventoryStopped {
    const IDENTITY: &'static str = "inventory.stopped.v1";
}

pub struct InventoryTotal<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for InventoryTotal<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = InventoryInput;
    type Output = A;
    type Partition = Zone;
    type Reuse = InventoryReuse;
    type Stopped = InventoryStopped;
    const IDENTITY: &'static str = "inventory.total.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    // DETERMINISM defaults to canonical bitwise equality.
}

pub struct InventoryOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, InventoryTotal<A>>
    for InventoryOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = InventoryLine;
    type Gathered = Vec<f64>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = Infallible;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<InventoryLine>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<InventoryLine>,
        WorthQueryComputationInputDenial<Infallible>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |line| PartitionItemId(line.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        item: &InventoryLine,
    ) -> Result<Zone, WorthQueryComputationInputDenial<Infallible>> {
        Ok(Zone(item.zone))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        partition: WorthQueryComputationPartitionMembers<'_, Zone, InventoryLine>,
    ) -> Result<Vec<f64>, WorthQueryComputationInputDenial<Infallible>> {
        Ok(partition.items().map(|(_, line)| line.quantity).collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Zone, Vec<f64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<Infallible>> {
        let mut total = 0.0;
        for quantity in partition.gathered() {
            checkpoint.poll()?;
            total += quantity;
        }
        Ok(total)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| 0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, Infallible> {
        Ok(reduced)
    }
}

pub fn declare_inventory<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<InventoryTotal<A>>()
        .finish()
}

pub fn install_inventory<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, InventoryTotal<A>, InventoryOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, InventoryTotal<A>, _>(InventoryOwner(PhantomData))
}

pub fn run_inventory<S, F, A, Op, B, Value: 'static, Domain: 'static>(
    installed: &WorthQueryInstalledPartitionedComputation<
        S,
        F,
        InventoryTotal<A>,
        InventoryOwner<Op>,
    >,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<InventoryLine>,
    complete: impl FnOnce(f64) -> HandlerResult<Value, Domain> + 'static,
) -> WorthQueryDecisionPlan<Value, Domain>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
    B: ApplicationMutationBinding<S, Operation = Op>,
{
    installed.decision_plan(reader, input, complete)
}
```

A derived collection additionally names its `Computation` and `Completeness`
associated types. `Completeness` implements `ApplicationCollectionCompleteness`:
its `Value` and `Stopped` types must match the installed partitioned owner's
output and stop types, and its `Lineage` must match the collection declaration.
Declare both the collection and its computation on the feature, then install
with `setup.derived_collection::<Feature, Collection, _>(owner)`. Ordinary
`partitioned_computation` registration cannot satisfy a declared collection.
Installation checks the exact types before the initial-state callback or
checkpoint restoration. The existing completion path checks the actual value
before allowing publication; it does not grant graph access to the check.

Completeness remains a domain claim requiring runtime evidence. Keep known empty
membership distinct from missing evidence, and retain pending physical measures
when their required contributors are unavailable. A membership-only count need
not gather geometry. Managed collection views present admitted rows; they do not
replace the producing computation or establish aggregate completeness. The
[neutral region collection](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/collection.rs)
executes the checked binding through real publication and retained canonical
reduction, with missing/misbound installation and completion-refusal twins.

Initial/Create and Preserve/Preserve producers can share one operation, input
type and installed checked owner. Each producer selects its own mutation binding
and fixed exactly-one output role. The shared input implements
`ApplicationMutationInputScope<Schema, Binding>` for each selected binding;
it needs no ordinary mutation intent. This is authored selection, not a resolved
identity or an authorization grant. Query still resolves the installed binding,
scope, principal and source before contacting the handler.

Use the concrete installed schema for explicit associations. The region
fixture keeps its existing ordinary intent and adds these two associations:

<!-- compiled: checkpoint_recovery/computation_partition/oracle/collection/lifecycle/binding.rs -->
```rust
impl ApplicationMutationInputScope<CheckpointSchema, InitialBinding<CheckpointSchema>>
    for RegionOutputInput
{
    fn scope_for_binding(&self) -> PlanarMutationScope {
        PlanarMutationScope::new(BodyKey::at("BodyKey"), self.scope_key.clone())
    }
}
impl ApplicationMutationInputScope<CheckpointSchema, PreserveBinding<CheckpointSchema>>
    for RegionOutputInput
{
    fn scope_for_binding(&self) -> PlanarMutationScope {
        PlanarMutationScope::new(BodyKey::at("BodyKey"), self.scope_key.clone())
    }
}
```

The [lifecycle fixture](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/collection/lifecycle.rs)
installs the collection once and clones its installed handle for the two typed
handlers. The [native journey](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/collection/lifecycle/custody.rs)
checks real entry edits, output incarnation and retained comparison. Producer
edition changes may require a cold preparation before same-edition reuse.
The selected producer role stays fixed and exactly-one; the fixture also
publishes and preserves the two supporting members required by its native
planar topology, including their original incarnations.
Each producer has its own readiness binding with its exact producer dependency,
even when the operation and checked owner are shared. For a required pair, the
feature contract derives that association; do not copy it into a readiness
`REQUIRED_PRODUCERS` list. Other genuine conditional dependencies remain explicit.
When their complete portable package contracts are identical, contribution
composition emits that operation/node contract once; each binding still installs
and validates its own route. Different meanings for the same operation remain a
package conflict, refused before initial effects.
Publication validates the issued contribution-binding inventory, with exactly
one admitted route per binding; it does not mistake the number of shared
portable definitions for the number of separately installed bindings.
Before initialization, composition independently compares every installed
operation and workflow-stage target with the distinct declared targets. An
unbound target refuses even when every declared binding is configured. Stage
declarations select their full location with `package_contract.in_workflow_stage(stage)`;
a same-named operation node cannot stand in for a workflow-stage node.
`installed_conditional_targets_for_schema` exposes those complete read-only
locations for inspection; they grant no binding or execution authority.

A managed computation declares only itself. `managed_computation` adds its
declaration to the feature; the feature's operations, ports, and artifacts are
declared on the same builder. Program validation refuses a computation whose
output artifact its feature does not declare
(`MissingManagedComputationArtifact`), whose partition key does not fit its
execution posture (`MismatchedManagedComputationPartition`), or whose identity
repeats within the feature (`DuplicateManagedComputation`). Installation
refuses, with a `WorthQueryPrimaryGraphInstallationDenial`, an owner installed
through the call for the other execution posture, a computation whose
determinism contract is `ContractEquivalent`, and a second owner for one
computation.

Supply a unique, stable ID for every line: duplicate item identities are
denied. Keys use canonical encoding, and partitions are ordered by the
identity derived from that encoding, not by the numeric order of zones. The
reducer fixes its association tree, so floating-point reduction preserves the
same result bits at every admitted worker count.

Choose partition membership from the input and observed facts before placement.
The [installed worker policy](#one-process-execution-service) supplies a concrete
worker count. Serial placement runs on the caller's thread; Automatic placement
can use the same process service's admitted workers. Placement does not rename a
partition or change its reduction association.

Call `checkpoint.poll()` during computation to observe the original request's
cancellation, deadline and untaken nested stop. Propagate a typed denial before
authoring a candidate from the result. Completion returns the decision to the
candidate-building and commit steps.

During plan preparation, the runtime derives item, input and partition-key
identities through canonical encoding. The original request's controls are polled
while serializing values, ordering map entries and growing temporary buffers.
Encoding rejection, allocation failure and interruption propagate through the
same typed preparation outcome. A serializer that catches an interruption still
leaves that original interruption latched at the encoding boundary.

For an explicitly accounted request, `checkpoint.advance(work)` records work
against that request and its ancestors. Calling it in a worker-only request
returns `Resource(AccountingUnavailable)`; use `poll()` for control checks.

For specialized lower-level execution and diagnostics, `prepare`, `compute`, and `complete` each fail with a
`WorthQueryPartitionedComputationDenial<Stopped>`, where `Stopped` is the
owner's own stop type. A denied run has no result. Ordinary handlers use
`decision_plan` to propagate these causes rather than writing a dispatcher.
The lower-level denial topology is:

| Denial | What it reports |
|---|---|
| `Owner(stopped)` | The owner refused to name the input's items, to key one, or to complete the reduced result. |
| `Read(..)` | A read was refused while the owner named the items or keyed one. |
| `Partition { partition, cause }` | One partition stopped. `cause` is a `WorthQueryComputationPartitionStop`: `Owner(stopped)`, `Read(..)`, `Panicked`, `Resource(..)`, `Interrupted(..)`, or `NestedPatternStopped`. When several partitions stop, the one with the least partition identity is reported. No partition is computed unless every partition gathers. |
| `PartitionIdentityCollision { partition }` | Two different keys derive one partition identity. Every run is denied while both keys are in the input. |
| `DuplicateItem { item }` | The plan named one item identity twice; the least such identity is reported. |
| `InputNotEncodable(..)`, `ItemNotEncodable { item, denial }` | The input value or an item refused canonical encoding, so a retained run has no digest for it. |
| `KeyNotEncodable { item, denial }` | An item's partition key refused canonical encoding. |
| `Resource(..)` | A `WorthQueryManagedComputationResourceDenial`: the original request refused execution or allocation, or checked size/work arithmetic overflowed. There is no caller-supplied result-byte quote. |
| `Interrupted(..)` | `Cancelled` or `DeadlineExceeded`: the request was canceled or ran out of time outside any one partition. |
| `NestedPatternStopped` | A pattern started inside a partition's kernel stopped, outside any one partition. |
| `PreparationPanicked` | Input preparation or canonical identity encoding panicked before partition execution. |
| `ReducerPanicked` | The reducer panicked. |
| `ReducedEncodingInvalid` | A reduced value's canonical bits disagree with the length it declares. |
| `ReductionInputInvalid(..)` | The reduction refused the identities or values it was given. The platform builds both from one canonical list, so no run is expected to produce it; match it for exhaustiveness. |

Inside `compute_partition`, `checkpoint.poll()?` converts a refused
checkpoint (`WorthQueryManagedComputationCheckpointDenial`: `Resource`,
`Interrupted`, or `NestedPatternStopped`) into the kernel's
`WorthQueryManagedComputationDenial`. A refusal attributed to a partition
surfaces as `Partition { partition, cause }`; one that belongs to no partition
surfaces as the top-level `Resource`, `Interrupted`, or `NestedPatternStopped`.

A second example, from sensor telemetry, keeps the same traits and changes the
item, the key, and the reducer. It groups readings by station, takes each
station's peak, and reduces the peaks to one maximum. Integer maximum is exact,
so the reducer needs no floating-point care.

<!-- compiled: sensor_telemetry.rs -->
```rust
use serde::Serialize;
use std::convert::Infallible;
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_operation::ApplicationMutationBinding, application_program::*,
        application_schema::ApplicationSchema,
    },
    primary_graph::{
        DecisionReader, HandlerResult, WorthQueryDecisionPlan,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

#[derive(Clone, Serialize)]
pub struct Reading {
    id: u64,
    station: u32,
    millivolts: u64,
}
impl ApplicationComputationPartition for Reading {
    const IDENTITY: &'static str = "telemetry.reading.v1";
}

pub struct TelemetryInput;
impl ApplicationComputationInput for TelemetryInput {
    type Value = Vec<Reading>;
    const IDENTITY: &'static str = "telemetry.input.v1";
}
#[derive(Serialize)]
pub struct Station(u32);
impl ApplicationComputationPartition for Station {
    const IDENTITY: &'static str = "telemetry.station.v1";
}
pub struct TelemetryReuse;
impl ApplicationComputationReuse for TelemetryReuse {
    const IDENTITY: &'static str = "telemetry.reuse.v1";
}
pub struct TelemetryStopped;
impl ApplicationComputationStopped for TelemetryStopped {
    const IDENTITY: &'static str = "telemetry.stopped.v1";
}

pub struct PeakReading<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for PeakReading<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = TelemetryInput;
    type Output = A;
    type Partition = Station;
    type Reuse = TelemetryReuse;
    type Stopped = TelemetryStopped;
    const IDENTITY: &'static str = "telemetry.peak.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}

pub struct PeakOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, PeakReading<A>> for PeakOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = Reading;
    type Gathered = Vec<u64>;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = Infallible;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<Reading>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Reading>,
        WorthQueryComputationInputDenial<Infallible>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |reading| PartitionItemId(reading.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        item: &Reading,
    ) -> Result<Station, WorthQueryComputationInputDenial<Infallible>> {
        Ok(Station(item.station))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        partition: WorthQueryComputationPartitionMembers<'_, Station, Reading>,
    ) -> Result<Vec<u64>, WorthQueryComputationInputDenial<Infallible>> {
        Ok(partition
            .items()
            .map(|(_, reading)| reading.millivolts)
            .collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Station, Vec<u64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<Infallible>> {
        let mut peak = 0_u64;
        for millivolts in partition.gathered() {
            checkpoint.poll()?;
            peak = peak.max(*millivolts);
        }
        Ok(peak)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| (*left).max(*right))
    }

    fn complete(&self, reduced: u64) -> Result<u64, Infallible> {
        Ok(reduced)
    }
}

pub fn declare_peak<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<PeakReading<A>>()
        .finish()
}

pub fn install_peak<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, PeakReading<A>, _>(PeakOwner(PhantomData))
}

pub fn run_peak<S, F, A, Op, B, Value: 'static, Domain: 'static>(
    installed: &WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<Reading>,
    complete: impl FnOnce(u64) -> HandlerResult<Value, Domain> + 'static,
) -> WorthQueryDecisionPlan<Value, Domain>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
    B: ApplicationMutationBinding<S, Operation = Op>,
{
    installed.decision_plan(reader, input, complete)
}
```

Planning does not depend on the order the input holds its readings in: items
are met in item identity order and partitions in partition identity order.

To use either example, declare the artifact and managed computation on the
feature, install the owner during contribution setup, and store the returned
handle with that operation's handler. Call its `run_inventory` or `run_peak`
function from `prepare_decision`. Declare every graph fact read by
`partitions`, `partition_key`, and `gather` on the operation. For retained
per-partition reuse, also declare and install the producer that runs that
operation; a direct mutation computes successfully but keeps no producer state.
The inventory example sums warehouse quantities; the telemetry example reduces
sensor peaks. Both use the same consumer API, through `worth-query-host`'s
facade and the declarations exported by `worth-query-decl`.

**The handler contract.** Within `OperationHandler<Schema, Binding>`, the
current signature and default implementation are:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
    fn prepare_decision(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        WorthQueryDecisionPlan::immediate(self.decide(input, reader))
    }
```

Override this method to return the prepared computation's decision plan. The
following complete handler excerpt is compiled in the planar geometry example.
Its `Decision` alias expands the reader's three lifetimes and binding. Its
installed `run` prepares the warehouse or telemetry computation shown above;
completion maps the computed result to the operation's decision. `decide`
resolves a declared scope and builds its mutation target. `build_candidate`
authors the field change after computation completes.

The fixture's schema declares `Body`, `PositionY`, and the entry-set facts;
`RegionTotalsDemandBinding` binds those reads and writes. The full declarations
and installation accompany this excerpt in the consumer fixture.

<!-- compiled: checkpoint_recovery/computation_partition/demand.rs -->
```rust
pub(super) type Decision<'borrow, 'reader, 'runtime, Schema> =
    DecisionReader<'borrow, 'reader, 'runtime, Schema, RegionTotalsDemandBinding<Schema>>;

/// What the handler runs over the demanded set inside its decision.
type Run<Schema> = Box<
    dyn Fn(
            &mut Decision<'_, '_, '_, Schema>,
            &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
        ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
        + Send
        + Sync,
>;

/// Decides a demand: runs what it was installed with over the demanded set,
/// then moves the scope's source so every decision is a new one.
pub(super) struct RegionTotalsHandler<Schema: TopologySchemaBinding>(Option<Run<Schema>>);

impl<Schema: TopologySchemaBinding> RegionTotalsHandler<Schema> {
    /// The topology's own handler: nothing to run.
    pub(super) const fn idle() -> Self {
        Self(None)
    }

    pub(super) fn running(
        run: impl Fn(
                &mut Decision<'_, '_, '_, Schema>,
                &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
            ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self(Some(Box::new(run)))
    }
}

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, RegionTotalsDemandBinding<Schema>>
    for RegionTotalsHandler<Schema>
{
    fn decide(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial> {
        let decided = (|| {
            let scope = reader.resolve_entity(BodyKey::at("BodyKey"), input.scope_key.clone())?;
            if reader.field(&scope, PositionY::at("PositionY"))?.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&scope).map(Some)
        })();
        match decided {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn prepare_decision(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> WorthQueryDecisionPlan<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial>
    {
        let plan = if let Some(run) = &self.0 {
            let set = match reader
                .resolve_entity(EntrySetKey::at("EntrySetKey"), input.entries.clone())
            {
                Ok(set) => set,
                Err(error) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                }
            };
            run(reader, &set)
        } else {
            WorthQueryDecisionPlan::immediate(HandlerResult::Completed(()))
        };
        let decided = self.decide(input, reader);
        plan.and_then(move |()| decided)
    }

    fn build_candidate(
        &self,
        input: &RegionTotalsDemand,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, RegionTotalsDemandBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, PositionY::at("PositionY"), input.replacement_y)
        });
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

```

---

## 4. Install the application graph

Installation derives graph-obligation, query and authorization identities with an
internal allowance of 16 times the admitted declaration's canonical entries and
bytes. Package validation admits declaration meaning under the shared 64 MiB
ceiling and byte-derived entry-storage bound; a size refusal occurs there with
its typed kind and requested/maximum counts. There is no smaller fixed identity
ceiling for an application resource profile to tune. An operation's installed
aftermath identity is bounded by that same ceiling, so a recorded inverse may
demand every field and relation its operation reads. Distinct aspect contracts
count once, references count per occurrence, and a complete explicit field mask
counts once per projection. Partial masks, touches and bindings retain exact
meaning. See the factor proof in the
[installation contract](../workspaces/worth-query/crates/worth-query-installation/README.md#derived-identity-bound).
Canonical derivation refusals retain their typed cause and requested/maximum
counts through query, operation, resource and schema-binding installation
denials' `canonical_denial()` accessors.

### Application build identity

Each complete build supplies a fresh UUID through `WORTH_IMPLEMENTATION_BUILD_ID`.
The application embeds it with `env!`, parses it through
`host::application_installation::ApplicationBuildIdentity::parse`, and passes
it once after contribution configuration to every fresh/reopen installation
entry point. Missing identity fails compilation; malformed or zero identity
fails parsing before installation.

Build from the repository root with:

```sh
python scripts/application_build.py cargo build --locked
```

The wrapper supplies a fresh UUID for the invocation and forwards the Cargo
arguments and other environment inputs. Use it for each complete build command.
Applications outside this repository should issue and embed a fresh UUID in
their own build entry point.

The fresh environment value must be supplied for **every complete build
invocation**, including a dirty checkout, dependency/configuration rebuild or
an otherwise Cargo-fresh invocation. `env!` embeds that value and makes it a
compiler input. Keep the embedded identity when launching or reopening with the
same artifact. Runnable application examples use this same environment input;
test worlds use explicit fixture identities.

Query seals the same identity into every installed producer edition, including
producers with `INPUT_REUSE=None`. A changed build makes old derived output
stale even when its value is unchanged. Reopen preserves authored source and
references, then regenerates derived output as required by the installed program.


Installation turns a validated program, a schema declaration, the
contributions' configuration, and resource limits into a running runtime. The
`initial_state` closure seeds the primary graph before the first publication.

### One process execution service

Construct the physical execution authority at the process composition root,
after validating immutable startup inputs and before installing applications.
Keep it in an `Arc` above application, document and session lifetimes. Every
fresh installation and checkpoint reopen receives a clone of that same `Arc`
through `limits.with_execution_authority(Arc::clone(&service))`. Retain the
process handle while applications close and reopen.

The runnable [shared execution example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/examples/shared_execution.rs)
shows this complete sequence: two installed applications share one service,
one closes and reopens from its original checkpoint, the reopened application
edits its own source, and the other application retains its independent model
state. Construct the authority once per process: later construction returns
`ConstructionDenial::AlreadyConstructed`, including after the final handle drops.
For an application installation retry, reuse the retained process service.

| Composition decision | API and responsibility |
|---|---|
| Physical worker service | `ExecutionAuthorityConfig::for_workers(workers)` and `ExecutionAuthority::try_construct(config)`; retain the resulting `Arc` at process scope. `ExecutionAuthorityConfig::available_worker_count()` supplies the machine's available width when that is the intended policy. |
| Ordinary request policy | `ExecutionRequestPolicy::for_workers(ExecutionPosture::Serial, DeterminismContract::CanonicalBitwise, workers)` selects placement, determinism and worker width. |
| Application installation | Pass that policy through the ordinary World resources and supply the same service with `with_execution_authority`. The application still supplies its own schema, configuration, build identity and model state. |
| Reopen and overlapping applications | Retain the process service while applications close or install; pass another `Arc::clone` to each installation alongside its checkpoint or initial state. |
| Operation execution | Use the application's authorized request. Handlers and retained computations borrow that request's execution authority. |

Start with **Serial** placement. To use admitted workers for independent compute
partitions, select **Automatic** and measure the application's workload at the
intended worker width. Both placements preserve cancellation, deadlines,
canonical settlement and typed allocation failures. Retained computations check
the fresh request's controls when reusing qualified unchanged inputs.

Platform owners that need a dependent native stage use
`ExecutionRequest::run_native_with_checkpoint` and `KernelContext::poll`;
`run_native` lends the same request's optional lease to a native pattern.
Both return an `ExecutionPhysicalReport`. Native map reports and retained runs
expose optional cost through `accounted_cost()`: `None` means cost was not
measured; `Some((work, span))` describes an accounted run. Physical worker
observations are available independently.

For a caller that needs work or memory accounting, use
`ExecutionRequestPolicy::new(..., ExecutionBudget::new(...))` and the explicit
accounted scope APIs. Inside that contract, an absent work or memory ceiling
still leaves accounting enabled. Supplied ancestor and process limits apply to
descendants. See [memory policy](#optional-request-memory-policy) for reservation
lifetimes, failure types and measured-result handling.

### 4.1 One program

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-testkit-certification/src/certification/program_example_authority/product_workflow_support/application.rs
use worth_query_host::facade::{application_installation, primary_graph, runtime};

let declaration = TemporalHostSchema::declaration().expect("the example schema is valid");
let runtime = application_installation::in_memory_program(
    program::validated_program(),                  // ValidatedApplicationProgram<S, P>
    declaration,                                   // ApplicationSchemaDeclaration<S>
    (TemporalContributionConfiguration { clock_source },), // one entry per contribution
    application_installation::ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))
        .expect("the build pipeline embeds one nonzero UUID"),
    example_limits(),                              // WorthQueryInMemoryApplicationLimits
    |_phase, graph, installed| {                           // seed the primary graph
        let principal_binding = installed
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the temporal principal binding must install");
        seed_graph(graph, &principal_binding, gate);
        Ok(())
    },
)
.expect("the validated temporal program must install");
```

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Same file: limits.
application_installation::WorthQueryInMemoryApplicationLimits::new(
    product_world_resources(),
    runtime::WorthQueryApplicationQueryResourceProfile::new(
        std::num::NonZeroUsize::new(128).expect("concurrent graph work is nonzero"),
    ),
    primary_graph::SignalConditionalEvaluationBudget::development(),
)
```

The result is a `WorthQueryProgramApplicationRuntime<Schema, Program>`.

### 4.2 Several program revisions: the roster

A host that must be able to adopt a successor program rosters it at install.
Bank installs `BankApplication` and rosters `BankApplicationP1` beside it:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/identity_runtime/installation.rs (abridged)
let program = validated_bank_application()?;
let successor = validated_bank_application_p1()?;
let declaration = BankSchema::declaration()?;
let runtime = in_memory_rostered_program(
    program,
    WorthQueryApplicationProgramRoster::new().support(successor),
    declaration,
    ((), (), ()),          // three contributions, none configured
    application_installation::ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))
        .expect("the build pipeline embeds one nonzero UUID"),
    limits,
    initialize,            // |_phase, graph, installed| { ... Ok(()) }
)?;
```

Other constructors on `application_installation`:
`in_memory_program_with_authorization_time_source`,
`in_memory_rostered_program_with_authorization_time_source`, and the
`*_from_checkpoint` variants.

A reopened downstream demand may consume a checkpoint root before any separate
root demand. Query first verifies the complete retained producer facts and native
output witness, then retains bounded custody of that exact settlement through
downstream publication. This static custody contains no executable source query:
changing the root invalidates the old result and requires genuine typed source
admission before it can refresh. Ordinary missing or retired managed settlements
continue to refuse publication.

### 4.3 What the runtime gives you

`WorthQueryProgramApplicationRuntime` dereferences to the primary-graph
application runtime, so `request(..)`, `discovery()`, `current_world()`,
`installed_schema()`, `branches()`, and `conditional::<T>()` are all available
on it.

For an isolated branch from a selection already admitted in a host advancement,
consume that selection with `runtime.branches().fork_expected(selected)` and
create it with the same runtime's phase. The returned candidate retains its
source; use `candidate.branch()` for ordinary requests and `candidate.close(&phase)`
to discard it. After the requests succeed, `candidate.adopt(&mut model.head, &phase)`
replaces the actual model-owned branch slot only while the original source is
still current. It returns the previous branch for ordinary close. A refused
adoption leaves the slot unchanged and returns the candidate with its typed
denial; a later refused close of the previous branch does not undo adoption.
Source movement also refuses creation, and a refused candidate close retains
the candidate with its original denial. See
[fork from an admitted selection](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#fork-from-an-admitted-selection).

For a move-only application head, consume the initial candidate with
`into_owned_head()`. Store the returned owner, target requests with
`head.branch()`, and adopt subsequent candidates with `head.adopt(candidate,
&phase)`. Success returns an owned previous head for explicit close. A close
refusal returns either the live head (`Retained`) or its actual retired cleanup
obligation (`Retired`). See
[own an application branch head](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#own-an-application-branch-head).

To bind that head to one model, declare `ApplicationHistoryBinding` with the
model's ordinary READ intent and register an `ApplicationHistoryDefinition` in
its contribution. Pass an explicit metadata disclosure contract to
`ApplicationHistoryDefinition::for_binding(..)`: use
`ApplicationQueryDisclosureContract::public()` for public step metadata, or a
governed contract covering the history's occurrence, ancestry and continuation
facets. This policy is separate from payload READ and edit permission.
On the head's branch call
`request.attach_history::<Binding>(&phase, head, model_read)`. The returned
document owns the slot; `document.read(&request)` freshly admits the original
model READ. For an original retained no-output action,
`document.assess_edit(&request, &operation)` observes current READ/action
authorization, declared operation membership and exact model/head correspondence.
It borrows both owners and grants no execution permission. An externally changed
head or recreated model refuses. Close with
`document.close(&phase)` and retain any live-document or retired-cleanup failure.
See [attach a model and read its owned head](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#attach-a-model-and-read-its-owned-head).

To include operations scoped to owned entities, declare `.owns(relation)` on the
history and enroll `.owned_operation::<Binding>(label)`. Native proves instance
membership from the stored full-generation ownership edges at the current head;
registered source-scope-owned publication members participate through the same
current-custody owner. Installation enforces one document owner, including
custody first installed in the prospective publication. See the
[generic owned-model example](history-owned-models.md). The explicit
`OwnedModelGraph` navigation path remains under qualification. Its transfer
policy restores the selected owner of a live transferred member at its original
identity, while preserving the outside owner's other state. Missing owner
sources, second ownership and current invariant violations refuse typed; undo
and redo never silently retire a selected live member.

A configured producer source may exist only in the private restored candidate.
In that case navigation uses its retained original description to check current
principal, permission, capability and request facts at the current head first.
It then validates the source's full generation, liveness and owned-domain
membership at that exact candidate. Restored grants never authorize the call;
a refusal leaves selection unchanged. When a new discovery or child-source
intent resolves only at the candidate, a denied current audience receives the
ordinary current scope refusal, without additional historical identity details.
The same admission applies below a verified parent and a genuinely regenerated
historical parent. The installed discovery query computes the child set,
including empty sets; a restored stale child can regenerate while its healthy
model root stays unopened. Give a changed discovery implementation a new build
identity: the selected parent then takes its actual fallback and the installed
query computes the new child set. Corrupt parent/publication lineage refuses
before producer work. A healthy discovered child also verifies without projecting its producer
source, whether its parent was verified or genuinely regenerated. It keeps
read-only parent custody for further discovery, with no synthetic settlement;
closed traversal reauthorizes each source at C. The final currentness pass
checks all restored routes at the one endpoint. A fallback can create a child
source at a later private candidate. Its identity and liveness are checked at
that exact candidate, and declared ownership or current publication custody must
connect it to the selected domain. A matching business key at C can name a
different generation; that older entity cannot supply its permission. Every
grant still comes from C. A transferred member authorizes its selected subset,
including new publication children, without authorizing the outside owner's
unrelated members. All-tree edit recording remains under qualification.

To bound selectable history across all futures, declare separate
`ApplicationHistoryWindowBinding` and `ApplicationHistoryBookmarkBinding`
recipes and install them with `.window_policy::<Window>()` and
`.bookmark_policy::<Bookmark>()`. The default unpinned window is 200 steps;
current and bookmarked steps are protected separately. Set a positive
`ApplicationHistoryRecentSteps` with `document.set_history_window(target)`,
then call the returned borrow's `advance::<Window>(&request)` until
`pending_window` is empty. Each call has fresh READ, metadata and window
admission and excludes at most one entry. Resume a dropped borrow with the same
target. Use `set_history_bookmark::<Bookmark>(&request, node, true/false)`
for independently admitted pin/unpin, `history_retention_status` for counts
and pending ownership, and `maintain_history` for one release attempt.
For governed metadata, use `advance_governed`,
`set_history_bookmark_governed` and `history_retention_status_governed` with
the fresh metadata capability, operation and input. The policy roles remain
separately admitted. Cancellation after a batch can leave a committed prefix;
inspect status and resume the same document and target with a fresh request.
Pages distinguish `Root`, `Retained(parent)` and `Truncated`; do not invent
an edge through an excluded node. Existing previews and registered checkout
retain their original observations, while new requests refuse excluded IDs.
Release holders independently and explicitly close the session, retaining
refused cleanup. See the
[window, bookmark and disposal example](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#configure-a-navigation-window-and-bookmarks)
and its linked compiled public journeys. This bounds availability, without
promising storage reclamation or a byte limit.

This compiled example uses the retention application's `Document`, `Request`
and installed `Window`/`Bookmark` recipes. `Failure` is
`WorthQueryApplicationHistoryDenial`. `protect_and_shrink` performs one bounded
batch. If `pending_window` remains, resume `set_history_window(target)` with
fresh requests until it clears; pinning succeeded even if the later window
change refuses. Release independent previews, unpin unwanted bookmarks, and
advance `maintain_history` before consuming the document for another edit.
`dispose_document` returns the actual close failure owner: retain it and retry
its cleanup after outstanding holders release.

<!-- compiled: document_history_retention.rs -->
```rust
fn protect_and_shrink(
    document: &mut Document<'_>,
    request: &Request<'_, '_, '_>,
    bookmark: WorthQueryHistoryNodeId,
    target: ApplicationHistoryRecentSteps,
) -> Result<WorthQueryHistoryRetentionStatus, Failure> {
    document.maintain_history(request)?;
    document.set_history_bookmark::<Bookmark>(request, bookmark, true)?;
    document
        .set_history_window(target)
        .advance::<Window>(request)
}

fn dispose_document<'runtime>(
    document: Document<'runtime>,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<
    WorthQueryDocumentHistoryCloseReceipt,
    WorthQueryApplicationHistoryCloseFailure<'runtime, DocumentRetentionSchema, RetentionHistory>,
> {
    document.close(phase)
}
```

The executable [document consumer](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history.rs)
uses [the ordinary installation and document setup](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/setup.rs)
and bounded pages as a disposable view. Each page and redo child page carries
`context()`: a format version, the actual installed schema binding and an opaque
process-local document identity. `document.commands(&context)` rejects unsupported
formats, foreign bindings and foreign documents before dispatch. Its borrowed
adapter grants no permission; preview, navigation and bookmark/window commands
still freshly enter their existing READ, metadata and role admission.

Keep a queued navigation's original intent, expected selection and key together.
`prepare` returns the genuine navigation owner with its original document borrow;
retain it when directly advancing or discarding an attempt. The borrowed command
adapter transfers that borrow into the navigation builder. `advance` uses the
same preparation path for one transient request.
This consumer's `Command` does that. On advancement refusal, `advance` returns
the original typed denial and posture. Between requests it drops only the frontend handle:
the same document retains the real Native attempt and SDK output custody.
Reusing the command never refreshes its expectation to a later selection.
Inspect the cause as well as the posture before retrying. A retryable refusal
before adoption can continue the same command with fresh admission. To abandon
an unselected attempt, retain the owner returned by `prepare` and call its
`discard`, preserving any refused cleanup owner. If only the command was kept,
fresh admitted preparation recovers that same registered attempt; consuming
the owning document for close also preserves its actual pending cleanup.

A refusal with `posture: WorthQueryHistoryNavigationPosture::Selected` means
adoption already occurred. A freshly admitted retry of the same command returns
the original completion, including when another command has since selected a
different node. It does not reset that later selection. Read
`document.selection(&fresh_request)` to display the current head.

Executable handling examples show [restoring current authority](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/failures/authority.rs),
[competing gestures and original disposal](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/failures/selection.rs),
[recovering lost delivery](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/failures/delivery.rs)
and [aborting a partially completed group](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/failures/group.rs).

The `Failure` and `Progress` variants below are defined beside the
[compiled adapter](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history/navigation.rs).

<!-- compiled: document_history_commands.rs -->
```rust
pub(super) fn prepare<'d, 'r>(
    document: &'d mut Configured<'r>,
    request: &Request<'_, '_, '_>,
    command: &Command,
) -> Result<Navigation<'d, 'r>, Failure> {
    let navigation = document
        .commands(&command.context)
        .map_err(Failure::Context)?
        .navigate::<PlanarHistoryNavigation<ConsumerSchema>>();
    let target = match command.intent {
        WorthQueryHistoryNavigationIntent::Checkout(node) => navigation.checkout(node),
        WorthQueryHistoryNavigationIntent::UndoStep => navigation.undo_step(),
        WorthQueryHistoryNavigationIntent::RedoTo(child) => navigation.redo_to(child),
    };
    target
        .expected_selection(command.expected.clone())
        .idempotency(command.key)
        .prepare(request)
        .map_err(Failure::Preparation)
}

pub(super) fn advance(
    document: &mut Configured<'_>,
    request: &Request<'_, '_, '_>,
    command: &Command,
) -> Result<Progress, Failure> {
    match prepare(document, request, command)?.advance(request) {
        WorthQueryHistoryNavigationAdvance::Selected(done) => Ok(Progress::Selected(done)),
        WorthQueryHistoryNavigationAdvance::Pending(owner) => {
            Ok(Progress::Pending(owner.posture()))
        }
        WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => Err(Failure::Advancement {
            cause,
            posture: owner.posture(),
        }),
    }
}
```

For consuming edits and groups, check the same context with
`check_history_command_context`, then transfer the original document into the
ordinary typed builder or group. Preserve the returned owner on refusal. Run
the complete example with
`cargo run --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-consumer-root -- --document-history`.
Its renderer descriptions carry no historical payload lease and cannot reopen a
document after process loss.

Owned graph navigation can retain a runtime-length set of original typed
output trees with `with_output_trees::<Program, FirstRoot>(..)` followed by
`with_output_tree::<Program, OtherRoot>(..)`. Each entry retains its actual
program continuation and disposal custody. One navigation qualifies the complete
collection at the endpoint derived from all original publication movements; a
later fallback cannot leave an earlier family stale. Native supplies the selected
publication routes independently of the configured collection. Healthy restored snapshots verify every Native route through the existing
currentness owner before opening any original root or disclosing a producer
source. They need no fallback driver. An actual fallback still freshly admits
its source at C. A healthy leaf family remains unopened while a stale sibling falls back;
the existing currentness owner checks skipped routes again at the one derived
endpoint. The same-build cold-checkpoint path reconstructs original expected
output versions from recorded facts; verification still checks those versions
against the selected sources and current Native custody. A changed build falls
back only for the affected family. Query role metadata that disagrees with the
selected Native publication refuses as `OutputPublicationCorrespondence`; its
`WorthQueryHistoryPublicationCorrespondenceMismatch` names producer, family and
source, plus the original publication coordinate. The refusal holds diagnostic
data without retaining Native publication or historical-basis custody. For non-leaf trees, a healthy parent can discover children from its restored
output without running its root producer. Discovery runs the installed query
with current C permission, including for empty results; the publication roster
never supplies the child list. Each recorded step retains the canonical discovery
request recipe with its source intent. Navigation runs discovery with that frozen
input, even if the caller's retained demand has since changed. A new live edit
records its new input. Principals, permissions and request scope stay current;
installed discovery meaning is never frozen. Missing, corrupt or foreign-document
recipes refuse typed with the selected step, without adopting a partial head.
Pre-recipe steps are never backfilled from a live demand. A parent matches its family, full-generation scope
and admitted parameter partition. Discovery can read an output scope restored
only at the exact private candidate D: current C still supplies every principal,
permission and request fact. A denied audience receives no additional historical
existence information. The same rule covers sources created by a real fallback
at a progressed private candidate, including a new generation of C's business
key. Corrupt lineage refuses typed; changed discovery meaning takes the real
parent fallback. For isolated source or fixed edits, the configured trees also complete on the private candidate before adoption of the single recorded step. Healthy leaves verify before opening their original roots. Recording after checkout can retain a healthy non-leaf parent without inventing a settlement or reopening its source: installed discovery and dependent reads use fresh current document control. A completed ordinary fixed action keeps its genuine original execution and discovery posture. A configured source that the edit retired or moved outside the document has a distinct Native-backed completion. Final qualification checks every whole tree, rechecks these completions and verifies Native’s independent current obligations. An omitted stale family or a later tree invalidating an earlier result refuses before adoption. Configure the exact declared fixed root when retaining a fixed action, alongside its other families. Group recording and codec-retired selection remain under qualification. See the
[owned-model plan](../plans/WORTH-query/history-owned-models.md).

To record an isolated edit, consume the attached document with
`document.edit(intent).idempotency_owned(&request, key)?`, then
`retain_in_program::<Program, Action>(host)`. Repeatedly call `advance(&request)`
on the returned owner; retain that owner on every progress or refusal. The
document's `read` continues to use its selected head until candidate adoption.
When ready, `finish(&request)` returns the document and either its sealed step
with the original action completion, the original step for an equivalent retry,
or the original terminal action outcome. A refusal after adoption can still
report `Selected`; preserve its completion and finish it with a fresh request.
Use `discard(&request)` to explicitly abandon unselected candidate work, keeping
the returned owner when cleanup refuses. This entry accepts declared
source-completion actions and fixed required-output graphs with no mutation
source, workflow control or external effects. A fixed graph returns
`WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery }`; `delivery` is
the original `WorthQueryClosedFixedDelivery`, retaining its actual settlement
and close evidence. A terminal fixed edit returns `NotRecorded { terminal }`;
inspect `advance_denial()`, `source_outcome()`, `source_result()` and `close_evidence()` separately
because a source can perform before its required output becomes unavailable.
Unavailable completion keeps descriptive source-receipt evidence without
retaining the discarded candidate's physical history.
See [record an edit and retain its original outcome](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#record-an-edit-and-retain-its-original-outcome).

This example uses the retention application's [installed history declaration](../workspaces/worth-query/crates/worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/document_history.rs),
[ordinary program installer](../workspaces/worth-query/crates/worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/host.rs)
and [head construction](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/mod.rs).
`Action` names its declared `SetRetentionBinding` action with source-completion
delivery. The original head and second document are retained here so the
[complete executable journey](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/journeys.rs)
can independently check isolation, advance the edit, inspect its exact result
and dispose both documents.

<!-- compiled: document_history_edit.rs -->
```rust
    let host = publish_on_first_program();
    let owned = head(&host);
    let original = owned.branch();
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let document = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(owned.branch())
                .attach_history::<RetentionHistory>(
                    &phase,
                    owned,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut edit = document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 3,
            },
        })
        .idempotency_owned(&request, 0x9182_0400)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("declared action constructs: {cause:?}"));
```

Closing a document releases its internal history pins and disposes all its
owned heads. `Closing(owner)` retains unfinished disposal across multiple heads;
retry `owner.retry(&phase)` after releasing blocking readers. A closing owner
cannot read or edit the document. A successful close does not release history
held independently by other consumers.

Use `document.current_step(&request)` for public step metadata. For governed
metadata, call `current_step_governed(&request, capability, operation, input)`
with the declared metadata purpose. The ordinary model READ still executes its
body and filters; payload permission alone cannot disclose a step. Each call
checks the current metadata grant before looking up the journal.
Fresh governed edit enrollment and attempt lookup use
`edit.advance_with_metadata(&request, capability, operation, input)`; retain the
returned edit on refusal. An authorized retry returns its original step without
repeating the edit. See the [compiled metadata and retry journey](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/metadata.rs)
and [metadata declarations](../workspaces/worth-query/crates/worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/document_history.rs).

List completed steps with `document.history_page(&request, limits, after.as_ref())`.
`WorthQueryHistoryPageLimits` requires positive `maximum_entries`,
`maximum_examined_entries` and `maximum_owned_bytes`. Rows carry a sealed step
with its parent, label and exact source description. `retained_source_at_read()`
means the document held that source when this admitted page was assembled; the
row does not hold a payload lease or promise later preview availability.

Keep `page.continuation()` for the next fresh read. A cursor belongs to one
document and completed view; a completed edit invalidates it with a typed
`StaleCursor`, while a foreign cursor returns `ForeignCursor`. A refusal leaves
the input cursor unchanged. Pending or discarded edits do not create page rows
or change continuation hints. `edit.history_page(..)` lists the still-selected
completed view during such work. Drop pages when finished to release their
metadata backing; derived layout data can be rebuilt from fresh pages.
Governed metadata uses `history_page_governed(&request, capability, operation,
input, limits, after.as_ref())`. Both the original READ and current metadata
permission run before cursor or storage diagnostics. See the
[compiled bounded-page journeys](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/mod.rs).

Open a historical holder with `document.preview(&request, node_id)?`. Read it
with `preview.read(&document, &request)?`, supplying the actual current document
on every call. Later edits can move that document while the holder keeps its
original composite source. The result exposes admitted `rows()`, the exact
descriptive `source()` and the actual Native `basis()` used by the query.
For a recorded fixed graph, `required_root_posture()` carries the original
qualified root settlement. The step's endpoint is separately qualified against
every completed root and dependent output and the original discovery evidence.
The source action, root output and common endpoint can have different commits.
`None` means the step carries no required-root settlement; it does not
mean an output is unavailable. Preview reads execute no source command or producer.
The holder and result expose no retained-selection or mutation conversion.
Call `result.close()` and `preview.close()` when finished; another independently
opened holder remains usable. Governed metadata uses `preview_governed(..)` and
`read_governed(..)` with its current capability purpose on each call. See the
[compiled historical-read journey](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/preview/mod.rs).

To edit from an old node, explicitly declare an entity-record history domain and
an `ApplicationHistoryNavigationBinding` with its separate NAV capability.
Start a new click with an actual current selection. This compiled callback uses
the planar example's `ConsumerSchema` and `PlanarHistoryNavigation` declaration;
use your own schema and navigation binding in an application. It returns the
original expected selection and first advancement result:

<!-- compiled: document_history_navigation.rs -->
```rust
    let expected = document.selection(request)?;
    let navigation = document
        .navigate::<PlanarHistoryNavigation<ConsumerSchema>>()
        .checkout(node)
        .expected_selection(expected.clone())
        .idempotency(key)
        .prepare(request)?;
    Ok((expected, navigation.advance(request)))
```

Keep the original expected selection, node and key with the click. Advance a
`Pending(owner)` or `Refused { cause, owner }` using that retained owner and a
fresh request. After losing a handle, prepare the same original description;
capturing a later selection under the old key changes the intent and refuses.
Checkout creates a
writable context and selects the existing node without adding an authored edit;
the next edit becomes its child, preserving the original future. Current grants
and other records outside the model remain current. Read the
[checkout contract and supported component meaning](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#select-a-historical-step-for-editing)
before enabling navigation. A retained preview or page supplies no NAV authority.

Set `ApplicationHistoryBinding::NAVIGATION_RESOURCES.maximum_unfinished_attempts`
to bound unfinished navigation attempts per attached document (default 8).
At capacity a new key receives `UnfinishedAttemptLimitExceeded` before target
lookup or candidate creation. Retrying an existing key shares its original slot.
Selection or completed `discard(&request)` releases that slot; a dropped handle
or refused disposal keeps the original attempt registered. Use a fresh request
to finish or discard it. This limit does not bound completed history or its
separately owned cleanup.

For undo and redo buttons, capture the expected selection for each new click.
Represent its command as `Button::Undo` or `Button::RedoTo(child)` and lower it
through the same prepared navigation owner:

<!-- compiled: document_history_buttons.rs -->
```rust
    let builder = document.navigate::<PlanarHistoryNavigation<ConsumerSchema>>();
    let target = match button {
        Button::Undo => builder.undo_step(),
        Button::RedoTo(child) => builder.redo_to(child),
    };
    let navigation = target
        .expected_selection(expected.clone())
        .idempotency(key)
        .prepare(request)?;
    Ok((button, expected, navigation.advance(request)))
```

Keep the returned command, expected selection and key for retries. Use
`document.redo_choices(&request, limits, after)` to read a bounded child page and
its `NoChild`, `Single(child)` or `ChoiceRequired` hint. Offer an explicit child
choice when several futures exist; preparation revalidates even a remembered
single-child preference. The application keeps the actual pending/refused owner
and preserves the document's output mode. Navigation selects an existing state;
[semantic correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md)
publishes a new inverse, compensation or reapplication under its own contract.

For a gesture containing several ordinary actions, use an
owning group. It keeps the original model selected until all members complete,
then records one history edge:

For a runtime-length mix of action types, open
`document.gesture(host, request, key)` and add each entry with
`push::<Intent, Action>(request, intent, member_key)`. The first advancement
freezes the member list. The same group owner advances and completes it at one
endpoint, and `finish` returns each original completion for typed recovery with
`original::<Intent, Action>()`. Installed member and retained-result limits apply.
The [mixed Model/Member gesture](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_model/gesture.rs)
shows runtime membership, atomic adoption and refused partial work.

<!-- compiled: document_history_group_open.rs -->
```rust
    document.group_edits::<_, RetentionProgramP0, Action, 2>(
        host,
        request,
        key,
        [
            (
                SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 3,
                    },
                },
                0x9182_5001,
            ),
            (
                SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: second,
                    },
                },
                0x9182_5002,
            ),
        ],
    )
```

Advance the original group with fresh requests; this example uses `Posture` as
an alias for `WorthQueryHistoryGroupPosture`:

<!-- compiled: document_history_group_advance.rs -->
```rust
    let (document, execution) = group.parts();
    let posture = execution.advance(document, request)?;
    if posture == Posture::ReadyToComplete {
        execution.complete(document, request)?;
    }
    Ok(group.posture())
```

The caller keeps `group` after every refusal. On a terminal member, inspect its
original native outcome and abort the group; earlier candidate publications
remain real effects with their own cleanup. After selection, finish the group
to obtain its document, one history step and original member completions. Keep
any returned cleanup owner on refusal. A group accepts a homogeneous list of
source-completion or fixed required-output actions and preserves the document's
payload and output modes. Each fixed member completes its actual root and
children before the next member starts. Its completion is an original
`WorthQueryClosedFixedDelivery`; earlier deliveries keep their historical
settlements even when a later member makes current delivery stale. The final
step names the last complete tree's endpoint.

The group also checks every installed producer that its declared root and child
families can select. A workflow or external-effect contract in one of those
families refuses the group before candidate creation or source effects.
An unrelated producer family does not prevent grouping. Installed
`GROUP_RESOURCES` bounds members/results (64 each by default, one unfinished
group). See
[group completion, retries and disposal](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#group-ordinary-edits-into-one-step)
for cancellation, competing navigation and partial effects.

For a fixed member, supply a fresh request on `delivery.branch()` and validate
its original complete output set at the retained observation:

<!-- compiled: document_history_group_fixed_delivery.rs -->
```rust
    let original = request.at(delivery.settlement().observation());
    delivery.deliver_current_at(&original)
```

An ordinary retained query can then read that member's historical artifacts
under its own READ contract. A later group member does not replace this
delivery's original settlement.

For a fixed required-output tree, configure the document with
`document.with_outputs::<Program, Root>(&phase, &request, &application, demand, controls)`
inside an advancement of its owning host. A refusal returns the original
document with its cause. The successful document carries
`WorthQueryHistoryWithOutputs<Schema, Program, Root>` as its fifth type argument;
preserve that mode through application wrappers, edits and close results.
Navigation then owns the original root, discovered children and pending recovery
until all required outputs settle before adoption. See
[configure required outputs](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#configure-required-outputs)
for retry and explicit close.

To store the selected basis as application data, serialize
`selected.describe_basis()` before releasing the selection. After loading that
record through an ordinary READ request, deserialize it as
`WorthQueryExpectedProductBasisDescription` and call
`runtime.branches().select_expected(&description, &phase)`. This asks the original
installed World and Native owners to admit that exact basis; stale, retired,
foreign and unsupported descriptions refuse. The description retains no live
selection and supplies no READ or mutation permission. Pass the returned
selection to `fork_expected`, then use the same candidate/adoption lifecycle.
See [store an expected basis](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#store-an-expected-basis-and-select-it-again).

Its own methods:

| Method | Returns |
|---|---|
| `runtime()` | The underlying primary-graph application runtime |
| `installed_program()` | The installed program meaning: `identity`, `revision`, `features`, `actions`, `connections`, `rules` |
| `contains_action::<Binding>()`, `contains_output_root::<Root>()` | Whether the installed program owns an action or output root |
| `supported_program::<P>()` | `Option<WorthQuerySupportedProgramHandle>` for a rostered program. The handle implements the `WorthQueryProgramOwner` trait; import the trait to call `owned_revision()`, which is the adoption target |
| `retire_program_support(&revision)` | Retires support for a revision no branch still needs |
| `retain_workflow_spec(..)` | Binds a workflow vocabulary ([Â§6.2](#62-install-the-workflow-vocabulary-once)) |
| `close_conditional_runtime()` | Closes conditional resources at shutdown |

`discovery()` lists query requests, mutations, queries, and fields.
Features, connections, and rules come from `installed_program()`, not from
discovery.

Managed asynchronous resource requests are available through Decl/Host following
[Managed async resource contract](../plans/WORTH-query/milestone-9.18.1.md#phase-8-managed-async-resources-through-decl-and-host).
[Application resources](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-resources.md)
documents the concrete `request.resources()` start/advance/inspection/close path,
installed provider, finite ingress, exact selector reclaim and recovery handoff.
Resource declarations and installed typed-profile lookup separate Start/Inspect/Advance/Close authority
from availability, state, payload, domain failure and cleanup-detail disclosure.
Installed access purposes keep the corresponding action and facets together;
plain Close requires no Inspect or payload disclosure. Their ResourceRead plans
retain genuine provider capacity until terminal release, but grant no lifecycle
action or payload permission by themselves.
Availability is boolean existence; protected queue/provider counts belong to
cleanup detail and require Count as well as Explanation influence. A receipt is
process-local selection provenance; fresh original-branch action and requested
facet admission precede protected row/provider/clock access. Explicit runtime
close refuses allocated rows and preserves their owners. Rust Drop/process
termination does not attest to provider or Native cleanup.
The existing lower World/Bridge path already preserves the
exact committed source through request admission and revalidation; an empty
source returns `NoCommittedSource` before creating a Signal request. This source
binding does not supply an application provider or authorize external completion.
See [native source and cleanup custody](../crates/worth-runtime-world/README.md#owned-async-source-binding).

---

### 4.4 Checkpoint program transitions

Ordinary `in_memory_rostered_program_from_checkpoint` requires the recovered
activation to name a supported program. For an app-owned predecessor mapping,
`in_memory_rostered_program_from_checkpoint_with_transition` checks an exact
`WorthQueryCheckpointProgramPredecessor` rendering and admits the current target
through the ordinary program/roster installation path. The predecessor rendering
never becomes a program revision or a roster member.

For fresh installation with a separately funded setup request, borrow that original
request with `host::application_contribution::with_bootstrap_advancement_from_request`.
Call `host::application_installation::in_memory_rostered_program_in_advancement` with
the phase first, followed by the validated program, roster, declaration,
configuration, build identity, World limits and ordinary seed callback. It uses the
same construction and roster core as default installation. The phase's original
work, memory, cancellation and deadline govern all setup; the installed World's
request policy remains the one declared in its limits. A larger seed grant therefore
cannot make a zero-work runtime query succeed. Default `in_memory_program` and
`in_memory_rostered_program` still open their bootstrap from the World policy.
The public-entry funding, cancellation and memory regressions live in
`topology_entry/src/checkpoint_recovery/support/bootstrap_funding.rs`.

When installation belongs to an already admitted execution attempt, borrow its
`ExecutionRequest` through `host::application_contribution::with_bootstrap_advancement_from_request`
and pass the resulting phase to
`host::application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement`.
This variant takes the existing transition arguments with the phase first and a
required capture policy. It borrows the original cancellation/deadline and
remaining cumulative work through construction, roster admission, native transition,
final assembly and capture. Native nested scopes charge the original parent; they
do not renew its allowance. Capture is capped by the caller-named policy, which must
be `Execution(&lease)` for the funding lease itself or a lease descending from it,
for example a child with its own smaller memory limit; there is no default and no
silent inheritance. Entry checks that descent before any work or charge: a sibling,
a parent or another lineage's lease refuses with
`ExecutionDenied(ExecutionResource { NestedLeaseMisuse })`. A capture child that runs
out after the native ACK returns the acknowledged capsule with the child's own
refusal (`Policy { ancestor: 0 }`) for repair.
`SystemAllocation` and bare serial backing have no admitted capture allocation policy and refuse before
effects with `ExecutionDenied(ResourceEnvelopeMismatch)`, rather than falling back
to SystemAllocation. The ordinary convenience constructor retains its explicit
owned-request/policy contract. Its `Execution(&lease)` policy funds the whole
transition: the bootstrap runs on that lease, as if borrowed through
`with_bootstrap_advancement_from_request`, so the lease carries the transition's
work as well as the captured frame.

The bootstrap phase cannot escape its callback or stand in for an installed-runtime
phase. `OpeningCustody` still refuses a second Query opening from a host advancement
or managed computation. The opener preserves a callback result that ran: completion
owners check the original request before success or domain failure, and a generic
outer stop preserves a performed transition's recovery capsule.

For that capsule, call `repair_to_checkpoint_with_request(original_request)`.
It holds itself outside the opening callback until entry succeeds; refusal returns
the same capsule, including on entry cancellation or exhausted work. Its typed
`capture_denial()` may contain `Execution` with the original Query execution cause;
the original `finalization_denial()` remains distinct. Fresh-request retry repairs
the same successor without reauthoring. The existing phase/policy repair method
remains available for its explicit contract; its `Execution(&lease)` must be the
caller lease that funds the call or one of its ancestors, and any other lease
refuses `UnrelatedNestedLease`. The runnable
[original request fixture](../workspaces/worth-query/crates/worth-query-testkit-host/src/host/temporal_conditional_operation/contribution_installation/checkpoint_transition/original_request.rs)
uses the real nonempty native transition; its
[capture repair companion](../workspaces/worth-query/crates/worth-query-testkit-host/src/host/temporal_conditional_operation/contribution_installation/checkpoint_transition/admission.rs)
retains the native capsule across stopped attempts.

Its bounded `WorthQueryCheckpointMigrationWriter` authors new typed entity seeds
and relations between entities created in that batch. It currently cannot read or
rewrite recovered records, or link a new record to an existing endpoint. Query
commits those effects, the target activation and complete supported entity and
relation revalidation in one native candidate before exposing a World. Surviving
relations are judged through native read demands that preserve their fields,
endpoints and record versions. Rules overlapping retired relations still refuse;
retained workflows remain unsupported. Native candidate/publication limits remain
in force alongside explicit selection and authoring bounds.

For a derived-only codec change, use that same entry point with the target schema
and program and an empty author callback (`|_, _| Ok(())`). Native recovery must
first establish each affected present field's actual producer publication and
complete payload coverage. The target must install the actual birth and successor
producers with their complete output-role meaning for the same native kinds;
a Preserve producer may itself create dynamic members. Before any disposition, each accepted output must match the complete original
native publication; an optional role cannot be omitted when its native output
is present. Each accepted output must then either retire completely or survive
completely.
A survivor must match live exact native identities, kinds and present payload,
its original performed producer association and the complete installed output
role meaning. A mixed or incomplete correspondence refuses. Surviving output
metadata retains historical locators with no producer facts; it cannot establish
target readiness. The installed producer must genuinely publish before that
output can become current.
An empty callback, a role declaration, a revision advance, or a copied producer
description grants no retirement permission.

The supported cutover retires whole output records and rebuilds native
entity-field equality indexes. Native producer-created internal relations may
retire in that same candidate when both endpoints are independently proven in
that same performed publication's complete output closure and the present
relation payload still matches its
native creation or authenticated producer succession. Retirement deletes the exact admitted relations before their
entities. Related ordering indexes are prepared before publication and require
unchanged ordering-key codecs. External or rewritten relations, ordinary or
subsequently overwritten payload, affected custom invariants, changed ordering
keys, join consequences and multiple current branch heads remain refused.
Codec cutover cannot append authored rows. Unaffected surviving relations are
rejudged by the target program's actual installed rules before publication.
Transition capture and the acknowledged recovery capsule
retain the same funded historical output roster. Capture refusal returns that
same acknowledged capsule for funded repair; repair does not reauthor or copy
predecessor currentness.

Historical roots and the original checkpoint keep their old meaning. Demand
output through the installed target producer after installation, then use
ordinary edits and checkpoint capture/reopen. Keep the original runtime
available for requests that resume its existing continuations. Each token owns
its historical selection and ordered-index generation; a fresh authorised
request on that original runtime can consume it after current values advance.
The target installation is a foreign owner. A token, observation or catalogue
comparison cannot grant target-current authority.

Start a page on the current branch with `.page(...)`; public paging does not
accept `.at(observation)`. Resume with the token itself and a fresh request:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// The application declares read_children and the query.
let first = original_request.query(read_children.clone()).page(width)?;
let (owned_rows, continuation, _) = first.into_parts();
let token = continuation.expect("another child page");
// Keep the original owner available while installing/regenerating the target.
let second = fresh_original_request.query(read_children).resume(token, width)?;
```

### Protected continuation pages

An ordinary continuation advances one ordered Many child collection below an
ExactlyOne root with pinned basis support. Open a governed first page with
`page_governed` and resume with `resume_governed`, supplying the actual capability,
operation and purpose on every call. The `_approved` forms additionally require
current approved elevation. `readmit_resume_governed` and its `_approved` twin
lend the canonical admitted plan to an owning callback when needed.

This compiled maintained-Catalog court uses real protected child rows:

<!-- compiled: protected_continuation.rs -->
```rust
    let first = request
        .query(input())
        .page_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            width(),
        )
        .unwrap();
    let (rows, continuation, _) = first.into_parts();
    let second = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            continuation.expect("another child page"),
            width(),
        )
        .unwrap();
```

Each page admits current capability and independent READ before rows, count,
ordering, cursor or completion metadata. Hidden ordering fields need explicit
Ordering and Pagination influence; permitted hidden values remain omitted.
The cursor retains the exact old World, parameters and Native ordering generation.
Regrant can authorize an independently issued old cursor while current data has
changed; it does not replace the cursor's data with current values. A failed
resume consumes its move-only token. Retain separate genuinely issued cursors
when separate attempts are required. A requested page width cannot widen the
installed binding limit.

Managed-index root selection and related-collection continuation remain distinct
declared shapes. Combining them refuses during declaration; a managed candidate
hit supplies no child-page position or root-pagination authority.

The [related-output journey](../workspaces/worth-query/crates/worth-query-testkit-certification-canonical/src/certification/application_graph/canonical_transition/related/continuation.rs)
publishes three output roles and two internal relations. An original token still
reads the old child total `12` after Preserve advances all three original outputs
to `37`; the genuine target producer emits child total `1012`, then Preserve
advances it to `1037` before current-format reopen. Live delivery, paired
reconstruction and retained snapshot reads also retain their original request
and decode custody through completion. A swallowed seed refusal or late
cancellation denies the final page, image or successful delivery. Closing a
live consumer and closing its publisher are separate lifecycle operations.

`CheckpointTransitionCaptureStopped` retains the acknowledged phase when its
first checkpoint capture stops. Consuming `repair_to_checkpoint` returns a
target checkpoint, or the same capsule if capture stops again. A later installation failure returns
`CheckpointTransitionFinalizationStopped` if target finalization refused before
capture; its capsule retains the acknowledged successor and exposes the original
typed `finalization_denial()`. Capture retries preserve that original cause.
A later installation failure after checkpoint capture returns
`CheckpointTransitionAcknowledged`
with the already acknowledged target checkpoint and the original phase denial;
fix the installation configuration and ordinary-restore that checkpoint. An
unknown durable fate for the native transition is a typed
`RelationalDurabilityUnresolved` installation denial and issues no acknowledged
successor.

---

### 4.5 Declaration failures

Keep the original `decl::application_schema::ApplicationSchemaDeclarationDenial`
when reporting an installation refusal. Its three `MissingOperation...Dependency`
variants retain the owned operation name and an `ApplicationOperationDependencyLocus`.
`ProgramTarget`, `DecisionReadTarget` and `MutationPrecondition` retain the actual
rejected target, including its field path or relation endpoints. `Body` identifies
whole-body closure without claiming that one field was uniquely responsible;
effect and occurrence loci retain their declared identities. Displaying the native
denial includes this context without looking up a field from another registry.

The denial is `Clone`, not `Copy`. Borrow it when inspecting the cause, or move the
original through the application's typed installation/start refusal. These values
describe a rejection; they grant no mutation, currentness, retry or cleanup authority.

## 5. Run the program

Requests start from the runtime with `WorthQueryApplicationRequestExt`
(`worth_query_host::facade::application_entry`). Every request names two
things: **who** is asking (an authenticated principal) and **how long** it may
run (a request scope).

### 5.1 Who is asking: principal and request scope

A **principal** comes from an authentication adapter the host admits against
the installed schema. The adapter validates a credential and returns an
external identity. Query maps that identity to an application principal
through the schema's principal binding.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../examples/product_workflow_support/application.rs and read.rs (abridged)
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
};

let adapter = admit_authentication_adapter(
    application.runtime.installed_schema(),
    WorthQueryAuthenticationAdapterAdmission::new(
        WorthQueryAuthenticationAudience::new("host")?,
        WorthQueryAuthenticationMethod::new("example")?,
        admission_clock.clone(), // the observer installed with the runtime time source
    ),
    IdentityAdapter,          // your impl of WorthQueryAuthenticationAdapter
)?;
let principal = adapter.authenticate(credential, &scope).await?;
// -> WorthQueryAuthenticatedExternalPrincipal<Schema>
```

`IdentityAdapter` implements `WorthQueryAuthenticationAdapter`: it names its
`Credential` type and returns a `WorthQueryValidatedExternalPrincipal` from
`validate` (see `.../product_workflow_support/adapters.rs`). The external
identity must already be bound to a principal in the graph. The example does
that while seeding, in the `initial_state` closure from Â§4.1:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/application.rs
graph.bind_principal(
    principal_binding,        // installed.principal_binding(MyPrincipalBinding::reference())
    primary_graph::WorthQueryApplicationPrincipalKey::new("product-example")?,
    1_u64,                    // the application's own principal id
    declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
        "https://issuer.example",
        "product-example",
    )?,
    declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
)?;
```

Each executing host call opens one advancement under its installed execution
policy before its first read. Bootstrap installation lends that one request to
preparation, recovery, seeding, and publication. Its memory and aggregate work
limits apply in serial placement as well as leased placement. Internal phases
borrow custody; staged products carry facts and never retain the request.

Reentering an executing public door from an active host callback or its
descendant worker is refused with `ExecutionRequest(NestedOpening)`. An unrelated
execution lease remains the separate `Resource(NestedLeaseMisuse)` cause.
An unrelated thread's public call owns its own request. Borrow the callback's phase
for internal execution rather than opening another budget. A request
already canceled or past its deadline is refused at request admission as
`ExecutionRequest(Interrupted(Cancelled))` or
`ExecutionRequest(Interrupted(DeadlineExceeded))`, before principal resolution
or a reader. Mutation opening refusals use the same cause projection as
queries. An open live subscription is the exception: its `next` ends the
subscription with the `Cancelled` or `DeadlineExceeded` outcome, and every
later `next` reports `Closed`.
An opening refusal is `ExecutionRequest`, not a handler failure: no handler ran.
`ExecutionRequest` also carries resource and misuse refusals encountered while
an admitted request is executing.
An interruption observed after admission keeps the executing owner's cancellation
or timeout outcome.
A decision handler can poll `reader.checkpoint()` during synchronous work.
Preparation retains the first failed poll and refuses before candidate authoring,
even if the handler catches the error. Successful polls preserve eligibility for
completed-decision input reuse; the producer's canonical-input/context contract
and all source and publication checks still apply.
For an output demand refresh these are `Cancelled` and `TimedOut`; they are
separate from admission's `ExecutionRequest(Interrupted(..))` refusals.
Serial and leased placement both use checked leases. A reservation denied by
the policy reports `MemoryLimit { requested, admitted, level }` with the policy
level; the request's memory limit is not waived by serial placement.

A stopped read is not a missing record or a stale index. Entity resolution
separates `Stopped(cause)` from `Refused(kind)`; handle the typed stop instead
of retrying it as absence. A pinned read checks the live request when the pin
is acquired and holds the selected root and index payload for later reads.
Later reads check the original request and admit only their reached regions; a
cold-source failure remains a typed read refusal.

Zero memory reports a policy `MemoryLimit { requested, admitted: 0, level }`
at every placement, including its requested bytes.

A **request scope** carries an optional deadline and a cancellation token. With no deadline,
there is no time limit; work and memory remain policy-bounded, and the request
remains cancellable. It is
not an entity and grants nothing:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/adapters.rs
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

let cancellation = WorthQueryCancellationSource::new();
let scope = WorthQueryRequestScope::new(
    Some(admission_clock.now() + Duration::from_secs(60)),
    cancellation.token(),
    admission_clock.clone(),
);
```

#### Read governed data at one observation

Retain a read observation when the data and a later basis description must name
the same committed state. A governed query at that observation uses its original
data and freshly checks the current principal, capability and READ policy on the
requested branch. Revoking current READ therefore also refuses an older read.

This sequence uses the query and capability names from the compiled
[model reader](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/model.rs):

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let request = runtime.request(&principal, &scope).on_branch(model_branch);
let observation = request.retain_current_read()?;
let rows = request.at(&observation)
    .query(ReadModel { model: label.clone() })
    .execute_governed(
        ReadDomain::reference(), InspectDomain::reference(), ReadAccess { model: label },
    )?;
```

Keep the observation private until admission succeeds. To describe that same
basis, use `observation.select_on(&phase, runtime)?.describe_basis()` inside the
request's `with_application_advancement` call. The description grants no further
permission and may already be stale for `select_expected` if the head moved.
Drop the observation when its reads are finished. Managed-index open and rebuild
require a current request and refuse retained mode with `RequestMode`.

For storage certification, use the [retained-owner census](../crates/worth-relational/API_OVERVIEW.md#retained-owner-footprints)
outside publication timing. It unions shared allocations across current, held
and queued roots and reports catalog footprints separately. Keep requested-object
allocation peaks, checkpoint readmission and complete first-answer costs as
separate measurements.

The entity a mutation acts on is its intent's *scope binding*, shown next.

### 5.2 Mutations: intent, binding, handler

A mutation has three parts:

| Part | Trait | Written by | Role |
|---|---|---|---|
| Intent | `ApplicationMutationIntent<Schema>` | Your request code | The value you pass to `mutate(..)`: its input and the entity it targets |
| Binding | `ApplicationMutationBinding<Schema>` | Declared once | Ties the operation to its input, result, denial, decision type, idempotency key, principal binding, scope field |
| Handler | `OperationHandler<Schema, Binding>` | Registered in the contribution's `configure` | Decides, then builds the candidate |

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../examples/product_workflow_support/application_entry.rs (abridged)
impl ApplicationMutationBinding<TemporalHostSchema> for AmendTemporalBinding {
    type Input = AmendTemporalInput;
    type Result = AmendTemporalResult;
    type Denial = AmendTemporalDenial;
    type IdempotencyKey = u64;
    type Operation = AmendTemporal;              // the operation declared in the schema
    type Decision = WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>;
                                                 // what decide() hands to build_candidate
    type ScopeBinding = AmendTemporalScope;      // which entity field selects the target
    type SourceExpectation = NoApplicationMutationSource;
    // ... value bindings for input, result, and denial; principal binding types ...
    const IDENTITY: &'static str = "worth.query.example.temporal-amend.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.example.temporal-amend-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.example.temporal-amend-command.v1";
    // fn scope_field, principal_binding
}

impl ApplicationMutationIntent<TemporalHostSchema> for AmendTemporalIntent {
    type Binding = AmendTemporalBinding;
    fn input(&self) -> &AmendTemporalInput { &self.amendment }
    fn scope_binding(&self) -> AmendTemporalScope {
        AmendTemporalScope::new(IntentIdentityField::reference(), self.identity.clone())
    }
}

impl OperationHandler<TemporalHostSchema, AmendTemporalBinding> for AmendTemporalHandler {
    fn decide(&self, input: &AmendTemporalInput, reader: &mut DecisionReader<'_, '_, '_, _, _>)
        -> HandlerResult<Decision, AmendTemporalDenial> { /* read, then decide */ }
    // `Decision` here is the binding's Decision type (see above).
    fn build_candidate(&self, input: &AmendTemporalInput, target: Decision, writer: &mut CandidateWriter<'_, _, _>)
        -> HandlerResult<AmendTemporalResult, AmendTemporalDenial> { /* write fields */ }
}
```

#### Prepared decisions and typed effects

Use `DecisionReader` for the actual declared reads during `decide` or
`prepare_decision`. Carry the resulting typed targets, owned keys and prepared
values in the binding's `Decision`, then consume them in `build_candidate` through
`CandidateWriter`. The framework opens the effect program directly after the
admitted decision. A handler supplies domain intent and valid effects, without
predicting field, link, output or emission storage.

The writer enforces the operation's declared create, write, relation and emission
authority. Existing-entity targets must come from tracked decision reads. Bind the
declared output roles or family members to the actual created or preserved
entities; required roles and cardinalities remain checked before publication. Typed payload
validation, checked sizes, request cancellation and fallible effect-buffer growth
remain with their original owners. Invariant validation and atomic commit still
precede successful publication.

When a stored reference includes the process-local Native runtime identity, compare
it with `reader.reader().runtime_instance_id()` before using its entity coordinates.
This readonly value describes the projection's owning runtime; it grants no access.
The ordinary opaque `reader()` access keeps the decision's existing restriction on
completed-decision reuse. Compare the admitted scope's full entity identity and
observe its declared relation to the target before requesting a mutation target.
The [compiled retention handler](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/retention/handler.rs)
shows these checks against a stored typed edit.

For an authored deletion, declare the entity read with
`operation_read_entity(Operation::reference(), Entity::reference())` alongside
the delete grant. In `decide`, resolve the target, call
`reader.entity(&identity, Entity::reference())`, then carry
`reader.mutation_target(&identity)` into the candidate. The entity observation
supplies the native dependency used by deletion lowering. In `build_candidate`,
project that target, call `writer.delete_entity(...)`, and bind its declared
Retire role with `writer.retire_output::<Role>(&target)`. The
[compiled retirement handler](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/source_output_handlers.rs)
shows this sequence with ordinary installed read and delete authority.

The full, compiling version of all three is
`.../examples/product_workflow_support/application_entry.rs` (about 320
lines). The schema registers the binding with
`.application_mutation_binding::<AmendTemporalBinding>()`, and the
contribution registers the handler with `setup.handler::<Binding, _>(..)`
([Â§2](#2-declare-the-schema-and-its-contributions)).
`worth_query_mutation_binding!` (decl facade) writes the binding for you in
common cases.

Ordinary `mutate(intent)` continues to dispatch through
`ApplicationMutationIntent`. When a producer already selects that binding and
its input is itself the intent, `ApplicationMutationInputScope` delegates to the
existing scope expression. Explicit downstream associations generic over a
schema can overlap this blanket, even without a current intent implementation;
author them for the concrete installed schema instead. This does not add mutation dispatch or
change the binding's source, permissions or output contract.

The input and idempotency key types derive `serde::Serialize`, and that is all
a binding says about retries. Query derives both identities from a canonical
encoding of what `Serialize` emits, so a retry that reuses a key with any
changed serialized field is refused as `IdempotencyIntentDrift` rather than
replayed. Bindings never write these hashes: a request encodes its key and
input once, at the entry point, into
`ApplicationMutationIdentities::<Schema, Binding>::encode(&key, &input)`. Every
later step (admission, the handler, the commit) reuses those identities and
never encodes again, and a denial says whether the key or the input failed to
encode. A workflow operation encodes its input once when it binds to the
workflow transition, as an `ApplicationEncodedInput`, and the request's
identities reuse it. The two derivations are reported in the receipt's
admission-phase canonical work, for a replayed retry as for a fresh commit.

Application values outside mutation requests can use the same canonical
encoder through `application_value_identity`. Choose `DomainValue` and a
namespace that names the value's meaning and semantic version:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_decl::facade::application_operation::{
    application_value_identity, ApplicationValueIdentityDomain,
};

let label = application_value_identity(
    ApplicationValueIdentityDomain::DomainValue,
    "example.label.v1",
    "catalog-item",
)?;
```

For example, a catalog group can derive its identity from every field that
determines membership, including optional fields:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
#[derive(serde::Serialize)]
struct CatalogGroup<'a> {
    product: &'a str,
    axis: Option<u8>,
}

let group = application_value_identity(
    ApplicationValueIdentityDomain::DomainValue,
    "example.catalog-group.v1",
    &CatalogGroup { product: "catalog-item", axis: None },
)?;
let digest = group.identity();
let encoding_work = group.work();
```

An absent axis differs from `Some(2)`, and changing the namespace or version
changes the identity. Maps are ordered by their encoded keys; sequences keep
their supplied order and floats keep their exact bits. Serialization rejection
returns `CodecRejected` without a digest. The reported work describes this
encoding; it does not charge a live request automatically. A digest grants no
authorization, currentness or publication authority, and does not replace a
mutation request's input/idempotency identities. Include every meaningful
distinction in the serialized value: skipped fields, lossy custom serializers
and untagged variants can erase distinctions the encoder never sees. Use an
explicit version change when the application's identity meaning changes.

The binding is part of the intent too. Only typed constructors build a
`WorthQueryApplicationIdempotencyBinding`, and a mutation request builds its
own with `WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)`
from the `ApplicationMutationIdentities` it encoded, which names the binding
without encoding anything again. The handler takes that same
`ApplicationMutationIdentities`, which holds the key and input it was encoded
from, so the identities and the input the handler decides on cannot come from
different requests.

The program owner binds `Binding` into the idempotency binding it is given, so
a caller of `compare_and_commit_program_action::<Binding>` passes the
`&ApplicationMutationIdentities` it encoded, plus a closure that adds anything
the request also binds (`std::convert::identity` when nothing), and the door
builds the idempotency binding itself; a caller never names the mutation. At
commit, a candidate built by another binding's handler is refused as
`MutationBindingMismatch`, and one whose handler decided on a different input
than the idempotency binding derives from is refused as `MutationInputMismatch`.
A capability admission governs the request's own input identity rather than
encoding the input again, and the handler refuses a request whose identities
encode a different input (`MutationHandlerExecutionDenial::InputNotAdmitted`).
Hosts that author a commit without a mutation request use
`for_host_commit::<Schema, Operation, _, _>(&key, &intent)`, the only host
constructor: the same key and intent under two operations never replay each
other, and it names no mutation binding. Capability workflows (elevation,
review, delegation and revocation requests) build their binding inside Query:
the input is encoded once, its identity is both the governed input and the
intent, and the key is scoped to the operation and the requesting principal,
so the same key from another principal or under another operation never
replays the request.

Capability delegation and revocation use retained native commands. Call
`request.retain_capability_delegation_in_program(..)` or
`request.retain_capability_revocation_in_program(..)` once with the owned original
input and key. Capture refusal returns `(input, key, prepare, denial)` intact.
The preparation closure may repeat under a fresh admission; it projects and
materializes the native command program, while Query alone commits effects.
Advance the returned owner with a fresh request rather than reissuing intent.
A refusal returns the same command. A completed owner exposes native receipt
evidence and still requires `close_with_evidence(&fresh_request)`; close refusal
returns that same completed owner. See [owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands).

For a business reversal, declare a domain action with its own authorization,
source preconditions and effects, then execute it through its ordinary owner.

The identity is exactly the serialized value, so keep that value complete and
ordered:

- Serialize every field that distinguishes command meaning. Fields omitted by
  `#[serde(skip)]`, `skip_serializing_if` or a lossy custom serializer are also
  omitted from identity; inputs differing only there replay each other.
- Put anything that must matter in a serialized field. `PhantomData` and
  generic type parameters emit no data, so a `Money<C>` currency marker is not
  in the identity.
- Use ordered collections. Map entries are sorted by their encoded keys, so a
  `HashMap` and a `BTreeMap` with the same entries agree. A `HashSet` and every
  sequence keep iteration order, so a retry of the same set can be refused as
  drift. Prefer `BTreeSet` or a sorted `Vec`.
- Keep variants distinguishable. `#[serde(untagged)]` emits no variant name, so
  variants with the same payload share one identity. Floats encode their exact
  bits, so `-0.0` and `0.0` differ.
- Integers encode by value, so a `u8` and a `u64` holding the same number
  agree, but a signed and an unsigned integer holding the same number differ.
  Keep a field's integer type stable across versions.
- Treat serialized type, field and variant names as part of the durable
  identity. Renaming one changes the identity of every in-flight retry and
  of every workflow requirement that names the input; version the input
  binding identity when you do.

## 5.3 Execute through the program

Every ordinary application installs a program with `in_memory_program` or
`in_memory_rostered_program`. Declare a no-output action with
`.action::<Binding, Delivery>() ... .finish_action()`; its delivery mode is
`ApplicationSourceCompletion` or `ApplicationRetainedCommit`. Select the exact
`ApplicationActionReference`, rather than a first matching binding.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let operation = request.mutate(intent)
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(key)?
    .retain_operation::<Program, Action>(&runtime)?;
let next = operation.advance(&fresh_request);
// Retain Progress/Refused custody for a later fresh-authorized attempt.
// Completed owns evidence and lawful close; Terminal has no advance method.
```

The owner performs native prepare/commit, unpublished recovery and keyed resolution.
Capability authorization follows the binding, including recovery; an authenticated
principal alone grants no permission. Native policy supplies each attempt's
allocation and bounded work. Callers neither reset a failed attempt nor recreate
its original descriptor. Source-only completion may be followed by a separate
declared demand or assessment; it cannot replace required-output settlement.

The runnable `ordinary_product_workflow` declares a real source-only action,
advances its owner, observes native fresh receipt/result evidence, closes cleanup,
and reads both current and retained product truth through the public facade.

#### Execute a stored typed edit

Decode the stored input through its declared field codec, then pass the typed
value to the ordinary installed action. Register a delivery whose mode is
`ApplicationSourceCompletion` and construct the owning operation with
`request.mutate(input).without_source().idempotency_owned(key)` followed by
`retain_operation::<Program, Action>(runtime)`. Supply a fresh authenticated
request to `advance` and retain the returned owner until lawful close.

READ admission for the record and mutation admission for the edit use their own
declared policies. An operation's `assess_current_authorization(&request)` observes
current mutation authorization without executing it. Its successful assessment
does not authorize a later `advance`: execution checks the current grant again.
The [compiled grant transition](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/mutation_authorization.rs)
revokes the mutation grant while both model and proposal reads remain lawful,
then restores it and advances the same retained operation.

Enforce value limits with an installed commit-boundary invariant. The
[compiled retention action](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/mutation.rs)
publishes ten days and refuses eleven through the installed maximum rule. The
invariant violation returns `Terminal` with its exact commit denial and close
custody. A domain refusal also retains its original native outcome in `Terminal`.
Only outcomes classified by the native owner as retryable return `Progress` with
`RetryableNoEffect` custody. An admission refusal returns `Refused` with the
operation and typed denial. Match these variants before deciding what can be
retried or closed.

To evaluate ordered edits before selecting them as a model's state, create an
exact-source candidate with `select_expected` and `fork_expected`. Check that
the admitted product branch equals the model owner's actual head before
construction: copied branches can contain the same entity occurrences. Assess
current mutation authority on the source, then construct each ordinary execution
owner on the candidate branch. A captured source-bound operation cannot be
retargeted to the candidate.

For that source assessment, use
`request.advanced().mutate(input).without_source().assess_current_capability_authorization()`
when the binding requires a capability. The raw request's
`assess_current_authorization()` checks conventional policy. Both return an
observation without claiming a key or running a handler; retain no execution
permission from the result. An already retained operation's
`assess_current_authorization(&fresh_request)` selects its binding's declared
authorization mode itself.

The [compiled staging example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/candidate.rs)
loads a recorded three-day edit under a fresh caller, reads three on the staged
candidate while the model still reads seven, and uses `candidate.adopt` to replace
the model's actual mutable head. Prebuild metadata before this commit point and
move it into the model while holding exclusive access to that model. Keep the
returned previous branch and actual ordinary completion evidence; close the
previous branch separately. In its two-edit case, the second request for eleven
days receives the installed invariant refusal. The first staged edit remains
readable as three, the model remains seven, and explicit close disposes of the
retained candidate and operation.

Handle a close refusal according to its owning posture. A refusal before
retirement retains the candidate for a later close. `OwnerCleanupPending` means
the candidate is already retired and carries the original cleanup owner. To
dispose of that staged work, release its terminal evidence and retained readers,
then retry that cleanup owner. A held reader can keep returning
`WorldHistoryStillRetained`; keep the cleanup it returns until the reader is
released and cleanup completes.

#### Register attempts on a stable product

Use an ordinary action on a separately owned, stable storage product to bind an
attempt key to the complete selected input: proposal occurrence and revision,
model, expected basis, and ordered typed edits. Reusing that native key resolves
its original registration before another basis comparison. Changed parameters,
targets, order or vector length receive ordinary `IdempotencyIntentDrift`.
The native key includes its declared namespace and serialized caller key;
the resolved principal and scope also bind the recorded intent. Reusing the same
key from another principal therefore receives intent drift. A registration
receipt proves registration; it supplies no model-adoption authority.

The [compiled registration and result example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/attempts.rs)
uses a `LoadedProposal` to retain the originally selected domain data and its
issuing storage product. It retains no read lease or permission. On each request,
freshly admit proposal READ on that actual storage, current model READ, mutation
authorization for the exact frozen edits, and result READ before filtering cached
results. A different readable storage product cannot substitute for the issuer.
The [storage-affinity and revocation cases](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/attempts/disclosure.rs)
show these checks with real grant transitions and cached results.

Declare registration and result recording as `ApplicationSourceCompletion`
actions and construct their owning operations with `retain_operation`. Advance
with a fresh request on the original storage branch. Keep the returned owner on
progress or refusal, and close completed or terminal custody through its native
close method. If a write succeeds but result readback or close refuses, retain
that completed operation alongside the original input and actual cause. A normal
equivalent retry returns recovered evidence; it need not pass through a pending
delivery state.

Keep one pending acceptance inside the model that owns the mutable head. The
[compiled ingress](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/acceptance/ingress.rs)
freshly admits reads and the incoming frozen edits, then attempts to resolve the native
registration key before inspecting that slot or a cached result. Only an unseen
key with an empty slot may start a new acceptance. A registered key with no result
and no live continuation refuses; missing delivery is not permission to fork.
An occupied slot retains its original input until registration has established
a native intent binding. A different incoming input cannot replace it.

Revise stored proposal intent through an ordinary observed field write. The
[compiled revision operation](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/revision.rs)
checks the actual proposal occurrence, membership and expected domain revision,
then increments that revision while preserving format, model and recorded basis.
Read the proposals collection for its current revision reference; loading the
old reference refuses.
An already registered attempt keeps its original frozen intent. Revised intent
under that same attempt key receives native drift. An unregistered stale load
cannot register against the revised record. Renew the recorded basis explicitly
when starting from a moved model.

A target's display name does not preserve its occurrence after retirement.
The [compiled target replacement](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/target_replacement.rs)
requires a separate management ability through actual grantor ownership, then
unlinks, deletes, creates and links through observed native targets in one
candidate. Retention-edit authority alone cannot perform that replacement.
Read the replacement's actual target reference and record its fresh basis before
accepting a new proposal. Old target references and another model incarnation's
proposal storage refuse even when their display names are identical.

If native resolution returns `ProviderUnavailable` after that same fresh
admission, the compiled ingress can advance only the matching retained Claim
recovery owner. Changed meaning under the same key still receives native intent
drift. Missing or initial operation custody cannot use this recovery path or
start a replacement attempt.

Advance the retained acceptance with fresh requests. After adoption, retain the
actual adoption owner while recording and reading back the result, then close
the previous owned head. A result-READ refusal leaves that same adoption pending;
restore the ordinary grant and resume it. Successful retries read the recorded
result without adopting again. Closing the model while an acceptance is pending
returns the whole model so its operation and branch custody remain owned.

To abandon a pre-adoption attempt, use the model owner's `dispose_pending` with
a live request scope. It closes the actual metadata or candidate owner; it does
not execute an unexecuted edit. This owner-local cleanup works even after the
storage object has retired, without reading business data or disclosing inputs,
results or identifiers. The response is opaque; detailed native refusal evidence
stays inside the attempt and requires ordinary admission to inspect.

If candidate close returns `OwnerCleanupPending`, retry the retained cleanup
owner rather than closing the retired branch again. Release external readers
and disposed evidence receipts before expecting their retained history to clear.
Inside `with_application_advancement`, pass its phase to
`cleanup.retry_in_advancement(&phase)`. This borrows the receiving runtime's
admitted request. Standalone callers can use `cleanup.retry()` outside an active
advancement.
Disposing a registered attempt leaves its real Claim row committed; the same key
with no result or continuation still refuses. After adoption, disposal retains
the committed continuation. Use acceptance for result recording and normal
previous-head close. Inspect retained close failures through the admitted
`inspect_attempt` entrance.

A failed preparation that created no candidate can also be disposed. This
releases its pending slot while leaving its committed Claim intact; retrying
that old key without a result still refuses. Record a fresh basis and use a new
attempt key to proceed. Unpublished creation recovery, existing candidate
custody and unresolved panicked handoffs remain retained rather than being
discarded as owner-free refusals.

For a preparation's `ProductUnpublished` creation failure, `dispose_pending`
passes its phase to the actual creation recovery's
`release_obligations_in_advancement(&phase, minimum_age_ticks)`. A recovery refusal
retains that owner. Once release transfers work to owner cleanup, further
disposal retries that cleanup instead of releasing creation again. The
[compiled creation cleanup](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/acceptance/disposal/preparation.rs)
preserves the native failure for admitted inspection and completes with the real
creation-release or cleanup receipt. A late request stop preserves the completed
disposal and reports its handoff; it does not recreate a candidate.

If previous-head close returns `OwnerCleanupPending`, the head has already
retired. Keep the performed adoption evidence and stored result while transferring
that exact failure into its cleanup owner. The
[compiled recovery](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/candidate/adoption/retired.rs)
retries the original cleanup with a fresh scope, retaining its actual refusal
while readers still hold history. Release those readers and retry the same owner.
The selected new head stays in place throughout.

Inside `runtime.with_application_advancement(scope, |phase| ...)`, call the
application cleanup owner's `retry_in_advancement(&phase)`. It uses that original
runtime's admitted request for history, conditional and component cleanup. A
foreign or interrupted phase returns the original cleanup with a typed denial
before effects. Use `failure.into_cleanup()` for a fresh retry. The standalone
`retry()` owns a separate serial request; an existing advancement supplies its
phase instead. Preserve an actual successful cleanup receipt if the outer
advancement reports a later handoff interruption.

The example's `FinishedAdoption` distinguishes an ordinary previous-head close
receipt from a settled retired-head cleanup receipt. Inspect the corresponding
variant; cleanup settlement supplies its actual cleanup receipt. Keep adoption,
retirement and cleanup handoff denials with their respective outcomes. If cleanup
succeeds before a late handoff refusal, retain that settled outcome and deliver
the recorded result on retry.

#### Record an actual adoption result

Keep canonical result data separate from the command that records it. In the
example, the adoption owner has private construction and retains the original
ordinary completion evidence. `RecordResult::from_adopted` checks that owner
against the supplied claim reference and its full frozen proposal. Its command
supports serialization for ordinary mutation identity, with no deserialization
or construction from a stored result row. Fresh results retain their actual
returned values; recovered completions remain explicitly recovered.

If a request stops after a source write completes, keep the refused owning
operation. Advance that same owner with a fresh request: native authorization
and key resolution run again, and its retained Fresh result remains Fresh. The
[compiled late-stop example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/attempts/handoff.rs)
preserves the original returned value and input while the committed basis stays
unchanged on retry. Close the completed owner after consuming its evidence.

The [ordinary result handler](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/attempts/recording/handler.rs)
readmits actual claim membership and full-generation identity, observes caller correspondence
and complete result absence before creating the record. This lets publication
validate those facts, including competing writes. Its recording receipt proves
that write. Read the result through its declared canonical codec and current
governed result policy; a cached row grants no permission to execute or adopt.

#### Capture complete original meaning

Install a program and declare the exact typed action occurrence and its delivery.
`request.mutate(intent)` returns `WorthQueryOperationBuilder`; it exposes original
source preparation, preconditions and `idempotency_owned(key)`, followed by
`retain_operation::<Program, Action>(&runtime)`. Fixed, discovered and source-only callers
cannot execute, prepare or recover through that builder or its borrowed-key view.
The selector carries composition instance, feature, binding and delivery;
Installation resolves the complete actual action, including every metadata
contract. A binding label alone cannot select its contract.

`idempotency_owned` first opens the initiating request's native capture phase and
returns `Result<Description, (original_builder, owned_key, capture_denial)>`.
Retain all three refusal fields. No business effect or workflow-slot transfer has
occurred. A later capture retry uses that builder and key with a fresh native
opening. Success freezes one typed input payload under the existing Execution
allocation policy; encoded clones and native capability consumers share that
same backing and keep its charge until their last owner releases it. The quote
covers typed payload, not nested caller Clone heaps or allocator/Arc metadata.
The original Intent is moved, never cloned by this capture.
An adapter whose initiating scope ends can consume the returned builder with
`builder.into_capture_refusal(key, denial)`. This transfers the original input,
conditions, source, key and branch into `WorthQueryOperationInputCaptureRefusal`
without retaining the initiating request or principal. Its `retry_capture`
requires a fresh request on the original runtime/branch and delegates the same
native capture; every refusal returns the owning original. It never executes a
source, consumes a workflow occurrence or creates a completed description on
failure. Bank construction returns this owner in
`BankOperationConstructionRefusal::Capture`; declaration refusal retains the
funded description separately. Transport may decline an unexecuted request,
which is distinct from losing a post-effect cleanup obligation.

Fixed, discovered and source-only owners retain the authentic installed runtime
allocation and complete installed action by value. Their returned types have no
construction-runtime lifetime; they can be stored after the constructing borrow
ends or the constructing handle is dropped. Keep a separate genuine runtime
handle for fresh request construction. Operation owners expose no runtime
accessor or `Deref` for admission. Retention shares the existing allocation; it
copies neither the graph nor native obligations and grants no permission.

Configure workflow vocabulary and other unique-runtime setup before cloning
handles or retaining operations. While an operation retains the allocation,
runtime close refuses with `RuntimeKeepaliveRetained`. Successful operation
close releases that operation's keepalive; refusal returns the complete owner.
Dropping an owner abandons custody and does not perform successful cleanup.

For an eligible Required fixed leaf, a producer's terminal domain refusal can
close through the same ordinary terminal owner. `terminal.close(&fresh_request)`
delegates native negative completion before cleanup; a refusal returns the same
owner. If native finish succeeds but cleanup refuses, inspect
`terminal.output_unavailable()` and retry close with fresh authority. Successful
close carries that actual evidence in `closed.output_unavailable()`: the original
source was saved and no current output was produced. This is neither successful
required-output settlement nor permission to publish a positive result. Native
eligibility still refuses unresolved preparation, partial work, already settled
roots, dependent trees and discovered aggregates. No source or producer is
reexecuted to close. The executable ordinary fixed-leaf court is
[`terminal_unavailable.rs`](../workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/producer_predicate_checkpoint/ordinary_source/terminal_unavailable.rs).

Capture the original meaning once. Supply a fresh authenticated ordinary request
for every `advance`, delivery and close. Native admission checks current authority
and the original runtime/branch/program affinity. The descriptor is not a grant.
Recovery promotes the original native carrier; registered replay rejoins its
original outputs without running the source handler. Recovered evidence does not
manufacture the original handler result.

A fixed settled owner borrows its complete settlement for `read_current` or a
finite `read_current_with_policy`; the currentness check and declared query share
one observation and one native phase. Read refusal retains the owner. Stale
settlement supports `read_historical` and `close`, never output advancement.
No-projection delivery uses `deliver_current` without a dummy query. Source-only
completion is distinct from required-output settlement; retained delivery selects
the action's authentic original commit observation.

Completion and delivery retain independent cleanup. `close` returns the complete
owner with a typed denial when cleanup refuses. Drop is abandonment, not external
cleanup or crash durability. Only existing native checkpoint/recovery contracts
survive process loss.

<!-- compiled: owning_entry.rs -->
```rust
#[allow(
    clippy::result_large_err,
    reason = "The example returns the complete original builder and key without allocation"
)]
pub fn capture_original<'a, 'p, 's, S, I>(
    request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
    intent: I,
    preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation,
        <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    source: WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> Result<
    WorthQueryOperationDescription<S, I>,
    (
        WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared>,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
        WorthQueryOperationInputCaptureDenial,
    ),
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
{
    request
        .mutate(intent)
        .preconditions(preconditions)
        .expect_result_set(source)
        .idempotency_owned(key)
}
```

The real result-set court calls this function with nondefault preconditions before
an actual post-effect append failure. It reads the resulting source and outputs
independently after native recovery. No original intent clone is required per pump.

The ordinary workflow binding retains `WorthQueryWorkflowOperation` through
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
Advance it with a fresh request; retain the returned owner on refusal or progress.
The original slot, native recovery and cleanup remain owned by Query.

#### Advanced native mutation control

Use the [ordinary owning entry](#53-execute-through-the-program) for fixed,
discovered, source-only and guarded workflow actions. Deliberate native callers
select `request.advanced().mutate(intent)`. Only that type exposes manual
fixed/discovered/source-only execute, two-phase prepare, unpublished recovery,
keyed resolution and fixed output start/recovery. Borrowed native workflow
execution binds the issued requirement before selecting
`for_workflow_operation(..)?.advanced().execute_in_program(..)`.
A runtime installed without a
program (`application_installation::in_memory`) is advanced native control;
its `execute` cannot be an ordinary application default. On a program runtime
non-program execution refuses with `ApplicationProgramRequired`.
Native calls take an explicit `ExecutionAllocationPolicy`; the owning attempt
uses its admitted native policy. Unsupported predecessors refuse before effects
or executable readmission and preserve original bytes; no compatibility entrance
is retained.

#### Storage beyond the construction borrow

`WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>` and
`WorthQueryDiscoveredOperation<Schema, Intent, Program, Root, Action>` carry no
runtime lifetime parameter. Return or store the actual owner across commands;
retain the operation returned on each progress or refusal. Fresh requests must
come from a separate genuine handle for that same allocation and original
branch, not from the operation itself.

The fixed consumer below first clones its authentic runtime handle for request
construction, moves the original handle into `constructing`, retains the
operation, and drops `constructing` before advancement. It uses the still-owned
original allocation to construct request authority, resolves and admits the external
provider's actual value and checks the resulting source and outputs. This is
the same installed fixed journey, not a second runtime or a fabricated lifetime.

### Compiled owning call bodies

These excerpts are byte-bound to their actual compiled consumer fixtures or
registered examples. Their enclosing sources supply the installed program,
authenticated request, schema imports and real seed world. They are not
standalone programs. The existing bounded fixture pump calls one fresh native
advancement at a time; production code retains the returned owner between
requests, including every refusal and pending result.

Fixed: the two-instance court selects its complete primary action, uses admitted
external input and the installed evaluator, drops the constructing runtime
handle, advances the original owner, observes source/output truth and closes.

<!-- compiled: owning_fixed.rs -->
```rust
    use crate::application_program::{
        composed_action::CatalogInputRequirement,
        external_input::{NeutralExternalDenial, NeutralExternalProvider},
    };
    use worth_query_topology_entry::AdjustPlanarSource;
    let external = constructing
        .installed_program()
        .external_input_provider::<AdjustPlanarSource, NeutralExternalProvider>()
        .unwrap();
    let provider = NeutralExternalProvider::new(7);
    let before_input_refusal = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let captured = external.resolve(&provider, "replacement-y").unwrap();
    provider.change(2);
    assert_eq!(
        captured.admit(&provider).err(),
        Some(NeutralExternalDenial::Changed)
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before_input_refusal
    );
    let accepted = external
        .resolve(&provider, "replacement-y")
        .unwrap()
        .admit(&provider)
        .unwrap()
        .into_resolution();
    assert_eq!(accepted.revision(), &2);
    let requirement = constructing
        .installed_program()
        .evaluated_requirement::<AdjustPlanarSource, CatalogInputRequirement>()
        .unwrap()
        .evaluate(accepted.revision());
    assert_eq!(
        requirement.required_input_guidance(),
        Some(&"catalog-value")
    );
    assert_eq!(
        requirement.enforce_submission(false).unwrap_err().missing(),
        &"catalog-value"
    );
    requirement.enforce_submission(true).unwrap();
    let primary_input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(*accepted.values()),
    };
    reset_planar_source_adjustment_contacts(&primary_input);
    let source = request
        .query(PlanarRead {
            body_key: primary_input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let primary = request
        .mutate(primary_input.clone())
        .expect_source(source)
        .idempotency_owned(10_193)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<ActionInstanceProgram, ConsumerFixedAction>(&constructing)
        .unwrap_or_else(|(_, denial)| panic!("primary exact contract: {denial:?}"));
    drop(constructing);
    assert_eq!(world.application.close_conditional_runtime().unwrap_err().kind(),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained);
    // Fresh requests come from the independently retained authentic allocation.
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let request = world.application.request(&fresh_principal, &fresh_scope);
    let mut primary = Some(primary);
    let primary = (0..32)
        .find_map(|_| match primary.take().unwrap().advance(&request) {
            Advance::Progress { operation, .. } => {
                primary = Some(operation);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native fixed advancement: {denial:?}")
            }
        })
        .expect("the declared fixed graph settles");
    assert_eq!(primary.declared_action(), &primary_declaration);
    assert_eq!(
        primary.declared_action().delivery().unwrap().identity(),
        <ConsumerFixedDelivery as ApplicationActionDelivery<ConsumerSchema, Binding>>::IDENTITY
    );
    assert_eq!(primary.settlement().output_count(), 4);
    let output_delivery = primary.read_current(&request).unwrap();
    assert_eq!(output_delivery.rows()[0].body_key, "anchor-a");
    assert_eq!(output_delivery.rows()[0].value, length(3));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: primary_input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(*accepted.values())
    );
    assert_eq!(planar_source_adjustment_contacts(&primary_input), (1, 1));
    primary
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("primary close: {cause:?}"));
    world.application.close_conditional_runtime().unwrap();
```

Discovered: the existing public journey carries its original input through required-set settlement, guarded delivery and close.

<!-- compiled: owning_discovered.rs -->
```rust
    let operation = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency_owned(10_030)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("declared discovered construction: {denial:?}"));
    let mut operation = Some(operation);
    let settled = settle(|| match operation.take().unwrap().advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation: next, ..
        } => {
            operation = Some(next);
            None
        }
        WorthQueryOperationAdvance::Settled(settled) => Some(settled),
        WorthQueryOperationAdvance::Refused { denial, .. }
        | WorthQueryOperationAdvance::Terminal { denial, .. } => {
            panic!("ordinary discovery: {denial:?}")
        }
    });
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(42)
    );
    let roots = settled.settlement().root_outputs().collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    assert_eq!(
        roots
            .iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.settlement().output_count(), 6);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0,
        "settling all discovered roots releases the performed source and observation"
    );
    for (key, expected) in [
        ("sibling-b", length(22)),
        ("sibling-c", length(31)),
        ("remote-b", length(42)),
    ] {
        assert_eq!(
            request
                .at(settled.settlement().observation())
                .query(PlanarOutputRead {
                    body_key: key.to_owned()
                })
                .execute()
                .expect("each settled output is readable")
                .rows()[0]
                .value,
            expected
        );
    }
    assert_eq!(
        request
            .at(&prior)
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the prior authored source remains coherent")
            .rows()[0]
            .y,
        length(1)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful discovered close: {denial:?}"));
```

Source-only: the runnable ordinary product example retains a no-output action, distinguishes real Fresh evidence from recovery and closes.

<!-- compiled: owning_source_only.rs -->
```rust
    let operation = request
        .mutate(AmendTemporalIntent {
            identity: "intent-1".to_owned(),
            amendment: AmendTemporalInput {
                revision: 2,
                due: 11,
                lifecycle: "active".to_owned(),
                input: "published-through-application-entry".to_owned(),
                gate: "ready".to_owned(),
            },
        })
        .without_source()
        .idempotency_owned(0x51_u64).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<product_workflow_support::program::TemporalExampleProgram,
            product_workflow_support::program::AmendTemporalAction>(&application.runtime)
        .unwrap_or_else(|(_, denial)| panic!("declared temporal action: {denial:?}"));
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("fresh attempt refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("this successful native attempt has no pending publication")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("native terminal: {:?}", terminal.native_outcome())
        }
    };
    let committed_version = match completed.evidence() {
        WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } => {
            assert_eq!(result.revision, 2);
            receipt.commit_reference().version_id.as_u64()
        }
        WorthQuerySourceCompletion::Recovered { .. } => {
            panic!("a new key must produce the actual handler result")
        }
    };
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("native cleanup refused: {denial:?}"));
```

Workflow: the runnable authored example obtains the actual awaited slot, captures before binding it, advances the exact installed action and closes. A later workflow advance remains separate.

<!-- compiled: owning_workflow.rs -->
```rust
    let required = match advance_instance(application, instance.clone(), key)
        .expect("the approved effect requirement prepares")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("the approved effect must be awaited, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .without_source()
        .idempotency_owned(key + 1).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_|panic!("exact original approved occurrence"))
        .retain_operation::<document_retention_model::programs::RetentionProgramP0,actions::ReviewedAction>(application.program_runtime())
        .unwrap_or_else(|_|panic!("installed approved effect action"));
    let effect=match effect.advance(&runtime.request(&principal,&scope).on_branch(instance.branch())) {
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Completed(done)=>done,
        _=>panic!("lawful approved effect"),
    };
    effect
        .close(
            &runtime
                .request(&principal, &scope)
                .on_branch(instance.branch()),
        )
        .unwrap_or_else(|_| panic!("native effect cleanup"));
    assert_eq!(read_retention(runtime, instance.branch()), days);
```

### 5.4 Mutations with required outputs

Fixed owning construction validates the installed governed root artifact's
actual source locality, change/succession and dependencies before minting an
effect-capable owner. Refusal returns the original operation description and
`WorthQueryOperationConstructionDenial::OutputSource`; the borrowed authentic
runtime remains with the caller, and no source effect has run. This uses the
same native artifact-source guard as later required-output preparation.


For a fixed required-output action, declare its exact composition occurrence,
output root and projection. `ApplicationFixedActionDelivery<Schema, Binding>`
names `Root` and `Projection`. For a query, choose `ApplicationQueryProjection<Read>`
and implement `ApplicationFixedQueryDelivery` with `Read` and
`read_input(&Binding::Input)`. For required outputs without a projected read,
choose `ApplicationNoProjection`; it has no query binding or selector. The
`ApplicationActionDelivery` contract supplies the single stable `IDENTITY` and
`ApplicationFixedDeliveryMode<Root, Projection>`.
The installed declaration binds the concrete `Delivery` implementation. Another
type cannot substitute its selector by copying the delivery identity, root, and
read metadata. The identity still supplies stable canonical meaning; runtime type
stamps are not persisted.

Author that source with `.action::<Binding, Delivery>() ... .finish_action()`; use an
`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>` to
select it. Program validation requires the delivery root to belong to that
program and to start at the action's actual source occurrence.

An action with no native output-source obligation declares
`ApplicationActionDelivery::Mode = ApplicationSourceCompletion` for no projection,
or `ApplicationRetainedCommit` for its authentic commit observation. Author it
with `.action::<Binding, Delivery>() ... .finish_action()` and select its exact
`ApplicationActionReference` through the same `retain_operation`. A copied
identity cannot replace a required-output delivery, and the runtime's actual
output-source inventory must also permit the no-output lane.

The description freezes encoded input custody once. Native initial admission,
recovery and keyed resolution share that immutable input; attempts borrow the
original preconditions and fresh request authority. Source completion returns
receipt-only success, while retained commit also returns its native observation.
Both execution paths derive allocation policy from the supplied admitted phase.

Binding-selected admission uses `ApplicationMutationAdmissionBinding::Admission`:
`ApplicationConventionalAdmission`, or `ApplicationCapabilityAdmission<Capability>`
with the existing `ApplicationCapabilityMutationBinding`. Generated conventional
mutation bindings supply the conventional association; the macro's explicit
`admission` type selects another declared association. Fresh native initial,
unpublished-recovery and keyed-read handoffs borrow the original intent, key,
source and full preconditions. They store no principal or request scope.

For fixed delivery, an idempotent `AlreadyCommitted` source advances through
`SourceReplayObserved` and then `RegisteredOutputsRecovered`. Each attempt uses
fresh binding-selected admission and the exact installed action. Query resolves
the original key and rejoins the registered native output demand from the
original input; it never reruns the source handler or supplies a missing handler
result. Keep the returned owner on `Refused` and retry with a fresh request.
Missing, retired or superseded registration returns terminal historical source
evidence instead of output settlement. Terminal `close` releases actual cleanup
obligations; it does not mean the declared outputs completed.

The executable [fixed reply-loss journey](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_replay.rs)
checks registered recovery, real observation-pressure refusal and retry, and
historical replay after completion through the same ordinary entrance.

The SourceOnly owning adapter delegates the shared native admission, recovery
and keyed-resolution spines. Source completion uses advance with a fresh request;
retained commit additionally requires the explicit NonZeroUsize history-work
bound on each advance. Replay/recovery select the exact original World observation.
A bounded history refusal retains operation and evidence for a fresh funded retry.
See the [retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md).
Ordinary SourceOnly callers retain this owner rather than coordinating manual
execution and recovery.

`WorthQueryOperationCleanup` retains actual pending records and release failures.
Its phase-taking close returns a cleanup receipt, not whole-operation completion.
Only the owning lane may report closed after its other native obligations are
lawfully discharged. Refused cleanup retains custody for a fresh admitted retry.

The owning entrance retains the original intent, key, typed preconditions,
row or result-set expectation, and branch. Construction admits installed meaning
without performing an effect. A refused constructor returns the original
`WorthQueryOperationDescription`, so it can be retained or lawfully constructed
against the correct declaration. It retains no authenticated principal or grant.

Shape, with illustrative names; the executable counterpart is
[`fixed_delivery.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_delivery.rs):

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Shape. Illustrative names.
let description = request
    .mutate(AdjustDraft { /* ... */ })
    .expect_source(source)                 // or expect_result_set(result_set)
    .preconditions(original_conditions)
    .idempotency_owned(key)?;
let operation = description.retain_operation::<MyProgram, AdjustDraftAction>(&runtime);
// On construction refusal, retain the returned (description, denial).
let operation = retain_or_handle_construction(operation);

match operation.advance(&fresh_request) {
    WorthQueryOperationAdvance::Progress { operation, progress } =>
        retain_for_next_request(operation, progress),
    WorthQueryOperationAdvance::Refused { operation, denial } =>
        retain_refused_operation(operation, denial),
    WorthQueryOperationAdvance::Terminal { operation, denial } =>
        retain_terminal_native_custody(operation, denial),
    WorthQueryOperationAdvance::Settled(settled) => {
        consume_or_retry_delivery(settled.read_current(&fresh_request));
        retain_or_finish_close(settled.close(&fresh_request));
    }
}
```

Each advance receives fresh request authority for the same runtime and branch.
Query borrows the original description into its existing native admission and
execution owners. A real unpublished source continues its original partial;
recovery and promotion never invoke the source handler again. Genuine native
handoffs are reported as progress. Native output `Pending` is reported only
when the existing output owner reports it.

Settlement does not invent a recovered handler reply: `handler_result()` and
`recovered_progress()` remain distinct. `read_current` retains one fresh
observation, requires every settled output's native lineage at that observation,
and executes the declared read there. Refusal retains settlement and original
meaning for a later read; it does not perform the source or outputs again.
`read_current_with_policy` accepts `WorthQueryOperationDeliveryPolicy` for one
attempt. Ordinary currentness and query work have no intrinsic operation-count
ceiling. Explicit currentness, query or attempt restrictions still intersect the
receiving policy. Omit `.operation_projection_work_budget(..)` to declare no
decision-read work ceiling. Ordinary output demands can omit controls; use
`WorthQueryOutputDemandControls::default().source_currentness_work(work)` or
`.settlement_attempts(attempts)` only for a deliberate restriction of the existing
currentness or settlement policy.
Signal runtime presets also leave waiter-resolution and upstream-dependency work
absent. `with_maximum_waiter_resolution_visits(Some(n))` and
`with_maximum_upstream_dependency_visits(Some(n))` are explicit finite choices;
`None` keeps the original request and physical admission without a local work cap.
The installed workflow fact-custody rule likewise checks every candidate on the
original request without imposing a fixed candidate count.
### Producer source and publication

A `WorthQueryApplicationProducerProvider` supplies its semantic identity,
`operation_input` and `idempotency_key` from the authentic source. Its optional
`domain_denial_reason` supplies diagnostic text. The installed operation's handler
prepares and writes the actual effects through the same typed mutation path.
Source admission, output contracts, currentness and atomic publication remain
enforced without a producer resource forecast.

#### Source-owned producer publications

A producer defaults to independent publications. Opt in to source-scope cascade
retirement on its `WorthQueryApplicationProducerBinding`:

```rust,ignore
const OWNERSHIP: ProducerPublicationOwnership =
    ProducerPublicationOwnership::SourceScopeCascade;
```

The declaration means that the producer's complete publications belong to its
recorded source scope. Installation requires an entity-specific input-field
scope and a source query rooted at that same entity, without a context traversal,
component/orbit selection or managed-index root selection. Shared/context and
principal scopes refuse with `ProducerOwnershipRequiresEntitySourceScope`, naming
the producer. Initial and Preserve producers in one family must agree on ownership.
The ownership rule participates in installed producer identity.

Native retains each live owned publication's members, edges, route and generation
as private current records. Current custody checks use those facts through cold
reopen and ordinary source edits; they do not require historical envelopes.
Only the publication commit path maintains the records. Document checkout must
restore them together with the producer outputs; their ordinary owned relations
let the same closure selection include them. Direct metadata writes refuse
`OrdinaryMetadataWrite`. Older stores without the current roster require the
new format and refuse readmission; no migration is supplied.

Deleting a member, unlinking an owned edge or changing its endpoints requires
that exact route's current succession or an admitted cascade covering it.
Another route's Preserve grants no custody, even when both publications share
one commit ID. The refusal retains the current publication and actual record
and owner subjects.

An authored `retire_created` or `restore_retired` binding supplies the exact source
identity. Native admits an ordinary authored Create as the distinguished anchor
and independently computes every live opted-in publication scoped by that source,
then publications scoped by their members. Submit exactly the anchor and every
live member of those complete publications. Partial, repeated, extra and
foreign-source rosters refuse with typed causes and affected native records;
retirement publishes once. Inspect `commit_denial.authored_retirement_error()` for
its original `authored_retirement_refusal()` and `context.affected_records`.
`restore_retired` restores the source and every owned
publication in one transaction, preserving each producer's provenance and route
while allocating fresh native generations and remapping owned relation endpoints.
A restoration supplies historical output association, not producer readiness.

A correction may explicitly carry co-anchors with
`retire_created::<C, Family, Source, Inverse>()`
`.co_retire_created::<C, CoFamily, CoSource, CoInverse>()`. For a Retire-to-Create
inverse, use `restore_retired` followed by `co_restore_retired` instead. Each
co-role declares its own family and Create/Retire correspondence, with matching
exactly-one or zero-or-one cardinality. An absent optional role remains absent.
Every present co-anchor must be genuinely authored and belong to no live
publication. Native selects one closure from all anchors together; an undeclared
extra Retire role is still an extra. Each restored anchor comes from its own
inverse Create, retaining its logical key and receiving a fresh native ID.
Declare and observe the demanded pre-image fields for each present anchor and
set the pre-image byte bound for their total payload.
The demand may include different entity kinds, such as a Feature and its
co-born Body, when every kind is a declared authored anchor of that operation.
Installation refuses an unrelated entity kind with that kind as its subject.
Read a particular co-anchor with
`preimage.field_for_output(&CoRole::DESCRIPTOR, &locus)`; equal field loci on
different anchors retain their own original values.

An authored anchor may also own a variable number of independently keyed
members. Extend its existing authored correction declaration:

```rust
ApplicationAuthoredCorrection::restore_retired::<C, AnchorFamily, Retired, Created>()
    .owned_members::<C, MemberFamily, RetiredMembers, CreatedMembers, _, _>(
        MemberOf::reference(),
    )
```

`RetiredMembers` and `CreatedMembers` are ordinary output-role families with
minimum zero and the respective single lifecycle posture. `MemberFamily`
declares the member's native logical-key field. Installation requires the named
owner relation to run from that member kind to the anchor kind, with exactly
one target per live member (`source_min = source_max = 1`). The roster is
exactly the live sources of that relation into the anchor; there is no stored
roster list and no implied ownership from another relation.

The ordinary candidate binds every retiring member, unlinks its owner and
intra-roster relations, and deletes the anchor and members in the same commit.
Native compares the actual owner adjacency with those roles before relation-law
validation. `MissingOwnedMember`, `ForeignOwnedMember`, `RepeatedOwnedMember`,
`ExtraOwnedMember` and `DanglingOwnedMember` retain their affected records through
`authored_retirement_error()`. A member cannot substitute for its typed anchor.
Restoration creates that anchor and all original members with their logical keys
and fresh Native IDs. Declare and observe the member key and every field to
restore in the existing `DeclaredPreImageDemand`, and add the owner and changed
intra-roster relation loci with `with_relations`. The recorded inverse compares
relations after remapping the original anchor and member endpoints to those
fresh IDs. Exactly-one owner cardinality still refuses a restored member without
its anchor. An outside relation into a member remains outside the closure;
`RejectDeleteWithLiveRelations` still produces the typed `EndpointDeletion` denial.

The demand's `maximum_encoded_bytes` bounds the total logical retained payload,
across the anchor and every member. Fields charge their native scalar semantic
width, including the scalar tag. Each relation transition charges 86 bytes for
its complete record/kind and before/after endpoint coordinates; retained content
also spends the same bound. A field total over the bound refuses as
`WorthQueryPreImageRetentionDenial::ExceedsByteBound`; a relation total over it
refuses as `WorthQueryRelationPreImageDenial::ExceedsByteBound`, before any effect.
A whole-roster replace may retire the old anchor and create its replacement
under the same logical key in one candidate. Declare the replacement's optional
Create-to-Retire co-anchor with `co_retire_created` and give its members their own
Create-to-Retire output families through `owned_members`. The retired anchor and
members use the opposite correspondence. The existing recorded inverse then
retires the exact replacement and restores the prior anchor, every prior member
and the original links, including removed members. Membership always comes from
the owner relation; the replacement intent is an ordinary typed command, not a
stored source snapshot. Each lifecycle family has its own explicit posture.
The unique-field owner admits a replacement key only when that same sealed
batch deletes its exact observed holder. The original indexed selection stays
in decision support; deleting another holder or claiming the value twice still
refuses. An empty roster retains its intra-member empty loci from the genuinely
observed empty owner adjacency, rather than inventing a member to read.

A local recorded correction can delete zero to many members of several
families while their owners stay live. Declare each family with its own owner
relation:

```rust
ApplicationAuthoredCorrection::live_owner_members::<C, MemberFamily, Deleted, Restored, _, _>(
    MemberOf::reference(),
)
.and_live_owner_members::<C, NoteFamily, DeletedNotes, RestoredNotes, _, _>(
    NoteOf::reference(),
)
```

`Deleted` and `Restored` are output-role families with minimum zero and one
lifecycle posture each; an exactly-one role is refused at installation. The
inverse answers each source role through the inverse family role with the same
suffix. Each owner relation is the same exactly-one relation used by the
roster and must be in the relation demand; this never requests a roster
replacement. Demand and observe every changed incident relation, the logical
key and all fields to restore. A demanded relation the mutation did not change
retains as empty, so `relation_count` there reports the mutation's own changes.
A mutation that moves no member commits, and its inverse restores none.
The retained fields are the mutation's own changes: every demanded field of
each member it retires, and each demanded field it changes on a record that
stays. The live owner's kind is covered by the declared owner relation in the
relation demand, so one inverse may also restore changed owner fields beside
its deleted members. An unrelated kind still requires its own authored anchor;
a relation absent from the demand cannot cover its owner's fields.
Witness a staying member through a Preserve family role and read its
prior value with `field_for_family_member`; a field the mutation left alone is
absent. A retiring member may lack a demanded optional field or optional
aspect: read it in the handler, and the observed absence retains nothing, so
`field_for_family_member` answers `None` and the inverse leaves it unset. A
demanded field the handler never read on a retiring member refuses with
`MissingDemandedField`, present or not. This holds for every authored anchor
and co-anchor, and costs one probe per absence the attempt observed.

Each original Retire role identifies one deleted member. Native's validated
relation footprint matches its retained family key to exactly one fresh member
and carries that old-to-fresh pair into relation inversion. The family's
logical key is the only correspondence. The original live owner must remain its
owner, and the retained member fields must match exactly. A changed key refuses
with `IncompleteInverse`, duplicate recreated keys with `RepeatedInverse`, a
different owner with `ForeignEndpoint`, a recreated member of a declared family
that the source did not retire with `ExtraInverse`, and changed retained fields
with `RestoredFieldMismatch`. These typed pre-image failures name the affected
member through `relation_preimage_failure()` and publish no effects. This uses
the same total retained-byte bound and never re-evaluates the original
candidate or recreates the unaffected roster. The member pass costs one
correspondence probe per created member against the recorded members; it never
reads a roster.

Historical member fields are read using `family_member_suffix`,
`family_member_entity` and `field_for_family_member` on the recorded pre-image.
These descriptive values grant no current read or mutation authority.

If a retired authored source is still linked by a surviving source-owned
publication, declare that owner's root through the program's existing output
connection. The mutation binding implements
`WorthQueryApplicationRequiredOutputSource<Schema, BodyRoot>` and lists the
connection in its mutation declaration. Coupled demands implement `Send` so the
mutation request can move between execution workers:

```rust,ignore
coupled_outputs |visitor| {
    visitor.required::<BodyRoot>();
},
```

`demand_from_source` derives the typed demand from the original input;
`demand_from_preimage` can also use the retained correction material. For example,
a stage retirement can derive its body demand from that stage's body key.
`ApplicationCorrectionBinding::visit_coupled_outputs` declares the correction's
roots and defaults to the source binding's declaration. Install those same roots
and connections in the program output graph.

Query runs the declared owners' Preserve producers against the unpublished
candidate. Each producer removes and replaces its own records and edges. Native
independently determines the required owner set; missing, repeated, extra or
foreign participants refuse with typed causes and affected records. The authored
retirement and every admitted Preserve publish in one atomic commit. A required
coupling with no declaration refuses before any effect. Declarations remain
inert when no surviving owner is required; a closed owner already covered by the
retirement closure does not run a producer.

A recorded creation inverse may retire a producer, its owned publication and
its generated roster while retained owners perform their own cleanup. Aggregate
preparation treats paths incident to an entity retired by this same candidate as
retired paths, including while a coupled writer still holds its deferred edges.
Native compares the actual before and candidate images; an endpoint already
unavailable before this candidate still refuses with `RecordUnavailable`.
Commits that delete no entities carry only a constant-size deletion summary and
perform no additional endpoint acquisitions.

Correction derives the same declared coupling with fresh source IDs. Native
restores closed publications, while surviving owners run their own Preserves.
A subsequent demand for a recreated owner follows ordinary Preserve and reuse
rules using the actual producer antecedent, including after repeated Native-only
restore/retire cycles. Restoration grants no producer readiness or replayable
producer-input contract.

For an authored splice that changes relations, add typed relation loci to the
recorded inverse's pre-image demand:

```rust,ignore
let demand = DeclaredPreImageDemand::new(field_loci, maximum_encoded_bytes)
    .with_relations([
        DeclaredPreImageRelationLocus::from_relation(PreviousStage::reference()),
    ]);
```

Use `relations_only` when the inverse needs no field material. The original
operation must declare and perform the relation reads, including absence at a
new endpoint pair. Correction material distinguishes an authenticated empty
locus from unavailable material with `relation_count`; `relation_at` supplies
each original/performed endpoint pair. `output_entity` supplies the original
role coordinate. These coordinates describe history and grant no current write
access: reacquire a surviving endpoint with `reader.resolve_historical_entity`,
and use each inverse Create for a restored authored anchor. Remove the original
replacement links and recreate exactly the prior endpoint pairs. A changed,
missing, extra or foreign inverse refuses before effect with
`relation_preimage_failure()` and its actual `affected_records`. Audit-retained
relation deletion currently refuses `UnsupportedVisibility` before the source
edit because the existing correction support rules cannot reverse that visibility.
Explicit Unlink supports a true deletion and exact restoration.

A tracked orbit whose aggregate and complete walk are wholly within the actual
admitted producer's own predecessor publication supplies output reads. On succession Query retains the
actual committed postcondition, so replacing an owned edge does not cause a
second publication. A real upstream change still re-runs the producer, and a
read of another publication remains a source observation. The original raw reads
still participate in candidate admission and complete-candidate support checks.

Native derives private aggregate membership deletion and recreation through its
existing aggregate consequence owner. Applications supply the exact changes for
application-visible owned edges; they never supply private membership records.

An owned publication cannot use the independent single-publication retirement
path, including a warm publication capsule. It refuses with
`OwnedPublicationRequiresCascade`, identifying the publication and its owner.
Anchorless mixed Create/Retire candidates receive the same refusal.
An intermediate source created by a producer is not a genuine authored anchor.
When a producer succession deletes live members of its own predecessor publication,
Native treats those exact deleted members as cascade anchors. It computes the
complete transitive closure of opted-in publications they own from recorded
history. The producer candidate must delete every live closure member and owned
edge and declare exactly those retirements, in the same atomic commit as its
ordinary Create/Preserve/Retire membership. Partial, missing, extra, repeated and
foreign retirements refuse with typed causes and affected records. Borrow
`demand_denial.authored_retirement_error()` for that original Native error.
The surviving generator keeps its own Preserve origin; every retired publication
retains its own provenance, route and predecessor. This also covers an independent
generator whose removed members own opted-in children.

Regrowth creates fresh source generations and fresh owned publications through
the existing producer rules. Undo the authored size edit through its recorded
correction, then demand outputs again; redo selects the recorded reversal
alternative and uses reapplication. These size-edit inverses do not restore the
old derived generations. Whole-source authored restoration retains its existing
`restore_retired` rules. Every intermediate publication must opt
in for a producer-created source to join the authored root's cascade.
An owned publication cannot be
restored or republished without its live source (`PublicationOwnerUnavailable`).
Commit admission also fences ordinary Delete and Unlink operations against
current opted-in publication membership and owned edges, including calls with
no output roles. Only that publication's authentic succession or an admitted
cascade covering it can discharge custody. A refusal names the exact entity or
relation and its owner, before any effect. Deleting an external endpoint also
requires its live owning publications to participate; it does not adopt the
endpoint as an owned member.

An input dependency or a reference never implies ownership. Two independent
producers may read the same source without entering its cascade. Opting in does
not make an external endpoint of a producer-owned relation an owned entity.
See the [custody contract](../plans/WORTH-query/authored-retirement-custody.md).

#### Declared predecessor reads

An owned producer can read its inputs as its exact predecessor publication
consumed them. Declare its installed read-only operations beside `OWNERSHIP`:

```rust,ignore
const PREDECESSOR_READS: Option<WorthQueryPredecessorReadContract<Schema>> =
    Some(WorthQueryPredecessorReadContract::new(&[
        WorthQueryPredecessorReadOperation::new(ReadInputs::reference()),
    ]));
```

These declaration types live in `host::application_contribution`. Installation
requires `SourceScopeCascade`, no duplicate operation, and actual installed
operations with no mutation touches, emissions, external effects or executable
body. The contract participates in producer equality and edition identity.
A declaration grants nothing. Each read independently authorizes that read
operation's installed authorization and abilities **at the current control basis
C**, using the producer's current principal and original source scope. It never
borrows a historical grant or substitutes the producer operation's permission.

Inside the producer handler, use the sealed decision-reader door:

```rust,ignore
let source = reader.scope().clone();
let previous = reader.predecessor_observation(ReadInputs::reference(), |old| {
    let source = old.entity(Source::reference(), &source)
        .map_err(HandlerExecutionDenial::new)?;
    old.field(&source, InputValue::at("InputValue"))
        .map_err(HandlerExecutionDenial::new)
});
```

The outer result describes observation admission; the closure's result describes
its read. The restricted reader exposes tracked entity, field and relation
reads. Its `PredecessorEntity` and `PredecessorRelation` handles are distinct from
current projection identities and cannot mint mutation targets. There is no raw
reader, nested query, computation or publication access. Reads use the existing
Native loan, request checks and projection limits. Current control contention
returns `CurrentControlBusy` without waiting on an earlier lock from that loan.
Admission and control loss veto publication even when a handler catches them.

Native chooses exactly the predecessor checked by the ordinary succession fence.
Its decision root is retained by reference under the live head/candidate lifecycle;
succession replaces the hold and retirement releases it. Historical keys and
publication/basis provenance remain separate from current dependencies. Current
READ authorization support remains a current dependency. No old value is
substituted with a current value, and no old root is retained by Query evidence.

A first publication or cold reopen refuses `PredecessorBasisUnavailable`; a
producer can then choose full computation. Undeclared reads refuse
`UndeclaredPredecessorRead`, and conflicting predecessor coordinates refuse
`PredecessorMismatch`. HistoricalSelection and authored-restoration publications
refuse `PredecessorBasisUnavailableForOrigin`, naming the origin kind. Their
navigation precommit basis is not the producer's consumed input basis; follow-up
#94 owns that restoration contract. No durable format change is introduced.
The `host::primary_graph` audience exposes `RelationalPredecessorBasisDenial`
and `RelationalPredecessorBasisOrigin` for matching that original typed cause.

#### Typed producer domain denial

Every provider declares `type DenialValue` in one of three forms:

- a canonical readable scalar binding whose `Value` is exactly the operation's
  `Denial`;
- `WorthQueryProjectedProducerDenial<P>`, where `P` implements
  `WorthQueryProducerDenialProjection<Denial>` with a binding and a `project`
  from each denial onto that binding's value, for a denial wider than the value
  its callers read. `project` returns `None` for a denial without such a value;
- the explicit `WorthQueryUndeclaredProducerDenial`.

A declared binding needs a finite wire maximum. Installation refuses one
without it as `ProducerDenialValueUnbounded`.
A foreign denial type declares its binding through the entry's delegated
canonical contract, like a foreign invariant refusal.

When the handler returns `HandlerResult::DomainDenied(denial)`, the
`ProducerDomainDenied` demand denial carries the declared binding's canonical
bytes next to `domain_reason()`. The caller decodes them with
`decode_producer_denial::<Binding>()`. `producer_denial_bytes()` returns the
exact bytes, and diagnostic clones keep them. Decoding never returns an empty or
guessed value:

- another binding returns `TypeMismatch` before its decoder runs;
- an undeclared producer returns `Undeclared`;
- a projection that gives this denial no value returns `Absent`;
- other demand denials return `NotDomainDenied`;
- if the declared binding could not encode this denial, for example because it
  exceeds the wire maximum, the denial stays `ProducerDomainDenied` and decoding
  returns `EncodingRefused` with the cause.

The binding is the authority for the value. `domain_reason()` is diagnostic text
only.

Declare an artifact's output, locality, retention, succession, required posture,
producer family, dependencies, reuse rule and stopped outcome through
`ApplicationDerivedArtifact`. Current and restored output reuse checks original
source and installed meaning without contacting a producer for work that will
not run.

Operation-body requirement expressions retain their separately admitted Foundation
expression profile; an absent request ceiling adds no further work restriction.
Retry uses fresh
request authority on the same settled owner. Native consumed-fact movement
returns `Stale`; unrelated progress does not itself make settlement stale.
`read_historical` reads the original declared projection at the retained settlement
basis with fresh query authorization. Neither history nor close advances outputs.
For `ApplicationNoProjection`, `deliver_current` checks every settlement at one
authentic observation and returns that observation without issuing a query.

After a performed action's required tree settles, use
`settled.settlement().performed_source()` to borrow its original source
provenance. `source.receipt()` is the action's receipt before output production;
`root_receipt()` describes the root producer and may name a later publication.
The provenance holder retains the action's exact original read occurrence with
the settled tree. `source.root_fulfillment()` borrows the actual required root's
fulfillment of that action's output obligation. Its `source_commit()` names the
action occurrence, `output_commit()` names the root's qualified output
occurrence, and `posture()` preserves whether that output was performed, reused
or recovered. Reuse and recovery do not imply that the producer ran again for
this action. This fact describes the root; use ordinary settlement delivery to
check the whole tree at the current observation.

A tree started from a read returns `None` for
`performed_source()`, including when its producer performs a new publication.
The source receipt supplies descriptive evidence; inspecting it still requires
fresh current access, and it grants no navigation or mutation permission.

Use `WorthQueryOperationDeliveryPolicy::default()` for an attempt without an
additional work ceiling. A caller may explicitly choose a finite attempt as below.
This shape uses application-specific names;
the real compiled consumer counterpart is
[`fixed_delivery.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_delivery.rs).

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::WorthQueryOperationDeliveryPolicy;

let policy = WorthQueryOperationDeliveryPolicy::currentness(
    NonZeroUsize::new(64).unwrap(),
).query(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(32).unwrap());
let delivery = settled.read_current_with_policy(&fresh_request, policy);
// Keep settled on refusal; a later fresh request may use another finite policy.
```

For a realistic retained-output journey, settle the original action once, then
handle delivery separately. A currentness work refusal or query work refusal
leaves that same settled owner available for retry. A `Stale` result instead
means native consumed-source facts moved: read the original declared projection
with `settled.read_historical(&fresh_request)` when historical data is useful,
then close the owner. Historical reads still require fresh query authorization;
an installed-action mismatch on current delivery is not permission to execute a
replacement selector. `close` can refuse and return the owner with its remaining
cleanup, so retain it for lawful retry.

When no projected rows are needed, declare `ApplicationNoProjection` and use
`settled.deliver_current(&fresh_request, policy)`. This still settles and validates
the real required outputs. It returns an authentic observation, not query
permission or rows. The compiled public API example and negative counterpart are
on [`deliver_current`](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/current_delivery.rs).
`ApplicationOutputProjection` and `WorthQueryOutputProjectionInstallation` are
sealed composition contracts; applications choose the provided projection shape
rather than implementing another installation or authorization policy.

Retry delivery through the retained settled owner and its declared selector,
using a fresh authorized request and the installed delivery policy.

For several reads with the same installed query binding, declare
`ApplicationQueryBatchProjection<Read, N>` and implement
`ApplicationFixedBatchDelivery<Schema, Binding, N>`. Its pure `read_inputs`
selector borrows the original mutation input and returns an ordered array of
exactly `N` query inputs. `N` must be positive. The binding and cardinality are
installed canonical meaning; the concrete delivery implementation is also
checked at construction and current delivery. Callers cannot supply another
selector or replace the batch at read time.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
fn read_inputs(input: &SourceAdjustment) -> [OutputRead; 2] {
    [OutputRead { body_key: input.scope_key.clone() },
     OutputRead { body_key: format!("final:{}", input.scope_key) }]
}
```

`settled.read_batch_current::<N>(&fresh_request)` checks every actual settlement
and executes all items at that same authentic observation in one admitted phase.
The installed binding and native host profile supply physical custody allowances
and any explicitly chosen aggregate work ceiling. Absent work remains absent
through child reads and currentness checks. Each item still receives independent ordinary scope and
disclosure authorization. The aggregate allowance covers read work and retained
result/source custody; it does not claim to meter all authorization or
application-authored selector work.

Use `read_batch_current_with_policy::<N>` to narrow an attempt. The policy's
`batch(native_limits)` restricts aggregate work, retained bytes and per-item result
bytes; `query(results, work)` narrows each installed query allowance. A larger
aggregate request cannot enlarge installed policy. No item starts another phase
or renews the shared loan. A late refusal returns no partial collection and
releases staged results; the settled operation and cleanup remain yours.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let policy = WorthQueryOperationDeliveryPolicy::currentness(currentness_work)
    .batch(narrower_batch_limits);
match settled.read_batch_current_with_policy::<2>(&fresh_request, policy) {
    Ok(batch) => inspect_ordered_items(batch.items()),
    Err(denial) => retain_for_fresh_authorized_retry(denial),
}
```

The real two-output example is
[`fixed_batch.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_batch.rs).
It reads two distinguishable required-output values, refuses when the common
loan cannot fund both, then retries on the same owner with fresh authority.
Inspect each item's ordinary receipt and the collection receipt; the latter is
accounting evidence, not combined scope or mutation authority.
`read_batch_historical::<N>` uses the original declared selectors at the retained
settlement basis with fresh authorization. It does not claim currentness or
advance outputs. This batch shape is homogeneous: all items use one installed
query binding. Scalar query and explicit no-projection delivery remain separate
closed shapes.


`close` funds native recovery-record release and owner cleanup. A refusal returns
the settled owner with the exact remaining cleanup. Holding a real World caller
capability can lawfully refuse record release; dropping that capability and
supplying a fresh request permits retry. A closed result reports discharged
cleanup, rather than treating a failed release as success.

The fixed owning entrance selects conventional or capability admission from the
declared mutation binding and checks fresh request authority through the existing
native owner. It does not reinterpret a capability or workflow denial as ordinary
permission. Other lifecycle entrances retain their own native custody and support
boundaries.

When the owner reaches output traversal, one native `advance` can process the caller's dependency chain and other queued
required outputs in several waves. The required-set owner admits a wave only
against a basis where its consumed upstream outputs have committed. Compute
steps of independent members can overlap; completion, commits, and publications
follow canonical keys that order by producer identity, then by observed source
epoch. Every wave and nested computation
shares this call's request bounds. Match the typed stop rather than treating
work exhaustion, memory refusal, cancellation, deadline, panic, or a domain
denial as interchangeable. Earlier performed effects remain recoverable; no
member beyond the applied prefix is published. A queued stopped member does
not prevent unrelated required work from advancing.
An owner-completion failure does not refund compute already settled, even if
that failure is earlier than the compute stop. Treat the applied prefix and
canonical compute prefix as different boundaries.

Closing a continuation releases its interest, while other holders can keep an
output required. Drop releases this holder's interest and that row's own claims,
follows closed claims through at most two rows, charges no work, starts no
computation, and never scans the whole registry. It may hand on at most
one existing closed-leaf claim; remaining cleanup stays recorded for a later
funded call. Readers still see the effective successor of a deferred claim.
An explicitly accounted advancement's bill covers every wave, including a
failed call. Read physical observations separately from optional cost data.
Handle output progress as `Pending` or `Settled`, and propagate typed denials.

**Two independent members in one wave.** The planar geometry application has
two separate source rings, `diamond-left` and `diamond-right`. Each producer
reads its own ring and returns a managed computation plan. A third output,
`diamond-join`, consumes both outputs. The application declares and installs
this program in the consumer fixture's `concurrent_waves/application.rs`.

Before this excerpt, the fixture seeds the three rings, authenticates the
principal, opens demands for both roots and the dependent, and settles their
initial outputs. The root demands stay open. The excerpt changes both source
rings, advances the dependent once, and reads both published results. The
fixture verifies that both root computations finish before either root's
owner completion runs, and that each runs once: the roots share one wave.
The same calls work with serial backing or a host policy admitting parallel
workers; the host chooses placement, not the application.

`Program`, `PlanarRead`, `PlanarSourceAdjustment`, `PlanarOutputRead`, and
`length` are this application's declarations and value constructor. The
imported Query API comes from `worth-query-decl` and `worth-query-host`.
`application`, `principal`, `scope`, and the dependent's demand `join` come from
the setup above.

#### Advanced native required-output control

Use the [ordinary owning entry](#53-execute-through-the-program) for fixed actions.
The following fixture deliberately coordinates native output custody through
`advanced()`; it is not an ordinary lifecycle recipe.

<!-- compiled: checkpoint_recovery/required_chain/concurrent_waves/application.rs -->
```rust
        use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;

        let request = application.request(&principal, &scope);
        for (key, y, idempotency) in [
            ("diamond-left", 5, 0x9176_8200_u64),
            ("diamond-right", 54, 0x9176_8201_u64),
        ] {
            let selected = request
                .query(PlanarRead {
                    body_key: key.to_owned(),
                })
                .execute()
                .unwrap();
            let changed = request
                .advanced()
                .mutate(PlanarSourceAdjustment {
                    scope_key: key.to_owned(),
                    replacement_y: length(y),
                })
                .expect_source(selected.observed_sources()[0].clone())
                .idempotency(&idempotency)
                .execute_performed::<Program, program::ChainRoot>(
                    &application,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            assert!(matches!(
                changed,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ));
        }
        assert!(matches!(
            join.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        for (key, expected) in [("diamond-left", 6), ("diamond-right", 55)] {
            let output = request
                .query(PlanarOutputRead {
                    body_key: key.to_owned(),
                })
                .execute()
                .unwrap();
            assert_eq!(output.rows()[0].value, length(expected));
        }
```

---

#### Invalidation retention

`host::runtime::WorthQueryInvalidationResources::new()` (also `Default`) takes no
policy: no marking-work, preparation-byte, retained-byte or retained-position
ceiling. The subscription retains every delivered position, so no reader falls
outside a window or reconstructs to catch up. The value carries only the
receiver's differential verification controls.

Absence does not disable admission accounting or authorize memory allocation.
Preparation still checked-adds every original byte quote; retained tickets share
one ledger and stay charged while current, historical, prepared or consumed-output
owners hold them. Carried admissions retain the same original counters and
optional ceilings. Only the final holder's drop refunds its ticket. Request
cancellation and precommit atomicity are unchanged; real allocator refusals remain
owned by their allocating operation. This installation does not bound aggregate
process RSS or guarantee available RAM.

`RetentionCounterOverflow` reports an unrepresentable retained total with its
original requested/retained operands. It is distinct from finite
`RetentionCapacityExhausted` and does not trigger finite-capacity eviction.
Companion admission reports a finite retained ceiling through
`RetainedCompanionCapacityExhausted`.
Preparation counter overflow likewise remains distinct from a supplied ceiling.
Keep the returned original owner and its policy values on refusal; retry that
owner with a fresh authorized request.

#### Producer publication diagnostics

Keep the original operation on a terminal or refused advance and borrow its
returned `WorthQueryOperationAdvanceDenial::Outputs` denial. A native demand
cause, under `DemandExecution` or `Demand(Demand(..))`, exposes
`WorthQueryOutputDemandDenial` through `host::primary_graph`. Inspect
`publication_companion_stop()` and `producer_admission_stop()` separately; their
getters identify the original commit or producer-admission boundary. The
[host facade example](../workspaces/worth-query/crates/worth-query-host/src/facade.rs)
compiles this borrow-only path without consuming the operation or its cleanup.

`publication_companion_stop()` retains the original `CompanionPreflightStop` when a
producer commit maps a publication-companion resource refusal. Preparation exhaustion
carries `required` and `maximum` bytes; retained-companion exhaustion carries
`requested`, `retained` and `maximum` bytes. Work exhaustion preserves
`required` when supplied and `maximum`. Counter overflow stays an overflow
without a fabricated quote. Other denial paths return `None`; that absence does
not identify a budget. `producer_admission_stop()` instead retains the native
stop from producer preparation or source/output readmission, including topology,
selected source/position, a foreign cell or a registration change, as well as
original meter stops. The existing denial kind remains unchanged: those
structural stops still report `RetentionBudgetExceeded`. Inspect the native stop
to identify the cause.
`output_verification_capacity_stop()` separately retains an original
`host::primary_graph::VerificationCapacityStop` from output verification,
including a producer's consumed-edge check before publication. Its `Meter`
variant holds the original `CompanionPreflightStop`; `ScratchSizeOverflow` and
`ScratchAllocationRefused { bytes }` identify distinct native scratch failures.
The commit denial exposes the same getter specifically for its precommit
consumed-output check. The demand getter also exposes existing capacity evidence
from preparation/readmission and may overlap `producer_admission_stop()`; its
presence alone does not prove precommit ran. It is separate from native
transaction preparation. The existing kind and recovery posture remain unchanged.
If all three getters return `None`, the cause remains unknown; other denial
paths need not originate in these owners. These diagnostics
neither change kind or recovery posture nor authorize a retry, a capacity
increase, output completion or cleanup.

Consumed edges and restored marks in their original installed invalidation
ledger share retention custody of the same original fact allocation. Each holder still funds
its own native basis and metadata; new output projections and index storage
remain separately admitted. The full fact reservation and its weak-cache
funding end with their last external holder, even when historical commit
evidence still owns the original payload. Its empty lookup remains under the
payload's original native or codec ownership. A different installation receives
no credit from that ledger.
This changes neither currentness checks nor the configured physical capacity.

### 5.5 Own a complete no-source output tree

Use the ordinary demand builder when a read request needs the complete installed
output graph without an authored source mutation:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Concrete call bodies live in the neutral tree fixture.
let operation = request.demand(original_demand)
    .controls(WorthQueryOutputDemandControls::host_policy())
    .retain_program_outputs::<Program, FullTree>(&program);
```

Success returns `WorthQueryProgramOutputOperation<Schema, Program, FullTree>`.
Construction refusal owns the original demand, controls, branch, authentic program
runtime and any already issued observation. Retain it past the initiating scope
and call `retry(&fresh_request)`. A retry never replaces an issued basis;
`into_parts()` returns the actual original inputs for an explicit correction.
No principal, request scope or phase is retained.

Call `advance(&fresh_request)` explicitly. `Pending` and `Refused` return the same
original request custody; `Settled` owns all original output settlements and
complete discovery evidence; `Terminal` owns the genuine refusal and cleanup.
A root-only `start_in_program::<Program, FullTree>` demand does not traverse the
dependent tree and cannot substitute for this contract.

On the settled owner, `read_current(&fresh_request, typed_query_intent)` checks the
fresh selected program/root, every original settlement and original discovery
membership, then executes the actual typed query at that same native observation
in the same phase. `read_current_with_policy` narrows installed native limits;
`deliver_current` returns the checked observation without a query. Refusal leaves
the owner intact. These calls do not create another traversal or replace the
original source/demand.

Initial, settled, terminal and construction-refusal owners expose consuming
`close(&fresh_request)`. Success returns `WorthQueryClosedProgramOutputRequest`:
this proves discharge of the caller's read custody, never positive output,
source completion or platform shutdown. Close validates the original allocation
and branch and fresh cleanup authority. It does not require the original source
or root to remain current after supersession or program adoption.

Close refusal owns the whole original context and any already discharged native
prefix. Retain `WorthQueryProgramOutputCloseRefusal` and call `retry_close` with
fresh original-runtime/branch authority. That stage cannot advance released
interests. Actual producer cleanup, typed required successors and selected
unpublished checkpoints remain owned until their native obligations permit
disposal. An actual producer unpublished partial releases through its original
native recovery owner; refusal retains that same owner for fresh close retry.
When native release requires settlement, close may finish that original owner's
retained settlement before asking native release again. Neither release nor
close reruns the producer or publishes a current result.
Physical release never erases unresolved producer recovery. Dropping an owner is
abandonment, not a successful close. Performed-source required and
discovered aggregate completion rules, including fixed-leaf unavailable
completion, are unchanged.

#### Observe a member without progressing its producer

A checked computation that must publish complete Count alongside Pending volume
uses the same-reader [source-aligned availability](../workspaces/worth-query/crates/worth-query/docs/capabilities/source-aligned-output-availability.md)
route. Its candidate `reader.output_availability(&native_member, typed_demand)`
retains the actual source Query permission/session and native completion fact
through the original publication attempt. Native consumer qualification remains
open. Ordinary demand advancement remains driving; a denial or missing output
cannot be converted to publishable Pending. This route changes neither original
output custody nor the current delivery checks above.

#### Delivery after native close

When final publication must follow cleanup, call the settled parent's
`close_for_delivery(&fresh_cleanup_request)`. A Fixed parent returns
`WorthQueryClosedFixedDelivery`; a complete no-source parent returns
`WorthQueryClosedProgramOutputDelivery`. Each sealed carrier owns the original
complete settlement, original installed meaning and authentic runtime, beside
actual native close evidence. Its immutable `settlement()` includes the genuine
root correspondence even for `StableReused` with `root_receipt() == None`.
No original demand or controls survive no-source close. The closed read-tree
carrier retains its original readonly source basis as provenance.

Close validates original allocation/branch and fresh cleanup authority; stale
sources or changed selected programs do not prevent disposal. Fixed refusal
returns the same settled parent and typed cause. No-source refusal owns the
positive settlement and all remaining native custody, including any discharged
prefix; `WorthQueryProgramOutputDeliveryCloseRefusal::retry_close` is close-only.
It cannot deliver, advance, or convert back into a live tree.

For a complete no-source tree, an already-open live phase from the owning runtime
can dispose of native custody without an application request. Use
`operation.close_in_advancement(&phase)` for cleanup only, or
`settled.close_for_delivery_in_advancement(&phase)` to keep the original readonly
delivery. Their close-only refusals expose `retry_close_in_advancement(&phase)`.
The phase must belong to the original runtime and remain live; an identically
typed runtime's phase is refused. These calls open no nested advancement and
admit no payload read or new output demand. If the enclosing advancement stops,
retain the returned owner in your owned state before leaving its closure.
Closed delivery still retains readonly history pins until you drop it.

Only after actual native close succeeds, use the carrier's fresh `deliver_current`
or `read_current`. Fixed declared query/batch shapes retain their original
selectors and authorization; no-query delivery requires `ApplicationNoProjection`.
These calls reuse the live owner's exact complete output/discovery and installed
context validation, then read at that native observation in the same phase.
Any refusal borrows and preserves the already-closed carrier. It cannot rerun
source/producer work, create another demand or reopen cleanup. Plain `close`
remains available for cleanup-only intent.

Complete-tree currentness is as of that fresh native observation. Additional
application projection facts must be read again at the delivered observation;
old frozen views and descriptive commit equality provide no substitute.
Closed carriers and returned observations pin readonly lineage/history and the
runtime; they do not certify whole-application shutdown or own uncleared cleanup.
Drop them promptly after final publication, before whole-runtime close or OS join.
Use `receipt.descriptive_copy()` when you need original commit, intent and
terminal evidence without retaining the publication's physical history. The
copy carries no performed-change witness or full product basis. Ordinary
`Clone` still retains its original publication custody. Making a descriptive
copy does not retire the source or release another holder.
The neutral `application_program::closed_delivery` fixture exercises both owners,
including a real native source perturbation for Stable/no-receipt reuse. Its
rostered-successor court selects compatible meaning before capturing the original
operation, then checks genuine post-close current delivery. Later changed/removed
actions deny delivery while original native read interests still close. Restoring
a program does not restore an unavailable exact native output basis: matching
action meaning is necessary, and complete-tree currentness remains a separate
requirement. These checks do not replay a source or producer.
Any program adoption after settlement may make the original retained basis
terminally unavailable (`RetainedBasisUnavailable`). Matching action meaning
does not repair that basis; the application must run a fresh operation.

To present several closed carriers together, retain one observation and deliver
every carrier at it with `deliver_current_at(&request.at(&observation))`.
It exists on closed no-projection Fixed and closed no-source carriers. It runs
the same native gate as `deliver_current` at that caller's observation:
no-projection Fixed re-resolves its original installed action, no-source checks
its selected typed root, and both validate their complete original lineage under
the installed currentness ceiling. A closed Fixed carrier with a declared query
projection delivers only through its declared read (`read_current`,
`read_batch_current`); it has no common-observation delivery today. That is a
known gap, not permission to pair a unit gate with a replacement selector. The request's scope is checked fresh. A
foreign runtime, branch or observation is refused before any installed meaning is
read. Bare `request.at(&observation).require_current_program_output(...)` checks
lineage only: its output witness, comparable source facts and original consumed
outputs at that same native observation. Changed upstream facts refuse current
delivery even before regeneration; an unresolved upstream remains pending,
never current. Missing or empty comparable facts still require the original
own-publication decision. This check neither revalidates the selected action or
root nor replaces this gate. Success returns `()` because the caller already holds the
observation. Read every displayed fact at that same observation.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let observation = request.retain_current_read()?;
let pinned = request.at(&observation);
for carrier in &closed_no_projection_fixed {
    carrier.deliver_current_at(&pinned)?; // installed action + full lineage
}
for carrier in &closed_trees {
    carrier.deliver_current_at(&pinned)?; // selected typed root + full lineage
}
let membership = pinned.query(membership_read).execute()?; // facts at the same basis
```

For a retained record of a closed read tree's complete qualification, use
`closed_tree.qualify_at(&request.at(&observation))`. It checks the selected typed
root, every original root and dependent settlement, and original discovery at
that exact observation under a fresh request scope. The returned
`WorthQueryCompletedReadOutputs` retains readonly source and output history.
Its `source_commit()` identifies the original read admission, `root_output_commit()`
describes the root's own settlement, and `output_commit()` identifies the exact
common observation you supplied. Those commits can differ. A receiptless
`StableReused` root is legal; the fact does not assert producer reexecution.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let observation = request.retain_current_read()?;
let pinned = request.at(&observation);
let qualified = closed_tree.qualify_at(&pinned)?;
let rows = pinned.query(output_read).execute()?;
assert_eq!(qualified.output_commit(), observation.selected_commit());
```

Qualification is retrospective. A later edit does not change an issued fact;
qualifying at the later observation must check the original whole tree again.
A still-current root cannot stand in for stale dependents or discovery. Retaining
an older genuine observation lets you qualify at that historical basis explicitly.
The fact grants no current permission and cannot select, query or publish.
Qualification never advances a producer or reopens cleanup. Refusal preserves
the same closed carrier. Drop the carrier, facts and observations when finished
to release their readonly history before closing the runtime.

To qualify the original tree's own endpoint, call
`closed_tree.qualify_original(&request)`. It follows the genuine retained output
publications from the original read source, using World's exact before/after
references, then checks the selected typed root and complete tree there. It
never chooses an endpoint from the largest commit ordinal or a later current
head. If no output publication follows the source, the endpoint stays at that
source; retained or receiptless output reuse remains legal.

The returned `WorthQueryCompletedOriginalReadOutputs` retains the original
source, the genuine root read and the qualified endpoint separately. The root's
producer correspondence remains bound to its own read basis. Ordinary
`WorthQueryCompletedReadOutputs` cannot be converted into this original-endpoint
fact. Both types are retrospective evidence and grant no selection permission.

Later unrelated work can leave the original endpoint fully qualified while
moving the current head elsewhere. An unrelated publication between original
outputs cannot be used to connect their publication chain. If the reachable
prefix cannot qualify the complete tree, or no original read retains its exact
endpoint, qualification refuses and preserves the carrier. Qualification at an
explicitly supplied later observation can still be lawful; use `qualify_at` for
that distinct question. Neither call reruns producer work.

A refusal for one carrier leaves every carrier closed and unchanged. Only a
typed native `Superseded` (Fixed `Stale`, no-source `Currentness(Output(..))`
of kind `Superseded`) tells the application to run a fresh operation for that
carrier. `RetainedBasisUnavailable` and other causes keep their own posture.
The compiled shape is in Host `closed_delivery_contract`.

#### Original root output evidence

A settled parent's `settlement().root_settlement()` borrows its actual immutable
`WorthQueryApplicationOutputDemandSettlement<RootQuery>`. Inspect its existing
`outputs_of::<Contract>()` and typed `entity::<Role>()` correspondence to validate
which native entity fills the original root role. A genuinely reused root can
have `root_receipt() == None` and still carry that correspondence; no receipt,
new demand, key lookup or source-query identity needs to replace it.

This borrow is root-only retained evidence. It performs no fresh currentness
check and discharges no tree or cleanup custody. Keep the parent operation;
use its `read_current` or `deliver_current` for complete-tree validation, and
its `close` for lawful original read-request disposal. Projection at the
aggregate basis uses `settlement().observation()`, not `root_observation()`.
The executable `checkpoint_recovery::root_delivery` fixture exercises the
no-receipt case and closes the same original parent and runtime.

### 5.6 Retain preview custody across refusal

An application preview starts from an exact retained read observation:
`runtime.begin_preview(&phase, &observation, WorthQueryApplicationPreviewRequest::new(identity))`.
The request and session types are exported from
`worth_query_host::facade::application_installation` as
`WorthQueryApplicationPreviewRequest` and `WorthQueryApplicationPreviewSession`.

`preview.readmit(&phase, &runtime)` returns either the readmitted owner or
`(original_preview, WorthQueryApplicationPreviewReadmissionDenial)`. A foreign
phase/runtime, stopped selection or stale source leaves that original preview
available for explicit discard. Each preview retains one authentic installed
runtime clone until its native owner and source custody are discharged; closing
the installed runtime while a preview retains it returns `RuntimeKeepaliveRetained`.
Readmission never replaces its selected source or automatically discards it. A new request can retry the same owner when the
cause permits; stale source does not become fresh by retrying.

`readmitted.promote()` finalizes the original admitted native session and execution
proof. Failure returns `(same_readmitted_owner, BridgeSpeculationError)`.
Promotion is the preview evidence transition; it does not reselect current Query
source, save source, settle required outputs, or certify current publication.
An admitted source action may legitimately commit between readmission and preview
promotion; that action's authentic source and output contracts still govern its
publication. Successful promotion consumes the original native preview once.

Both preview phases expose `discard() -> Result<(), (Self, BridgeSpeculationError)>`.
Discard uses the original native owner and can release a stale preview without
selecting a new source. On refusal, retain the returned owner and its original
application context; retry lawful explicit discard on that owner. Dropping either
phase is abandonment, and does not post a successful discard record. Preview
liveness observation is not proof of cleanup or promotion.

### Optional request-memory policy

Use the [process execution service](#one-process-execution-service) for application
execution. When a caller needs memory accounting, select an accounted request
policy and use the following reservation APIs.

Within that contract, use `ExecutionBudget::new(workers, None, work_ceiling)`
for no request-memory ceiling, or pass `Some(bytes)` for an explicitly finite caller policy.
`SerialMemoryBudget::new(None)` has the same meaning for a serial owner. Supply
the worker count explicitly and handle actual allocation failures separately.

Each explicit accounted reservation charges the same serial or lease ledger, including its
ancestors and process. An absent child ceiling does not escape a finite ancestor.
`available_memory_bytes()` returns the minimum remaining supplied ceiling, or
`None` when that lineage has no ceiling. Actual held-byte getters remain concrete
counters. A finite refusal is `MemoryLimitDenial::Capacity`; an unrepresentable
live charge is `MemoryLimitDenial::CounterOverflow`. Refusal preserves the old
reservation, and final-holder release refunds only its original charge.
Cancellation and deadlines retain the original request and lineage.

An explicitly accounted result without a genuine prior heap maximum uses the original
`request.run_measured_result(ceiling, known_bytes, &mut reservation, kernel)`.
The callback borrows its `ExecutionResultMemory` view to reserve source-owned
clone/growth backing on that same ticket before allocation. The view protects
known input/framework bytes. For an opaque callback, the framework measures and
admits the actual returned value or domain error before retention or publication;
that does not claim pre-allocation admission. The result/error remains charged on
the caller's original reservation until released or transferred. The returned
`ScopeOutcome` keeps the genuine admission/callback stop and original report.
Known finite Scan/Map result declarations retain their existing checks.

### Typed refusal from installed invariants

For proposed/committed access, prepared scope, retained snapshot readers and the
shared diagnostic counts, see [Invariants and read observations](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md).

An installed `WorthQueryApplicationInvariantRule` declares `type Refusal` and
`type RefusalBinding`. The binding is the application's existing readable scalar
value declaration for that type, with the canonical JSON contract and a finite
`ValueDecodeBounds` wire maximum. Installation refuses a missing canonical
contract or an unbounded wire representation. There is no separate refusal codec.
A cause-free rule explicitly declares `Refusal = ()` and uses
`WorthQueryEmptyInvariantRefusal` as its binding.

Entry-local value declarations can derive
`worth_query_decl::facade::application_schema::WorthValue` with
`#[worth_value(crate = "worth_query_decl::facade::application_schema")]`.
That facade exports the existing canonical shape and bounded seed vocabulary,
including delegates for values constructed by a separate domain owner; the
entry needs no lower platform dependency.

For a foreign domain type, an entry-owned `ValueSeedDelegate<Value = DomainType>`
can be the root grammar as well as a field grammar. Its binding declares
`ApplicationCanonicalValueContract::delegated::<Delegate>(bounds)` and uses
`encode_delegated_canonical_value` and `decode_delegated_canonical_value` (or the
admitted decoder). These call the same canonical writer, bounded seeds and
canonical comparison as ordinary values. They borrow the original value for
encoding; no domain-value clone, alternate codec or platform trait on the pure
domain type is needed.

Evaluation returns `WorthQueryApplicationInvariantVerdict::Pass` or
`Violation(refusal)`. Preparation and evaluation errors use
`WorthQueryApplicationInvariantError<Self::Refusal>`. Return `Cause(value)` when
application arithmetic or another domain prerequisite prevents evaluation. The
same `RefusalBinding` encodes both outcomes; there is no second cause codec or
message that owns the application's meaning. SDK access, interruption and
allocation failures have named error variants. A cause that exceeds its declared
wire maximum remains an encoding failure rather than a domain violation. Cancellation
and deadlines retain the original request's control outcome. The adapter admits the
maximum retained refusal bytes against the original evaluation meter before encoding,
including ordinary serial rule hooks.

Under a multiworker policy, rules at the same execution point may evaluate
concurrently and finish in any order. The original publication result carries
the first violation in installation binding order, whatever the worker count or
completion order. Its `WorthQueryApplicationCommitDenial` exposes
`decode_invariant_refusal::<YourRefusalBinding>()`; a mismatched binding returns
`WorthQueryInvariantRefusalDecodeDenial::TypeMismatch` before decoding. The
unchanged canonical bytes are also available through `invariant_refusal_bytes()`.
Relational retains that same payload in the transaction conflict and diagnostic
history. Neither reading the denial nor decoding it reruns the rejected candidate.

At installation, seed refusals use `WorthQueryInMemoryApplicationDenial::Publication`;
conditional publication uses `ConditionalPublication` and
`WorthQueryConditionalRuntimeInstallationDenialKind::PrimaryGraphPublication`.
Inspect the enclosed graph denial's `custom_invariant_denial()`. Both installation and commit retain
`WorthQueryCustomInvariantDenial::{Violation, Failure}` with the original invariant
identity. `Failure` also retains its phase, failure kind and typed SDK cause.
`decode_refusal::<Binding>()` decodes the original application value in either
case; commit's `decode_invariant_refusal::<Binding>()` and byte accessor do the
same. A closed application enum can distinguish, for example, a membership
violation from `Capacity { field, maximum, observed }` without parsing a message.

Checkpoint recovery callers use `primary_graph::{RelationalRecoveryStop,
RecoveryFailureClass, DurabilityError}`. The installation denial's
`native_recovery_cause()` retains every original Native stop, including
`Durability(error)` with `CorruptCheckpoint` or `UnsupportedLegacySemantics`.

Native recovery checkpoints retain symbol meanings from installation configuration,
published canonical envelopes, retained roots and the historical mirror, including
aspect/field revision keys and structural fingerprint families. Capture and
readmission use the storage owner's same symbol-closure traversal. Client
keys interned only while preparing an unpublished attempt do not change the
checkpoint. The runtime's live symbol interner can still resolve those pending
keys; recovery persists the selected retained truth.

The [temporal integrity rule](../workspaces/worth-query/crates/worth-query-testkit-certification/src/certification/program_example_authority/product_workflow_support/integrity.rs)
returns `TemporalIntegrityRefusal::InvalidState` with the candidate revision and
lifecycle. The [installed refusal courts](../workspaces/worth-query/crates/worth-query-court-execution-package-archive/tests/worth_query_court_execution_package_archive/domain_computation/primary_graph/tests/application_attempt/typed_invariant_refusal/mod.rs)
exercise original-caller decoding, unchanged native checkpoints, cancellation,
installed order, size refusal, decode mismatch, and reproducible payload bytes.

### Optional invariant work and original interruption

Within an explicitly accounted execution policy, the work ceiling can be absent:
`ExecutionBudget::new(workers, charged_memory_bytes, None)`. Serial and leased
requests retain that same optional ceiling through accounted child scopes; a child cannot
widen a finite parent. Charged counters and typed overflow remain. Signal's
`SignalConditionalEvaluationBudget::development()` has no attempt-visit ceiling;
its retained-slot and byte limits still apply. Explicit `Some` limits remain
caller choices. Use `for_workers` and checkpoint polling when only placement and
request controls are needed. Supply invariant work policies separately.

Retained source-only delivery uses `owner.advance(&fresh_request, None)` for
no additional history work ceiling. `Some(NonZeroUsize)` explicitly bounds an
attempt. Both use the same original World history owner, physical protection and
request interruption; refusal returns the same operation for a lawful fresh retry.

`RelationalInvariantWorkBudget::new(None)` declares no local operation-count
ceiling; `Some(NonZeroU64)` retains an explicit caller-selected ceiling. Installation,
canonical identity and package archives preserve that distinction. Absence never
removes receipt authority, physical allocation admission or graph validity.

An application invariant's `Context::check_live()` and `ScopePlanner::check_live()`
are pure observers of the original serial or leased request. They return the
original native `CustomInvariantInterruption::{Cancelled, DeadlineElapsed}` through
`worth_query_host::facade::application_invariants`. A numerical callback may borrow
`|| context.check_live().is_err()` without charging work or creating a request. The
engine checks the same original interruption before accepting an invariant result.

Ordinary Fixed/discovered advancement returns genuine native progress or the same
owned Pending state. The advanced `settle` convenience's per-call advancement
quantum limits batching only: it returns the original Pending owner, never treats
quantum exhaustion as a failed output, and does not limit total work or depth.

## 6. Author, publish, and run workflows

Guarded no-output effects use the original descriptor and issued requirement:
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
A refusal from this effect-free `retain_operation` returns the bound owner.
Use its consuming `into_parts()` to recover the original description and issued
requirement outside the borrowed workflow runtime, then correct the declared
action/runtime and rebind that same description and requirement. No effect has
run and no slot has been consumed at this construction boundary.
Every `advance(&fresh_request)` rechecks current native authority. Effect
completion retains native cleanup; workflow acceptance returns that completed
owner on refusal and keeps it alongside successful native progression.
Assessment has a separate retained demand: `into_assessment_demand(..)?`
`.controls(controls).retain_operation(program)?`, then `advance(&fresh_request)`.
Observer close, producer cleanup, evidence acceptance and approval are distinct.

A workflow is a branch-local, versioned graph of governed steps. The host
installs a **vocabulary** once: which operations, assessments, conditions,
and approvals a workflow may use, and which capabilities control it. After
that, definitions are *data*: authored, validated, published on a branch,
discovered, started, and advanced at run time.

For explicitly driven discovered programs, retain the completed aggregate and use
`request.at(&observation).require_current_discovered_program_outputs(&settlement,
maximum_work)` to validate that exact original required set. The check includes
root and dependent discovery consumption and every retained output settlement.
Changing membership refuses delivery even when every surviving root remains
current. An originally empty set still carries its original negative facts;
retiring a native root does not erase those facts or invent an output settlement.
Refusal preserves the borrowed aggregate and its historical observation. This
native guard does not run a projection. Ordinary discovered actions use
`retain_operation::<Program, Action>`, then fresh-request `advance`; their settled
owner composes this complete guard and the installed declared query or batch at
one authentic observation and phase. See [owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions).


Every workflow action is a governed mutation. There is no background
scheduler, and no timer or callback advances an instance: **the caller pumps
progress**, one request per step. An external effect completes through an
authenticated inbound occurrence (`inbound_occurrences().receive(..)` on the
installed runtime), which completes the effect's owner. After a caller accepts
the owner's receipt (`accept_operation_from_owner`), an `AwaitInbound` node
observes that owner result on the next advance. Callback bytes never resume
an instance. See
[Workflows: Caller-pumped progression](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#caller-pumped-progression)
and [Waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion).

### 6.1 The spec marker

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-domain/src/schema/workflow.rs
pub struct ApprovedBusinessPaymentWorkflow;

impl ApplicationWorkflowSpec for ApprovedBusinessPaymentWorkflow {
    type Schema = BankSchema;

    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.bank.approved-business-payment-workflow.v1");
}
```

### 6.2 Install the workflow vocabulary once

The vocabulary is installed against the running program, then retained on
the program runtime. The result is a
`WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>`, bound to one
spec. `retain_workflow_spec` consumes the program runtime, and the workflow
runtime dereferences back to it.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Direct Query vocabulary setup; Bank retains refusal custody in identity_runtime/workflow_binding.rs.
use worth_query_host::facade::domain::WorthQueryApplicationWorkflowSpecInstallation;

let workflow = WorthQueryApplicationWorkflowSpecInstallation::<
    BankSchema,
    ApprovedBusinessPaymentWorkflow,
    BankApplication,
>::begin(
    runtime.runtime().installed_schema(),
    runtime.installed_program(),
    payment_workflow_resources(),          // WorthQueryApplicationWorkflowResourceCeiling
)
.operation::<ApprovedBusinessPaymentAuthoringBinding>()?
.operation::<ApprovePaymentMutationBinding>()?
.assessment::<PaymentDetailQueryBinding>()?
.condition_operand::<PaymentAmountQueryBinding>()?
.approval::<ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentApprovalOperation, ApprovePayment>()?
.authoring_capability::<ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringOperation, ApprovePayment>()?
.instance_start_capability::<ApprovedBusinessPaymentInstanceStart, ApprovedBusinessPaymentInstanceStartOperation, ApprovePayment>()?
.advance_capability::<ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentAdvanceOperation, ApprovePayment>()?
.finish()?;

let workflow_runtime = runtime.retain_workflow_spec(workflow, approval_authentication.signing_owner())?;
```

- `.operation::<Binding>()`, `.assessment::<Binding>()`, and
  `.condition_operand::<Binding>()` admit the actions and reads a definition
  may use.
- `approval` and the three `*_capability` calls each take three types:
  `<Capability, Operation, Input>`. `Capability` is the capability marker the
  caller must hold. `Operation` is the control operation it gates, declared
  in the schema with `worth_query_operation!`. `Input` is that operation's
  input value type. Bank's four control operations share one input type,
  `ApprovePayment`.
- The three **control capabilities** gate every control step. The authoring
  capability publishes and retires definitions. The instance-start capability
  starts, cancels, migrates, and continues instances on a fork. The advance
  capability advances and navigates back.
- The **signing owner** makes approvals verifiable. Install it with
  `admission::authentication_event::install_authentication_event_owner::<Schema, Clock, _, _>(schema, clock_source, verifier, policy, capacity)`
  and pass its `signing_owner()` here. Bank does this in
  `bank-server/src/approval_authentication.rs`, with a 60-second single-use
  event policy.
- A host supporting several programs calls `support_workflow_spec` on the
  returned workflow runtime for each extra program.

Bind this vocabulary, and support any extra program vocabularies, before cloning
runtime handles or retaining operations. `retain_workflow_spec` returns
`WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>` on any refusal.
Inspect `denial()` and consume `into_parts()` to recover the original runtime,
installed spec, signing owner and denial, in that order. Release the real
keepalives before retrying a `RuntimeKeepaliveRetained` refusal. No refusal
silently drops those inputs or requires a replacement runtime or signing owner.
`support_workflow_spec(&mut self, installed)` returns `Err((installed, denial))`;
the caller still owns the borrowed workflow runtime and the returned original
spec. Keep signing custody with its original owner; log the denial description.

Bank retains its surrounding application inputs too:
`BankIdentityRuntimeBuildError::WorkflowRuntimeBinding { binding, denial }`
returns an opaque `BankIdentityRuntimeWorkflowBinding` alongside the denial.
Keep that stage. When correcting vocabulary, `replace_workflow_spec` returns
the original installed spec to the caller. After releasing actual dependent
operations, `(*binding).bind()` retries the same authentic runtime, spec,
signing owner and Bank companions. A repeated refusal returns the complete
stage again; keeping only its denial would discard retry custody.

**Control bindings.** Each control step (authoring, instance start, advance,
approval) is a mutation intent whose binding sets `WORKFLOW_CONTROL = true`.
The workflow kernel authorizes and records these steps itself. No handler may
serve one, and installing one is refused. `worth_query_mutation_binding!` does
not set that constant, so a control binding is written by hand. Bank writes
its four with one local macro. This is the part that differs from an ordinary
binding:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// The Bank control-binding shape, using current operation-access authoring:
worth_query_operation!(pub $operation for BankSchema, input ApprovePaymentInputBinding;
    access { read_fields: [PaymentIdentityField]; }
);

pub struct $binding;

impl ApplicationMutationBinding<BankSchema> for $binding {
    type Input = ApprovePayment;
    type Operation = $operation;
    type Decision = ();                       // no handler decides a control step
    // ... the same associated types as any mutation binding (Â§5.2) ...
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;
    // ... identities, scope_field, and principal_binding ...
}

impl ApplicationCapabilityMutationBinding<BankSchema> for $binding {
    type Capability = $capability;            // the marker passed to *_capability above
}

impl ApplicationMutationIntent<BankSchema> for $intent { /* input, scope_binding */ }
impl ApplicationCapabilityRequest<BankSchema, $capability> for ApprovePayment {
    /* project the request onto the capability's entity and context */
}
```

Install its access with `schema.declared_operation::<Operation>(definition)`
and the existing explicit policy. Keep the workflow-control binding's native
authorization rules. The macro is invoked once for each control step, for example
`ApprovedBusinessPaymentAuthoringOperation`, `â€¦AuthoringBinding`,
`â€¦AuthoringIntent`, and `ApprovedBusinessPaymentAuthoring`. The whole file is
244 lines.

#### Create and enroll through one retained action

When the subject must be born with its workflow, declare
`creation.enroll_in_workflow::<Spec, Binding, CreatedRole>()` on the actual
creation body and install `.birth_operation::<Binding, CreatedRole>()` in the
vocabulary. The creator has its own container-scoped capability; it does not use
the unborn subject's current-operation authority. Publish a real definition and
prepare `RequiredWorkflowBirth` through native workflow control.

Ordinary creation and retry then use the original captured descriptor:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let original = workflow.request(&principal, &scope)
    .on_branch(required.branch()).mutate(intent)
    .without_source().idempotency_owned(logical_key)?;
let operation = original
    .for_workflow_birth::<Spec, CreatedRole>(&workflow, &required)?
    .retain_operation::<Program, CreateAction>(workflow.program_runtime())?;
// Supply fresh authority to advance; preserve Progress/Refused or close completion custody.
```

Use SourceCompletion or RetainedCommit delivery for mutation retry. Binding
checks the frozen original input and key;
construction failures return original custody without consuming birth authority.
Advance the retained operation under fresh authorization for atomic
creation/enrollment and recovery after interruption. See
[workflow-enrolled creations](../workspaces/worth-query/crates/worth-query/docs/authoring/workflow-enrolled-creations.md).

### 6.3 Author a definition

`ApplicationWorkflowDefinitionBuilder<Spec>` creates typed nodes, then
connects them. This is Bank's approved-payment workflow. The excerpt keeps
every node, connection, and limit of
`approved_business_payment_definition()`. It puts each `.control(..)` call on
one line, adds the `//` annotations, and omits the function signature and the
denial enum:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/workflows.rs (abridged)
use bank_domain::queries::{PaymentAmountQuery, PaymentDetailQuery};
use bank_domain::schema::{
    approved_payment_inbound_binding, ApprovePaymentMutationBinding,
    ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentWorkflow, ApprovedPaymentSettlementEffect,
};
use std::time::Duration;
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentLimits,
    ApplicationWorkflowConditionOperands, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowInboundWait,
    ApplicationWorkflowValidationDenial, ValidatedWorkflowDefinition,
};

/// The operand the approval limit reads: the payment's amount in cents.
pub const APPROVAL_LIMIT_OPERAND: &str = "amount_cents";

/// Workflow approval covers payments up to $150.00; a larger payment is
/// rejected before any approver is asked.
pub const APPROVAL_LIMIT: &str = "amount_cents <= 15000";

let mut builder = ApplicationWorkflowDefinitionBuilder::<ApprovedBusinessPaymentWorkflow>::new(
    "approved-business-payment",
    ApplicationWorkflowDefinitionLimits::new(
        10,                                                  // nodes
        21,                                                  // connections
        2,                                                   // effects
        ApplicationWorkflowComponentLimits::new(8, 2, 16, 32, 32).unwrap(),
        16 * 1_024,                                          // canonical bytes
    )
    .and_then(|limits| limits.with_total_deadline(Duration::from_secs(7 * 24 * 60 * 60)))
    .expect("approved-payment limits and deadline are valid"),
)
.expect("approved-payment identity is valid");

// Nodes. Each call returns a typed node reference.
let propose =
    builder.operation::<ApprovedBusinessPaymentAuthoringOperation>("propose", false)?;
let payment = builder.assessment::<PaymentDetailQuery>("review/payment")?;
let independent = builder.assessment::<PaymentDetailQuery>("review/independent")?;
let evidence = builder.evidence_join(
    "review/evidence",
    ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
)?;
let limit = builder.condition(
    "approval/limit",
    APPROVAL_LIMIT,
    ApplicationWorkflowConditionOperands::new()
        .query::<PaymentAmountQuery>(APPROVAL_LIMIT_OPERAND),
)?;
let approval = builder.approval::<ApprovedBusinessPaymentApproval>("approval")?;
let apply = builder.operation_binding::<ApprovePaymentMutationBinding>("apply")?;
let await_completion = builder.await_inbound::<ApprovedPaymentSettlementEffect>(
    "await-inbound",
    &apply,
    approved_payment_inbound_binding(),
    ApplicationWorkflowInboundWait::UntilInstanceDeadline,
)?;
let completed = builder.terminal("completed")?;
let rejected = builder.terminal("rejected")?;

// Control: where each outcome goes.
builder
    .start(&propose)
    .control(&propose, ApplicationWorkflowControlOutcome::Completed, &payment)
    .control(&payment, ApplicationWorkflowControlOutcome::Completed, &independent)
    .control(&independent, ApplicationWorkflowControlOutcome::Completed, &evidence)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceSatisfied, &limit)
    .control(&limit, ApplicationWorkflowControlOutcome::ConditionSatisfied, &approval)
    .control(&limit, ApplicationWorkflowControlOutcome::ConditionUnsatisfied, &rejected)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceFailed, &rejected)
    .control(&approval, ApplicationWorkflowControlOutcome::Approved, &apply)
    .control(&approval, ApplicationWorkflowControlOutcome::Rejected, &rejected)
    .control(&apply, ApplicationWorkflowControlOutcome::Completed, &await_completion)
    .control(&await_completion, ApplicationWorkflowControlOutcome::Completed, &completed)
    // Data: what each step receives.
    .proposal_for_assessment(&propose, &payment)
    .proposal_for_assessment(&propose, &independent)
    .assessment_evidence(&payment, &evidence)
    .assessment_evidence(&independent, &evidence)
    .condition_subject(&propose, &limit)
    .proposal_for_approval(&propose, &approval)
    .joined_evidence(&evidence, &approval)
    .approval_authority(&approval, &apply)
    .operation_input(&propose, &apply);

Ok(builder.finish()?.validate()?)
```

`apply` is a guarded effect whose external settlement goes into owner custody;
`await_completion` waits for that owner's inbound completion. An
`AwaitInbound` node requires an instance deadline, which is why the limits
carry `with_total_deadline` (without one, validation refuses with
`MissingInboundDeadline`).

**Node authoring.** The [source-derived node inventory](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#source-derived-vocabulary) owns the complete `ApplicationWorkflowNodeKind` list. The following table explains builder usage:

| Builder method | Node | Notes |
|---|---|---|
| `operation::<Op>(id, false)`, or `operation_binding::<Binding>(id)` for an unguarded binding | Operation: proposal slot | The caller records proposed input with `prepare_workflow_proposal`. No handler runs and no business state changes. |
| `operation::<Op>(id, true)`, or `operation_binding::<Binding>(id)` for a `workflow_authority required` binding | Operation: guarded effect | An advance issues authority for this occurrence (`AwaitingOperation`); the caller executes the business mutation through `for_workflow_operation`. `operation_binding` takes the authority flag from the binding. |
| `assessment::<Query>(id)`, `assessment_for`, `assessment_when_related_relation_present` | Assessment | A query whose settled result becomes evidence |
| `await_inbound::<Effect>(id, &origin, inbound, wait)` | AwaitInbound | Waits for the inbound completion of the named preceding operation's external effect. `wait` is `ApplicationWorkflowInboundWait::UntilInstanceDeadline`, so the definition needs a total deadline. |
| `condition(id, source, operands)` | Condition | A Bool expression over named, typed query results |
| `approval::<Capability>(id)` | Approval | A signed human decision under a capability |
| `evidence_join(id, policy)` | Evidence join | `AllRequiredPassing` or `AllRequiredCompleted` |
| `terminal(id)` | Terminal | An end state |

There is **no** query, read, branch, or retry node. A read that feeds a
decision is an assessment or a condition. Branching is routing different
`ApplicationWorkflowControlOutcome`s. A retry is a connection:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/main.rs
let revise = ApplicationWorkflowRetry::new(
    ApplicationWorkflowControlOutcome::Rejected,
    "revise the proposal",
    2,                                   // maximum attempts
)
.expect("the revision bound is valid");
// ...
workflow.retry(&approval, revise, &propose);
```

A retry source must also route `RetryExhausted`.

**Connections** on the definition builder: `start`, `control`, `retry`, and
the data flows `proposal_for_assessment`, `condition_subject`,
`proposal_for_approval`, `assessment_evidence`, `joined_evidence`,
`approval_authority`, `operation_input`. Endpoints are typed: an edge that
makes no sense does not compile.

**Reusable components.** `ApplicationWorkflowComponentBuilder<Spec>` builds a
fragment with named `input_port` and `output_port`s. Place it with
`builder.expand_component("checks", &component)?`, then connect to
`expanded.input(&port)?` and `expanded.output(&port)?`. Expanded node paths
are prefixed with the occurrence name (`checks/structural`). See
`examples/authored_workflow/review.rs`.

**Three front ends, one builder.** All of these produce the same
`AuthoredWorkflowDefinition<Spec>`, and all must still be validated:

1. The typed builder shown above.
2. The `worth_query_workflow!` macro. It is a thin block around the same
   builder, applies `?` to `new`, and returns the unvalidated `finish()`
   result:

   <!-- illustrative -->
   ```rust,ignore
   // Illustrative: not compiled.
   let authored = worth_query_workflow! {
       spec: ReviewedChange;
       identity: "reviewed-change";
       limits: limits();
       build: |builder| {
           let propose = builder.operation::<ProposeChange>("propose", false)?;
           // ... nodes and connections, exactly as with the builder ...
       }
   }?;
   let validated = authored.validate()?;
   ```

3. `ApplicationWorkflowCommandAdapter::author::<Spec>(identity, limits, commands)`,
   which lowers a sequence of `ApplicationWorkflowAuthoringCommand`s
   (`Node`, `Start`, `Connection`). Use it when a UI, an API, or an AI agent
   authors a workflow as data.

**Validation.** `validate()` returns `ValidatedWorkflowDefinition<Spec>` or an
`ApplicationWorkflowValidationDenial`. Kinds include `MissingStart`,
`UnreachableNode`, `MissingControlOutcome`, `AmbiguousControlOutcome`,
`ControlCycle`, `MissingTerminal`, `MissingWorkflowAuthority`,
`UnexpectedWorkflowAuthority`, `MissingOperationInput`, `IncompleteApproval`, and
`MissingInboundDeadline`. `MissingWorkflowAuthority` means a guarded operation
has the wrong number of `approval_authority` edges for its installed
requirement: more than one, or not exactly one when the requirement asks for
an approval (including the legacy default). `UnexpectedWorkflowAuthority`
means an unguarded operation carries an approval edge or an execution
requirement. See [Who may run a guarded operation](#65-who-may-run-a-guarded-operation).
`MissingOperationInput` means a guarded operation has zero or more than one
`operation_input` source. It needs exactly one compatible source, including when
its installed requirement needs no approval. Unguarded proposal slots have no
input-cardinality requirement. Unsupported stored definitions are refused before
executable readmission; their original bytes are preserved.
A validated definition carries no branch, publication status, or authority to
start anything.

### 6.4 Fixed and user-authored workflows are the same thing

There is no separate "fixed workflow" construct. Bank's approved-payment
workflow is written in Rust and ships with the application, but it is
published at run time through the same kernel as a workflow a user authors
in a UI. What is static is the spec marker, the installed vocabulary, and the
control bindings. What is data is every definition, instance, proposal,
piece of evidence, and approval.

### 6.5 Who may run a guarded operation

An Operation node is one of two things, decided by its binding:

- **Proposal slot** (unguarded binding): the caller records proposed input with
  `prepare_workflow_proposal`. This runs no handler and writes no business state.
- **Guarded effect** (`workflow_authority required`): an advance issues authority for the
  occurrence; the caller then executes the business operation through `for_workflow_operation`.

A workflow-bound business operation passes up to three checks. They answer
different questions, are declared in different places, and none grants
another.

| Check | Question it answers | Declared by | Checked when |
|---|---|---|---|
| Operation authorization | May this principal perform this business operation on this scope? | The operation's capability or policy binding | Ordinary mutation admission, on every execution, including workflow-bound ones |
| Workflow occurrence authority | Is this exactly the operation, input, and branch the instance is waiting for now, and not already spent? | `workflow_authority required` on the mutation binding | Issued by an advance that reaches a guarded Operation node (`AwaitingOperation`); bound by `for_workflow_operation`; rechecked at commit |
| Required approval evidence | Must a current signed Approve decision also exist? | The binding's `workflow_requirements` plus any `approval_authority` edge authored in the definition | Resolved by the advance before authority is issued; the approver's grant is rechecked at commit |

Decision procedure:

1. An unguarded business mutation needs check 1 only.
2. A guarded mutation (`workflow_authority required`) needs checks 1 and 2, always.
3. It also needs check 3 when the installed requirement asks for an approval,
   or when the definition wires an `approval_authority` edge to the operation.
4. Approving needs the approval capability plus a fresh signature. Advancing
   needs the advance capability. Neither satisfies check 1. Permission to pump
   an instance is not permission to execute the business operations in it.
5. An approval is evidence, not permission. Workflow authority is a binding to
   one occurrence, not permission.

The installed requirement (`workflow_requirements` inside
`worth_query_mutation_binding!`):

| Declared | Meaning |
|---|---|
| Guarded requirement omitted | Compile refusal; manual bindings must implement the requirement method explicitly. |
| `ApplicationWorkflowExecutionRequirements::not_applicable()` | Explicit non-workflow applicability; refused for a guarded binding. |
| `ApplicationWorkflowExecutionRequirements::occurrence_only()` | No additional evidence. Zero or one approval edge; an authored edge is still enforced. |
| `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` | An approval by `Cap` is required, and an edited definition cannot remove it. |

Guarded bindings require an explicit declaration. Omission does not select a
legacy policy. Manual implementations return the direct requirement type.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/retention_entry/occurrence_write.rs (abridged)
worth_query_decl::facade::worth_query_mutation_binding!(
    pub OccurrenceSetRetentionBinding for SetRetentionInput, schema DocumentRetentionSchema,
    // ... identity, input, operation, result, idempotency, decision, handler ...
    workflow_authority required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only(),
    // ... outputs, principal, scope ...
);
```

The sequence, from proposal to settlement:

1. **Propose.** At an unguarded Operation node, a proposal records the
   operation, input type, an input-identity digest, and the node path. No
   handler runs and no business state changes. The input value itself is not
   retained: it is the caller's domain data.
2. **Assess, branch, join.** Each assessment and condition takes its subject
   from the proposal through `proposal_for_assessment` or
   `condition_subject`: the scope resource the proposal covers, or a related
   or context entity in its coverage. Its queries read that subject's current
   state on the branch. The proposed input value is not retained, so no
   assessment or condition can read it.
3. **Approve (optional).** A signed Approve or Reject decision about the
   proposal. Evidence, not permission.
4. **Authorize.** An advance at a guarded Operation node returns
   `AwaitingOperation` carrying single-use authority for that occurrence.
   Nothing is committed. The authority lives in memory; if it is lost, advance
   again.
5. **Retain and advance the approved operation.** Capture the original input
   fallibly with `idempotency_owned`, bind that description with
   `for_workflow_operation`, then select the installed declared action with
   `retain_operation`. Capture refusal returns the original builder and key
   before the issued slot transfers. Advance the retained owner with a fresh
   request; it delegates native admission, proposal-input matching and the
   atomic business-write/node-settlement funnel. Keep the returned owner on
   refusal or pending publication; do not rebuild after an uncertain effect.
   See [the owning workflow entry](#6-author-publish-and-run-workflows) for the call shape.
6. **External effects** settle through owner custody and an `AwaitInbound` node.
7. **Advance again** to move to the successor or terminal.

`AwaitingProposal(RequiredWorkflowProposal)` names the node, operation and input
type. Publish the proposal through fresh `prepare_workflow_proposal` admission;
the need grants no authority and waiting records no transition.

Full reference: [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations).
The runnable unsigned, occurrence-only journey is
[`examples/authored_workflow/information.rs`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs):
propose, advance to `AwaitingOperation`, execute through
`for_workflow_operation`, and advance to the terminal, with no approval node.

### 6.6 Publish, discover, start, advance

Every lifecycle call is a method on the ordinary mutation request chain. The
mutation intent is the matching control intent (authoring, instance start,
advance, or approval), and the workflow runtime from Â§6.2 is passed in.

`workflow_runtime` dereferences to the program runtime, so
`workflow_runtime.request(&principal, &scope)` is the request entry for every
call below. (Bank wraps it in its own `runtime.request(..)`, which maps its
principal type and then makes exactly this call.)

**Bind and publish a definition** on a branch. Publication names the
predecessor it expects to replace; there is no last-writer-wins overwrite.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs
let contract = workflow_runtime
    .workflow_spec()
    .bind_definition(approved_business_payment_definition()?)?;
let outcome = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAuthoringIntent { input: authority })
    .without_source()
    .idempotency(&command_key)
    .prepare_workflow_publication(contract, expected_predecessor)
    .map(|request| request.execute())?;   // WorkflowDefinitionPublicationOutcome

// Take the published definition out of the outcome. The other variants are
// Application(uncommitted), where nothing landed, and ProjectionDenied(receipt),
// where the commit landed but no reference could be projected.
let WorkflowDefinitionPublicationOutcome::Published(published) = outcome else {
    return Err(/* your error for a refused publication */);
};
let definition: PublishedWorkflowDefinitionRef = published.definition().clone();
// published.receipt() is the commit receipt; published.replayed() is true when
// the idempotency key replayed an already-landed commit.
```

`expected_predecessor` is `WorkflowDefinitionExpectedPredecessor::Absent` for
the first revision, or `Published(previous_ref)` for a revision.

**Discover** metadata only through an explicitly governed retained read. The
workflow must declare a `definition_metadata`
rule naming the concrete installed read binding. Subject permission does not
open this separate route.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let basis = request.retain_current_read()?;
let target = ApplicationWorkflowDefinitionReadTarget::<PurchaseWorkflowSpec>::new(identity);
let found = request.at(&basis)
    .discover_workflow_definition::<PurchaseWorkflowSpec, DefinitionMetadataRead>(
        workflow, &target, independently_supplied_read_input,
    )?;
// found.basis() identifies the exact data; found.definition() is Current,
// Retired or Unpublished. Neither the result nor a reference authorizes a start.
```

Fresh current capability admission precedes any existence-dependent lookup.
Foreign-Spec names are undisclosed as `Unpublished`; technical/native corruption
failures remain typed errors. A revoked disclosure grant cannot read even an old
retained basis. There is no execution-request-only discovery overload.

**Start** an instance, **propose**, and **advance**:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs and approved_payment_workflow/progression.rs
let started = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentInstanceStartIntent { input: authority })
    .without_source()
    .idempotency(&start_key)
    .prepare_workflow_instance_start(&workflow_runtime, definition)
    .map(|request| request.execute())?;      // WorkflowInstanceStartOutcome
let WorkflowInstanceStartOutcome::Started(started) = started else {
    // Superseded(..) names the current definition; Retired(..), Application(..),
    // and ProjectionDenied(..) are the other refusals. Nothing was started.
    return Err(/* your error */);
};
let instance: PublishedWorkflowInstanceRef = started.instance().clone();

let proposed = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAuthoringIntent { input: proposal_input })
    .without_source()
    .idempotency(&proposal_key)
    .prepare_workflow_proposal(&workflow_runtime, instance.clone())
    .map(|request| request.execute())?;      // WorkflowProposalOutcome
let WorkflowProposalOutcome::Published(proposed) = proposed else {
    return Err(/* Application(..) or ProjectionDenied(..) */);
};
let proposal: PublishedWorkflowProposalRef = proposed.proposal().clone();
// proposed.transition() is the transition entity; proposed.node_path() the node.

let progress = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
    .without_source()
    .idempotency(&advance_key)
    .prepare_workflow_advance(&workflow_runtime, instance.clone())
    .map(|request| request.execute())?;      // WorkflowProgressOutcome
```

Use a fresh idempotency key per logical command. Reusing a key replays the
recorded outcome instead of performing a second step.

`WorkflowProgressOutcome` says what happened or what the instance waits for:

| Variant | Meaning |
|---|---|
| `Completed` | A step was recorded |
| `AwaitingProposal(RequiredWorkflowProposal)` | Names the node, operation and input type; publish through fresh `prepare_workflow_proposal` admission |
| `AwaitingActor` | The caller is not authorized for the next step; an authorized caller pumps the same instance |
| `AwaitingAssessment`, `AwaitingCondition`, `AwaitingEvidence` | Settle the named read, then accept it |
| `AwaitingApproval` | A signed approval is required |
| `AwaitingOperation` | The guarded effect must run ([Â§6.5](#65-who-may-run-a-guarded-operation)) |
| `Application`, `PreparationDenied`, `AuthenticationDenied`, `IdempotencyDenied` | No step was recorded; each says why |
| `ProjectionDenied` | The commit landed and carries its receipt; only its projection was refused |
| `ProjectionReadStopped { receipt, cause }` | The commit landed and its receipt is retained; the projection read it requires stopped (`cause` is a `RelationalExecutionDenialCause`) |

`AwaitingProposal(RequiredWorkflowProposal)` names the node, operation and input
type. Publish the proposal through fresh `prepare_workflow_proposal` admission;
the need grants no authority and waiting records no transition.

**Assessments and conditions.** Turn an advance into an assessment demand
with `.into_assessment_demand(demand)`, settle it, then accept it with
`.accept_assessment(&settlement)`. A condition is accepted by naming each
operand's published result:
`.condition(&required).operand::<Binding, _>("name", result).accept()`.

**Approve or reject.** An approval needs a fresh authentication event and a
signature. Replaying a recorded key is the only exception.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs
let signing = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentApprovalIntent { input: authority })
    .without_source()
    .idempotency(&approval_key)
    // `required` is the RequiredWorkflowApproval from
    // WorkflowProgressOutcome::AwaitingApproval(required).
    .prepare_workflow_approval(&workflow_runtime, instance, &required, &proposal, decision)?;
let Some(intent) = signing.authentication_intent().cloned() else {
    return signing.execute_replay();
};
let event = authenticate(credential, intent).await?;   // your authentication adapter
signing.sign(&event).map(|request| request.execute())
```

`decision` is `WorkflowApprovalDecision::{Approve, Reject}`.

**Run the guarded effect** when an advance returns `AwaitingOperation`. Bank's
`ApprovePaymentMutationBinding` explicitly requires its approval capability.
Authority is issued only after that exact approval is satisfied. A binding that
declares `ApplicationWorkflowExecutionRequirements::occurrence_only()` can run
without an approval ([Â§6.5](#65-who-may-run-a-guarded-operation)).

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Exact original meaning is retained once; construction performs no effect.
let owner = workflow_runtime.request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(operation_key)?
    .for_workflow_operation(&workflow_runtime, &required)?
    .retain_operation::<BankApplication, ApprovedPaymentEffectAction>(workflow_runtime.program_runtime())?;
let fresh = workflow_runtime.request(&fresh_principal, &fresh_scope)
    .on_branch(instance.branch());
match owner.advance(&fresh) {
    WorthQueryWorkflowOperationAdvance::Progress(owner) => retain_for_next_request(owner),
    WorthQueryWorkflowOperationAdvance::Refused { operation: owner, denial } => {
        report_refusal(denial);
        retain_for_next_request(owner);
    }
    WorthQueryWorkflowOperationAdvance::Completed(completed) => {
        // Native Fresh result and Recovered evidence remain distinguishable.
        let accepted = fresh.mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source().idempotency(&accept_key)
            .prepare_workflow_advance(&workflow_runtime, instance)?
            .accept_owned_operation(completed)?;
        record_native_progress(accepted.progress);
        // Refusal returns the actual completed owner, never a replacement receipt.
        accepted.operation.close(&fresh)?;
    }
    WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
        record_native_stop(&terminal);
        terminal.close(&fresh)?;
    }
}
```

Local guarded completion already includes the operation settlement in its business
commit. A caller may close that actual completed owner and freshly advance the
successor directly. Optional completed-owner acceptance authenticates and projects
the original node/receipt as replay; it does not perform another settlement or
execute the successor, and it does not record a new confirmation command key.
For external effects, genuine native recovery/authenticated inbound custody must
first permit the existing settlement path. Closing source cleanup or disposing a
dispatch handle does not substitute for that evidence, assessment acceptance or approval.

**Everything else in the lifecycle:**

| Call | Does |
|---|---|
| `prepare_workflow_navigate_back` | Records `NavigatedBack` |
| `prepare_workflow_instance_cancellation` | Ends an instance where it stands. Not a rollback: the outcome names every performed step. |
| `prepare_workflow_instance_migration(workflow, instance, target, resume_at)` | Continues an instance on a later definition from a named node |
| `prepare_workflow_fork_continuation` | Continues an instance on a forked branch |
| `prepare_workflow_definition_retirement` | Retires a definition lineage |
| `accept_owned_operation(completed)` | Fresh selected admission confirms original local atomic settlement as replay without a new commit/key write; external settlement requires actual recovery/inbound evidence. Success and refusal retain the same completed owner and cleanup. |
| `operation.advance(&fresh_request)` | Continues original native unpublished/keyed/dispatch custody without reconstructing input, key, authority or recovery handle. |
| `completed.close(&fresh_request)` | Releases actual cleanup; refusal returns the same owner. Physical release does not make the workflow terminal. |
| `mutate(i).workflow(&phase, workflow, authored)â€¦`, `start_workflow`, `run_workflow` | The ordinary convenience lane, which validates, binds, publishes, starts, and runs in fewer calls. `phase` is the `WorthQueryAdvancementPhase` of an open advancement, such as the one `with_host_advancement` passes its closure. |

**Observing an instance.** There is no instance-status query. You learn state
from each typed outcome, from `discover_workflow_definition`, from an
assessment demand's notifications, and from the adoption workflow inventory
([Â§7.4](#74-workflows-and-custody-during-adoption)).

**What refuses:**

- Starting from a superseded revision returns
  `WorkflowInstanceStartOutcome::Superseded(..)`, which names the current
  definition. Nothing is written. A retired lineage returns `Retired`.
- Passing another workflow's runtime is refused with `RuntimeMismatch` by
  the instance-start, proposal, advance (and approval), definition-retirement,
  and operation-binding preparations. Definition publication binds its
  contract to the spec up front, so it has no such variant.
- A canceled instance refuses further steps
  (`WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled`).
- A control intent sent through the ordinary mutation lane is refused
  (`MutationHandlerExecutionDenial::WorkflowControl`).

Run the full authored-workflow journey. The example first runs
[`information.rs`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs),
an unsigned, occurrence-only operation, then the signed review journey in
`main.rs`:

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example authored_workflow
```

---

### 6.7 Capture, approve, archive, and observe one workflow

The neutral [observable workflow example](../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow.rs)
connects the ordinary owners in one application. It captures a document's
retention, checks the retained value, requires a different admitted approver,
and waits for the original archive operation's inbound completion. Use this
shape for a process that combines local data and an external effect.

Run the complete example target:

```sh
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example observable_workflow
```

The application imports `worth-query-decl` for meaning and `worth-query-host`
for requests, owners and observations. Its [definition](../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow/definition.rs)
explicitly installs subject, definition, metadata and audit read rules and a
recorded-actor policy. Capture declares occurrence-only requirements; archival
requires the authored approval. The admitted proposal and approval supply their
actors. Business input never supplies an actor override.

#### Keep a basis, reacquire a locator

This smallest call sequence assumes the example's installed vocabulary and read
bindings. Retain the original `prepared.discovery_intent()` before executing the
start. A later request uses that typed data; it does not recompute start identity
from the currently installed program. The receiver owns admission internally:
`retain_current_read()` takes no phase argument. Inside an already admitted host
call, the separate advanced `retain_read(&phase)` receiver remains available.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let request = runtime.request(&principal, &scope).on_branch(branch);
let basis = request.retain_current_read()?;
let found = request.at(&basis)
    .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
        application.vocabulary(), &original_start, subject_input, audit_input,
    )?;
let target = found.target(); // Option<&WorthQueryWorkflowReadTarget>
```

Match Live, Historical or Missing before choosing what to do. Only Live returns
a usable instance locator; a historical view is data. For an existing result,
clone `observation.target()` or `found.target()` rather than assembling an
identity. Discovery intent, locators, targets and pages retain data custody;
none retains a grant, principal, request, approval need or command permission.
Every read uses fresh current disclosure admission even at an old data basis.
Subject read and audit use independent disclosure: approvals, pending owners and
incomplete recovery remain visible according to subject-read permission even
when actor details are withheld.

For current-format restore, retain the typed original start intent as ordinary
intent data and use a new authenticated request against the restored runtime's
own retained basis. Reacquire references there; do not serialize live locators,
reuse an old runtime's retained basis, or interpret a format refusal as Missing.
Unsupported displaced artifacts are refused intact before installation. Logical
migration and native fork continuation remain separate current operations; see
[discovery and observation](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md).

#### Publish a proposal when the owner asks for one

The example starts from the installed definition, then checks the actual
`AwaitingProposal` need before publishing a proposal with an admitted request.
The full principal identity read here is an independent example assertion
against the native receipt, not business input or a source of authority.

<!-- compiled: observable_proposal.rs -->
```rust
    let waiting = advance(&application, &instance, &[10, 11]);
    let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingProposal(need)) =
        waiting.stop()
    else {
        panic!("the first node must expose its actual proposal need: {waiting:?}");
    };
    assert_eq!(need.node_path(), "propose");
    assert_eq!(
        need.operation(),
        WorkflowDefinitionAuthoringOperation::reference().name()
    );
    assert_eq!(
        need.input_type(),
        WorkflowDefinitionAuthoringOperation::reference()
            .input_identity()
            .as_str()
    );
    assert!(waiting.transitions().is_empty());
    let proposal = runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent { input: input() })
        .without_source()
        .idempotency(&20_u64)
        .prepare_workflow_proposal(&application, instance.clone())
        .unwrap()
        .execute();
    let (proposal, expected_proposer) = match proposal {
        WorkflowProposalOutcome::Published(published) => {
            let principal = published.receipt().principal_scope().principal();
            (
                published.proposal().clone(),
                [
                    u64::from(principal.partition_id()),
                    principal.local_slot(),
                    u64::from(principal.generation()),
                ],
            )
        }
        other => panic!("original proposal publication: {other:?}"),
    };
```

Next it advances to capture, runs the original occurrence-bound owner, and
independently reads retention `9`. The real condition selects the approval
branch. A separately authenticated reviewer answers the original
`RequiredWorkflowApproval` challenge and signs it. Observed actors and history
never stand in for that required need or authentication.

#### Keep the original external operation through settlement and cleanup

The archive transport launches this same executable as a local OS child and
writes the original payload through owned pipes to an actual file. Its installed
verifier authenticates that bounded local channel. This is a local producer,
not a remote cryptographic service or durable crash-recovery demonstration.
The declared inbound operation retains a continuing owner until native ingress.
Before dispatch, the caller drives that same owner while the transport has no child result.
As soon as authenticated child output exists, it admits genuine ingress through
the installed verifier before any further recovery advance. Transport failure is
an error, not an instruction to retry a possibly performed external effect.
Receiving the declared inbound completion is distinct from corrective recovery
or permission to retry the external effect. The archive declares no corrective
aftermath; the caller supplies genuine ingress and leaves settlement to native
inspection of the original dispatch.
At this example's real pending seam, `exercise_pending_custody` is an
example-local assertion phase: close refuses while retaining the owner, a
cancelled fresh advance returns typed cancellation with that owner, and a legal
fresh advance still waits before ingress. It checks the committed head and
actual handler/transport entry counts to reject repeated work. It returns the
original owner; it is not a framework recovery or authority API.
For this declared inbound effect, native `Progress` retains the original owner
until authenticated ingress is admitted. The example requires that pending
custody exercise before accepting the original performed operation under fresh
admission and independently closing cleanup:

<!-- compiled: observable_operation.rs -->
```rust
    let mut operation = request
        .mutate(crate::effect::ArchiveIntent {
            input: crate::journey::input(),
        })
        .without_source()
        .idempotency_owned(80_u64)
        .unwrap_or_else(|(_, _, cause)| panic!("original archive capture: {cause:?}"))
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_| panic!("exact approved occurrence"))
        .retain_operation::<MixedProgram, crate::effect::ArchiveAction>(
            application.program_runtime(),
        )
        .unwrap_or_else(|_| panic!("installed archive action"));
    let receive = |envelope: &[u8]| {
        application
            .runtime()
            .inbound_occurrences()
            .receive(verifier, envelope, &scope)
            .execute()
            .expect("native provenance admission of the real completion");
    };
    let mut ingress_received = false;
    let completed = loop {
        match operation.advance(
            &application
                .runtime()
                .request(&actor, &scope)
                .on_branch(instance.branch()),
        ) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => {
                operation = next;
                // Progress can precede dispatch. Actual child output, not this
                // variant, determines when ingress must precede further recovery.
                if !ingress_received
                    && let Some(envelope) = archive.envelope().expect(
                        "inspect actual child completion without retrying a failed dispatch",
                    )
                {
                    operation = exercise_pending_custody(operation, application, archive);
                    receive(&envelope);
                    ingress_received = true;
                }
            }
            WorthQueryWorkflowOperationAdvance::Completed(done) => {
                if !ingress_received {
                    let envelope = archive
                        .envelope()
                        .expect("inspect actual synchronous child completion")
                        .expect("native completion must retain genuine authenticated child output");
                    receive(&envelope);
                }
                break done;
            }
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("archive advance refused with original custody: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                panic!(
                    "archive terminal: source={:?}, recovery={:?}, abort={:?}",
                    terminal.denial(),
                    terminal.recovery_denial(),
                    terminal.original_abort()
                );
            }
        }
    };
    assert!(
        ingress_received,
        "the original owner must exercise pending custody before ingress/completion"
    );
    assert_eq!(archive.dispatch_calls(), 1, "one original child dispatch");
    assert_eq!(
        crate::effect::handler_invocations(),
        [1, 1],
        "one original handler execution"
    );
    let accepted = request
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&90_u64)
        .prepare_workflow_advance(application, instance.clone())
        .unwrap()
        .accept_owned_operation(completed)
        .unwrap_or_else(|(_owner, cause)| panic!("exact original external settlement: {cause:?}"));
    assert!(
        matches!(&accepted.progress, WorkflowProgressOutcome::Completed(performed) if performed.node_path() == "archive")
    );
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("original dispatch and native cleanup must close"));
```

A refused advance returns its original owner; retain it before deciding whether
to retry with a fresh request. `Completed` is not permission to abandon native
cleanup. Before ingress, this declared inbound wait mints no corrective recovery
handle or retry authority. Close refuses `NativeObligationRetained` with the same
owner until a fresh advance observes native settlement. This example requires
that wait and closes only after genuine ingress and acceptance.
After settlement the caller advances through `AwaitInbound` to the terminal and
checks that exactly one file contains the original identity and retention.
[Operations and ownership](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations)
explains the other stop postures; callers do not resubmit effects or decode
private receipts to drive their own retry engine.

#### A live mark does not refresh a retained read

The example first reacquires a history target at an old basis. The operator has
subject access; the reviewer independently has audit access. History preserves
proposal and outcome rows while returning `WorthQueryWorkflowHistoryAudit::Withheld`
for the operator. `page(..)` below calls the public
`read_workflow_history::<MixedWorkflow, SubjectRead, AuditRead>` with both inputs,
a width of `16`, `PublishedOnly`, and no prior page. A wider history continues
with the returned page under the same exact basis and selection; each request
rechecks disclosure. No page is authority to advance.

The closure `approve()` performs the actual original approval with fresh reviewer
authentication. Polling uses the existing native envelope subscription. `Changed`
marks relevant changes without altering `old`. The caller explicitly retains
`now`, reacquires a target and requests audit again:

<!-- compiled: observable_history.rs -->
```rust
    let mut subscription = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&old)
        .subscribe_workflow_history::<MixedWorkflow, SubjectRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .expect("native envelope interest at the old data basis");
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Pending
    );
    let expected_approver = approve();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Changed
    );
    // A mark reports relevant change. It does not mutate the old retained read.
    let historical = page(application, &reviewer, &scope, &old, &target);
    assert_eq!(historical.basis().selected_commit(), old.selected_commit());
    assert!(historical
        .entries()
        .iter()
        .all(|row| row.approval().is_none()));
    // Refresh is an explicit caller choice with new current admission.
    let now = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .retain_current_read()
        .expect("explicitly retain a fresh post-approval basis");
    let fresh_discovery = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&now)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .unwrap();
    let fresh_target = fresh_discovery
        .target()
        .expect("same native instance at fresh data");
    let fresh = page(application, &reviewer, &scope, &now, fresh_target);
    assert_eq!(fresh.basis().selected_commit(), now.selected_commit());
    let approved = fresh
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("actual recorded approval");
    assert_eq!(
        approved.approval().unwrap().decision(),
        WorkflowApprovalDecision::Approve
    );
    assert_eq!(
        approved.approval().unwrap().proposal_identity(),
        proposal.identity()
    );
    match approved.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            approver: Some(actor),
            ..
        } => {
            let actor = actor.entity_id();
            assert_eq!(
                [
                    actor.partition_value_u64(),
                    actor.local_slot_value(),
                    u64::from(actor.generation_value())
                ],
                expected_approver,
                "the performed native approval is the actor oracle"
            );
            assert_ne!(actor, proposer);
        }
        _ => panic!("independent audit reveals the performed approver"),
    }
    let fresh_withheld = page(application, &operator, &scope, &now, fresh_target);
    assert_eq!(fresh.entries().len(), fresh_withheld.entries().len());
    for (audited, withheld) in fresh.entries().iter().zip(fresh_withheld.entries()) {
        assert_eq!(audited.occurrence(), withheld.occurrence());
        assert_eq!(audited.node_path(), withheld.node_path());
        assert_eq!(audited.outcome(), withheld.outcome());
        assert_eq!(audited.proposal(), withheld.proposal());
    }
    let approved_without_audit = fresh_withheld
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("withholding cannot hide approval state");
    assert!(matches!(
        approved_without_audit.audit(),
        WorthQueryWorkflowHistoryAudit::Withheld
    ));
    // Releases native subscription custody only; never operation cleanup proof.
    subscription.close();
```

`subscription.close()` takes no request or execution argument: it releases native
subscription custody. It does not close an operation, settle an inbound effect,
accept workflow evidence or prove cleanup. Cancellation and read/reconstruction
resource refusals remain typed; do not convert them to absent workflow state.

The full example includes installation, principals, grants and policies. Use the
[request and workflow usage guide](capability-index.md#use-requests-and-retained-operations)
for admission, progress and cleanup handling.

---

### 6.8 Retain capability commands instead of reissuing them

Delegation and revocation use ordinary request-level command owners, separately
from workflow-bound domain effects. Begin with
`request.retain_capability_delegation_in_program(..)` or
`request.retain_capability_revocation_in_program(..)`. Each captures the original
input, owned key, preconditions and native preparation closure against the
installed program and principal binding. Capture refusal returns those original
values. The [workflow guide's command example](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands)
shows the typed arguments; do not substitute a domain mutation handler.

| Returned owner state | Caller responsibility |
| --- | --- |
| `WorthQueryCapabilityCommandAdvance::Progress(command)` | Retain that command. Release any pending managed cleanup through its receiver before the next advancement. Supply a fresh request to `command.advance(&request)`. |
| `Refused { command, denial }` | Keep the returned original command and inspect the typed denial. Correct the request/admission or resolve the native condition before choosing another advancement; never recapture the command as a retry. |
| `Completed(done)` | Inspect `done.evidence()` for the native Fresh/Recovered result. Independently call `done.close_with_evidence(&request)`; success returns the closed operation and original evidence, while refusal returns the same completed owner and denial. |
| `Terminal(terminal)` | Stop effect advancement. Inspect the native outcome/idempotency resolution, and independently release cleanup or close the returned terminal owner. Terminality does not authorize resubmission. |

A later request must match runtime, principal and command meaning and pass fresh
authorization. Recover deferred publication and ambiguous commits through this
owner to obtain the original native result. Use the corresponding receivers for
command cleanup, workflow operation cleanup and history subscription release.

---

## 7. Adopt a new program on a branch

Adoption moves **one exact branch** from the program it runs to a rostered
target revision, in one product publication. Other branches, including
earlier forks, keep their programs.

### 7.1 The call chain

The program currently activated on the branch declares its adoption permission
through `ApplicationProgramDefinition::adoption_authorization`. Its
`ApplicationProgramAdoptionBinding` names the installed capability, operation,
input binding and principal binding. Declare the corresponding capability
contract in the schema and issue real grants through your applicationâ€™s grant
owner. A program with no declaration refuses adoption with
`WorthQueryProgramAdoptionAuthorizationDenial::CapabilityNotDeclared`.

Bank uses `DeployBankProgram`, scoped to the exact currently governing source
program revision. Its deployment grants are separate from estate and payment
permissions. Installation issues only explicitly supplied
`BankProgramDeploymentGrantSeed` values; it grants nothing to an actor merely
because that actor authenticated. The recorded issuer can revoke one through
`revoke_program_deployment_grant`. An old source-revision grant does not permit
deployment after that branch adopts another revision.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let grant = BankProgramDeploymentGrantSeed::for_program::<BankApplication>(
    "operator-p0-deployment", issuer, operator, valid_from, valid_through,
)?;
let seed = BankWorldSeed::new(snapshot).program_deployment_grant(grant);
```

`compare`, `adopt`, `prepare`, publication and recovery each admit the declared
capability in their current request phase. The source program governs; the
target cannot select a weaker permission for its own adoption. Grants and
authentication must still be current at publication.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-testkit-certification-application-graph/src/certification/application_graph/adoption/branch_adoption.rs
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

let target = *host
    .supported_program::<RetentionProgramP1>()
    .expect("P1 is rostered")
    .owned_revision();
let programs = host
    .runtime()
    .request(&principal, &scope)
    .on_branch(main)
    .programs();
let requirements = programs
    .compare(&target)
    .expect("the host must describe P0 to P1 requirements");
let prepared = programs
    .adopt(&requirements)
    .expect("the source capability admits adoption")
    .prepare(64)                                   // maximum selection work
    .expect("the seed satisfies P1 and adoption must prepare");
match host.runtime().request(&principal, &scope).on_branch(main).programs().publish(prepared) {
    WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
        /* no adoption effects; retain prepared for a fresh authorized request */
    }
    WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, prepared } => {
        /* execution admission refused; retain the reservation */
    }
    WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
        /* the native publication attempt stopped */
    }
    WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => { /* branch runs P1 */ }
    WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => { /* e.g. stale head */ }
    WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
        /* recover: unpublished.into_recovery(), then programs().recover(..) */
    }
}
```

The same chain in production code is
`bank-server/src/program_adoption.rs` (`prepare_branch_program_adoption`).

| Step | Call | Returns |
|---|---|---|
| Target | `host.supported_program::<P>().expect(..).owned_revision()` (needs the `WorthQueryProgramOwner` import) | `&ApplicationProgramRevision` |
| Inspect | `.on_branch(b).programs().inspect()` | `Result<WorthQuerySelectedProgramInspection, WorthQueryApplicationProgramInspectionDenial>` |
| Compare | `.compare(&target)` | `WorthQueryProgramAdoptionRequirements` or `WorthQueryApplicationProgramAdoptionPreparationDenial` |
| Adopt | `.adopt(&requirements)` | `Result` containing a preparation request or typed authorization denial |
| Optional | `.migration(prepared_migration)`, `.workflow(dispositions)` | The same request |
| Prepare | `.prepare(maximum_selection_work)` | `WorthQueryPreparedBranchAdoption` (move-only, fields private) |
| Publish | `programs.publish(prepared)` | `WorthQueryBranchAdoptionPublicationOutcome` |
| Recover | `programs().recover(unpublished.into_recovery())` | `Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryApplicationProgramAdoptionRecoveryFailure>` |

`compare` comes first because the requirements are the input to `adopt`.
`WorthQueryProgramAdoptionRequirements` exposes `semantic_diff()`,
`validation_scopes()`, `added_rules()`, and the predicates
`requires_existing_state_validation()`, `requires_migration_assessment()`,
`requires_custody_inventory()`, and `semantically_equivalent()`.

### 7.2 What each outcome means

| Situation | What you get |
|---|---|
| Declared permission is missing, revoked or expired | Typed `Authorization(..)` refusal; publication returns the exact prepared owner in `AuthorizationDenied` |
| Existing state satisfies the target | `prepare` is `Ok`; `selected_entity_count()` says how much was validated |
| Existing state violates a target rule | `TargetRuleRejected { identity }` |
| The target needs a migration | `requires_migration_assessment()`, then `MigrationAssessmentRequired(..)` if you did not supply one |
| The migration's source changed after it was prepared | `MigrationSourceChanged` |
| The migration was prepared for a different target | `MigrationTargetMismatch` |
| Requirements went stale | `RequirementsChanged`: compare again |
| Selection is too large | `SelectionLimitExceeded { maximum_work_units, consumed_work_units }` |
| The branch head moved before publish | `NoEffect(..)` whose `cause()` is `NoEffectCause::StaleExpectedProductHead` |
| Publication could not complete | `ProductUnpublished(..)`: recover it; recovery never re-runs migration authoring |
| Adopted | `Performed(..)` with `source()`, `target()`, `selected_entity_count()`, `migration()`, `custody()` |

The preparation denials in this table (`TargetRuleRejected`,
`MigrationAssessmentRequired`, `MigrationSourceChanged`,
`MigrationTargetMismatch`, `RequirementsChanged`, `SelectionLimitExceeded`)
and `WorkflowDispositionRequired` ([Â§7.4](#74-workflows-and-custody-during-adoption))
are variants of `WorthQueryBranchAdoptionPreparationDenial`. They arrive
wrapped as
`WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(..)`; its
`Authorization(..)` variant reports the capability refusal, and
`ProductSelection(..)` means the branch could not be selected.

A writer still running under the old program after adoption lands gets a
stale commit outcome, never a silent commit under the wrong law.

### 7.3 Migrations
#### Advanced program migration preparation

Use the [ordinary owning entry](#53-execute-through-the-program) for application actions.
Migration candidates are deliberate native control selected with `advanced()`.


When the target needs existing state repaired, prepare the migration as a
candidate under the target program, then hand it to the adoption:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// tests/application_graph/adoption/migration.rs (abridged)
let migration = match host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .advanced().mutate(SetRetentionIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_program_migration(&target)?
{
    WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
    other => { /* DomainDenied, Cancelled, DeadlineExceeded */ }
};
let programs = host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();
let requirements = programs.compare(&target)?;
let prepared = programs
    .adopt(&requirements)?
    .migration(migration)
    .prepare(64)?;       // migration and validation prepare atomically
```

A prepared migration commits nothing and registers no idempotency record
until the adoption publishes.

### 7.4 Workflows and custody during adoption

The runtime privately checks every current definition and live instance before
adoption. That complete safety inventory is not a consumer read surface.
For disclosure, select one typed definition with `read_workflow_instances`,
its installed Definition read binding and fresh principal/input. A metadata
read does not grant instance enumeration.

Submit explicit choices for opaque references already possessed, or request
native carry-compatible handling of every fact found by the fresh safety check:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowDispositions, WorthQueryWorkflowDefinitionDisposition,
};
let choices = WorthQueryWorkflowDispositions::new()
    .definition(&definition, WorthQueryWorkflowDefinitionDisposition::Retire);
let prepared = programs.adopt(&requirements)?.workflow(choices).prepare(1_024)?;
// When every fresh fact should be carried, use instead:
let choices = WorthQueryWorkflowDispositions::carry_compatible();
```

Explicit choices must totally and legally decide the fresh native inventory.
An omitted live instance refuses; retiring a definition does not cancel its
instances. A performed effect or pending external-owner/approval obligation
cannot be cancelled away. The existing source-program adoption authority is
still required. References and read views grant no adoption permission.

`WorkflowDispositionRequired` and `WorkflowDispositionRejected` carry no
inventory or occurrence identities. Native candidate preparation logs also stay
private; execution cancellation/resource causes retain their typed category. A new start can invalidate explicit
choices; take a fresh governed view and decide again. A start after preparation
is still fenced by the native expected head at publication. The carry-compatible
policy explicitly applies to fresh native truth; it is not a cached inventory.

### 7.5 Several branches

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let prepared = request
    .on_branches(coverage, &ordered_targets)?     // WorthQueryProgramAdoptionCoverageDenial
    .programs()
    .adopt(&target)?
    .prepare(maximum_selection_work_per_branch)?; // WorthQueryPreparedBranchSetAdoption
```

This prepares adoption across a set of branches. It is **not atomic**:

- Every remaining branch uses its own current source-program permission.
  Advancement rechecks it in a fresh request and returns a refused pending
  native owner to the set, preserving the completed prefix.
- Preparation preflights every branch before any branch publishes. One
  incompatible branch denies the whole preparation and names that branch.
- `prepared.advance(runtime, &principal, &scope)` publishes the next branch and returns its progress
  (`None` when every branch is done). After a `NoEffect` or unpublished
  branch it is denied with `ResolutionRequired` until you resolve it.
- `prepared.cancel()` stops the rest; branches already adopted stay adopted.
  It returns `Result<WorthQueryBranchSetAdoptionCancellation, Self>`: while a
  branch awaits resolution it hands the adoption back, so resolve it and
  `advance` first.
- `resume(stopped, max)` continues a stopped set.

Coverage comes from `runtime.branches().program_adoption_coverage(..)`;
`on_branches` orders it against your target list and refuses a mismatch with
`WorthQueryProgramAdoptionCoverageDenial`. A branch-set member that committed
but did not publish is recovered with
`request.recover_branch_set_adoption(recovery)`.

### 7.6 Siblings, history, and retirement

- **Siblings keep their program.** Adoption writes only the adopting branch.
  A branch forked earlier keeps its own activation, so it keeps running and
  enforcing the old program's rules.
- **Retained reads pin their program.** A retained selection or read keeps
  the old program's support and occurrence alive for as long as it is held.
- **Retirement.** `retire_program_support(&revision)` succeeds only when no
  current branch, retained interpretation, or mandatory custody still uses
  that revision.

---

<a id="8-rules-for-ai-agents"></a>

## 8. Application execution checklist

- Import only `worth_query_decl::facade` and `worth_query_host::facade`.
- Start ordinary features with the [complete definition](#complete-feature-definitions)
  and consume its schema, program, runtime and output projections. Use
  `ApplicationFeatureSpec::root::<S, F>()` (or `at`) for specialized composition;
  finish each builder before validation.
- Validate programs with
  `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` and
  install with `application_installation::in_memory_program` or
  `in_memory_rostered_program`.
- Run fixed, discovered and source-only actions through `idempotency_owned(key)` and
  `retain_operation::<Program, Action>(&runtime)`, then supply fresh ordinary
  requests for advancement, delivery and close. Native admission checks the
  original runtime, branch and full action meaning on every attempt.
- Retain guarded workflow effects with their original issued requirement; use a
  separate owning assessment demand for observation, then explicitly accept
  evidence or request approval as the workflow requires.
- Match the owning lane's `Progress`, `Refused`, `Terminal`, `Settled` or `Completed`
  variants, retaining each returned owner. `NoEffect`, `ProductUnpublished` and
  `Commit(..)` are native advanced outcomes; workflow `Awaiting*` progress is
  distinct from business-effect completion. Inspect a stop's cause before retrying.
- `compare` before `adopt`, and pass the requirements you got.
- Name the expected predecessor when you publish a workflow definition.
- Pump workflows yourself: one advance per request, until an advance returns
  `Completed(t)` with `t.terminal() == true`. `Completed` alone means one step
  was recorded.

See [request and workflow usage](capability-index.md#use-requests-and-retained-operations)
for the retained-owner lifecycle.


---

## 9. Use the consumer vocabulary

Use the declarations and host functions exported by `worth-query-decl` and
`worth-query-host`:

| Task | API |
|---|---|
| Compose application meaning | `ApplicationProgramDefinition` and complete feature definitions, or `ApplicationFeatureSpec` for specialized composition |
| Declare an action | `action::<Binding, Delivery>()` on the feature builder; inspect it through `ApplicationActionDeclaration` |
| Adopt a revision | `compare(&revision)` â†’ `adopt(&requirements)` â†’ `[.migration(..)] [.workflow(..)]` â†’ `prepare(max)` |
| Inspect semantic changes | `ApplicationSemanticChangeKind::{Added, Removed, Changed}` per family |
| Retain a published workflow definition | `PublishedWorkflowDefinitionRef` |
| Install workflow vocabulary | `retain_workflow_spec` on the program runtime, then `support_workflow_spec` on the returned workflow runtime |

---

## 10. Where to go deeper

| Topic | Guide |
|---|---|
| Programs, rosters, adoption, branch sets, retirement, full denial lists | [Programs and Adoption](../workspaces/worth-query/crates/worth-query/docs/foundations/programs-and-adoption.md) |
| Every workflow call, budget, denial, and a worked example | [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) |
| Feature capsules, ports, connections, and the authoring rule | [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) |
| Application action preparation, execution, recovery and cleanup | [Ordinary Application Front Door](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md) |
| Disclosed record generations, bounded canonical history, fresh payload and metadata permission | [Record And Aspect History](../workspaces/worth-query/crates/worth-query/docs/capabilities/record-and-aspect-history.md) |
| Contribution composition, installation and output demand | [`worth-query-host` README](../workspaces/worth-query/crates/worth-query-host/README.md) |
| Declaration surface | [`worth-query-decl` README](../workspaces/worth-query/crates/worth-query-decl/README.md) |
| The complete pure-facade reference application | [Bank public consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) |
| What the runtimes guarantee underneath all of this | [How WORTH Works Â§9, Â§12, Â§13](how-it-works.md#9-query-the-life-of-one-request) |
| Find the call for a task; what to do before concluding a capability is missing | [Capability Index](capability-index.md) |

The Query guides use names exported through the two application facades.

### Whole-aspect absence

An aspect listed directly in `Entity::Aspects` is required. `Option<A>` permits
whole-aspect absence. A fieldâ€™s `Option<T>` permits that field to be absent;
it does not make its aspect optional. The SDK derives the contract revision
from these laws and the record shape.

<!-- compiled: optional_aspects.rs -->
```rust
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct OptionalRecords;
impl Fragment for OptionalRecords {
    const NAME: &'static str = "optional_records";
    type Bindings = OptionalBindings;
    type Members = (Component,);
}
pub struct OptionalBindings;
impl BindingSet for OptionalBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<ComponentIdentifier> for OptionalBindings {
    type Binding = ComponentIdentifierBinding;
    const WRITABLE: bool = false;
}
impl Scalar<String> for OptionalBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub struct Component;
impl Entity for Component {
    const NAME: &'static str = "Component";
    type Aspects = (Identity, Option<Annotation>);
    type Relations = ();
}
impl Component {
    pub fn reference<Schema: ApplicationSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(OptionalRecords, Self)
    }
}
pub struct Identity;
impl Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = ComponentIdentity;
}
pub struct Annotation;
impl Aspect for Annotation {
    const NAME: &'static str = "Annotation";
    type Row = ComponentAnnotation;
}
pub struct ComponentIdentity {
    pub identifier: Option<ComponentIdentifier>,
}
pub struct ComponentAnnotation {
    pub text: String,
}
impl fragment::Record<ComponentIdentity> for OptionalBindings {
    fn record() -> impl Shape<ComponentIdentity, Self> {
        field("Identifier", |row: &ComponentIdentity| {
            row.identifier.clone()
        })
        .equality_queryable(true)
        .build(|identifier, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ComponentIdentity { identifier })
        })
    }
}
impl fragment::Record<ComponentAnnotation> for OptionalBindings {
    fn record() -> impl Shape<ComponentAnnotation, Self> {
        field("Text", |row: &ComponentAnnotation| row.text.clone()).build(
            |text, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ComponentAnnotation { text })
            },
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentIdentifier(String);
impl ComponentIdentifier {
    fn text(&self) -> worth_foundational::facade::InternedString {
        self.0.clone().into()
    }
    fn checked(value: worth_foundational::facade::InternedString) -> Option<Self> {
        value.into_raw_text().map(Self)
    }
}
impl From<String> for ComponentIdentifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub ComponentIdentifierBinding for ComponentIdentifier { identity: "worth.query.fixture.component-identifier.v1", scalar: String, encode: ComponentIdentifier::text, decode: ComponentIdentifier::checked, });
pub type Identifier =
    FieldSelection<OptionalRecords, Component, Identity, Option<ComponentIdentifier>>;
pub type Text = FieldSelection<OptionalRecords, Component, Annotation, String>;
```

### Runtime World policy ceilings

Construct `host::runtime::WorthQueryProductWorldResources::new` with the clock,
invalidation resources, execution policy and a `RuntimeWorldBudgets` value.
Choose `RuntimeWorldBudgets::without_policy_limits()` for the standard lifecycle,
or install explicit finite policies for branches, commits, history, observations,
publication, recovery and retained custody with `RuntimeWorldBudgets::install`.
Observation clones share one obligation; distinct observations retain their own
obligations until final drop, including after owner close.

`WorthQueryProductWorldResources::install` accepts nonzero finite installation
values. `RuntimeWorldBudgetLimit::get()` returns `Some(maximum)` for a finite
policy and `None` otherwise. `host::runtime::WorthQueryInvalidationResources::new()`
takes no retention policy: the invalidation subscription retains every delivered
position, so a reader never falls outside a window and never reconstructs its
state to catch up. A long run of commits between two observations is delivered
in order, and replacing a conditional runtime remains the explicit
reinstallation path. Output-lineage retention reads the same subscription.

Set optional published-snapshot, active-snapshot and prepared-candidate policies
in `PublicationConfig`; `Some(amount)` selects a finite policy. Retain snapshots
and candidates through their owning runtime and generation. Handle stale or
foreign-runtime refusals and release each original holder when finished.

### Construct committed provider indexes

Declare an `ApplicationManagedIndexBinding` with its complete Source query,
candidate entity, admitted Probe codec and versioned provider identity. Add its
`ApplicationManagedIndexDefinition` to the schema and select it on the Parent
query with `.managed_index(Index::reference(), probe_parameter)`. The contribution
installs the matching pure provider with `setup.managed_index::<Index, _>(provider)`.
Providers borrow disclosed, query-bound Source rows and finite node/candidate
writers; they receive no mutation handle or reusable read permission.
`Node` and `Summary` require `Clone`, so summaries may be owned exact values,
including heap-backed rational bounds. `node_summary` borrows the node and returns
its summary by reference; `merge_summaries` borrows its parts, and summary and
predicate evaluation borrow their inputs. Probes do not clone visited summaries.
Writer `get` returns a borrow guard; copy or clone only when an edit needs an owned
node. Query moves cold nodes into sealed pages and clones touched warm pages to
preserve the retained predecessor. `Denial` stays `Copy`.

For repeated required String reads, use
`row.with_text(name_selector, |name| name.starts_with("al"))`. Nested callbacks
can compare two rows while borrowing their decoded text. Query keeps this memo
inside one actual Source image and provider invocation; before/after images
remain separate. Each use still checks the current request, disclosed field
contract and installed Rust binding. The text cannot escape its callback; return
an owned result such as a boolean or ordering. A text read returns either a
projection denial or the original controlled stop.


Source queries have no parameters and use Public disclosure without capability or
rule governance. Source authorization may be Public or an installed conventional
READ ability. An ability-authorized Source requires a current-only Parent; the
ordinary Source query may still declare its own current and pinned support.
Each Source execution admits its actual principal, scope and READ policy. Completed
caches retain that original authorization support and recheck it before provider
callbacks; Native maintenance also checks the actual Source READ independently of
the Parent. A current structural root or Ready cache does not grant permission.
Parent queries may use Public or Governed disclosure, with their own installed
READ ability and capability admission. Opaque `InstalledPolicyRequired` disclosure is unsupported. Run index
requests as current committed one-shot reads with the same admitted principal
and full-generation scope.
Parent admission and current READ/security checks precede cache/provider access;
ordinary Parent predicates, final ordering, result limits and disclosure still
apply after candidate selection.

Call `request.query(intent).open_index::<Index>()` to obtain a Created registration,
then `request.query(intent).rebuild_index(&index)` for explicit cold construction.
Ordinary `execute()` returns typed `NotReady` while Created or Evicted; it does
not silently rebuild or fall back to a scope-root query. Probe values share the
same complete Source image. Source authored ordering remains intact; the provider
receives a separate canonical full-generation source-slot order. Native owns the
public cache at the actual selected image and publishes a cold successor in its
compound effect. This conservative cache does not require an invariant Source
association; supported ordinary Source guards remain usable. If Native has
performed while World publication is pending, a fresh read at the unchanged World
can still use its genuine selected predecessor cache. After World adopts the
successor, indexed reads return `NotReady` until explicit rebuild.

For a governed Parent, use `open_index_governed` and `rebuild_index_governed`
with the actual capability, operation and request input, then `execute_governed`
for each read. Approved elevation has corresponding `open_index_approved`,
`rebuild_index_approved` and `execute_approved` entrances. Each call freshly
admits the Parent capability and READ policy before cache or provider access;
its normal final disclosure still applies. Source admission and a Ready cache
leave the independent Parent authorization check in force.

Repeated opens share one registration for the actual installed Index/Parent
binding, principal, full-generation scope and World incarnation. Two bindings to
the same Query have independent
registrations. Ordinary lookup and repeated open use indexed routing; unrelated
retained registrations do not add a full scan. Both routing directions retain
their original memory grants, and close removes only its issued registration id.
`index.lifecycle()` reports retained cache status, not fresh authorization.
Public-only status observes the Native head and
can report Evicted while an unchanged World still selects a retained predecessor.
`evict()` releases its image; explicit rebuild
is required before another indexed read. `close()` disposes and unregisters it
for every sibling handle; later open creates a fresh registration. Dropping a
handle leaves the runtime-owned cache alive; explicit close or final runtime
disposal releases the registration's image. Outstanding results, Source
observations or in-flight selections can retain their own backing. Final
release requires those owners and all runtime keepalives to close or drop.
A stopped cold rebuild may leave Failed status; it publishes no Ready image.
Use a fresh lawful request for another explicit rebuild. A retained managed
one-shot read can run while its genuine World and Native selection still equals
the current image. It retains read-only custody. A moved historical image remains
unsupported before provider lookup; ordinary historical Source reads keep their
existing contract.

Declare `STRUCTURE = ApplicationManagedIndexSourceStructure::BinarySourceTree` and
implement Node-only `decompose` to expose each node's stored genuine Source
occurrence and binary children. Capture actual Source slots during build/update;
`decompose` cannot consult a Source roster that changes independently of the Node. Query checks exact Source membership, unique slots and parents,
connectivity and absence of cycles before retaining the root. Warm edits carry
that correspondence using actual changed node coordinates and Native Source
deltas. Opaque structure remains an explicit conservative profile. Definition v1
means Opaque; v2 transports structure; v3 also binds the membership strategy.
Old v1/v2 definitions always retain candidate-only membership.

During a checked warm edit, `nodes.before_source_node(slot)` locates the genuine
occurrence's predecessor node without scanning the Source or adding a provider
lookup table. It returns a descriptive coordinate; `nodes.get` reads the staged
writer and checks liveness. Read the old key before editing that occurrence.
Cold, opaque, missing or foreign occurrences refuse. The box provider uses this
lookup to move a leaf within its pure spatial ordering, without rereading Source.

A Source-complete tree alone does not establish complete query membership.
Declare `MEMBERSHIP = ApplicationManagedIndexMembership::ExhaustiveSourceTree`
and implement `validate_probe` plus `matches_source(row, probe, control)` to use
Query's complete traversal. Query validates the probe even for an empty Source,
visits every checked region and applies the installed pure candidate predicate
to each genuine Source row. It emits that row's slot itself and returns only
after the complete walk and final controls succeed. An interrupted traversal
refuses; it cannot return a partial or empty success. The explicit-rebuild prefix
application below chooses this profile through its `PREFIX_MEMBERSHIP` constant. The provider's legacy `query` callback is
unused on this path; candidate-only providers keep their existing contract.

Exhaustive membership deliberately visits the whole Source and provides no
locality or pruning claim. Candidate membership is distinct from exact domain
refinement such as geometric contact, and grants no mutation eligibility.
Source dependencies remain conservative.

Providers keeping Candidates or ExhaustiveSourceTree may use `type Summary = ()`;
their summary callbacks remain unused. For checked subtree pruning, choose
`ApplicationManagedIndexMembership::CheckedSummarySourceTree`. The provider's
`Summary` is `Copy + Eq`; `source_summary` describes the actual Source row,
`merge_summaries` folds its own contribution and qualified children, and
`node_summary` reads the cached value from the actual Node. Query recomputes and
compares summaries bottom-up before retaining the root. Warm qualification follows
actual Node writes and changed Native Source rows through every final ancestor,
including a changed row for which the provider wrote no Node. A mismatch returns
`SummaryMismatch` and preserves the predecessor.

`merge_summaries` and `summary_may_match` receive
`WorthQueryManagedIndexSummarySources`. Its only row lookup accepts a genuine slot
inside the current checked region; a same-Source slot outside that subtree returns
`ForeignSummarySource`. Summary semantics cannot depend on an unrelated region.
Each receiving summary callback keeps four positive membership facts for its
immutable region. A paid parent walk may stop at an already proved descendant;
eviction falls back to the actual parent chain. Every lookup still checks the
original request, resolves the genuine slot in the current Source and admits the
row before lending it. A miss may traverse its full ancestor chain.

The pure provider owns its enclosure and comparison algebra. Query uses that
qualified summary premise to skip precisely its actual subtree, then applies
`matches_source` to surviving genuine rows itself. The small box example and the
maintained prefix/geometry compositions use this same strategy. Prefix summaries
keep exact UTF-8 extrema; geometry summaries use the numerical owner's closed
bounds and union, without Query-specific tolerances or field policy.

Both complete strategies record the actual visited regions and checked prunes.
Before releasing slots, Query verifies that this cut covers the checked tree and
that matched Source occurrences correspond exactly to those slots. The receiver
checks the actual Source/root pair, probe and read session. Public selections
retain the verified cut with their Source observation; Native committed and
proposed invariant reads require complete evidence in their own read posture.
An interrupted or incomplete cut refuses. Physical traversal shape does not
change semantic Source equality after an index rebuild.

To retain a whole-Parent watch on a complete Native-maintained Source, implement
`WorthQueryManagedIndexPredicateProvider` on the same installed provider and
configure it with `setup.managed_index_with_predicate_routing::<Index, _>(provider)`.
Declare Live eligibility on the Public managed Parent. Calling
`request.query(input).watch_index::<Index>()` returns its complete initial result
and a `WorthQueryManagedIndexWatch`. The opening subscribes before reading and
registers against that exact World image; a concurrent preparation or publication
can refuse the opening. Retry with a fresh request.

Call `fresh_request.query(original_input).next_watch(&mut watch)` for each refresh.
Every call independently admits the Parent, principal, purpose, full parameters,
result limit and World occurrence before inspecting watch metadata. `Delivered`
contains a whole current Parent read and its receipt, plus the actual notification
publication. A notification at B can be served by current read C; only B is
acknowledged, so later notifications remain queued. `Pending`, `Overflow`,
`Unavailable` and `Closed` are distinct. Overflow or unavailable coverage requires
a fresh opening. `watch.close()` or dropping the watch releases its exact posting
and queue custody. Retaining an old result never makes it current.

Query owns the inverse tree and active watch IDs. The provider supplies pure probe
ordering, fixed summary algebra and region matching; it owns no subscriptions.
Prefix and closed-box providers use this same path. Supported profiles require the
complete canonical Source projection. Projected root refinements may conservatively
wake every matching provider candidate; delivery applies the full Parent again.

For governed Parents, open with `watch_index_governed::<Index, _, _, _>` and
poll with `next_watch_governed`, passing the capability, operation and original
purpose input each time. The corresponding `_approved` methods take a genuine
current approved elevation. These methods use the ordinary capability owner and
independent Parent READ admission on every call. Retained request correspondence
binds the actual installed capability, principal/resource, action/purpose,
elevation, related/field/magnitude/cardinality and context. It retains no old
grant or decision authority. Revocation and READ denial leave a queued
notification unacknowledged; a fresh lawful regrant can deliver it.

A governed managed Live Parent, and every governed exact-refined Parent even
without Live, must explicitly permit membership influence for every Parent
scalar projection through its internal field-use rules. Live additionally
requires `LiveMembership`. The exact installed Source/Parent projection profile
then covers implicit provider inputs such as names and bounds. Projecting or
omitting a value does not itself grant membership influence. A permitted hidden
field remains omitted by the ordinary final projector and current disclosure
receipt; a missing influence permission refuses before provider execution.
Broader Source/projection profiles must extend this closure before admission.

Untracked inputs, candidate-only membership, public-cache-only Sources and
continuations refuse this watch entrance. Ordinary protected child pages use
[their existing continuation owner](#protected-continuation-pages); they do not
paginate managed root results. No verified narrowed footprint is issued by
these traversal or watch strategies.

For a domain predicate narrower than candidate membership, declare an optional
`ApplicationManagedIndexRefinement::row_local("domain.identity", version)` in
`Index::REFINEMENT`. Implement `WorthQueryManagedIndexRefinementProvider` on the
same provider with matching `REFINEMENT_IDENTITY` and `REFINEMENT_VERSION`, a
required `validate_refinement_probe`, and `refine_source`. The callbacks receive
only the same typed probe, genuine complete Source row projection (including its
projected relations), and original control. No roster or external reader is
available. Qualify that exact acceptance implies candidate acceptance.

Install that provider with
`setup.managed_index_provider::<Index, _>(provider).with_refinement().install()?`.
Add `.with_predicate_routing()` before `.install()` when it supports watches.
These optional mechanics share the actual provider object and Source. Select
exact access explicitly with `.managed_index_refined(Index::reference(), probe())`
in a Parent definition, or `context.refined_managed_index(&prepared)` for Native
committed/proposed reads. Existing `.managed_index` entrances keep their
candidate meaning. Missing contracts, executables, complete candidate coverage,
or compatible Source/Parent projection profiles refuse exact access.

The maintained exact-name Parent selects that explicit access mode:

<!-- compiled: managed_exact_refinement.rs -->
```rust
pub fn definition() -> ApplicationQueryDefinitionBuilder<
    Catalog,
    ExactQuery,
    source::Parameters,
    source::ItemRow,
    Directory,
> {
    source::base(
        ExactQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index_refined(prefix::PrefixIndex::reference(), probe())
    .order_by(
        source::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    )
}
```

Query verifies the complete candidate cut first, then evaluates every candidate
once. It retains the original cut, all accepted/rejected dispositions and the
accepted subset before releasing exact results. Probe validation still runs for
an empty candidate set. Domain refusal or interruption refuses the whole result.
Each retained watch uses its actual Parent's candidate or refined mode during
before/after routing; equal probes keep distinct registrations. Current full
Parent read, ordering, disclosure and ordinary Source dependencies still apply.

The maintained fixture separately qualifies exact name equality and closed
segment/rectangle contact against prefix/AABB candidates. Segment endpoints must
be genuine point regions in the fixture's declared integer length unit and XY
frame; non-point regions refuse. This numerical predicate supplies no CAD
topology or private adoption evidence.

Complete Public Parents over Native-maintained Sources can retain verified
candidate influence when their installed full Source/Parent projection profile
covers every input. `has_verified_realized_footprint()` on the observed row or
result set describes that posture; it grants no read or mutation authority.
The original complete candidate cut includes exact rejects and all projected
inputs, root predicates, related endpoints, ordering and negative coverage.

The ordinary `expect_source` and `expect_result_set` mutation entrances consume
these observations under the current request. At the actual target image, Query
uses the same installed forward provider and exact evaluator to verify complete
candidate coverage again. Equal membership, exact decisions and complete
candidate bodies can preserve an expectation across edits to noncandidates or
restoration of candidate content. Shared immutable bodies provide the fast path;
otherwise Query pays for exact occurrence, dependency and the complete declared
scalar/relation projection comparison. Commitments only shortlist that comparison. A previously nonmatching
record entering an empty range invalidates it. A changed candidate input prevents
equality even if the returned exact rows stay empty. Installed slot correspondence,
full-generation relation/row identities, predicate support and order remain part
of the comparison. Semantic restoration alone never proves canonical authored
history cancellation. Qualified retained managed facts can join ordinary recorded
correction through Native scope-impact candidates. For qualified Public Parents,
actual historical affected rows in both Source images can exclude nonmatching
edits using the installed forward predicate; exact refinement never removes a
matching candidate. Missing/expired material stays conservative. Every remaining
candidate needs genuine performed cancellation before strict original and complete
target comparison. The same ordinary correction path qualifies prefix and
indirect box/contact inputs; governed historical relevance stays conservative. See the
[correction guide](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#qualified-managed-source-support).
Independent ordinary field and adjacency revision facts still apply. Thus equal
restored managed content does not guarantee that every populated observation
remains eligible. Actual root-predicate reads keep their revision facts even
when their output is empty. The restoration positive qualifies exact-refinement-
rejected candidate support through a real mutation expectation; the separate
populated and root-filter-empty courts preserve their independent refusals.

The commit receiver retains and atomically pins the checked target Native Source
observation. A writer after revalidation refuses at the existing Native basis
check. No returned DTO, digest or old observation substitutes for that pin.
Unsupported projection profiles retain the conservative whole-Source posture.
A complete governed Parent also qualifies when the actual admitted governance
owner retains passive full capability-use and disclosure correspondence. That
capture checks its real session, capability, query/parameters, principal/scope
and exact installed Parent; it holds no old authorization or READ grant. Public
reads, watches and pages still require fresh READ and disclosure admission;
internal consistency verification exposes no rows.

Qualified semantic Source identity covers the complete candidate cut, full
projected content and exact dispositions, including rejected candidates. In the
Public producer pipeline, an unrelated Source edit can preserve genuine generated-
output reuse.
Every demand still admits fresh principal/scope/READ and disclosure, verifies the
retained output at the actual target, and keeps the original Native expectation.
A reused committed output can report `Performed` with its original historical
receipt and zero new producer contacts; `StableReused` is a separate stable-lineage
posture. Identity alone grants no reuse or publication authority.

A qualified complete Public managed Source also participates in the declared
canonical producer-input cutoff. Fresh actual selection compares every candidate
identity and exact decision, the outer field/relation membership and full admitted
query affinity. If canonical operation input and the installed producer remain
equal, the existing cutoff still verifies prior handler facts, decision context,
consumed outputs and the actual Native output witness. A real Source content
change can then yield `StableReused`, zero handler contacts and no new mutation
receipt. This does not make changed operation input reusable.

Managed Source reverse postings remain conservative. Such a stable alias retains
full verification and authentic output projection custody; it creates no clean
equality links or downstream discharges. Later relevant Source/output changes
still refuse delivery. Generation substitution cannot reuse a prior selected-membership key;
incomplete/restored or unqualified captures cannot supply one.

A governed observation preserves its ordinary omission receipts while retaining
hidden candidate influence. Its source expectation is descriptive input to the
separately admitted mutation; it supplies no current Query grant. Passive use
includes actual action/purpose/context and admitted disclosure meaning. Distinct
Public/Governed uses cannot collapse into one canonical Source obligation, and a
changed rejected body still refuses even when disclosed exact rows are unchanged.
Fresh grant revocation or independent READ refusal still stops the next public
query/watch before provider callbacks or metadata. Governed producer/output-demand
readmission and canonical input-cutoff integration remain unqualified here; the
selected governed execution mode still refuses that pipeline. The producer reuse
and cutoff results above apply to the supported Public pipeline.

The governed profile retains full projected candidate inputs; this conservative
support does not claim minimal per-field tracing. Governed history keeps all
scope candidates. The same correction court qualifies Public prefix and indirect
box/contact inputs, including exact-rejected matching candidate protection.
Successful regeneration after changed inputs depends on the declared producer
lifecycle; an Initial-only/Create operation cannot replace a prior live output.
Keep the observation until the mutation consumes it, then reread and reassess
if equality cannot be proved.
Reuse the operation's idempotency identity for a recorded retry; it does not
repeat the completed mutation. See the [currentness evidence](../plans/WORTH-query/milestone-9.19-phase4-currentness-evidence.md) and
[semantic reuse plan](../plans/WORTH-query/milestone-9.19-phase4-semantic-reuse-plan.md) and
[selected-membership plan](../plans/WORTH-query/milestone-9.19-phase4-membership-plan.md) and
[governed influence plan](../plans/WORTH-query/milestone-9.19-phase4-governed-plan.md).

The maintained prefix and geometry courts also exercise genuine equal-value
rename/move commits. A watch may remain Pending or deliver unchanged rows;
notification suppression is not promised. An empty read overlapped by a matching
entrant refuses stale registration, and a fresh opening includes the entrant.
Completed replay after unpublished recovery preserves the selected World,
provider maintenance counts and drained notification state; changed input under
the same key returns `IdempotencyIntentDrift`. See the
[Phase 3 qualification matrix](../plans/WORTH-query/milestone-9.19-phase3-closure.md)
for the supported contracts, current cost receipts and explicit later-phase work.

Use the request-bearing mutation entrances for indexed observations. A lower-level
conversion without an Execution request returns `ManagedSourceExpansionUnsupported`;
supplying an allocation policy alone does not make that conversion available.

An index belongs to its installed provider version, runtime, branch and
full-generation scope. Open a new registration when any of those changes;
a handle from another branch, installation or generation cannot be reused.
Keep result owners only as long as needed, and close the shared registration
when the application no longer needs the index.

Declare `ApplicationManagedIndexBinding::MAINTENANCE` as
`ApplicationManagedIndexMaintenance::Native { cold:
ApplicationManagedIndexColdSource::AdmitRebuild }` to require Native maintenance
for public reads even when no invariant consumes the index. Use the same
contribution `setup.managed_index::<Index, _>(provider)` entrance. The default
`WhenRequired` adds no public requirement; existing invariant requirements still
apply. When public and invariant audiences share an index, `Refuse` wins over
`AdmitRebuild`. Unsupported Source shapes return a typed installation refusal.
Initialization and updates retain the original request's controls and budget.

A Source and its Parent can declare the same finite nested direct-relation shape
with the existing `.relation(selector, child_builder)` surface. Forward and reverse
routes preserve every physical occurrence, including convergent paths and empty
child collections. Native parent-occurrence custody supplies the projected fields;
entity coordinates and caller-owned DTOs cannot issue them. Complete projected
candidate influence includes every declared descendant, even for rejected candidates.
Original work, memory and cancellation controls apply to acquisition, comparison
and nested Entry content. OptionalAspect Source fields are supported at the root
and in nested shapes, using ordinary `optional_aspect_field` projection and row
reading. Native distinguishes absent aspects, absent optional children and present
fields. Child predicates remain unsupported. Entry content includes the present
field-slot roster, so aspect addition and removal affect comparison. One-hop
entries use v1 framing; the compiled nested profile uses v2 framing, including
currently empty children.
See the executable [nested shape](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/nested/shape.rs)
and [ordinary integration courts](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/nested/court.rs).
This qualifies the finite nested Source profile, not arbitrary payload traversal,
new historical correction policy, or a new public CAD model.

Explicit maintenance is portable v5 meaning, with optional refinement. Existing
default definitions retain their v3/v4 bytes and canonical meaning. A one-shot
Parent can declare `ApplicationQueryBasisSupport::current_only()`; actual current
selection succeeds, while an explicit retained `.at(...)` read returns
`BasisUnsupported` even at the same unchanged head. Result rows still come from
one snapshot selected at admission. This does not enable historical managed
reads or continuation without pinned support. The
[public geometry consumer](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/public_only.rs)
uses a distinct maintained index with no invariant requirement. Bank integration
and the generic provider qualification are tracked in Query9.19 Phase5; real CAD
adoption belongs to Phase6.

The shared Query-free box/tree/segment-contact mechanics support both the signed
fixture `Box2` and unsigned `IntegerBox2<ModelLocalMetre>` domain. The raw
`{ lower, upper }` serde carrier retains its default i64 meaning. Typed domain
classes cannot be combined accidentally and do not establish model-instance
origin affinity. The concrete topology length binding retains explicit bounded
admitted UInt64 decoding, its original identity/wire/Metre/frame and nonzero
constructor, and original admission refusal. Strict managed reads never silently
fall back to an unadmitted decoder.

Phase5 uses Bank and the existing bounded provider fixtures for contract evidence.
The former public Scene/Face application has been removed; real CAD adaptation
uses its private numerical/topology/contact owners in Phase6. See the
[cleanup evidence](../plans/WORTH-query/milestone-9.19-phase5-public-cad-cleanup-evidence.md).
The [Phase5 contract matrix](../plans/WORTH-query/milestone-9.19-phase5-closure.md)
records the qualified public boundary, evidence provenance and remaining private gate.

A result-set/source expectation also retains the original Source READ support.
Revoking a Source-only permission refuses a later non-querying mutation before
publication. Recorded correction includes those policy fields and adjacency in
ordinary history selection, before managed predicate narrowing. Regranting READ
allows a fresh Source read; it does not erase uncancelled policy history from an
older recorded operation. Current-only handler reads preserve their actual
operation selection; retaining an equal snapshot does not promote it to current.
See the [Source authorization consumer](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/source_authorization.rs).

The
[public prefix consumer](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/journey.rs)
shows explicit construction, UTF-8 prefix reads, an ordinary scan oracle and
shared-handle cleanup; its
[provider adapter](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/prefix.rs)
uses the same finite storage contract as other providers. Prefix v1 uses
unnormalized, case-sensitive UTF-8 prefix matching. Construction consumes
canonical full-generation Source order; the Parent returns final rows in
identity order.

The prefix and closed-box adapters declare the same source/probe/provider
contract. Their probe codecs retain their complete canonical meaning:

<!-- compiled: managed_prefix_index.rs -->
```rust
impl ApplicationManagedIndexBinding<Catalog> for PrefixIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = super::super::schema::Item;
    const NAME: &'static str = "catalog_prefix_index";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const SOURCE_SCOPE:
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceScope =
        super::super::PREFIX_SOURCE_SCOPE;
    const MAINTENANCE: ApplicationManagedIndexMaintenance = super::super::PREFIX_MAINTENANCE;
    const REFINEMENT: Option<
        worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
    > = super::super::PREFIX_REFINEMENT;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP:
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        super::super::PREFIX_MEMBERSHIP;
}
```

<!-- compiled: managed_box_index.rs -->
```rust
impl ApplicationManagedIndexBinding<Catalog> for BoxIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = super::coordinates::Binding;
    type Entity = Item;
    const NAME: &'static str = "catalog_box_index";
    const PROVIDER: &'static str = "managed-catalog.closed-box2.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}
```

The box probe carries the numerical owner's explicit integer-length unit and XY
frame. Its admitted codec reconstructs coordinates; `Box2::new` rejects reversed
intervals before BVH navigation, preserving `BoxIndexDenial::Bounds` as the typed
provider cause. Overlap is inclusive, so a shared boundary is a match, and
distinct Source occurrences with equal bounds remain distinct rows. The
[box scan comparison](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/boxes/geometry.rs)
and [governed grant lifecycle](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/boxes/authority.rs)
show ordinary final ordering, explicit rebuild and separate capability/READ
revocation handling.

### Prepare candidate structural Source changes

An installed invariant's planner can acquire
`prepare_derived_relation_changes(&binding)` through the existing Native
before/after source owner. Retain the returned proof in the rule's `Scope`, then
use `context.derived_relation_changes(&binding, &prepared)` during evaluation.
Both calls use the genuine current candidate; a descriptor, entity coordinate
or previous proposal cannot replace that custody. The full recipe's declared
read and affected kinds govern preparation, and evaluation freshly checks its
reader audience. Structural inverse-prefix support includes empty adjacency.
Application code uses the ordinary `worth-query-host` audience. The
[internal engine handoff](../workspaces/worth-query/crates/worth-query/docs/modeling/schema-validation.md#candidate-structural-source-changes)
describes the corresponding Native boundary.

<!-- compiled: candidate_source_changes.rs -->
```rust
pub fn prepare_changes<S, R, F, T>(
    planner: &WorthQueryApplicationInvariantScopePlanner<'_, '_, S>,
    binding: &WorthQueryApplicationInvariantDerivedRelationBinding<S, R, F, T>,
) -> Result<WorthQueryPreparedInvariantDerivedChanges<S, R, F, T>, WorthQueryInvariantAccessDenial>
{
    planner.prepare_derived_relation_changes(binding)
}

pub fn read_changes<S, R, F, T>(
    context: &WorthQueryApplicationInvariantContext<'_, '_, S>,
    binding: &WorthQueryApplicationInvariantDerivedRelationBinding<S, R, F, T>,
    prepared: &WorthQueryPreparedInvariantDerivedChanges<S, R, F, T>,
) -> Result<usize, WorthQueryInvariantAccessDenial> {
    Ok(context
        .derived_relation_changes(binding, prepared)?
        .affected_roots()
        .len())
}
```

Use this loan for structural before/after pullback. It does not grant projected
scalar reads, managed-index queries or publication authority. Declare a managed
Source requirement separately when a rule needs a provider-backed read.

### Maintain a provider Source for an invariant

Add `.managed_index_source(Index::reference())` to the invariant definition.
Choose `managed_index_cold_sources(ApplicationManagedIndexColdSource::AdmitRebuild)`
when its original request may construct an unopened scope; `Refuse` instead
requires available material. Install the matching provider through the same
contribution as the index. Implement its `update` receiver using the supplied
before/after Source rows, changed occurrences and isolated node writer. Share
unchanged nodes; do not replace a warm update with a complete Source rebuild.
Provider and controlled-work refusals prevent publication.

The Source uses `SameAdmittedScope` or declares `SOURCE_SCOPE =
ApplicationManagedIndexSourceScope::OrderedPredecessor`, with `NoParameters`, public disclosure,
physical scalar fields and supported stored/reverse/composed/union membership
paths. Direct endpoint result relations may project scalar fields from the
related entity. Their actual relation occurrences preserve cardinality: two
physical links to the same endpoint are still two results. Finite nesting of
these direct relations, including root to child to grandchild, is supported.
Optional parent-aspect result fields are supported at every level through
`optional_aspect_field`; optional children use `optional_field`. Aspect addition
and removal change Source rows even when all surviving values stay equal.
Child predicates and child-field ordering remain unsupported; ordering may
refer to root fields. Unsupported shapes return `UnsupportedInvariantSource`
during installation.
Source result limits remain enforced; unavailable material is not an empty set.
Native discovers affected scopes from actual before/after changes, including
scopes with no open public handle, and prepares their roots before validation.
Applications do not supply an affected-scope list.

`OrderedPredecessor` requires exactly one forward `ordered_events` membership
root and a to-one order relation. Native derives the predecessor from that
relation; another membership or a non-to-one predecessor returns a typed
installation refusal. The retained provider program and version must match.
At birth, a successor shares immutable predecessor pages and applies its step
delta through copy-on-touch. The first scope builds its own membership. A missing
retained predecessor permits one cold successor build under `AdmitRebuild`, or a
typed refusal under `Refuse`; preparation never recursively builds predecessors.
Changed prepared successors use same-scope updates; unprepared successors cannot
be read as current. Every read retains independent branch, generation, image
and READ admission.

Inside a mutation handler, read a managed Parent through
`DecisionReader::index_query`. Declare the Parent's projected fields in the
operation's reads and register its exact binding with
`schema.operation_managed_index_read::<PrefixBinding, _, _>(RenameItem::reference())`.
Installation rejects a Parent whose scope mapping differs from that mutation;
execution requires the resolved scope to equal the operation's admitted scope.
Parent admission and the Source's own fresh principal, scope
and READ checks still run. The decision retains the same qualified candidate
influence or conservative whole-Source expectation described above, including
future entrants and empty membership, for the actual consuming candidate.
Ordinary row/result-set expectations use `expect_source` and `expect_result_set`.

<!-- compiled: managed_prefix_decision.rs -->
```rust
                let rows = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: input.directory.clone(),
                    prefix: input.prefix.clone(),
                });
```

Resolve the index with `resolver.typed_managed_index(Index::reference())` in its
installed rule factory. Retain `planner.retain_managed_index(&binding)` in the
rule's `Scope`, then use `context.managed_index(&prepared)` during evaluation.
Each committed/proposed query borrows that actual Native observation and original
request. A retained preparation from another observation cannot supply this loan.

<!-- compiled: managed_prefix_invariant.rs -->
```rust
        let pair = context.managed_index(scope).map_err(execution)?;
        let probe = query::Probe {
            prefix: "taken".into(),
        };
        let after = pair.proposed().query(&probe).map_err(execution)?;
```

An ordinary successful commit publishes the matching graph and provider image.
An admitted `open_index` attaches that maintained root as `Ready`; it does not
call the provider's cold build. A current read still performs fresh Source READ.
The standalone explicit-rebuild registration described above retains its own
Created/Ready lifecycle. Closing a public attachment does not remove material
still owned by an installed invariant or a retained Native/World basis.
The handle's request-free `evict` returns `InvariantOwned` for such an
attachment. To make its selected material cold, use
`request.query(parent).evict_index(&index)`. This entrance freshly admits the
Parent, validates the handle's runtime, installation, principal, scope and branch
incarnation, and funds copied cache paths through that request. Governed and
approved Parents use `evict_index_governed` and `evict_index_approved` with their
usual capability arguments. Cleanup performs no Source content read or provider
callback. A completed cleanup remains successful if cancellation arrives at the
outer request's final poll; pre-effect interruption still refuses.

A fresh content read of that cold image returns the typed
`WorthQueryManagedIndexRefusal::RootUnavailable` before provider query callbacks.
The installed invariant's `AdmitRebuild` policy permits the next affected candidate
to reconstruct from its actual Native committed Source before validation; it
still has to satisfy the original request and every rule. Explicit
`request.query(parent).rebuild_index(&index)` freshly admits the Parent and the
Source's independent Public READ, then rebuilds provider nodes outside Native
locks and refills that same selected Native cache. Governed and approved Parents
use `rebuild_index_governed` and `rebuild_index_approved`. An intervening eviction
or refill rejects the stale reconstruction before install; cancellation before
install preserves the previous canonical cache (a cold cache stays cold). A
completed install survives the outer request's final cancellation poll. This cold lane performs the ordinary Source query and
a second Native occurrence projection; both use the original request, and their
combined peak memory is reconstructive cost. It does not initialize a nonexistent
scope. Initial construction under `Cold::Refuse` uses the separate bootstrap
entrance below.
Eviction and reconstruction do not change a complete Source condition when its
actual support stays unchanged.

An invariant's default `Cold::Refuse` also refuses new scopes during ordinary
preparation. For a fresh unpublished installation, explicitly select each index
whose newly created scopes may be constructed in the next seed-batch attempt:

<!-- compiled: managed_prefix_initialization.rs -->
```rust
    graph.initialize_managed_index::<prefix::PrefixIndex>(phase)?;
```

Bind the ordinary principal/entity/relation seeds, then call
`graph.commit_seed_batch(phase)` or finish normal bootstrap publication. Initial
program activation still precedes those seeds, and all normal invariants judge
the canonical roots before the application becomes visible. Initialization does
not change the declared cold policy or grant public Source READ. A foreign typed
binding refuses even if its descriptive names match. The next attempted batch
consumes selection; an empty attempt returns `EmptyBootstrap` and poisons that
bootstrap. A nonempty unrelated batch also consumes unused selection. A recovered
bootstrap rejects initialization; an installed runtime has no initialization method.

Once that attempt ends, ordinary new/cold scopes still refuse under `Refuse`.
A fresh public `rebuild_index` can refill an existing cold association after its
independent Source READ; it cannot create a nonexistent scope. A refusing seed
commit preserves its original Source cause through
`denial.managed_source_denial()` and custom-rule verdict through
`denial.custom_invariant_denial()`. Failure does not turn an unavailable committed
view into absent membership. Original proof/session/basis backing releases with
its actual owners, beyond runtime guards.

Cleanup follows the current World product's genuine selected Native component.
A `ReuseExact` product can therefore evict and rebuild its selected older image while newer
main and fork images remain warm. Earlier Native/World images and observations
may retain their immutable temporal shares; eviction does not erase historical
material. Physical backing releases after its last relevant owner closes.

After its Native component deletion completes, closing the owning product also
releases its live managed Source lookup.
An existing `ReuseExact` alias retains its selected immutable component, so fresh
admitted reads, eviction and refill through that alias remain available. This
retained read custody grants no fresh Native branch admission, fork or write.
The [close and alias court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/court/close.rs)
checks the complete journey and final allocation balance.

See the [authored maintained catalog](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/host.rs)
and [its installed rule](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/rule.rs).

For an edge Source, declare the start and end relations in its result shape and
include both endpoint entities and relation kinds in the invariant's read and
affected closure. A vertex field edit then pulls back through those actual
relations to incident edge rows and every affected directory, on both physical
sides. Rewiring removes the former endpoint's dependency. The box provider keeps
its integer unit/frame binding and delegates validation and enclosure union to
its numerical owner. The [geometry composition](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry.rs)
uses this path with the same provider installation and both reader entrances.

The [whole-path growth court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/court/growth.rs)
authors 1,000/2,000-vertex chains and 50,000 genuine remote enclosures through
installed actions. Both affected directories contain the remote geometry; the
edited vertex keeps degree two and the fixed probe keeps sparse output. Complete
ordinary Source reads are checked against authored coordinates outside measured
work. The checked Source-tree and summary path passes this court's original
whole-request SDK growth guards, including Source acquisition, reverse-scope
discovery, provider updates, rules and publication. Before/after public probes
also verify authored membership with bounded summary comparisons. Cold setup is
admitted and invoiced separately; unchanged warm roots are never rebuilt.
This evidence covers the indirect geometry composition. Ordered-predecessor
chain qualification is separate from these same-scope prefix courts. Retaining a one-shot response does not register a
live predicate or qualify later retained-query routing.

A branch created through `ForkExact` attaches the selected immutable managed
Source under a fresh Native association for that branch. Subsequent edits on
either sibling maintain their own image. `ReuseExact` continues reading the
actual selected Native component even after sibling heads advance. Neither path
rebuilds the provider merely to attach an unchanged Source. A retained Source
expectation from another branch returns `ForeignBranch` before provider work;
consume the observation only through its original branch and fresh admission.
The [fork affinity court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/court/affinity.rs)
exercises these paths through the installed public program.

A validation failure's `invariant_execution_failure().and_then(|failure|
failure.managed_source_denial())` borrows the original Source preparation cause.
Its `provider_failure().cause()` preserves the owned provider or projection error
chain, including `WorthQueryApplicationProjectionDenial` with
`RelationCardinalityMismatch`. Request interruption remains a typed request stop.

Handle `BranchMaterializationSuspended` or `SnapshotUnavailable` from a current
physical read during generated-output suspension. Retaining an earlier result
does not make it current. After resumption, open the index through a fresh
admitted request and handle its returned lifecycle status; an old payload or
equal values cannot establish a new physical basis.
The [generated Catalog journey](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/court.rs)
uses a real fixed-delivery source action, producer and reconstruction receiver.
Suspension preserves logical invariant rows in both affected scopes while current
public reads refuse. Reconstructing the exact manifest still runs the Source
rules on the proposed values; the valid reconstruction publishes material used
by fresh public reads. A retained result keeps its original values throughout.
For an exact reconstruction, Native compares the actual authoritative manifest
records with their retained suspension source before the first rule. It can then
share the selected suspension image's immutable Source backing with a fresh
available association. Query still checks the actual provider installation and
World lineage; every required rule runs, and committed suspended Source reads
remain unavailable. This conservative proof declines even an unprojected field
change, which follows the declared cold-source policy. The same journey checks
zero provider build/update calls on exact restoration, an ItemBounds change that
rebuilds both scopes, and original-request cancellation after the first real rule
query followed by a fresh retry.
The [generated endpoint journey](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/court.rs)
publishes an Edge and both Vertices as declared output roles, including both
Directory memberships and the real Starts/Ends relations. Full-range rule queries
check the complete logical roster during suspension; changing a reconstructed
endpoint reaches the existing enclosure rule in unopened B. Exact reconstruction
reuses the same payload without provider build/update. Cancelling the original
suspension request after a real rule query preserves selection and available
public reads, then a fresh suspension succeeds. Use
`WorthQueryGeneratedOutputSuspensionFailure::execution_control_stop()` to borrow
its typed request stop; a preparation failure still owns its original Native
denial. These are integer enclosure and Source lifecycle checks.
The [cold generated Source court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/court/cold_history.rs)
evicts previously attached B, closes its public registration and proves its
derived material stays cold while ordinary Native Source rows remain readable.
Original cold-build cancellation reaches no rule; retry reconstructs B before
both scope callbacks. This is separate evidence from never-opened B.

A fresh `request.at(&observation).query(source).execute()` still resolves its
principal and scope on the current product. During suspension it preserves
`BranchMaterializationSuspended`; historical data cannot replace current security.
After lawful endpoint-20 restoration, a fresh retained ordinary Source query
returns the original endpoint 10 while current reads return 20. Managed Parent
queries in retained mode continue to refuse `UnsupportedMode` before provider
callbacks. Retaining decoded rows is a separate immutable-result property.
The [pending materialization court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/court/pending.rs)
uses the existing World comparison pause after real Native work. While publication
holds activation admission, fresh selection returns `ProductActivationUnavailable`
before disclosure. Cancelling there returns the actual `ProductUnpublished`
suspension/restoration recovery carrier. Once the entry returns, fresh reads use
the predecessor World image: available endpoint 10 for pending suspension, or the
suspended principal refusal for pending restoration despite Native being ahead.

Continue the returned recovery with a fresh request. A cancelled continuation
returns its carrier through `into_recovery()`; successful continuation adopts the
settled Native result without rebuilding provider data or rerunning rules. The
court restores endpoint 20 and independently reads retained ordinary endpoint 10.
This qualifies process-local pending recovery, separately from a competing
publisher race or crash persistence.
The compiled endpoint shape preserves each actual `Starts` and `Ends` relation
occurrence and checks its declared cardinality before provider maintenance:

<!-- compiled: managed_geometry_source.rs -->
```rust
pub(super) fn shape<Q: ApplicationQueryMarkerIdentity<Catalog>>(
) -> TypedApplicationQueryResultShape<Catalog, Q, Edge, Row, ResultBinding> {
    ApplicationQueryResultShapeBuilder::new(Edge::reference())
        .field(identity())
        .relation(
            start(),
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Vertex, Row, ResultBinding>::new(
                Vertex::reference(),
            )
            .field(start_region()),
        )
        .relation(
            end(),
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Vertex, Row, ResultBinding>::new(
                Vertex::reference(),
            )
            .field(end_region()),
        )
        .build()
}
```

An unavailable maintained Source returns its typed Native Source refusal.
Logical invariant material never authorizes a current physical public read.

### Ordinary Query resource policies

Output-demand, performed-output, discovered-output and workflow assessment settlement
helpers continue owner-issued `Progressed` under the same request when the step
policy is absent. `Pending` parks for a later call rather than retrying a waiting
owner. Explicit finite step controls return `Progressed` at their boundary;
`advance` remains a step. Both unfinished outcomes retain the original owner;
only `Settled` carries settlement authority.

Ordinary `WorthQueryOutputDemandResourceProfile::standard()` (also `Default`)
advances native `Progressed` outcomes until settlement or genuine waiting.
`new(settlement_attempts)` supplies an explicit finite convenience-step policy.
Retain the returned owner through waiting, cancellation and normal close.

Prepare mutation values and native targets in the decision, then consume them
through the installed writer. Declare required output roles and families with the
action so successful execution can settle its complete output.

`WorthQueryApplicationQueryResourceProfile::new(maximum_concurrent_graph_work)`
controls native graph-work concurrency. Supply the installed query's inputs and
handle its result cardinality under fresh request admission.
`request.query_batch(intents).execute(&phase)` issues exact installed read slots;
foreign plans, unscheduled reads and a second planning extension still refuse.
Each read receives fresh authorization. A result's shared backing or schedule
association grants no source permission or currentness.

Parallel pair reads share the request's execution policy. Handle interruption
and first-stop denials while preserving the returned canonical prefix. Platform
callers with an explicit bounded partition contract can use
`from_keyless_partitions`.

Use `SignalConditionalEvaluationBudget::development()` and
`BridgeConditionalRetentionBudget::development()` for development installations.
Supply `Some` values for deliberate finite policies and keep the sealed Signal
owner policy consistent across branches. Retain shared backing until its final
holder drops, and supply prepared node/root footprints before copying or publication.

With no settlement-step policy, convenience settlement drives the native
advancement through genuine `Progressed` answers on the same request and parks
on `Pending`. It does not spin retrying blocked
work. A subsequent advancement uses a fresh authorized request; explicit finite
step controls preserve their limit. Keep original unpublished-idempotency and
branch-commit owners through recovery and cleanup, then release their backing
when their obligations are complete.

Declared query roots can traverse complete endorelation reachability with
`ApplicationQueryRootPath::forward_closure` and `reverse_closure`, composed with
ordinary hops and guards. See [declared query closure](../workspaces/worth-query/crates/worth-query/docs/foundations/declared-query-closure.md)
for reflexive selection, source currentness and per-step receipt observations.


### Derived query root paths

Use a declared named derived relation to select physical entity roots:

<!-- illustrative -->
```rust,ignore
// Illustrative application marker names; the consumer court compiles this shape.
let path = ApplicationQueryRootPath::from(Event::reference())
    .forward_derived(LiveRows::reference());
```

`reverse_derived` selects in the other direction. Compose ordinary query paths
with physical hops, other derived steps and guards. Maintained Sources retain
their existing guard and predicate restrictions. The installed path retains the
Stored or Derived namespace, so a same-named stored relation cannot satisfy a
derived step. This entrance requires physical entity endpoints and physical
intermediate source domains. Derived closure steps are refused; a named recipe
already defines its complete source meaning.

For newest-first history, declare `LiveRows` with
`ApplicationDerivedRelation::ordered_events(previous, introduces, retires)`.
The same registered Native evaluator supplies ordinary query roots and maintained
Source membership. A history edit can change a direct Source even when every
rendered row stays identical: complete physical support and negative probes are
part of that observation. Empty Sources retain their negative membership too.
Watches and request-bearing Source expectations use those actual dependencies;
a fresh observation remains the lawful input after a stale one is refused.

Original pinned reads retain their selected history. New reads and currentness
checks retain the original request controls and allocation custody, including
cancellation and failed-acquisition work receipts. This query entrance does not
extend operation-body path-input or component-read grants.

The ordinary Decl/Host event-text [consumer court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/events/court.rs)
exercises forward, inverse, composed, empty and changed-history Sources.


## Inspect original source and query evidence

Use inspection for an audit view, a debugging explanation, or a comparison of
what two executions returned. It reads saved evidence, so a later write does not
replace the result you are explaining. Each new request checks current access.

The entry points are `request.inspect::<ReadBinding>(&original)` and
`request.inspect_source::<SourceDisclosure>(&source_receipt)`. Their request and
result types are in `worth_query_host::facade::causal_inspection`; declarations
use `worth-query-decl`.

Keep the original before dropping a query result. Opt into its values when you
will compare or materialize them:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let result = request.query(Read { key })
    .retain_projection_for_inspection().execute()?;
let original = result.receipt().inspect().observation()
    .ok_or(AppError::ObservationUnavailable)?.clone();
drop(result);
let report = fresh.inspect::<ReadBinding>(&original)
    .why_observed().reference_only().execute_governed(query_metadata)?;
let availability = report.evidence_availability();
report.close();
original.close();
```

`Read`, `ReadBinding`, `AppError` and the capability arguments belong to your
application. Inspection consumes freshly admitted metadata/diagnostic access;
a payload query's refs-and-input `execute_governed` call is a different interface.
See [admit inspection access](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#admit-current-inspection-access)
for the Host calls that create those access values.

| Question | Request |
| --- | --- |
| What was this result? | `why_observed().reference_only()` |
| Did two saved results differ? | `why_changed(&before).reference_only()` with diagnostics |
| Did this source create the observed output? | `why_observed().caused_by::<SourceDisclosure>(&source_receipt)` |
| Did a producer compute changed or equivalent output? | `why_observed().evaluated_by::<SourceDisclosure>(&source_receipt)` |
| What did an authored operation record? | `inspect_source::<SourceDisclosure>(&source_receipt).why_performed()` |

For a producer explanation, query through
`request.at(settlement.observation()).query(...)` so the original carries that
settlement's supplier association. Keep the earlier authored source receipt for
`caused_by`/`evaluated_by`; the producer publication receipt is a different object.
A plain head query does not infer a producer from equal values.

Add `.materialize(detail)` when you need original values. Select
`WorthQueryApplicationMaterializationRequest::family(WorthQueryApplicationEvidenceFamily::QueryInspection)` for the
query projection, `source_effects()` for source-only effects, or combine
`.with_source_effects()`, `.with_producer_evidence()` and
`.with_conditional_history()` on a connected request. Admit query and source
metadata/diagnostics independently. Equivalent output preserves its observed
revision and cannot claim a new creation.

Handle unavailable and unrequested detail explicitly. Availability is a snapshot,
not a retention lease: optional history can be evicted before a later rich read.
Request cancellation or revoked access refuses the new inspection; neither expires
an immutable original. Close reports and the final original holder when done.

Follow [Explain original query results and source operations](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md)
for contract setup, capability admission, supported shapes, materialized values,
loss outcomes and retry handling.


## Read Bank accounts through the installed managed provider

Bank exposes `runtime.query(queries::current_accounts()).as_principal(actor)`
through the same ordinary query receiver and `BankReadControls` as its existing
reads. Supply current controls and call `execute()`; result rows remain
`VisibleAccount` ordered by AccountId. The [compiled receiver journey](../workspaces/worth-query-bank-world/crates/worth-query-bank-court-external-rail-server/tests/worth_query_bank_court_external_rail_server/server/ordinary_reads/current_accounts/mod.rs)
compares these rows with the independent ordinary `queries::accounts()` Source.

The Parent declares current-only basis support and references the unchanged
principal-specific v1 account-discovery Source. Both reads admit the actual
principal and `DiscoverOwnAccounts`; the managed root supplies no READ grant.
Personal ownership, authorized-user and business-owner paths retain their real
Bank definitions. Grant/revoke and actual account creation update the affected
scope through Native maintenance. Stable SourceSlot order permits checked linear
cold construction; warm changes use logarithmic tree insertion/removal. Returning
all accounts still visits every returned Source row and pays the existing
ordering and source-meaning bookkeeping costs.

Use `queries::accounts()` for its existing pinned-read contract. Applying a
retained basis to `current_accounts()` refuses as `BasisUnsupported`, including
an equal-head retained basis. The managed Parent preserves the original result
limit and cancellation controls. Pending-payments, activity ranking and cursor
receivers keep their existing contracts; their behavior is covered by their
ordinary Bank courts and is not supplied by this all-accounts index.

### Published roles and cascade closure evidence

A settled producer's `outputs_of::<Contract>()` reports the roles in that route's canonical Native publication. A shrink may also require Retire witnesses for publications owned by the deleted children. Those witnesses are admission evidence, and are exposed separately through `root_settlement().closure_evidence_of::<Contract>()` and its typed `family_entries::<Family>()`. A closure entry describes a retired identity; it is not an output-role entity and grants no current identity or producer custody. Only the canonical roles participate in producer correspondence and cold readmission. The closure evidence describes this host's performed admission and adds no durable replay obligation. A program may refresh a producer again before it delivers its final root settlement. Each publication has its own correspondence and evidence; the final settlement does not accumulate witnesses from earlier publications in that wave.

Source-scope-owned output cannot be physically suspended independently. The ordinary suspension door refuses `OwnedPublicationRequiresCascade` before publication; the output remains readable and its producer remains settled. Independent output retains the ordinary suspension and exact rematerialization path.
