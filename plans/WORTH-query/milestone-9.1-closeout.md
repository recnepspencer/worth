# Milestone 9.1 Closeout: Query-Owned Subscription Declaration, Lowering, And Admission

## Status

Milestone 9.1 is closed as of 2026-04-23 for the runtime-backed
query-owned subscription declaration, family selection, bridge lowering,
basis binding, admission, diagnostics, support, certification, and
compile-time boundary scope in `worth-query`.

This closeout reflects the declaration and admission boundary only. Active
subscription lifecycle, sharing, fanout, continuation, preview isolation,
delivery windows, durable subscription artifact persistence, restart-stable
subscription metadata, and store-backed restart parity remain explicit later
milestone debt.

## Governing Source Summary

- `MENTALITY.md`: protects adversarial, mechanically enforced foundations over
  MVP feature shape. The 9.1 closeout therefore treats subscription declaration
  as a proof chain, not a convenience subscribe API.
- `arch_laws.md`: protects proof-bearing phase progression, sealed
  construction, authority boundaries, typed errors, and self-describing
  envelopes. The implementation closes only where the compiler or tests enforce
  the phase chain.
- `perf_laws.md`: protects counter-visible bounded work and rejection before
  expensive construction. The 9.1 surface exposes family, slice, bridge,
  admission, drift, allocation, and scale-slope counters.
- `domain_laws.md`: protects domain-aligned decomposition and single
  responsibility. The implementation keeps family selection, equivalence,
  declaration, slice intent, bridge lowering, admission, diagnostics, support,
  and certification as separate subscription responsibilities.
- `worth_query_vision.md`: protects one typed query model that can become live
  without separate host-specific subscription semantics. 9.1 closes the
  declaration side of read-to-subscribe promotion.
- `worth_query_roadmap.md`: protects the sequence from policy-safe live meaning
  into subscription declaration before active lifecycle. 9.1 now provides the
  proof-bearing activation input that 9.2 may consume.
- `test-requirements.md`: protects certification-grade, machine-checkable
  query evidence. 9.1 closes against the `Query Subscription Declaration And
  Lowering Parity Test`.
- `milestone-9-closeout.md`: protects the policy, tenant, relationship-proof,
  live, and delivery admission surfaces that subscription declaration must
  consume rather than reimplement.

## Adversarial Constraint Closed

Milestone 9.1 had to survive the condition where the same live query is
authored directly, through scopes, templates, saved exact reuse, and facade
helpers; narrowed by policy, tenant, relationship proof, view shape, and basis;
then lowered into a subscription without host observer inference, raw CDC
fallback, one generic subscription kind, or post-admission digest mutation.

The closed surface now enforces this by requiring one typed progression:

1. `LiveQueryAdmissionArtifact`
2. `QuerySubscriptionFamilySelection`
3. `QuerySubscriptionDeclarationArtifact`
4. `QuerySubscriptionBasisBindingRequest`
5. `BridgeSubscriptionLoweringPlan`
6. `QuerySubscriptionAdmissionArtifact`
7. `SubscriptionActivationInput`

Each step consumes the previous proof type. No public path creates activation
input from raw live descriptors, raw bridge declarations, raw CDC filters, host
callbacks, or generic subscription shortcuts.

## Shipped Scope

Milestone 9.1 delivered:

- query-owned subscription family vocabulary in
  [crates/worth-query/src/subscription/family.rs](../../crates/worth-query/src/subscription/family.rs)
- subscription family selection and dimension validation in
  [crates/worth-query/src/subscription/selection.rs](../../crates/worth-query/src/subscription/selection.rs)
- subscription equivalence and meaning digests in
  [crates/worth-query/src/subscription/equivalence.rs](../../crates/worth-query/src/subscription/equivalence.rs)
- live admission input and immutable policy/tenant/relationship-proof
  evidence accessors in
  [crates/worth-query/src/subscription/input.rs](../../crates/worth-query/src/subscription/input.rs)
- query-owned declaration artifacts, declaration digests, delivery intent, and
  slice intent in
  [crates/worth-query/src/subscription/declaration.rs](../../crates/worth-query/src/subscription/declaration.rs),
  [crates/worth-query/src/subscription/declaration_digest.rs](../../crates/worth-query/src/subscription/declaration_digest.rs),
  [crates/worth-query/src/subscription/delivery.rs](../../crates/worth-query/src/subscription/delivery.rs), and
  [crates/worth-query/src/subscription/slice.rs](../../crates/worth-query/src/subscription/slice.rs)
- explicit query-to-bridge family and slice maps in
  [crates/worth-query/src/subscription/bridge_family.rs](../../crates/worth-query/src/subscription/bridge_family.rs) and
  [crates/worth-query/src/subscription/bridge_slice.rs](../../crates/worth-query/src/subscription/bridge_slice.rs)
- bridge lowering, basis binding requests, and signal strategy requests in
  [crates/worth-query/src/subscription/bridge_lowering.rs](../../crates/worth-query/src/subscription/bridge_lowering.rs),
  [crates/worth-query/src/subscription/basis_request.rs](../../crates/worth-query/src/subscription/basis_request.rs), and
  [crates/worth-query/src/subscription/signal_strategy.rs](../../crates/worth-query/src/subscription/signal_strategy.rs)
- runtime-backed admission, activation input, support profile, admission
  diagnostics, and certification in
  [crates/worth-query/src/subscription/admission.rs](../../crates/worth-query/src/subscription/admission.rs),
  [crates/worth-query/src/subscription/activation.rs](../../crates/worth-query/src/subscription/activation.rs),
  [crates/worth-query/src/subscription/support.rs](../../crates/worth-query/src/subscription/support.rs),
  [crates/worth-query/src/subscription/admission_diagnostics.rs](../../crates/worth-query/src/subscription/admission_diagnostics.rs), and
  [crates/worth-query/src/subscription/certification.rs](../../crates/worth-query/src/subscription/certification.rs)
- exact declaration/admission counters in
  [crates/worth-query/src/subscription/counters.rs](../../crates/worth-query/src/subscription/counters.rs)
- milestone certification in
  [crates/worth-query/src/harness/milestone_nine_one_certification](../../crates/worth-query/src/harness/milestone_nine_one_certification)
- public facade exposure for the admitted subscription declaration surface in
  [crates/worth-query/src/facade.rs](../../crates/worth-query/src/facade.rs)
- compile-fail proof boundaries in
  [crates/worth-query/tests/ui](../../crates/worth-query/tests/ui)

The semantic center that now exists is:

an admitted live query can be classified into one query subscription family,
frozen into one query-owned declaration, lowered into one explicit bridge
declaration and bridge basis request, admitted into one runtime-backed
subscription artifact, and handed to 9.2 only as `SubscriptionActivationInput`.
Unsupported combinations fail typed before the next proof type exists.

## Acceptance Mapping

Milestone 9.1 is considered closed against:

- [milestone-9.1.md](./milestone-9.1.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [milestone-9-closeout.md](./milestone-9-closeout.md)

because the runtime-backed subscription declaration and admission boundary now
exists directly and is certified by machine-checkable artifacts.

### `Query Subscription Declaration And Lowering Parity Test`

Covered by:

- [crates/worth-query/src/harness/milestone_nine_one_certification/mod.rs](../../crates/worth-query/src/harness/milestone_nine_one_certification/mod.rs)
- [crates/worth-query/src/harness/milestone_nine_one_certification/tests.rs](../../crates/worth-query/src/harness/milestone_nine_one_certification/tests.rs)
- [crates/worth-query/src/harness/certification/requirements.rs](../../crates/worth-query/src/harness/certification/requirements.rs)

What is proven:

- the named certification suite exists as
  `Query Subscription Declaration And Lowering Parity Test`
- required canonical rows are present, including direct/scope/template/saved
  parity, facade-helper parity, collection bridge parity, grouped and
  inspector shared-bridge distinctions, bounded materialization lowering,
  activation certification, policy/tenant basis binding, relationship-proof
  binding, and scale-slope honesty
- required rejection rows are present, including view-family mismatch, bridge
  family denial, masked detail/table/grouped slice denial, relationship-proof
  drift, durable reload overclaim, scale source mismatch, and zero-row scale
  drift
- rows prove equality, inequality, typed failure, and zero-residue assertion
  classes
- certification bundles expose the required verification outputs:
  `query_digest`, `live_family_digest`, `subscription_family_digest`,
  `subscription_declaration_digest`, `subscription_equivalence_digest`,
  `policy_digest`, `tenant_basis_digest`, `relationship_proof_digest`,
  `view_shape_digest`, `basis_digest`, `bridge_declaration_digest`,
  `bridge_basis_digest`, `signal_strategy_digest`, `admission_digest`,
  `failure_digest`, `fixture_digest`, `compile_fail_boundary_digest`,
  `counter_snapshot`, and `support_matrix_digest`

### `Family selection and subscription meaning`

Covered by:

- [crates/worth-query/src/subscription/selection.rs](../../crates/worth-query/src/subscription/selection.rs)
- [crates/worth-query/src/subscription/equivalence.rs](../../crates/worth-query/src/subscription/equivalence.rs)
- [crates/worth-query/src/subscription/tests/family_selection.rs](../../crates/worth-query/src/subscription/tests/family_selection.rs)
- [crates/worth-query/src/subscription/tests/equivalence.rs](../../crates/worth-query/src/subscription/tests/equivalence.rs)
- [crates/worth-query/src/subscription/tests/diagnostics.rs](../../crates/worth-query/src/subscription/tests/diagnostics.rs)

What is proven:

- `DetailExact`, `CollectionMembership`, `BoundedMaterialization`,
  `GroupedCollectionMembership`, and `InspectorDetailExact` are explicit query
  subscription families
- grouped and inspector query-side meanings remain distinct even where they
  lower onto bridge collection or detail families
- policy, tenant, and relationship-proof context changes alter subscription
  declaration and basis-request meaning rather than drifting silently
- relationship-proof posture drift fails before declaration or bridge lowering
- counter evidence is honest: relationship-proof drift and exhausted lookup
  budget deny before claiming a registry lookup

### `Query subscription declaration artifact`

Covered by:

- [crates/worth-query/src/subscription/declaration.rs](../../crates/worth-query/src/subscription/declaration.rs)
- [crates/worth-query/src/subscription/slice.rs](../../crates/worth-query/src/subscription/slice.rs)
- [crates/worth-query/src/subscription/delivery.rs](../../crates/worth-query/src/subscription/delivery.rs)
- [crates/worth-query/src/subscription/tests/declaration_parity.rs](../../crates/worth-query/src/subscription/tests/declaration_parity.rs)
- [crates/worth-query/src/subscription/tests/declaration_budget.rs](../../crates/worth-query/src/subscription/tests/declaration_budget.rs)
- [crates/worth-query/src/subscription/tests/delivery_intent.rs](../../crates/worth-query/src/subscription/tests/delivery_intent.rs)

What is proven:

- declaration construction consumes `QuerySubscriptionFamilySelection`
- declaration digests bind family selection, equivalence, basis posture,
  delivery intent, and slice intent
- semantically equivalent construction sources produce identical declaration
  digests
- masked slice requests, unsupported grouping slices, unsupported bounded
  materialization slices, delivery-intent denial, slice-budget denial, and
  allocation denial fail typed before bridge lowering
- exact structural counters track declared slices, deduplicated slices, digest
  parts, masked denial, delivery denial, allocation denial, and bridge-lowering
  absence

### `Bridge lowering and basis binding`

Covered by:

- [crates/worth-query/src/subscription/bridge_lowering.rs](../../crates/worth-query/src/subscription/bridge_lowering.rs)
- [crates/worth-query/src/subscription/bridge_family.rs](../../crates/worth-query/src/subscription/bridge_family.rs)
- [crates/worth-query/src/subscription/bridge_slice.rs](../../crates/worth-query/src/subscription/bridge_slice.rs)
- [crates/worth-query/src/subscription/basis_request.rs](../../crates/worth-query/src/subscription/basis_request.rs)
- [crates/worth-query/src/subscription/signal_strategy.rs](../../crates/worth-query/src/subscription/signal_strategy.rs)
- [crates/worth-query/src/subscription/tests/bridge_lowering.rs](../../crates/worth-query/src/subscription/tests/bridge_lowering.rs)

What is proven:

- every admitted query subscription family maps to an explicit bridge family
  and admitted bridge slice set
- grouped and inspector semantics remain query declaration metadata rather
  than being smuggled into bridge protocol meaning
- bridge declaration digest is stable for equivalent declaration inputs
- basis request digest changes when basis posture changes
- unsupported bridge family, unsupported bridge slice, unsupported basis, and
  deferred bridge posture fail typed before admission
- bridge family lookup, slice count, basis binding, signal strategy, and
  lowering counters are exact and assertion-covered

### `Admission, activation input, support, and diagnostics`

Covered by:

- [crates/worth-query/src/subscription/admission.rs](../../crates/worth-query/src/subscription/admission.rs)
- [crates/worth-query/src/subscription/activation.rs](../../crates/worth-query/src/subscription/activation.rs)
- [crates/worth-query/src/subscription/admission_diagnostics.rs](../../crates/worth-query/src/subscription/admission_diagnostics.rs)
- [crates/worth-query/src/subscription/diagnostic.rs](../../crates/worth-query/src/subscription/diagnostic.rs)
- [crates/worth-query/src/subscription/support.rs](../../crates/worth-query/src/subscription/support.rs)
- [crates/worth-query/src/subscription/tests/admission.rs](../../crates/worth-query/src/subscription/tests/admission.rs)
- [crates/worth-query/src/subscription/tests/diagnostics.rs](../../crates/worth-query/src/subscription/tests/diagnostics.rs)

What is proven:

- admission consumes `BridgeSubscriptionLoweringPlan`
- `SubscriptionActivationInput` is produced only from
  `QuerySubscriptionAdmissionArtifact`
- successful admission binds bridge declaration digest, bridge basis digest,
  signal strategy digest, support profile, admission diagnostics, and exact
  counters
- durable reload requests deny before activation input exists and emit denied
  runtime support plus explicit durable debt
- active lifecycle allocation requests deny during admission and do not create
  lifecycle state inside Milestone 9.1
- diagnostics localize denial stage for view mismatch, declaration, delivery
  intent, bridge family lowering, bridge slice lowering, basis binding,
  durable overclaim, active lifecycle allocation, and relationship-proof drift

### `Compile-time enforcement and facade boundary`

Covered by:

- [crates/worth-query/src/subscription/mod.rs](../../crates/worth-query/src/subscription/mod.rs)
- [crates/worth-query/src/facade.rs](../../crates/worth-query/src/facade.rs)
- [crates/worth-query/tests/phase_boundaries_compile_fail.rs](../../crates/worth-query/tests/phase_boundaries_compile_fail.rs)
- [crates/worth-query/tests/ui](../../crates/worth-query/tests/ui)

What is proven:

- external code cannot fabricate `QuerySubscriptionFamilySelection`,
  `QuerySubscriptionDeclarationArtifact`, `QuerySubscriptionBasisBindingRequest`,
  `BridgeSubscriptionLoweringPlan`, `QuerySubscriptionAdmissionArtifact`,
  `SubscriptionActivationInput`, `QuerySubscriptionCertificationBundle`, scale
  reports, signal strategy requests, diagnostic evidence, or slice intent
  proof types
- raw live descriptors, raw bridge declarations, raw CDC filters, raw bridge
  families, raw bridge slices, host observer callbacks, generic subscription
  kind shortcuts, boolean family shortcuts, and durable reload shortcut APIs are
  not public construction paths
- `prepare_subscription_activation` cannot be called without admission
  evidence
- bridge lowering cannot be reached without declaration evidence
- admission cannot be reached without bridge-lowering evidence
- policy, tenant, and relationship-proof digest patch attempts after admission
  are uncompilable; callers may inspect immutable admitted digests but cannot
  mutate them in place

## Performance And Counter Closure

Milestone 9.1 closes the performance claims that belong to the runtime-backed
declaration boundary:

- query family selection performs one admitted family lookup when lookup budget
  allows it, and zero family lookups on relationship-proof drift or exhausted
  lookup budget
- bridge family lookup count is explicit and constant for admitted family
  lowering
- declaration slice width is derived from declared projection, ordering,
  grouping, relation-scope, and view metadata width
- grouped metadata width and inspector metadata width are counted explicitly
  as query declaration meaning
- declaration and bridge-lowering costs are certified against small, medium,
  and larger fixture row counts through scale-slope reports
- budget denials are typed separately from semantic unsupported-family,
  unsupported-slice, unsupported-basis, and durable-overclaim denials
- forbidden heap allocation, active lifecycle allocation, declaration-time
  checkpoint allocation, raw CDC fallback, host observer inference, and generic
  subscription fallback counters remain visible in admission/certification
  evidence

The final QA pass specifically corrected a counter-honesty issue where
relationship-proof drift and zero lookup budget could claim a family registry
lookup before that work had actually occurred.

## Explicit Deferred Scope

Milestone 9.1 is closed for runtime-backed subscription declaration,
bridge-lowering, basis binding, runtime-backed admission, activation input
handoff, diagnostics, support, certification, and compile-time boundaries only.

The following remain explicit later work:

- active subscription lifecycle handles
- subscription sharing and deduplication beyond producing equivalence digests
- fanout state
- continuation windows
- preview isolation for active subscriptions
- delivery windows and acknowledgement frontiers
- active subscription diagnostics beyond declaration/admission localization
- automatic subscription-family selection above the explicit declaration API
- durable subscription artifact persistence
- restart-stable subscription metadata
- durable continuation checkpoints
- store-backed restart parity
- bridge-owned subscription protocol expansion beyond the admitted detail and
  collection-membership bridge families

These are not hidden Milestone 9.1 implementation gaps. They are intentionally
reserved for Milestones 9.2, 9.3, 10, and 11.

## What Later Milestones May Now Assume

Later milestones may safely assume:

- subscription declaration is query-owned and proof-bearing
- subscription family selection is explicit before declaration
- declaration identity binds query, family, basis, policy, tenant,
  relationship-proof, view-shape, delivery, and slice intent meaning
- bridge lowering consumes query declaration artifacts rather than raw live
  descriptors or host observer state
- bridge basis requests are derived from admitted query/live basis posture
- admission is the only path to `SubscriptionActivationInput`
- unsupported families, unsupported slices, unsupported bases, durable reload
  overclaims, active lifecycle allocation requests, relationship-proof drift,
  raw CDC fallback, host observer inference, and generic subscription kinds
  fail closed before activation
- certification bundles emit the machine-checkable evidence fields required by
  `test-requirements.md`
- public callers may read admitted policy, tenant, and proof digests but cannot
  patch them after admission

Later milestones must not assume:

- active subscription lifecycle state exists
- sharing, fanout, continuation, preview isolation, or delivery windows exist
- subscription artifacts are durable or restart-stable
- store-backed subscription replay is admitted
- bridge protocol owns grouped or inspector query semantics
- raw CDC can substitute for query-owned declaration

## Final QA Outcome

The final reconciliation passes were run against:

- the Milestone 9.1 spec
- WORTH Query vision, roadmap, and test requirements
- Milestone 9 closeout
- the WORTH coding guideline documents
- subscription family selection, equivalence, declaration, bridge lowering,
  admission, support, diagnostics, certification, facade, and trybuild
  boundaries

The last meaningful findings were:

- relationship-proof drift was denied early but counter evidence could claim a
  family registry lookup that had not happened
- diagnostic tests were too often stage/digest-only rather than asserting typed
  denial, exact zero-residue counters, and selected family/width evidence
- certification bundles compressed required verification evidence into broader
  digests instead of exposing each spec-required output
- representative direct/scope/template/saved and facade-helper rows were
  covered semantically but not by explicit named rows
- policy, tenant, and relationship-proof post-admission drift ownership needed
  an explicit compile-time boundary rather than only an implementation
  assumption

Those findings were corrected before this closeout.

After the correction passes, I do not see a remaining meaningful Milestone 9.1
gap for the runtime-backed declaration and admission scope. The remaining work
is explicit active-lifecycle, store-backed, and durable scope, not hidden 9.1
implementation debt.

## Verification Baseline

Milestone 9.1 closeout is grounded in:

- `cargo fmt -p worth-query`
- `cargo test -p worth-query milestone_nine_one`
- `cargo test -p worth-query`
- `git diff --check`

The final full test run passed:

- 588 `worth-query` unit tests
- the `phase_boundaries_compile_fail` trybuild suite, including the Milestone
  9.1 subscription compile-fail targets and baselines
- doc tests

`git diff --check` passed with only existing CRLF normalization warnings in the
working tree.

## Operational Conclusion

Milestone 9.1 is now the normative runtime-backed subscription declaration and
admission boundary for `worth-query`.

`worth-query` no longer depends on host observer inference, raw CDC filters,
bridge-local family guessing, one generic subscription kind, post-admission
policy/tenant/proof mutation, or activation from raw live descriptors to express
the admitted Milestone 9.1 surface.

The next work can build active lifecycle and delivery behavior on top of one
canonical subscription declaration, one bridge lowering, one basis request, one
admission artifact, and one activation input rather than rediscovering
subscription meaning from runtime handles.

## Self-Check

- Does the milestone solve a real structural problem rather than packaging
  work cosmetically? Yes. It freezes the boundary between admitted live query
  meaning and active subscription lifecycle.
- Is the adversarial constraint precise and load-bearing? Yes. The tests cover
  alternate construction paths, view-shape variation, policy/tenant/proof
  meaning changes, bridge unsupported paths, masked slices, durable overclaims,
  and scale drift.
- Does the milestone preserve crate authority boundaries? Yes. `worth-query`
  owns declaration and query meaning; bridge protocol semantics remain explicit
  bridge lowering; active lifecycle remains later scope.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Certification rows, compile-fail targets, exact counters, and support
  evidence are required closure artifacts.
- Could a competent engineer map this closeout back into honest types,
  modules, and tests? Yes. Each acceptance section points at the concrete
  module and test surfaces.
- Does the milestone belong in this roadmap sequence? Yes. It depends on
  Milestone 9 policy/tenant/proof/live admission and creates the activation
  input required by Milestone 9.2 lifecycle work.
