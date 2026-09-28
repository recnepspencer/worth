# Milestone 6 Engineering Spec: Change Stream Protocol And Multi-Consumer Contracts

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-5.md](./milestone-5.md)
>
> **Primary architectural driver:** turn truth-side patch consumption into a stable, replay-safe bridge protocol that supports more than one downstream consumer shape without letting host feed quirks, per-consumer checkpoints, or delivery heuristics redefine canonical bridge semantics
>
> **Companion docs:**
> - [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
> - [worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
> - [worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [architectural_guidelines.md](../../docs/coding-guidelines/arch_laws.md)
> - [domain_standards.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [performance_guidelines.md](../../docs/coding-guidelines/perf_laws.md)

## Summary

Milestones 1 through 5 established that:

- committed truth enters the bridge through canonical committed envelopes
- fine-grained routing, continuity, historical view selection, and bulk planning
  are explicit bridge-owned proof chains
- large workloads lower once into canonical packet sets, reduction summaries,
  and legality-bearing execution plans
- replay and diagnostics already have a trustworthy canonical planning substrate

That is enough to make one bridge runtime consume one canonical truth workload
correctly.

It is not enough to make bridge-side change consumption a product-grade
protocol surface.

Without Milestone 6, the bridge still risks a weaker and more ad hoc answer to
these questions:

- how does a consumer resume after partial progress or restart?
- what exactly is being checkpointed: host transport position, canonical bridge
  position, or consumer-local interpretation progress?
- how do multiple downstream consumers share one canonical truth stream without
  silently disagreeing about order, coalescing, idempotence, or replay?
- how does the bridge express backpressure and bounded batching honestly
  without letting delivery pressure redefine canonical truth order?
- how do replay and live consumption remain semantically identical when host
  feeds, cursor stores, and buffering policies differ?

Milestone 6 exists because a bridge that plans and routes correctly but still
consumes changes through one host-shaped feed path is not a stable protocol
boundary yet.

The bridge must be able to say:

`this exact canonical truth change stream, under this exact stream protocol identity and this exact consumer contract identity, delivered this exact ordered batch window, produced this exact checkpoint basis, and can be resumed or replayed without changing routing meaning`

not:

`the bridge subscribed to some host feed, advanced some local cursor, and downstream consumers happened to stay in sync`

The bridge still does not own:

- truth-side CDC production semantics
- truth mutation authority
- signal scheduling or execution semantics
- host transport internals

It owns:

- the bridge-facing change stream contract
- canonical stream position identity
- consumer contract identity and admission
- checkpoint, resume, replay, idempotence, coalescing, and backpressure
  interpretation rules
- canonical stream diagnostics, counters, and replay records

## Goal

Make bridge-side change consumption a deterministic, replay-safe, multi-consumer
protocol surface instead of one host-specific feed path.

## Why This Milestone Exists

Milestone 6 belongs immediately after Milestone 5 because Milestone 5 supplied
the scale-path substrate that change consumption must now expose as protocol
truth instead of internal planning detail:

- canonical workload identity
- canonical packet identity
- canonical reduction identity
- explicit legality and fallback classification
- replay-safe planning and reduction artifacts

Without Milestone 5, a change-stream milestone would be forced to choose
between two bad options:

- protocolize raw per-event host feeds and leave bulk/reduction semantics out
  of contract truth
- or smuggle scale-path planning into delivery heuristics where consumer
  behavior becomes dependent on host batching

Milestone 6 therefore exists after Milestone 5 so stream consumption can expose
canonical bridge work rather than raw feed accidents.

Milestone 6 also belongs before Milestone 7 because reactive source protocols
should not ship before the bridge has an equally explicit truth-change
consumption contract. Read contracts and change-stream contracts are the two
halves of the same boundary; productizing one without the other would create an
asymmetric bridge surface.

Later roadmap items also depend on Milestone 6:

- reactive source protocols need stable stream semantics for invalidation and
  freshness coordination
- merge-aware bridge semantics need replay-safe multi-parent stream
  interpretation, not one-off host adapters
- speculative preview and policy propagation need explicit checkpoint and
  resume semantics
- end-to-end certification needs a canonical answer to "what stream material
  did each consumer observe and acknowledge?"

Milestone 6 earns its place in the roadmap by solving the next real structural
problem after planned bulk routing: protocol-grade consumption correctness.

## Adversarial Constraint

Milestone 6 must survive the following hostile condition:

> A long-lived system with canonical truth patches arriving through different
> host feed shapes, multiple downstream consumer contracts with different batch
> widths and richness policies, partial progress and restart, replay after
> checkpoint publication, bounded coalescing under heavy burst load,
> backpressure from slower consumers, diagnostics tiers that vary by
> environment, and broad workloads that already lower through Milestone 5 bulk
> planning must consume the same canonical ordered truth material the same way
> every time, publish checkpoint and resume truth that is explicit and
> replay-safe, preserve deterministic routing meaning across consumer shapes,
> and never let host transport quirks, consumer-local buffering, or scheduler
> timing redefine canonical bridge semantics.

Concretely, the design must remain correct when all of the following are true:

- one truth commit burst produces many canonical change envelopes in quick succession
- one consumer wants narrow low-latency delivery while another wants broader
  coalesced batch windows
- one consumer checkpoints after every admitted batch while another checkpoints
  after a larger acknowledged window
- replay occurs after restart from a bridge checkpoint rather than a live host cursor
- the same canonical stream is presented through more than one host integration
  shape
- backpressure causes delayed delivery for one consumer but not another
- diagnostics richness changes between environments
- coalescing is profitable for some consumer contracts and forbidden for others
- one consumer is route-focused while another is replay/audit-focused

If any supported path:

- changes canonical order because host delivery order drifted
- lets a consumer-local checkpoint masquerade as global stream authority
- lets coalescing erase semantically distinct canonical stream members
- lets backpressure alter interpretation rather than only pacing
- requires live host state to explain resume or replay
- weakens Milestone 5 packet, reduction, or replay semantics at the protocol
  boundary
- cannot explain which canonical stream members were acknowledged, skipped,
  replayed, coalesced, or rejected

then Milestone 6 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- change consumption is a first-class bridge protocol, not a convenience wrapper
  over one host feed adapter
- the bridge protocol is defined in terms of canonical truth change members and
  bridge-lowered interpretation artifacts, not host transport offsets
- stream position identity, checkpoint identity, and consumer contract identity
  are distinct concepts and must remain distinct types
- multiple downstream consumer shapes are supported through explicit consumer
  contracts, not by letting each consumer invent its own cursor and coalescing
  rules
- backpressure is a protocol-visible pacing signal, not permission to reorder,
  widen, or weaken canonical stream meaning
- coalescing is contract-declared and semantically bounded; it is never an
  accidental side effect of buffer fullness
- replay and resume are protocol operations over canonical bridge stream truth,
  not host-adapter recovery tricks
- truth runtime remains the authority for canonical patch order and patch
  content; the bridge remains the authority for consumption interpretation,
  checkpoint semantics, and consumer contract shaping
- Milestone 6 productizes protocol-grade change consumption only; it does not
  productize generic read-source contracts or merge semantics

Normative consequence:

- public bridge APIs that expose raw host offsets as the primary resume token
  are out of spec
- consumer-specific ad hoc coalescing code outside the bridge protocol is out
  of spec
- checkpoint publication without explicit acknowledged member identity is out
  of spec
- silent drop-or-retry semantics hidden behind backpressure are out of spec
- delivery paths that reinterpret canonical stream members differently during
  replay are out of spec

## Scope

### In Scope

- one bridge-owned declaration surface for change-stream consumption
- canonical stream member identity and canonical stream window identity
- bridge-owned cursor, checkpoint, resume, replay, idempotence, coalescing, and
  backpressure vocabulary
- bridge-owned consumer contract vocabulary supporting more than one downstream
  consumer shape
- admission and lowering of consumer contracts before delivery
- deterministic planning of stream windows into bridge-consumable batch truth
- replay-safe checkpoint and replay records
- typed diagnostics and counters for stream progress, coalescing, checkpoint,
  resume, replay, and mismatch behavior
- harness certification for restart, replay, multi-consumer determinism, and
  backpressure-visible pacing behavior

### Explicitly Out Of Scope

- generic reactive source/read protocol productization
- full host builder/configuration cleanup beyond what this protocol boundary
  requires
- merge-aware interpretation of multi-parent truth history
- speculative preview lifecycle or speculative branch coordination
- bridge-mediated writeback or effect production
- scheduler-owned downstream execution semantics inside `worth-signal`

Milestone 6 must stay focused on protocol-grade truth-change consumption rather
than absorbing the later source, merge, policy, or writeback milestones.

## Governing Design Rules

### 1. Truth Owns Stream Production, Bridge Owns Stream Consumption Contract

The truth runtime defines:

- canonical patch order
- canonical committed change envelope content
- CDC publication truth
- retention or historical availability of change material where supported

The bridge defines:

- the public change-stream declaration surface
- canonical stream member identity and stream window identity
- cursor and checkpoint vocabulary
- consumer contract vocabulary
- coalescing, idempotence, replay, resume, and backpressure interpretation
- stream diagnostics and replay records

The signal runtime and other downstream systems define:

- what they do with admitted bridge change batches after delivery

The bridge must not redefine truth-side patch order or patch semantics.

The bridge facade is the only public change-stream consumption surface.

### 1.1 Canonical Stream Member Ontology Must Be Fixed

Milestone 6 must not leave "stream member" as an implementation-defined noun.

For this milestone, one canonical stream member is:

- one canonical committed change envelope admitted by the truth runtime as one
  ordered change publication unit
- carrying one canonical committed-envelope identity
- carrying one canonical source branch / commit / patch / snapshot basis
- optionally lowering into one or more Milestone 5 workload members, packets,
  reductions, or delivered consumer effects

Rules:

- a canonical stream member is not a raw host transport event
- a canonical stream member is not a Milestone 5 packet family member
- a canonical stream member is not a coalesced delivery window
- a canonical stream member is not a consumer-local checkpoint unit
- if one truth-side publication contains N canonical committed envelopes, it
  contains N canonical stream members even if one consumer later coalesces them
  into fewer delivery windows
- all stream positions, checkpoints, replay records, duplicate-detection rules,
  and coalescing contracts must reference canonical stream member identity first

### 2. Stream Position, Checkpoint, And Consumer Contract Must Be Distinct Proof Chains

Milestone 6 must not collapse these concepts:

- `CanonicalStreamPosition`
  - where a canonical stream member sits in bridge-visible order
- `ConsumerCheckpointToken`
  - what a specific consumer contract has acknowledged and may resume from
- `AdmittedConsumerContract`
  - under what interpretation rules the consumer is consuming the stream

Rules:

- a stream position is not itself permission to resume any consumer
- a checkpoint token is only valid for the consumer contract identity that
  produced it
- changing consumer contract identity invalidates prior checkpoint reuse unless
  an explicit compatibility rule is declared and certified
- replay records must carry stream position truth and consumer-contract truth
  separately
- host transport offsets may contribute to adapter internals but are never the
  public bridge truth

### 2.1 Checkpoint Frontier Semantics Must Be Explicit

Milestone 6 must not leave checkpoint meaning at the level of "what a consumer
has acknowledged."

For this milestone, a `ConsumerCheckpointToken` must carry at least:

- `consumer_contract_identity`
- `stream_protocol_identity`
- `checkpoint_frontier_kind`
- `contiguous_acknowledged_through_position`
- `acknowledged_member_set_digest`
- `checkpoint_member_count`
- `source_retention_anchor`
- `protocol_semantics_version`

The admitted `checkpoint_frontier_kind` values for this milestone are:

- `ContiguousFrontier`
  - the consumer has acknowledged every canonical stream member through one
    canonical stream position with no gaps
- `ContiguousFrontierWithObservedDuplicates`
  - the same contiguous frontier is acknowledged, but duplicate observation
    records are retained as explanatory protocol truth

Rules:

- sparse acknowledged sets with holes are out of scope for Milestone 6
- a checkpoint token must never mean "latest seen"
- checkpoint identity must depend on the full acknowledged-member basis, not
  only the highest acknowledged stream position
- resume from checkpoint is legal only when the runtime can prove compatibility
  between the checkpoint retention anchor and the currently available canonical
  stream material
- checkpoint truncation must mean "the required canonical member frontier is no
  longer materializable," not merely "the host cursor moved"

### 3. Consumption Must Begin From One Declaration Surface

Each stream consumer must begin from one explicit bridge-owned declaration that
contains at least:

- consumer contract selector
- batching/coalescing intent
- checkpoint publication mode
- replay/resume mode
- diagnostics mode
- delivery intent

Representative proof chain:

```rust
pub struct ChangeStreamDeclaration { ... }
pub struct ValidatedStreamProtocol { ... }
pub struct ResolvedConsumerContract { ... }
pub struct PlannedChangeStreamWindow { ... }
pub struct MaterializedChangeBatch { ... }
pub struct LoweredConsumedChangeSet { ... }
pub struct DeliveredConsumerBatch { ... }
```

Rules:

- scattered registration of cursor handling, coalescing policy, and checkpoint
  publication across separate calls is out of spec
- delivery may consume only lowered, admitted stream batches
- replay may consume only canonical replay records plus admitted contract truth
- every proof-bearing type in this chain must have sealed constructors and
  private fields

### 4. Consumer Shapes Must Be Explicit And Closed Per Milestone

Milestone 6 must support more than one downstream consumer shape, but through a
closed vocabulary rather than open-ended host strings.

The first admitted consumer shapes for this milestone should be:

- `RoutingConsumer`
  - consumes canonical change batches to produce routing and invalidation work
- `ReplayAuditConsumer`
  - consumes canonical change batches to verify or reconstruct replay-visible
    bridge behavior

Rules:

- a consumer shape declares what interpretation artifacts it is allowed to
  receive
- consumer shapes may differ in richness and pacing policy, but not in the
  canonical meaning of the underlying stream members
- adding new consumer shapes requires spec update first
- generic "custom consumer kind" strings are out of spec for Milestone 6

Diagnostics richness remains an artifact policy over canonical protocol truth,
not a third consumer shape.

Rules:

- diagnostics policy may change what explanation, counters, and retained records
  are materialized
- diagnostics policy may not create a distinct stream meaning, checkpoint
  frontier, or coalescing legality basis
- a diagnostics-only observer that changes no acknowledged frontier is not a
  consumer contract in Milestone 6; it is a derived observability path

### 5. Coalescing Must Be Semantically Declared Before Delivery

Milestone 6 must not treat coalescing as a transport-side optimization.

Coalescing is only admitted when the consumer contract declares:

- the eligible coalescing family
- the coalescing boundary
- the identity basis of the coalesced batch
- the replay basis of the pre-coalesced members
- the reason coalescing preserves routing meaning

Rules:

- coalescing may group only semantically mergeable canonical members
- coalescing may not cross explicit branch, snapshot, or protocol-boundary
  changes
- coalescing may not erase required duplicate visibility when idempotence proof
  depends on duplicate observation
- uncoalesced member identity must remain recoverable from canonical records
- buffer fullness alone is never a legal coalescing basis

### 5.1 Coalescing Families And Key Contracts Must Be Closed

Milestone 6 must write the first admitted coalescing families as exact,
identity-bearing contracts rather than leaving them to adapter policy.

The admitted coalescing families for this milestone are:

- `RoutingWindowCoalescing`
  - groups adjacent canonical stream members only when they lower into the same
    consumer contract, the same branch identity, the same checkpoint-publication
    mode, and one compatible Milestone 5 reduced routing basis
- `ReplayAuditWindowCoalescing`
  - groups adjacent canonical stream members only when all member identities
    remain individually recoverable and replay/audit meaning is unchanged

Forbidden for Milestone 6:

- branch-crossing coalescing
- snapshot-basis-crossing coalescing
- contract-crossing coalescing
- checkpoint-mode-crossing coalescing
- host-buffer-driven coalescing with no canonical member-family proof

Required coalesced-window identity tuples:

- `RoutingWindowCoalescing`
  - `(consumer_contract_identity, first_stream_position, last_stream_position, coalescing_family, member_set_digest, reduced_routing_basis_digest, checkpoint_publication_mode)`
- `ReplayAuditWindowCoalescing`
  - `(consumer_contract_identity, first_stream_position, last_stream_position, coalescing_family, member_set_digest, replay_basis_digest, checkpoint_publication_mode)`

Rules:

- coalesced-window identity must never discard the full member-set digest
- coalescing legality is proven from canonical member adjacency plus declared
  family contract, not from elapsed time or buffer occupancy
- if two candidate coalesced windows have the same first and last position but
  different member-set digests, they are different windows

### 6. Backpressure Must Pace Delivery Without Altering Meaning

Backpressure is protocol-visible and typed.

Representative classes:

- `NoPressure`
- `ConsumerBuffered`
- `ConsumerSaturated`
- `CheckpointLagged`
- `ReplayCatchUp`

Rules:

- backpressure classes may alter pacing, window width ceilings, or admission of
  optional richness
- backpressure classes may not alter canonical stream order
- backpressure classes may not silently promote unsupported coalescing
- backpressure classes may not hide dropped work behind "best effort" language
- pressure changes must be visible in counters and decision records

### 7. Resume And Replay Must Depend On Canonical Protocol Truth

Milestone 6 must not let resume or replay depend on ambient host state.

Resume requires:

- a validated consumer contract identity
- an admitted resume mode
- either a canonical checkpoint token or an explicitly supported stream
  position reference
- compatibility with the current protocol semantics version

Replay requires:

- canonical stream replay records
- the consumer contract identity used for the original run
- typed mismatch handling for incompatible protocol, checkpoint, or stream
  material

Rules:

- replay must preserve routing meaning even if transport shape changes
- resume mismatch classes must distinguish checkpoint incompatibility, protocol
  incompatibility, stream truncation, and consumer-shape mismatch
- live transport state may assist materialization but may not be required to
  explain or validate the replayed interpretation
- a direct stream-position resume is only admitted for this milestone when it
  is explicitly marked as non-checkpoint resume and carries no acknowledgement
  claim beyond "begin materialization at this canonical stream position"
- checkpoint resume and stream-position resume must remain different proof
  chains and different failure classes

### 8. Idempotence Must Be Declared, Not Assumed

Milestone 6 must define exactly what "idempotent consumption" means for each
admitted consumer shape.

At minimum, the protocol must distinguish:

- duplicate canonical member observation
- duplicate coalesced batch observation
- duplicate checkpoint publication attempt
- replayed historical observation

Rules:

- idempotence claims must name the identity basis used for duplicate detection
- a consumer shape that is not fully idempotent for a given operation must say
  so explicitly and fail typed where duplication would be unsafe
- checkpoint advancement must never rely on undocumented assumptions that all
  consumer work is idempotent

### 9. Stream Planning Must Respect Milestone 5 Bulk Truth

Milestone 6 must not introduce a second planning world for streamed changes.

Rules:

- streamed canonical members lower through Milestone 5 workload, packet, and
  reduction truth where those artifacts are required
- stream windows must point at canonical member identities, not re-describe raw
  feed payloads
- consumer contracts may request different delivery shapes, but they may not
  redefine routing or reduction semantics already lowered by the bridge
- protocol replay must be able to explain which Milestone 5 plan or reduced
  artifact each streamed window depended on

### 10. Canonicality Must Be Declared For Every Protocol Artifact

For each canonical Milestone 6 artifact, the spec must define:

- ordered input basis
- ordering key
- deduplication rule
- identity-bearing fields
- explanatory-only fields
- digest basis

Canonicality must cover at least:

- stream declaration ordering
- stream member ordering
- stream window ordering
- checkpoint ordering
- replay record ordering
- consumer contract ordering
- coalescing record ordering
- backpressure record ordering

If host adapter ordering, buffer timing, or diagnostics mode can alter any
canonical protocol artifact, the design is out of spec.

### 10.1 Exact Identity Contracts Must Be Written Before Implementation

Milestone 6 must follow the tightened Milestone 5 standard: every primary
protocol artifact needs an explicit identity tuple before implementation.

Required identity-bearing tuples:

- `ChangeStreamDeclaration`
  - `(consumer_shape, resume_mode, checkpoint_publication_mode, coalescing_intent, replay_mode, delivery_intent, protocol_semantics_version)`
- `CanonicalStreamPosition`
  - `(stream_protocol_identity, canonical_stream_member_identity, ordinal_position)`
- `AdmittedConsumerContract`
  - `(consumer_shape, stream_protocol_identity, admitted_resume_mode, admitted_checkpoint_mode, admitted_coalescing_family, admitted_replay_mode, admitted_delivery_intent, contract_semantics_version)`
- `PlannedChangeStreamWindow`
  - `(consumer_contract_identity, first_stream_position, last_stream_position, member_set_digest, coalescing_family, checkpoint_publication_mode)`
- `ConsumerCheckpointToken`
  - `(consumer_contract_identity, stream_protocol_identity, checkpoint_frontier_kind, contiguous_acknowledged_through_position, acknowledged_member_set_digest, source_retention_anchor, protocol_semantics_version)`
- `CanonicalStreamReplayRecord`
  - `(consumer_contract_identity, stream_window_identity, checkpoint_token_identity, replay_basis_digest, protocol_semantics_version)`
- `BackpressureDecisionRecord`
  - `(consumer_contract_identity, stream_window_identity, pressure_class, pressure_reason_family)`

Rules:

- no implementation may introduce alternate ad hoc identities for these
  artifacts without updating the spec first
- `ordinal_position` is explanatory only when the canonical member identity is
  already enough to determine order; otherwise it is identity-bearing
- `diagnostics_policy_class` is intentionally not identity-bearing for
  `ChangeStreamDeclaration` in Milestone 6 because diagnostics richness must
  not change canonical stream meaning
- `checkpoint_member_count` is a required carried field on
  `ConsumerCheckpointToken`, but it is not part of the identity tuple because
  it is derivable from the contiguous frontier basis
- replay and delivery intent are identity-bearing on
  `AdmittedConsumerContract` because they change which protocol surfaces the
  admitted consumer is allowed to execute
- `BackpressureDecisionRecord` is a window-derived decision artifact in
  Milestone 6, not a contract-declared policy-class identity
- `diagnostics_policy_class` may participate in declaration identity but must
  not alter stream member identity, stream position identity, consumer contract
  identity, checkpoint identity, or replay identity unless the policy changes
  semantic protocol behavior, which Milestone 6 does not admit
- any future compatibility or upgrade path must be defined as an explicit
  identity transformation rule, not as "best effort decode"

### 11. Observation And Delivery Must Remain Phase-Typed

Milestone 6 must satisfy Architectural Laws 18, 30, and 41.

Rules:

- a materialized stream batch is read-only protocol observation, not mutation
  authority
- delivery may not gain branch retargeting, stream widening, or host cursor
  mutation powers that planning did not admit
- phase transitions must make illegal ordering uncompilable
- runtime checks for protocol facts already carried by the proof-bearing types
  are a design failure

## Phases

### Phase 1: Canonical Stream Identity And Contract Vocabulary

Phase 1 exists to make streamed change consumption structurally representable
before restart, replay, and multi-consumer behavior are claimed.

Milestone 6 must first define:

- one canonical change-stream declaration surface
- canonical stream member identity and stream window identity
- the canonical stream-member ontology and exact identity tuples for declaration,
  position, contract, window, checkpoint, replay, and pressure artifacts
- canonical cursor, checkpoint, resume, replay, coalescing, and backpressure
  vocabulary
- the closed vocabulary of admitted consumer shapes and protocol mismatch
  classes

This phase leaves the system in a coherent state where:

- stream consumption is described through bridge-owned artifacts
- host transport offsets no longer masquerade as bridge protocol truth
- multiple consumer shapes can be discussed without inventing ad hoc contracts
- checkpoint frontier meaning is explicit before restart or replay surfaces exist

### Phase 2: Consumer Admission, Window Planning, And Checkpoint Semantics

Phase 2 exists to turn protocol vocabulary into deterministic, admitted,
consumer-specific consumption plans.

Milestone 6 must then implement:

- validation and admission of consumer contracts
- deterministic planning of stream windows into consumable bridge batches
- typed checkpoint publication and resume eligibility
- typed coalescing and backpressure decisions
- counters and decision records for stream breadth, coalescing, checkpoint, and
  pressure behavior

This phase leaves the system in a coherent state where:

- identical canonical stream material and contract inputs lower to identical
  stream windows
- restart and resume behavior are explicit and typed
- coalescing and backpressure are visible protocol decisions rather than hidden
  buffer behavior
- no consumer contract can silently reinterpret what one canonical stream member
  means

### Phase 3: Replay, Multi-Consumer Certification, And Protocol Diagnostics

Phase 3 exists to prove that the protocol is trustworthy instead of merely
plausible.

Milestone 6 must finally ship:

- replay-safe stream records and checkpoint records
- delivery surfaces that consume admitted stream windows only
- multi-consumer harness suites covering divergence in pacing, checkpoint
  frequency, and richness policy
- hostile certification for restart, replay, coalescing, and backpressure
  visibility

This phase leaves the system in a coherent state where:

- the bridge can certify stream consumption behavior mechanically
- replay and restart validate the protocol rather than host adapter accidents
- later source, merge, and policy milestones can rely on one stable change
  consumption contract

## Must Ship

- canonical change-stream declaration, stream member, and stream window
  artifacts
- typed consumer contract vocabulary for more than one downstream consumer shape
- typed cursor, checkpoint, resume, replay, coalescing, and backpressure
  artifacts
- deterministic window planning and consumer admission
- typed failures for contract mismatch, checkpoint mismatch, replay mismatch,
  coalescing illegality, backpressure policy violation, and protocol-version
  incompatibility
- counters and decision-log records for stream width, acknowledged width,
  checkpoint width, coalescing width, replay width, and pressure classes
- replay-safe stream and checkpoint records
- harness certification lanes for restart, replay, multi-consumer parity, and
  diagnosable protocol failure behavior

## Must Preserve

- truth runtime remains the authority for canonical truth patch order and patch
  semantics
- signal runtime remains the authority for downstream scheduling and execution
- no weakening of Milestone 5 canonical planning, reduction, or replay truth
- no host-specific feed glue becoming the public bridge contract
- no silent loss of semantically distinct canonical members under coalescing
- no backpressure-driven semantic drift
- canonical ordering and replay-safe protocol identities
- clean facade boundaries rather than consumer reach-through into host adapters

## Acceptance Evidence

Milestone 6 is complete only when the bridge harness can prove:

- identical canonical stream material and consumer-contract inputs lower to
  identical admitted stream windows
- resumed and replayed consumption preserve routing semantics
- multiple consumer shapes observe the same canonical stream meaning even when
  pacing and checkpoint frequency differ
- checkpoint and resume behavior remain explicit, typed, and diagnosable
- coalescing preserves declared semantics and remains reconstructable from
  canonical records
- backpressure changes pacing without changing canonical interpretation
- protocol richness changes diagnostics only, not stream meaning
- protocol mismatches and replay mismatches fail explicitly and typed

## Architectural Notes

### Expected Internal Subdomains

Milestone 6 should extend the bridge crate with subdomains such as:

- `stream/declaration/`
- `stream/protocol/`
- `stream/contracts/`
- `stream/window/`
- `stream/checkpoints/`
- `stream/replay/`
- `stream/coalescing/`
- `stream/backpressure/`
- `delivery/stream/`
- `diagnostics/stream/`
- `harness/fixtures/stream_restart.rs`
- `harness/fixtures/multi_consumer.rs`
- `harness/fixtures/stream_backpressure.rs`

This follows workspace domain standards:

- protocol validation is not the same responsibility as consumer contract admission
- checkpoint publication is not the same responsibility as replay record construction
- coalescing legality is not the same responsibility as backpressure signaling
- delivery is not the same responsibility as diagnostics reconstruction

### Minimum Counter Floor

Milestone 6 must add counters such as:

- `stream_member_count`
- `stream_window_count`
- `stream_window_member_count`
- `stream_consumer_contract_count`
- `stream_checkpoint_count`
- `stream_checkpoint_member_count`
- `stream_resume_attempt_count`
- `stream_resume_rejection_count`
- `stream_replay_count`
- `stream_replay_mismatch_count`
- `stream_coalesced_member_count`
- `stream_coalesced_window_count`
- `stream_duplicate_member_observation_count`
- `stream_backpressure_signal_count`
- `stream_consumer_saturated_count`
- `stream_checkpoint_lag_count`
- `stream_protocol_mismatch_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Protocol Failure Policy

Milestone 6 must carry protocol failures structurally rather than narratively.

Required failure classes:

- `UnsupportedConsumerShape`
- `UnsupportedResumeMode`
- `ProtocolVersionMismatch`
- `CheckpointContractMismatch`
- `CheckpointStreamMismatch`
- `CheckpointTruncated`
- `IllegalCoalescingBoundary`
- `NonIdempotentDuplicateObservation`
- `BackpressurePolicyViolation`
- `StreamReplayMismatch`
- `StreamDeliveryRejected`

Rules:

- every admitted stream window receives exactly one delivery outcome
- failure remains visible in canonical protocol truth
- failure must include the protocol boundary that failed
- checkpoint or replay rejection must not degrade into silent full restart

## Test And Harness Model

Milestone 6 must follow the same structural testing discipline as earlier
bridge milestones.

Expected first-class test surfaces:

- restart and resume scenarios
- checkpoint compatibility and incompatibility scenarios
- replay parity and replay drift scenarios
- multi-consumer parity scenarios
- coalescing legality and illegality scenarios
- backpressure visibility scenarios
- diagnostics-tier invariance scenarios
- counter certification scenarios

Milestone 6 is not complete with only direct fixture tests. It must establish a
real protocol certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for canonical stream timelines
- `MutationBatch` for bursty committed change groups and restart cut points
- `ExecutionRequest` for declaration validation, stream-window planning,
  delivery, checkpoint publication, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, coalescing, and pressure sweeps
- `ParitySuite` for consumer-to-consumer and run-to-run parity
- `CertificationMatrix` for hostile restart, checkpoint, and pressure coverage

Minimum certification families:

- fixed deterministic restart-from-checkpoint fixtures
- fixed deterministic coalesced and non-coalesced consumer fixtures
- seeded multi-consumer pacing matrices
- replay-after-restart certification from canonical stream and checkpoint records
- protocol mismatch rejection certification
- exact counter assertions for named stream-width and checkpoint-width scenarios

Minimum representative test names:

- `tests::contracts::identical_stream_inputs_lower_to_identical_stream_windows`
- `tests::delivery::resume_from_checkpoint_preserves_routing_semantics`
- `tests::delivery::consumer_pacing_differences_do_not_change_stream_meaning`
- `tests::delivery::illegal_coalescing_boundary_fails_explicitly`
- `tests::replay::stream_replay_matches_original_canonical_records`
- `tests::pressure::backpressure_changes_pacing_without_semantic_drift`

## Target API And Module Plan

### New Files Expected

- `crates/worth-runtime-bridge/src/stream/mod.rs`
- `crates/worth-runtime-bridge/src/stream/declaration.rs`
- `crates/worth-runtime-bridge/src/stream/protocol.rs`
- `crates/worth-runtime-bridge/src/stream/contracts.rs`
- `crates/worth-runtime-bridge/src/stream/window.rs`
- `crates/worth-runtime-bridge/src/stream/checkpoints.rs`
- `crates/worth-runtime-bridge/src/stream/replay.rs`
- `crates/worth-runtime-bridge/src/stream/coalescing.rs`
- `crates/worth-runtime-bridge/src/stream/backpressure.rs`
- `crates/worth-runtime-bridge/src/delivery/stream.rs`
- `crates/worth-runtime-bridge/src/diagnostics/stream.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/stream_restart.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/multi_consumer.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/stream_backpressure.rs`
- `crates/worth-runtime-bridge/src/tests/stream/contracts.rs`
- `crates/worth-runtime-bridge/src/tests/stream/delivery.rs`
- `crates/worth-runtime-bridge/src/tests/stream/replay.rs`
- `crates/worth-runtime-bridge/src/tests/stream/pressure.rs`

### Existing Files Expected To Change

- [facade.rs](../../crates/worth-runtime-bridge/src/facade.rs)
- [lib.rs](../../crates/worth-runtime-bridge/src/lib.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/delivery/mod.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/diagnostics/mod.rs)
- [replay.rs](../../crates/worth-runtime-bridge/src/historical/replay.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/harness/mod.rs)

## Implementation Phases

Milestone 6 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished protocol foundations with host-local cursor
glue.

### Phase M6.0 - Protocol Taxonomy And Boundary Lock

Purpose:

- define the one change-stream declaration surface
- define the canonical protocol vocabulary
- lock what truth publishes versus what the bridge interprets

Required work:

- define `ChangeStreamDeclaration`
- define canonical stream member identity and stream window identity
- define `CanonicalStreamPosition`
- define the first closed Milestone 6 consumer-shape taxonomy
- define checkpoint, resume, replay, coalescing, and backpressure vocabulary
- define explicit unsupported protocol classes

Exit criteria:

- the declaration surface is singular and explicit
- host offsets are not the public protocol identity
- unsupported consumer or resume modes are named rather than deferred

### Phase M6.1 - Consumer Contract Resolution And Admission

Purpose:

- resolve protocol applicability before delivery begins

Required work:

- define `ValidatedStreamProtocol`
- define `ResolvedConsumerContract`
- define the closed set of consumer-shape, resume-mode, checkpoint-mode, and
  coalescing outcomes
- define which outcomes become admitted plans versus typed rejections
- define canonical ordering and digest basis for contract resolution

Exit criteria:

- consumer applicability is fully resolved before stream-window planning
- unsupported combinations become typed outcomes rather than delivery-time drift
- later phases can consume one admitted contract instead of ambient knobs

### Phase M6.2 - Stream Window Planning, Coalescing, And Checkpoint Basis

Purpose:

- lower canonical stream material into deterministic consumer-specific windows

Required work:

- define `PlannedChangeStreamWindow`
- define `MaterializedChangeBatch`
- define the exact stream member fields that participate in window identity
- define the legal coalescing families and prohibited boundaries
- define `ConsumerCheckpointToken`
- define exact acknowledged-member identity basis for checkpoints
- add exact counters and decision-log records

Exit criteria:

- identical canonical stream material and contract truth lower to identical
  windows
- checkpoint truth is explicit and replay-safe
- coalescing is planned rather than inferred from transport buffers

### Phase M6.3 - Replay, Resume, And Protocol Diagnostics

Purpose:

- make restart and replay mechanically explainable

Required work:

- add canonical stream replay records and checkpoint replay records
- define typed resume mismatch and replay mismatch classes
- add explanation reconstruction over canonical protocol truth
- define identity relationships among declaration, contract, checkpoint, and
  replay records

Exit criteria:

- replay and resume can be validated without ambient host interpretation
- mismatch classes are closed, typed, and attributable
- canonical diagnostics can explain what was acknowledged and resumed

### Phase M6.4 - Multi-Consumer Certification And Hostile Coverage

Purpose:

- make protocol claims certifiable rather than plausible

Required work:

- add `worth-harness` fixtures, parity suites, and certification matrices for
  restart, replay, multi-consumer pacing, and pressure conditions
- add hostile checkpoint truncation, protocol mismatch, and illegal coalescing
  lanes
- add exact counter assertions for stream width, checkpoint width, and pressure
  classes

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- multi-consumer parity is mechanically certified
- protocol richness changes observability only, not canonical interpretation

## Explicit Failure Taxonomy For Milestone 6

Milestone 6 must ship typed bridge failures for at least:

- unsupported consumer shape
- unsupported resume mode
- protocol version mismatch
- unresolved consumer contract conflict
- checkpoint contract mismatch
- checkpoint stream mismatch
- truncated checkpoint basis
- illegal coalescing boundary
- non-idempotent duplicate observation
- backpressure policy violation
- stream replay mismatch
- stream delivery rejection

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- exposing raw host feed offsets as the public bridge resume contract
- letting each consumer define its own ad hoc checkpoint meaning
- treating coalescing as a hidden buffer side effect
- letting backpressure reorder or semantically merge canonical stream members
- reinterpreting stream members differently during replay than during live consumption
- productizing a single host adapter and calling it the protocol
- hiding protocol behavior behind elapsed-time metrics with no explanatory counters

## Sequencing Notes

Milestone 6 must land before:

- Milestone 7 reactive source protocol productization, because read contracts
  and change-stream contracts should become explicit together
- Milestone 9 merge-aware bridge semantics, because merge-bearing histories need
  replay-safe consumption contracts rather than one-off adapter logic
- Milestone 10 speculative coordination, because speculative flows will still
  need explicit checkpoint, replay, and resume semantics
- Milestone 11 policy propagation, because protocol-visible backpressure,
  coalescing, and richness decisions should already have a named contract
- Milestone 13 bridge certification, because a bridge is not certifiable while
  its stream consumption semantics remain host-shaped

Milestone 6 must not attempt to pre-solve:

- generalized read-source setup and registration
- merge ontology or multi-parent interpretation
- preview lifecycle
- execution-policy propagation
- derived writeback

Those become stronger because Milestone 6 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the bridge cannot honestly be a protocol boundary while
stream consumption still depends on one host-shaped feed path and one
consumer-specific cursor story.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of equating transport offsets with canonical stream truth, hiding
coalescing behind buffers, and letting backpressure or restart redefine
meaning.

The milestone preserves authority boundaries because truth still owns canonical
patch order, signal still owns downstream execution, and the bridge owns only
the consumption protocol and replay/checkpoint interpretation between them.

The milestone defines proof obligations rather than implementation chores
because canonical stream identity, typed consumer contracts, explicit
checkpoint/replay semantics, bounded coalescing, and multi-consumer parity are
required for closeout.

A competent engineer should be able to map this spec into honest stream
artifacts, checkpoint types, consumer-contract modules, counters, and harness
suites without inventing the architecture during implementation.

## Closeout Standard

Milestone 6 is complete only when all of the following are true:

- canonical truth change members lower into one canonical stream protocol
  surface per consumer contract
- stream position, checkpoint truth, and consumer contract truth remain
  structurally distinct
- resume and replay depend on canonical protocol records rather than ambient
  host interpretation
- multiple admitted consumer shapes preserve the same canonical stream meaning
- coalescing is explicit, bounded, reconstructable, and semantically honest
- backpressure changes pacing only and remains visible in counters and decision
  records
- protocol truth is replay-safe and diagnostics-tier-invariant
- harness certification proves restart parity, replay parity, multi-consumer
  determinism, and explicit failure behavior under hostile stream pressure

If code lands but stream consumption still depends on raw host offsets, hidden
buffer coalescing, consumer-local checkpoint folklore, replay-time
reinterpretation, or explanation-only protocol mismatch handling, Milestone 6
is not complete.
