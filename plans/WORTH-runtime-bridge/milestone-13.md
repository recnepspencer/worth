# Milestone 13 Engineering Spec: End-To-End Causality, Failure Taxonomy, And Bridge Certification

> **Status:** Complete
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-12b.md](./milestone-12b.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Running closeout:** [milestone-13-closeout.md](./milestone-13-closeout.md)
>
> **Signal companion:** [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
>
> **Relational companion:** [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
>
> **Bridge DX companions:** [dx_plan.md](./dx_plan.md), [dx_canonical_surface_spec.md](./dx_canonical_surface_spec.md), [dx_boundary_spec.md](./dx_boundary_spec.md), [dx_boundary_cleanup_spec.md](./dx_boundary_cleanup_spec.md)
>
> **Primary architectural driver:** turn the bridge from a collection of individually strong capabilities into one certifiable causal protocol boundary with one canonical reference workload, one bridge-native failure topology, one coherent diagnostics entrypoint, and one machine-checkable certification bundle story spanning truth commit, routing, branch-local evaluation, speculative discard or commit, writeback, and replay

## Summary

Milestones 1 through 12 gave the bridge real capabilities:

- truth-to-signal routing
- aspect mapping
- lineage continuity
- historical and branch-aware reads
- bulk planning
- protocolized stream and source contracts
- structural remapping
- merge-aware consumption
- speculative coordination
- explicit policy propagation
- bridge-mediated writeback

What the bridge still lacks is one final proof:

- can those capabilities compose into one end-to-end causal story
- can hostile failures be localized into bridge-native types instead of host
  strings
- can an auditor diagnose the bridge from canonical artifacts alone
- can a real dual-runtime workflow prove the architecture rather than only
  exercising isolated feature lanes

Milestone 13 exists to close that gap.

It does so with two equally important deliverables:

1. one bridge-owned certification model that unifies the milestone 6 through 12
   artifact story
2. one concrete Rust-only reference workload that proves the bridge on a
   product-shaped dual-runtime scenario without making the bridge a domain
   authority

The reference workload for this milestone is a pricing-shock matrix:

- `worth-relational` owns authoritative products, components, branches, and base
  component costs
- `worth-signal` owns derived pricing nodes such as tariffs, taxes, margin, and
  final retail price
- the bridge owns causality transfer, branch-local coordination, preview or
  authoritative execution distinction, diagnostics, and replay-safe
  certification artifacts

This is intentionally Rust-only. No UI is required for the milestone. The
"slider", "split screen", and "buttons" become harness verbs and certification
lanes.

The implementation is now expected to converge on one top-level pricing-shock
workload certification bundle rather than a loose collection of scenario-local
assertions. That bundle should carry the ordinary path, hostile failure,
discard, promotion, fanout, replay, restart, merge-history, writeback, and
historical-provenance stories together.

## Goal

Make the bridge certifiable as a standalone causal protocol boundary by
shipping:

- one canonical end-to-end causality bundle model
- one bridge-native failure taxonomy covering the real adversarial surfaces
- one public diagnostics entrypoint that serves routing, history, branch,
  preview, merge, policy, writeback, and replay stories coherently
- one concrete reference workload in pure Rust that proves live updates,
  speculative forks, isolated branch comparison, discard zero-residue, and
  authoritative commit promotion under canonical bundle comparison

## Why This Milestone Exists

Milestone 13 belongs immediately after Milestone 12b because Milestone 12b is
the last point where writeback-family extensibility can still disappear into
host-local mapper folklore instead of becoming a bridge-native protocol
surface.

Milestone 13 is the first point where the bridge can honestly be asked:

- what exact truth basis produced this derived state
- what branch or preview contract governed the evaluation
- why did this route, remap, reject, merge, or write back the way it did
- if this failed, where exactly did it fail
- if I replay this from canonical artifacts alone, do I get the same answer
- if I discard a speculative branch, what proves nothing authoritative remains

It also belongs here in the roadmap sequence because the pricing-shock workload
depends on already-landed bridge capabilities:

- Milestone 4 historical and branch-aware evaluation
- Milestone 5 bulk planning and scale counters
- Milestone 10 speculative truth-branch to signal-branch coordination
- Milestone 11 request-scoped policy propagation
- Milestone 12 bridge-mediated writeback and no-op or commit classification
- Milestone 12b bridge-native extensible writeback families and mapper
  containment

Before those milestones, the workload would have been demo theater.
After those milestones, it becomes a legitimate certification target.

## Hard Part

The hard part of Milestone 13 is not creating a neat example domain.

The hard part is proving that one mixed workload can move through all of these
states without semantic drift:

- main truth branch receives rapid cost changes
- signal invalidation remains precise and deterministic
- a speculative truth branch forks from a stable basis
- branch-local cost shocks propagate only into the speculative signal branch
- main and speculative branches stay causally distinct under interleave
- speculative work can either discard with zero residue or commit through the
  existing authoritative boundary
- original execution and replay preserve the same causal meaning and failure
  meaning

If any of those stories requires host logs, adapter folklore, or "we know how
to read this test" explanation, the bridge is not yet certifiable.

## Adversarial Constraint

Milestone 13 must survive the following hostile condition:

> A Rust-only reference workload with at least 100 products, shared component
> costs, high-fanout price derivations, live cost churn equivalent to a 60 Hz
> "steel cost slider", speculative branch-local shocks such as `rubber +300%`,
> interleaved main-branch authoritative updates, preview and authoritative
> request variation, diagnostics-tier variation, restart and replay boundaries,
> strategy-bearing writeback, failure injection, and repeated discard or commit
> cycles must preserve the same main-branch truth, the same speculative-branch
> truth, the same causality digests, the same typed failures, the same no-op
> versus commit classification, the same zero-residue guarantees, and the same
> offline-diagnosable certification bundles every time.

If any path:

- lets speculative branch-local truth leak into the main branch
- lets main-branch authoritative writes corrupt or silently retarget the
  speculative branch basis
- changes pricing meaning under diagnostics-tier variation
- requires host memory or logs to explain the result
- collapses branch-local failure into untyped strings
- loses causality across routing, evaluation, writeback, or replay
- leaves temporary bridge residue after discard
- changes bundle meaning between original execution and replay

then Milestone 13 has failed.

## Explicit Assumptions

Milestone 13 makes these assumptions explicit:

- the pricing workload is a harness-owned reference domain, not bridge-owned
  business semantics
- `worth-relational` remains the owner of products, components, branch state,
  mutation history, and final truth commit
- `worth-signal` remains the owner of dependency shape, invalidation,
  recomputation, staged execution, and explanation of derived pricing nodes
- the bridge owns only the boundary story: routing, source selection,
  branch-local coordination, causality transfer, writeback handoff, failure
  localization, diagnostics, and certification artifacts
- "live graph", "speculative fork", "split-screen comparison", and "commit vs
  discard" are all expressible as harness workloads without requiring any UI
- the milestone may use a narrow reference schema such as
  `Product -> ComponentRequirement -> ComponentCost`, but that schema remains a
  certification fixture rather than a bridge API contract

If implementation pressure tries to move product, pricing, or tax semantics into
the bridge crate itself, the milestone must be revised before code lands.

## Product Decision Lock

- Milestone 13 is Rust-only certification infrastructure, not a UI milestone.
- The pricing-shock workload is mandatory as a reference certification domain.
- The reference workload must prove both ordinary live propagation and
  speculative branch coordination.
- The bridge must expose one public diagnostics entrypoint rather than a bag of
  separate milestone-local debug helpers.
- Every reference-workload run must emit one canonical certification bundle
  family that can be judged offline.
- Causality identity must remain first-class from authoritative truth trigger
  through routing, read basis, signal evaluation, writeback or no-op outcome,
  and replay.
- Failure classes must remain bridge-native and structured even when the root
  cause originated in a parent runtime or adapter.
- Main-branch and speculative-branch outcomes must remain simultaneously
  comparable without collapsing into one branch model.
- Discard is a positive artifact story, not an absence story. The bundle must
  prove residue is zero.
- Commit promotion must pass through the already-authoritative relational
  strategy boundary established by Milestone 12 and the family-admission
  boundary established by Milestone 12b.

## Scope

In scope:

- bridge-owned causality bundle unification
- bridge-owned failure taxonomy completion
- one public diagnostics entrypoint and canonical bundle access surface
- one pricing-shock reference workload with Rust harness verbs
- one mixed workload matrix covering live updates, speculative fork, discard,
  commit promotion, replay, and hostile failure injection
- certification bundles satisfying suites 25 through 27 in
  [test-requirements.md](./test-requirements.md)
- one top-level pricing-shock workload certification bundle capable of offline
  diagnosis across the ordinary, hostile, lifecycle, fanout, replay,
  restart, merge-history, writeback, and historical-provenance lanes

Out of scope:

- UI implementation
- domain-general finance framework design
- new truth authority semantics beyond the existing relational roadmap
- new signal scheduler semantics beyond the existing signal roadmap
- broad domain libraries for tariffs, tax engines, or retail policy modeling

## Governing Design Rules

### 1. The Reference Workload Must Be Domain-Shaped But Boundary-Honest

The pricing workload must be concrete enough to prove the architecture:

- products depend on components
- shared component costs fan out into many products
- price outputs compose from several derived layers
- branch-local shocks and live updates can coexist

But the bridge must not own:

- product storage
- component authority
- pricing formulas as bridge semantics
- tax or tariff policy as bridge policy

The domain lives in harness fixtures and parent-runtime adapters.
The bridge owns only the protocol boundary.

### 2. Every Story Must End In A Canonical Bundle

At minimum the milestone must unify:

- truth trigger basis
- routing basis
- truth-view basis
- policy basis
- speculative versus authoritative request basis
- signal explanation basis
- writeback or no-op basis
- failure basis
- replay basis
- residue or cleanup basis

No story may terminate in "inspect the log" or "compare runtime state
manually."

For the pricing-shock reference workload specifically, the canonical bundle
must be sufficient to answer:

- what the main branch did under ordinary live churn
- what the speculative branch did under branch-local shock
- what replay preserved from the ordinary route and branch basis
- what discard proved about residue
- what promotion proved about authority-boundary handoff
- what the 100-product fanout lane proved about breadth and boundedness
- what restart-safe replay preserved
- what restart-shaped drift rejection localized and counted
- what retained historical commits reveal about the upstream shock criteria

### 3. Main And Speculative Branches Must Be Comparable Without Identity Fusion

The milestone must support a reference workload that can answer:

- what is true on the main branch right now
- what is true on the speculative branch right now
- what shared basis did they fork from
- what exact deltas differ between them
- what causality records are branch-local versus shared

This is the Rust-only equivalent of the requested split-screen comparison. The
comparison is artifact-grade, not UI-grade.

### 4. Discard Must Prove Zero Residue Mechanically

Discard is not complete because memory "probably" dropped.

The milestone must emit explicit residue evidence covering:

- authoritative truth residue
- bridge route or replay residue
- temporary source or snapshot residue
- temporary signal-branch residue
- temporary writeback artifact residue

If any residue category cannot be proven zero from canonical artifacts and
bounded counters, the discard story is incomplete.

### 5. Failure Taxonomy Must Be Bridge-Native

Milestone 13 must collapse the bridge's real failure surfaces into one typed
taxonomy.

Minimum families:

- stream and checkpoint failures
- source capability and source transport failures
- routing and mapping failures
- continuity and remap failures
- merge interpretation and merge denial failures
- preview and speculative lifecycle failures
- policy validation and admission failures
- writeback and authority-boundary failures
- replay mismatch and certification bundle insufficiency failures

Host strings may decorate these failures. They may not define them.

### 6. Bundle Sufficiency Must Be Tested Offline

Certification Matrix Sufficiency is not a documentation claim.
It is a proof obligation.

Milestone 13 must include at least one harness lane where pass or fail is
judged from canonical bundles alone with no live runtime access.

## Complexity Contracts

Milestone 13 must name and prove boundedness for:

- reference workload bundle assembly
- branch comparison bundle assembly
- discard residue proof assembly
- failure localization lookup
- diagnostics entrypoint bundle reconstruction
- replay bundle comparison

Minimum counters:

- `causality_bundle_count`
- `causality_bundle_replay_match_count`
- `causality_bundle_replay_mismatch_count`
- `failure_taxonomy_classification_count`
- `failure_taxonomy_unclassified_count`
- `diagnostics_entrypoint_request_count`
- `diagnostics_entrypoint_reconstruction_count`
- `speculative_branch_bundle_count`
- `speculative_discard_residue_check_count`
- `speculative_discard_residue_nonzero_count`
- `branch_comparison_bundle_count`
- `offline_bundle_diagnosis_count`
- `offline_bundle_insufficiency_count`

No implementation may require scanning arbitrary historical host logs or
process-local scratch state to classify a Milestone 13 result.

## Phases

### Phase 1: Canonical Reference Workload And Causality Bundle Model

Define and implement:

- one pricing-shock reference workload in Rust harness form
- canonical workload fixtures for products, component costs, and derived price
  nodes
- bridge-owned end-to-end causality bundle records spanning truth trigger,
  routing, read basis, signal explanation, writeback or no-op outcome, and
  replay identity
- branch-comparison and residue-proof bundle shapes for speculative flows

Phase 1 is complete only when the bridge can run:

- a live main-branch cost wave
- a speculative `rubber +300%` branch shock
- a main-vs-speculative branch comparison
- a discard or commit decision

and emit canonical bundles for each step.

### Phase 2: Failure Taxonomy And Adversarial Certification Matrix

Implement:

- one bridge-native failure taxonomy spanning milestone 6 through 13 surfaces
- failure-localization bundles that identify exact failed boundaries
- adversarial workload lanes covering:
  - main-branch live churn
  - speculative fork isolation
  - speculative discard zero-residue
  - speculative commit promotion
  - diagnostics-tier perturbation
  - replay and restart parity
  - writeback failure containment
  - intentional wrong-branch, wrong-policy, wrong-source, and wrong-authority
    requests

Phase 2 is complete only when suites 25 through 27 have honest control, hostile,
and replay lanes over the reference workload and the bridge never degrades into
untyped failure strings.

### Phase 3: Public Diagnostics Entrypoint And Offline Bundle Sufficiency

Ship:

- one coherent public diagnostics entrypoint for bridge certification artifacts
- canonical offline bundle export or retrieval surfaces
- bundle sufficiency checks proving routing, branch, merge, historical, policy,
  preview, and writeback stories can be diagnosed from bundles alone
- exact counter assertions for representative Milestone 13 certification lanes

Phase 3 is complete only when an auditor can diagnose a failed reference
workload run from canonical bridge artifacts without live runtime memory.

## Must Ship

- one Rust-only pricing-shock reference workload
- one canonical end-to-end causality bundle family
- one bridge-native failure taxonomy covering milestone 6 through 13 surfaces
- one public diagnostics entrypoint for bridge artifacts
- one main-vs-speculative comparison bundle shape
- one discard residue proof bundle shape
- one commit-promotion bundle shape
- exact counters for bundle assembly, failure classification, residue checks,
  and offline sufficiency checks
- certification satisfying suites 25 through 27

## Must Preserve

- truth authority remains in `worth-relational`
- derived execution authority remains in `worth-signal`
- the bridge remains a protocol boundary, not a domain runtime
- speculative and authoritative outcomes remain mechanically distinct
- diagnostics richness changes retained detail only, not semantic truth
- replay remains canonical and bundle-equivalent
- discard leaves zero authoritative or bridge residue
- commit promotion never bypasses the Milestone 12 authority boundary or the
  Milestone 12b family-admission boundary

## Acceptance Evidence

Milestone 13 is complete only when the bridge harness can prove all of the
following on the reference workload:

- a live 100-product cost wave over a shared component such as `steel` updates
  the intended final price surfaces deterministically and replay-equivalently
- a speculative branch-local shock such as `rubber +300%` remains isolated from
  the main branch under interleaved main-branch updates
- main and speculative branch bundles compare against the same fork basis while
  preserving distinct branch-local causal identity
- discard of the speculative branch leaves zero authoritative truth residue,
  zero bridge residue, and zero stale branch-visible signal residue
- commit promotion of the speculative branch produces canonical authoritative
  outcomes through the existing writeback strategy boundary
- diagnostics-tier variation changes retained richness only, not causality,
  price meaning, branch comparison meaning, or failure class
- replay reconstructs the same causality digest, failure class, branch
  comparison result, and discard or commit classification from canonical
  artifacts alone
- offline bundle diagnosis can distinguish routing, branch isolation, policy,
  source, preview, merge, writeback, and residue failures mechanically
- retained historical pricing commits can reveal their upstream shock criteria
  through bridge-visible truth and canonical bundle artifacts alone
- exact Milestone 13 counters match their declared values for representative
  certification lanes
- certification suites 25 through 27 pass with canonical machine-checkable
  bundles

## Architectural Notes

Milestone 13 should extend the bridge crate with subdomains such as:

- `certification/causality_bundle.rs`
- `certification/reference_workload.rs`
- `certification/reference_matrix.rs`
- `failure/taxonomy.rs`
- `failure/localization.rs`
- `diagnostics/entrypoint.rs`
- `diagnostics/certification_bundle.rs`
- `speculation/residue.rs`
- `comparison/branch_comparison.rs`

The reference workload should not live as public bridge business API.
It should live as certification substrate and harness-owned fixtures.

Expected facade growth should look more like:

- `capture_causality_bundle(...)`
- `classify_bridge_failure(...)`
- `compare_branch_bundles(...)`
- `prove_speculative_discard_zero_residue(...)`
- `export_certification_bundle(...)`
- `diagnostics_entrypoint(...)`

and not like one giant "debug everything" method.

## Test And Harness Model

Milestone 13 is certification work first.

The reference workload must define at least these scenario verbs:

- `apply_component_cost_wave("steel", ...)`
- `fork_speculative_branch(...)`
- `apply_speculative_component_shock("rubber", +300%)`
- `capture_branch_price_matrix(...)`
- `discard_speculative_branch(...)`
- `execute_pricing_strategy(...)`

The harness must vary:

- branch identity
- truth-view basis
- diagnostics tier
- policy bundle
- replay boundary
- writeback mode
- failure injection boundary
- source adapter shape where admitted

Minimum certification outputs:

- `causality_digest`
- `routing_digest`
- `truth_view_digest`
- `branch_comparison_digest`
- `price_matrix_digest`
- `speculative_residue_digest`
- `failure_digest`
- `replay_digest`
- `diagnostics_digest`
- `counter_snapshot`

## Anti-Patterns Explicitly Rejected

- building a demo-only host binary and calling the bridge certified
- putting pricing semantics into the bridge core
- proving discard by absence of panic or by ad hoc memory inspection only
- using host logs as the primary failure surface
- treating replay parity as "final prices match" while causality or failure
  meaning drifted
- collapsing speculative and authoritative requests into one ambient mode
- adding separate diagnostics surfaces for each milestone instead of one bridge
  diagnostics entrypoint

## Sequencing Notes

Milestone 13 builds directly on:

- Milestone 10 speculative and preview boundary work
- Milestone 11 policy propagation
- Milestone 12 writeback and authority-boundary causality preservation
- Milestone 12b extensible writeback-family admission and mapper containment

It also depends on the bridge DX hardening program defined in:

- [`dx_plan.md`](./dx_plan.md)
- [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`dx_boundary_cleanup_spec.md`](./dx_boundary_cleanup_spec.md)

The reference workload should be implemented against that hardened public bridge
boundary rather than against ad hoc subsystem seams.

For execution order, [`dx_plan.md`](./dx_plan.md)
is now the implementation-guide authority.
Milestone 13 work should track the active DX phase explicitly:

- finish Phase 2 if the code surface still needs hardening to support honest
  workload usage
- move into Phase 3 to widen the pricing-shock matrix and hostile
  certification pressure
- only treat docs or publication polish as primary work once Phase 4 is active

It should also deliberately reuse:

- `worth-signal` adversarial-matrix discipline for bug-class-oriented
  certification structure
- `worth-signal` fintech-style domain-matrix thinking for keeping the reference
  workload honest rather than decorative

## Self-Check

- This solves a real structural problem: the bridge still lacks one final
  certifiable causal story across its milestone surfaces.
- The adversarial constraint is precise and load-bearing: rapid fanout updates,
  speculative branch isolation, discard zero-residue, failure localization, and
  replay parity are the real failure modes.
- Authority boundaries are preserved: `worth-relational` owns truth,
  `worth-signal` owns derivation, the bridge owns coordination and proof.
- The spec defines proof obligations, not chores: bundle equivalence, failure
  taxonomy sufficiency, residue proof, offline diagnosability, and replay
  parity are all machine-checkable.
- A competent engineer can map this into honest modules, types, and tests.
- The milestone belongs in sequence: it closes the bridge only after writeback,
  policy, and speculation semantics exist.

## Closeout Standard

Milestone 13 is complete only when the bridge can run a Rust-only pricing-shock
reference workload that proves live high-fanout propagation, speculative branch
isolation, main-vs-speculative comparison, discard zero-residue, authoritative
commit promotion, typed hostile-failure localization, end-to-end causality
bundle equivalence, historical shock provenance, and offline
certification-bundle sufficiency from canonical artifacts alone.

If the bridge still needs host logs to explain failures, if speculative
discard still leaves unproven residue, if replay can still preserve final price
truth while losing causal identity, if failure classes still collapse into
adapter-local strings, or if the reference workload still looks like a demo
instead of a certification harness, Milestone 13 is not complete.
