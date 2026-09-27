# Milestone 2: Canonical UI Source, Lowering, And Runtime Artifact

## Goal

Make repo-authored and Rust-authored Worth UI composition lower through one
snapshot-bound, proof-bearing pipeline into one canonical runtime artifact so
later hot reload, execution plans, shell semantics, Query-bound surfaces, and
tooling all consume the same platform-owned UI meaning instead of rebuilding it
from source strings, mutable registries, or host-local glue.

## Why This Milestone Exists

Milestone 1 closed the registration and snapshot authority boundary. Worth UI
now has typed capability identity, family law, immutable snapshot freeze, and
facade-only app construction.

That is necessary but not sufficient.

If Milestone 2 is weak, the rest of the roadmap inherits the wrong runtime:

- hot reload becomes ad hoc file patching instead of canonical artifact swap
- shell work depends on mutable builder state or source folklore instead of
  artifact meaning
- bindings, tokens, commands, and Query-facing surfaces resolve through local
  helper logic instead of one lowered contract
- execution plans have to rediscover structure, legality, and identity on the
  hot path
- inspection and diagnostics become reverse engineering instead of artifact
  observation

This milestone therefore exists to close the single most important follow-on
boundary after capability registration: source and composition input must
become one canonical runtime artifact before any later milestone broadens the
platform.

## Governing Summaries

- `MENTALITY.md`: the spec must solve the hostile scale and authority problem
  first, not ship a cute source format and retrofit canonical lowering later.
- `arch_laws.md`: Milestone 2 must be a proof-bearing phase chain where each
  boundary produces a stronger artifact than the last; mutable builder state,
  parsed source, resolved source, and canonical artifact cannot collapse into
  one bag.
- `composition_laws.md`: source, parsing, lowering, legality, identity,
  artifact assembly, and inspection must each have named responsibility rather
  than hiding inside one giant lowering function or god module.
- `domain_structure_laws.md`: source input, frozen capability authority,
  canonical artifact authority, and derived diagnostics/inspection/provenance
  must remain structurally distinct so the tree teaches what is authoritative
  and what is derived.
- `perf_laws.md`: legality, capability resolution, identity, and equivalence
  must be decided before the hot path; no later runtime lane should have to
  rediscover broad structural facts, rescan registries, or resolve by strings.
- `worth_ui_roadmap.md`: Milestone 2 is the canonical source/lowering/artifact
  milestone that must land before hot reload, execution plans, shell growth,
  and component breadth.
- `worth-ui-vision.md`: file-authored UI and Rust-authored composition must
  converge on one canonical artifact, and that artifact is the runtime-owned
  meaning later plan swap and execution consume.
- `milestone-1.md` and `milestone-1-closeout.md`: later Worth UI work must
  start from frozen capability snapshots rather than mutable registries or app-
  local Rust control flow.
- `crates/worth-query/docs/AI_README.md`: serious downstream runtimes should
  consume existing runtime-owned categories where they already exist:
  support/admission, canonical query/read plans, typed binding/resolver
  surfaces, projection consumption, async/result-state posture, recovery,
  inspection, cross-runtime explanation, and signal-facing continuation over
  the runtime's existing dependency graph and incremental invalidation lanes.
  Worth UI should not build UI-local pseudo Query layers, local status
  taxonomies, caller-owned identity/recovery wrappers, or a rival dependency/
  invalidation model above those runtime lanes; it should lower UI artifacts
  so those existing runtime surfaces remain the authority.

## Adversarial Constraint

One running Worth UI app must be able to consume a multi-file UI package with
at least 200 components/surfaces/bindings/tokens/commands referenced across
modules, lower it against a frozen `CapabilitySnapshot`, reject illegal
structure and unsupported capability posture before artifact assembly, produce
the same canonical runtime artifact and digest under replay and source-file
reordering, and retain enough identity/provenance/dependency metadata that
later hot reload can swap only valid changed meaning instead of rebuilding or
re-deciding the whole UI from mutable registries, recursive scans, or host-
local caches.

## Product Decision Lock

- File-authored UI and Rust-authored composition are different authoring lanes,
  not different semantic runtimes.
- Milestone 2 lowering starts from a frozen `CapabilitySnapshot`, never from
  mutable registries or builder-owned collections.
- Where Query already owns a serious runtime lane for view binding,
  projection consumption, async/result-state posture, recovery, inspection,
  explanation, signal compatibility, continuation, dependency/invalidation
  posture, and related support/admission contracts, Worth UI must carry typed
  references into that lane instead of recreating local UI-owned pseudo
  runtimes, flattened status wrappers, or independent invalidation graphs.
- Source parsing, capability resolution, structural legality, identity seeding,
  and canonical artifact assembly are separate boundaries with separate proof
  obligations.
- The canonical runtime artifact is the authoritative UI meaning; diagnostics,
  provenance, and inspection are derived views over that meaning.
- Structural legality is part of lowering authority, not a renderer concern and
  not a later hot-reload heuristic.
- Stable identity seeds start in this milestone so later reload reconciliation
  does not depend on guesswork.
- The artifact must already be runtime-shaped enough that Milestone 4 can lower
  execution plans without re-resolving string names or re-scanning broad
  topology.
- Incremental dependency metadata belongs in Milestone 2 because future reload
  narrowing depends on canonical dependency truth, not on file watcher folklore.

## Implementation Recommendations

These recommendations are not spec-level escape hatches. They are the current
preferred implementation direction for choices the milestone does not need to
leave architecturally open.

- `WorthUiSourceModuleId` should come from canonicalized workspace-relative
  module identity, but that identity should remain a typed internal/runtime
  value rather than a raw path string propagated through later phases.
- The parse layer should preserve enough CST/span fidelity for diagnostics and
  provenance, while semantic lowering should consume a normalized parsed model
  rather than raw syntax structure.
- `WorthUiArtifactInput` should be authoring-neutral and more semantic than raw
  syntax, but it should remain pre-snapshot, pre-resolution, and pre-legality.
- Capability references should resolve into typed snapshot-backed handles as
  early as possible; free-form strings should survive past the resolution
  boundary only where diagnostics or provenance specifically need them.
- Query-facing UI surfaces should prefer existing Query-owned typed artifacts
  and support posture over UI-local shadow models; if a runtime lane already
  has canonical identity, readiness, inspection, recovery, or projection
  semantics, Worth UI should bind to that lane rather than restating it.
- Where the runtime already owns dependency-graph truth, invalidation
  narrowing, continuation posture, or recomputation boundaries, Worth UI
  should carry typed references to that truth rather than deriving a second
  watcher-local or widget-local dependency model.
- Stable identity seeding should prefer explicit authored IDs. Structural
  fallback identity should be deterministic and based on canonical parent/slot/
  role structure rather than source file order or builder registration order.
- `WorthUiArtifactDigest` should cover canonical semantic artifact meaning, not
  diagnostics richness, source formatting variation, incidental provenance
  ordering, or other derived observation detail.
- Incremental dependency metadata should track both source-package/module
  dependencies and canonical artifact/subtree dependency facts so Milestone 3
  can narrow reload honestly without reconstructing either view.
- Rust-authored composition parity should be proved at the canonical artifact
  and digest layer, not only through visual/demo parity or behavior-level
  plausibility.

## Phase Plan

### Phase 1: Source Package And Module Identity Boundary

Freeze the repo-authored source package boundary so Worth UI can reason about
multi-file UI input as one canonical package instead of a pile of unrelated
text files.

**Relevant subsystems**

- source package loading
- module identity
- import/include graph
- package-level canonical ordering

**Relevant APIs**

- `WorthUiSourcePackage`
- `WorthUiSourceModuleId`
- `WorthUiSourceImport`
- `WorthUiSourcePackageDigest`

**Warnings**

- Do not treat UI source as implicitly single-file just because the first demo
  app is small.
- Do not let filesystem path text become the long-term semantic module
  identity.
- Do not allow ambiguous import resolution or duplicate module ownership.

**Test requirements**

- `equivalent_module_graphs_produce_equivalent_package_identity`: equivalent
  source packages with equivalent module graphs produce equivalent package
  identity and canonical ordering.
- `cyclic_source_module_import_rejected_before_parsing_progression`: cyclic
  imports fail before later lowering phases begin.
- `duplicate_module_identity_rejected`: two files cannot silently claim the
  same canonical source-module identity.

**Engineering decisions**

- Multi-file package identity is a first-class artifact, not a future reload
  optimization.
- Canonical module ordering belongs here so later digests and equivalence do
  not depend on filesystem traversal accidents.
- Canonical module identity should come from normalized workspace-relative
  module identity, but later phases should consume it as a typed module ID
  rather than a raw path string.
- Source package loading remains authoring input authority only; it does not
  resolve capabilities or structural legality yet.

**Open questions**

- None.

### Phase 2: Parsed Source Boundary

Freeze the syntax-only parse layer so Worth UI can hold a structured
representation of source text without prematurely mixing in capability
resolution, legality, or runtime artifact meaning.

**Relevant subsystems**

- source parser
- parse diagnostics
- source spans
- syntax-only module model

**Relevant APIs**

- `WorthUiParsedSourcePackage`
- `WorthUiParsedSourceModule`
- `WorthUiSourceSpan`
- `WorthUiParseDiagnostic`

**Warnings**

- Do not let parsing resolve capability IDs or perform snapshot-bound semantic
  checks.
- Do not erase source-span fidelity just because later phases also produce
  diagnostics.
- Do not collapse parse failure and semantic lowering failure into one generic
  error family.

**Test requirements**

- `equivalent_source_text_produces_equivalent_parsed_structure`: equivalent
  source text with equivalent formatting-insensitive structure produces
  equivalent parsed module structure.
- `malformed_source_localizes_parse_diagnostics_to_source_spans`: malformed
  input fails with source-localized parse diagnostics rather than generic
  package failure.
- `parse_replay_is_deterministic`: replaying the same source package yields the
  same parsed structure and diagnostic ordering.

**Engineering decisions**

- Parsed source is a proof-bearing step in its own right, not a private helper
  inside semantic lowering.
- The parse layer should preserve enough CST/span fidelity for diagnostics and
  provenance even if later semantic phases consume a normalized parsed model.
- Parse diagnostics are derived observation artifacts over source text, not the
  canonical runtime artifact.
- Source spans established here must survive into later provenance surfaces.

**Open questions**

- None.

### Phase 3: Shared Artifact-Input IR Boundary

Freeze the shared artifact-input IR so file-authored UI and Rust-authored
composition converge before semantic lowering rather than carrying separate
meaning pipelines deeper into the runtime.

**Relevant subsystems**

- source-to-IR lowering
- Rust composition input lowering
- authoring-neutral IR model
- IR normalization

**Relevant APIs**

- `WorthUiArtifactInput`
- `WorthUiArtifactInputNode`
- `WorthUiArtifactInputModule`
- `WorthUiArtifactInputNormalizer`

**Warnings**

- Do not let file-authored source and Rust-authored composition lower directly
  into separate semantic pipelines.
- Do not make the IR a thin alias for text syntax on one side and Rust builder
  calls on the other.
- Do not attach frozen capability meaning to the IR yet; that belongs to the
  next phase.

**Test requirements**

- `equivalent_file_and_rust_authoring_produce_equivalent_artifact_input`:
  file-authored and Rust-authored composition that express the same UI meaning
  produce equivalent authoring-neutral IR.
- `artifact_input_normalization_is_canonical`: equivalent authoring variation
  normalizes into one canonical IR ordering and structure.
- `authoring_specific_escape_hatch_does_not_bypass_ir_boundary`: Rust-authored
  composition cannot bypass the shared IR and inject later artifact state
  directly.

**Engineering decisions**

- The shared IR is the semantic convergence boundary for authoring, not the
  final runtime artifact.
- Rust composition is an authoring escape hatch, not a second runtime lane.
- The IR should be authoring-neutral and more semantic than raw syntax, while
  remaining pre-snapshot, pre-resolution, and pre-legality.
- IR normalization belongs here so later semantic phases work from one
  canonical authoring-neutral form.

**Open questions**

- None.

### Phase 4: Snapshot-Bound Resolution Entry Boundary

Freeze the semantic-lowering entry boundary so all capability and support
resolution starts from a frozen `CapabilitySnapshot` instead of mutable builder
registries or ad hoc runtime lookups.

**Relevant subsystems**

- artifact-input resolution
- capability snapshot lookup
- support posture narrowing
- semantic-resolution diagnostics

**Relevant APIs**

- `CapabilitySnapshot`
- `WorthUiResolvedArtifactInput`
- `WorthUiResolutionDiagnostic`
- typed capability handle/reference surfaces for components, surfaces,
  commands, tokens, bindings, and related families
- Query `Support And Admission`
- Query `Basis Capability Lifecycle`
- Query `Typed Binding And Retained Artifact Reuse`

**Warnings**

- Do not reopen Milestone 1 authority by reading mutable registries during
  lowering.
- Do not treat visible capability vocabulary as admitted support without the
  snapshot's support posture.
- Do not keep unresolved string identity alive past this boundary when a typed
  resolved handle should exist.
- Do not use this phase to invent UI-local facades for runtime concepts Query
  already owns, such as support/admission posture, typed binding identity, or
  later inspection/recovery surfaces.

**Test requirements**

- `same_artifact_input_and_same_snapshot_produce_equivalent_resolution`:
  identical artifact input lowered against the same snapshot produces the same
  resolved capability graph.
- `missing_or_deferred_capability_rejected_at_resolution_boundary`: missing,
  deferred, unsupported, or platform-internal capability references fail at the
  resolution boundary rather than later artifact assembly.
- `resolution_does_not_scan_broad_registry_families_for_direct_lookup`: direct
  named capability resolution is structurally index-backed and does not depend
  on broad family scans.

**Engineering decisions**

- The frozen capability snapshot is the sole capability authority at
  lowering-time.
- Support posture is part of semantic resolution, not a later UX detail.
- This phase turns caller-owned textual references into runtime-owned typed
  references wherever semantic resolution is admitted.
- Query-facing references should resolve toward existing Query-owned identity
  and support lanes where available; Worth UI resolution is a consumer of that
  runtime meaning, not a second author of it.
- For Query-backed lanes, this phase should resolve toward the existing Query
  `Support And Admission`, `Basis Capability Lifecycle`, and `Typed Binding And
  Retained Artifact Reuse` surfaces instead of inventing Worth-UI-local
  readiness or binding-identity facades.
- Capability references should resolve into typed snapshot-backed handles as
  early as possible; free-form strings should survive this boundary only where
  diagnostics or provenance specifically need them.

**Open questions**

- None.

### Phase 5: Structural Legality And Mosaic Semantics Boundary

Freeze structural legality as its own lowering boundary so artifact shape,
region nesting, placement meaning, sizing law, and scroll ownership are proven
before canonical artifact assembly.

**Relevant subsystems**

- mosaic structure legality
- region nesting legality
- placement/sizing legality
- scroll and growth ownership

**Relevant APIs**

- `WorthUiStructuralLegalityReport`
- `WorthUiLegallyStructuredArtifactInput`
- typed mosaic legality/support artifacts over region, placement, sizing, and
  state-slot references

**Warnings**

- Do not treat mosaic legality as a renderer concern or a later shell cleanup.
- Do not flatten structural law into component-local booleans or host-local
  postvalidation.
- Do not allow illegal structure to leak into artifact assembly and fail there
  indirectly.

**Test requirements**

- `equivalent_legal_mosaic_structures_produce_equivalent_legality_artifacts`:
  equivalent legal structural compositions produce equivalent legality reports
  and lowered structural facts.
- `illegal_region_or_scroll_or_sizing_shape_rejected_before_artifact_assembly`:
  illegal region containment, scroll ownership, overlay/pinned misuse, or
  growth/sizing misuse fails before canonical artifact assembly.
- `structural_legality_does_not_depend_on_component_runtime_execution`: the
  legality lane proves structure from declared artifact meaning, not from
  component execution or renderer behavior.

**Engineering decisions**

- Structural legality is part of authoritative lowering.
- Mosaic meaning remains the platform's structural language, not a thin label
  over arbitrary nested widget trees.
- This phase should produce explicit typed facts later phases can consume rather
  than repeating raw legality predicates.

**Open questions**

- None.

### Phase 6: Binding And Capability Semantics Boundary

Freeze the semantic binding lane so command placement, view bindings, theme
token references, plugin slots, native-capability references, and similar
capability semantics are resolved and validated separately from structural
legality.

**Relevant subsystems**

- command binding resolution
- view-binding resolution
- theme/icon/native/plugin reference legality
- semantic binding diagnostics

**Relevant APIs**

- `WorthUiBoundArtifactInput`
- `WorthUiBindingDiagnostic`
- typed bound capability reference handles for commands, projections,
  view-bindings, tokens, icons, plugin slots, and native capabilities
- Query `Live Views And Live Promotion`
- Query `Projection Consumption`
- Query `Async Resources And Result State`
- Query `Recovery`
- Query `Inspection`

**Warnings**

- Do not mix semantic binding legality with structural mosaic legality in one
  branch-heavy phase.
- Do not let bindings silently downgrade to inert placeholders when support
  posture or capability shape rejects them.
- Do not re-derive Query-bound or runtime-outcome posture through local UI
  helper logic once the registry/snapshot already named it.
- Do not create Worth-UI-owned query plans, result-state taxonomies, recovery
  models, or projection-fact wrappers when Query already exposes a stronger
  runtime-backed public lane for that concept.

**Test requirements**

- `equivalent_binding_inputs_produce_equivalent_bound_semantics`: equivalent
  bound artifact input yields equivalent command/view/token/native/plugin
  binding semantics.
- `binding_family_mismatch_localizes_to_binding_boundary`: mismatched command,
  binding, token, plugin, or native-capability references fail at this boundary
  instead of surfacing later as artifact corruption.
- `query_bound_view_reference_preserves_query_owned_posture`: Query-bound
  binding semantics carry Query-owned support/authority posture instead of
  collapsing into UI-local booleans or strings.

**Engineering decisions**

- Structural legality and capability-binding legality are separate proof
  families and therefore separate phases.
- Bound semantics should preserve upstream runtime posture rather than
  flattening it.
- Query-bound surfaces in this phase should bind to existing Query-owned
  `Live Views And Live Promotion`, `Projection Consumption`, `Async Resources
  And Result State`, `Recovery`, and `Inspection` artifacts where those lanes
  already exist, rather than restating them in UI-local semantic wrappers.
- The output of this phase is still pre-artifact meaning, but it is now fully
  bound and semantically typed.

**Open questions**

- None.

### Phase 7: Stable Identity Seeding Boundary

Freeze the identity-seeding rules for artifact nodes so later reload,
reconciliation, inspection, and persistence do not depend on heuristic identity
guessing.

**Relevant subsystems**

- explicit authored identity
- structural fallback identity
- identity replacement semantics
- durable-state eligibility classification

**Relevant APIs**

- `WorthUiArtifactIdentitySeed`
- `WorthUiIdentitySeededArtifactInput`
- `WorthUiIdentityReplacementClass`
- `WorthUiDurableStateEligibility`

**Warnings**

- Do not defer identity definition until hot reload needs it.
- Do not derive identity from incidental file positions or registry iteration
  order.
- Do not let every node implicitly claim durable identity just because later
  runtime state might attach to it.

**Test requirements**

- `same_authored_identity_and_same_structure_produce_same_identity_seed`: the
  same authored/stable structure produces the same identity seed under replay.
- `meaningful_identity_change_is_classified_as_replacement`: a real identity
  change is classified as replacement rather than accidental carry-forward.
- `identity_seed_is_not_file_order_folklore`: source file reorderings that do
  not change canonical meaning do not change identity seeds.

**Engineering decisions**

- Identity seeding begins in Milestone 2 because Milestone 3 needs real seeds,
  not best-effort reconstruction.
- Explicit IDs dominate structural fallback IDs where both exist.
- Structural fallback identity should be deterministic and based on canonical
  parent/slot/role structure rather than source file order or builder
  registration order.
- Durable-state eligibility must be explicit so later reconciliation can
  preserve only what the artifact model actually admits.

**Open questions**

- None.

### Phase 8: Canonical Artifact Assembly Boundary

Freeze the authoritative runtime artifact assembly boundary so all prior
proof-bearing lowering work becomes one canonical runtime-owned artifact with
typed handles and normalized structure.

**Relevant subsystems**

- artifact node assembly
- normalized child ordering
- typed runtime handles
- artifact-level diagnostics attachment

**Relevant APIs**

- `WorthUiArtifact`
- `WorthUiArtifactNode`
- `WorthUiArtifactHandle`
- `WorthUiArtifactAssemblyReport`

**Warnings**

- Do not assemble an editor-friendly tree that later runtime phases must
  reinterpret into the real artifact.
- Do not preserve caller-owned strings where a typed handle now exists.
- Do not mix provenance/inspection derivation into the artifact authority path
  itself.

**Test requirements**

- `equivalent_seeded_bound_input_produces_equivalent_canonical_artifact`:
  equivalent fully lowered input produces equivalent canonical artifacts.
- `artifact_assembly_consumes_proven_inputs_only`: artifact assembly cannot run
  on unresolved, structurally illegal, or unseeded input.
- `canonical_artifact_normalizes_child_ordering`: legal authoring variation
  that claims equivalent meaning produces one canonical artifact ordering.

**Engineering decisions**

- The canonical runtime artifact is the authoritative UI meaning boundary for
  later runtime lanes.
- Artifact assembly consumes already-proven inputs instead of re-deciding
  legality, support, or identity.
- Handles should already be runtime-shaped enough that later execution-plan
  lowering does not re-resolve names.

**Open questions**

- None.

### Phase 9: Artifact Provenance And Inspection Boundary

Freeze the derived artifact-observation lane so later tooling, diagnostics, and
runtime explanation can inspect source/capability/artifact relationships
without reopening source text or Rust control flow.

**Relevant subsystems**

- source-to-artifact provenance
- capability-reference provenance
- artifact inspection
- derived observation reports

**Relevant APIs**

- `WorthUiArtifactProvenanceMap`
- `WorthUiArtifactInspection`
- `WorthUiArtifactNodeInspection`
- `WorthUiArtifactCapabilityReferenceInspection`
- Query `Inspection`
- Query `Cross-Runtime Causal Inspection`
- Query `Projection Consumption`

**Warnings**

- Do not force tooling to reconstruct provenance from raw artifact nodes.
- Do not let provenance become authoritative artifact meaning.
- Do not make inspection depend on mutable source package or builder state
  remaining live.
- Do not rebuild a second explanation or runtime-archaeology lane in Worth UI
  where Query inspection, projection-consumption receipts, async/result-state
  surfaces, or cross-runtime causal explanation already own the concept.

**Test requirements**

- `artifact_inspection_explains_source_and_capability_origin`: inspection can
  name which source spans and registered capabilities produced representative
  artifact nodes.
- `provenance_replay_is_deterministic`: replaying the same lowering input
  produces the same provenance and inspection relationships.
- `inspection_does_not_require_rust_control_flow_archaeology`: equivalent
  file-authored and Rust-authored composition can be inspected through the same
  derived artifact lane.

**Engineering decisions**

- Provenance and inspection are derived views over canonical artifact meaning.
- The observation lane must be self-describing enough for future tooling and
  certification to depend on it.
- Worth UI inspection should compose with, point at, or embed existing
  Query `Inspection`, `Cross-Runtime Causal Inspection`, and `Projection
  Consumption` artifacts where applicable instead of inventing rival
  explanation surfaces for the same runtime facts.
- Source spans proven in parsing and handles proven in lowering must survive
  into this boundary.

**Open questions**

- None.

### Phase 10: Artifact Digest And Equivalence Boundary

Freeze the artifact digest and equivalence contract so reuse, parity, reload
comparison, and certification all depend on declared sameness semantics instead
of heuristic tree comparison.

**Relevant subsystems**

- artifact digesting
- canonical equivalence rules
- replay parity
- artifact sameness contracts

**Relevant APIs**

- `WorthUiArtifactDigest`
- `WorthUiArtifactEquivalence`
- `WorthUiArtifactEquivalenceBasis`
- `WorthUiArtifactDigestReport`

**Warnings**

- Do not treat pointer identity, parse order, or source file ordering as
  canonical artifact identity.
- Do not define reuse surfaces without explicit sameness semantics.
- Do not let richer diagnostics or provenance change canonical digest meaning.

**Test requirements**

- `same_artifact_meaning_produces_same_digest`: equivalent artifact meaning
  produces the same digest under replay and legal authoring variation.
- `meaningful_artifact_difference_changes_digest`: intentionally different
  artifact meaning produces a mechanically different digest or equivalence
  classification.
- `diagnostic_richness_does_not_change_artifact_digest`: richer derived
  observation output does not change canonical artifact identity.

**Engineering decisions**

- Digest and equivalence are separate from artifact assembly because sameness is
  its own contract.
- Canonical artifact reuse must always state its identity basis explicitly.
- The canonical digest should cover semantic artifact meaning only, not source
  formatting variation, diagnostics richness, incidental provenance ordering,
  or other derived observation detail.
- Later reload and certification should consume this contract, not invent their
  own comparison logic.

**Open questions**

- None.

### Phase 11: Incremental Dependency Metadata Boundary

Freeze the dependency metadata that future hot reload and tooling will consume
so change narrowing is based on canonical package/artifact relationships rather
than file watcher folklore.

**Relevant subsystems**

- module dependency graph
- subtree dependency metadata
- artifact impact metadata
- incremental narrowing facts

**Relevant APIs**

- `WorthUiArtifactDependencyGraph`
- `WorthUiArtifactSubtreeDigest`
- `WorthUiArtifactImpactMetadata`
- `WorthUiIncrementalInvalidationBasis`
- Query `Signal Compatibility And Continuation`
- Query `Region-Scoped Live Invalidation And Stream Contracts`
- Query `Live Views And Live Promotion`
- Query `Async Resources And Result State`

**Warnings**

- Do not postpone dependency truth until Milestone 3 and then rediscover it
  from changed files.
- Do not let every change imply full-package or full-artifact rebuild by
  default when narrower truth is already knowable.
- Do not treat filesystem diffs, UI-tree containment, or widget adjacency as
  the authoritative invalidation graph when the runtime already owns stronger
  incremental invalidation structure.
- Do not confuse this metadata with actual reload orchestration; this phase
  names dependency truth only.

**Test requirements**

- `equivalent_artifacts_produce_equivalent_dependency_metadata`: equivalent
  canonical artifacts produce equivalent dependency/impact metadata.
- `dependency_narrowing_does_not_require_full_tree_scan`: representative impact
  lookup for changed modules/subtrees is structurally narrower than full-tree
  rediscovery.
- `dependency_metadata_changes_when_meaningful_upstream_relationships_change`:
  import/containment/dependency changes that matter to reload semantics change
  the dependency metadata explicitly.
- `dependency_metadata_preserves_runtime_graph_hooks`: artifact dependency
  metadata retains the typed relationship handles or basis needed for later
  reload/invalidation lanes to compose with runtime-owned graph truth instead
  of re-deriving a second invalidation model.

**Engineering decisions**

- Incremental dependency truth belongs in Milestone 2 because later reload
  narrowing is a consumer of this artifact, not its author.
- Dependency metadata is derived from canonical source/artifact structure, not
  from the external file watcher.
- Dependency metadata should track both source-package/module relationships and
  canonical artifact/subtree dependency facts so later reload narrowing does
  not have to reconstruct either view.
- When a UI artifact binds into runtime-owned Query/signal surfaces, the
  dependency metadata should preserve typed linkage to the runtime's existing
  invalidation/evaluation basis rather than collapsing those links into
  anonymous UI-local edges.
- The specific runtime lanes that matter here are Query `Signal Compatibility
  And Continuation`, `Region-Scoped Live Invalidation And Stream Contracts`,
  `Live Views And Live Promotion`, and `Async Resources And Result State`; the
  reload layer should consume their typed invalidation/evaluation posture
  rather than inferring a second graph from files or widgets.
- Worth UI owns artifact dependency truth for its own source/artifact
  structure, but it should compose with runtime-owned dependency truth where
  UI meaning already depends on Query/signal graph semantics.
- This phase closes the proof boundary that a later incremental lane may rely
  on without broad rediscovery.

**Open questions**

- None.

### Phase 12: Rust Composition Parity Boundary

Freeze the proof that Rust-authored composition is a real authoring escape
hatch over the same source/IR/lowering/artifact model, not a privileged bypass
around it.

**Relevant subsystems**

- Rust composition authoring
- shared IR consumption
- parity comparison
- escape-hatch containment

**Relevant APIs**

- `WorthUiRustCompositionInput`
- `WorthUiArtifactInput`
- `WorthUiArtifact`
- parity/certification helpers over file-authored versus Rust-authored lanes

**Warnings**

- Do not let Rust composition acquire special access to mutable registries,
  canonical artifact constructors, or post-lowering mutation seams.
- Do not settle for broad visual parity when canonical artifact parity is the
  real contract.
- Do not let the escape hatch become the easier lane for serious platform work.

**Test requirements**

- `rust_and_file_authored_equivalent_ui_produce_equivalent_canonical_artifacts`:
  equivalent Rust-authored and file-authored composition yield equivalent
  canonical artifacts and digests.
- `rust_composition_cannot_bypass_snapshot_bound_resolution`: Rust-authored
  input still fails on missing/deferred/unsupported snapshot capability posture
  through the same semantic lane.
- `rust_escape_hatch_remains_authoring_lane_only`: compile-time or structural
  proof shows Rust composition cannot skip the shared IR and canonical lowering
  pipeline.

**Engineering decisions**

- Rust composition is a first-class authoring lane with second-class privilege.
- Parity must be stated at the canonical artifact and digest layer, not in demo
  behavior terms or visual plausibility terms alone.
- This phase closes the architectural promise that later hot reload may consume
  replaceable Rust-authored artifact input when admitted.

**Open questions**

- None.

### Phase 13: Sample App And Hostile Certification Boundary

Close the milestone with one realistic sample app and hostile certification that
proves Worth UI now owns canonical source-to-artifact lowering honestly under
scale, replay, structural rejection, and parity pressure.

**Relevant subsystems**

- multi-file sample app
- hostile lowering certification
- parity/replay certification
- scale/cost counters for lowering-time boundaries where needed

**Relevant APIs**

- sample app source package and Rust composition fixtures
- canonical artifact digests and inspection/provenance surfaces
- certification bundles over source package, snapshot, artifact, digest, and
  diagnostic outputs

**Warnings**

- Do not close the milestone on a single tiny happy-path example.
- Do not certify only visual plausibility; certify canonical artifacts,
  diagnostics, provenance, and equivalence.
- Do not leave multi-file, unsupported-capability, or identity-stability cases
  to future milestones.

**Test requirements**

- `multi_file_source_package_lowers_to_one_canonical_artifact`: a realistic
  multi-file sample app lowers through the full source-to-artifact pipeline
  successfully.
- `hostile_source_rejection_localizes_before_artifact_authority`: malformed
  source, missing support, illegal structure, and binding mismatch all fail at
  the narrowest expected boundary with typed diagnostics.
- `replay_and_reordering_preserve_canonical_artifact_parity`: source replay and
  legal module-order variation preserve canonical artifact/digest parity.
- `file_and_rust_authoring_sample_paths_compare_equal_where_claimed`: the same
  sample app authored through file and Rust lanes compares equal on canonical
  artifact meaning where parity is claimed.

**Engineering decisions**

- Certification must prove canonical artifact ownership, not just parser
  plausibility.
- The sample app should be realistic enough to exercise package identity,
  bindings, mosaic legality, provenance, and artifact inspection.
- Later milestones may build on these certification bundles instead of
  rebuilding Milestone 2 truth from scratch.

**Open questions**

- None.

## Must Ship

- a repo-authored source package model with canonical module identity, import
  resolution, and package digest posture
- a syntax-only parsed source boundary with source spans and parse diagnostics
- one shared artifact-input IR that both file-authored and Rust-authored
  composition must pass through
- snapshot-bound capability resolution over frozen `CapabilitySnapshot`
- separate structural-legality and semantic-binding lowering boundaries
- stable identity seeding for artifact nodes that later reload and persistence
  lanes may consume
- one canonical runtime artifact with typed handles and normalized structure
- derived provenance and inspection surfaces over the canonical artifact
- explicit artifact digest and equivalence contracts
- incremental dependency metadata suitable for later reload narrowing
- Rust-authored composition parity proof against the same lowering pipeline
- one realistic sample app and hostile certification program

## Must Preserve

- Milestone 2 must not reopen mutable registration authority closed by
  Milestone 1
- source parsing, semantic lowering, artifact authority, and derived inspection
  must remain structurally separate
- file-authored source and Rust-authored composition must differ only in
  authoring lane, not in canonical lowering semantics
- the canonical runtime artifact must remain the source of UI meaning after
  lowering completes
- legality, support posture, and identity must be decided before later runtime
  hot paths consume artifact meaning
- later runtime lanes must not need to rediscover capability meaning from
  strings, registry scans, or source text
- Query-bound and runtime-outcome-facing UI semantics must preserve upstream
  typed posture rather than inventing local pseudo-runtime wrappers

## Acceptance Evidence

- equivalent source packages and snapshots lower to equivalent canonical
  artifacts where sameness is claimed
- intentionally different source, structure, support posture, or identity basis
  yields mechanically different artifact or typed rejection where expected
- malformed source, unsupported capability posture, structural illegality, and
  binding mismatch fail at the narrowest appropriate phase boundary
- canonical artifact provenance can explain which source and capability meaning
  produced representative runtime nodes
- Rust-authored and file-authored composition compare equal on canonical
  artifact meaning where parity is claimed
- incremental dependency metadata and artifact digests are deterministic enough
  to support later reload narrowing without rebuild folklore

## Sequencing Notes

This milestone belongs immediately after Milestone 1 because frozen capability
snapshots are now the real lowerable authority, and the platform must convert
authoring input into canonical runtime artifact truth before hot reload,
execution-plan specialization, shell behavior, or component breadth expands the
surface area.

It belongs before Milestone 3 because hot reload should consume already-closed
artifact identity, legality, provenance, and dependency truth rather than
inventing them while implementing plan swap.

It belongs before Milestone 4 because execution plans should lower from a real
canonical artifact with typed handles and equivalence contracts, not from raw
source trees, mutable registries, or renderer-local rediscovery logic.
