# Milestone 5.6 Closeout: Unified Application Facade And Unified Runtime Configuration

## Status

Milestone 5.6 is closed as of 2026-04-19 for the admitted unified-facade and
unified-configuration scope in `worth-query`.

`worth-query` now owns the application-facing daily-driver surface for the
currently admitted runtime-backed capability mix, while preserving:

- subsystem-owned configuration
- typed capability families
- typed capability witnesses
- machine-checkable support truth
- typed unsupported/deferred/configuration denial
- legacy-wall containment for new 5.6 composition-first APIs

The semantic center shipped in this milestone is:

developers can now configure and use the admitted `worth-query` capability mix
through one coherent application-facing facade without flattening subsystem
ownership into one bag-shaped config, without collapsing capability families
into one broad witness, and without letting support claims drift from actual
admission behavior.

## Shipped Scope

Milestone 5.6 delivered:

- subsystem-owned unified configuration in
  `config.rs`
- typed capability witnesses, admission artifacts, and facade entrypoints in
  [facade.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/facade.rs),
  [resolution.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/resolution.rs),
  [errors.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/errors.rs),
  and
  [witnesses.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/witnesses.rs)
- support registry, support matrix, and support report authority surfaces in
  [registry.rs](../../workspaces/worth-query/crates/worth-query/src/application/support/registry.rs)
  and
  [report.rs](../../workspaces/worth-query/crates/worth-query/src/application/support/report.rs)
- milestone-native certification in
  [crates/worth-query/src/harness/unified_facade_certification](../../workspaces/worth-query/crates/worth-query/src/harness/unified_facade_certification)
- compile-fail proof boundaries in
  `crates/worth-query/tests/ui`

## Acceptance Mapping

Milestone 5.6 is considered closed against
[milestone-5.6.md](./milestone-5.6.md),
[milestone-5.6-build-checklist.md](./milestone-5.6-build-checklist.md),
[worth_query_roadmap.md](./WORTH_query_roadmap.md),
and
[test-requirements.md](./test-requirements.md)
because the required facade/configuration proof chain now exists directly.

### `Unified Facade And Configuration Boundary Test`

Covered by:

- [mod.rs](../../workspaces/worth-query/crates/worth-query/src/harness/unified_facade_certification/mod.rs)
- [lane.rs](../../workspaces/worth-query/crates/worth-query/src/harness/unified_facade_certification/lane.rs)
- `matrix.rs`
- [row_catalog.rs](../../workspaces/worth-query/crates/worth-query/src/harness/unified_facade_certification/row_catalog.rs)
- [tests.rs](../../workspaces/worth-query/crates/worth-query/src/harness/unified_facade_certification/tests.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5.6
  closeout surface
- required canonical rows are present and exercised:
  - `unified-query-read-capability`
  - `unified-query-context-capability`
  - `unified-live-capability`
  - `unified-preview-capability`
  - `unified-workflow-capability`
  - `unified-historical-capability`
  - `unified-config-section-explicitness`
  - `capability-support-metadata-sync`
- required rejection rows are present and exercised:
  - `missing-owning-live-section`
  - `invalid-workflow-support-posture`
  - `deferred-durable-artifacts`
  - `invalid-unified-configuration`
- the suite asserts exact counter values rather than row presence alone:
  - capability lookup counts
  - section-resolution counts
  - unsupported denial counts
  - deferred denial counts
  - support-report generation counts
  - configuration-validation denial counts for invalid config

### `Typed application facade and one-family-one-acquisition-path discipline`

Covered by:

- [facade.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/facade.rs)
- [resolution.rs](../../workspaces/worth-query/crates/worth-query/src/application/capability/resolution.rs)
- `tests.rs`

What is proven:

- the public daily-driver surface now exposes one family-named witness
  acquisition path per capability family
- validated-config identity is carried into capability admission decisions
- unsupported and deferred capability families fail typed and early with
  distinct counters
- the earlier duplicate public acquisition shape has been removed from the
  normative API surface

### `Support truth and config posture synchronization`

Covered by:

- [registry.rs](../../workspaces/worth-query/crates/worth-query/src/application/support/registry.rs)
- [report.rs](../../workspaces/worth-query/crates/worth-query/src/application/support/report.rs)
- `tests.rs`

What is proven:

- support truth is frozen through explicit registry/matrix/report artifacts
- support reports now derive from the same owned support matrix the facade
  uses, rather than from a second equivalent-but-parallel derivation path
- section posture, validated-config identity, admitted/deferred/unsupported
  family sets, and support report counters are machine-checkable

### `Compile-time enforcement and legacy-wall containment`

Covered by:

- `tests/ui/validated_worth_query_config_constructor_private.rs`
- `tests/ui/worth_query_support_report_constructor_private.rs`
- `tests/ui/capability_admission_decision_constructor_private.rs`
- `tests/ui/query_read_capability_constructor_private.rs`
- `tests/ui/live_query_capability_constructor_private.rs`
- `tests/ui/preview_session_capability_constructor_private.rs`
- `tests/ui/workflow_orchestration_capability_constructor_private.rs`
- `tests/ui/historical_evaluation_capability_constructor_private.rs`
- `tests/ui/query_context_capability_constructor_private.rs`
- `tests/ui/facade_has_no_dynamic_capability_routing.rs`
- `tests/ui/internal_application_module_not_public.rs`
- `tests/ui/facade_query_read_capability_has_no_live_promote.rs`
- `tests/ui/facade_preview_capability_cannot_admit_workflow.rs`
- `tests/ui/facade_historical_capability_cannot_bind_query_context.rs`
- `tests/ui/legacy_broad_facade_has_no_preview_workflow_shortcut.rs`

What is proven:

- capability witnesses themselves remain externally unconstructable
- support-owned and admission-owned artifacts remain sealed
- dynamic `capability(family)` routing is forbidden
- cross-family witness misuse remains uncompilable
- internal `application/*` modules remain outside the public facade boundary
- the legacy broad facade does not grow a new preview/workflow composition
  shortcut that would compete with the application facade

## Final QA Outcome

The last hostile QA loop was run directly against the 5.6 spec, the build
checklist, the production application facade/config/support modules, the
unified-facade certification slice, and the compile-fail boundary set.

The last meaningful findings were:

- the facade still exposed two public acquisition paths per capability family,
  which made the daily-driver surface ambiguous
- support reports were derived through a second equivalent path instead of from
  the exact same support-matrix authority artifact used by the facade
- the unified-facade certification tests were not explicitly requiring the key
  5.6 compile-fail boundary set, only the legacy shortcut boundary

Those findings were corrected before this closeout. After the correction pass,
I do not see a remaining meaningful 5.6 spec/code mismatch for the admitted
unified-facade scope.

The last strict QA pass also closed two lower-level proof gaps that were still
too soft for production-grade signoff:

- the denial taxonomy now distinguishes missing owning-section denial from
  invalid composed-support-posture denial instead of flattening both into one
  generic unsupported capability class
- the compile-fail boundary set now explicitly proves that capability witness
  types themselves cannot be externally constructed through the public facade
  surface

With those corrections in place, the admitted 5.6 scope is considered
production-ready rather than merely implemented.

## Explicit Deferred Scope

Milestone 5.6 is closed for the admitted unified-facade and unified-
configuration scope only.

The following remain later-milestone work, not implied completeness:

- durable/store-backed capability admission beyond the explicit deferred debt
  posture
- broader Milestone 6 historical/diff/durable closure
- any future capability family not yet admitted through the 5.6 proof chain
- eventual reduction of the compatibility-oriented broad re-export wall once
  later milestones no longer need it for transition safety

The current 5.6 surface is intentionally honest about what is admitted,
deferred, and unsupported.

## What Later Milestones May Now Assume

Milestones 6 and later may safely assume:

- `worth-query` already owns an application-facing daily-driver facade
- unified configuration is already subsystem-owned and validated before
  capability admission
- capability support truth is already frozen through registry/matrix/report
  artifacts
- capability admission already produces proof-bearing decisions tied to
  validated-config identity
- compile-time and certification guardrails already exist for dynamic routing,
  cross-family witness misuse, and legacy-wall shortcut regression

Those milestones must not assume:

- deferred durable/store-backed capability families are secretly available
- the broad re-export wall is the normative place for new composition-first
  APIs
- support advertisement can be inferred from method presence
- one generic witness or one generic facade operation is an allowed shape

## Verification Baseline

Milestone 5.6 closeout was verified with:

- `cargo test -p worth-query application --quiet`
- `cargo test -p worth-query unified_facade_certification --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`

This passes cleanly and includes:

- application/facade unit tests
- unified-facade certification and closeout artifact tests
- trybuild compile-fail proof-boundary tests
- full regression coverage for `worth-query`

## Operational Conclusion

Milestone 5.6 is now closed at the unified application-facade and unified
runtime-configuration layer.

`worth-query` no longer depends on broad-facade convenience drift, capability-
shaped config bags, method-presence support discovery, or duplicate public
acquisition surfaces to present the admitted runtime-backed capability mix. It
now has subsystem-owned unified configuration, proof-bearing validated config,
registry-owned support truth, one-family-one-acquisition-path capability
surfaces, exact counter-bearing certification, compile-time facade boundary
enforcement, and explicit legacy-wall containment that later milestones can
build on safely.

Production-readiness statement:

Milestone 5.6 is ready to treat as the normative application-facing surface for
the admitted runtime-backed capability mix. The remaining later work is
explicit deferred scope, not hidden architectural debt inside the 5.6 boundary.
