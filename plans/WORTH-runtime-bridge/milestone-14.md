# Milestone 14 Engineering Spec: Bridge-Native Subscription Declaration Families, Admission, and Lifecycle

> **Status:** Implemented and closed 2026-04-21
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-13.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/milestone-13.md)
>
> **Prior closeout:** [milestone-13-closeout.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/milestone-13-closeout.md)
>
> **Bridge certification companion:** [test-requirements.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/test-requirements.md)
>
> **Signal companion:** [milestone-11-plan.md](/Users/shepworth/Documents/programming/WORTH/plans/worth_signal/milestone-11-plan.md)
>
> **Earlier subscription substrate companions:** [milestone-2.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/milestone-2.md), [milestone-3.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/milestone-3.md)
>
> **Primary architectural driver:** turn fine-grained subscription slices, basis-aware bridge artifacts, and `worth-signal` observation strategies into one bridge-owned subscription declaration framework that a manual host can admit, activate, diagnose, and replay without host-local folklore

## Summary

Milestones 1 through 13 gave the bridge the ingredients for subscriptions:

- canonical patch routing and snapshot-backed evaluation
- fine-grained subscription slices
- lineage-aware continuity
- historical and branch-aware truth basis
- protocolized stream and source contracts
- preview and speculative boundary work
- writeback-grade causality and certification artifacts

What the bridge still lacks is one lower-level product surface that says:

`a host may declare an ongoing truth-backed subscription family once, bind it to an explicit truth basis once, admit it once, lower it once into admitted worth-signal observation strategies, and then reason about that subscription as a bridge protocol entity rather than as host-local glue`

That missing surface matters because the stack now wants two things at once:

1. WORTH Query should be able to automatically lower live query intent into
   bridge-native subscription machinery.
2. A careful manual host should still be able to assemble the same class of
   subscription explicitly without going through Query.

Milestone 14 exists to close that gap.

It does not invent a second observation runtime.

It freezes the bridge-owned declaration and admission framework that sits
between:

- truth-side slice and basis semantics already owned by the bridge
- `worth-signal` observation policies and extensible delivery strategies that
  already exist in the signal runtime

This milestone is therefore not "add subscriptions somehow."

It is:

- bridge-native subscription declaration families
- bridge-owned admission and rejection artifacts
- explicit basis binding
- explicit lifecycle typestate
- canonical family identity plus instance identity
- explicit lowering into admitted `worth-signal` observation and delivery
  strategies
- replay-safe diagnostics explaining why a declaration was admitted, denied,
  or shaped the way it was

## Goal

Make bridge subscriptions first-class protocol entities by shipping one
bridge-owned declaration-family framework with explicit admission, basis
binding, lifecycle identity, and `worth-signal` strategy lowering.

## Why This Milestone Exists

Milestone 14 belongs immediately after Milestone 13 because Milestone 13 is the
first point where the bridge became a certifiable causal protocol boundary
rather than a set of individually strong features.

That certification work closed:

- one coherent diagnostics entrypoint
- one machine-checkable artifact story
- one proof that branch, preview, writeback, and replay semantics can compose

What it did not close is a manual-host subscription contract.

Right now the bridge can explain:

- how truth routes to fine-grained slices
- how continuity is preserved
- what basis a read used
- how replay or preview behaved

But it still cannot honestly say:

- what exact bridge-native subscription family has been declared
- how that declaration is admitted or rejected
- what makes two host declarations the same subscription versus different
- what signal observation strategy was selected as a consequence of that
  declaration

Without this milestone:

- Query would have to synthesize a large part of the subscription story above
  the bridge
- manual hosts would still be assembling long-lived observation from slices,
  streams, and observer handles without one canonical bridge contract
- Store-facing subscription support would have weak lower-level identity to
  persist

Milestone 14 therefore earns its place by solving the next real structural
problem after bridge certification: turning the existing subscription substrate
into one bridge-native, family-aware protocol surface.

## Hard Part

The hard part is not tracking handles.

The hard part is freezing one exact separation among four things that naive
designs blur together:

- truth-side fine-grained slice identity
- bridge subscription family meaning
- signal observation and delivery strategy selection
- host-local consumer or callback shape

The design fails if:

- the bridge treats one fixed "subscription" shape as universal and erases the
  family distinctions higher layers need
- the bridge reaches into `worth-signal` internals instead of lowering into
  admitted observation and delivery strategies
- declaration meaning is reconstructed from host-local builder order or ambient
  consumer state
- basis binding is inferred rather than explicit
- lifecycle identity is fused with raw stream identity or slice identity
- diagnostics can explain the slices but not the declared subscription family
- declaration admission performs repeated broad scans over family registries,
  strategy sets, or slice registrations on every host call
- cheap-looking facade calls hide graph walks, repeated sort/canonicalize work,
  or per-call allocation churn

Milestone 14 therefore has to make declarations strong enough that:

- Query can later compile to them honestly
- manual hosts can use them directly
- `worth-signal` remains the owner of observation semantics
- the bridge remains the owner of truth-side subscription protocol semantics
- declaration and admission cost stay bounded by normalized declaration width,
  admitted family metadata, and actual lowered slice width rather than by total
  registry size or total bridge history

## Adversarial Constraint

Milestone 14 must survive the following hostile condition:

> A long-lived host with explicit snapshot-bound truth, branch-head truth,
> overlapping fine-grained slice registrations,
> multiple declaration-builder paths, diagnostics-tier variation, restart and
> replay pressure, and more than one admitted subscription family must admit
> the same semantically equivalent declaration into the same canonical
> subscription-family identity, the same basis binding, the same lowered slice
> set, and the same selected `worth-signal` observation strategy every time,
> while rejecting unsupported or ambiguous declarations explicitly before
> activation.

If any supported path:

- makes declaration meaning depend on builder order
- silently falls back from one family to another
- silently widens basis selection to "latest reachable truth"
- fuses subscription identity with stream identity, consumer identity, or raw
  slice identity
- allows host-local callback shape to redefine bridge meaning
- or cannot replay the same declaration and admission result from canonical
  bridge artifacts alone

then Milestone 14 has failed.

## Explicit Assumptions

- `worth-relational` remains the authority for truth identity, lineage,
  historical retention, branch semantics, and truth-view legality.
- `worth-signal` already owns observation policy, delivery strategy,
  coalescing, commit-bounded delivery, and observer lifecycle semantics.
- the bridge already owns fine-grained slice identity, continuity artifacts,
  basis-aware reads, and replay-safe diagnostics from prior milestones.
- this milestone does not invent app-level watchers, UI-facing callbacks, or
  frontend ergonomics.
- this milestone may define bridge-owned declaration-family registries,
  lowering records, and typestates, but it must not redefine either truth
  semantics or signal observation semantics.
- the bridge facade remains the only public surface for bridge subscription
  declaration and admission.
- declaration-family and strategy semantics must be replay-stable across
  process restart and diagnostics-tier variation once the bridge registry is
  frozen for a given build/runtime boundary.

## Product Decision Lock

- bridge subscriptions are family-aware declarations, not one universal
  subscription shape.
- declaration-family selection is a bridge concern; raw host callback shape is
  not.
- declaration-family registration must freeze before declaration admission; no
  admitted declaration may depend on mutable runtime registration order.
- basis binding is explicit and typed for explicit snapshot-bound and
  branch-head truth views in Milestone 14.
- the bridge lowers into admitted `worth-signal` observation policies and
  delivery strategies; it does not invent a second observation engine.
- subscription family identity and subscription instance identity are distinct
  from stream identity, consumer identity, and raw slice identity.
- unsupported family or basis combinations fail at admission time, not later
  during activation.
- lifecycle states are bridge-owned protocol states, not ambient host booleans.
- compile-time construction should make it impossible for external callers to
  synthesize admitted declarations, lifecycle handles, or basis-checked states
  without passing through bridge-owned proving functions.

Normative consequence:

- host-local "subscribe however you want and we will infer the family later" is
  out of spec
- a default catch-all subscription family is out of spec
- direct bridge code that reaches past admitted signal strategy surfaces is out
  of spec
- implicit basis fallback to latest current truth is out of spec

## Scope

### In Scope

- one bridge-owned subscription declaration-family framework
- one bridge-owned frozen declaration-family registry with canonical family ids
- canonical family identity and canonical subscription instance identity
- basis-bound declaration artifacts for explicit snapshot-bound and
  branch-head truth views
- admission and rejection artifacts for supported and unsupported declaration
  combinations
- explicit lowering from bridge declaration families into admitted
  `worth-signal` observation policies and delivery/coalescing strategies
- one admitted signal-strategy registry or equivalent canonical strategy table
  with family-specific admissibility
- lifecycle typestate carried by declaration artifacts, admitted artifacts,
  activation-ready handles, deactivated handles, and typed rejection artifacts
- diagnostics and replay artifacts explaining declaration meaning, basis
  binding, family selection, slice lowering, and signal-strategy selection
- harness coverage satisfying suites 28 through 30 in
  [test-requirements.md](/Users/shepworth/Documents/programming/WORTH/plans/worth-runtime-bridge/test-requirements.md)

### Explicitly Out Of Scope

- active delivery fanout, continuation, resume, and checkpoint semantics beyond
  the identity and lifecycle preparation needed for declaration and admission
- shared-consumer delivery behavior
- preview discard and promotion lifecycle beyond declaration-time basis
  legality
- durable subscription persistence or store-backed restart support
- query-owned semantic subscription family selection
- frontend, wasm, or app-facing watch ergonomics

Milestone 14 must leave clean extension points for Milestones 15 and 16 without
pretending to ship them now.

## Governing Design Rules

### 1. The Bridge Owns Declaration Families, Not Observation Semantics

The bridge must define:

- what declaration families exist
- when declaration families freeze into a canonical registry
- what truth-side basis each family may bind to
- what fine-grained slice and continuity artifacts a family consumes
- what admitted signal strategies a family may lower into

The bridge must not define:

- what observation means inside `worth-signal`
- how signal internally stores observers
- what UI-facing or app-facing callback semantics should look like

The bridge answers:

- what was declared
- what basis was requested
- what family was selected
- what admitted signal strategy was lowered

It must not answer:

- how `worth-signal` executes that strategy once lowered

The public bridge contract must therefore expose:

- one family registry/freeze boundary
- one declaration normalization boundary
- one admission boundary

and not allow external callers to bypass them.

The performance consequence is:

- family lookup must consume pre-frozen registry structures rather than
  scanning mutable registration bags
- strategy admissibility must consume pre-lowered family metadata rather than
  rediscovering compatibility at admission time

### 2. Family Identity And Instance Identity Must Both Be Canonical

Milestone 14 must introduce:

- canonical declaration-family identity
- canonical admitted subscription instance identity

Those identities must be:

- independent of host builder order
- independent of diagnostics richness
- independent of raw callback identity
- replay-safe from canonical bridge artifacts alone

The family identity answers:

- what semantic declaration family is this

The instance identity answers:

- what basis-bound admitted subscription instance of that family is this

Neither may be collapsed into stream identity, consumer identity, or raw slice
identity.

Canonical declaration-family identity must include only:

- declaration family id
- normalized declaration shape
- normalized truth-slice intent
- normalized basis request class where the family semantics require it

Canonical declaration-family identity must exclude:

- consumer fanout
- pacing or coalescing state
- diagnostics richness
- activation state
- callback/object address identity

Canonical admitted-instance identity must include only:

- canonical declaration-family identity
- admitted truth basis identity
- admitted lowered slice set identity
- admitted signal strategy identity

Canonical admitted-instance identity must exclude:

- consumer set membership
- lifecycle transitions after admission
- stream checkpoint position
- retained diagnostics detail

### 3. Basis Binding Must Be A Proof-Bearing Admission Step

Basis selection is not a late runtime convenience.

It is part of admission.

Every admitted declaration must carry proof of:

- which truth-view class was requested
- which truth-view class was admitted
- whether the family allows that basis class
- whether ambiguity or illegality was rejected

No later lifecycle or delivery phase should need to rediscover basis meaning.

Where a family's admissible basis classes are statically knowable, the spec
prefers compile-time admissibility encoding through witness types, family-
specific traits, or equivalent sealed capability surfaces rather than late
stringly rejection.

At minimum, external callers must be unable to invoke activation-facing APIs
with a declaration that has not passed the basis-proof boundary.

### 4. Lowering To Signal Strategies Must Be Explicit And Replay-Visible

The bridge must not treat signal strategy choice as an implementation detail.

Every admitted declaration must lower into:

- one admitted observation policy identity
- one admitted delivery or coalescing strategy identity
- one canonical lowered slice set

Those lowerings must be retained in replay-visible bridge artifacts so an
auditor can answer:

- why did this bridge family select this signal strategy
- and not another one

The bridge must expose a canonical admitted-strategy vocabulary rather than an
open-ended runtime string or mapper-local convention.

For each declaration family, the spec requires:

- an explicit admitted strategy set
- explicit typed rejection when no admitted strategy exists
- replay-visible proof of which admitted strategy was chosen and why fallback
  or rejection occurred

### 5. Declaration, Admission, And Lifecycle Must Be Phase-Typed

Milestone 14 must not use one mutable bag that accumulates state.

The public bridge contract should read like a proof chain:

- raw declaration intent
- basis-checked declaration
- admitted declaration
- lifecycle-typed instance handle
- rejected declaration artifact

If a rejected declaration can be mistaken for an admitted declaration, or if an
admitted declaration can be activated without basis proof, the phase boundary
is structurally weak.

The spec requires these proof-bearing states to be represented as distinct
types with:

- private fields
- non-public constructors
- bridge-owned transition functions
- facade-only public construction paths

Compile-fail tests should prove that external code cannot:

- synthesize an admitted declaration
- skip basis proof
- construct lifecycle handles directly
- transition rejected declarations into activation-ready handles

### 6. Manual Host Construction Must Stay Honest

This milestone exists partly so manual hosts can do the same class of work that
Query will later automate.

That means:

- the declaration-family surface must be explicit enough for careful manual use
- but not so low-level that hosts must reconstruct slice meaning or signal
  semantics themselves

The bridge should expose one declaration framework, not a checklist of
unrelated helper calls.

That framework should be structurally declarative:

- a host declares intent once
- the bridge freezes/normalizes it once
- the bridge proves basis legality once
- the bridge lowers it once

and not require manual sequencing of loosely related helper calls.

The API-shape consequence is:

- facade entrypoints that may normalize, admit, or lower declarations must
  surface structural counters or reports sufficient to show what work they did
- no facade method should read like a cheap getter if it may canonicalize,
  sort, lower, or bind truth basis

### 7. Performance-Critical Facts Must Be Precomputed At Freeze Time

Milestone 14 should not rediscover family semantics on every declaration.

The family registry freeze boundary must precompute and retain:

- canonical family ordering
- family id lookup structures
- admitted basis-class masks or equivalent proof tables
- admitted signal-strategy sets per family
- any canonical slice-category ordering needed for lowering

Admission should consume those precomputed structures directly.

If family admissibility, strategy admissibility, or canonical ordering are
recomputed per declaration, the design violates the bridge's own planning-first
stance.

### 8. Bulk Declaration Admission Must Remain Available

Hosts will not always declare one subscription at a time.

Even if Milestone 14's primary facade remains declaration-by-declaration, the
architecture must preserve an honest bulk boundary so later work does not force
bulk domains through scalar orchestration surfaces.

That means:

- normalized declarations should be representable as a bulk packet
- admission and lowering counters should be interpretable across one-or-many
  declaration lanes
- canonicalization and ordering work should be amortizable across a batch where
  intermediate host observation is not required

Milestone 14 does not need to ship a rich bulk API, but it must not paint
Milestone 15 or Query lowering into a scalar-only corner.

### 9. Allocation And Artifact Construction Must Follow Lifecycle Scope

Declaration and admission are bridge control-plane work, but they still sit on
hot usage paths for live systems.

This milestone should therefore require:

- pre-sized or reusable buffers for normalized declaration parts where width is
  predictable
- reuse-friendly retained artifact assembly rather than per-call heap churn for
  identical small shapes
- no cloning of lowered slice sets unless the clone buys an explicit boundary
  such as retained replay artifact ownership

If representative declaration flows allocate proportional to registry size or
clone slice vectors merely for convenience, the implementation is
architecturally wasteful.

## Complexity Contracts

Milestone 14 must name and prove boundedness for:

- declaration normalization
- declaration-family registry freeze
- basis admission
- declaration-family lowering
- signal-strategy selection
- lifecycle record emission
- replay reconstruction of declaration and admission artifacts

The named boundary contracts should be stated in terms of:

- `d`: normalized declaration width
- `f`: admitted family metadata width for the selected family only
- `s`: admitted strategy count for the selected family only
- `l`: lowered slice count for the admitted declaration
- `b`: bulk declaration count when declarations are admitted as one packet

Representative complexity targets:

- declaration normalization: `O(d log d)` if canonical sorting is required,
  otherwise `O(d)`
- family lookup after registry freeze: `O(1)` or `O(log F)` where `F` is total
  family count, but never linear scan over all families on the hot path
- basis admissibility lookup: `O(1)` against precomputed family metadata
- strategy admissibility and selection: `O(s)` or better for the selected
  family only, never scan across unrelated families
- lowering artifact assembly: `O(l)`
- replay reconstruction: `O(d + l)` against retained canonical artifacts, not
  against live host registration state
- batch admission where supported: amortized by `b` without repeating registry
  freeze work per declaration

Minimum counters:

- `subscription_declaration_count`
- `subscription_family_registry_freeze_count`
- `subscription_family_selection_count`
- `subscription_basis_admission_count`
- `subscription_basis_rejection_count`
- `subscription_signal_strategy_selection_count`
- `subscription_signal_strategy_fallback_count`
- `subscription_lifecycle_record_count`
- `subscription_replay_reconstruction_count`
- `subscription_replay_mismatch_count`
- `subscription_diagnostics_bundle_count`
- `subscription_family_lookup_scan_count`
- `subscription_basis_lookup_count`
- `subscription_strategy_candidate_count`
- `subscription_lowered_slice_count`
- `subscription_normalization_sort_count`
- `subscription_bulk_admission_count`
- `subscription_allocation_count`
- `subscription_clone_count`

No implementation may:

- scan arbitrary host registrations repeatedly during admission
- permit mutable family semantics after registry freeze
- rediscover declaration-family meaning during lifecycle transitions
- linearly scan all registered families for every declaration on the hot path
- linearly scan all admitted strategies outside the selected family
- allocate in proportion to total registry size for ordinary declaration paths
- clone lowered slice collections without an explicit retained-artifact or
  ownership-boundary reason
- or require live host callback objects to reconstruct declaration meaning

## Phases

### Phase 1: Canonical Declaration-Family Model And Identity

Define and implement:

- one minimal admitted declaration-family starter set sufficient to prove the
  framework end to end before broadening family coverage
- a concrete normalized declaration artifact shape rather than an abstract
  "normalized declaration" placeholder
- the bridge-owned subscription declaration-family taxonomy
- the canonical family registry/freeze boundary and registry identity artifact
- canonical declaration-family identity and canonical admitted-instance identity
- one family-aware declaration normalization path
- proof-bearing declaration and rejection artifact types

Phase 1 implementation guidance:

- begin with the smallest admitted family set that can exercise canonical
  declaration identity without dragging Milestone 15 delivery concerns into the
  implementation
- prefer one detail-oriented family and one collection-membership-oriented
  family as the initial proving lane; grouped or audit-oriented families may
  remain explicitly unadmitted until the declaration framework is stable
- define the normalized declaration artifact with explicit fields for:
  - requested family id or family selector
  - requested truth-basis class
  - normalized truth-slice intent set
  - optional family-specific parameter payload
  - optional delivery-intent class if it affects family identity
  - diagnostics payload retained outside canonical identity
- implement canonicalization for that artifact first, and only then freeze
  family registry identity around it
- make the first registry implementation closed and code-defined; later
  extensibility must grow from a proven freeze boundary rather than an early
  plug-in surface

Phase 1 is complete only when the bridge can represent semantically equivalent
host declarations as one canonical family identity and one canonical normalized
declaration artifact independent of host builder order.

### Phase 2: Basis Admission And Signal-Strategy Lowering

Implement:

- basis-bound declaration admission for explicit snapshot-bound and
  branch-head truth views
- typed rejection for unsupported, ambiguous, and illegal basis/family
  combinations
- explicit lowering from admitted bridge declaration families into admitted
  `worth-signal` observation and delivery/coalescing strategies
- replay-visible admission and lowering artifacts
- explicit family-to-strategy admissibility tables and typed rejection when no
  admitted strategy exists
- precomputed family metadata structures sufficient to keep basis and strategy
  lookup out of repeated broad scans

Phase 2 implementation guidance:

- carry basis proof by wrapping existing bridge truth-basis artifacts rather
  than inventing a second basis model for Milestone 14
- make the first admission path consume the normalized declaration artifact from
  Phase 1 directly; do not create a parallel "admission request" shape unless
  it is mechanically derived and retained
- define one concrete bridge-side lowering seam, such as a signal-strategy
  lowering table or lowerer trait, that maps:
  - admitted family identity
  - admitted basis class
  - normalized family parameters
  - lowered slice set
  into canonical admitted `worth-signal` strategy descriptors
- Milestone 14 lowering may stop at canonical admitted strategy descriptors and
  retained lowering artifacts; it does not need to perform full live observer
  registration yet
- if grouped or other higher-complexity families require materially different
  basis or lowering logic, leave them explicitly unadmitted rather than
  widening the first lowering seam into a catch-all abstraction

Phase 2 is complete only when an admitted declaration can be explained from:

- family identity
- basis identity
- lowered slice set
- selected signal strategy identity

with no host-local reconstruction.

### Phase 3: Lifecycle Typestate, Diagnostics, And Certification Lanes

Ship:

- bridge-native lifecycle typestate for declaration artifacts, admitted
  artifacts, activation-ready handles, deactivated handles, and typed
  rejection artifacts
- one coherent diagnostics path for declaration, basis, family, and strategy
  selection
- compile-fail or equivalent enforcement coverage for facade-only proving and
  illegal state construction denial
- harness and certification coverage for suites 28 through 30
- exact counter assertions for representative declaration, rejection, and
  replay lanes
- exact counter assertions proving representative admission lanes do not widen
  family lookup breadth, strategy lookup breadth, allocation count, or clone
  count beyond declared bounds

Phase 3 implementation guidance:

- lifecycle bring-up should stop at bridge-owned activation-ready or
  deactivation-ready handles plus canonical transition artifacts; Milestone 14
  does not need to ship shared delivery or live fanout behavior
- if an activation-facing adapter is needed for compile-time proof, keep it a
  thin stub over retained admission artifacts rather than letting Milestone 14
  absorb Milestone 15 execution semantics
- diagnostics should first explain:
  - normalized declaration contents
  - admitted family identity
  - basis proof result
  - lowered slice identity
  - selected signal strategy descriptor
  before adding richer lifecycle storytelling
- replay should reconstruct declaration and admission results solely from
  retained artifacts emitted in Phases 1 and 2; it should not consult live host
  observer state
- compile-fail coverage should land alongside the facade surface, not as a late
  cleanup pass after lifecycle types exist

Phase 3 is complete only when an auditor can compare declaration equivalence,
basis-binding outcomes, and lifecycle replay parity from canonical bridge
artifacts alone.

## Must Ship

- one bridge-native subscription declaration-family framework
- one frozen declaration-family registry with replay-visible registry identity
- one canonical declaration normalization path
- one canonical family identity plus one canonical admitted-instance identity
- basis-bound declaration admission for explicit snapshot-bound and
  branch-head truth views
- typed rejection artifacts for unsupported, ambiguous, and illegal
  declaration combinations
- explicit lowering into admitted `worth-signal` observation policies and
  delivery/coalescing strategies
- one canonical admitted-strategy vocabulary and family-to-strategy
  admissibility surface
- bridge-native lifecycle typestate for declaration artifacts, admitted
  artifacts, activation-ready handles, deactivated handles, and typed
  rejection artifacts
- declaration, admission, basis, and strategy diagnostics visible through the
  bridge diagnostics entrypoint
- retained declaration/admission artifacts with enough structured fields to
  explain family identity, basis identity, lowered slice identity, selected
  strategy identity, rejection class, and registry identity
- exact counters for declaration, admission, rejection, strategy selection, and
  replay reconstruction
- certification satisfying suites 28 through 30

## Must Preserve

- truth authority remains in `worth-relational`
- observation and delivery execution authority remains in `worth-signal`
- the bridge remains a protocol boundary rather than a second observation
  runtime
- the public facade remains the only legal external construction boundary for
  bridge subscription declaration and admission
- family identity remains distinct from slice identity, stream identity, and
  consumer identity
- basis meaning remains explicit and typed
- diagnostics richness changes retained detail only, not family identity,
  basis identity, or selected strategy identity
- unsupported combinations fail before activation rather than degrading into
  ambient host behavior

## Acceptance Evidence

Milestone 14 is complete only when the bridge harness can prove all of the
following:

- semantically equivalent declarations within the same admitted family lower to
  the same canonical `subscription_digest`
- intentionally different declarations differ mechanically on canonical family
  identity or admitted instance identity
- family registry identity remains stable across replay-equivalent runs and
  mismatches localize mechanically when registry semantics drift
- basis binding to explicit snapshot identities and branch-head requests
  remains explicit and replay-safe
- unsupported, ambiguous, and illegal basis/family combinations fail before
  activation with typed failure artifacts
- every admitted declaration records which `worth-signal` observation and
  delivery strategy it selected
- lifecycle records remain canonical and replay-safe under diagnostics-tier
  variation
- external callers cannot construct admitted declarations or lifecycle handles
  without facade-owned proof transitions
- representative declaration lanes keep family lookup, normalization sort,
  and declaration counters within the declared boundary contracts
- no representative declaration or replay lane requires host-local callback
  objects or host logs to explain the result
- certification suites 28 through 30 pass with canonical machine-checkable
  bundles

## Architectural Notes

Milestone 14 should extend the bridge crate with subdomains such as:

- `subscription/declaration_family.rs`
- `subscription/family_registry.rs`
- `subscription/declaration_identity.rs`
- `subscription/basis_binding.rs`
- `subscription/admission.rs`
- `subscription/lifecycle.rs`
- `subscription/signal_lowering.rs`
- `subscription/diagnostics.rs`
- `subscription/replay.rs`

Recommended implementation order:

- first land `subscription/declaration_family.rs`,
  `subscription/family_registry.rs`, and
  `subscription/declaration_identity.rs`
- then land `subscription/basis_binding.rs`,
  `subscription/admission.rs`, and `subscription/signal_lowering.rs`
- finally land `subscription/lifecycle.rs`,
  `subscription/diagnostics.rs`, and `subscription/replay.rs`

Expected facade growth should look more like:

- `declare_subscription_family(...)`
- `freeze_subscription_families(...)`
- `admit_subscription_declaration(...)`
- `bind_subscription_basis(...)`
- `lower_subscription_to_signal_strategy(...)`
- `inspect_subscription_declaration(...)`

and not like:

- raw host callbacks passed straight through to signal internals
- externally visible constructors for admitted or lifecycle-typed subscription
  states
- loose helper calls that force hosts to manually coordinate declaration,
  basis, and strategy selection

The declaration framework should be structurally capable of supporting:

- later bridge delivery families in Milestone 15
- later end-to-end certification bundles in Milestone 16
- later Query lowering into bridge declarations

without forcing Milestone 14 to absorb delivery fanout, checkpointing, or
durability semantics now.

Temporary seams allowed during Milestone 14 bring-up:

- signal lowering may terminate in canonical admitted-strategy descriptors and
  retained lowering records rather than full observer registration
- lifecycle may terminate in activation-ready bridge handles without shared
  consumer or fanout semantics
- replay may certify declaration and admission parity from retained artifacts
  without rehydrating live signal observers
- higher-complexity declaration families may remain explicitly unsupported
  until they can lower through the same canonical admission and artifact path

## Test And Harness Model

Milestone 14 is declaration-and-admission certification first.

The harness must define at least these scenario verbs:

- `declare_equivalent_subscription_family_variants(...)`
- `admit_subscription_for_basis(...)`
- `reject_subscription_for_basis(...)`
- `replay_subscription_declaration(...)`
- `inspect_subscription_lifecycle_record(...)`

The harness must vary:

- declaration builder order
- declaration-family selection
- truth basis class
- diagnostics tier
- replay boundary
- admitted versus rejected declaration paths

Minimum certification outputs:

- `subscription_digest`
- `subscription_basis_digest`
- `subscription_lifecycle_digest`
- `subscription_registry_digest`
- `routing_digest`
- `diagnostics_digest`
- `failure_digest`
- `replay_digest`
- `counter_snapshot`

## Anti-Patterns Explicitly Rejected

- treating one universal subscription shape as sufficient for all bridge needs
- inferring family selection from callback shape or consumer identity
- leaving family registration mutable after admission begins
- selecting signal strategies inside activation or callback dispatch instead of
  during lowering
- using open-ended strings or host mappers as the real strategy identity
- implicit fallback from explicit snapshot or branch-head requests to ambient
  latest truth
- fusing declaration identity with stream identity, slice identity, or consumer
  identity
- using host logs as the primary explanation surface for declaration or basis
  decisions
- allowing replay to reconstruct declaration meaning from ambient host state

## Sequencing Notes

Milestone 14 builds directly on:

- Milestone 2 fine-grained subscription slices
- Milestone 3 continuity identity and remap groundwork
- Milestone 4 historical and branch-aware truth basis
- Milestone 10 preview and speculative boundary work
- Milestone 13 diagnostics, replay, and certification-bundle discipline

It also depends deliberately on `worth-signal` Milestone 11:

- the bridge must lower into the already-existing signal observation-policy and
  extensible delivery-strategy substrate rather than inventing a parallel
  observation runtime

This milestone belongs before Milestone 15 because delivery, sharing, resume,
and continuation are only honest once declaration-family identity, basis
binding, and signal-strategy lowering are already canonical.

## Self-Check

- This solves a real structural problem: the bridge has fine-grained slice and
  replay substrate, but still lacks one lower-level manual-host subscription
  contract.
- The adversarial constraint is precise and load-bearing: declaration-order
  variation, basis ambiguity, multi-family lowering, and replay pressure are
  the real ways a naive design would drift.
- Authority boundaries are preserved: truth owns truth meaning, signal owns
  observation execution, the bridge owns declaration-family protocol semantics.
- The spec now demands stronger compile-time enforcement for proof states,
  facade-only construction, and illegal-transition denial.
- The spec defines proof obligations, not chores: canonical identity, typed
  basis admission, strategy-lowering visibility, replay parity, and suite
  closure are all machine-checkable.
- A competent engineer can map this into honest modules, types, and tests.
- The milestone belongs in sequence: it is the declaration-and-admission
  substrate Milestone 15 and Query lowering both need.

## Closeout Standard

Milestone 14 is complete only when the bridge can admit and reject
bridge-native subscription declaration families through one canonical
framework, bind them to explicit truth bases, lower them into admitted
`worth-signal` observation and delivery strategies, replay the same declaration
meaning from canonical artifacts alone, and certify suites 28 through 30 with
exact counters and typed failures.

If declaration meaning still depends on host builder order, if basis binding
still falls back implicitly, if selected signal strategy is still hidden inside
bridge internals, if family identity still collapses into raw slice or
consumer identity, if external callers can still synthesize admitted states
without proof-bearing transitions, or if replay still requires ambient host
state to explain an admission result, Milestone 14 is not complete.
