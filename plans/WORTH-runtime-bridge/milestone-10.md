# Milestone 10 Engineering Spec: Speculative Truth-Branch To Signal-Branch Coordination And Preview Flows

> **Status:** Completed. Closeout: [milestone-10-closeout.md](./milestone-10-closeout.md).
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-9.md](./milestone-9.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make speculative truth branches, speculative signal branches, preview sessions, discard boundaries, and explicit promotion-to-authority records first-class bridge protocol surfaces so preview and branch-local evaluation remain deterministic, replay-safe, and fail-closed without collapsing the two runtimes into one speculative model
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

Milestones 1 through 9 established that the bridge already has strong protocol
surfaces for:

- canonical patch ingestion and deterministic invalidation routing
- lineage-aware continuity, historical truth-view selection, and branch-aware
  evaluation
- planned bulk routing, packetized preparation, and replay-safe execution
- source protocol productization and structural-identity-aware advisory remap
- merge-aware ordered-history consumption with explicit denial and explanation

That is enough to make the bridge honest about authoritative truth, branch-local
truth, structural ambiguity, and merge-bearing history.

It is not enough to make the bridge honest about speculation and preview.

Without Milestone 10, the bridge still has a dangerous blind spot:

- it can read branch-local truth and explain merge-bearing history, but it does
  not yet own a canonical correspondence contract between speculative truth
  branches and speculative signal branches
- it can route and replay authoritative or branch-local work, but preview flows
  still risk becoming ad hoc host conventions with unclear discard boundaries
- it can explain merge-aware authority outcomes, but it cannot yet explain when
  speculative work stayed non-authoritative, when it was discarded, and when it
  crossed an explicit promotion boundary into canonical authority
- it can certify authoritative replay, but preview churn still risks leaving
  checkpoint, routing, diagnostics, or resource residue that later authoritative
  flows could misread

Milestone 10 exists because Milestone 9 deliberately stopped at merge-aware
truth consumption. That milestone taught the bridge how to say:

`this exact merge-bearing truth artifact produced this exact bridge result`

Milestone 10 must now teach the bridge how to say:

`this exact speculative truth branch, coordinated with this exact speculative signal branch under this exact preview contract, produced this exact non-authoritative bridge result, was either discarded with zero authoritative residue or explicitly promoted through this exact commit boundary, and can be replayed and explained without confusing preview state for authoritative state`

not:

`the host did some preview work and later decided whether it seemed committed enough`

## Goal

Make speculative truth-to-signal coordination and preview evaluation a
deterministic, replay-safe, bridge-owned protocol so hosts can explore
branch-local and non-authoritative flows without allowing speculative derived
state, temporary bridge artifacts, or preview diagnostics to drift into
authoritative bridge meaning accidentally.

## Why This Milestone Exists

Milestone 10 belongs immediately after Milestone 9 because speculative branch
coordination becomes unsafe if the bridge still lacks canonical, replay-safe
merge-aware truth consumption. Preview branches must inherit the same
branch-local and merge-aware authority basis as authoritative flows before the
bridge can reason honestly about discard and commit boundaries.

Milestone 9 established:

- canonical ordered-history and merge-aware authority consumption
- replay-safe merge explanation and typed denial
- branch- and merge-aware continuity and advisory remap inputs
- deterministic canonical result bundles over hostile history variation

Milestone 10 now needs to establish the matching speculative truths:

- canonical branch correspondence between truth-side and signal-side
  speculative identities
- explicit preview-session declarations and admission
- explicit discard and promotion boundaries
- explicit lifecycle ownership for temporary bridge artifacts and resources

If Milestone 10 shipped before Milestone 9, speculative flows would inherit an
incomplete notion of branch-local authority and could silently treat merge-aware
history as a host-local special case.

Milestone 10 also belongs before Milestone 11 because policy propagation is only
safe once the bridge can distinguish authoritative requests from speculative and
preview requests structurally. Policy provenance should refine an already-clean
request kind boundary; it should not be used to invent one retroactively.

Milestone 10 therefore earns its place in the roadmap by solving the next real
structural problem after merge-aware history: keeping speculative work useful,
bounded, and explainable without turning the bridge into a second authority or
into a hidden cache of preview leftovers.

## Adversarial Constraint

Milestone 10 must survive the following hostile condition:

> A long-lived system with many concurrent speculative truth branches,
> speculative signal branches, preview sessions, discard-heavy interactive
> churn, selective promotion to authoritative publication, merge-bearing branch
> ancestry, diagnostics tiers that vary by environment, restarts between preview
> and promotion, and host adapters with very different temporary-resource models
> must produce the same branch-correspondence judgment, the same preview
> execution result, the same discard-versus-promotion classification, the same
> replay result, and the same authoritative residue outcome every time, while
> never allowing speculative state to become authoritative except through one
> explicit canonical promotion boundary.

Concretely, the design must remain correct when all of the following are true:

- several preview sessions share similar truth ancestry but must remain isolated
- speculative truth branches and speculative signal branches are created in
  different orders on different hosts
- one preview session is discarded after heavy routing, diagnostics, and remap
  activity
- another preview session is promoted while an equivalent authoritative flow
  runs in parallel
- restarts occur between preview execution, discard, and promotion
- diagnostics richness changes between environments
- merge-bearing ancestry influences preview routing basis
- temporary bridge resources are implemented differently by different host
  adapters, but canonical bridge meaning must not vary

If any supported path:

- treats preview and authoritative requests as the same request class with only
  a loose boolean flag
- leaves checkpoint, routing, replay, diagnostics, or writeback-shaped residue
  after discard
- allows speculative branch identity to be inferred from ambient latest branch
  state during replay
- lets preview diagnostics retention fabricate authoritative-looking artifacts
- promotes speculative output without an explicit canonical promotion record
- allows one preview session to observe another session's temporary bridge state
- depends on host-local cleanup folklore rather than bridge-owned lifecycle
  contracts

then Milestone 10 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- speculative truth branches and speculative signal branches remain distinct
  authorities and must remain distinct types
- preview session identity, branch correspondence, execution result, lifecycle
  transition, discard record, and promotion record are distinct concepts and
  must remain distinct types
- preview lifecycle must be modeled as a closed typestate progression rather
  than an open status string or loosely coordinated booleans
- preview is a first-class bridge request mode, not a disguised authoritative
  request with post hoc cleanup
- discard and promotion are lifecycle transitions over an admitted preview
  session, not peer request modes interchangeable with fresh execution
- discard is authoritative about non-publication and residue cleanup; it is not
  "best effort"
- promotion to authoritative meaning happens only through an explicit canonical
  bridge promotion surface that records the preview-origin basis
- promotion is admitted only when the preview-origin basis still matches a
  closed canonical admissibility proof; "close enough" promotion is out of spec
- preview artifacts may inform explanation and replay of preview behavior, but
  they may not become canonical authoritative truth artifacts on their own
- truth runtime remains the authority for speculative truth branches and truth
  publication
- signal runtime remains the authority for speculative execution branches and
  derived execution behavior
- the bridge owns only branch correspondence, preview lifecycle, non-
  authoritative artifact shaping, and promotion-boundary coordination
- diagnostics richness may change retained detail, but not branch
  correspondence, discard-vs-promotion meaning, or replay result
- preview reuse is admitted only under an explicit bridge equivalence contract;
  branch identity, truth-view basis, merge/history basis, request shape, and
  retained artifact version mismatches fail closed by default
- Milestone 10 productizes speculation and preview coordination only; it does
  not yet productize cross-runtime policy propagation or bridge-mediated
  writeback authority

Normative consequence:

- APIs that expose a generic "session" without typed preview-versus-
  authoritative request kind are out of spec
- host adapters that promote preview outcomes without bridge-owned promotion
  records are out of spec
- host adapters that retry promotion against stale preview-origin basis without
  bridge-owned re-admission are out of spec
- discard that leaves any authoritative-looking bridge residue is out of spec
- replay that reconstructs speculative branch meaning from ambient current state
  rather than canonical preview artifacts is out of spec
- diagnostics-only distinctions between preview and authoritative flows are out
  of spec
- "commit later if unchanged" shortcuts that skip explicit promotion recording
  are out of spec

## Configuration And Defaults

Milestone 10 should expose only a small set of explicit speculation and preview
configuration surfaces. Authority boundaries and discard semantics are not
configurable.

### Admitted Configurable Surfaces

- speculative request mode
  - default: `Preview`
- preview reuse policy
  - default: `NoReuseWithoutExactEquivalenceProof`
- preview retention richness
  - default: retain enough canonical preview evidence to replay preview result,
    discard, and promotion boundaries without ambient state lookup
- preview diagnostics richness
  - default: structured standard diagnostics, not maximum forensic retention
- temporary resource retention policy
  - default: request-scoped and discard-scoped; no ambient cross-request reuse
- promotion publication policy
  - default: explicit promotion only, with canonical preview-origin record
- preview branch reuse mode
  - default: exact correspondence only; mismatch fails closed

### Non-Configurable Surfaces

- implicit preview-to-authority promotion
  - default: never admitted
- authoritative reuse of discarded preview artifacts
  - default: never admitted
- cross-session temporary-resource sharing without a declared equivalence
  - default: never admitted
- promotion on stale preview-origin proof
  - default: never admitted
- branch identity reconstruction from ambient latest state during replay
  - default: never admitted
- diagnostics retention changing preview-vs-authoritative meaning
  - default: never admitted

The bridge should therefore feel configurable at preview richness and retention
boundaries, but closed and fail-safe at promotion and discard authority
boundaries.

## Guideline Influence

### 1. `MENTALITY.md`

This document directly shapes the milestone:

- adversarial constraint first:
  the spec starts from preview churn, residue hostility, restarts, and
  promotion confusion rather than from the pleasant feature phrase "support
  preview mode"
- solve the hard problem first:
  request-kind separation, discard cleanup, and promotion proof surfaces ship
  before ergonomic helper APIs
- enforce mechanically, not by convention:
  preview-versus-authoritative meaning, branch correspondence, and discard
  outcomes must be represented by proof-bearing types and typed failures
- spec is architecture is code:
  the spec names request classes, lifecycle records, counters, and subdomains
  that implementation must map directly
- separate what/how/whether:
  speculative truth and execution branches are the `what`, bridge coordination
  and lifecycle are the `how`, and preview diagnostics richness is the `whether`
- authority first, derivation second:
  authoritative truth publication stays upstream while preview artifacts remain
  derived and destroyable

### 2. `architectural_guidelines.md`

This document determines the structural boundaries:

- speculation, preview lifecycle, diagnostics, replay, and promotion cannot be
  one flattened "preview.rs" responsibility
- the bridge facade must expose a small speculation surface while internal
  correspondence, lifecycle, and artifact publication remain private
- discard cleanup is not the same responsibility as promotion publication
- replay reconstruction is not the same responsibility as diagnostics shaping

### 3. `domain_standards.md`

This document constrains domain naming and ownership:

- names must reflect bridge-domain concepts such as preview session,
  speculative branch binding, discard record, and promotion record rather than
  generic managers or handlers
- branch correspondence and promotion provenance must be expressed as domain
  nouns and proof-bearing artifacts
- preview lifecycle rules must remain aligned with domain boundaries rather than
  hidden inside helper utilities

### 4. `performance_guidelines.md`

This document constrains the execution model:

- preview churn must remain bounded by semantic delta and request scope, not by
  total historical preview volume
- temporary resource retention must follow explicit lifecycle scope rather than
  ambient caches
- branch correspondence, preview planning, and promotion lowering must occur
  before execution rather than being rediscovered inside the hot path
- preview APIs must be honest about resource retention, replay retention, and
  discard cost

## Scope

In scope for Milestone 10:

- bridge-owned speculative request classification
- explicit branch correspondence contracts between truth-side and signal-side
  speculative identities
- preview execution flows and non-authoritative bridge artifact publication
- discard records and discard cleanup verification
- explicit preview-to-authority promotion boundaries
- replay and diagnostics for preview, discard, and promotion behavior
- resource lifecycle accounting for temporary bridge artifacts created by
  preview flows

Out of scope for Milestone 10:

- cross-runtime policy vocabulary and propagation
- bridge-mediated writeback semantics
- inventing new speculative semantics inside either parent runtime
- making preview artifacts authoritative without explicit promotion
- broad host-specific UI workflows around preview consumers

## Governing Design Rules

- every bridge request must declare whether it is authoritative or preview
  before admission, and lifecycle transitions must declare the preview session
  they target
- preview sessions must advance through a closed typestate graph:
  `PreviewDeclared -> PreviewAdmitted -> PreviewActive -> PreviewDiscarded | PreviewPromoted`
- discard and promotion must consume only `PreviewActive` sessions
- `PreviewDiscarded` and `PreviewPromoted` are terminal states and must not
  admit further execution, discard, or promotion transitions
- preview branch correspondence must consume explicit truth-branch and
  signal-branch identities; it may not infer them from ambient current state
- discard must emit a canonical outcome class even when the physical cleanup
  work is minimal
- promotion must consume a canonical preview-origin record and produce a
  canonical promotion record
- promotion must re-check a closed admissibility basis rather than relying on
  session identity alone
- preview diagnostics are derived artifacts and must remain subordinate to
  canonical preview and promotion records
- temporary bridge resources created for preview flows must belong to an
  explicit lifecycle scope and be measurable through counters
- any preview reuse must satisfy an explicit equivalence contract recorded in
  canonical artifacts
- preview replay must consume canonical preview records rather than recreating
  host-local branch selection logic
- merge-aware and branch-aware authority basis from Milestone 9 remain
  load-bearing inputs to speculative coordination when histories require them

## Promotion Admissibility Basis

Milestone 10 must not treat promotion as "the preview looked good, so publish
it."

Promotion admission must consume and verify a closed proof bundle that includes
at minimum:

- preview-session identity
- preview execution record identity
- truth-branch identity
- signal-branch identity
- truth-view basis digest
- merge/history basis digest when applicable
- structural/remap basis digest when applicable
- admitted source-capability digest
- request-shape digest
- retained artifact schema/version digest

Promotion must fail explicitly if any required basis entry:

- is missing
- changed since preview execution
- belongs to a discarded session
- belongs to a different branch binding
- was retained under an incompatible artifact version or request shape

Milestone 10 does not yet need the full cross-runtime policy provenance of
Milestone 11, but it must already leave room for that work by keeping promotion
admissibility closed and typed rather than ambient.

## Lifecycle Typestate Requirement

Milestone 10 should make illegal preview lifecycle transitions mechanically hard
or impossible.

The preferred architectural shape is a closed typestate family along the lines
of:

- `BridgePreviewSession<Declared>`
- `BridgePreviewSession<Admitted>`
- `BridgePreviewSession<Active>`
- `BridgePreviewSession<Discarded>`
- `BridgePreviewSession<Promoted>`

The exact Rust spelling may differ, but the mechanical guarantees are required:

- only `Declared` sessions may enter admission
- only `Admitted` sessions may enter preview execution
- only `Active` sessions may be discarded or promoted
- `Discarded` sessions cannot be promoted, executed, or reused
- `Promoted` sessions cannot be discarded, executed, or re-promoted
- lifecycle transitions must consume the prior state and produce the next state
  rather than mutating an ambient shared status field

Equivalent enforcement mechanisms are acceptable if they are comparably strong,
for example:

- sealed state markers
- private constructors plus state-specific transition methods
- enum wrappers that preserve move-only transition ownership and forbid illegal
  re-entry

The following are out of spec unless wrapped by stronger enforcement:

- public mutable status fields
- string or integer lifecycle codes
- transition APIs that accept any session regardless of state and fail only by
  convention
- helper functions that take a raw session ID and "figure out" whether discard
  or promotion is legal at runtime

This requirement exists because preview lifecycle mistakes are authority
mistakes. The wrong transition should be difficult to express, not merely
detectable after the fact.

## Preview Reuse Equivalence Contract

Preview reuse must be explicit and fail closed.

The bridge may reuse temporary preview resources only when an exact equivalence
contract proves all of the following remain equal:

- truth-branch identity and truth-view basis digest
- signal-branch identity
- merge/history basis digest when applicable
- structural/remap basis digest when applicable
- request-shape digest
- source-capability digest
- retained artifact schema/version digest
- lifecycle state remains preview-active rather than discarded or promoted

If any one of those differs, reuse is illegal rather than degraded.

This rule exists to prevent "temporary" preview caches from quietly turning into
ambient cross-request state.

## Residue Taxonomy

Milestone 10 must define residue structurally rather than narratively.

Every bridge-owned artifact created by preview flows must be classified as one
of:

- `AuthoritativeCanonical`
- `PreviewDerivedRetained`
- `DiscardMustDestroy`
- `ReplayRetainedNonAuthoritative`

At minimum, the taxonomy must classify:

- preview execution records
- branch-binding artifacts
- packet plans and routing records
- structural/remap artifacts
- merge explanation artifacts
- diagnostics artifacts
- replay artifacts
- temporary resource handles and indexes
- source-read caches if any exist
- checkpoint-shaped or stream-shaped bridge artifacts if any exist

Discard completeness means:

- every `DiscardMustDestroy` artifact is destroyed or rendered unreachable
- every `ReplayRetainedNonAuthoritative` artifact remains mechanically marked as
  non-authoritative
- no `AuthoritativeCanonical` artifact is fabricated by discard or preview
  execution

The residue taxonomy must be part of the type and test surface, not just a
closeout checklist.

## Complexity Contracts

Milestone 10 must name the expected complexity of each hot-path speculation
operation and back those claims with counters and proof tests.

At minimum, the spec should treat the following as named contracts:

- preview admission
  - expected bound: proportional to declared request shape and branch-binding
    proof width, not total preview history
- preview publication
  - expected bound: proportional to the preview's admitted execution and
    artifact width, not total retained preview sessions
- discard cleanup
  - expected bound: proportional to artifacts owned by the targeted preview
    session, not global preview churn or branch count
- promotion admission
  - expected bound: proportional to the closed admissibility proof width, not
    broad branch or history rediscovery
- replay
  - expected bound: proportional to retained canonical preview bundle width, not
    ambient current-state reconstruction

Milestone 10 must add counters sufficient to prove those claims, including at
minimum:

- preview-session count touched
- branch-binding proof width
- admissibility proof width
- preview artifact count
- discard artifact count
- destroyed artifact count
- retained non-authoritative artifact count
- promotion proof checks
- replay bundle width

Any implementation that requires broad scans over all retained preview sessions,
all branches, or all diagnostics artifacts to perform discard, promotion, or
replay is out of spec unless explicitly marked as debt with named proof gaps.

## Phases

### Phase 1: Preview Lifecycle And Authority Lock

Milestone 10 must first define:

- the closed request-kind vocabulary for authoritative and preview execution
  flows
- the closed lifecycle-transition vocabulary for preview discard and preview
  promotion
- the closed typestate graph for preview lifecycle progression
- bridge-owned proof-bearing branch correspondence types between speculative
  truth branches and speculative signal branches
- canonical preview-session identity and lifecycle-state types
- typed failure classes for branch mismatch, invalid reuse, discard failure, and
  promotion illegality
- explicit authority boundaries for what the bridge may retain, publish, or
  destroy during preview flows
- the closed promotion admissibility proof bundle
- the residue taxonomy for all preview-created bridge artifacts

This is the hard-foundation workload bucket for the milestone. It should end
with the bridge able to represent preview sessions, branch bindings, lifecycle
states, admissibility proofs, residue classes, and typed failures mechanically,
with illegal lifecycle transitions made unrepresentable or uncompilable.

Phase 1 is complete only when:

- preview lifecycle and authority boundaries are mechanically locked
- promotion admissibility has a closed proof surface
- discard, promotion, and reuse legality can be decided from typed bridge
  artifacts rather than host folklore
- the facade can expose speculation entrypoints without relying on raw status
  fields, raw IDs, or ambient state

Phase 1 must not ship:

- preview execution publication
- discard execution and cleanup
- promotion publication
- replay or diagnostics flows that depend on speculation execution artifacts

### Phase 2: Preview Execution And Discard Cleanup

Milestone 10 must then implement:

- admission of preview declarations and speculative branch bindings
- lowering of preview requests into replay-safe preview execution records
- discard publication that proves zero authoritative residue and explicit
  temporary-resource cleanup outcome
- fail-closed reuse admission through explicit equivalence proofs only
- bounded counters for preview planning width, resource retention, and cleanup
  breadth

This is the non-authoritative execution workload bucket. It should end with
preview sessions running end-to-end, admitted preview artifacts being published
as non-authoritative bridge records, and discard behaving as a real lifecycle
transition with zero-residue cleanup for discarded work.

Phase 2 is complete only when:

- `PreviewAdmitted -> PreviewActive` execution exists as a real bridge path
- `PreviewActive -> PreviewDiscarded` exists as a real bridge path
- every discard path proves residue classification and cleanup outcome
- non-authoritative preview publication remains bounded by the named complexity
  contracts

Phase 2 must not ship:

- preview-to-authority promotion
- authoritative publication derived from preview outputs
- replay or explanation that depends on promotion artifacts
- milestone closeout certification

### Phase 3: Promotion, Replay, And Certification

Milestone 10 must finally ship:

- promotion publication that records exactly how a non-authoritative result
  crossed into authoritative meaning
- replay-safe preview, discard, and promotion records
- explanation artifacts that distinguish preview, discard, and promotion
  behavior without changing canonical meaning across diagnostics tiers
- harness certification for residue-free discard, explicit promotion, and
  preview churn isolation
- proof tests for the named complexity contracts
- machine-checkable canonical bundles satisfying suites 13 through 15 in
  [test-requirements.md](./test-requirements.md)

This is the authority-crossing and closure workload bucket. It should end with
explicit preview-to-authority promotion, replay-safe lifecycle records,
diagnostics shaping, and the certification evidence needed to close the
milestone.

Phase 3 is complete only when:

- `PreviewActive -> PreviewPromoted` exists as a typed, proof-checked bridge
  path
- stale, duplicate, and post-discard promotion attempts fail typed
- replay can reconstruct preview, discard, and promotion outcomes from
  canonical artifacts alone
- suites 13 through 15 pass with canonical machine-checkable bundles

Phase 3 may assume:

- Phase 1 lifecycle and authority lock is already in place
- Phase 2 non-authoritative preview and zero-residue discard are already
  implemented and verified

## Must Ship

- explicit bridge request kinds for authoritative, preview, discard, and
  promotion-coordinated flows
- proof-bearing speculative branch correspondence artifacts
- preview-session declaration and admission surfaces
- replay-safe preview execution records
- discard records and residue-cleanup verification artifacts
- explicit preview-to-authority promotion records
- closed promotion admissibility proof surfaces
- explicit preview reuse equivalence contracts
- a typed residue taxonomy for preview-created bridge artifacts
- diagnostics for branch mismatch, invalid reuse, preview misuse, discard
  residue, and promotion illegality
- counters for preview planning, temporary resources, discard cleanup breadth,
  promotion work, admissibility proof width, and replay bundle width
- harness certification satisfying Milestone 10 suites 13 through 15

## Must Preserve

- truth runtime remains the authority for speculative truth branches and truth
  publication
- signal runtime remains the authority for speculative execution branches and
  execution behavior
- speculative derived state never becomes authoritative accidentally
- branch identity remains explicit end-to-end
- discard leaves zero authoritative bridge residue
- diagnostics richness changes detail only, not preview-versus-authoritative
  meaning

## Acceptance Evidence

Milestone 10 is complete only when the bridge harness can prove:

- speculative truth branches and speculative signal branches are coordinated
  deterministically through canonical branch bindings
- discarded preview flows leave zero authoritative stream, checkpoint, routing,
  replay, or writeback residue
- promotion to authority occurs only through explicit canonical promotion
  records
- stale, duplicate, or post-discard promotion attempts fail explicitly and typed
- preview replay reproduces the same preview result, discard classification, or
  promotion provenance after restart
- discard, promotion, and replay satisfy the named complexity contracts through
  counter proof tests
- diagnostics-tier variation changes retained detail only, not preview meaning
- the Milestone 10 certification suites in
  [test-requirements.md](./test-requirements.md)
  pass with canonical machine-checkable bundles

## Architectural Notes

Milestone 10 should extend the bridge crate with subdomains such as:

- `speculation/`
- `speculation/types.rs`
- `speculation/admission.rs`
- `speculation/preview.rs`
- `speculation/discard.rs`
- `speculation/promotion.rs`
- `speculation/replay.rs`
- `diagnostics/speculation.rs`

Responsibilities should separate as follows:

- `speculation/types.rs`
  - proof-bearing request, branch-binding, lifecycle, discard, and promotion
    types
- `speculation/admission.rs`
  - request-kind validation, branch correspondence admission, and illegal reuse
    rejection
- `speculation/preview.rs`
  - lowering and publication of non-authoritative preview execution artifacts
- `speculation/discard.rs`
  - discard records, cleanup verification, and residue classification
- `speculation/promotion.rs`
  - explicit preview-origin consumption and promotion publication
- `speculation/replay.rs`
  - replay reconstruction from canonical preview and promotion records
- `diagnostics/speculation.rs`
  - diagnostics shaping derived from canonical speculation artifacts

The bridge facade should expose bridge-owned types such as:

- `BridgeRequestKind`
- `BridgeSpeculativeBranchBinding`
- `BridgePreviewSessionDeclaration`
- `BridgePreviewSession<Declared>`
- `BridgePreviewSession<Admitted>`
- `BridgePreviewSession<Active>`
- `BridgePreviewSession<Discarded>`
- `BridgePreviewSession<Promoted>`
- `BridgePreviewExecutionRecord`
- `BridgePreviewLifecycleTransition`
- `BridgeDiscardRecord`
- `BridgePromotionRecord`
- `BridgePromotionAdmissibilityProof`
- `BridgePreviewReuseEquivalence`
- `BridgePreviewResidueClass`
- `BridgeSpeculationFailure`
- `BridgeSpeculationCounters`

These names are illustrative, but the separation is mandatory:

- request-kind classification is not the same responsibility as branch
  correspondence admission
- typestate progression is not the same responsibility as artifact publication
- preview execution record publication is not the same responsibility as discard
  cleanup verification
- promotion publication is not the same responsibility as authoritative truth
  mutation
- replay reconstruction is not the same responsibility as diagnostics shaping

## Test And Harness Model

Milestone 10 must follow the same structural testing discipline as earlier
bridge milestones and must satisfy the Milestone 10 certification suites in
[test-requirements.md](./test-requirements.md).

The harness should expose an execution request shape able to vary:

- request kind
- truth-branch identity and ancestry basis
- signal-branch identity
- preview-session identity
- discard versus promotion outcome
- diagnostics richness
- restart boundary placement
- host-adapter implementation shape

The certification bundle for Milestone 10 should include at minimum:

- `speculative_resource_digest`
- `discard_residue_report`
- `speculative_commit_digest`
- `preview_vs_authoritative_matrix`
- `preview_lifecycle_digest`
- `branch_isolation_matrix`
- `diagnostics_digest`
- `counter_snapshot`

The harness must specifically prove:

- preview churn remains request-scoped and bounded
- discard cleanup is explicit and replay-stable
- promotion and discard cannot be confused by adapters or diagnostics tiers
- preview-state isolation survives restarts and equivalent-host variation
- duplicate promotion, stale promotion, and promotion-after-discard fail closed
- discard, promotion, and replay counters satisfy the named complexity
  contracts

## Target API And Module Plan

Milestone 10 should add or extend bridge-owned surfaces along these lines:

- facade speculation entrypoints
  - preview-session declaration
  - admission from declared to admitted
  - execution from admitted to active
  - discard from active to discarded
  - promotion from active to promoted
- speculation types
  - request kind
  - branch binding
  - preview session
  - preview typestate markers
  - discard outcome
  - promotion provenance
  - failure classes
  - counters
- diagnostics
  - preview explanation
  - discard residue explanation
  - promotion provenance explanation

The facade should not expose:

- raw host-local temporary resource handles as the public contract
- adapter-specific cleanup internals
- implicit "commit if accepted" shortcuts that skip promotion recording
- ambient global preview state

## Implementation Phases

Milestone 10 must execute in strict order.

### Phase M10.0 - Request Kind And Branch Correspondence Lock

- add bridge-owned request-kind vocabulary
- add lifecycle-transition vocabulary distinct from request kind
- add preview typestate markers and transition ownership rules
- add branch correspondence proof types
- add admission-time mismatch and reuse failure classes
- add promotion admissibility proof types
- add residue classification types
- make preview-versus-authoritative distinction unambiguous in the facade

### Phase M10.1 - Preview Publication And Discard Cleanup

- lower admitted preview requests into canonical preview execution records
- publish discard records and cleanup verification artifacts
- add fail-closed preview reuse equivalence admission
- add speculation counters for preview and discard work
- reject residue-bearing cleanup paths structurally

### Phase M10.2 - Promotion Provenance And Replay

- publish explicit promotion records from preview-origin inputs
- reject stale, duplicate, and post-discard promotion attempts through typed
  admissibility failures
- replay preview, discard, and promotion bundles from canonical records
- add diagnostics shaping derived from canonical speculation artifacts

### Phase M10.3 - Certification

- wire suites 13 through 15 into canonical bridge harness bundles
- verify diagnostics-tier invariance and host-variation parity
- verify zero authoritative residue under discard-heavy churn
- verify named complexity contracts through counter proof tests

## Anti-Patterns Explicitly Rejected

- preview mode as a loose flag on authoritative requests
- discard and promotion modeled as peer execution modes instead of lifecycle
  transitions over preview sessions
- mutable lifecycle status fields or raw status codes as the primary lifecycle
  enforcement surface
- cleanup that depends on adapter goodwill rather than bridge-owned records
- authoritative publication inferred from absence of discard
- shared ambient temporary caches across preview sessions
- replay that reconstructs branch bindings from current latest state
- promotion admitted from session identity alone without full proof re-check
- diagnostics retention that fabricates authoritative-looking preview artifacts

## Sequencing Notes

Milestone 10 must land before:

- Milestone 11 cross-runtime policy propagation, because policy provenance must
  refine already-clean preview-versus-authoritative semantics rather than
  invent them
- Milestone 12 bridge-mediated writeback, because writeback must not reuse
  speculative artifacts through ambiguous authority boundaries

Milestone 10 builds directly on:

- Milestone 4 historical and branch-aware evaluation
- Milestone 7 reactive source protocol and clean host surfaces
- Milestone 8 structural-identity-aware remapping
- Milestone 9 merge-aware bridge semantics and multi-parent history consumption

Milestone 10 must not attempt to pre-solve:

- the full cross-runtime policy vocabulary of Milestone 11
- bridge-mediated writeback authority and idempotence of Milestone 12
- end-to-end causality bundle unification of Milestone 13

Those become stronger because Milestone 10 exists; they do not need to be
productized here.

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically?
  - Yes. It closes the speculative-authority gap between branch-local truth
    support and later policy/writeback work.
- Is the adversarial constraint precise and load-bearing?
  - Yes. It centers restart-safe preview churn, zero-residue discard, and
    explicit promotion boundaries under hostile host variation.
- Does the milestone preserve crate authority boundaries?
  - Yes. Truth still owns speculative truth and publication; signal still owns
    speculative execution; the bridge owns only correspondence, lifecycle, and
    boundary records.
- Does the milestone define proof obligations, not just implementation tasks?
  - Yes. Discard residue, promotion admissibility, replay parity, branch
    isolation, and complexity contracts are defined as machine-checkable
    obligations.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  - Yes. The spec names subdomains, facade surfaces, counters, failure classes,
    and certification bundles directly.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  - It belongs here. Merge-aware branch authority had to land first; policy and
    writeback must come later.

## Closeout Standard

Milestone 10 is complete only when all of the following are true:

- preview and authoritative requests are mechanically distinct at the facade
  and admission boundaries
- preview lifecycle progression is mechanically enforced through a closed
  typestate-shaped transition surface or equivalent-strength enforcement
- canonical branch correspondence, preview execution, discard, and promotion
  records exist and replay safely
- discard proves zero authoritative residue
- promotion is admitted only through a closed proof bundle and fails typed when
  stale, duplicate, or post-discard
- preview churn remains bounded and request-scoped under hostile harness
  scenarios
- discard, promotion, and replay satisfy named complexity contracts with proof
  counters
- diagnostics tiers change retained detail only, not speculation meaning
- certification suites 13 through 15 pass with canonical machine-checkable
  outputs

If preview can still become authoritative accidentally, if discard leaves
authoritative-looking residue, or if promotion meaning lives only in
diagnostics, Milestone 10 is not closed.
