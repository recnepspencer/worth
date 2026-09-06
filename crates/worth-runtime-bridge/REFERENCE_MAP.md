# Reference Map

This page is the bridge docs index for people who already know what they are
looking for.

If `README.md` is the landing page and `QUICKSTART.md` is the essentials page,
this file is the public reference map.

## Standard Path

Use [`worth_runtime_bridge::facade`] for the standard path:

- `RuntimeBridge`
- `RuntimeBridgeBuilder`
- `BridgeRouteRequest`
- `BridgeRoute`
- `BridgeEvaluationTarget`
- `BridgeTruthViewEvaluationRequest`
- `BridgeTruthViewEvaluation`
- `BridgeSpeculativeSessionRequest`
- `BridgeSpeculativeSessionHandle`
- `BridgeSpeculativeComparison`
- `BridgeSpeculativePromotionOutcome`
- `BridgeDiagnostics`
- `BridgeStandardDiagnosticsExplanation`

Primary guides:

- [`QUICKSTART.md`](./QUICKSTART.md)
- [`DAILY_WORKFLOWS.md`](./DAILY_WORKFLOWS.md)
- [`DIAGNOSTICS.md`](./DIAGNOSTICS.md)

## Explicit Control

Use [`worth_runtime_bridge::facade`] for deliberate non-default runtime
control.

Main domains:

- semantic aspect correspondence and conditional lowering
- runtime policy and diagnostics tiers
- truth-view declarations and materialization
- bulk planning and delivery
- stream delivery, replay, and resume
- structural comparison and merge-aware work
- explicit writeback authority integration

Primary guides:

- [`API_OVERVIEW.md`](./API_OVERVIEW.md)
- [`SEMANTIC_CORRESPONDENCE_AND_CONDITIONAL_EXECUTION.md`](./SEMANTIC_CORRESPONDENCE_AND_CONDITIONAL_EXECUTION.md)
- [`ROUTING_AND_EVALUATION.md`](./ROUTING_AND_EVALUATION.md)
- [`BRANCHING_AND_SPECULATION.md`](./BRANCHING_AND_SPECULATION.md)
- [`WRITEBACK_AND_PROMOTION.md`](./WRITEBACK_AND_PROMOTION.md)
- [`HISTORY_AND_REPLAY.md`](./HISTORY_AND_REPLAY.md)
- [`RUNTIME_POLICY.md`](./RUNTIME_POLICY.md)
- [`CHANGE_STREAMS_AND_SOURCES.md`](./CHANGE_STREAMS_AND_SOURCES.md)
- [`MAPPING_CONTINUITY_AND_REMAP.md`](./MAPPING_CONTINUITY_AND_REMAP.md)
- [`MERGE_AND_STRUCTURAL_COMPARISON.md`](./MERGE_AND_STRUCTURAL_COMPARISON.md)

Representative advanced types:

- `BridgeRuntimePolicy`
- `BridgeDiagnosticsTier`
- `BridgePolicyDeclaration`
- `HistoricalEvaluationDeclaration`
- `SourceDeclaration`
- `MaterializedTruthViewPacketSet`
- `BridgeBulkWorkloadRequest`
- `BridgeBulkWorkloadPlan`
- `ChangeStreamDeclaration`
- `StructuralIdentityDeclaration`
- `MergeHistoryDeclaration`

## Replay And Certification

Use [`worth_runtime_bridge::facade`] when the job is retained artifacts,
replay, parity proof, or certification.

Main domains:

- canonical route and evaluation records
- preview replay and promotion proof artifacts
- low-level planning and packet artifacts
- retained diagnostics and protocol records
- certification and workload proof surfaces

Primary guides:

- [`CERTIFICATION_AND_HARNESS.md`](./CERTIFICATION_AND_HARNESS.md)
- [`CAUSAL_BUNDLES_AND_GUARANTEES.md`](./CAUSAL_BUNDLES_AND_GUARANTEES.md)
- [`HOST_ADAPTERS.md`](./HOST_ADAPTERS.md)

Representative specialist types:

- `BridgeCanonicalRouteRecord`
- `BridgeCanonicalHistoricalEvaluationRecord`
- `BridgeCanonicalBulkPlanRecord`
- `BridgePreviewReplayBundle`
- `BridgeWritebackReplayBundle`
- `BridgeRouteContractProof`
- `CanonicalBridgeWorkloadRequest`
- `BridgeReplayRecord`
- `BridgeContractDiagnosticsRecord`

## Learning Order

If you are new:

1. [`README.md`](./README.md)
2. [`QUICKSTART.md`](./QUICKSTART.md)
3. [`DAILY_WORKFLOWS.md`](./DAILY_WORKFLOWS.md)
4. [`API_OVERVIEW.md`](./API_OVERVIEW.md)
5. [`DIAGNOSTICS.md`](./DIAGNOSTICS.md)

If you are integrating something deeper:

1. [`REFERENCE_MAP.md`](./REFERENCE_MAP.md)
2. the relevant domain guide
3. only then the deeper control or replay-oriented parts of the facade

## Product history and coordinated publication

[`worth-runtime-world`](../worth-runtime-world/README.md) owns product branches,
composite commit history and coordinated Relational/Signal publication. Bridge
owns installed semantic correspondence. Admit that exact installed witness through
`RuntimeBridge::runtime_world_correspondence_port().admit_installed_basis(...)`
after binding it to the actual Signal graph; descriptors or detached registration
requests cannot replace the installed witness.

Use the [Runtime World example](../worth-runtime-world/examples/runtime_world_publication.rs)
for real same-graph construction and three-way publication handling. Bridge
speculation/replay is not product-head or committed-terminal authority.
