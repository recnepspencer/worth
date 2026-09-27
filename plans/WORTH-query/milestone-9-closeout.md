# Milestone 9 Closeout: Policy-Aware Narrowing, Tenant Scope, And Delivery Contracts

## Status

Milestone 9 is closed as of 2026-04-21 for the admitted runtime-backed
policy-aware narrowing, tenant truth/schema basis, relationship-proof
admission, policy-aware execution seam, live admission, delivery contract, and
certification scope in `worth-query`.

This closeout reflects the runtime-backed admitted surface only. Store-backed
execution parity, durable policy cursors, durable artifact reload, durable
delivery metadata reload, restart-stable subscription metadata, and durable
tenant/query artifact portability remain explicit WORTH Store and later
milestone debt.

## Shipped Scope

Milestone 9 delivered:

- policy and tenant basis admission in
  [crates/worth-query/src/policy_basis](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_basis)
  and
  [crates/worth-query/src/tenant_basis](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/tenant_basis)
- policy mask snapshots, authorized projection artifacts, masked influence
  denial, and immutable mask boundaries in
  [crates/worth-query/src/authorized_projection](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/authorized_projection)
- query-authored relationship-proof descriptors, admission, budgets, and typed
  denials in
  [crates/worth-query/src/relationship_proof](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/relationship_proof)
- pre-execution narrowing, policy-aware validation reports, optimizer inputs,
  and saved-policy narrowing reuse classification in
  [crates/worth-query/src/policy_narrowing](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_narrowing)
- policy-aware current, branch, runtime-historical, runtime-diff, and
  store-deferred plan lowering in
  [crates/worth-query/src/policy_plan](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_plan)
- policy-aware execution seam identities, counters, deferred handoff honesty,
  and durable overclaim denials in
  [crates/worth-query/src/policy_execution_seam](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_execution_seam)
- policy-aware live admission, drift evidence, density evidence, and dense
  restart debt classification in
  [crates/worth-query/src/policy_live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_live)
- policy-aware delivery shape lowering and placeholder masking denial in
  [crates/worth-query/src/policy_delivery](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_delivery)
- concrete EmployeeRecord certification fixtures, scale-slope evidence, mask
  parity, composition parity, view-shape parity, and identity-aware inspector
  parity in
  [crates/worth-query/src/policy_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_certification)
- Milestone 9 certification in
  [crates/worth-query/src/harness/milestone_nine_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_nine_certification)
- public facade exposure for the admitted Milestone 9 surfaces in
  [crates/worth-query/src/facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/facade.rs)
- compile-fail proof boundaries in
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

The semantic center that now exists is:

one canonical query meaning can be admitted under explicit policy, tenant
truth, tenant schema, branch, mask, and relationship-proof bases; narrowed
before execution; lowered through current, branch, runtime historical, runtime
diff, live, delivery, saved, scope, template, and view-shape seams; and
certified through hostile rows without post-read redaction, host-local
authorization callbacks, ambient tenant filters, hidden unmasked live
maintenance, or fake durable claims.

## Acceptance Mapping

Milestone 9 is considered closed against:

- [milestone-9.md](./milestone-9.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [milestone-8-closeout.md](./milestone-8-closeout.md)

because the admitted runtime-backed policy, tenant, relationship-proof,
execution-seam, live, delivery, and certification surfaces now exist directly.

### `Policy, Tenant Schema, And Relationship-Proof Boundary Test`

Covered by:

- [crates/worth-query/src/harness/milestone_nine_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_nine_certification/mod.rs)
- [crates/worth-query/src/harness/milestone_nine_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_nine_certification/tests.rs)

What is proven:

- the named certification artifact exists as the Milestone 9 closeout suite
- required canonical rows are present and exercised, including policy/tenant
  admission, masked aspect removal, relationship-proof admission, current read,
  branch read, runtime historical read, runtime diff, live admission, delivery
  shape, saved/scope/template/view-shape parity, EmployeeRecord fixture
  evidence, tenant schema variation, mask parity, live drift/density posture,
  policy scale slope, and identity-aware inspector parity
- required rejection rows are present and exercised, including masked
  predicate/order/grouping/live/aggregation/cursor/view-membership influence,
  host-callback proof attempts, unbounded recursion, query-shape proof
  conflicts, unknown cost, branch denial, tenant schema mismatch, store-backed
  deferred execution, durable cursor/artifact/delivery deferred claims,
  placeholder masking, policy allocation, cross-tenant fanout, saved-policy
  bypass, and unsupported workflow composition
- the matrix proves equality, inequality, typed failure, zero-residue posture,
  declared hostile/parity lane semantics, and machine-checkable output fields
  rather than row presence alone

### `Policy and tenant admission before truth touch`

Covered by:

- [crates/worth-query/src/policy_basis](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_basis)
- [crates/worth-query/src/tenant_basis](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/tenant_basis)
- [crates/worth-query/src/policy_narrowing](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_narrowing)

What is proven:

- policy rule snapshots, branch access snapshots, tenant truth basis, and tenant
  schema basis are admitted before execution
- unknown policy cost, branch denial, tenant schema mismatch, and query-shape
  proof conflicts fail typed and early
- `PolicyMaskSnapshot` binds mask evidence to the admitted policy digest
- `NarrowedPolicyQueryArtifact` is the handoff artifact for optimizer, plan,
  live, and delivery seams
- saved-policy narrowing reuse cannot skip projection or proof drift checks

### `Aspect masking and hidden influence denial`

Covered by:

- [crates/worth-query/src/authorized_projection](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/authorized_projection)
- [crates/worth-query/src/policy_delivery](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_delivery)
- [crates/worth-query/src/policy_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_certification)

What is proven:

- masked projection fields are excluded from authorized projection rather than
  read and redacted after materialization
- masked predicate, ordering, grouping, derived-field, aggregation, cursor, and
  view-membership influence fail typed before narrowing
- non-disclosing predicate use remains explicit and does not admit unrelated
  masked influence
- placeholder masking is denied as an execution-seam/delivery contract failure,
  not as a late presentation convention
- masked and unmasked policy contexts produce distinct certified semantics
  where they should

### `Policy-aware execution seam and delivery parity`

Covered by:

- [crates/worth-query/src/policy_plan](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_plan)
- [crates/worth-query/src/policy_execution_seam](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_execution_seam)
- [crates/worth-query/src/policy_live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_live)
- [crates/worth-query/src/policy_delivery](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_delivery)

What is proven:

- current, branch, runtime-historical, runtime-diff, live, and delivery seams
  consume `NarrowedPolicyQueryArtifact`
- optimizer input cannot be derived from raw canonical, planned, or validated
  artifacts
- live policy drift and tenant-basis drift are certified against admitted live
  plan evidence
- live density posture binds authorized relevance width to admitted live
  relevance contracts
- delivery-width evidence is recomputed across scalar, narrow collection,
  grouped delta, and diff delta width classes
- store-backed and durable surfaces are explicitly denied or deferred instead
  of silently falling back to runtime semantics

### `Performance, boundedness, and support honesty`

Covered by:

- [crates/worth-query/src/policy_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/policy_certification)
- [crates/worth-query/src/harness/milestone_nine_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/milestone_nine_certification)

What is proven:

- policy work budgets and cost posture are part of admission evidence
- mask width, relationship-proof topology width, delivery width, live drift,
  live density, allocation, cross-tenant fanout, and scale-slope counters are
  certification-visible
- EmployeeRecord query families produce distinct scenario evidence rather than
  abstract labels
- policy scale-slope digest changes when structural counter slope drifts
- support diagnostics are row-derived and require executable evidence fields
  or typed rejection bundles, not just row names
- Phase 4 verified evidence uses machine digests instead of human
  "certified" labels

### `Compile-time enforcement and sealed boundaries`

Covered by:

- [crates/worth-query/tests/phase_boundaries_compile_fail.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/phase_boundaries_compile_fail.rs)
- [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

What is proven:

- admitted policy/tenant contexts, tenant truth identities, policy snapshots,
  mask snapshots, narrowed policy artifacts, validation reports, execution
  plans, live drift reports, delivery denials, EmployeeRecord bundles, and
  scale-slope reports cannot be fabricated through public constructors
- raw canonical/planned/validated artifacts cannot enter policy-aware plan,
  live, delivery, or optimizer surfaces
- masked projection artifacts cannot be substituted for authorized projection
  artifacts
- relationship-proof host callbacks and bare bool shortcuts remain forbidden
- saved-policy exact reuse cannot be minted from matching strings outside the
  query-owned narrowing path

## Final QA Outcome

The final reconciliation passes were run against:

- the Milestone 9 spec and implementation plans
- the WORTH Query test requirements document
- the Milestone 8 closeout surface that Milestone 9 must govern
- policy, tenant, authorized projection, relationship proof, narrowing, plan,
  live, delivery, certification, facade, and trybuild boundary surfaces
- the Milestone 9 certification matrix and Phase 4 support diagnostics

The last meaningful findings were:

- live drift evidence needed to bind current policy and tenant basis to the
  admitted live plan
- live density evidence needed to bind authorized relevance width to the
  admitted live relevance contract
- support diagnostics were too willing to trust row names instead of
  executable evidence
- certification tests still allowed label-like Phase 4 evidence and weak
  control lanes
- the certification bundle was missing `result_digest`, despite the test
  requirements demanding it

Those findings were corrected before this closeout.

After the correction passes, I do not see a remaining meaningful Milestone 9
gap for the admitted runtime-backed scope. The remaining gaps are explicit
store/durable scope, not hidden Milestone 9 implementation debt.

## Explicit Deferred Scope

Milestone 9 is closed for admitted runtime-backed policy-aware narrowing,
tenant scope, relationship-proof admission, execution seam lowering, live
admission, delivery contracts, saved/scope/template/view-shape policy parity,
and certification only.

The following remain explicit later work:

- store-backed policy execution parity and pushdown
- durable policy cursor resume
- durable policy artifact reload
- durable policy delivery metadata reload
- restart-stable subscription metadata
- durable tenant/query artifact portability
- persistence-backed saved policy/tenant artifact replay
- broader relationship-proof families beyond the admitted descriptor set
- broader tenant-schema/runtime integration once WORTH Store is ready

## What Later Milestones May Now Assume

Later milestones may safely assume:

- policy and tenant admission are structural query artifacts
- authorized projection is the execution-facing result of policy masking
- relationship proofs are query-authored descriptors with typed denial
  semantics
- policy-aware optimizer, plan, live, and delivery seams start from
  `NarrowedPolicyQueryArtifact`
- masked fields cannot be read and redacted later in admitted M9 paths
- tenant truth basis and tenant schema basis are paired, digest-bound, and
  certification-visible
- saved, scope, template, and view-shape paths are governed by the same policy
  and tenant narrowing model as direct construction
- certification already proves exact row coverage, typed denial coverage,
  machine evidence, counter visibility, and compile-time boundary enforcement
  for the admitted surface

Later milestones must not assume:

- WORTH Store already provides store-backed policy execution parity
- durable saved policy artifacts, cursors, or delivery metadata are restart
  safe
- store-backed historical/diff parity is admitted through M9 alone
- durable tenant/query portability exists without WORTH Store evidence
- host middleware may apply policy after ordinary plan lowering

## Verification Baseline

Milestone 9 closeout is grounded in:

- `cargo fmt -p worth-query`
- `cargo test -p worth-query milestone_nine_certification --lib`
- `cargo test -p worth-query policy_certification --lib`
- `cargo test -p worth-query --test phase_boundaries_compile_fail -- --test-threads=1`
- `cargo test -p worth-query`

These runs cover:

- policy/tenant admission and narrowing regression coverage
- authorized projection and relationship-proof regression coverage
- policy-aware plan, live, delivery, and execution-seam regression coverage
- EmployeeRecord fixture and performance/counter certification coverage
- Milestone 9 certification
- trybuild compile-fail proof boundaries

## Operational Conclusion

Milestone 9 is now the normative runtime-backed policy, tenant, relationship
proof, and delivery-contract surface for the admitted `worth-query` product
scope.

`worth-query` no longer depends on post-read redaction, ambient tenant filters,
host-local relationship-proof callbacks, unmasked live maintenance, delivery
metadata derived from raw truth, or row-name-only certification to express the
admitted Milestone 9 surface.

The remaining work is explicit WORTH Store and later-milestone scope, not
hidden architectural debt inside the Milestone 9 boundary.
