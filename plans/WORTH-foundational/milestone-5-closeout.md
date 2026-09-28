# Milestone 5 Closeout: Branching, Merging, And Commit Vocabulary

Date: 2026-05-13

## Status

Milestone 5 is implementation-complete for `worth-foundational`.

The crate now owns the shared transition language for branch-local candidates,
merge planning and verdicts, proof-bearing committed authority, commit
receipts, discard/closeout evidence, transition bundles, transition canonical
basis and locator participation, profile reuse, current-basis readmission, and
production-test readiness evidence.

Crate-facing transition docs now also exist under
[crates/worth-foundational/docs/branching-merging-and-commit-vocabulary](../../crates/worth-foundational/docs/branching-merging-and-commit-vocabulary),
with one landing page and one feature doc per shipped transition seam.

This milestone is ready for production-shaped testing through `worth-harness`
or adopting-crate migration work. It does not claim that any adopting runtime
has already lowered its real branch, merge, commit, receipt, or strategy
surfaces into the foundational transition language correctly.

## Completed Surface

- Typed branch-local vocabulary now exists for branch identity, candidate
  identity, fork basis, observation basis, fork observation basis, and
  comparison basis.
- Branch-local candidate and staged branch surfaces are mechanically distinct
  and remain explicitly non-authoritative.
- Merge planning now lowers staged branch work into typed merge candidates and
  typed merge verdicts with explicit strategy identity, descriptor digest,
  contract basis, correspondence basis, remap basis, merge basis, merge-base
  selection basis, and branch-basis drift.
- Merge admission now standardizes on `worth-proof::TransitionOutcome` instead
  of local verdict folklore.
- Committed authority now exists as a proof-bearing transition artifact with
  explicit authority-transition classes, structured no-op causes, canonical
  ordered parentage, merge ancestry basis, and committed delta summaries.
- Commit receipts now issue only from committed authority and carry a real
  receipt evidence floor rather than thin metadata.
- Non-authoritative discard/closeout evidence now has an explicit typed lane
  instead of disappearing into absence.
- Coordinated transition bundles now exist so committed authority, summary,
  report, and receipt can be emitted together without local result bags.
- Transition provenance rows now remain structured and blind-consumer
  interpretable rather than prose-only explanations.
- Transition surfaces now lower through the Milestone 2 canonicalization lane
  and reuse Milestone 3 profile attachment/materialization law instead of
  rebuilding either locally.
- Stronger current-basis transition claims now reuse the existing Milestone 2
  and Milestone 4 trust-boundary/readmission lane rather than a transition-
  local substitute.
- Milestone 5 production-test readiness now exists as a proof-bearing artifact
  with exact certified-surface inventory, hostile-pressure inventory,
  compile-fail inventory, `worth-proof` appendix, assumptions,
  non-assumptions, residual debt, and concrete evidence rows.

## Phase Crosswalk

### Phase 1: Branch-Local Separation

Shipped homes:

- [branches/mod.rs](../../crates/worth-foundational/src/transitions/branches/mod.rs)
- [branch_local.rs test](../../crates/worth-foundational/tests/certification/transitions/branch_local.rs)
- [ui/branch_local](../../crates/worth-foundational/tests/ui/transitions/branch_local)

What closed:

- branch-local identity and basis vocabulary
- candidate versus staged separation
- explicit non-authoritative branch-local state
- branch-local denial boundaries against stronger authority APIs

### Phase 2: Merge Verdict Law

Shipped homes:

- [merges/mod.rs](../../crates/worth-foundational/src/transitions/merges/mod.rs)
- [builder.rs](../../crates/worth-foundational/src/transitions/merges/builder.rs)
- [verdict.rs](../../crates/worth-foundational/src/transitions/merges/verdict.rs)
- [strategy.rs](../../crates/worth-foundational/src/transitions/merges/strategy.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/transitions/merges/vocabulary.rs)
- [merge_verdicts.rs test](../../crates/worth-foundational/tests/certification/transitions/merge_verdicts.rs)
- [ui/merge_admission](../../crates/worth-foundational/tests/ui/transitions/merge_admission)

What closed:

- typed merge candidate and merge verdict law
- strategy-bearing merge meaning
- explicit stale/deferred/rebind/failure topology through `TransitionOutcome`
- structural summary, conflict loci, and drift visibility

### Phase 3: Committed Authority Transition Law

Shipped homes:

- [commits/mod.rs](../../crates/worth-foundational/src/transitions/commits/mod.rs)
- [authority.rs](../../crates/worth-foundational/src/transitions/commits/authority.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/transitions/commits/vocabulary.rs)
- [committed_authority.rs test](../../crates/worth-foundational/tests/certification/transitions/committed_authority.rs)
- [ui/committed_authority](../../crates/worth-foundational/tests/ui/transitions/committed_authority)

What closed:

- proof-bearing committed authority admission
- transition classes and no-op causes
- canonical parentage and merge ancestry basis
- committed delta summaries and commit-eligibility denials

### Phase 4: Receipts, Reports, And Bundles

Shipped homes:

- [receipts/mod.rs](../../crates/worth-foundational/src/transitions/receipts/mod.rs)
- [issuance.rs](../../crates/worth-foundational/src/transitions/receipts/issuance.rs)
- [bundle.rs](../../crates/worth-foundational/src/transitions/receipts/bundle.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/transitions/receipts/vocabulary.rs)
- [receipts_and_bundles.rs test](../../crates/worth-foundational/tests/certification/transitions/receipts_and_bundles.rs)
- [ui/receipt_boundaries](../../crates/worth-foundational/tests/ui/transitions/receipt_boundaries)

What closed:

- proof-bearing receipt issuance from committed authority
- structured transition provenance rows
- non-authoritative discard/closeout evidence
- typed coordinated transition bundles

### Phase 5: Basis, Locators, Profiles, And Current-Basis Reuse

Shipped homes:

- [canonical.rs](../../crates/worth-foundational/src/transitions/basis/canonical.rs)
- [canonical_branch.rs](../../crates/worth-foundational/src/transitions/basis/canonical_branch.rs)
- [canonical_merge.rs](../../crates/worth-foundational/src/transitions/basis/canonical_merge.rs)
- [canonical_commit.rs](../../crates/worth-foundational/src/transitions/basis/canonical_commit.rs)
- [current_basis.rs](../../crates/worth-foundational/src/transitions/basis/current_basis.rs)
- [profiles.rs](../../crates/worth-foundational/src/transitions/basis/profiles.rs)
- [transition_locator.rs](../../crates/worth-foundational/src/locators/transition_locator.rs)
- [phase5_basis.rs test](../../crates/worth-foundational/tests/certification/transitions/phase5_basis.rs)
- [ui/phase5_boundaries](../../crates/worth-foundational/tests/ui/transitions/phase5_boundaries)

What closed:

- transition canonical basis lowering
- typed transition locators
- Milestone 3 profile reuse
- current-basis bridge/readmission reuse for transition artifacts

### Phase 6: Production-Test Readiness

Shipped homes:

- [readiness/mod.rs](../../crates/worth-foundational/src/transitions/readiness/mod.rs)
- [authority.rs](../../crates/worth-foundational/src/transitions/readiness/authority.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/transitions/readiness/vocabulary.rs)
- [inventory.rs](../../crates/worth-foundational/src/transitions/readiness/inventory.rs)
- [report.rs](../../crates/worth-foundational/src/transitions/readiness/report.rs)
- [certification.rs](../../crates/worth-foundational/src/transitions/readiness/certification.rs)
- [readiness.rs test](../../crates/worth-foundational/tests/certification/transitions/readiness.rs)
- [ui/readiness_boundaries](../../crates/worth-foundational/tests/ui/transitions/readiness_boundaries)

What closed:

- proof-bearing readiness artifact
- exact certified-surface, hostile-pressure, and compile-fail inventories
- real `worth-proof` appendix bound to implementation lanes
- runtime assumptions, non-assumptions, and residual debt

## WORTH-Proof Standardized Lane

Milestone 5 uses `worth-proof` where the spec required stronger claims and does
not pull plain transition vocabulary into the proof kernel.

Proof-bearing surfaces standardized here:

- merge admission through `worth-proof::TransitionOutcome` in
  [builder.rs](../../crates/worth-foundational/src/transitions/merges/builder.rs)
- committed-authority admission through
  [authority.rs](../../crates/worth-foundational/src/transitions/commits/authority.rs)
- receipt issuance through
  [issuance.rs](../../crates/worth-foundational/src/transitions/receipts/issuance.rs)
- current-basis trust-boundary bridge and readmission through
  [current_basis.rs](../../crates/worth-foundational/src/transitions/basis/current_basis.rs)
- production-readiness certification through
  [readiness](../../crates/worth-foundational/src/transitions/readiness)

Concrete `worth-proof` APIs the readiness artifact now freezes as the chosen
lane:

- `TransitionOutcome`
- `AuthorityWitness::from_authority_marker`
- `Proof::from_authority_witness`
- `Artifact::with_proofs_and_current_basis`
- `Artifact::with_current_basis`
- `bridge_trust_boundary`
- `readmit_with_authority`

Plain transition vocabulary deliberately stayed local:

- branch ids, merge ids, and commit ids
- strategy identity/family/version/ownership vocabulary
- basis, correspondence, and remap vocabulary
- branch-local, merge, parentage, and delta descriptive nouns

## Test-Requirements Mapping

Milestone 5 now satisfies the Milestone-5-specific bars that were added to
[plans/worth-foundational/test-requirements.md](./test-requirements.md).

### Branch-Local Versus Authority Separation

Evidence:

- [branch_local.rs test](../../crates/worth-foundational/tests/certification/transitions/branch_local.rs)
- [ui/branch_local](../../crates/worth-foundational/tests/ui/transitions/branch_local)

What is proved:

- branch-local candidates and staged state remain non-authoritative
- branch-local surfaces do not expose commit or receipt APIs

### Merge-Verdict Topology And Strategy Visibility

Evidence:

- [merge_verdicts.rs test](../../crates/worth-foundational/tests/certification/transitions/merge_verdicts.rs)
- [ui/merge_admission](../../crates/worth-foundational/tests/ui/transitions/merge_admission)

What is proved:

- merge candidates and verdicts remain distinct
- merge topology, strategy influence, and basis drift stay explicit
- merge surfaces cannot jump early into committed-authority or receipt lanes

### Committed Authority Proof Lane

Evidence:

- [committed_authority.rs test](../../crates/worth-foundational/tests/certification/transitions/committed_authority.rs)
- [ui/committed_authority](../../crates/worth-foundational/tests/ui/transitions/committed_authority)

What is proved:

- committed authority requires proof-bearing admission
- no-op, metadata-only, promotion, and ordinary commit classes remain distinct
- parentage and delta evidence are preserved

### Receipt Evidence Floor And Bundle Legality

Evidence:

- [receipts_and_bundles.rs test](../../crates/worth-foundational/tests/certification/transitions/receipts_and_bundles.rs)
- [ui/receipt_boundaries](../../crates/worth-foundational/tests/ui/transitions/receipt_boundaries)

What is proved:

- receipts can only issue from committed authority
- report-only or discard paths cannot fake receipt attestation
- coordinated bundles preserve category honesty

### Canonical Basis, Locator, And Current-Basis Parity

Evidence:

- [phase5_basis.rs test](../../crates/worth-foundational/tests/certification/transitions/phase5_basis.rs)
- [ui/phase5_boundaries](../../crates/worth-foundational/tests/ui/transitions/phase5_boundaries)

What is proved:

- semantically identical transition evidence lowers to one canonical meaning
- transition current-basis lanes remain proof-bearing and readmission-gated
- profile reuse does not reopen profile law locally

### Readiness Closure

Evidence:

- [readiness.rs test](../../crates/worth-foundational/tests/certification/transitions/readiness.rs)
- [inventory.rs](../../crates/worth-foundational/src/transitions/readiness/inventory.rs)

What is proved:

- certified surfaces, hostile pressures, compile-fail boundaries, `worth-proof`
  APIs, assumptions, non-assumptions, and debt are exact
- every hostile pressure and readiness-only boundary has one concrete evidence
  row

## Final QA Fixes

- Strengthened merge planning so source basis, target basis, and comparison
  basis cannot silently disagree with the actual branch pair being admitted.
- Preserved structural summaries on admitted merge verdicts so planning breadth
  and touched-scope evidence do not disappear after admission.
- Corrected report-only transition bundles so they no longer fake receipt
  attestation fields when no receipt was actually issued.
- Tightened the readiness artifact so hostile pressures and readiness-only
  compile-fail boundaries are backed by concrete evidence rows rather than
  exact-but-ceremonial inventory.
- Narrowed the readiness `worth-proof` appendix to the real shipped Milestone 5
  `Artifact` lane and removed the stray `rebind_with_authority` overclaim.

## Proof Evidence

- Certification tests cover branch-local separation, merge verdict law,
  committed authority, receipt issuance and bundle law, canonical basis and
  current-basis reuse, and Phase 6 readiness.
- Compile-fail tests prove branch-local, merge, receipt, current-basis, and
  readiness surfaces cannot impersonate stronger transition lanes.
- Blind-consumer style certification tests prove branch basis, merge basis,
  parentage, delta evidence, receipt attestation, provenance rows, and
  readiness inventories remain interpretable without producer-private state.
- Misuse-pressure tests now attack ambient basis choice, hidden strategy
  influence, thin receipts, generic transition-result bags, and cheap
  convenience bypasses.
- Topology checks show the touched transition production and test files remain
  under the 400-line cap through responsibility-shaped subdivision.
- Crate-facing feature docs now exist for branch-local, merge, strategy/basis,
  committed-authority, receipt/bundle, canonical/current-basis, and readiness
  transition seams.

## Verification

The final QA pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational --test certification transitions::readiness -- --nocapture
cargo test -p worth-foundational
git diff --check
```

All passed.

## Explicit Deferrals

Milestone 5 does not implement:

- diagnostics ontology
- lineage or provenance ontology beyond transition provenance rows and receipt
  evidence
- real adopting-runtime transition lowering parity
- strategy registries, hook executors, merge runtimes, or commit runtimes
- geometry kernels, remap engines, or correspondence engines
- Milestone 6 or any later roadmap milestone

Those remain downstream roadmap work. Milestone 5 closes the shared transition
language, proof lane, basis/locator integration, and readiness contract that
later diagnostics, provenance, and migration work must consume.
