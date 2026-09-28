# Milestone 6 Closeout: Diagnostics And Explanation Ontology

Date: 2026-05-14

## Status

Milestone 6 is implementation-complete for `worth-foundational` through
Phase 7.

The crate now owns the shared diagnostics and explanation language for
diagnostic primitives, outcome and absence topology, family-distinct rows,
materialization/support-report law, canonical comparison bundles, proof-bearing
certified attachment compatibility, production-test readiness evidence, and
crate-facing feature documentation for the shipped diagnostics surface.

## Completed Surface

- Typed diagnostic code, scope, severity, artifact-kind, delivery-class, and
  availability vocabulary now exists and is crate-controlled.
- Outcome and absence families now remain mechanically distinct for accepted,
  advisory, denied, unsupported, deferred, partial, mismatch, violation,
  not-retained, redacted, reconstruction-denied, and missing-evidence cases.
- Diagnostic subjects and locators now reuse transition and boundary-artifact
  surfaces instead of recreating those meanings locally.
- Decision, failure, comparison, support, and provenance-ready rows are
  family-distinct and blind-consumer interpretable.
- Materialization now has explicit `plan(...)` and `materialize(...)` seams
  with visible availability, support posture, fallback debt, repeated
  rediscovery, and named-gap partiality.
- Reduced-richness diagnostic profiles now narrow breadth without mutating
  authoritative outcome meaning.
- Diagnostic bundles and support reports now lower through the Milestone 2
  canonical basis lane, with explicit comparison/mismatch bundles rather than
  bool-shaped parity.
- Certified diagnostic bundles now reuse the existing `worth-proof` lane for
  stronger current-basis and attachment claims instead of introducing a second
  proof substrate.
- Diagnostics readiness now exists as a proof-bearing artifact with exact
  certified surfaces, hostile pressures, compile-fail boundaries, canonical
  golden-artifact inventory, property-seed inventory, harness expansion points,
  runtime adoption assumptions, non-assumptions, downstream failure pressures,
  residual debt, and adoption-shaped followthrough.
- Crate-facing diagnostics docs now exist under
  [crates/worth-foundational/docs/diagnostics-and-explanation-ontology](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology),
  with one landing page and one feature doc per shipped capability seam.

## Phase Crosswalk

### Phase 1: Diagnostic Primitive And Category Law

Shipped homes:

- [primitives.rs](../../crates/worth-foundational/src/diagnostics/primitives.rs)
- [categories.rs](../../crates/worth-foundational/src/diagnostics/categories.rs)
- [primitives.rs test](../../crates/worth-foundational/tests/certification/diagnostics/primitives.rs)
- [ui/diagnostics/primitives](../../crates/worth-foundational/tests/ui/diagnostics/primitives)

What closed:

- canonical token validation for diagnostic code and scope ids
- typed severity, denial-class, breach-class, evidence-posture, delivery, and
  availability families
- artifact-kind definitions and explicit materialization legality
- compile-time non-substitution for primitive families and artifact-kind
  definitions

### Phase 2: Outcome, Subject, Locator, And Row Topology

Shipped homes:

- [outcomes.rs](../../crates/worth-foundational/src/diagnostics/outcomes.rs)
- [subjects.rs](../../crates/worth-foundational/src/diagnostics/subjects.rs)
- [rows/mod.rs](../../crates/worth-foundational/src/diagnostics/rows/mod.rs)
- [labels.rs](../../crates/worth-foundational/src/diagnostics/rows/labels.rs)
- [types.rs](../../crates/worth-foundational/src/diagnostics/rows/types.rs)
- [rows.rs test](../../crates/worth-foundational/tests/certification/diagnostics/rows.rs)
- [ui/diagnostics/rows](../../crates/worth-foundational/tests/ui/diagnostics/rows)

What closed:

- outcome and absence topology
- typed diagnostic subjects and typed locator wrappers
- family-distinct public row law
- omission-versus-denial separation for support rows
- explicit locality and widened-fallout meaning

### Phase 3: Materialization, Support Reports, And Named-Gap Partiality

Shipped homes:

- [materialization/mod.rs](../../crates/worth-foundational/src/diagnostics/materialization/mod.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/diagnostics/materialization/vocabulary.rs)
- [planning.rs](../../crates/worth-foundational/src/diagnostics/materialization/planning.rs)
- [surfaces.rs](../../crates/worth-foundational/src/diagnostics/materialization/surfaces.rs)
- [materialization.rs test](../../crates/worth-foundational/tests/certification/diagnostics/materialization.rs)
- [materialization_support.rs](../../crates/worth-foundational/tests/certification/diagnostics/materialization_support.rs)
- [ui/diagnostics/materialization](../../crates/worth-foundational/tests/ui/diagnostics/materialization)

What closed:

- explicit plan/materialize seam
- support-claim strength and availability posture
- fallback debt and repeated rediscovery as typed meaning
- typed named-gap partiality
- durable/certified support overclaim rejection

### Phase 4: Canonical Basis, Comparison, And Blind-Consumer Bundle Law

Shipped homes:

- [basis/mod.rs](../../crates/worth-foundational/src/diagnostics/basis/mod.rs)
- [canonical.rs](../../crates/worth-foundational/src/diagnostics/basis/canonical.rs)
- [comparison.rs](../../crates/worth-foundational/src/diagnostics/basis/comparison.rs)
- [entries.rs](../../crates/worth-foundational/src/diagnostics/basis/entries.rs)
- [row_entries.rs](../../crates/worth-foundational/src/diagnostics/basis/row_entries.rs)
- [tokens.rs](../../crates/worth-foundational/src/diagnostics/basis/tokens.rs)
- [basis.rs test](../../crates/worth-foundational/tests/certification/diagnostics/basis.rs)
- [basis_support.rs](../../crates/worth-foundational/tests/certification/diagnostics/basis_support.rs)
- [ui/diagnostics/basis](../../crates/worth-foundational/tests/ui/diagnostics/basis)

What closed:

- diagnostics-specific canonical basis domain participation
- explicit comparison/mismatch bundle law
- named-gap and evidence-posture canonicalization
- family-distinct blind-consumer row readers

### Phase 5: Certified Bundle And Attachment Compatibility Law

Shipped homes:

- [certified/mod.rs](../../crates/worth-foundational/src/diagnostics/certified/mod.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/diagnostics/certified/vocabulary.rs)
- [authority.rs](../../crates/worth-foundational/src/diagnostics/certified/authority.rs)
- [attachments.rs](../../crates/worth-foundational/src/diagnostics/certified/attachments.rs)
- [surfaces.rs](../../crates/worth-foundational/src/diagnostics/certified/surfaces.rs)
- [certified.rs test](../../crates/worth-foundational/tests/certification/diagnostics/certified.rs)
- [certified_support.rs](../../crates/worth-foundational/tests/certification/diagnostics/certified_support.rs)
- [ui/diagnostics/certified](../../crates/worth-foundational/tests/ui/diagnostics/certified)

What closed:

- proof-bearing certified diagnostic bundles
- typed coverage matrices and coverage classes
- current-basis transition and boundary-artifact attachment compatibility
- trust-boundary bridge and readmission reuse through `worth-proof`

### Phase 6: Production-Test Readiness

Shipped homes:

- [readiness/mod.rs](../../crates/worth-foundational/src/diagnostics/readiness/mod.rs)
- [authority.rs](../../crates/worth-foundational/src/diagnostics/readiness/authority.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/diagnostics/readiness/vocabulary.rs)
- [inventory.rs](../../crates/worth-foundational/src/diagnostics/readiness/inventory.rs)
- [report.rs](../../crates/worth-foundational/src/diagnostics/readiness/report.rs)
- [certification.rs](../../crates/worth-foundational/src/diagnostics/readiness/certification.rs)
- [readiness.rs test](../../crates/worth-foundational/tests/certification/diagnostics/readiness.rs)
- [ui/diagnostics/readiness_boundaries](../../crates/worth-foundational/tests/ui/diagnostics/readiness_boundaries)

What closed:

- proof-bearing diagnostics readiness artifact
- exact certified-surface, hostile-pressure, compile-fail, and worth-proof
  inventories
- canonical golden-artifact inventory and property-seed inventory
- harness expansion points, downstream failure pressures, and adoption-shaped
  followthrough
- runtime assumptions, non-assumptions, and residual debt

### Phase 7: Feature Documentation And Crate-Docs Integration

Shipped homes:

- [README.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/README.md)
- [diagnostic-primitives-and-categories.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-primitives-and-categories.md)
- [diagnostic-outcomes-subjects-and-rows.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-outcomes-subjects-and-rows.md)
- [diagnostic-materialization-and-support-reports.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-materialization-and-support-reports.md)
- [diagnostic-canonical-basis-and-comparison.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-canonical-basis-and-comparison.md)
- [certified-diagnostic-bundles-and-attachments.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/certified-diagnostic-bundles-and-attachments.md)
- [diagnostic-production-readiness.md](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-production-readiness.md)

What closed:

- crate-facing documentation now exists for every shipped Milestone 6
  capability seam
- the docs are organized by milestone feature, not by generic concept buckets
- the common descriptive lane, plan/materialize seam, canonical comparison
  lane, stronger certified lane, and readiness lane each have a stable public
  documentation home
- the docs are written against the actual shipped facade surface rather than
  milestone-only naming

## WORTH-Proof Standardized Lane

Milestone 6 uses `worth-proof` only where the spec required stronger claims.
Plain diagnostics vocabulary stayed local to `worth-foundational`.

Proof-bearing surfaces standardized here:

- certified diagnostic attachment authority in
  [attachments.rs](../../crates/worth-foundational/src/diagnostics/certified/attachments.rs)
- proof-bearing certified bundle carrier in
  [surfaces.rs](../../crates/worth-foundational/src/diagnostics/certified/surfaces.rs)
- trust-boundary bridge/readmission for stronger certified bundles in
  [surfaces.rs](../../crates/worth-foundational/src/diagnostics/certified/surfaces.rs)
- production-test readiness certification in
  [readiness/certification.rs](../../crates/worth-foundational/src/diagnostics/readiness/certification.rs)

Concrete `worth-proof` APIs the readiness artifact freezes:

- `AuthorityWitness::from_authority_marker`
- `Proof::from_authority_witness`
- `Artifact::with_proofs_and_current_basis`
- `bridge_trust_boundary`
- `readmit_with_authority`

Deliberately not moved into the proof kernel:

- primitive ids and categories
- outcome, absence, denial, breach, and evidence-posture vocabulary
- plain explanation/support/comparison rows and descriptive bundles
- materialization, richness, availability, and fallback-debt vocabulary

## Test-Requirements Mapping

Milestone 6 now satisfies the diagnostics-specific proof bar in
[plans/worth-foundational/test-requirements.md](./test-requirements.md).

### Primitive And Category Law

Evidence:

- [primitives.rs test](../../crates/worth-foundational/tests/certification/diagnostics/primitives.rs)
- [ui/diagnostics/primitives](../../crates/worth-foundational/tests/ui/diagnostics/primitives)

What is proved:

- primitive non-substitution
- canonical token law
- deterministic ordering
- explicit artifact-kind and materialization legality

### Row Topology And Blind-Consumer Law

Evidence:

- [rows.rs test](../../crates/worth-foundational/tests/certification/diagnostics/rows.rs)
- [ui/diagnostics/rows](../../crates/worth-foundational/tests/ui/diagnostics/rows)

What is proved:

- family-distinct row law
- subject and locator parity
- omission-versus-denial separation
- canonical row ordering

### Materialization, Richness, And Named-Gap Law

Evidence:

- [materialization.rs test](../../crates/worth-foundational/tests/certification/diagnostics/materialization.rs)
- [ui/diagnostics/materialization](../../crates/worth-foundational/tests/ui/diagnostics/materialization)

What is proved:

- reduced richness preserves truth
- availability/absence stays explicit
- fallback debt and repeated rediscovery remain typed
- durable/certified support cannot overclaim
- named-gap partiality stays typed and fail-closed

### Canonical Basis And Comparison Law

Evidence:

- [basis.rs test](../../crates/worth-foundational/tests/certification/diagnostics/basis.rs)
- [ui/diagnostics/basis](../../crates/worth-foundational/tests/ui/diagnostics/basis)

What is proved:

- independent-producer canonical parity
- semantic row-order tie breaking
- explicit mismatch-basis carriage
- blind-consumer family-distinct interpretation

### Certified Coverage And Attachment Law

Evidence:

- [certified.rs test](../../crates/worth-foundational/tests/certification/diagnostics/certified.rs)
- [ui/diagnostics/certified](../../crates/worth-foundational/tests/ui/diagnostics/certified)

What is proved:

- hostile coverage is mandatory for stronger certified claims
- typed named-gap partial coverage remains honest
- missing source digest and fake family coverage fail closed
- proof-bearing attachment reuses the existing `worth-proof` lane

### Readiness Closure

Evidence:

- [readiness.rs test](../../crates/worth-foundational/tests/certification/diagnostics/readiness.rs)
- [inventory.rs](../../crates/worth-foundational/src/diagnostics/readiness/inventory.rs)

What is proved:

- exact inventories for certified surfaces, hostile pressures, compile-fail
  boundaries, golden artifacts, property seeds, harness expansion points,
  assumptions, non-assumptions, downstream failure pressures, residual debt,
  and adoption followthrough
- one-to-one evidence rows for certified surfaces, hostile pressures,
  compile-fail boundaries, golden artifacts, property seeds, and harness
  expansion points

## Final QA Fixes

- Tightened primitive token law so empty diagnostic code/scope segments fail
  closed and callers cannot mint built-in artifact-kind definitions.
- Strengthened materialization law so fallback debt cannot be faked with
  zero-count row-scan entries and durable support cannot overclaim while
  surfacing no visible rows at the chosen richness tier.
- Tightened canonical row ordering so family-specific semantics participate in
  canonical sort order rather than preserving producer input order under ties.
- Tightened the readiness checklist so exact closure is enforced by the
  proof-bearing artifact itself rather than only by external tests.
- Expanded the diagnostics readiness artifact to satisfy the full shared
  production-test-readiness contract from
  [test-requirements.md](./test-requirements.md),
  including canonical golden artifacts, property seeds, harness expansion
  points, downstream failure pressures, and adoption-shaped followthrough.

## Proof Evidence

- Certification tests cover primitives, rows, materialization, basis,
  certified bundles, and readiness.
- Compile-fail tests prove primitive non-substitution, generic-row rejection,
  materialization seam preservation, comparison bundle boundaries, certified
  bundle boundaries, and readiness-only stronger claims.
- Canonical basis tests assert exact diagnostic row and bundle meaning rather
  than debug-string shadows.
- Misuse-pressure coverage now attacks generic-row collapse, omission-as-denial
  drift, hidden rediscovery debt, thin or empty support overclaim, hidden
  source-digest WORTHry, and explanation/provenance boundary collapse.
- Crate-facing feature docs now exist under
  [crates/worth-foundational/docs/diagnostics-and-explanation-ontology](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology),
  with one doc per shipped capability seam plus a landing page.

## Verification

The final broad gap-close pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational --test certification diagnostics::readiness -- --nocapture
cargo test -p worth-foundational --test compile_time_boundaries diagnostic_production_readiness_requires_certified_artifact -- --nocapture
cargo test -p worth-foundational
git diff --check
```

All passed.

## Explicit Deferrals

Milestone 6 is complete, but it still deliberately defers:

- Milestone 7 provenance/receipt deepening
- adopting-runtime lowering parity
- one generic diagnostics runtime, store, replay engine, or certification
  registry
- runtime-specific support taxonomies

Those remain explicit follow-on work. Milestone 6 closes the shared
diagnostics and explanation ontology plus the machine-facing readiness
contract that later documentation, provenance, and migration work must
consume.
