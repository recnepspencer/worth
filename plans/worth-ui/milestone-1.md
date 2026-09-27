# Milestone 1: Platform Skeleton, Facade, and Capability Registries

## Goal

Make Worth UI enter the codebase as one domain-agnostic platform subsystem with
a narrow public facade, typed capability identities, domain-agnostic registry
families, mosaic structural vocabulary, and proof-bearing registration
snapshots that later source lowering can consume without rediscovering meaning
from strings or app-local code.

## Why This Milestone Exists

Worth UI cannot safely build hot reload, canonical artifacts, mosaic shell
layout, Query-bound surfaces, plugins, or execution plans until compiled Rust
capabilities are registered through a single platform-owned facade. This
milestone creates the vocabulary and mechanical boundary that later milestones
will lower against.

## Governing Summaries

- `MENTALITY.md`: protects architecture from MVP drift by requiring adversarial
  constraints, mechanical enforcement, one canonical artifact, facade-only
  access, and proof-bearing tests before feature breadth.
- `arch_laws.md`: protects authority and lifecycle boundaries by requiring
  typed contracts, proof-carrying phase transitions, structured envelopes,
  facade enforcement, and compiler-visible propagation when subsystems grow.
- `composition_laws.md`: protects local code shape from god files and unnamed
  steps; the spec must map to named modules, narrow files, semantic phases, and
  tests that are organized by responsibility.
- `domain_structure_laws.md`: protects the tree from becoming storage; Worth UI
  structure must encode authority, lifecycle, dependency direction,
  source-truth/projection separation, and facade-only public access.
- `perf_laws.md`: protects future hot paths by requiring upstream resolution,
  bounded lookup, explicit equivalence, named measurement boundaries, and no
  repeated rediscovery during lowering or execution.
- `worth_ui_roadmap.md`: protects Worth UI from becoming a widget bundle by
  requiring platform-owned capability registration, canonical artifacts,
  hot-lowered composition, mosaic layout structure, Query/runtime boundaries,
  and performance certification.
- `crates/worth-query/docs/AI_README.md`: protects the Query boundary by
  requiring domain work to start from Query's public facade, preserving Query
  artifacts and support posture instead of inventing local pseudo-Query layers
  in UI.

## Adversarial Constraint

A downstream app must be able to register at least 100 commands, 100
components, 50 surfaces, 50 mosaic structural contracts, 50 settings, 50 theme
tokens, 50 icons, 25 Query-view bindings, 25 runtime outcome projections, 25
task presentation families, 25 plugin slots, and 10 native adapter capability
descriptors through the Worth UI facade; freeze those registrations into an
immutable snapshot; and let later lowering validate references using only typed
IDs and snapshot indexes, with deterministic diagnostics for duplicates,
unsupported families, illegal mosaic placement, raw magic layout numbers, and
facade bypass attempts.

## Product Decision Lock

- Worth UI is domain-agnostic, but not structure-agnostic.
- Mosaic is the structural space-allocation language for shell and page
  composition.
- Built-ins are platform primitives and contracts, not product-domain nouns.
- Registries describe admitted capability; later artifacts compose admitted
  capability.
- Raw layout numbers are invalid public artifact input unless introduced by a
  named token, sizing contract, or measurement definition.
- Query-backed UI registration references Query-owned meaning; it does not
  create a UI-owned query runtime.

## Registration Surface Rule

Every registry family must expose the same public registration shape:

- a descriptor type that names the capability being contributed
- a facade builder method that accepts the descriptor
- a typed family registry owned behind the facade
- a frozen snapshot family that later lowering can inspect
- structured diagnostics for rejection, support posture, and cross-family
  references

Registry phases must therefore specify not only what is registered, but also
which facade method admits it and which frozen snapshot family later consumes
it. The builder is the only public registration surface. Internal registries
are implementation detail.

## Phase Plan

Every phase must name its hostile tests as implementation targets. Test names
below are required semantic cases; implementation may adjust exact file names
to match local test topology, but it must preserve the responsibility each name
describes.

### Required Implementation Surfaces By Phase

These are the intended code and proof surfaces for the milestone. Exact module
names may adjust during implementation, but changes must preserve the same
responsibility boundaries and facade direction.

- Phase 1 uses `workspaces/worth-ui/crates/worth-ui/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui/src/facade/mod.rs`, and facade compile tests.
- Phase 2 uses `workspaces/worth-ui/crates/worth-ui/src/facade/builder.rs`,
  `workspaces/worth-ui/crates/worth-ui/src/facade/app.rs`,
  `workspaces/worth-ui/crates/worth-ui/src/capability/snapshot/mod.rs`, and builder lifecycle
  tests.
- Phase 3 uses `workspaces/worth-ui/crates/worth-ui/src/capability/identity/mod.rs` and ID
  validation compile-fail tests.
- Phase 4 uses `workspaces/worth-ui/crates/worth-ui/src/capability/support/mod.rs` and support
  posture admission tests.
- Phase 5 uses `workspaces/worth-ui/crates/worth-ui/src/capability/diagnostics/mod.rs` and
  deterministic diagnostic replay tests.
- Phase 6 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/command.rs` and command
  registry tests.
- Phase 7 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/component.rs` and
  component descriptor tests.
- Phase 8 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/surface.rs` and
  domain-agnostic surface tests.
- Phase 9 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/mosaic_region.rs` and
  mosaic region law tests.
- Phase 10 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/mosaic_placement.rs`
  and mosaic placement law tests.
- Phase 11 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/mosaic_sizing.rs` and
  magic-number denial tests.
- Phase 12 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/mosaic_state.rs` and
  UI-state ownership tests.
- Phase 13 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/view_binding.rs` and
  Query-boundary tests.
- Phase 14 uses
  `workspaces/worth-ui/crates/worth-ui/src/capability/registry/runtime_outcome_projection.rs` and
  runtime posture projection tests.
- Phase 15 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/settings.rs` and typed
  settings tests.
- Phase 16 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/task_presentation.rs`
  and task presentation boundary tests.
- Phase 17 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/theme_token.rs` and
  semantic token tests.
- Phase 18 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/icon.rs` and icon
  identity tests.
- Phase 19 uses
  `workspaces/worth-ui/crates/worth-ui/src/capability/registry/command_projection.rs` and command
  projection tests.
- Phase 20 uses `workspaces/worth-ui/crates/worth-ui/src/capability/registry/plugin_slot.rs` and
  plugin contribution-slot tests.
- Phase 21 uses
  `workspaces/worth-ui/crates/worth-ui/src/capability/registry/native_capability.rs` and native
  capability posture tests.
- Phase 22 uses `workspaces/worth-ui/crates/worth-ui/src/capability/snapshot/freeze.rs`,
  `workspaces/worth-ui/crates/worth-ui/src/capability/snapshot/index.rs`, and snapshot determinism
  and cost tests.
- Phase 23 uses `workspaces/worth-ui/crates/worth-ui/tests/ui/facade_visibility` and compile-fail
  fixtures for private constructors, proof fields, and snapshot internals.
- Phase 24 uses `workspaces/worth-ui/crates/worth-ui/tests/integration/minimal_structural_app.rs`
  or an equivalent example-backed integration test.
- Phase 25 uses `workspaces/worth-ui/crates/worth-ui/src/capability/family_inventory.rs`,
  snapshot/facade construction sites, and registry-extension propagation tests.

### Phase 1: Facade Crate Entry

Create the single public entrypoint downstream applications import when they
build a Worth UI app.

**Relevant subsystems**

- Worth UI public facade
- crate export topology
- implementation visibility boundary

**Relevant APIs**

- `worth_ui`
- `WorthUi`
- `WorthUiAppBuilder`
- facade re-export module

**Warnings**

- Do not mirror internal module topology through public exports.
- Do not let convenience re-exports become alternate construction paths.

**Test requirements**

- `facade_only_empty_app_compiles`: a downstream crate builds a minimal empty
  Worth UI app using only the facade imports.
- `internal_registry_module_import_fails`: a compile-fail test proves internal
  registry modules cannot be imported by downstream code.
- `facade_reexport_does_not_expose_internal_topology`: public exports do not
  mirror implementation module names or storage types.

**Engineering decisions**

- Public entry must be narrower than internal topology.
- Internal modules default to private or `pub(crate)`.
- Facade files aggregate public contracts and do not implement registry logic.

**Open questions**

- None.

### Phase 2: App Builder Lifecycle

Introduce the builder lifecycle that collects registrations and freezes them
into a platform-owned capability snapshot.

**Relevant subsystems**

- facade builder
- registration lifecycle
- snapshot lifecycle

**Relevant APIs**

- `WorthUiAppBuilder`
- `WorthUiApp`
- `RegisteredCapabilitySet`
- `CapabilitySnapshot`

**Warnings**

- Do not allow hidden global registration.
- Do not allow mutation after snapshot freeze.

**Test requirements**

- `equivalent_builder_inputs_freeze_to_equivalent_snapshots`: two builders
  receiving identical registrations produce equivalent immutable snapshots.
- `register_after_snapshot_freeze_fails`: registering after freeze fails
  through type state or an explicit structured rejection.
- `hidden_global_registration_does_not_affect_snapshot`: a separately created
  builder cannot observe registrations from another builder.

**Engineering decisions**

- The builder is mutable registration authority.
- The snapshot is immutable lowering input.
- The built app owns the snapshot; later phases may derive artifacts from it.

**Open questions**

- Whether freeze should consume the builder or produce a sealed frozen handle
  is an implementation decision.

### Phase 3: Capability Identity Types

Add stable, typed identifiers for every registry family so future UI source
cannot use arbitrary strings as semantic authority.

**Relevant subsystems**

- capability identity
- registry keys
- diagnostics references

**Relevant APIs**

- `CommandId`
- `ComponentId`
- `SurfaceId`
- `MosaicRegionKindId`
- `MosaicPlacementPolicyId`
- `MosaicSizingContractId`
- `MosaicStateSlotId`
- `ViewBindingId`
- `RuntimeOutcomeProjectionId`
- `SettingId`
- `TaskPresentationId`
- `ThemeTokenId`
- `IconId`
- `CommandProjectionId`
- `PluginSlotId`
- `NativeCapabilityId`

**Warnings**

- Do not use one generic ID type for distinct semantic families.
- Do not expose constructors that bypass validation and canonicalization.

**Test requirements**

- `same_text_different_id_families_are_not_interchangeable`: structurally
  identical textual IDs in different families are not interchangeable at
  compile time.
- `invalid_id_text_rejected_before_descriptor_construction`: invalid ID text is
  rejected before any registry entry can be constructed.
- `validated_id_constructor_is_not_publicly_mintable`: compile-fail coverage
  proves callers cannot bypass ID validation by constructing proof-bearing
  fields directly.

**Engineering decisions**

- IDs are semantic wrappers with family-specific types.
- IDs must be stable enough for hot reload, diagnostics, and persisted shell
  state later.
- IDs should support deterministic ordering inside snapshots.

**Open questions**

- Exact textual grammar can be finalized during implementation, but it must be
  shared by all ID validators.

### Phase 4: Support Posture

Represent whether a capability family or entry is admitted, deferred,
unsupported, or platform-internal so public vocabulary never implies runtime
support by accident.

**Relevant subsystems**

- support classification
- capability descriptors
- diagnostics

**Relevant APIs**

- `CapabilitySupportPosture`
- `AdmittedCapability`
- `DeferredCapability`
- `UnsupportedCapability`

**Warnings**

- Do not teach support from autocomplete.
- Do not allow deferred entries to lower as if admitted.

**Test requirements**

- `equivalent_admitted_entries_preserve_support_posture`: equivalent admitted
  entries produce equivalent support posture in snapshots.
- `deferred_entry_reference_rejected_as_not_admitted`: a deferred entry
  referenced by a sample validation request produces a structured rejection.
- `unsupported_entry_reference_rejected_without_fallback`: unsupported entries
  never silently downgrade to deferred or admitted posture.
- `admitted_posture_witness_is_not_publicly_mintable`: compile-fail coverage
  proves admitted posture cannot be WORTHd by downstream callers.

**Engineering decisions**

- Posture belongs in Milestone 1 because the facade is a support contract.
- Public vocabulary may exist before a family is fully admitted, but posture
  must remain visible and machine-checkable.

**Open questions**

- Whether posture is family-wide, entry-specific, or both should be settled by
  the descriptor shape in implementation.

### Phase 5: Registration Diagnostics

Create structured diagnostics for registration and snapshot construction
without flattening failures into strings.

**Relevant subsystems**

- diagnostics
- registration validation
- snapshot freeze

**Relevant APIs**

- `CapabilityRegistrationDiagnostic`
- `CapabilityRegistrationReport`
- `CapabilityDiagnosticSeverity`
- `CapabilityDiagnosticCode`

**Warnings**

- Do not couple rich diagnostics to successful registration.
- Do not let diagnostic richness change the accepted snapshot.

**Test requirements**

- `invalid_registration_replay_produces_identical_diagnostics`: replaying the
  same invalid registration sequence produces identical diagnostic codes and
  ordering.
- `shuffled_invalid_registration_diagnostics_are_canonical`: shuffled invalid
  inputs produce canonical diagnostic ordering.
- `diagnostic_codes_distinguish_failure_topology`: duplicate IDs, unsupported
  posture references, missing dependencies, and family mismatches each produce
  distinct typed diagnostics.
- `diagnostics_do_not_change_accepted_snapshot_digest`: rich diagnostics and
  minimal diagnostics produce the same accepted capability meaning.

**Engineering decisions**

- Diagnostics are observation artifacts, not authority.
- Snapshot freeze may return a report even when no errors occurred.
- Diagnostic payloads should reference typed IDs and family names.

**Open questions**

- None.

### Phase 6: Command Registry

Register domain-agnostic application actions as typed command capabilities.

**Relevant subsystems**

- command spine seed
- icon references
- command projection eligibility
- runtime posture references

**Relevant APIs**

- `CommandDescriptor`
- `WorthUiAppBuilder::register_command`
- `CommandRegistry`
- `FrozenCommandCapabilities`
- `CommandCategory`
- `CommandReadinessBinding`
- `worth_query::facade::WORTHQueryDeclarationEntryReadinessReport`
- `worth_query::facade::WORTHQueryDeclarationEntryReadinessStatus`

**Warnings**

- Do not implement command routing or execution in this milestone.
- Do not flatten readiness or admission posture into booleans.

**Test requirements**

- `equivalent_command_descriptors_produce_equivalent_indexes`: registering the
  same command descriptor sequence produces equivalent command snapshot
  indexes.
- `duplicate_command_id_rejected_before_snapshot_freeze`: duplicate command IDs
  are rejected before snapshot freeze succeeds.
- `command_projection_references_unknown_projection_surface_rejected`: invalid
  projection eligibility is rejected before command descriptors become
  admitted.
- `command_readiness_cannot_be_flattened_to_bool`: compile-fail or descriptor
  validation proves readiness must use structured posture.

**Engineering decisions**

- Built-in command fields are ID, label, description, icon reference, default
  shortcut reference, category, readiness binding shape, and projection
  eligibility.
- Runtime intent binding is only a typed placeholder until later milestones.
- Commands remain domain actions supplied by apps, not product-domain built-ins.

**Open questions**

- Shortcut conflict resolution belongs to a later command milestone.

### Phase 7: Component Registry

Register compiled Rust renderable capabilities that hot-lowered UI may compose
later.

**Relevant subsystems**

- component descriptors
- prop schemas
- accessibility metadata
- execution lane hints

**Relevant APIs**

- `ComponentDescriptor`
- `WorthUiAppBuilder::register_component`
- `ComponentRegistry`
- `FrozenComponentCapabilities`
- `ComponentPropSchema`
- `ComponentChildPolicy`
- `ComponentStateOwnership`

**Warnings**

- Do not let components secretly own shell layout.
- Do not allow custom components to bypass accessibility and focus metadata.

**Test requirements**

- `equivalent_component_descriptors_produce_equivalent_entries`: components
  with identical descriptors and IDs produce equivalent snapshot entries across
  canonical registration order.
- `component_with_untyped_props_rejected`: a component without a typed prop
  schema is rejected.
- `component_missing_state_ownership_rejected`: a component without state
  ownership classification is rejected.
- `component_with_illegal_child_policy_rejected`: illegal child policy fails
  before snapshot freeze.
- `component_references_missing_theme_token_rejected`: token dependency checks
  catch cross-registry drift.

**Engineering decisions**

- Built-in descriptor fields include ID, prop schema, child policy, state
  ownership class, accessibility support, focus support, token dependencies,
  command binding slots, and execution lane hint.
- Component registration admits capability; it does not instantiate UI nodes.
- App and plugin components must enter through the same descriptor path.

**Open questions**

- Exact prop schema encoding can be chosen during implementation.

### Phase 8: Surface Registry

Register product-meaning surfaces without encoding product-domain nouns into
the platform.

**Relevant subsystems**

- surface descriptors
- mosaic placement classes
- state preservation classes
- Query binding references

**Relevant APIs**

- `SurfaceDescriptor`
- `WorthUiAppBuilder::register_surface`
- `SurfaceRegistry`
- `FrozenSurfaceCapabilities`
- `SurfaceKind`
- `SurfacePlacementClass`
- `SurfaceStateClass`

**Warnings**

- Do not bake in concepts like project explorer, problems panel, document
  editor, dashboard, or AI panel.
- Do not collapse surfaces and components; surfaces have shell meaning.

**Test requirements**

- `equivalent_app_defined_surfaces_produce_equivalent_entries`: equivalent
  app-defined surfaces lower into equivalent surface snapshot entries
  independent of descriptor construction path.
- `surface_references_missing_component_rejected`: a surface cannot reference
  an unknown component.
- `surface_claims_unsupported_placement_class_rejected`: unsupported placement
  claims fail with typed diagnostics.
- `surface_uses_invalid_state_class_rejected`: invalid state class fails before
  snapshot freeze.
- `platform_builtin_surface_domain_name_rejected`: built-in surface kinds
  cannot encode product-domain names such as project explorer, document editor,
  or dashboard.

**Engineering decisions**

- Built-in surface kinds stay structural: primary content, auxiliary content,
  transient content, modal content, overlay content, status content, settings
  content, diagnostics content.
- App-domain names are registered by applications as surface IDs and labels.
- Surfaces reference components, commands, placement classes, and optional view
  binding posture.

**Open questions**

- Whether `diagnostics content` is a surface kind or runtime projection family
  can be refined, but it must remain domain-agnostic.

### Phase 9: Mosaic Region Registry

Register legal structural region kinds for mosaic-owned space allocation.

**Relevant subsystems**

- mosaic structure
- shell layout vocabulary
- focus scopes
- scroll ownership

**Relevant APIs**

- `MosaicRegionKindDescriptor`
- `WorthUiAppBuilder::register_mosaic_region_kind`
- `MosaicRegionRegistry`
- `FrozenMosaicRegionCapabilities`
- `MosaicRegionRole`
- `MosaicScrollOwnership`
- `MosaicFocusScopeKind`

**Warnings**

- Do not treat mosaic as a generic flexbox/grid replacement.
- Do not encode product-domain meanings into built-in region kinds.

**Test requirements**

- `equivalent_mosaic_region_descriptors_produce_equivalent_entries`:
  equivalent region descriptors produce equivalent region snapshot entries and
  stable ordering.
- `mosaic_region_missing_sizing_behavior_rejected`: region kinds must declare
  sizing behavior.
- `mosaic_region_missing_scroll_ownership_rejected`: region kinds must declare
  scroll ownership.
- `mosaic_region_missing_focus_scope_rejected`: region kinds must declare
  focus scope.
- `platform_builtin_region_domain_name_rejected`: built-in region roles cannot
  encode product-domain names such as file browser, property editor, or issue
  list.

**Engineering decisions**

- Built-in region roles are primary, auxiliary, side, bottom, status, toolbar,
  stack, split, overlay, modal, floating, and viewport.
- Each region declares sizing behavior, scroll ownership, child rules, allowed
  surface classes, persistence posture, focus scope, clipping, and hit-test
  posture.
- Concrete layout instances wait for Milestone 2.

**Open questions**

- The exact names of built-in roles can change, but they must remain structural
  rather than domain-specific.

### Phase 10: Mosaic Placement Registry

Register legal movement and containment policies for surfaces and regions.

**Relevant subsystems**

- mosaic placement law
- surface eligibility
- reload reconciliation seed

**Relevant APIs**

- `MosaicPlacementPolicyDescriptor`
- `WorthUiAppBuilder::register_mosaic_placement_policy`
- `MosaicPlacementRegistry`
- `FrozenMosaicPlacementCapabilities`
- `MosaicPlacementAction`
- `MosaicPlacementEligibility`

**Warnings**

- Do not let plugins or apps imperatively rearrange shell state behind the
  runtime.
- Do not accept placement policies that lack source and target families.

**Test requirements**

- `equivalent_mosaic_placement_policies_produce_equivalent_legality_tables`:
  equivalent placement policies produce equivalent placement legality tables in
  the snapshot.
- `illegal_surface_to_region_placement_rejected`: invalid source/target
  combinations fail during registration validation.
- `cyclic_mosaic_containment_policy_rejected`: placement policies cannot admit
  cycles.
- `unsupported_float_or_overlay_policy_rejected`: unsupported floating or
  overlay policies are rejected with typed diagnostics.
- `plugin_cannot_imperatively_rearrange_mosaic_state`: extension paths can
  contribute placement permissions, not mutate runtime shell state directly.

**Engineering decisions**

- Built-in placement actions are dock, tab, split, pin, collapse, overlay,
  float, modal, status projection, and toolbar projection.
- Placement policies declare valid source surface classes, valid target region
  roles, persistence behavior, stable identity behavior, conflict behavior, and
  reload reconciliation posture.
- Placement capability is separate from concrete placement state.

**Open questions**

- None.

### Phase 11: Mosaic Sizing Registry

Register named sizing contracts so layout numbers enter the platform only
through typed, inspectable measurement definitions.

**Relevant subsystems**

- mosaic sizing
- token references
- layout diagnostics

**Relevant APIs**

- `MosaicSizingContractDescriptor`
- `WorthUiAppBuilder::register_mosaic_sizing_contract`
- `MosaicSizingRegistry`
- `FrozenMosaicSizingCapabilities`
- `MosaicSizingKind`
- `NamedMeasurementToken`

**Warnings**

- Do not accept raw width, height, gap, padding, z-order, timing, or breakpoint
  numbers as public layout input.
- Do not reintroduce CSS-style implicit overflow or percentage-height folklore.

**Test requirements**

- `equivalent_named_sizing_contracts_produce_equivalent_entries`: equivalent
  named sizing contracts produce equivalent sizing snapshot entries and
  diagnostics references.
- `raw_width_value_rejected_outside_named_measurement`: raw width values are
  invalid public artifact input.
- `raw_gap_value_rejected_outside_named_measurement`: raw gap values are
  invalid public artifact input.
- `raw_z_order_value_rejected_outside_named_measurement`: raw z-order values
  are invalid public artifact input.
- `raw_timing_value_rejected_outside_named_measurement`: raw timing values are
  invalid public artifact input.
- `raw_breakpoint_value_rejected_outside_named_measurement`: raw breakpoint
  values are invalid public artifact input.
- `unitless_measurement_definition_rejected`: numeric values inside named
  definitions must still carry unit and constraint metadata.
- `sizing_contract_without_overflow_policy_rejected`: sizing contracts must
  declare overflow behavior explicitly.

**Engineering decisions**

- Built-in sizing kinds are fixed, fill, ratio, bounded, hug, min/max,
  grow-then-scroll, content-measured, persisted-user-size, and explicitly
  admitted viewport-relative.
- Each sizing contract declares measurement authority, resize permission,
  persistence, overflow behavior, parent-growth behavior, and constrained
  viewport behavior.
- Numeric values are allowed only inside named measurement definitions,
  semantic tokens, or platform-owned low-level internals.

**Open questions**

- Exact unit taxonomy can be finalized during implementation.

### Phase 12: Mosaic State Slot Registry

Register durable UI state classes that may survive hot reload and restore
without becoming authoritative domain truth.

**Relevant subsystems**

- stable identity
- reload reconciliation seed
- persisted UI state

**Relevant APIs**

- `MosaicStateSlotDescriptor`
- `WorthUiAppBuilder::register_mosaic_state_slot`
- `MosaicStateSlotRegistry`
- `FrozenMosaicStateCapabilities`
- `MosaicStateSlotKind`
- `MosaicStatePersistencePolicy`

**Warnings**

- Do not let UI state masquerade as Query or relational truth.
- Do not preserve state without stable owner identity.

**Test requirements**

- `equivalent_state_slots_produce_equivalent_reconciliation_keys`:
  re-registering equivalent state slots produces equivalent snapshot entries
  and stable reconciliation keys.
- `state_slot_without_owner_identity_rejected`: durable state slots must declare
  stable owner identity.
- `state_slot_without_persistence_posture_rejected`: persistence posture is
  required before state can be preserved.
- `state_slot_without_replacement_rules_rejected`: reload replacement behavior
  must be explicit.
- `ui_state_slot_cannot_claim_authoritative_truth`: UI state slot descriptors
  cannot masquerade as Query or relational truth.

**Engineering decisions**

- Built-in state slot kinds are splitter position, active stack item, region
  visibility, collapsed posture, pinned posture, scroll position, focused
  region, active primary surface, active auxiliary surface, selection token,
  and draft input state.
- State slots describe preservation capability only; concrete state storage and
  reconciliation arrive in later milestones.
- Persisted UI state remains structurally separate from authoritative runtime
  truth.

**Open questions**

- Selection token scope may need refinement when Query-bound surfaces arrive.

### Phase 13: View Binding Registry

Register UI-facing seams for Query-owned view meaning without creating a
UI-owned query layer.

**Relevant subsystems**

- Query public workspace facade references
- Query support matrix and admission posture
- Query view shape and result shape references
- Query basis capability posture
- surface descriptors

**Relevant APIs**

- `ViewBindingDescriptor`
- `WorthUiAppBuilder::register_view_binding`
- `ViewBindingRegistry`
- `FrozenViewBindingCapabilities`
- `ViewBindingFamily`
- `worth_query::facade::WORTHQueryWorkspace`
- `worth_query::facade::WORTHQueryRuntimePublicSupportMatrix`
- `worth_query::facade::WORTHQueryRuntimeFacadeFamily`
- `worth_query::facade::WORTHQueryRuntimePublicApiFamilyContract`
- `worth_query::facade::WORTHQuerySupportReport`
- `worth_query::facade::WORTHQuerySupportMatrix`
- `worth_query::facade::WORTHQueryCapabilityDescriptor`
- `worth_query::facade::WORTHQueryCapabilityRegistry`
- `worth_query::facade::WORTHQueryCapabilityStatus`
- `worth_query::facade::QueryCompositionSupportProfile`
- `worth_query::facade::runtime_backed_query_composition_support_profile`
- `worth_query::facade::QueryFamily`
- `worth_query::facade::ResultShapeFamily`
- `worth_query::facade::AuthoredQueryBundleRequest`
- `worth_query::facade::CanonicalQueryBundle`
- `worth_query::facade::CanonicalQueryArtifact`
- `worth_query::facade::CanonicalResultShapeArtifact`
- `worth_query::facade::ValidatedQueryBundle`
- `worth_query::facade::ValidatedResultShapeArtifact`
- `worth_query::facade::ViewShapeDescriptor`
- `worth_query::facade::ViewShapeFamily`
- `worth_query::facade::ViewShapeDigest`
- `worth_query::facade::AdmittedViewShape`
- `worth_query::facade::runtime_backed_view_shape_support_profile`
- `worth_query::facade::BasisLifecycleSupportMatrix`
- `worth_query::facade::BasisLifecycleSupportRow`
- `worth_query::facade::BasisFamily`
- `worth_query::facade::BasisSupportPosture`
- `worth_query::facade::BasisOperationLane`
- `worth_query::facade::RawBasisIntent`
- `worth_query::facade::NormalizedBasisIntent`
- `worth_query::facade::AdmittedBasisCapability`
- `worth_query::facade::DeniedBasisCapability`
- `worth_query::facade::QuerySubscriptionSupportMatrix`
- `worth_query::facade::QuerySubscriptionSupportPosture`
- `worth_query::facade::QuerySubscriptionFamily`

**Warnings**

- Do not build local pseudo-Query surfaces.
- Do not claim support because a Query-shaped descriptor exists.
- Do not register lower bridge, relational, signal, branch, snapshot, or
  subscription internals through Worth UI.
- Do not execute Query, open live views, inspect retained handles, or consume
  projection facts in Milestone 1.

**Test requirements**

- `equivalent_query_view_references_produce_equivalent_bindings`: equivalent
  Query-owned view capability references produce equivalent UI view binding
  snapshot entries.
- `view_binding_without_query_support_posture_rejected`: Query support posture
  is required before a view binding can be admitted.
- `view_binding_without_basis_posture_rejected`: basis posture is required so
  UI does not infer Query authority.
- `view_binding_without_result_shape_rejected`: result shape metadata is
  required for later surface validation.
- `admitted_query_view_binding_witness_is_not_publicly_mintable`: compile-fail
  coverage proves UI cannot WORTH admitted Query support.
- `local_pseudo_query_binding_rejected`: UI-owned query/cache descriptors are
  rejected instead of admitted as Query view bindings.

**Engineering decisions**

- Built-in binding families are collection, detail, grouped, relationship,
  ordered-event, spatial, and custom-admitted.
- Each binding records Query-owned handle shape, result shape, basis/support
  posture, live compatibility, visible-state binding declarations, and
  denial/advisory presentation shape.
- Worth UI registers presentation capability; Query owns legality, planning,
  basis, live semantics, and support admission.
- Registration accepts only Query public facade surfaces: workspace/public
  support surfaces, Query capability/support descriptors, authored/canonical/
  validated query and result-shape artifacts, view-shape descriptors and
  admitted view-shape posture, basis lifecycle support/admission posture, and
  subscription support posture.
- Later milestones may execute, inspect, subscribe, or consume projected facts
  only through Query's admitted facade surfaces.

**Open questions**

- Exact Query facade types should be chosen only when implementation reaches
  the Query integration seam.

### Phase 14: Runtime Outcome Projection Registry

Register presentation projections for structured runtime posture without
inventing local UI status enums.

**Relevant subsystems**

- runtime outcome presentation
- command readiness display
- surface posture display

**Relevant APIs**

- `RuntimeOutcomeProjectionDescriptor`
- `WorthUiAppBuilder::register_runtime_outcome_projection`
- `RuntimeOutcomeProjectionRegistry`
- `FrozenRuntimeOutcomeProjectionCapabilities`
- `RuntimeOutcomeFamily`
- `worth_query::facade::WORTHQueryOrdinaryOutcome`
- `worth_query::facade::WORTHQueryOrdinaryPosture`
- `worth_query::facade::WORTHQueryOrdinaryPostureKind`
- `worth_query::facade::WORTHQueryOrdinaryRuntimePosture`
- `worth_query::facade::WORTHQueryOrdinaryRuntimePostureKind`
- `worth_query::facade::WORTHQueryRuntimeAsyncResultState`
- `worth_query::facade::WORTHQueryRuntimeAsyncResultStateKind`

**Warnings**

- Do not flatten runtime outcomes into loading/success/failure booleans.
- Do not allow presentation customization to alter runtime meaning.

**Test requirements**

- `equivalent_outcome_projections_preserve_family_identity`: equivalent outcome
  projection descriptors produce equivalent snapshot entries and retain the
  same outcome family identity.
- `unknown_runtime_outcome_family_rejected`: unknown outcome families are
  rejected.
- `local_status_enum_projection_rejected`: local UI status enums cannot replace
  structured runtime posture.
- `outcome_projection_missing_denial_posture_rejected`: denial posture must be
  represented where the family admits denial.
- `outcome_projection_missing_recovery_posture_rejected`: recovery posture must
  be represented where the family admits recovery.
- `outcome_projection_does_not_change_runtime_meaning`: changing labels, icons,
  or tones does not change outcome identity in the snapshot.

**Engineering decisions**

- Built-in outcome families are loading, ready, denied, advisory, violation,
  stopped, recoverable, stale, failed, completed, cancelled, retrying, and
  revalidating where admitted by the backing runtime.
- Apps may project labels, icons, tones, and affordance hints onto structured
  outcomes.
- Outcome meaning remains runtime-owned.

**Open questions**

- Some outcome families may be deferred until Query/runtime support admits
  them; support posture must make that visible.

### Phase 15: Settings Registry

Register typed settings capability without creating untyped app-local config
bags.

**Relevant subsystems**

- settings descriptors
- scope ownership
- future settings surfaces

**Relevant APIs**

- `SettingDescriptor`
- `WorthUiAppBuilder::register_setting`
- `SettingsRegistry`
- `FrozenSettingCapabilities`
- `SettingScope`
- `SettingValueSchema`

**Warnings**

- Do not accept arbitrary key/value maps as platform settings.
- Do not make settings persistence authoritative domain truth.

**Test requirements**

- `equivalent_setting_descriptors_produce_equivalent_defaults`: equivalent
  setting descriptors produce equivalent snapshot entries and default values.
- `setting_without_scope_rejected`: settings must declare scope.
- `setting_without_value_schema_rejected`: settings must declare value type.
- `setting_without_default_posture_rejected`: settings must declare default
  posture.
- `setting_without_validation_posture_rejected`: settings must declare
  validation posture.
- `arbitrary_key_value_setting_bag_rejected`: untyped config bags cannot enter
  the platform settings registry.

**Engineering decisions**

- Built-in scopes are user, workspace, project, app, plugin, theme, density,
  keyboard, and accessibility.
- Each setting declares ID, scope, value type, default, validation, migration
  posture, UI editor hint, and ownership metadata.
- Settings panels are later composition over settings metadata, not Milestone 1
  behavior.

**Open questions**

- Migration artifact shape belongs to the persistence milestone.

### Phase 16: Task Presentation Registry

Register domain-agnostic task presentation families for future background work
UX without owning task execution semantics.

**Relevant subsystems**

- task presentation descriptors
- status projection
- diagnostics references

**Relevant APIs**

- `TaskPresentationDescriptor`
- `WorthUiAppBuilder::register_task_presentation`
- `TaskPresentationRegistry`
- `FrozenTaskPresentationCapabilities`
- `TaskPresentationFamily`

**Warnings**

- Do not build a task runtime in Worth UI.
- Do not couple task presentation to authoritative domain truth.

**Test requirements**

- `equivalent_task_presentations_produce_equivalent_projection_eligibility`:
  equivalent task presentation descriptors produce equivalent snapshot entries
  and projection eligibility.
- `task_presentation_without_lifecycle_posture_rejected`: lifecycle posture is
  required.
- `task_presentation_without_cancellation_posture_rejected`: cancellation
  posture must be explicit.
- `task_presentation_without_failure_posture_rejected`: failure posture must be
  explicit.
- `task_presentation_cannot_claim_task_runtime_authority`: descriptors cannot
  own execution semantics.

**Engineering decisions**

- Built-in presentation families are progress, cancellable, retryable,
  blocking, background, completed, failed, and status-projected.
- Task execution remains owned by the runtime or application authority that
  produces structured task state.
- Worth UI owns how admitted task posture can be surfaced.

**Open questions**

- Whether task posture should reuse runtime outcome projection descriptors can
  be refined when implementation reaches shared descriptor design.

### Phase 17: Theme Token Registry

Register semantic presentation tokens so visual values are named, inspectable,
and themeable.

**Relevant subsystems**

- design tokens
- component token dependencies
- runtime outcome tones

**Relevant APIs**

- `ThemeTokenDescriptor`
- `WorthUiAppBuilder::register_theme_token`
- `ThemeTokenRegistry`
- `FrozenThemeTokenCapabilities`
- `ThemeTokenFamily`
- `ThemeTokenAlias`

**Warnings**

- Do not expose raw colors as component-facing contracts.
- Do not let plugins silently redefine platform token meaning.

**Test requirements**

- `equivalent_token_graphs_resolve_to_equivalent_entries`: equivalent token
  graphs with the same aliases resolve to equivalent snapshot token entries.
- `missing_theme_token_dependency_rejected`: missing token dependencies are
  rejected.
- `theme_token_alias_cycle_rejected`: alias cycles fail before snapshot freeze.
- `raw_color_outside_token_definition_rejected`: raw color usage outside token
  definitions is rejected.
- `plugin_cannot_silently_override_platform_token_meaning`: plugin token
  contributions must be explicit aliases or admitted custom tokens.

**Engineering decisions**

- Built-in token families are surface, elevated surface, text, muted text,
  border, accent, selection, focus, danger, warning, success, advisory,
  disabled, overlay, shadow, chart series, and runtime state.
- Components depend on semantic token IDs, not raw visual values.
- Theme values are definitions behind semantic tokens.

**Open questions**

- Full theme modes belong to the design system milestone.

### Phase 18: Icon Registry

Register stable icon capabilities that commands, surfaces, outcomes, and
projection entries can reference without asset-path coupling.

**Relevant subsystems**

- icon descriptors
- command descriptors
- surface descriptors
- token references

**Relevant APIs**

- `IconDescriptor`
- `WorthUiAppBuilder::register_icon`
- `IconRegistry`
- `FrozenIconCapabilities`
- `IconFamily`
- `IconSourceDescriptor`

**Warnings**

- Do not let file paths become public semantic icon IDs.
- Do not make icon packs mutate command or surface descriptors directly.

**Test requirements**

- `equivalent_icon_descriptors_produce_equivalent_entries`: equivalent icon
  descriptors with equivalent sources produce equivalent snapshot entries.
- `unknown_icon_reference_rejected`: command, surface, or outcome descriptors
  cannot reference unknown icons.
- `unsupported_icon_source_kind_rejected`: unsupported icon sources fail with
  typed diagnostics.
- `theme_incompatible_icon_descriptor_rejected`: icons that cannot satisfy
  declared theme posture are rejected.
- `icon_asset_path_cannot_stand_in_for_icon_id`: raw asset paths cannot replace
  stable icon IDs in public descriptors.

**Engineering decisions**

- Built-in icon families are command, surface, status, runtime outcome,
  navigation, toolbar, and custom-admitted.
- Icon descriptors declare stable ID, source/provider metadata, size support,
  fill/stroke support, theme posture, and accessibility posture.
- Registries reference icon IDs, not raw assets.

**Open questions**

- Asset packaging belongs to native integration and delivery milestones.

### Phase 19: Command Projection Registry

Register the domain-agnostic places where commands may appear without letting
apps imperatively build menus and toolbars.

**Relevant subsystems**

- command projection descriptors
- command registry
- mosaic placement references

**Relevant APIs**

- `CommandProjectionDescriptor`
- `WorthUiAppBuilder::register_command_projection`
- `CommandProjectionRegistry`
- `FrozenCommandProjectionCapabilities`
- `CommandProjectionSurface`

**Warnings**

- Do not let projection entries define new command meaning.
- Do not duplicate command readiness logic inside projection descriptors.

**Test requirements**

- `equivalent_command_projections_produce_equivalent_entries`: equivalent
  projection descriptors over the same command IDs produce equivalent
  projection snapshot entries.
- `projection_references_unknown_command_rejected`: projection entries cannot
  reference unknown commands.
- `projection_references_unsupported_surface_rejected`: unsupported projection
  surfaces fail with typed diagnostics.
- `projection_with_conflicting_required_grouping_rejected`: required grouping
  conflicts are rejected.
- `projection_cannot_define_new_command_meaning`: projection descriptors cannot
  introduce labels, readiness, or handlers that diverge from command identity.

**Engineering decisions**

- Built-in projection surfaces are menu bar, command palette, toolbar, context
  menu, region header action, status action, tab action, and auxiliary action.
- Projection descriptors declare eligible command categories,
  ordering/grouping, shortcut visibility, readiness display policy,
  icon/label policy, and overflow behavior.
- Projections are command-spine views, not separate action systems.

**Open questions**

- Shortcut conflict resolution belongs to the command spine milestone.

### Phase 20: Plugin Slot Registry

Register typed extension contribution slots while keeping extension power
runtime-owned and capability-bounded.

**Relevant subsystems**

- plugin contribution descriptors
- permission posture
- registry extension paths

**Relevant APIs**

- `PluginSlotDescriptor`
- `WorthUiAppBuilder::register_plugin_slot`
- `PluginSlotRegistry`
- `FrozenPluginSlotCapabilities`
- `PluginContributionFamily`
- `PluginCapabilityPermission`

**Warnings**

- Do not implement plugin loading in this milestone.
- Do not let plugin slots become arbitrary global UI mutation hooks.

**Test requirements**

- `equivalent_plugin_slots_produce_equivalent_admitted_families`: equivalent
  plugin slot descriptors produce equivalent snapshot entries and admitted
  contribution families.
- `plugin_contribution_to_unknown_slot_rejected`: contributions to unknown
  slots are rejected.
- `plugin_contribution_to_unsupported_family_rejected`: unsupported
  contribution families are rejected.
- `plugin_contribution_without_permission_rejected`: missing permission posture
  rejects contribution.
- `plugin_slot_cannot_be_arbitrary_global_mutation_hook`: plugin slots must
  name admitted capability families instead of opaque callbacks.

**Engineering decisions**

- Built-in contribution families are command, component, surface, setting,
  view binding, theme token, icon, command projection, task presentation,
  runtime outcome projection, and native capability request.
- Each slot declares allowed families, permission posture, ordering posture,
  diagnostics posture, and support posture.
- Plugins add registered capabilities; they do not bypass registries.

**Open questions**

- Full plugin isolation and loading belong to the plugin architecture
  milestone.

### Phase 21: Native Capability Registry

Register native adapter capabilities as explicit platform seams without
implementing operating-system adapters.

**Relevant subsystems**

- native adapter descriptors
- support posture
- shell integration seed

**Relevant APIs**

- `NativeCapabilityDescriptor`
- `WorthUiAppBuilder::register_native_capability`
- `NativeCapabilityRegistry`
- `FrozenNativeCapabilities`
- `NativeCapabilityFamily`
- `NativePlatformPosture`

**Warnings**

- Do not hide platform-specific support behind ambient host checks.
- Do not let native capabilities redefine shell or runtime semantics.

**Test requirements**

- `equivalent_native_capabilities_produce_equivalent_support_entries`:
  equivalent native capability descriptors produce equivalent platform support
  entries.
- `unsupported_native_family_rejected`: unsupported platform families are
  rejected.
- `native_capability_missing_platform_posture_rejected`: platform posture is
  required.
- `native_adapter_claims_shell_authority_rejected`: native adapter descriptors
  cannot redefine shell semantics.
- `ambient_host_check_cannot_replace_native_capability_posture`: platform
  support must be registered explicitly.

**Engineering decisions**

- Built-in capability families are native menu adapter, file dialog,
  clipboard, drag/drop, notification, tray, URL/file association, OS theme, and
  keychain.
- Milestone 1 registers vocabulary and support posture only.
- Native adapters remain explicit lower boundaries, not app-local folklore.

**Open questions**

- Exact OS adapter crates belong to the native integration milestone.

### Phase 22: Capability Snapshot

Freeze all registered families into an immutable, indexed, deterministic
snapshot that later lowering can consume.

**Relevant subsystems**

- snapshot construction
- registry indexes
- diagnostics summary
- future lowering input

**Relevant APIs**

- `CapabilitySnapshot`
- `CapabilitySnapshotBuilder`
- `FrozenCapabilityFamily`
- `CapabilitySnapshotDigest`

**Warnings**

- Do not let lowering read mutable registries.
- Do not make snapshot identity depend on registration order where canonical
  ordering is available.

**Test requirements**

- `snapshot_digest_stable_under_registration_permutation`: registering
  equivalent capabilities in different orders produces equivalent snapshot
  digests and indexes where family semantics allow canonical ordering.
- `snapshot_diagnostics_stable_under_invalid_input_permutation`: invalid inputs
  in different orders produce canonical diagnostic ordering.
- `snapshot_missing_cross_family_reference_rejected`: missing cross-family
  references are rejected or diagnosed before lowering can consume them.
- `snapshot_deferred_entry_used_as_admitted_rejected`: deferred entries cannot
  satisfy admitted references.
- `snapshot_lookup_by_typed_id_is_index_backed`: ordinary snapshot lookup by
  typed ID and family exposes counters or structural proof that it does not
  require broad scans.
- `snapshot_internal_indexes_not_publicly_mutable`: compile-fail coverage
  proves downstream code cannot mutate frozen indexes.

**Engineering decisions**

- Snapshot construction is the authority boundary between registration and
  future artifact lowering.
- Snapshot entries carry typed IDs, support posture, descriptor metadata,
  dependency references, and diagnostic summaries.
- Snapshot lookup surfaces should be index-backed and not require broad scans
  for ordinary validation.

**Open questions**

- Exact digest algorithm can be chosen during implementation.

### Phase 23: Facade Visibility Proof

Prove the public facade is the only external surface and internal topology is
not part of the public contract.

**Relevant subsystems**

- crate visibility
- compile-fail tests
- facade exports

**Relevant APIs**

- public facade module
- internal registry modules
- UI compile tests

**Warnings**

- Do not rely on documentation to stop deep imports.
- Do not expose fields merely to make tests easier.

**Test requirements**

- `internal_registry_constructor_not_publicly_accessible`: compile-fail tests
  prove internal registry constructors are private.
- `mutable_registry_storage_not_publicly_accessible`: compile-fail tests prove
  mutable registry storage is private.
- `snapshot_internal_fields_not_publicly_constructible`: compile-fail tests
  prove snapshot internals and proof-bearing fields are private.
- `validated_descriptor_fields_not_publicly_mintable`: downstream code cannot
  synthesize proof-bearing validated descriptors directly.
- `facade_only_app_remains_ergonomic`: a facade-only app compile test proves
  ordinary app construction remains ergonomic despite private internals.

**Engineering decisions**

- Fields that carry proof are private.
- Constructors for validated/proof-bearing types are sealed behind proving
  functions.
- Tests may use public fixtures or crate-local test support, not public
  internals.

**Open questions**

- None.

### Phase 24: Minimal App Proof

Build one tiny facade-only app path that registers representative capabilities
without hidden global state or internal imports.

**Relevant subsystems**

- facade builder
- representative registries
- snapshot freeze

**Relevant APIs**

- `WorthUiAppBuilder`
- registration methods
- `CapabilitySnapshot`
- minimal sample app or integration test

**Warnings**

- Do not turn the sample into a product-domain template.
- Do not skip less visible families such as support posture and diagnostics.

**Test requirements**

- `minimal_structural_app_registers_representative_capabilities`: a minimal app
  registers one command, component, surface, mosaic region, placement policy,
  sizing contract, state slot, token, icon, and plugin slot through only the
  facade and freezes successfully.
- `minimal_structural_app_duplicate_command_rejected`: the same app cannot
  register a duplicate command without a typed diagnostic.
- `minimal_structural_app_raw_layout_number_rejected`: the same app cannot
  register raw layout numbers without a typed diagnostic.
- `minimal_structural_app_illegal_mosaic_placement_rejected`: the same app
  cannot register illegal mosaic placement without a typed diagnostic.
- `minimal_structural_app_snapshot_inspection_names_registered_capabilities`:
  snapshot inspection can name registered capabilities without internal module
  access.

**Engineering decisions**

- The proof app is a structural platform sample, not a workbench template.
- It should demonstrate representative registry flow rather than feature UX.
- The app should expose enough snapshot inspection to confirm what was
  registered without reading internal modules.

**Open questions**

- Whether this lives as an example crate or integration test can be decided
  during implementation.

### Phase 25: Registry Extension Proof

Prove that adding a new registry family or lifecycle boundary forces
compiler-visible updates through construction, snapshot, diagnostics, and
facade surfaces.

**Relevant subsystems**

- registry family enum or type map
- builder lifecycle
- snapshot construction
- diagnostics aggregation

**Relevant APIs**

- registry family inventory
- builder construction sites
- snapshot freeze path
- diagnostic report aggregation

**Warnings**

- Do not let registry extension become stringly typed.
- Do not hide new families behind generic maps that compile without lifecycle
  propagation.

**Test requirements**

- `new_registry_family_requires_builder_initialization_update`: compile-time or
  targeted construction coverage proves a new family cannot be added without
  builder initialization changes.
- `new_registry_family_requires_snapshot_freeze_update`: adding a family forces
  snapshot freeze propagation.
- `new_registry_family_requires_diagnostics_aggregation_update`: adding a
  family forces diagnostics aggregation propagation.
- `new_registry_family_requires_facade_exposure_decision`: adding a family
  forces an explicit facade exposure or internal-only decision.
- `unknown_or_omitted_registry_family_reported`: runtime adversarial coverage
  proves unknown or omitted families do not silently disappear from snapshot
  reports.

**Engineering decisions**

- Registry family inventory should be exhaustive enough that growth creates
  compiler pressure.
- Generic storage may be used internally only if typed family boundaries remain
  explicit at lifecycle edges.
- This proof is the acceptance gate for Milestone 1 completeness.

**Open questions**

- Exact enforcement mechanism depends on implementation language shape, but
  convention-only enforcement is not acceptable.

## Must Ship

- One public Worth UI facade and app builder.
- Typed capability IDs for all Milestone 1 registry families.
- Support posture and structured registration diagnostics.
- Domain-agnostic registries for commands, components, surfaces, mosaic region
  kinds, mosaic placement policies, mosaic sizing contracts, mosaic state
  slots, view bindings, runtime outcome projections, settings, task
  presentations, theme tokens, icons, command projections, plugin slots, and
  native capabilities.
- Immutable capability snapshots suitable for later lowering.
- Compile-time or test-enforced visibility boundaries.
- Minimal facade-only app proof.
- Registry extension proof.

## Must Preserve

- Worth UI remains a UI platform, not a second Query, truth, signal, bridge, or
  native runtime.
- Built-ins remain domain-agnostic structural primitives and contracts.
- Mosaic owns structural space allocation; components own content; surfaces own
  product-facing placement meaning.
- Raw layout numbers are invalid public artifact input outside named token,
  sizing, or measurement definitions.
- Registry descriptors define capability, not concrete UI instances.
- Snapshot construction is deterministic and immutable.
- Diagnostics observe and explain; they do not change accepted capability
  meaning.

## Acceptance Evidence

- A downstream app builds through only the facade.
- Deep imports and proof-bearing internals fail through compile-fail tests.
- Duplicate IDs, invalid IDs, unsupported posture, missing dependencies,
  illegal mosaic placement, missing sizing overflow policy, and raw layout
  numbers fail mechanically.
- Equivalent registration sets produce equivalent snapshots.
- The snapshot can answer what commands, components, surfaces, mosaic contracts,
  bindings, tokens, icons, plugin slots, and native capabilities exist without
  reading Rust control flow.
- Adding a registry family forces lifecycle propagation through builder,
  snapshot, diagnostics, and facade surfaces.

## Hostile Test Program

Milestone 1 test coverage is not complete until these named suites exist.

### Compile-Fail Boundary Suite

- internal registry modules cannot be imported by downstream crates.
- validated IDs, admitted support witnesses, Query admission witnesses,
  validated descriptors, snapshot internals, and mutable registry storage
  cannot be publicly constructed.
- frozen snapshots cannot be mutated.

### Determinism Suite

- equivalent registrations produce equivalent snapshots.
- equivalent registrations in randomized order produce equivalent snapshot
  digests.
- invalid registrations in randomized order produce canonical diagnostic
  ordering.

### Cross-Registry Rejection Suite

- surfaces cannot reference missing components.
- components cannot reference missing tokens.
- command projections cannot reference missing commands.
- plugin contributions cannot target unknown slots or unsupported families.
- view bindings cannot claim Query support without Query-owned posture.
- native capabilities cannot replace explicit platform posture with ambient
  host checks.

### Mosaic Law Suite

- built-in mosaic roles remain structural and reject product-domain names.
- illegal surface-to-region placement is rejected.
- cyclic region containment policy is rejected.
- sizing contracts without overflow policy are rejected.
- state slots without owner identity, persistence posture, or replacement rules
  are rejected.

### Magic Number Denial Suite

- raw width, height, gap, padding, z-order, timing, and breakpoint values are
  rejected outside named measurement, sizing, or token definitions.
- unitless measurement definitions are rejected.
- numeric values accepted through named measurement definitions remain
  inspectable in the snapshot.

### Domain-Agnostic Guard Suite

- platform built-ins cannot encode product-domain nouns.
- app-domain labels are admitted only as app-registered IDs, labels, or
  descriptors.
- templates and examples may use product-domain names only outside platform
  built-in registries.

### Snapshot Cost Suite

- ordinary typed-ID snapshot lookup is index-backed or otherwise carries a
  structural proof/counter that it does not scan all families.
- snapshot inspection reports family widths and lookup counters at the
  capability boundary.

## Sequencing Notes

Milestone 1 deliberately stops before source parsing, concrete artifact
lowering, hot reload, command routing, shell rendering, Query execution,
plugin loading, native adapter implementation, and visual component breadth.
Those later milestones consume the typed capability world created here.

This milestone belongs first because every later Worth UI claim depends on a
single facade, typed capability identities, domain-agnostic registry law,
mosaic structural vocabulary, and immutable snapshots that can be validated
without app-local folklore.

