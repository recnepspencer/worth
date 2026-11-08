# worth-runtime-bridge

`worth-runtime-bridge` is the causal protocol boundary between authoritative
Relational truth and derived Signal computation.

Use it when one runtime must translate committed semantic changes, snapshots,
branches, history, or writeback across that boundary without giving either
side the other side’s authority.

For Query-installed conditional operations, application and domain code starts
at `worth_query_decl::facade` and `worth_query_host::facade`. Query runtime construction supplies the
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

Execution basis admission distinguishes atomic attempts from managed steps.
`admit_atomic_execution_basis` binds the same intent, truth observation and
Signal request lifecycle without minting a step contract or queue authority.
Atomic bases may complete, cancel, reject or be abandoned; managed safe points,
queue mutations and yielding return typed refusals. `step_contract()` therefore
returns an optional managed capability. Yielded bases retain a concrete contract
because only managed bases can enter that lifecycle.

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
Standard routing, explicit controls, and replay/certification capabilities all
use this same root import. Those usage categories are documentation groupings.

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

### Read-Only Native Observation Custody

`BridgeConditionalDecisionEvidence::retain_causal_observation()` shares the
actual immutable decision, semantic observations, admitted snapshot identity and
performed Signal invalidation observation. `BridgeConditionalCausalObservation`
retains neither the admitted snapshot reader nor the operational reentry core.
It provides no source-read, execution, continuation or disclosure permission.
The operational seed remains the only retained conditional reentry entrance.

`computed_evaluation()` borrows the same Signal `AppliedEffectReport` from that
immutable backing. It preserves the actual computed verdict, suppression reason
and comparison; `signal_output_versions()` supplies the observed before/after
versions. Passive decisions have no computed report. The
[installed comparator court](src/builder/tests/correspondence/conditional_compatibility/equivalence_observation.rs)
contrasts genuine equivalent and changed evaluations and checks original custody
through operational evidence release. An evaluation of scheduling or readiness
still describes that installed contract; it cannot establish equivalence of an
application handler's output. Application producer explanation requires its
own exact source/output association and current admission.

`BridgeSealedRuntimeAssembly::compare_exact_output` evaluates a supplied u64
baseline and candidate through a source-free, on-demand Exact contract. It creates
one fresh private Signal session, initializes the exact baseline, then compares
the candidate in that same partition. Neither caller nor provider receives the
session. Other occurrences cannot replace its state, even with equal descriptive
labels. The returned `BridgeExactOutputComparison` retains only genuine paired
computed observations; its candidate's before version must equal its baseline's
after version. The compute provider must produce each supplied version exactly.

`BridgeOutputComparisonDenial` identifies admission, baseline or candidate failure
and retains the original typed cause. Partial failure releases the private session
and returns no comparison. This is fixed scratch evaluation, not an atomic Native
publication or a rollback guarantee. The [exact-pair courts](src/builder/tests/correspondence/conditional_compatibility/equivalence_observation/exact_pair.rs)
cover nonzero initialization, exact equivalence/change, occurrence isolation and
release, wrong baseline, candidate failure/retry, foreign owner and cancellation.
The pair supplies no Native-field or application source association by itself.

Bridge reserves the decision and observation headers before provider callbacks.
Signal funds its immutable invalidation backing at its existing post-execution
receipt-preparation boundary, preserving that boundary's typed failure posture.
Sharing an already-issued observation is infallible. Its original reservations
remain until the last corresponding share drops; no second ledger is created.
The unique performed receipt still transfers once to the delivery binder, and
reentry never reports a newly performed invalidation.

Query output-readiness delivery privately carries this actual Native observation
instead of retaining counters alone. This is an engine prerequisite: fresh
ordinary Decl/Host causal inspection remains unfinished in
[milestone 9.18.1 Phase 9](../../plans/WORTH-query/milestone-9.18.1.md#phase-9-causal-inspection-from-ordinary-application-receipts).
The [Native custody court](src/builder/tests/correspondence/conditional_compatibility/source_packet_affinity/causal_retention.rs)
checks reader release, original retention custody and the single receipt transfer.

### Native Conditional Evidence Assembly

`BridgeConditionalCausalEnvelopeRequest` carries the genuine readonly observation
and a reporting summary. `BridgeSealedRuntimeAssembly::assemble_conditional_causal_envelope`
checks the exact sealed owner before deriving references or reading evidence.
The shared envelope assembler distinguishes this source from legacy diagnostics:
legacy assembly still requires an actual retained Route record and cannot resolve
a Native observation from projected strings. Native assembly retains the same
immutable observation through the final envelope share, with no source reader,
provider contact or new performed invalidation receipt.

The Native summary binds a distinct causal-anchor identity and expected Query
observation identity. These descriptive identities grant no fresh Query authority;
matching digests cannot replace actual observation custody. Native binding counts
are separate from legacy registry lookups and external-authority references.
Original reservations continue to cover the shared observation; descriptor and
envelope allocations retain the existing lower diagnostic assembly posture.
Fresh ordinary Host inspection, metadata admission and diagnostic disclosure remain
required Phase 9 work.

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

### Query-owned performed-source reporting

`BridgeQueryObservationReportingRequest::from_performed_source` extends the
existing descriptive, funded reporting strategy for a Query-owned observation
of a canonical source operation. Its performed-source shape differs from the
completed-query-result shape, even for otherwise equal reporting associations.
`QueryObservation` identifies the Query evidence owner/family; this source form
does not claim that a SELECT query ran. Neither form admits READ, metadata,
DiagnosticMask or rich Native materialization. Query's actual fresh Inspection
session and consuming terminal retain those responsibilities. The common Bridge
assembler reports no Native binding, evaluation or invalidation for a source-only
reference. The performed-source reporting case and three affected Bridge controls
pass. This does not qualify connected producer assembly or rich Native materialization.

### Initial baseline for an attached World

`RecordedRuntimeWorldBaseline` records the versioned, uninstalled correspondence
baseline and its registered source's stable adapter and graph-role meaning.
`RuntimeWorldCorrespondencePort` exports only an actual admitted baseline with
no installed semantic dependency. Readmission compares that recipe with the
current registered source and consumes the Signal owner's fresh admitted
InitialEmpty seed to mint a new local Bridge basis. Unsupported sources,
installed dependencies and altered meaning refuse. Source profile and role
descriptions remain correlation data. Store's private Home composition supplies
the same actual Native owner and registered source; a matching caller-supplied
profile cannot construct the concrete Home stage.

## Authored conditional definition recovery

For platform composition, `BridgeConditionalRuntimeBuilder::capture_authored_definition`
captures the actual program's Signal images and Bridge's versioned, checksummed
section. That section retains conditional identities and locations, full
portable dependency meaning, declared slots and exact performed lowering claims.
Source runtime identities, providers, values and scheduling state stay outside
these bytes. The landed `RecordedRuntimeWorldBaseline` format is unchanged.

`Box<BridgeConditionalRuntimeBuilder>::readmit_authored_owner` consumes the fresh
program and `BridgeAuthoredInstallationBindings`. Signal supplies each actual
cold branch; Bridge readmits its mapping and graph capabilities, restores the
recorded slots, and rebuilds branch-local dependency indexes, lowering registry
and retention ledger. Branches share the consumed program rather than cloning
its runtime. Definition bytes retain their charges; publication candidate slots
start empty. Fresh source and provider objects come from installation.

For an empty Signal source, the same boxed builder offers
`prepare_initial_authored_source`. It captures its actual installed conditional
definitions through the same Bridge codec and retains its original program with
Signal's pending publication. `BridgeInitialSourcePublication::advance` drives
the original definition and branch frames; `try_into_published` returns pending
custody unchanged until the real branch ACK. The published handoff then consumes
fresh `BridgeAuthoredInstallationBindings` through the same cold installation
kernel. Preparation refusal returns the builder with its original Signal runtime
restored. Store-backed courts cover a refused branch step after definition ACK,
same-original retry, populated-source refusal and changed-build recomputation.
Registered no-effect refusals use Signal's concrete proof and explicit retry
port, preserving the same authored frames and builder. Ordinary observations
keep their terminal answer; retry cannot erase a pending or unresolved physical
original or publish a committed frame again.

The privately constructed `BridgeReadmittedAuthoredAssembly` owns the durable
Signal owner and exposes authored definition inspection. Cold multi-branch and
changed-build courts cover this intrinsic entry. Store-backed owner courts also
cover graceful reopen and a kill after all publications are terminal, comparing
three branches with the memory-only authored program and demanding derived
values anew under the same or a changed build. These courts use source-free
conditional definitions; they do not claim mixed owner effects. Home admission
and mixed publication remain pending.
Conditional execution and managed services retain their later phase boundaries.

`BridgeReadmittedAuthoredAssembly::begin_conditional_definition_publication`
prepares the successor through Signal's original durable task. It preserves
registration, allocation and performed-claim sections, reserves the new lowering
before effects, and activates it after Signal commits. Only the acting branch
changes. `with_authored_branch` lends a serving projection to its inspection;
preparation,
pending publication and a committed projection requiring readmission have typed
refusals. Resume the retained original rather than submitting another definition.

`begin_composite_conditional_definition_publication` consumes the actual Signal
binding and composing issuer's grant. It shares the same projection preparation,
Signal callback and committed-only activation as the standalone entry. The
original retains its grant across a disposed request observation; resume it with
`resume_composite_original`. The Signal advance records the attempt association.
A foreign grant refuses before the callback, and callback refusal restores the
predecessor projection. The standalone entry refuses after composite attachment.

Writeback execution records and replay-comparison records are process-local
diagnostics. Current enumeration returns retained entries; historical reads take
an owner-issued `BridgeWritebackDiagnosticReference`, which retains neither the
payload nor the owner. Reads distinguish `NeverRetained`,
`ExpiredUnderCountLimit` and `NotRetainedAcrossReopen`; a foreign owner refuses.
Latest-entry access names the current archive. Fresh installation starts that
archive empty and separates it from prior runtime aliases. These records never
authorize effect replay; recovery evidence comes from durable owner commits.

## Retire Home-owned Bridge custody

The concrete Native adapter's `try_retire_alias_for_request` consumes its
original local runtime alias. A foreign owner or request stop returns the same
source; independently retained snapshots keep their own Native read custody.
The retired source retains passive metadata until unbudgeted `reclaim`.

`RuntimeBridge::into_home_retirement` preflights its original concrete source
wrappers before extraction. Its advancing task retains configuration and each
original retired source. Unknown source wrappers, reader pools, continuity
sources and writeback owners refuse rather than running arbitrary supplier
cleanup. A retired Bridge is local capability retirement, not Native or Store
close evidence.

The sealed Home assembly funds its retirement storage before the final World
admission cut. Only the exact Store Home caller can prepare its producer-initial
empty Signal stop; a stopped assembly retains its original Signal and Bridge
allocations through later retirement. Nonempty operational inventories refuse
through the owner gates. This does not qualify Native physical source close.
