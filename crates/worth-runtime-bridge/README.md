# worth-runtime-bridge

`worth-runtime-bridge` is the causal protocol boundary between authoritative
Relational truth and derived Signal computation.

Use it when one runtime must translate committed semantic changes, snapshots,
branches, history, or writeback across that boundary without giving either
side the other side’s authority.

For Query-installed conditional operations, application and domain code starts
at `worth_query::facade::domain`. Query runtime construction supplies the
Bridge, Signal graph, semantic correspondences, and volatile providers. Domain
packages do not call Bridge APIs directly.

## What The Bridge Owns

The Bridge owns:

- deterministic truth-change routing
- authoritative source and snapshot binding
- installed correspondence from semantic truth dependencies to exact Signal
  graph, node, partition, and aspect targets
- exact or explicitly widened delivery precision
- lowering portable conditional meaning into installed Signal contracts
- branch-local speculation and promotion boundaries
- causal diagnostics and boundary receipts
- the Relational adapter, `RuntimeBridgeRelationalSource`, which implements
  the Bridge's source contracts over `worth_relational::facade::change_source`

The Bridge depends on Relational and Signal. Neither depends on the Bridge.

It does not own:

- Relational truth or schema interpretation. The adapter lowers a
  `RelationalChangeReceipt`, which only Relational can mint; it never decides
  what a commit changed.
- Query operation or workflow meaning
- Signal aspect versions, scheduling, condition decisions, or compute results
- application policy outside an installed provider contract

## Stable Entry Point

Use:

```rust
use worth_runtime_bridge::facade::*;
```

The facade exposes both the standard standalone Bridge workflow and the exact
installation surfaces Query uses. Do not import implementation modules.

## Standard Standalone Path

```rust
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode,
    MappingSelector, RuntimeBridge, SignalInvalidationScope, SnapshotReadContract,
    TruthCommitIdentity, TruthPatchScope,
};

let bridge = RuntimeBridge::builder()
    .with_truth_source(truth_source)
    .with_truth_branch_head_source(branch_heads)
    .with_compute_sink(compute_sink)
    .register_mapping(BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("pricing:catalog"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("item:widget"),
            AspectKey::new("cost").expect("valid aspect key"),
            FieldKey::new("usd".to_owned()).expect("valid field key"),
        ),
        SnapshotReadContract::scalar(
            AspectKey::new("cost").expect("valid aspect key"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::from_stable_name("order:total"),
        CoarseRoutingMode::Direct,
    ))
    .build()?;

// `commit_id` is the `u64` id of the Relational commit that carried the change.
let route = bridge.route(TruthCommitIdentity::from_relational_commit_id(commit_id))?;
let evaluation = bridge.evaluate_current(route.target())?;
let explanation = bridge.diagnostics().explain_last_route();
```

The standard path remains build, route, evaluate, and inspect.

## Installed Semantic Correspondence

Query-installed operations declare semantic dependencies using Foundational
aspect contracts, masks, Relational bindings, locality, and relevant change
kinds. Runtime construction translates each declaration into a
`BridgeSemanticDependencyCandidate` and pairs it with one or more
`BridgeSignalAspectTargetDeclaration` values.

`BridgeSemanticCorrespondenceRegistration` and the Query runtime installation
path admit:

- the exact Query runtime and installation generation
- the exact graph participation authority and adapter
- authoritative source profile and semantic precision
- one Signal graph instance
- canonical, non-empty Signal targets
- actual Signal node/aspect capabilities

Success produces `BridgeInstalledSemanticCorrespondence`. Its precision is
`Exact` or `DeclaredWidening`. Unsupported precision, ambiguity, mixed graphs,
capacity exhaustion, stale generations, and rebind requirements remain typed
non-success outcomes and mint no witness.

Runtime-local Signal slot allocation never becomes portable semantic identity.

## Conditional Lowering

The conditional installation surface uses:

- `BridgeConditionalInstallationRequest`
- `BridgeConditionalProviderSet`
- `BridgeInstalledConditionalLowering`
- `BridgeOwnedSignalRuntime`

The provider set holds volatile mechanics for typed domain conditions,
temporal wakes, on-demand triggers, dependency comparison, output comparison,
and artifact reuse. Missing or extra providers deny installation.

At execution, the Bridge obtains semantic observations from the exact snapshot,
invokes Signal’s installed conditional contract, and returns
`BridgeConditionalDecisionEvidence`. The Bridge does not reinterpret or
restamp Signal’s decision.

## One Signal Graph

`BridgeOwnedSignalRuntime` couples one Runtime Bridge with one Signal graph for
Query conditional execution. Correspondence and lowering must target that
graph instance.

Do not create a parallel application graph for the same Query operation. If a
separate graph has genuinely independent authority, declare it as Query graph
participation and bind it explicitly.

## Authoritative Change Delivery

The Bridge accepts canonical semantic change envelopes. Each change retains:

- aspect key, identity, and contract revision
- Relational binding
- change kind and optional field path
- exact or declared-widening precision
- authoritative source and commit identity

Delivery matches this material against the installed correspondence before it
updates Signal versions. Stable names, equal numeric slots, or diagnostic
digests cannot authorize delivery.

## Rebuildable Indexes

Correspondence registries keep authoritative registrations separately from
allocation indexes. Use
`RuntimeBridge::rebuild_correspondence_allocation_index()` to verify exact
reconstruction parity.

A rebuilt index must preserve admission, target allocation, denials, and exact
counters. The index is acceleration, not authority.

## Anti-Patterns

- Using a mapping label as semantic aspect identity.
- Accepting raw Signal aspects from portable Query declarations.
- Re-deriving what a Relational commit changed from raw patch fields instead of
  lowering the change receipt Relational minted.
- Silently widening semantic change precision.
- Re-deciding Signal condition eligibility in the Bridge.
- Building a second Signal graph for the same Query runtime.
- Treating diagnostics or rebuild reports as admission authority.

## Documentation Map

Getting started:

- [`QUICKSTART.md`](./QUICKSTART.md)
- [`DAILY_WORKFLOWS.md`](./DAILY_WORKFLOWS.md)
- [`API_OVERVIEW.md`](./API_OVERVIEW.md)
- [`DIAGNOSTICS.md`](./DIAGNOSTICS.md)
- [`REFERENCE_MAP.md`](./REFERENCE_MAP.md)

Core concepts:

- [`ROUTING_AND_EVALUATION.md`](./ROUTING_AND_EVALUATION.md)
- [`BRANCHING_AND_SPECULATION.md`](./BRANCHING_AND_SPECULATION.md)
- [`WRITEBACK_AND_PROMOTION.md`](./WRITEBACK_AND_PROMOTION.md)
- [`HISTORY_AND_REPLAY.md`](./HISTORY_AND_REPLAY.md)

Advanced integration:

- [`SEMANTIC_CORRESPONDENCE_AND_CONDITIONAL_EXECUTION.md`](./SEMANTIC_CORRESPONDENCE_AND_CONDITIONAL_EXECUTION.md)
- [`RUNTIME_POLICY.md`](./RUNTIME_POLICY.md)
- [`CHANGE_STREAMS_AND_SOURCES.md`](./CHANGE_STREAMS_AND_SOURCES.md)
- [`MAPPING_CONTINUITY_AND_REMAP.md`](./MAPPING_CONTINUITY_AND_REMAP.md)
- [`MERGE_AND_STRUCTURAL_COMPARISON.md`](./MERGE_AND_STRUCTURAL_COMPARISON.md)
- [`CAUSAL_BUNDLES_AND_GUARANTEES.md`](./CAUSAL_BUNDLES_AND_GUARANTEES.md)
- [`HOST_ADAPTERS.md`](./HOST_ADAPTERS.md)
