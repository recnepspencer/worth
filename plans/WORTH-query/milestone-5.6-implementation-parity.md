# Milestone 5.6 Implementation Parity

## Purpose

This note records the current implementation parity for Milestone 5.6 after
the unified-facade QA and correction loop. It exists to distinguish:

- shipped milestone substance
- shipped but intentionally narrow areas
- explicit debt that should not be silently mistaken for completion

## Shipped

- `worth-query` now owns a real application-facing subdomain in
  `crates/worth-query/src/application/` rather than adding more broad facade
  passthroughs.
- Unified runtime configuration is subsystem-owned rather than capability-
  shaped:
  - `query`
  - `relational`
  - `signal`
  - `runtime_bridge`
  - `store`
- raw root configuration now validates into a proof-bearing
  `ValidatedWORTHQueryConfig` before capability admission.
- configuration failures are typed and distinct:
  - `MissingRequiredSection`
  - `ContradictorySectionPosture`
  - `DeferredStoreBackedSection`
- capability support taxonomy is frozen and machine-checkable:
  - `WORTHQueryCapabilityFamily`
  - `WORTHQueryCapabilitySupportStatus`
  - `WORTHQueryCapabilityDescriptor`
  - `WORTHQueryCapabilityRegistry`
  - `WORTHQuerySupportMatrix`
  - `WORTHQuerySupportReport`
- the application facade now owns one support-matrix authority path and derives
  support reports from that same owned matrix rather than rebuilding support
  truth through a second parallel derivation path.
- capability admission now produces a proof-bearing
  `CapabilityAdmissionDecision` carrying:
  - family/status/owner/section descriptor identity
  - validated-config identity
  - exact facade counters
  - a decision digest
- the public daily-driver capability surface is now one family-named witness
  acquisition path per family:
  - `query_read_capability()`
  - `live_query_capability()`
  - `preview_query_capability()`
  - `workflow_query_capability()`
  - `historical_query_capability()`
  - `query_context_capability()`
  - `durable_artifact_capability()`
- the earlier duplicate public acquisition shape was removed; decision-only
  methods now remain internal-only.
- capability witnesses remain narrow and proof-bearing instead of collapsing
  into one broad application bag.
- deferred durable/store-backed capability posture remains explicit debt rather
  than implied support.
- compile-fail boundaries now cover:
  - private validated config construction
  - private support report construction
  - private admission-decision construction
  - internal `application/*` module access
  - forbidden dynamic `capability(family)` routing
  - cross-family witness misuse
  - legacy broad-facade shortcut attempts
- milestone-native unified facade certification now exists as a first-class
  slice in `crates/worth-query/src/harness/unified_facade_certification/`.

## Shipped But Still Narrow

- the unified facade is closed for the currently admitted runtime-backed
  capability mix. It does not yet imply durable/store-backed completion, and
  it does not claim later Milestone 6 durability or historical/store parity.
- support-report counters are intentionally small:
  `support_report_generation_count` is the shipped proof surface; richer
  diagnostic narratives remain unnecessary for 5.6 closure.
- capability denial topology is now distinct for unsupported, deferred, and
  invalid configuration, but the currently admitted surface still exercises only
  the capability families already closed in earlier milestones.
- the legacy broad re-export wall still exists for compatibility, but no new
  5.6 composition-first surface is shipped there and certification now guards
  against that regression.

## Explicit Debt

- None currently recorded for the Milestone 5.6 implementation boundary.

## Next Implications For Milestone 6

- Milestone 6 should compose through the application facade and validated
  config/support authority surfaces already frozen here rather than inventing a
  second application-layer admission engine.
- future capability families should follow the same pattern:
  - subsystem-owned config section
  - registry/matrix descriptor
  - family-named witness acquisition method
  - compile-fail boundary
  - certification rows
- future durable/store-backed work must remain explicit debt until it can be
  admitted honestly through this same support/admission chain.
