# Milestone 8 Closeout: Scopes, Templates, Saved Query Artifacts, And View-Shape Semantics

## Status

Milestone 8 is closed as of 2026-04-20 for the admitted composition,
ephemeral saved-query, view-shape, grouped-live, and identity-aware inspector
scope in `worth-query`.

This closeout reflects the runtime-backed admitted surface only. Durable
saved-query reload, portable artifact exchange, and later store-backed
continuation semantics remain explicit later-milestone debt.

## Shipped Scope

Milestone 8 delivered:

- canonical scope expansion, template instantiation, and composition lineage in
  [crates/worth-query/src/composition](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/composition)
- ephemeral saved-query freeze, reuse legality, and semantic drift reporting in
  [crates/worth-query/src/saved_query](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/saved_query)
- admitted view-shape planning and identity-consumption contracts in
  [crates/worth-query/src/view_shape](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape)
- live view-shape lowering, grouped execution, and inspector patch semantics in
  [crates/worth-query/src/view_shape_live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape_live)
- cross-crate grouped truth authority in
  [crates/worth-relational/src/grouped_truth](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/grouped_truth)
  and
  [crates/worth-runtime-bridge/src/source](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/source)
- Milestone 8 certification in
  [crates/worth-query/src/harness/milestone_eight_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_eight_certification)
- supporting Milestone 7 identity-evolution certification and inspector bridge
  seams in
  [crates/worth-query/src/harness/identity_evolution_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/identity_evolution_certification)
  and
  [crates/worth-query/src/identity_evolution](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/identity_evolution)
- compile-fail proof boundaries in
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)
  and
  [crates/worth-runtime-bridge/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/tests/ui)

The semantic center that now exists is:

one canonical query meaning can be authored directly, through scopes, through
templates, frozen into ephemeral saved-query artifacts, admitted as explicit
view shapes, lowered into planner-visible and live-visible semantics, and
certified through hostile rows without host repair, cosmetic regrouping,
payload rediscovery, identity flattening, or durable overclaim.

## Acceptance Mapping

Milestone 8 is considered closed against:

- [milestone-8.md](./milestone-8.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)

because the admitted runtime-backed composition, saved-query, view-shape,
grouped-live, and identity-aware inspector proof surfaces now exist directly.

### `Scope / Template / View-Shape Semantic Parity Test`

Covered by:

- [crates/worth-query/src/harness/milestone_eight_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_eight_certification/mod.rs)
- [crates/worth-query/src/harness/milestone_eight_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_eight_certification/tests.rs)

What is proven:

- the named certification artifact exists as the Milestone 8 closeout suite
- required canonical rows are present and exercised, including:
  - `direct-vs-scope-parity`
  - `direct-vs-template-parity`
  - `saved-query-freeze-parity`
  - `view-shape-non-cosmetic-planning-live`
  - `kanban-desired-state-to-delta-parity`
  - `kanban-delta-admission-boundary`
  - `grouped-refresh-honesty`
  - `grouped-bridge-truth-view-authority`
  - `grouped-query-execution-surface-authority`
  - `grouped-proof-chain-no-payload-rediscovery`
  - `inspector-observed-focused-distinction`
  - `identity-aware-focused-inspector-parity`
  - `identity-break-inspector-explicitness`
  - `support-profile-honesty`
- required rejection rows are present and exercised:
  - `unsupported-scope-family`
  - `unsupported-template-family`
  - `saved-query-support-profile-drift`
  - `durable-saved-query-deferred-debt`
  - `post-admission-view-mutation-forbidden`
  - `grouped-hidden-refresh-forbidden`
- the matrix proves equality, inequality, typed rejection, and digest
  stability rather than row presence alone

### `Canonical composition and saved-query freeze honesty`

Covered by:

- [crates/worth-query/src/composition](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/composition)
- [crates/worth-query/src/saved_query](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/saved_query)
- [crates/worth-query/src/saved_query/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/saved_query/tests.rs)

What is proven:

- direct construction, scope expansion, and template instantiation preserve one
  canonical query meaning for admitted families
- saved-query artifacts freeze canonical meaning plus composition, view, and
  identity-consumption metadata instead of introducing a second AST or host bag
- saved-query reuse classifies legality through an explicit rebinding matrix
- identity-aware inspector contract drift is classified as semantic drift or
  fresh-freeze-required behavior rather than cosmetic view change
- durable persistence remains honest deferred debt

### `View-shape planning and live semantic honesty`

Covered by:

- [crates/worth-query/src/view_shape](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape)
- [crates/worth-query/src/view_shape_live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape_live)
- [crates/worth-query/src/view_shape/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape/tests.rs)
- [crates/worth-query/src/view_shape_live/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/view_shape_live/tests.rs)

What is proven:

- view shape is distinct from result shape and planner-visible
- table, detail, observed inspector detail, focused inspector detail, and
  kanban grouped are explicit admitted families
- identity-aware inspector is an explicit inspector-only contract, not ambient
  metadata over all view families
- focused inspector patch delivery can carry explicit identity classification
  and preserve explicit identity break instead of degrading it to generic
  denial, empty patch, or missing-data semantics
- grouped live execution requires grouped admission and authoritative grouped
  truth rather than piggybacking on the ungrouped live entrypoint
- grouped desired state and grouped delta are derived from prior and next
  authoritative grouped truth rather than payload heuristics
- focused inspector widening and post-admission mutation remain denied by type
  and boundary

### `Cross-feature Milestone 7 / 8 seam honesty`

Covered by:

- [crates/worth-query/src/identity_evolution/inspector.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/identity_evolution/inspector.rs)
- [crates/worth-query/src/harness/identity_evolution_certification/matrix.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/identity_evolution_certification/matrix.rs)
- [crates/worth-query/src/harness/identity_evolution_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/identity_evolution_certification/tests.rs)
- [crates/worth-query/src/harness/milestone_eight_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_eight_certification/mod.rs)

What is proven:

- Milestone 7 now exposes explicit `IdentityBreak` meaning rather than hiding
  identity break in generic denial
- identity-evolution support truth now reports comparison basis families and
  inspector-consumable identity classifications explicitly
- the `identity-aware-inspector-consumption-parity` row proves inspector-facing
  identity wrapping does not flatten Milestone 7 classification
- the Milestone 8 rows `identity-aware-focused-inspector-parity` and
  `identity-break-inspector-explicitness` prove focused inspector delivery
  preserves identity classification rather than reinterpreting it in view code

### `Compile-time enforcement and exact spec traceability`

Covered by:

- [crates/worth-query/tests/phase_boundaries_compile_fail.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/phase_boundaries_compile_fail.rs)
- [crates/worth-runtime-bridge/tests/phase_boundaries_compile_fail.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/tests/phase_boundaries_compile_fail.rs)
- [crates/worth-query/tests/ui/post_admission_view_mutation_forbidden.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/post_admission_view_mutation_forbidden.rs)
- [crates/worth-query/tests/ui/identity_aware_inspector_bool_shortcut_forbidden.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/identity_aware_inspector_bool_shortcut_forbidden.rs)
- [crates/worth-query/tests/ui/identity_aware_inspector_post_admission_mutation_forbidden.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/identity_aware_inspector_post_admission_mutation_forbidden.rs)
- [crates/worth-query/tests/ui/inspector_identity_artifact_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/inspector_identity_artifact_constructor_private.rs)
- [crates/worth-query/tests/ui/identity_break_flattening_accessor_forbidden.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/identity_break_flattening_accessor_forbidden.rs)

What is proven:

- post-admission view mutation remains compile-fail, not just documentation
- identity-aware inspector cannot be selected through a bool shortcut
- inspector identity artifacts cannot be fabricated directly through the public
  surface
- identity-break semantics cannot be flattened behind a generic best-match
  accessor

## Final QA Outcome

The final reconciliation pass was run against:

- the Milestone 7 and Milestone 8 specs
- the identity-evolution, view-shape, view-shape-live, and saved-query code
  surfaces
- the Milestone 7 identity-evolution certification matrix
- the Milestone 8 certification matrix
- the compile-fail boundary set

The last meaningful seam findings were:

- Milestone 8 claimed identity-aware inspector parity without actually
  consuming Milestone 7 artifacts structurally
- Milestone 7 still modeled explicit identity break too generically for clean
  inspector consumption
- saved-query freeze and reuse did not yet bind identity-aware inspector
  contract drift
- support truth under-described the Milestone 7 comparison and
  inspector-consumable surfaces

Those findings were corrected before this closeout.

After the correction pass, I do not see a remaining meaningful Milestone 7 and
8 seam mismatch for the admitted runtime-backed scope.

## Explicit Deferred Scope

Milestone 8 is closed for admitted composition, ephemeral saved-query,
identity-aware inspector, grouped truth-view, and grouped live semantics only.

The following remain explicit later work:

- durable saved-query reload, import/export portability, and restart-stable
  continuation
- additional scope and template families beyond the admitted surface
- additional grouped, timeline, or chart families beyond grouped kanban
- richer lower-engine grouped execution beyond the admitted grouped truth-view
  substrate
- non-inspector identity-aware view families
- policy masking, tenant schema variation, and durable cursor or delivery
  continuation semantics

## What Later Milestones May Now Assume

Later milestones may safely assume:

- composition semantics are canonical and query-owned
- saved-query freeze has one semantic model with explicit reuse legality
- view shape is distinct from result shape
- identity-aware inspector is an admitted explicit contract for inspector
  families
- Milestone 7 identity classification can be consumed through Milestone 8 view
  semantics without flattening identity meaning
- grouped truth already flows through relational authority, bridge
  truth-view materialization, and query grouped execution as one typed proof
  chain
- the certification suites already prove exact row coverage, exact denial
  coverage, and compile-time boundary enforcement for the admitted surface

Later milestones must not assume:

- durable saved-query semantics are already available
- identity-aware semantics are admitted for non-inspector view families
- grouped truth may be rediscovered from raw payloads
- inspector identity may be reinterpreted by host or UI glue after delivery

## Verification Baseline

Milestone 8 closeout is grounded in:

- `cargo test -p worth-query --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`

These runs cover:

- composition and saved-query regression coverage
- view-shape and live view-shape regression coverage
- Milestone 7 identity-evolution certification
- Milestone 8 certification
- trybuild compile-fail proof boundaries, including the identity-aware
  inspector additions

## Operational Conclusion

Milestone 8 is now the normative runtime-backed composition and view-semantics
surface for the admitted `worth-query` product scope.

`worth-query` no longer depends on host-local scope/template sugar,
controller-owned saved-query conventions, cosmetic inspector labels, identity
flattening in focused inspector delivery, payload-based grouped rediscovery, or
hidden refresh fallback to express the admitted Milestone 8 surface.

The remaining work is explicit later-milestone scope, not hidden architectural
debt inside the Milestone 8 boundary.
