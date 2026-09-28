# WORTH Query Milestone 9.6 Bridge Truth Identity Closeout

> **Status:** Phase 10 closeout **Closed** for worth-query bridge-truth scope
> (`query-repair`, 2026-06-16). **One named deferral:** worth-spatial
> `public_api_contract` integration harness → separate optimization agent
> ([phase-10-closeout-ledger.md](./phase-10-closeout-ledger.md) §Postponed).
>
> **Prior closeout date:** 2026-06-15
>
> **Governing law:** Architecture Law 42 and WORTH Foundational identity lifecycle categories.

## Phase 10 zero-deferral requirements

The 2026-06-15 closeout is superseded for final milestone closure by the Phase 10
pass on branch `query-repair`. Phase 10 must complete **all** items below before
status returns to `Closed`. See also `phase-10-closeout-ledger.md` and the Phase 10
section in `milestone-9.6-bridge-truth-identity-lowering.md`.

1. **Full compile-fail matrix** — every gate in Verification Gates below, plus all
   worth-query `phase_boundaries_*` suites and worth-topo Phase 8 + Phase 9 trybuild
   drivers.
2. **worth-topo Phase 9 compile-fail extension** — `query_runtime_phase_nine`
   manifest, harness folklore scan (no `PHASE_EIGHT_EXCLUDED` skips), UI fixtures.
3. **worth-runtime-bridge subscription replay** — migrate `replay_tests.rs` from
   label-based `truth_*_fixture` helpers to typed relational constructors; close
   matrix row in this milestone (no separate owner milestone).
4. **worth-spatial `public_api_contract`** — **postponed** to separate
   worth-spatial optimization agent (harness performance/flake under parallel;
   serial gate intended; `cargo test -p worth-spatial --lib` 72/72 green).
   Does not block WS-6–8; blocks Phase 12 final 9.6 `Closed`.
5. **Phase 9 closeout evidence** — append gate results from `query-repair` hostile
   QA and compile-fail runs to this document.
6. **Compiler Failure Ledger** — no open in-scope rows without explicit fix path.

## Closure Summary

Milestone 9.6 closed the bridge truth identity seam by making identity authority a typed lifecycle property instead of a representation convention. Relational, signal, runtime bridge, Query, server, worth-topo, hadwiger-research, and worth-ui in-scope paths now either preserve typed authority/evidence, quarantine terminal projections, or are explicitly deferred to an owner milestone.

The remaining allowed strings are terminal reporting, diagnostics, JSON compatibility input, or historical certification labels. They do not satisfy current authority APIs.

## Authority Model

- Current authority must be carried by the owning typed identity or admitted owner witness.
- Boundary-crossed values must be bridged or revalidated before they can satisfy current authority.
- External tokens, digest evidence, projection labels, and raw representations cannot enter authority APIs.
- Reporting accessors are terminal and may not feed composition, lookup, admission, routing, or coherence checks.
- Bridge writeback/naming/route identities no longer expose raw `from_external_authority_evidence` constructors; callers must provide typed bridge evidence.

## Final QA Findings

1. **Certification row digests still used reporting references.** Fixed by hashing retained `reference_receipt_evidence_identity()` values in causal certification row-digest collection and named-slot composition.
2. **Bridge writeback/naming/route facade constructors still accepted raw evidence text.** Fixed by deleting public `from_external_authority_evidence` constructors and requiring `BridgeIdentityEvidence`.
3. **Mutation batch tests used fake raw bridge evidence and one non-relational resolved-target label.** Fixed by deriving test evidence from typed Query identities and using relational bridge record identities where the test asserts resolved-target variation.
4. **Closeout state was stale.** Fixed by adding this closeout and updating the milestone status/checklists.

## Post-Closeout QA Correction

The 2026-06-15 follow-up QA found additional projection re-entry in the
causal/subscription certification edge after the initial closeout. The fix
keeps report strings terminal and moves certification composition back onto
typed evidence identities:

- `subscription/runtime_certification/scope.rs` compares support, parity,
  diagnostic, lifecycle, and coverage handles through retained
  `WORTHQueryEvidenceIdentity` values and seals the scope digest through the
  evidence encoder.
- `runtime/inspection/causal/certification/*` now seals hostile rows,
  boundary audit, representative matrix rows, named evidence slots,
  reference collections, row-digest sets, proof-shape digests, certification
  scope, and certification bundle through `WORTHQueryEvidenceIdentity`
  composition instead of `hash_parts` over report strings.
- `query_basis_lifecycle/binding_evidence.rs` uses `field_bridge_identity`
  for retained bridge evidence rather than projecting `BridgeIdentityEvidence`
  through `as_str()`.

## Eight-Point QA Closure

1. `subscription/runtime_certification/scope.rs` no longer composes scope
   identity from `certification_bundle_for_reporting()` or `hash_parts`; it
   compares and seals retained typed evidence identities.
2. `SubscriptionLifecycleCertificationBundle` no longer stores parallel
   `*_for_reporting: String` fields for artifacts that already have typed
   lifecycle identities. Reporting accessors project from typed identities, and
   the bundle itself carries a `QuerySubscriptionAuthorityIdentity`.
3. Causal hostile rows no longer consume `artifact_for_reporting()`; artifact
   rows seal retained artifact evidence identities.
4. Causal row-digest slots and reference collections no longer join
   `reference_receipt_evidence_identity().as_str()` strings; they seal retained
   reference identities through the evidence encoder.
5. Query lower-runtime binding evidence no longer stringifies
   `BridgeIdentityEvidence`; both `binding_evidence.rs` and `raw_identity.rs`
   use `field_bridge_identity`.
6. WORTH Foundational authority categories are wired into production Query
   carriers: subscription certification bundles carry
   `QuerySubscriptionAuthorityIdentity`, causal certification bundles carry
   `QueryCausalInspectionAuthorityIdentity`, and lower-runtime basis bindings
   carry `QueryFeederAuthorityIdentity`.
7. The named Phase-7-adjacent residuals were cleaned: query-basis lifecycle
   digest construction no longer uses `hash_parts`; subscription certification
   errors and runtime-certification errors seal evidence identities; declaration
   bridge routing digest projections use value fields; digest wrappers no
   longer re-enter digest text as identity fields.
8. Matrix row 1055 no longer contradicts itself by marking the row `Fixed`
   while preserving an open "remaining risk" note.

## Verification Gates

### Phase 10 full matrix (required before final close)

**Workspace**

- `cargo check --workspace`
- `cargo check -p worth-runtime-bridge -p worth-query --lib`

**Compile-fail (all must pass)**

- `cargo test -p worth-runtime-bridge --test phase_boundaries_compile_fail`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_bridge_truth_identity_compile_fail`
- `cargo test -p worth-runtime-bridge --test phase_boundaries_bridge_truth_identity_digest`
- `cargo test -p worth-query --test phase_boundaries_bridge_truth_identity_compile_fail`
- `cargo test -p worth-query --test phase_boundaries_query_identity_authority_compile_fail`
- `cargo test -p worth-query --test phase_boundaries_intent_admission_compile_fail`
- `cargo test -p worth-query --test phase_boundaries_compile_fail`
- All other `cargo test -p worth-query --test phase_boundaries_*` suites
- `cargo test -p worth-topo --test phase_boundaries_query_runtime_phase_eight_compile_fail`
- `cargo test -p worth-topo --test phase_boundaries_query_runtime_phase_nine_compile_fail`
- `cargo test -p worth-relational --test phase_boundaries_compile_fail`
- `cargo test -p worth-signal --test phase_2a_signal_boundaries`

**Integration / certification**

- `cargo test -p worth-spatial --test public_api_contract -- --test-threads=1`
  (serial harness required; default parallel run has shared-state flake unrelated to
  identity authority — see phase-10-closeout-ledger P10-4)
- `cargo test -p worth-spatial --lib`
- `cargo test -p worth-topo --lib topology_read`
- `cargo test -p worth-server --test WORTH_native_facade_entry`
- `cargo test -p worth-server --test compat_http_phase_three`
- `cargo test -p worth-runtime-bridge subscription::replay --lib`

**Phase 7 certification lanes (2026-06-15 baseline — re-run for Phase 10)**
- `cargo test -p worth-query runtime::surface::mutation_evidence::batch --lib`
- `cargo test -p worth-query runtime::tests::causal_inspection::certification::row_digest --lib`
- `cargo test -p worth-query runtime::tests::causal_inspection::certification --lib`
- `cargo test -p worth-query subscription::tests::runtime_certification --lib`
- `cargo test -p worth-query query_basis_lifecycle --lib`
- `cargo fmt --check --all`

Evidence files for the final pass are stored under `plans/worth-query/goal_mode_final_*` and `plans/worth-query/goal_mode_bridge_*`.

## Deferred Scope (Phase 10)

**Closed in Phase 10:**

- ~~Subscription replay typed identity fixture cleanup~~ → **P10-3** ✓
- ~~worth-topo Phase 9 compile-fail extension~~ → **P10-2** ✓

**Postponed (named owner — not worth-query 9.6 WS-6+):**

- worth-spatial `public_api_contract` integration harness → **P10-4** — separate
  worth-spatial optimization agent; see [phase-10-closeout-ledger.md](./phase-10-closeout-ledger.md)
  §Postponed

**Still out of scope:** ordinary local/display formatting in worth-kernel and
WORTH-kernel unless a trace finds an authority path.
