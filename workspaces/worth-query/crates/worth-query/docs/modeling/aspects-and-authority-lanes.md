# Aspects And Authority Lanes

## What This Feature Is

Aspects describe stable slices of domain meaning. Authority lanes describe
which runtime is allowed to own or change a piece of state. Together they let
Query express exact dependencies without confusing authoritative truth with
derived computation or delivery state.

Use this model whenever a read, conditional operation, live surface, effect,
preview, or lower-runtime integration needs to say both “what data matters”
and “who owns it.”

## Why You Use It

- Keep field, relation-endpoint, structural, and lifecycle meaning explicit.
- Invalidate only the computation whose declared semantic slice changed.
- Keep derived Signal versions separate from Relational truth.
- Prevent preview, delivery, temporal, and async state from masquerading as
  authoritative domain state.
- Preserve enough meaning for exact inspection and early denial.

## Stable Entry Points

Portable semantic meaning comes from `worth_foundational::facade`:

- `AspectContract`
- `AspectKey`, `AspectIdentity`, and `AspectContractRevision`
- `AspectMask<ProjectionMask>`
- `AspectBinding`
- `AuthoritativeAspectChangeKind`
- `AspectValue` and contract-validated value artifacts

Query authoring and installation use:

- `worth_query_decl::facade::worth_query_aspect!`
- `domain::WorthQueryInstalledApplicationSchema::native_contracts`
- `domain::WorthQuerySemanticTruthDependency`
- `domain::WorthQueryOperationNativeProjectionContract`
- `domain::WorthQueryConditionalEvaluationCondition`
- `domain::WorthQueryConditionalNodeOutput`
- `runtime::WorthQueryAuthorityLane`

Relational publication and Runtime Bridge use:

- `worth_relational::facade::publication::PublishedAuthoritativeAspectChange`
- `worth_runtime_bridge::facade::BridgeSemanticDependencyCandidate`
- `worth_runtime_bridge::facade::BridgeInstalledSemanticCorrespondence`
- `worth_runtime_bridge::facade::BridgeSemanticAspectChange`

Signal uses its own runtime-local `Aspect` slots. Those slots are not portable
semantic aspect identities.

## Core Mental Model

There are three related but distinct layers:

### Semantic aspect

A semantic aspect is portable domain meaning. Its identity includes a stable
key, opaque identity, contract revision, shape, and absence/evolution law.

Examples include a vertex identity struct, a face normal vector, a relation
endpoint, or a body lifecycle state.

### Authoritative aspect change

Relational interprets a committed patch against schema and publishes the exact
meaning of the change:

- whole aspect set or clear
- field set or clear
- relation source or target endpoint
- structural create, update, delete, or retained-for-audit
- lifecycle create, delete, or retained-for-audit
- opaque change when precise interpretation is unavailable

The publication retains contract identity, revision, binding, optional field
path, change kind, and precision.

### Signal aspect

A Signal aspect is a runtime-local numeric slot on an installed Signal node.
It carries invalidation and version state. Its meaning is valid only inside
the exact Signal graph, node, partition, and installed lowering that owns it.

Runtime Bridge installs the correspondence between semantic aspects and Signal
aspects. Equal slot numbers do not imply equal meaning.

## Authority Lanes

The public lane vocabulary describes ownership:

- `AuthoritativeTruth`
- `BranchLocalTruth`
- `PreviewTruth`
- `DerivedRuntimeState`
- `EffectDeliveryState`
- `PendingWriteIntent`
- `BridgeExternalState`
- `TemporalExecutionState`
- `AsyncResourceState`

An aspect contract does not select an authority lane by itself. The admitted
runtime feature does. A face-normal contract can describe authoritative stored
truth in one operation and derived runtime output in another.

## Small Example

Use Foundational constructors so the contract carries canonical shape and
evolution meaning:

```rust
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, FieldDeclaration, FieldKey, FieldRequirement,
    ScalarAspectType, StructAspectShape,
};

let contract = AspectContract::struct_aspect(
    AspectKey::new("vertex-identity")?,
    AspectIdentity(0x9140_0001),
    AspectContractRevision(1),
    StructAspectShape::new([
        FieldDeclaration::new(
            FieldKey::new("id")?,
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )?,
    ])?,
);
```

The opaque identity and revision are not display labels. They are part of the
cross-runtime contract.

## Declare Application Aspect Identity Once

Application schemas declare the same Foundational identity and revision at the
Query entry boundary. Both values are mandatory in the stable macro form:

```rust
use worth_query_decl::facade::{
    application_schema::{AspectContractRevision, AspectIdentity},
    worth_query_aspect,
};

worth_query_aspect!(
    pub AccountFacts in BankingSchema, Account;
    identity = AspectIdentity(0x9161_1001),
    revision = AspectContractRevision(1),
);
```

Installation validates that declaration and retains one exact native contract
per `(schema binding, entity, aspect)` locus. Hosts inspect the sealed catalog
through `installed_schema.native_contracts()`, including exact lookup with
`aspect(entity, aspect)`, iteration with `contracts()`, and the declared
identity range. They do not allocate an identity, rebuild a contract from its
name, or copy provider-runtime schema into application meaning.

The catalog compiles once during installation. Every installed application
operation's native projection and field-touch contract borrows its meaning
from that catalog. Execution may separately install provider-internal runtime
schema for physical work; that schema is execution-owned and is never an
alternate identity source for the application contract.

## Bind Semantic Meaning To Relational Truth

`AspectBinding` says where the contract appears in authoritative graph-shaped
truth:

```rust
let binding = worth_foundational::facade::AspectBinding::EntityField {
    field: worth_foundational::facade::FieldKey::new("id")?,
};
```

Other bindings cover relation fields, relation endpoints, structural regions,
partitions and facets, and lifecycle transitions.

The binding is semantic. It is not a physical table/column address and does
not expose a storage backend.

## Select The Exact Slice

Use `AspectMask<ProjectionMask>` to select the part of the contract that the
operation reads:

```rust
let mask = worth_foundational::facade::AspectMask::new([
    worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new("id")?,
    ),
]);
```

A whole-aspect mask and a one-field mask are different semantic dependencies.
Do not widen one into the other because a backend cannot provide precision.
The runtime must either admit declared widening or deny the correspondence.

## Real Example

The operation’s native projection records the contract identity and mask. A
conditional dependency additionally records Relational binding, locality,
relevant change kinds, and graph-read role:

```rust
let dependency = domain::WorthQuerySemanticTruthDependency::new(
    domain::WorthQueryConditionalGraphReadRole::new("model")?,
    contract,
    mask,
    binding,
    domain::WorthQuerySemanticLocality::SourceRecord,
    [worth_relational::facade::schema::RelationalAspectChangeKind::FieldSet],
)?;
```

The enclosing operation graph-read contract must already authorize this
projection. The dependency cannot mint graph-read authority.

## How It Executes

The production path is:

```text
Relational commit
  -> schema-aware authoritative aspect change
  -> Bridge semantic change envelope
  -> installed semantic correspondence lookup
  -> exact or declared-widening delivery
     -> direct truth admitted by Query when no Signal work is required
     -> or producer-local Signal invalidation and performed execution evidence
  -> Query impact admission and query-shaped maintenance
  -> current consumer publication
```

Relational does not allocate Signal slots. Signal does not interpret
Relational field paths. Query does not decide whether a Signal computation is
semantically clean. Runtime Bridge is the only place where the installed
relationship is admitted and retained. A direct Bridge truth delivery and a
performed Signal consequence are distinct facts; neither may impersonate the
other.

## Exact And Widened Correspondence

Successful correspondence has one of two precision postures:

- `Exact`: the authoritative change meaning and Signal invalidation target
  preserve the declared slice.
- `DeclaredWidening`: a broader invalidation is permitted by explicit
  installation policy and reported as such.

Unsupported precision, ambiguity, mixed graphs, target collision, capacity
exhaustion, stale installation, and rebind requirements are non-success
outcomes. They do not produce an installed witness.

## Ordinary Computed And Effect Surfaces

Ordinary computed and effect declarations also use aspects to state reads,
outputs, and trigger slices. Their inspection surfaces report authority lanes
such as `DerivedRuntimeState` and `EffectDeliveryState`.

Use installed operation dependencies when the meaning belongs to a canonical
domain operation and must cross Relational, Bridge, and Signal. Use ordinary
computed/effect aspects for runtime-local application declarations that do not
need installed operation identity.

Do not translate between the two using string naming conventions. When the
same semantic aspect crosses the installed boundary, carry its Foundational
contract and binding.

## Ordinary Runtime Example

> **Internal engine surface.** This section uses `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

Ordinary computed declarations keep aspect meaning and authority-lane meaning
separate:

```rust
let titles = workspace
    .computed(
        "tasks.titles",
        |c| {
            c.depends_on_live(&live)
                .reads(["title.value"])
                .produces(["title.summary"])
        },
        TitleListMaintainer,
    )
    .unwrap();

let state = workspace.state(&titles).unwrap();
assert_eq!(state.authority_lane().as_str(), "derived-runtime-state");
```

`reads` and `produces` state semantic dependency. The state snapshot reports
that the result lives in `DerivedRuntimeState`; neither role substitutes for
the other.

Effects and previews preserve the same distinction:

```rust
use worth_query::facade::runtime::WorthQueryPreviewOptions;

let readiness = workspace
    .computed(
        "workflow.readiness",
        |c| {
            c.depends_on_computed(&titles)
                .reads(["validation.state"])
                .produces(["readiness.state"])
        },
        WorkflowReadinessMaintainer,
    )
    .unwrap();

let publish = workspace
    .effect("workflow.publish-readiness", |e| {
        e.when_computed(&readiness, ["readiness.state"])
            .condition_expression(
                "expr.ready-to-publish",
                ["readiness.state"],
                ["delivery.publish"],
            )
            .deliver("workflow.delivery")
            .meaningful_change_suppression()
    })
    .unwrap();

let preview_label =
    worth_query::facade::runtime::WorthQuerySessionLabel::scoped_strs(
        "workflow",
        ["approval-preview"],
    )
    .unwrap();
let mut preview = workspace
    .preview_with_options(
        preview_label,
        WorthQueryPreviewOptions::sandboxed_write_intent(),
    )
    .unwrap();
let preview_binding = preview.use_effect(&publish).unwrap();
```

The source truth remains `AuthoritativeTruth`; computed readiness is
`DerivedRuntimeState`; ordinary delivery is `EffectDeliveryState`; and the
sandboxed binding uses `PreviewTruth` plus `PendingWriteIntent`. Inspection of
the retained handles reports those lanes and trigger aspects without promoting
preview or delivery state into authoritative truth.

## Aspect Publication Across Declaration Boundaries

Aspect meaning also travels through declaration progression, foundational
evidence, route plans, receipts, envelopes, Relational routing, Runtime Bridge
correspondence, and Signal admission. Each boundary consumes the retained
semantic slice:

- envelopes publish the public semantic slice;
- Relational routing lowers only the authoritative binding and change kinds;
- Runtime Bridge freezes the exact or declared-widening correspondence;
- Signal admission checks dependency and produced-aspect posture against that
  installed target before execution;
- Query retains the resulting provenance instead of rebuilding aspect meaning
  from lower-runtime identifiers.

Aspects therefore carry load-bearing meaning across declaration, runtime,
Relational, Bridge, and Signal surfaces. They are never optional decoration.

## How It Relates To Other Features

- Runtime-installed operations retain semantic projections and dependencies
  as canonical operation meaning.
- Conditional nodes consume those dependencies but cannot widen graph-read
  authority.
- Relational publication owns committed change interpretation.
- Runtime Bridge owns correspondence and delivery precision.
- Signal owns producer-local scoped invalidation, readiness, local aspect
  versions, evaluation decisions, and performed execution receipts.
- Ordinary computed and effect surfaces use the same aspect/authority
  distinction without requiring installed operation identity.

## Inspection And Debugging

When a conditional operation did not run or invalidated too broadly, inspect
in this order:

1. Query dependency contract identity and revision.
2. Projection mask and graph-read role.
3. Relational binding, change kind, field path, and precision.
4. Bridge correspondence precision, target count, graph instance, and
   allocation evidence.
5. Signal node/aspect decision counters.
6. Query `conditional_provenance()` and execution counters.

For ordinary runtime surfaces, inspect the handle through
`workspace.inspections()?.inspect(...)` and verify the reported authority lane
and aspect contract.

## Anti-Patterns

- Treating aspect names as optional labels.
- Deriving an application aspect identity from a Rust type name, string hash,
  installed slot, or provider-runtime schema.
- Reconstructing an aspect contract beside
  `installed_schema.native_contracts()`.
- Using a numeric Signal aspect as portable domain identity.
- Authorizing delivery from a bridge stable name.
- Dropping contract revision or field mask at a crate boundary.
- Treating derived runtime state as authoritative truth.
- Letting preview or branch-local truth write directly to the authoritative
  lane.
- Interpreting Relational changes in Query or Signal.
- Silently widening field or endpoint changes to a whole aspect.

## Current Limits

- Temporal and async authority lanes require a support profile that admits
  their runtime behavior.
- Runtime-local Signal slot allocation is intentionally non-portable.
- Correspondence is installed before runtime publication and is tied to the
  exact Query runtime generation and Signal graph instance.
- Current sharing, granular invalidation, and query-shaped patch surfaces
  preserve the same semantic identities and decision provenance. Replay and
  reconstruction remain separate certification or recovery lanes rather than
  ordinary maintenance authority.

## Related Docs

- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
- [Granular Live Invalidation](../runtime-surfaces/granular-live-invalidation.md)
- [Native Aspect Values](../capabilities/native-aspect-values.md)
- [Computed](../runtime-surfaces/computed.md)
- [Effects](../execution/effects.md)
- [Branches And Previews](../foundations/branches-and-previews.md)
- [Inspection](../capabilities/inspection.md)
