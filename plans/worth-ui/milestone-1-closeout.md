# Milestone 1 Closeout: Platform Skeleton, Facade, And Capability Registries

## Status

Milestone 1 is complete as of 2026-06-11.

This closeout records completion of:

- `plans/worth-ui/milestone-1.md`
- `workspaces/worth-ui/crates/worth-ui`

The milestone closes Worth UI as a domain-agnostic platform subsystem with one
public facade, proof-bearing capability registration, typed registry families,
immutable indexed snapshots, and enough hostile proof coverage that later hot
reload, source lowering, mosaic composition, Query-bound UI surfaces, plugins,
and native seams can consume registered capability meaning without rediscovering
it from strings or app-local folklore.

## What Closed

Milestone 1 now ships the platform skeleton required before any Worth UI source
language or renderer can honestly exist:

- one public `worth_ui::facade` import surface
- one `WorthUi` entrypoint and `WorthUiAppBuilder` registration authority
- typed capability IDs for every Milestone 1 registry family
- support posture vocabulary for admitted, deferred, unsupported, and
  platform-internal capability meaning
- structured registration diagnostics with canonical replay behavior
- domain-agnostic registries for commands, components, surfaces, mosaic region
  kinds, mosaic placement policies, mosaic sizing contracts, mosaic state
  slots, view bindings, runtime outcome projections, settings, task
  presentations, theme tokens, icons, command projections, plugin slots, and
  native capabilities
- immutable `CapabilitySnapshot` construction with deterministic digest,
  family reports, typed lookup indexes, validation reports, and lookup-cost
  counters
- compile-fail facade visibility proof for private registries, proof-bearing
  fields, mutable storage, and snapshot internals
- a minimal facade-only structural app proof
- a typed registry-family inventory that makes family growth visible at builder,
  snapshot, diagnostics, and facade decision boundaries

## Architectural Outcome

The closed architecture is intentionally a capability platform, not a widget
catalog.

Worth UI now has the substrate required by later milestones:

- facade-only construction is the public authority boundary
- descriptors define admitted capability, not concrete UI instances
- internal registries own mutable collection before freeze
- snapshot freeze is the boundary between registration authority and later
  lowering input
- snapshot indexes are derived, immutable, and inspectable without deep imports
- mosaic owns structural space-allocation vocabulary
- components own renderable content capability
- surfaces own shell placement meaning
- Query-backed UI surfaces preserve Query-owned support posture instead of
  inventing local pseudo-Query meaning
- raw layout numbers are rejected outside named measurement or sizing authority
- diagnostics explain rejection without changing accepted snapshot meaning

The important milestone decision is now encoded mechanically: later Worth UI
source lowering must start from a frozen capability snapshot, not from arbitrary
Rust control flow, mutable registries, strings, or product-domain assumptions.

## Registry Families Closed

Milestone 1 closes over these registry families:

- `command`
- `command_projection`
- `component`
- `icon`
- `mosaic_placement_policy`
- `mosaic_region_kind`
- `mosaic_sizing_contract`
- `mosaic_state_slot`
- `native_capability`
- `plugin_slot`
- `runtime_outcome_projection`
- `setting`
- `surface`
- `task_presentation`
- `theme_token`
- `view_binding`

Each family now has:

- a typed descriptor or capability contribution shape
- a facade registration method where public registration is admitted
- internal registry storage behind the facade
- registration candidate and validation participation
- accepted-registration proof filtering
- frozen snapshot entry or family storage
- digest participation
- snapshot family reporting
- typed lookup or inspection where relevant
- compile-fail privacy coverage where public construction would bypass proof

## Proof Surfaces

Milestone 1 has proof coverage in these main lanes:

- facade-only compile-pass tests
- internal topology compile-fail tests
- typed identity validation tests
- support posture and proof-constructor denial tests
- deterministic registration diagnostic replay tests
- per-family registry determinism and rejection tests
- cross-registry dependency rejection tests
- mosaic law tests for structural region, placement, sizing, and state
- magic-number denial tests for raw layout values
- Query-bound view binding tests that preserve Query support posture
- runtime outcome, task presentation, plugin slot, and native capability posture
  tests
- snapshot determinism, validation, lookup-cost, and internal mutability tests
- minimal facade-only structural app integration proof
- registry-extension propagation proof

The milestone is not relying on documentation to protect the facade or lifecycle
boundaries. The closure evidence is compiler-visible, test-visible, or exposed
through typed runtime reports.

## QA Hardening

The final QA loops materially strengthened the milestone before closeout:

- snapshot inspection was tightened to assert index-backed lookup counters for
  representative capabilities
- minimal-app rejection cases now prove rejected capabilities stay out of the
  accepted snapshot while the valid representative app remains intact
- facade visibility coverage gained local proof that snapshot internals cannot
  be constructed directly
- registry-extension proof gained `RegistryFamilyInventoryAudit` so omitted,
  unknown, and duplicate reported family names cannot silently disappear
- facade exposure proof now requires an explicit public or internal-only
  exposure decision rather than assuming every family must be public
- diagnostics aggregation proof now covers duplicate diagnostics across command,
  component, setting, and task presentation families through real facade
  registration paths

Those corrections matter because the milestone is a foundation. A green suite
that only proved tidy happy paths would not have been enough.

## What This Does Not Claim

Milestone 1 deliberately does not ship:

- Worth UI source parsing
- concrete artifact lowering
- hot reload transport or renderer patching
- command routing or command execution
- shell rendering
- Query execution
- plugin loading or sandboxing
- native adapter implementation
- design-system visual breadth
- generated UI from source files

Those are downstream milestones. Milestone 1 closes the registration and
snapshot authority they need to consume.

## Allowed Debt After Closeout

No Milestone 1 completion debt is being carried for the facade, registry,
diagnostic, snapshot, minimal-app, or registry-extension proof surfaces.

Remaining work is future scope, not hidden incompleteness:

- source language and hot-reload artifact lifecycle
- runtime renderer integration
- command execution semantics
- full plugin architecture
- native platform adapters
- design-system theming and visual component breadth
- richer performance certification under large registration volumes

## Verification Snapshot

The closeout was verified against the active implementation with:

- `cargo fmt -p worth-ui --check`
- `cargo test -p worth-ui --test registry_extension_proof`
- `cargo test -p worth-ui`
- `git diff --check`
- touched-file line-cap sweep
- TODO/debug residue sweep

## Next Active Milestone

With Milestone 1 closed, the next Worth UI milestone can start from frozen
capability snapshots as its source of truth. The next milestone should not
reopen registration authority unless it is explicitly extending the registry
family inventory and proving the same lifecycle propagation boundaries again.

