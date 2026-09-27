# Milestone 10: Throughput And On-Demand Observation Vocabulary

## Status

Closed. Phases 1–9 have phase-local qa-loop, qa-tests, and code-quality-qa
review records in the phase ledger. Store durability, WAL, and crash courts
remain a later Store-owned milestone and are not part of this close.

Phases 1–7 establish the shared vocabulary, disposition and work disclosure
laws, public Foundational handoff, Signal's admitted → resolved → installed
policy path, on-demand observation sessions, and branch/restore absence
parity. Phase 8 closed the production courtroom, measured packet, and
deletion gate. `SignalRuntimePolicy::operational()` is the public
Throughput + OnDemand constructor. The idle-versus-introspective packet
remains evidence that OnDemand Operational beats Forensic Continuous; it
does not justify a second performance-implying constructor. Phase 9 cut
every remaining six-axis `FoundationalProfileSet` construction over to the
two new axes. It does not claim Store durability certification.

## Goal

Extend the shared Foundational profile and performance language with two
orthogonal concepts that the WORTH runtimes currently conflate:

- the operational objective used to choose among already-correct execution
  strategies
- the activation rule that decides when optional observation work is allowed
  to enter an ordinary execution lane

Then adopt that vocabulary in `worth-signal` through a compiler-visible runtime
policy pipeline and ship a real Signal throughput profile whose ordinary path
does not construct, retain, index, or count optional diagnostics, descriptive
lineage, provenance, replay sidecars, or performed-observation evidence unless
an observation session was explicitly admitted before execution.

The milestone standardizes shared meaning in `worth-foundational`, keeps
runtime policy execution in each adopting runtime, and uses `worth-signal` as
the first real adoption that proves the vocabulary can remove cost rather than
merely rename existing tiers.

## Central Claim

After Milestone 10, a WORTH boundary can state all of the following without
semantic ambiguity:

- whether the runtime is optimizing for bounded latency, balanced operation,
  or sustained throughput
- whether optional observation is continuously active or must be explicitly
  activated
- what diagnostic richness is eligible when observation is active
- how long eligible evidence is retained or delivered
- what optional work a performance claim included and excluded
- whether a descriptive surface is absent because it was not activated rather
  than because it was unsupported, redacted, lost, or not reconstructable

`Throughput` is an optimization objective, not a correctness level, durability
level, evidence strength, or promise that a particular speed was achieved.

`OnDemand` is an activation rule, not permission to erase authority,
correctness, security, durability, stable lineage identity, replay linkage, or
contractual cost truth.

## Roadmap Placement

Milestone 10 builds on the profile, artifact, diagnostics,
lineage/provenance/receipt, and performance vocabularies established by
Milestones 3 through 8. Implementation may not treat any still-open predecessor
surface as complete merely because this specification depends on it. Milestone
10 follows Milestone 9's scoped-merge vocabulary in numbering, but does not
depend on scoped-merge execution.

Milestone 10 must land before cross-crate migration and closure. Migration is
therefore Milestone 11, not a second Milestone 9. A migration milestone cannot
honestly converge runtime profiles while `worth-signal`, `worth-store`, and
other runtimes still use incompatible meanings for throughput, observation,
richness, and retention.

This milestone has one required adopting runtime: `worth-signal`.

`worth-store` adoption is a committed successor handoff. Milestone 10 must
design vocabulary that fits Store without importing durability, WAL, MVCC,
compaction, or storage policy into Foundational, but it does not claim Store's
hot paths have adopted the vocabulary until a Store-owned milestone proves
that through the real durable and embedded composition roots.

## Governing Inherited Contracts

### Foundational Milestone 3

Milestone 3 established that Foundational owns shared profile meaning while
domain runtimes own policy execution. It also established requested, admitted,
and materialized profile progression, canonical profile identity, typed
profile differences, and central descriptive elision.

Milestone 10 extends that closed model. It must not add a parallel throughput
configuration bag or a Signal-only profile dialect beside it.

### Foundational Milestone 7

Milestone 7 keeps lineage, provenance, and receipts distinct. Throughput may
elide optional descriptive lineage records and provenance materialization, but
it may not erase a receipt required to attest an effect or reuse descriptive
provenance as authority.

### Foundational Milestone 8

Milestone 8 established performance boundaries, evidence strength, execution
temperature, included/excluded work, structural counters, and policy-admission
receipts. Milestone 10 extends that vocabulary with precise optional
observation work classes and binds active profile identity to performance
claims. It does not create another performance report system.

### Signal Performance Architecture

Signal already declares the invariant:

> lineage identity always; rich materialization by profile; compact hot-path
> semantic facts in production; reconstructable deep provenance in richer
> lanes

Milestone 10 makes this real. Stable operational identity remains universal.
Rich record construction becomes policy- and session-controlled.

### Store Vision

Store already separates durable, embedded, and absent modes. Those are
durability and lifecycle contracts, not observation profiles. Milestone 10
must preserve that separation so a future `Durable + Throughput + OnDemand`
composition still performs every WAL, acknowledgement, recovery, integrity,
tenant, and custody action required by durable mode.

## Current Boundary

### Foundational today

`FoundationalProfileSet` currently contains six axes:

- diagnostic richness
- support posture
- compatibility posture
- admission/readiness posture
- retention/delivery posture
- certification posture

`DiagnosticRichnessProfile::OperationalMinimal` describes eligible richness,
but it does not say whether optional observation work is continuously active.

`RetentionDeliveryProfile::Ephemeral` describes lifetime, but it does not say
whether evidence should be constructed in the first place.

The current descriptive elision vocabulary distinguishes `FullFidelity` from
`OperationalSummary`, but the materialization planner cannot express that an
otherwise eligible surface is absent because no observation was activated.

The performance vocabulary can disclose authoritative, planning,
publication, replay, support, and forensic work, but its work classes are too
coarse to distinguish counter capture, diagnostic capture, descriptive
lineage maintenance, provenance capture, and replay-sidecar maintenance.

### Signal today

`SignalRuntimePolicy` is a runtime-policy request compiled by Signal rather
than a diagnostics-owned authority. It also selects:

- path class
- artifact policy
- execution strategy
- maintenance strategy
- parallel admission thresholds
- snapshot restore lineage behavior
- frontier tracing behavior

`DiagnosticsTier` remains a descriptive preset input. Presets choose coherent
defaults, but the compiler keeps execution objective and observation
activation independent: a caller may request throughput with continuous
observation, or balanced/latency-bounded execution with on-demand observation.

Signal's Operational tier is materially cheaper than Development in the
recorded fintech mixed-fanout baseline, but it still performs optional work on
ordinary paths:

- lineage records are cloned into node and artifact indexes
- lineage sequences and records are constructed during invalidation and
  finalized evaluation
- compact trace summaries and frontier summaries are retained
- general telemetry is updated
- twenty-four performed-counter atomics are updated even when no performed
  observation is active
- explanation and provenance remain reconstructable by policy

This is evidence that the profile distinction matters and evidence that the
lowest existing tier is not yet a true on-demand observation lane.

### Store today

Store has the correct high-level durability modes, but no shared observation
activation or execution-objective vocabulary. Future Store adoption must not
mistake WAL, MVCC/version history, canonical commit envelopes, CDC cursor
authority, schema boundaries, integrity evidence, or authoritative identity
lineage for optional diagnostics.

## Adversarial Constraint

The milestone must survive this condition:

> A long-lived geometry or simulation runtime executes repeated sparse edits
> across a large aspect-local graph while no observer is attached. The same
> semantic workload is also run under an explicit performed-observation
> session, through checkpoint/restore, and under richer diagnostic profiles.
> The no-observer throughput lane must perform zero optional observation work,
> while every lane produces identical operational truth, stable identity,
> invalidation causes, scheduling outcomes, committed deltas, and deterministic
> replay linkage. Activating observation must restore exactly the requested
> evidence without allowing post-hoc receipt minting.

A naive implementation fails if it does any of the following:

- renames Operational to Throughput while retaining the same hot-path work
- treats throughput as permission to weaken correctness or stable identity
- gates final materialization but still constructs and discards rich records
- gates lineage retention but still allocates, clones, hashes, or indexes the
  records
- skips counters only at receipt assembly while still performing atomic writes
- lets a diagnostic tier continue to select execution or maintenance strategy
- allows observation to be activated after execution and then claims a
  performed receipt
- changes branch, snapshot, restore, replay, reuse, or comparator outcomes
  because descriptive history is absent
- hides omitted evidence behind `None`, an empty vector, or an apparently
  complete report
- lets Store interpret Throughput as weaker durability

## Decisive Courtroom

### Production entry surface

The Signal proof must begin at the public `SignalRuntime` or `SignalApp`
composition root, install policy through the public runtime builder, mutate the
graph through ordinary production mutation APIs, execute through the ordinary
planner/evaluator/publication path, and observe consequences through public
operational and diagnostic surfaces.

A raw `SignalGraph` reconstruction of the scenario may supplement local proofs
but cannot certify the adoption claim.

### World

Create one deterministic domain-agnostic kernel world with:

- at least 16,384 nodes in the principal performance lane
- multiple independent regions and aspect-local branches
- a depth path long enough to expose repeated per-hop observation work
- mixed exact-aspect, partition, and detail-local dependencies
- comparator suppression and unsuppressed output commits
- branch creation, snapshot capture, restore, and continued execution
- both serial and parallel-admitted independent work

The small ordinary gate may use a 1,024-node version of the same compiled world.
The scheduled performance lane must also exercise at least 65,536 nodes or the
largest size the recorded environment can run without paging; the report must
state which bound governed.

Run at least 120 consecutive edit batches. Each batch touches approximately
one percent of nodes, with fixed seeds and a mixture of repeated same-aspect
edits, disjoint-region edits, and suppressed changes.

### Profile matrix

Run the identical mutation program under these profile combinations:

| Name | Execution objective | Observation activation | Richness | Purpose |
| --- | --- | --- | --- | --- |
| balanced continuous | Balanced | Continuous | Operational minimal | Current operational comparison |
| throughput idle | Throughput | On demand, inactive | Operational minimal | Principal zero-observation lane |
| throughput observed | Throughput | On demand, explicitly active | Operational minimal | Pay-when-requested proof |
| throughput rich session | Throughput | On demand, explicitly active | Standard or forensic | Orthogonality proof |
| introspective | Balanced | Continuous | Forensic | Rich-reference lane |
| latency bounded | Latency bounded | On demand | Operational minimal | Objective distinction proof |

### Independent operational oracle

For every run, independently compare a canonical operational digest built from:

- committed output values and output identities
- output versions and commit ordinals
- node operational states
- direct invalidation bases and canonical dependency cause sets
- dependency revisions and readiness epochs
- dependency snapshots
- scheduled and performed semantic work identities
- comparator/reuse verdicts required for correctness
- committed delta and publication order
- branch heads, snapshot operational truth, and restore/readmission outcomes
- stable lineage artifact identity and replay linkage required by Signal's
  universal semantics

Diagnostic records, retained history, performed counters, timing, allocation
counts, and descriptive sidecars are deliberately excluded from the
operational digest and compared separately.

### Required no-observer outcome

The throughput-idle run must have:

- zero diagnostic fact capture
- zero descriptive lineage record construction and index maintenance
- zero provenance fact capture
- zero frontier/wave sidecar retention
- zero replay-detail sidecar maintenance beyond compact facts required for
  deterministic operational replay linkage
- zero performed-counter atomic updates
- zero performed work-record retention
- zero optional observation allocations
- bounded diagnostic state independent of edit count

Stable lineage IDs, execution record IDs, semantic segment IDs, and any compact
facts required for correct reuse, branch behavior, restore, or deterministic
replay are not counted as optional observation work and must remain present.

### Required explicit-observation outcome

An observation session must be admitted before the operation or batch it
observes. While it is active:

- only the selected observation surfaces are captured
- performed counters and work identities describe only the admitted scope
- finishing the session consumes its active authority and returns a performed
  receipt
- a second finish, a stale session, or a post-hoc request is denied with a
  typed outcome
- nested or concurrent sessions are either independently identified and
  merged by explicit law or rejected before effects; last-writer-wins
  generation replacement is forbidden

### Restore pressure

Capture and restore checkpoints across these transitions:

- throughput idle to throughput idle
- Development/Forensic to throughput idle
- throughput idle to Development/Forensic
- active observation session interrupted by checkpoint or snapshot boundary

Operational truth and stable identities must survive. Optional historical
evidence that was never captured must remain explicitly unavailable with an
observation-specific absence cause. A richer post-restore profile may capture
new evidence but may not counterfeit missing historical evidence.

### Performance acceptance

Structural acceptance is primary:

- every optional observation work counter named above is exactly zero in the
  throughput-idle lane
- each counter becomes nonzero only when its corresponding explicit session
  requests that surface
- work stays delta- and locality-bounded

Measured acceptance is also required because a permanent Throughput profile
must provide real value:

- report warm median and p95 latency plus completed batches per second
- record hardware, build profile, feature set, thread count, workload seed,
  node count, batch width, and repetition count
- the throughput-idle lane may not regress median by more than three percent or
  p95 by more than five percent against balanced-continuous operation on any
  named principal workload
- at least one diagnostic-pressure or lineage-pressure workload must improve
  median completed-work throughput by at least ten percent

The measured bar remains observation-cost evidence for the Throughput +
OnDemand lane. It does not justify a second public constructor under a
performance-implying name. `SignalRuntimePolicy::operational()` is the only
public constructor for that posture.

### Mutation sensitivity

The courtroom must turn red if a compile-valid mutation:

- removes the observation-active guard from any representative counter,
  lineage, provenance, diagnostic, or sidecar owner
- moves a gate after record construction or allocation
- lets `DiagnosticsTier` choose an execution or maintenance strategy again
- changes an operational output only in Throughput
- omits stable lineage identity or replay linkage in Throughput
- allows a performed receipt without a pre-execution admitted session
- fabricates unavailable historical evidence after restore
- reports excluded observation work as included execution work or vice versa

## Product Decision Lock

### Two new profile axes

Milestone 10 adds exactly two shared profile families.

```rust
pub enum ExecutionObjectiveProfile {
    LatencyBounded,
    Balanced,
    Throughput,
}

pub enum ObservationActivationProfile {
    OnDemand,
    Continuous,
}
```

`ExecutionObjectiveProfile` chooses among correct, admitted strategies:

- `LatencyBounded` prioritizes bounded foreground work and predictable tails
  over maximum aggregate completion rate.
- `Balanced` chooses the runtime's ordinary compromise among latency,
  throughput, memory, and maintenance pressure.
- `Throughput` prioritizes sustained completed work and may prefer batching,
  amortized maintenance, wider parallel staging, pools, or arenas within
  declared budgets.

`ObservationActivationProfile` decides when optional observation work may
begin:

- `OnDemand` requires a runtime-owned observation admission before any
  selected optional capture occurs.
- `Continuous` requires the runtime to capture every optional surface selected
  by its admitted local observation plan on each governed boundary. Richness,
  retention, disclosure, and budget law still determine the eligible plan and
  must report any typed omission.

The axes are independent. Throughput does not imply OnDemand. OnDemand does not
imply Throughput. A certification run may use Throughput with an explicitly
active rich observation session, and a balanced runtime may use OnDemand.

### Existing profile axes keep their meaning

- diagnostic richness answers **what eligible evidence contains**
- observation activation answers **when optional evidence work starts**
- retention/delivery answers **how long evidence survives and where it may be
  delivered**
- support posture answers **which support audience posture is admitted**
- certification posture answers **what evidence strength is claimed**
- execution objective answers **which correct cost tradeoff the runtime should
  optimize**

No axis may be inferred from another.

### Actual observation disposition

Foundational must add descriptive vocabulary for the disposition that actually
applied at a boundary:

```rust
pub enum FoundationalObservationActivationScope {
    Operation,
    Batch,
    ManagedSession,
}

pub enum FoundationalObservationDisposition {
    Inactive,
    Continuous,
    ExplicitlyActivated {
        scope: FoundationalObservationActivationScope,
        session: BoundaryHandle,
        observed_epoch: BoundaryEpoch,
    },
}
```

These types describe what happened. They do not authorize capture, open
disclosure, or mint a performed receipt. Each runtime owns the authority that
admits an observation and the managed resource that keeps it alive.

The session handle and epoch provide canonical correlation only. They are not
capabilities and cannot be promoted back into runtime authority.

Foundational materialization planning must consume both the materialized
profile and the actual observation disposition. It may not infer an activated
session from `ObservationActivationProfile::OnDemand`, and the legacy
profile-only planning shape may survive only where the target has no
activation-sensitive surfaces. The current descriptive target inventories are
activation-sensitive, so their profile-only shorthand returns a typed
`ObservationDispositionRequired` denial for `OnDemand`; callers must select an
actual inactive or explicitly activated disposition. Stronger performed or certified artifacts must
also carry the runtime's real admission/performed proof; a freely constructed
descriptive disposition cannot open that lane.

### Profile composition and identity

`FoundationalProfileSet` becomes an eight-axis total profile. Partial builders,
unnamed defaults, maps, and optional new fields are forbidden.

Canonical profile basis and identity must include both new axes. Changing only
the execution objective or only observation activation must change canonical
profile identity and produce a family-specific difference classification.

The existing single optional narrowing record is insufficient once a runtime
may resolve several independent axes at one admission boundary. Replace it
with a canonical, family-keyed resolution record set that:

- carries every requested-to-admitted and admitted-to-materialized adjustment
- rejects duplicate records for one family and transition
- orders records canonically
- distinguishes monotonic richness/retention narrowing from objective
  adjustment and observation-activation adjustment
- cannot be used as authority to execute the resulting policy

### Surface absence law

Add an observation-specific absence cause such as
`ObservationNotActivated`. It must remain distinct from:

- omitted by active richness
- denied by budget
- not retained
- not reconstructable
- deferred by support posture
- uncertified for requested posture
- redacted or disclosure-denied evidence

An inactive OnDemand profile must not look like an empty but complete report.

### Performance work disclosure

Milestone 8 performance claims must be able to include or exclude these
separate work classes:

- structural counter capture
- diagnostic fact capture
- descriptive lineage record maintenance
- provenance fact capture
- replay sidecar maintenance

Stable identity maintenance, authority validation, and correctness-required
replay linkage stay in the appropriate authoritative work class. They cannot be
misreported as optional observation merely because diagnostics later displays
them.

Any claim that names a throughput or observation-sensitive execution lane must
bind the canonical admitted profile identity and the actual observation
disposition. A claim with the right counters but a different profile basis is a
different claim, not equivalent evidence.

### Authority boundary

`worth-foundational` owns:

- canonical profile meanings
- profile composition, comparison, identity, and difference law
- descriptive observation disposition and absence vocabulary
- shared optional-observation work classes
- boundary attachment and report meaning

`worth-proof` owns:

- proof-bearing progression artifacts where the existing requested,
  admitted, materialized, certified, or readmitted lanes require proof
- performed evidence that a completed boundary actually occurred
- Milestone 10 readiness certification

`worth-proof` does not own:

- execution-objective policy
- observation scheduling
- counter storage
- diagnostics capture
- lineage storage
- Signal runtime sessions
- Store durability admission

`worth-signal` owns:

- Signal policy admission and lowering
- Signal observation-session authority and lifecycle
- Signal hot-path gates and local representations
- Signal performed-counter and lineage/provenance capture
- Signal branch, snapshot, restore, and replay consequences

No public governed surface may accept a generic `AuthorityMarker`. Signal must
use concrete Signal-owned admission authorities and the concrete proof types
that attest performed execution.

## Signal Architectural Destination

### Public policy shape

Signal keeps one public runtime policy entry surface, but it is no longer owned
by diagnostics.

```rust
SignalRuntime::builder()
    .runtime_policy(SignalRuntimePolicy::operational())
    .build()?;
```

The preset is inspectable before construction:

```rust
let policy = SignalRuntimePolicy::operational();

assert_eq!(
    policy.foundational_profile().execution_objective(),
    ExecutionObjectiveProfile::Throughput,
);
assert_eq!(
    policy.foundational_profile().observation_activation(),
    ObservationActivationProfile::OnDemand,
);
```

Signal may retain named domain presets such as game, kernel, or fintech, but
each preset must lower visibly into the same orthogonal fields. Domain names
may not conceal a second policy engine.

### Compiler-visible policy progression

Signal must establish this phase sequence:

```text
SignalRuntimePolicyRequest
    -> AdmittedSignalRuntimePolicy
    -> ResolvedSignalRuntimePolicy
    -> InstalledSignalRuntimePolicy
```

- Request carries caller intent and a complete Foundational requested profile.
- Admission validates supported combinations, disclosure posture, runtime
  capabilities, and budgets before graph construction or mutation.
- Resolution fixes execution strategy, maintenance strategy, parallel
  thresholds, observation gates, retention budgets, reconstruction posture,
  and snapshot/replay behavior.
- Installation places the immutable resolved policy into the runtime. Hot
  owners consume resolved fields and may not re-derive strategy from
  `DiagnosticsTier`.

Each phase consumes the exact sealed artifact produced by its predecessor.
Adding a required resolved subsystem must break construction, fork, snapshot,
restore, and reconfiguration sites until it is propagated.

### Resolved Signal policy responsibilities

The resolved policy must have separately owned constituents:

- execution strategy
- maintenance strategy
- parallel admission
- observation activation and selected-surface gates
- diagnostic richness
- descriptive retention
- reconstruction eligibility
- snapshot and replay diagnostic posture
- correctness-required identity and replay-linkage posture

The last item is not optional and may not be disabled by profile.

### Operational production preset

`SignalRuntimePolicy::operational()` is the public constructor for the
production Throughput + OnDemand posture. Do not add a second constructor
that installs this same policy under a performance-implying name.

That preset must resolve at least:

- `ExecutionObjectiveProfile::Throughput`
- `ObservationActivationProfile::OnDemand`
- `DiagnosticRichnessProfile::OperationalMinimal`
- ephemeral or otherwise non-retained optional descriptive evidence
- sparse/delta-local execution where admitted by graph shape
- density-aware or explicitly scheduled maintenance
- parallel thresholds derived from the throughput objective, not diagnostic
  richness
- no optional reconstruction promise for evidence that was never captured

The preset must not disable correctness validation, canonical causes, stable
lineage identity, deterministic commit ordering, stale checks, comparator law,
branch identity, checkpoint authority, or replay linkage required by supported
Signal semantics.

### Observation session

Signal must expose one managed observation lifecycle rather than a global
boolean:

```rust
let (_result, receipt) = runtime.observe_execution(
    SignalObservationRequest::operation()
        .with_performed_counters()
        .with_descriptive_lineage(),
    |_runtime| {
        // Evaluate the requested target through the runtime's normal API.
        Ok(())
    },
)?;
```

The request is admitted before effects. The session owns its liveness,
selected surfaces, scope, disclosure posture, and completion. Dropping or
cancelling it produces an explicit non-performed outcome and cannot mint a
receipt.

### Hot-path cutover

The gate must sit before optional construction at each responsibility owner:

- performed-counter state checks activation before any atomic update
- executed-work retention checks activation before locking or cloning
- diagnostic recorders check the resolved selected-surface gate before fact
  construction
- lineage recorders preserve compact identity but check the gate before
  sequence allocation, record construction, cloning, and index mutation
- provenance recorders check before fact construction
- frontier/wave recorders check before allocating or retaining sidecars
- checkpoint writers omit optional diagnostic payloads before serialization
  work when the resolved profile does not retain them

Gating only at report rendering, queue trimming, or checkpoint output is
insufficient.

### Truth and descriptive lineage split

Signal must structurally separate:

- stable artifact/operation identity and replay linkage required by runtime
  semantics
- optional descriptive lineage records and query indexes

If current branch, reuse, restore, or replay correctness depends on a
diagnostics-owned record store, that dependency must be removed before the
store can be skipped. The throughput profile may not expose a hidden
capability regression.

### Reconfiguration

Runtime policy changes occur only at an admitted runtime boundary. A policy
change cannot race an active execution batch or observation session.

Changing from OnDemand to Continuous begins capture prospectively. Changing
from Continuous to OnDemand stops future optional capture after the transition
commits. Neither transition rewrites past truth or fabricates past evidence.

## Store Successor Contract

The shared vocabulary must support this future Store composition without
special cases:

```text
durability = Durable
execution objective = Throughput
observation activation = OnDemand
diagnostic richness = OperationalMinimal
```

Future Store adoption may elide:

- verbose storage diagnostics
- per-operation trace rows
- expanded provenance/support reports
- continuously updated certification counters
- diagnostic sidecars
- eager reconstruction of audit views

It may not elide:

- WAL records required for acknowledged durability
- canonical commit envelopes
- MVCC/version and branch ancestry
- crash-recovery sequencing and publication progress
- schema boundaries
- integrity, authenticity, tenant, key, and custody evidence
- CDC cursor authority
- authoritative identity-lineage events
- durability barriers and their contractual cost disclosure

Store durability modes remain Store-owned and orthogonal to all Foundational
profile axes.

## Required Destination Topology

### Foundational production topology

```text
crates/worth-foundational/src/
  profiles/
    mod.rs                                      [modified]
    families/
      mod.rs                                    [replaces families.rs]
      descriptive.rs                            [moved: richness/support/retention]
      compatibility.rs                          [moved: compatibility/readiness/certification]
      execution_objective.rs                    [created]
      observation_activation.rs                 [created]
    composition.rs                              [modified: eight-axis total set]
    progression.rs                              [modified: canonical multi-family resolution]
    difference.rs                               [modified]
    identity.rs                                 [modified]
    materialization/
      inventory.rs                              [modified]
      planning.rs                               [modified]
      vocabulary.rs                             [modified: disposition/absence]
    readiness/
      ...                                       [modified for Milestone 10 coverage]
  performance/
    primitives/
      work.rs                                   [modified: observation work classes]
    claims/
      ...                                       [modified: profile identity/disposition attachment]
    policy/
      ...                                       [modified: objective/activation admission disclosure]
    readiness/
      ...                                       [modified for new work disclosures]
  profiles_api/
    ...                                         [modified stable facade only]
  performance_api/
    ...                                         [modified stable facade only]
```

The `profiles/families/` directory is classified by profile meaning. It does
not contain runtime policy execution. `performance/` continues to own claims
about work and cost, not profile composition.

### Foundational test topology

```text
crates/worth-foundational/tests/certification/
  profiles/
    execution_objective.rs                      [created]
    observation_activation.rs                   [created]
    composition.rs                              [modified]
    progression_and_identity.rs                 [modified]
    observation_materialization.rs              [created]
  performance/
    observation_work_disclosure.rs              [created]
  composition/
    throughput_profile_boundary.rs              [created]
```

Tests remain organized by responsibility rather than milestone number.

### Signal production topology

```text
crates/worth-signal/src/
  runtime_policy/
    mod.rs                                      [created stable internal root]
    request.rs                                  [created]
    admission.rs                                [created]
    definition.rs                               [created caller request definition]
    compiler.rs                                 [created request admission/compiler owner]
    presets.rs                                  [moved/replaced]
    objective.rs                                [created independent objective lowering]
    parallel.rs                                 [created runtime parallel admission]
    resolved.rs                                 [created sealed resolved/installed authority]
  observation/
    session/
      mod.rs                                    [created]
      request.rs                                [created]
      admission.rs                              [created]
      active.rs                                 [created]
      completion.rs                             [created]
    performed/
      receipt.rs                                [moved/reworked]
      counters.rs                               [moved/reworked]
  diagnostics/
    policy/                                     [reduced to diagnostic-only meaning]
    runtime/
      recorder/                                 [modified: pre-construction gates]
      state/                                    [modified: optional descriptive stores]
  data/graph/runtime/graph/
    observation_state.rs                        [modified]
    performed_counter_state.rs                  [modified]
  state/
    ...                                         [modified: checkpoint/restore policy disposition]
  facade/
    runtime.rs                                  [modified stable public entry]
```

`runtime_policy/` is the cross-subsystem policy compiler. It may orchestrate
diagnostics, execution, maintenance, and observation policy, but it may not own
their runtime state. `observation/session/` owns the managed activation
lifecycle. Diagnostics keeps diagnostic meaning and storage only.

### Signal proof topology

```text
crates/worth-signal/src/tests/
  runtime_policy/
    foundational_lowering.rs                    [created]
    objective_richness_orthogonality.rs         [created]
    compiler_progression.rs                     [created]
  observability/
    on_demand_capture.rs                        [created]
    session_lifecycle.rs                        [created]
    checkpoint_profile_transitions.rs           [created]
  performance_profiles/
    throughput_world.rs                         [created]
    throughput_parity.rs                        [created]
    throughput_slopes.rs                        [created]
```

Existing relevant tests should be moved or consolidated by responsibility
rather than duplicated under the new names.

## Forbidden Destination Shapes

- no `ultra`, `fast`, `minimal`, or `no_diagnostics` boolean that combines the
  two new axes
- no generic `policy.rs` bag containing Foundational and Signal execution law
- no Signal `DiagnosticsTier` switch inside planner strategy selection after
  policy compilation
- no runtime-global ambient observation flag
- no optional-observation checks scattered through semantic leaf producers
- no second lineage identity or replay identity used only by Throughput
- no compile feature that silently changes runtime semantics
- no Store durability enum in Foundational profiles
- no Foundational executor, telemetry store, session manager, WAL policy, or
  scheduler
- no compatibility alias that leaves the old diagnostics-owned Signal policy
  as a second ordinary path

## Ordered Phase Plan

### Phase 1: Boundary Freeze, Baseline, And Red Controls

Implementation status: implemented; Phase 1 gate closed. The boundary and
baseline record are established, and the pre-cutover Operational red control
is executable. The later zero-work/session certification remains Phase 6/8
work.

What becomes true:

- the current six-axis profile shape, materialization absence law, M8 work
  classes, Signal tier-to-strategy coupling, unconditional counter writes, and
  lineage recorder entrances are inventoried
- current Operational, Development, and Forensic semantic and performance
  baselines are recorded
- the decisive world exists and is red against at least representative
  unconditional observation work

Required evidence:

- one executable red control proving no-observer Operational still performs
  optional counter and lineage work
- canonical operational digest parity for current tiers before repair
- recorded benchmark metadata and structural counters

The next phase may trust the current boundary and independent oracle, but may
not trust the new vocabulary yet.

### Phase 2: Foundational Objective And Activation Vocabulary

Implementation status: complete; Phase 2 gate closed after phase-local
qa-loop, qa-tests, and code-quality-qa review. Vocabulary, total profile shape,
identity, difference, and the multi-family resolution ledger are implemented
and covered by focused certification tests.

What becomes true:

- the two new profile families exist with exact meanings
- `FoundationalProfileSet` is an eight-axis total profile
- composition rejects partial, duplicate, and incoherent construction
- canonical profile identity and family-specific difference law include both
  axes
- requested/admitted/materialized progression carries canonical multi-family
  resolution records

Mechanical enforcement:

- old six-axis constructors stop compiling
- raw strings and maps cannot satisfy the new profile APIs
- one resolution record cannot silently hide another changed family

Required evidence:

- a 3×2 orthogonality matrix over every execution-objective and
  observation-activation value, with each selected value preserved
  independently
- independent objective-only and activation-only identity/difference twins,
  exact canonical tokens, and a combined-change incompatible case
- canonical resolution-ledger adversaries for omission, unexpected family,
  duplicate insertion, reverse insertion order, relation mismatch, and
  requested → admitted → materialized propagation
- compile-fail coverage for old six-axis construction, raw strings, adjacent
  family substitution, duplicate assignment, and unnamed defaults

The next phase may trust canonical profile meaning and identity.

### Phase 3: Observation Disposition, Absence, And Work Disclosure

Implementation status: complete for the Foundational boundary; Phase 3 is
closed after the profile-only planning and public-front-door repairs. Disposition, typed absence, and optional observation work
disclosure are implemented; runtime session gating remains Phase 6.

What becomes true:

- boundary artifacts can describe inactive, continuous, and explicitly
  activated observation
- inactive observation is a distinct typed absence cause
- M8 claims can include and exclude each optional observation work class
- hot-path claims cannot classify correctness-required identity or durability
  work as optional observation

Required evidence:

- a disposition matrix covering inactive, continuous, and explicitly
  activated observation, with selected-surface materialization preserving the
  exact descriptive disposition
- blind-consumer absence twins proving `ObservationNotActivated` is distinct
  from richness, budget, retention, reconstructability, support-posture, and
  certification denials
- independent active/inactive claim twins for all five optional observation
  work classes, plus a nonoptional authoritative-work control
- included/excluded work disclosure legality, overlap denial, and hot-path
  correctness/identity work separation
- canonical performance bundle/comparison/report evidence that retains the
  admitted profile identity and actual disposition

The next phase may trust Foundational boundary vocabulary for runtime adoption.

### Phase 4: Foundational Facade, Readiness, And Documentation Closure

Implementation status: implemented and now the current certification gate.
Phase 4 certifies public re-export parity, exact M10 readiness evidence,
baseline/documentation links, and the scoped facade exception. It does not
implement Signal policy adoption or runtime observation sessions; those remain
Phase 5 and later.

What becomes true:

- the stable profile and performance facades export the new vocabulary
- internal topology is not exposed as a second public API
- Milestone 10 Foundational readiness evidence names every certified surface,
  runtime assumption, non-assumption, hostile pressure, and Store handoff
- profile and performance developer docs teach the orthogonal model

Phase 4 boundary law:

- lower-lane modules are inspectable projections of the same production
  vocabulary, not alternate authorities
- the root/common facade remains the ergonomic entry point and the stronger
  readiness lane remains proof-bearing
- the M10 readiness report is descriptive handoff evidence; it cannot satisfy
  the proof-bearing production-readiness artifact
- every documented Phase 4 example and lower-lane symbol is compile-checked
  through its advertised public path

The next phase may import the shared vocabulary through public facades only.

### Phase 5: Signal Runtime Policy Compiler And Cutover

Implementation status: Phase 5 is closed by the phase-local evidence packet
and fresh independent review recorded in the ledger. The policy compiler owns
the admitted → resolved → installed progression; diagnostics retains only a
request mirror, while runtime decisions consume the installed projections.
Runtime observation sessions and performed receipts remain later phases.

What becomes true:

- Signal runtime policy no longer belongs to diagnostics
- public requests progress through admitted, resolved, and installed types
- execution, maintenance, parallel, observation, retention, and reconstruction
  decisions are resolved before execution
- `DiagnosticsTier` is retained only as a descriptive preset input; planner
  and maintenance owners consume the installed resolved policy rather than
  reading the tier directly
- current domain presets lower through the same compiler

Mechanical enforcement:

- executors accept installed/resolved policy, not caller requests
- adding a resolved field breaks builders, forks, snapshots, restores, and
  reconfiguration until propagated
- the old diagnostics-owned policy path is removed rather than aliased

The next phase may trust one installed policy authority.

### Phase 6: Signal On-Demand Observation And Performed Receipts

What becomes true:

- Throughput idle performs zero optional observation work
- the gate precedes construction, allocation, atomic update, locking, cloning,
  indexing, and serialization at every named owner
- explicit observation sessions activate only selected surfaces
- performed receipts require a pre-execution admitted session and completed
  execution

Required evidence:

- exact zero/nonzero counter twins for each optional work class
- session cancellation, stale completion, duplicate finish, and concurrent or
  nested admission tests
- mutation probes that move or remove representative gates

The next phase may trust pay-on-demand observation inside one live runtime.

### Phase 7: Signal Branch, Snapshot, Restore, Replay, And Reconfiguration

What becomes true:

- stable identity and correctness-required replay linkage are structurally
  separate from optional descriptive lineage
- checkpoint/restore preserves operational truth across profile changes
- evidence never captured remains explicitly unavailable
- reconfiguration is atomic with respect to execution and observation
  sessions
- WASM-supported Signal configurations preserve the same semantics

Required evidence:

- the restore transition matrix from the decisive courtroom
- branch and replay operational parity
- snapshot size and diagnostic payload comparisons by profile
- WASM build/check lane for the supported facade

The next phase may trust lifecycle parity.

### Phase 8: Signal Certification, Performance Proof, And Deletion

Implementation status: closed. The ordinary six-profile courtroom runs on an
honestly sized 1,024-output world with 120 one-percent batches and a hard
10-minute per-test budget. The measured idle-versus-balanced packet, named
scale slopes, and scheduled node-bound lane execute in the ordinary
certified set. The recorded scheduled bound is 4,096 outputs because 65,536
cannot finish inside the 10-minute budget. Ordinary slopes measure nodes,
edit-width, and fanout; depth is not an independent ordinary-lane axis on
this world. Independent qa-loop, qa-tests, and code-quality reviewers
accepted this slice. The constructor remainder elevates
`operational()` and deletes the duplicate `throughput()` name; independent
qa-loop, qa-tests, and code-quality reviewers accepted that cutover. The
idle-versus-introspective packet remains observation-cost evidence.

What becomes true:

- the full adversarial courtroom is green through the production composition
  root
- throughput benefit and structural zero-work claims are recorded honestly
- serial and parallel operational digests match across profile variants
- old tier-coupled strategy logic and compatibility paths are deleted
- the constitutional, line-cap, formatting, default, parallel, doctest, and
  documentation gates pass

Milestone 10's Signal adoption claim does not close on vocabulary alone.
Phase 8 closed the courtroom. The duplicate `throughput()` constructor was
removed; `operational()` is the public Throughput + OnDemand constructor.
The idle-versus-introspective packet (`idle_milli=19755` versus
`pressure_milli=5864` on the 1,024-output world) remains evidence that
OnDemand Operational beats Forensic Continuous. Clock-and-report assembly
that still runs when capture is off is ordinary later hot-path work.

### Remainder: public production constructor

Closed. There is no `SignalRuntimePolicy::throughput()` constructor.
`operational()` is the public Throughput + OnDemand constructor. Independent
qa-loop, qa-tests, and code-quality reviewers accepted the cutover. The
idle-versus-introspective packet stays in the ordinary certified set as
observation-cost evidence, not as justification for a synonym. Phase 9 Store
adoption does not reopen a second constructor.

### Phase 9: Eight-Axis Consumer Cutover

Implementation status: closed. Independent qa-loop, qa-tests, and
code-quality reviewers accepted the consumer cutover.

A document that tells Store to adopt later is not enforcement. Store, Query,
Relational, and bank-world already construct `FoundationalProfileSet`. After
Phases 1–8 that type has eight required families. Six-axis literals are a
present compiler break and a silent authority hole, not a successor memo.

What becomes true:

- every remaining `FoundationalProfileSetInput` / `FoundationalProfileSet::new`
  site outside Foundational names `execution_objective` and
  `observation_activation` explicitly; there is no six-axis compatibility
  constructor, default, or alias that fills those families in
- Store operational and durable evidence constructions use
  `ExecutionObjectiveProfile::Throughput` and
  `ObservationActivationProfile::OnDemand` unless the site is honestly
  forensic or continuous
- Query, Relational, and bank-world constructions name both axes for their
  real richness/support/certification posture; they do not copy Signal's
  production bundle as a second Store or Query runtime policy
- durability, WAL, acknowledgement, MVCC, recovery, integrity, custody, and
  CDC cursor authority remain Store-owned and are not routed through
  observation activation
- Store vision and runtime-integration docs state the mapping and the
  forbidden weakening; they do not claim a Store crash/WAL courtroom closed
- no `throughput()` constructor is reintroduced on Signal or invented on Store

Required evidence:

- source search showing no remaining six-field `FoundationalProfileSetInput`
  literals in Store, Query, Relational, or bank-world
- the named Store/Query/Relational/bank-world crates compile against the
  eight-axis type
- at least one Store operational construction asserts Throughput + OnDemand
  without changing retention, WAL, or recovery posture
- Foundational already denies omitted families at the front door; Phase 9
  does not add a second profile compiler

This phase is the consumer cutover of the shared vocabulary. It is not a
Store durability courtroom, Signal leftover clock-gating, renaming
`ExecutionObjectiveProfile::Throughput`, or parallelization.

## Documentation Deliverables

### Foundational developers

Revise:

- `crates/worth-foundational/docs/profiles-and-policy-vocabulary/README.md`
- `profile-families-and-composed-profile-sets.md`
- `requested-admitted-and-materialized-profile-progression.md`
- `descriptive-surface-materialization-and-elision.md`

Create:

- `throughput-and-observation-activation.md`

These documents must show the six-question model: objective, activation,
richness, retention, support audience, and certification strength.

### Performance claim authors

Revise:

- `crates/worth-foundational/docs/performance/README.md`
- `common-performance-claims-and-layout-intent.md`
- `counter-backed-performance-receipts.md`

Create:

- `observation-work-disclosure.md`

The docs must show that a throughput claim names included and excluded work and
does not derive authority from timing.

### Signal callers and maintainers

Revise:

- `crates/worth-signal/README.md`
- `plans/WORTH_signal/signal_performance.md`
- `plans/WORTH_signal/signal_performance_architecture.md`
- `plans/WORTH_signal/test-requirements.md`
- `plans/WORTH_signal/WORTH_signal_vision.md`

They must explain policy construction, observation-session lifecycle, typed
unavailability, profile reconfiguration, branch/restore behavior, and the
difference between stable lineage identity and optional descriptive lineage.

Public examples must compile against the real facade.

### Store successor maintainers

Revise as part of the Phase 9 cutover, not instead of it:

- `plans/worth-store/worth_store_vision.md`
- `plans/worth-store/runtime-integration-roadmap.md`

The docs must state the eight-axis mapping and that throughput never weakens
durability, acknowledgement, recovery, MVCC, integrity, custody, or
authoritative lineage. They may not claim the Store durability courtroom
closed.

## Must Ship

- `ExecutionObjectiveProfile`
- `ObservationActivationProfile`
- descriptive observation scope and disposition vocabulary
- observation-specific absence law
- eight-axis total Foundational profile composition
- canonical profile identity and difference support for the new axes
- canonical multi-family profile resolution records
- optional observation work classes in M8 performance claims
- updated Foundational public facades, docs, readiness, and tests
- compiler-visible Signal runtime policy progression
- one installed Signal policy authority
- `SignalRuntimePolicy::operational()` as the public Throughput + OnDemand
  constructor, together with the two profile axes; the idle-versus-introspective
  packet justifies calling that posture a throughput lane in docs, not a
  second constructor name
- managed Signal observation sessions
- pre-construction/pre-update gates for every named optional work owner
- performed receipts that require admitted observation plus performed execution
- branch/snapshot/restore/replay/reconfiguration parity
- real Signal production-root certification and performance evidence
- eight-axis cutover at every remaining `FoundationalProfileSet` construction
  site; Store operational mapping is Throughput + OnDemand; durability stays
  Store-owned and unweakened

## Must Preserve

- identical authoritative and operational Signal truth across profiles
- stable Signal lineage identity in every supported profile
- deterministic ordering, replay linkage, branch meaning, snapshot authority,
  and restore outcomes
- exact aspect, partition, detail, producer, revision, and cause binding
- comparator and reuse correctness
- profile identity and boundary interpretation across crates
- Foundational representation freedom
- `worth-proof` ownership of proof progression rather than descriptive policy
- Signal ownership of runtime execution and observation sessions
- Store ownership of durability and persistence policy
- explicit cost and absence disclosure
- WASM compatibility for supported Signal surfaces

### Scoped composition exception

The pre-existing `crates/worth-foundational/src/facade.rs` is a public
re-export-only compatibility facade and remains above the repository's default
400-line Rust cap in this milestone.  M10 adds only the new vocabulary exports;
it does not add implementation responsibility to that file.  This is an
explicit M10 exemption for the existing facade aggregator.  A later facade
topology milestone may split its responsibility-shaped re-export groups, but
M10 does not claim that refactor as part of phases 1–5.

The hand-maintained WASM declaration barrel
`crates/worth-signal-wasm/package/types/diagnostics.d.ts` is also explicitly
exempt from the default 400-line source-file cap for M10.  It is a public
declaration surface whose policy vocabulary additions remain adjacent to the
existing diagnostic unions and snapshot metadata; M10 adds no runtime
implementation responsibility to this file.  A later WASM facade-topology
milestone may split declarations by responsibility, but Phase 8 governs this
barrel through its package typecheck and public-shape checks.

## Acceptance Evidence

### Foundational local proof

- profile family distinction and composition tests
- canonical identity sensitivity for each new axis
- family-specific difference classification
- canonical multi-adjustment progression tests
- observation disposition and absence tests
- work-class included/excluded legality tests
- proof-lane and facade visibility tests only where invalid public progression is
  genuinely a compile-time product contract
- Milestone 10 readiness certification

### Signal integration proof

- public policy lowering and installed-policy tests
- compiler-visible phase progression evidence
- tier/objective/richness orthogonality matrix
- no-observer exact zero-work tests
- explicit-session exact performed-evidence tests
- stale, duplicate, cancelled, and concurrent session lifecycle tests
- operational digest parity across profiles
- branch/snapshot/restore/replay transition matrix
- serial/parallel and default/feature parity
- WASM check

### Performance proof

- recorded current baseline before implementation
- exact structural counters by optional work class
- allocation and retained-state observations
- warm median, p95, and completed-work throughput
- scale slopes across nodes, edit width, fanout, depth, and parallel regions
- no benchmark-scope transfer beyond the measured worlds

### Mutation probes

- representative counter gate deletion
- lineage gate relocation after construction
- provenance gate bypass
- tier-to-strategy recoupling
- stable identity omission
- post-hoc receipt minting
- restore evidence fabrication
- included/excluded work misclassification

## Explicit Non-Goals

- a universal Foundational runtime policy executor
- a universal telemetry, lineage, or provenance store
- a universal benchmark harness
- a new correctness or durability tier
- removal of stable lineage identity
- permission to disable deterministic replay semantics
- Store implementation inside Foundational
- a compile-time `no-observability` feature in the first implementation
- changing Signal's default preset silently
- renaming Operational to Throughput without measured and structural proof
- promising that Throughput wins on every workload
- embedding geometry, finance, chip, web, or database semantics in the shared
  vocabulary

## Successor Handoff

### Foundational Milestone 11

Cross-crate migration and closure consumes the eight-axis profile, actual
observation disposition, absence law, and optional-work disclosures. It must
retire central crate-local dialects rather than wrap them.

### Store durability courtroom milestone

Phase 9 cuts Store onto the eight-axis vocabulary. A later Store milestone
still owns durable and embedded composition-root courts. It must prove:

- identical canonical commits and recovered truth across observation profiles
- no weakening of WAL or acknowledgement order
- zero optional diagnostic persistence outside an admitted observation/audit
  scope
- bounded physical amplification with bytes, barriers, syscalls, and
  compaction work disclosed
- crash recovery at every ambiguous partial-effect boundary

### Later compile-time specialization

Only after runtime gating is measured may a separate milestone consider a
WASM/embedded build profile that physically removes unused observability code.
That work must prove identical public truth semantics and may not use feature
flags to change authority or supported outcomes silently.

## Self-Check

- Does the design make Throughput a real objective rather than a marketing
  label? Yes. It has structural zero-work and measured benefit bars.
- Can Throughput weaken correctness or Store durability? No. Those guarantees
  are explicitly outside profile authority.
- Does OnDemand merely skip rendering? No. Gates precede optional construction,
  allocation, updates, locks, clones, indexes, and serialization.
- Can evidence be minted after the operation? No. Signal observation authority
  must be admitted before effects and consumed at completion.
- Can diagnostic richness still select execution strategy? No. The Signal
  compiler resolves orthogonal axes and executors consume installed policy.
- Can missing evidence look complete? No. Observation-specific absence and
  actual disposition are typed boundary meaning.
- Does Foundational become a runtime? No. It owns vocabulary, comparison,
  identity, and attachment; runtimes own execution and resources.
- Is Signal adoption optional? No. The milestone closes only after the real
  production-root court and performance proof.
- Is Store falsely claimed complete? No. Phase 9 cuts Store onto the
  eight-axis vocabulary. Crash, WAL, and durability courts remain Store's
  successor milestone.
