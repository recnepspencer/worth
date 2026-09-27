# Milestone 9.8 Engineering Spec: Downstream Consumer Product Kit For Evidence Reports, Boundary Audits, And Support Pinning

> **Status:** Closed
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Primary predecessors:** [milestone-9.7.md](./milestone-9.7.md), [milestone-9.6.md](./milestone-9.6.md), [milestone-9.5.md](./milestone-9.5.md)
>
> **Purpose:** ship the runtime-owned kit that eliminates consumer-side
> folklore around Query's product contracts â€” declarative evidence-report
> scaffolding, a shipped boundary-bypass audit, exportable, pinnable support
> snapshots, and a shipped in-memory consumer test backend â€” and prove
> closure by reference-consumer adoption rather than by API presence.

## Goal

Make the cost of consuming Query correctly lower than the cost of consuming
it wrong. Today every downstream evidence report costs hundreds of lines of
hand-rolled struct, getter, error, and digest plumbing; the no-bypass rule is
enforced by consumer-written source greps; and support posture is re-derived
per consumer as hand-built gap rows. Each is a runtime-owned contract
materialized by consumer folklore. This milestone ships the kit that owns
them, and closes only when the reference consumer (`worth-kernel`) has
deleted its hand-rolled equivalents.

## Why This Milestone Exists

The first serious consumer pays a linear ceremony tax that masks how steep
the adoption curve is for every consumer after it:

- current `worth-kernel` construction support still repeats report struct,
  getter, local error/display, and digest plumbing across moved support
  surfaces such as branch-preview basis, projection-consumption parity,
  family coverage, compound parity support, and corpus replay support
- the Query hard prohibitions are enforced downstream by a consumer-owned
  audit that `include_str!`s 27 source files and greps for forbidden
  patterns like `.write(` â€” charming, unshippable, and reinvented (or
  skipped) by every future consumer
- support posture reaches consumers as hand-assembled gap rows with no
  serialized, versioned, diffable snapshot a downstream CI can pin, so a
  posture regression surfaces at runtime admission instead of at build time

Milestones `9.6` and `9.7` harden what Query *says*; this milestone hardens
what Query *gives consumers to build with*. Per `MENTALITY.md`, foundations
are slow and features are fast â€” this kit is what makes downstream features
fast.

## Current Reference-Consumer Debt Refresh

The original reference-consumer examples named earlier `worth-kernel`
`runtime_proof/` paths. Those files have since moved or been deleted, but the
same Query-consumption debt still exists in the current construction tree.
Milestone `9.8` must target the current surfaces, not the stale illustrative
paths.

The current debt census for
[crates/worth-kernel/src/construction](../../crates/worth-kernel/src/construction)
found:

- 81 local digest helper matches across 24 files
- 19 local report or row structs across 14 files
- 20 local error/display plumbing matches across 9 files
- 287 source-string audit matches across 15 files
- 4 hand-built Query support admission/gap-posture matches across 3 files
- zero current construction-tree matches for hand-implemented
  `WORTHQueryRuntime*Adapter` traits or hand-fabricated bridge receipts

The live high-value reference surfaces are:

- [tests/support/branch_preview_basis.rs](../../crates/worth-kernel/src/construction/tests/support/branch_preview_basis.rs):
  local Query-shaped report, local error/display, getter wall, and
  `digest_owned_parts` report digest assembly
- [digest_protocol.rs](../../crates/worth-kernel/src/construction/digest_protocol.rs):
  local construction digest sidecar; legitimate for worth-owned artifact
  identity only, not for Query evidence/support/report identities
- [authoring.rs](../../crates/worth-kernel/src/construction/authoring.rs):
  `REQUIRED_QUERY_FAMILIES` plus a local loop over
  `workspace.admit_public_api_family(...)`
- [tests/phase_eight_minimization.rs](../../crates/worth-kernel/src/construction/tests/phase_eight_minimization.rs),
  [tests/boundary.rs](../../crates/worth-kernel/src/construction/tests/boundary.rs),
  and [tests/boundary_phase_five/patterns.rs](../../crates/worth-kernel/src/construction/tests/boundary_phase_five/patterns.rs):
  current source-string audit clusters; some rules are worth-domain hygiene,
  but the Query-prohibition subset is Query-owned and belongs in this kit

The in-memory backend remains a valid Query consumer-kit surface, especially
for external consumers and future downstream test harnesses. It should not be
overclaimed as the primary current `worth-kernel` construction debt unless a
fresh audit finds hand-implemented Query adapters or hand-fabricated receipts
there. The reference-consumer adoption proof for `worth-kernel` must focus on
the live debt it actually has now: report/digest folklore, Query-prohibition
source scans, and support-pinning ceremony.

## Governing Summaries

- `MENTALITY.md`: enforce mechanically, not by convention â€” a prohibition
  list enforced by consumer greps is category-3 enforcement owned by the
  wrong party; the kit moves enforcement to the runtime that owns the rule.
- `arch_laws.md`: Law 6 (domain code returns what changed; the framework
  derives the ceremony), Law 26 (explicit equivalence contracts), Law 34
  (the framework owns resource lifecycle â€” including the lifecycle of its
  own contract enforcement), Law 41 (sealed proof-carrying types).
- `composition_laws.md`: the kit must produce named, predictable consumer
  files â€” a derive that hides meaning would trade boilerplate for fog; the
  kit names responsibilities, it does not bury them.
- `domain_structure_laws.md`: shared code must earn its shared location by
  shared authority â€” these three surfaces qualify precisely because every
  consumer depends on the same contracts for the same semantic reasons.
- `perf_laws.md`: structural waste dominates constant waste; the per-consumer
  ceremony tax is structural waste at the platform boundary, and the kit
  amortizes it across the largest semantically honest boundary â€” the runtime
  itself.
- `worth_query_roadmap.md`: the platform framework stance says ordinary
  developers stay inside `worth-query` for the majority of their work; that
  claim is dishonest while correct consumption requires folklore.

## Adversarial Constraint

A downstream domain crate must be able to author a digest-bearing evidence
report, enforce the no-bypass contract, pin its support-posture
dependencies, and obtain a valid honestly-postured test runtime using only
Query-shipped kit surfaces â€” and every divergence class must fail
mechanically in the consumer's build: a report field that escapes digest
participation, a bypass of a prohibited runtime seam, a support-posture
regression against a pinned row, a test backend faking a lane it cannot
honor, or a resurrection of hand-rolled scaffolding in covered consumer
surfaces.

This milestone fails if any covered path:

- ships scaffolding that produces digests outside the Milestone `9.6`
  canonical evidence-identity scheme
- enforces the bypass rules by string search where visibility or type
  boundaries can enforce them structurally
- exports a support snapshot that can disagree with the live support matrix
  for the same runtime version
- claims closure while the reference consumer still carries hand-rolled
  report plumbing, its grep audit, or hand-built gap rows in covered
  surfaces

## Product Decision Lock

- The kit is Query product surface, not test convenience; it ships with
  support rows, docs, and certification like every other family.
- Report scaffolding is declarative per arch law 6: consumers declare fields
  and semantics; the kit derives digest participation, accessors, and
  error/display plumbing.
- All kit-produced identity lowers through the `9.6` canonical
  evidence-identity primitive; the kit cannot mint a second digest scheme.
- Bypass enforcement prefers visibility and sealed types; the shipped audit
  artifact covers only what structure cannot reach, and it is runtime-owned.
- Support snapshots are serialized, versioned, schema-stable projections of
  the same support matrix the runtime answers from â€” one truth, derived
  views, per arch law 33.
- Closure is adoption-proven: the reference consumer deletes its folklore in
  the same change program, per the `MENTALITY.md` scope-expansion rule.

## Phase Plan

### Phase 1: Evidence Report Composition Kit Boundary

Ship the declarative evidence-report scaffolding: a consumer declares a
report's fields, semantic scope, and digest participation once; the kit
derives canonical digest construction, accessors, and error/display plumbing
over the `9.6` primitive.

**Relevant subsystems**
- consumer kit (new boundary home inside `worth-query`)
- canonical evidence identity (from Milestone `9.6`)

**Relevant Query source surfaces**
- the Milestone `9.6` evidence-identity primitive surface
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs) as the
  house example of the report shape the kit must be able to express

**Relevant downstream evidence (reference consumer)**
- [worth-kernel tests/support/branch_preview_basis.rs](../../crates/worth-kernel/src/construction/tests/support/branch_preview_basis.rs)
- [worth-kernel tests/support/projection_consumption.rs](../../crates/worth-kernel/src/construction/tests/support/projection_consumption.rs)
- [worth-kernel tests/support/family_coverage.rs](../../crates/worth-kernel/src/construction/tests/support/family_coverage.rs)
- [worth-kernel tests/support/compound_parity_support.rs](../../crates/worth-kernel/src/construction/tests/support/compound_parity_support.rs)
- [worth-kernel tests/support/corpus_replay_digest.rs](../../crates/worth-kernel/src/construction/tests/support/corpus_replay_digest.rs)

**Target shape (illustrative, not frozen API)**

The reference consumer's reports today are ~250â€“350 lines each: a report
struct, a getter wall, hand-rolled digest plumbing, and a three-variant error
enum with a hand-written `Display` â€” repeated nearly verbatim per report
file. The target shape collapses each to its semantic core:

```rust
// AFTER: declare fields and semantics once; digest participation, accessors,
// sealed construction, and error/display plumbing are derived â€” and the
// digest is canonical-scheme (9.6) by construction
#[derive(WORTHQueryEvidenceReport)]
#[evidence(scope = "worth-kernel.basis-lane")]
pub struct PrimitiveConstructionRuntimeBasisLaneReport {
    label: WORTHQuerySessionLabel,
    effect_policy: WORTHQueryEffectPolicy,
    authority_lane: WORTHQueryAuthorityLane,
    #[evidence(sequence)]
    evidence: Vec<String>,
}
// a field missing its digest-participation posture does not silently skip
// the digest â€” it fails to compile
```

**Warnings**
- Do not generate fog: derived accessors and digest participation must remain
  predictable from the declaration site per composition law 13; a macro that
  hides what enters the digest is worse than the boilerplate it replaces.
- Do not let the kit accept pre-rendered strings as digest contributions;
  field contributions are typed, tagged values per the `9.6` contract.
- Do not couple the kit to `worth-kernel` shapes; it must express Query's own
  report families first.

**Test requirements**
- Add a `Consumer Evidence Report Kit Parity Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: re-express a representative existing report
  (one Query-owned, one `worth-kernel`-shaped) through the kit and prove the
  kit-built report carries identical semantic content and canonical-scheme
  digests for identical inputs.
- Adversarial current-reference proof: re-express
  `BranchPreviewBasisReport` through the kit and prove the kit-built report
  carries identical semantic fields and a canonical-scheme digest for
  identical inputs.
- Adversarial Query-owned digest closure: a covered report whose identity is
  Query evidence must lower through `WORTHQueryEvidenceIdentity`; it must not
  call `worth-kernel`'s `digest_owned_parts`,
  `digest_owned_parts_with_scope`, or `ConstructionDigestScope`.
- Adversarial field inventory: the report declaration emits a
  `report_field_inventory_digest` and `digest_participation_digest`; omitting
  a field from digest participation fails to compile or constructs a typed
  rejection rather than silently changing equivalence.
- Adversarial rejection: prove a field that is declared but excluded from
  digest participation, or mutated after construction, cannot silently alter
  the report digest â€” the misuse fails at compile time or constructs a typed
  rejection, never a divergent digest.

**Engineering decisions**
- The kit's declaration surface is data-shaped (declaration structs or a
  derive over them), not closure-shaped, so reports stay inspectable.
- Kit-built reports are sealed after construction per arch law 41.

**Open questions**
- None.

### Phase 2: Prohibition Registry And Seam Visibility Boundary

Freeze the single source of truth for the hard prohibitions: one runtime-
owned prohibition registry, with every coverable seam sealed by visibility or
typed admission so the strongest enforcement tier carries as much of the rule
set as structure allows.

**Relevant subsystems**
- prohibition registry (new boundary home inside `worth-query`)
- runtime seam visibility

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs) (covered seams:
  `write`, `batch`, existing-truth binding and probe surfaces named by the
  Milestone `9.5` Phase 4 seam-retirement list)

**Relevant downstream evidence (reference consumer)**
- [worth-kernel tests/boundary.rs](../../crates/worth-kernel/src/construction/tests/boundary.rs)
- [worth-kernel tests/phase_eight_minimization.rs](../../crates/worth-kernel/src/construction/tests/phase_eight_minimization.rs)
- [worth-kernel tests/boundary_phase_five/patterns.rs](../../crates/worth-kernel/src/construction/tests/boundary_phase_five/patterns.rs)
- [worth-kernel certification/phase_five_boundary_closeout_tests.rs](../../crates/worth-kernel/src/construction/certification/phase_five_boundary_closeout_tests.rs)

**Warnings**
- Do not solve this entirely with the Phase 3 audit; every seam that can be
  sealed by visibility or typed admission must be, and the audit covers only
  the remainder per the `MENTALITY.md` enforcement hierarchy.
- Do not let the registry and the hard-prohibitions documentation drift; the
  registry is the single source of truth both derive from.

**Test requirements**
- Add a `Prohibition Registry And Seam Visibility Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial localization: a compile-fail contract proving a consumer crate
  cannot reach a sealed covered seam at all â€” the strongest enforcement tier
  doing its job before any audit exists.
- Adversarial agreement: prove the hard-prohibitions documentation and the
  registry name exactly the same seam set, so adding a prohibition in prose
  without a registry entry fails a Query-side completeness assertion.

**Engineering decisions**
- The prohibition registry is the single source of truth; docs, visibility
  decisions, and the Phase 3 audit all derive from it.
- Seam sealing is preferred over audit coverage wherever the type or
  visibility system can express the rule.

**Open questions**
- None.

### Phase 3: Shipped Bypass Audit Artifact Boundary

Ship the audit artifact for the residue structure cannot reach: derived from
the Phase 2 registry, structurally resolving (path-resolved usage, not text),
and consumable from any downstream crate's test suite as one call.

**Relevant subsystems**
- shipped audit artifact (new boundary home inside the consumer kit)
- prohibition registry (consumed from Phase 2)

**Relevant Query source surfaces**
- the Phase 2 prohibition registry surface

**Relevant downstream evidence (reference consumer)**
- [worth-kernel tests/boundary.rs](../../crates/worth-kernel/src/construction/tests/boundary.rs)
- [worth-kernel tests/phase_eight_minimization.rs](../../crates/worth-kernel/src/construction/tests/phase_eight_minimization.rs)
- [worth-kernel tests/boundary_phase_five/patterns.rs](../../crates/worth-kernel/src/construction/tests/boundary_phase_five/patterns.rs)

**Target shape (illustrative, not frozen API)**

The consumer-owned enforcement this phase replaces, as it exists today in
`worth-kernel`:

```rust
// BEFORE: 27 include_str!'d source files string-grepped for forbidden
// patterns â€” trips on comments, misses aliased calls, reinvented per consumer
let violation_count = AUDITED_FILES
    .iter()
    .flat_map(|(_, source)| {
        FORBIDDEN_RUNTIME_PATTERNS
            .iter()
            .map(|pattern| source.contains(pattern))
    })
    .filter(|found| *found)
    .count();
```

The target shape after this phase:

```rust
// AFTER: one runtime-shipped, structurally-resolving audit; the consumer's
// entire enforcement file becomes one test
#[test]
fn worth_kernel_stays_on_the_sanctioned_query_path() {
    worth_query::kit::boundary_audit()
        .covering_crate("worth-kernel")
        .assert_clean();
    // typed findings name the seam and call site; comments, doc examples,
    // and string literals cannot trip it
}
```

**Warnings**
- Do not ship an audit that trips on comments, doc examples, or string
  literals; classification must be structural (path-resolved usage), not
  textual.
- Do not let the audit grow its own pattern set beside the registry; coverage
  completeness is derived, never hand-curated.
- The Phase 3 shipped slice is a `syn` AST audit, not a compiler-backed type
  resolver. Method-call detection is honest as `ast-method-name-resolved`:
  it catches executable calls whose method names match registry public symbols
  and avoids comments, doc attributes, and string literals. Associated-path
  detection is stronger: it matches the registry public-symbol path suffix
  such as `WORTHQueryWorkspace::write`, including fully qualified paths that
  end in that suffix, while rejecting unrelated types with the same final
  method name. Type aliases, trait dispatch, and macro expansion remain outside
  this Phase 3 mechanism and must not be described as closed until a
  compiler-backed resolver is added.

**Test requirements**
- Add a `Shipped Query Bypass Audit Honesty Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial detection: seed a covered bypass (a prohibited seam usage in a
  fixture consumer crate) and prove the shipped audit fails the build with a
  typed finding naming the seam and site.
- Adversarial false-positive honesty: prove a comment mention, doc example,
  and string literal containing the forbidden pattern do not trip the audit.
- Adversarial drift localization: prove that adding a seam to the Phase 2
  prohibition registry without audit coverage fails a Query-side
  completeness test.
- Adversarial mechanism honesty: the audit must resolve real Rust usage where
  possible. If an implementation slice uses a `syn`/source inventory before a
  fuller compiler-backed resolver exists, the spec must name which classes are
  structurally resolved and which are temporary source-inventory checks; source
  matching alone cannot close a prohibition class that visibility or typed
  path resolution can cover.

**Engineering decisions**
- The audit artifact is versioned with the runtime so consumers cannot pin a
  stale rule set silently.
- Audit findings are typed artifacts naming seam and call site, suitable for
  the same evidence handling as every other kit output.

**Open questions**
- None.

### Phase 4: Support Snapshot Projection Boundary

Ship the serialized, versioned, schema-stable support snapshot: a derived,
digest-bound projection of the live support matrix that downstream tooling
and CI can store, diff, and reason about outside a running workspace.

**Relevant subsystems**
- support matrix projection (new boundary home inside the consumer kit)

**Relevant Query source surfaces**
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs)
- [runtime/support/profile.rs](../../crates/worth-query/src/runtime/support/profile.rs)
- [application/support/report.rs](../../crates/worth-query/src/application/support/report.rs)

**Warnings**
- Do not create a second support truth; the snapshot is a derived projection
  of the same matrix the runtime answers admission from, digest-bound to it.
- Do not freeze the snapshot schema informally; schema identity is versioned
  and participates in the snapshot digest.

**Test requirements**
- Add a `Support Snapshot Projection Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: prove snapshot content equals live matrix content
  for the same runtime version â€” row for row, posture for posture, digest
  for digest â€” and that re-export is deterministic.
- Adversarial denial: prove comparing or loading a snapshot across a schema
  version boundary fails typed rather than silently coercing rows.

**Engineering decisions**
- Snapshot identity lowers through the `9.6` evidence primitive.
- The snapshot is a projection per arch law 33: destroyable and rebuildable
  from the matrix alone.

**Open questions**
- None.

### Phase 5: Consumer Pinning Contract Boundary

Ship the typed consumer pinning contract over the Phase 4 snapshot: a
downstream crate declares the rows and postures it depends on, and its own
build fails with a localized typed finding when posture regresses.

**Relevant subsystems**
- consumer pinning contract (new boundary home inside the consumer kit)
- support snapshot projection (consumed from Phase 4)

**Relevant Query source surfaces**
- the Phase 4 snapshot projection surface

**Relevant downstream evidence (reference consumer)**
- [worth-kernel authoring.rs](../../crates/worth-kernel/src/construction/authoring.rs)
  (hand-built required-family admission)
- [worth-kernel tests/phase_eight_minimization.rs](../../crates/worth-kernel/src/construction/tests/phase_eight_minimization.rs)
  (current residue guard for `PrimitiveConstructionQueryGapRow`)

**Target shape (illustrative, not frozen API)**

The hand-rolled posture bookkeeping this phase replaces, as it exists today
in `worth-kernel`, is the `REQUIRED_QUERY_FAMILIES` constant plus local
runtime-admission loop in
[authoring.rs](../../crates/worth-kernel/src/construction/authoring.rs).
Older gap-row machinery has already been mostly deleted; current tests still
guard against `PrimitiveConstructionQueryGapRow` resurrection. The pinning
surface should make both patterns unnecessary:

```rust
// BEFORE
const REQUIRED_QUERY_FAMILIES: [WORTHQueryRuntimeFacadeFamily; 2] = [
    WORTHQueryRuntimeFacadeFamily::Write,
    WORTHQueryRuntimeFacadeFamily::Inspect,
];
// ...followed by hand-built gap-row assembly over the public api contract
```

The target shape after this phase:

```rust
// AFTER: a typed pin artifact; a posture regression in worth-query fails the
// consumer's build with a finding naming the row â€” not a runtime admission
// surprise
worth_query::kit::support_pins! {
    workspace = "worth-kernel",
    require = [Write: Supported, Inspect: Supported],
    observe = [BranchPreview], // surfaces as gap rows automatically when unsupported
}
```

**Warnings**
- Do not make pinning advisory; a pinned row whose posture regresses must
  fail the consumer build, not log a warning.
- Do not let pins reference rows by free-form strings; pin declarations bind
  to typed row identity from the snapshot schema.

**Test requirements**
- Add a `Worth-Kernel Support Pinning Drift Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial drift localization: regress one covered row's posture in a
  fixture runtime and prove exactly the consumers pinned to that row fail,
  with a typed finding naming the row, the pinned posture, and the actual
  posture â€” and no unpinned consumer fails.
- Adversarial rejection: prove a pin against a nonexistent row, or a pin
  whose declared posture vocabulary does not match the snapshot schema
  version, fails typed at pin evaluation rather than passing vacuously.
- Adversarial replacement: the covered `worth-kernel` construction authoring
  path no longer carries `REQUIRED_QUERY_FAMILIES`,
  `REPORTED_QUERY_FAMILIES`, or `PrimitiveConstructionQueryGapRow` for the
  Query rows the support pinning contract now owns.

**Engineering decisions**
- Pin declarations are typed artifacts in the consumer crate, not strings in
  CI configuration.
- Pin identity lowers through the `9.6` evidence primitive.

**Open questions**
- None.

### Phase 6: Shipped In-Memory Consumer Test Backend Boundary

Ship the in-memory test runtime as kit surface: one public, honestly-postured
backend a downstream crate can build a valid `WORTHQueryWorkspace` from for
reads, writes, previews, and invariant registration â€” without implementing
runtime-bridge adapter traits or fabricating receipts by hand.

**Relevant subsystems**
- consumer test backend (new boundary home inside the consumer kit)
- the internal `memory_workspace` machinery (currently private)
- runtime builder and backend parts intake
- the Milestone `9.5` raw runtime read bootstrap (the read-lane neighbor this
  phase generalizes for consumers)

**Relevant Query source surfaces**
- [memory_workspace/mod.rs](../../crates/worth-query/src/memory_workspace/mod.rs)
  (private today; the implementation seed for the shipped backend)
- [runtime/builder.rs](../../crates/worth-query/src/runtime/builder.rs)
- [runtime/backend/parts.rs](../../crates/worth-query/src/runtime/backend/parts.rs)

**Relevant downstream evidence (external consumers)**
- `workflow-editor` (`workflow_query_WORTH/src/workspace_reads/tests_support.rs`
  and its certification harness) hand-implements seven-plus
  `WORTHQueryRuntime*Adapter` traits plus a `RuntimeBridge` assembly in
  dev-dependencies just to test workspace reads
- `hadwiger-research` test suites implement
  `WORTHQueryRuntimeWriteAuthorityAdapter` and fabricate
  `WriteAuthorityExecutionReceipt` / `WORTHQueryMutationReceipt` values by
  hand to exercise invariant denial materialization

**Target shape (illustrative, not frozen API)**

```rust
// BEFORE (workflow-editor today): seven hand-written adapter impls plus a
// RuntimeBridge assembly in dev-deps, per consumer, just to test reads

// AFTER: one kit entry returning a valid, honestly-postured workspace
let mut workspace = worth_query::kit::in_memory_test_runtime()
    .with_schema(test_schema)
    .workspace("workflow-editor.tests")?;
// reads, writes, previews, and invariant registration are real lanes;
// anything the backend cannot honor fails closed through the ordinary
// support matrix, never by silently faking receipts
```

**Warnings**
- Do not ship the backend with optimistic support rows; everything it cannot
  honor must fail closed through the same admission discipline as production
  backends, or consumers will certify against fiction.
- Do not fork a second workspace semantics for testing; the backend is a
  backend, and the workspace above it is the ordinary workspace.
- Do not leave receipt construction public-by-necessity; if a consumer test
  still needs to fabricate a receipt by hand, the backend is incomplete.

**Test requirements**
- Add a `Shipped Test Backend Honesty Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: a representative covered workload (reads, writes,
  preview admission, invariant registration and denial) produces the same
  canonical artifacts, receipts, and support postures through the shipped
  backend as through the equivalent hand-assembled bridge-backed harness it
  replaces.
- Adversarial denial: every family the backend cannot honor fails closed with
  the same typed admission stops as a production backend â€” proven by
  exhaustively walking the support matrix rather than spot-checking.
- Adversarial residue: an exact assertion that the covered consumer test
  suites contain zero hand-implemented runtime adapter traits and zero
  hand-fabricated receipt values after adoption. Current `worth-kernel`
  construction is not the proof target for this residue class unless a fresh
  audit finds such assembly there; external consumers or fixture crates must
  carry this proof.

**Engineering decisions**
- The private `memory_workspace` machinery is promoted through the kit rather
  than duplicated; one in-memory truth implementation, one public posture.
- The backend ships with honest support rows from birth â€” it is a first-class
  backend posture, not a mock.

**Open questions**
- None.

### Phase 7: Reference Consumer Evidence Report Adoption Boundary

Prove the report kit by adoption: migrate `worth-kernel`'s covered evidence
reports onto the Phase 1 kit and the canonical digest scheme, deleting the
hand-rolled report and digest plumbing in the same change program.

**Relevant subsystems**
- `worth-kernel` construction runtime-proof report surfaces
- evidence report composition kit (from Phase 1)

**Relevant downstream surfaces**
- [worth-kernel tests/support/branch_preview_basis.rs](../../crates/worth-kernel/src/construction/tests/support/branch_preview_basis.rs)
- [worth-kernel tests/support/projection_consumption.rs](../../crates/worth-kernel/src/construction/tests/support/projection_consumption.rs)
- [worth-kernel tests/support/family_coverage.rs](../../crates/worth-kernel/src/construction/tests/support/family_coverage.rs)
- [worth-kernel tests/support/compound_parity_support.rs](../../crates/worth-kernel/src/construction/tests/support/compound_parity_support.rs)
- [worth-kernel tests/support/corpus_replay_digest.rs](../../crates/worth-kernel/src/construction/tests/support/corpus_replay_digest.rs)
- [worth-kernel digest_protocol.rs](../../crates/worth-kernel/src/construction/digest_protocol.rs)

**Warnings**
- Do not preserve the old report scaffolding beside the kit "for safety";
  surviving folklore is the failure mode this milestone exists to kill.
- Do not let adoption weaken `worth-kernel`'s evidence semantics; every
  migrated report keeps its semantic fields and its assertions, re-expressed
  rather than reduced.

**Test requirements**
- Add a `Reference Consumer Report Adoption Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: migrated `worth-kernel` reports preserve their
  semantic assertions and produce canonical-scheme digests for identical
  inputs.
- Adversarial residue: an exact structural assertion that covered
  `worth-kernel` report surfaces contain zero remaining hand-rolled digest
  construction and zero hand-written report/getter/error plumbing the kit
  now derives.
- Adversarial defended-residue classification: any remaining
  `worth-kernel` digest helper usage in the construction tree is classified as
  worth-domain artifact identity, not Query evidence identity, with an
  inventory digest naming the defended files.

**Engineering decisions**
- `worth-kernel` is the named reference consumer; its deletion diff is part
  of this milestone's acceptance evidence.
- Kit gaps discovered during report adoption are fixed in Phase 1, not worked
  around consumer-side â€” adoption is the kit's hostile review.

**Open questions**
- None.

### Phase 8: Reference Consumer Audit, Pinning, And Test-Backend Adoption Boundary

Prove the enforcement and posture surfaces by adoption: replace
`worth-kernel`'s grep audit with the shipped audit artifact, its hand-built
required-family and gap-row bookkeeping with support pins, and any
hand-assembled test harness surfaces with the shipped backend where it is the
honest fit.

**Relevant subsystems**
- `worth-kernel` audit and authoring authority surfaces
- shipped audit artifact (Phase 3), pinning contract (Phase 5), test backend
  (Phase 6)

**Relevant downstream surfaces**
- [worth-kernel authoring.rs](../../crates/worth-kernel/src/construction/authoring.rs)
- [worth-kernel tests/boundary.rs](../../crates/worth-kernel/src/construction/tests/boundary.rs)
- [worth-kernel tests/phase_eight_minimization.rs](../../crates/worth-kernel/src/construction/tests/phase_eight_minimization.rs)
- [worth-kernel tests/boundary_phase_five/patterns.rs](../../crates/worth-kernel/src/construction/tests/boundary_phase_five/patterns.rs)
- [worth-kernel certification/phase_five_boundary_closeout_tests.rs](../../crates/worth-kernel/src/construction/certification/phase_five_boundary_closeout_tests.rs)

**Warnings**
- Do not treat this phase as optional polish; per `MENTALITY.md`, cross-crate
  scope expansion to prove the foundation is the norm, and a kit no consumer
  has adopted is plausibility, not proof.
- Do not force the shipped backend where `worth-kernel`'s topology-backed
  runtime entry is the honest production-shaped harness; backend adoption
  covers the surfaces where hand assembly was the only reason for the
  current shape.

**Test requirements**
- Add a `Reference Consumer Enforcement Adoption Test` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- Adversarial equivalence: the shipped audit detects the same seeded
  violation classes the grep audit detected, through the shipped artifact,
  from `worth-kernel`'s own test suite.
- Adversarial residue: an exact structural assertion that covered
  `worth-kernel` surfaces contain zero `include_str!` audit machinery and
  zero hand-built required-family or gap-row assembly where pinning now
  answers.
- Adversarial classification: source-string audits that remain in
  `worth-kernel` are classified as worth-domain topology or legacy-deletion
  hygiene, not Query-prohibition rows; the classification itself has an
  inventory digest.

**Engineering decisions**
- Adoption order follows kit dependency order: audit, then pins, then
  backend.
- Kit gaps discovered during enforcement adoption are fixed in Phases 2â€“6,
  not worked around consumer-side.

**Open questions**
- None.

### Phase 9: Typed Consumer Residue Audit For Query Proof Folklore

Harden the Consumer Kit residue audit so Query owns the classification of
downstream Query-consumption residue, not only the narrow test-backend adapter
residue caught by the first 9.8 closeout. This phase does not reopen the kit
architecture; it closes a gap in the existing Consumer Kit authority boundary.

The current `query_test_backend_residue_audit(...)` surface catches consumers
that rebuild fake runtime/test-backend infrastructure. The addendum must broaden
the owned meaning to fake Query proof as well: local Query reports, local Query
proof structs, raw support snapshot row spelunking, support-matrix row searches,
debug-derived proof strings, delimiter-joined proof strings, and
delimiter-formatted proof strings.

**Adversarial constraint**
- A downstream consumer must not be able to replace Query-owned proof with a
  local report, local proof struct, raw support-row lookup, debug string, or
  delimiter string while still producing a clean Consumer Kit adoption signal.
- A tired downstream maintainer must be able to run one Query-owned audit over
  source roots and receive precise typed findings without inventing local grep
  patterns, local class names, or local replacement guidance.
- The audit must survive ordinary source noise: comments, documentation,
  unrelated debug formatting, unrelated delimiter joins, and Query-owned
  implementation files must not create false residue.

**Relevant subsystems**
- Consumer Kit test-backend residue audit
- Consumer Kit source inventory and boundary-audit source sets
- evidence identity for residue findings and reports
- `worth-kernel` reference-consumer adoption checks

**Relevant Query source surfaces**
- [consumer_kit/test_backend/residue_audit](../../crates/worth-query/src/consumer_kit/test_backend/residue_audit)
- [consumer_kit/boundary_audit/source_inventory](../../crates/worth-query/src/consumer_kit/boundary_audit/source_inventory)
- [consumer_kit/graph_obligation_adoption/local_ceremony_audit](../../crates/worth-query/src/consumer_kit/graph_obligation_adoption/local_ceremony_audit)
- [consumer_kit/evidence_report_adoption](../../crates/worth-query/src/consumer_kit/evidence_report_adoption)

**Required directory skeleton**
- `crates/worth-query/src/consumer_kit/consumer_residue/`
  - `mod.rs`
  - `audit.rs`
  - `registry.rs`
  - `detection.rs`
  - `syntax.rs`
  - `syntax_context.rs`
  - `finding.rs`
  - `report.rs`
  - `evidence.rs`
  - `source_text_mask.rs`
  - `tests.rs`
- `crates/worth-query/src/consumer_kit/test_backend/residue_audit/`
  remains as the compatibility entry point for test-backend-specific callers.
- `crates/worth-query/tests/consumer_residue_audit.rs`
  owns hostile public-facade coverage and false-positive coverage.

**Public API target**
- Add a typed `WORTHQueryConsumerResidueClass` enum. It must include the
  existing runtime/test-backend classes and the new proof-folklore classes:
  `local-query-report`, `local-query-proof`, `raw-support-snapshot-row`,
  `support-matrix-row-search`, `debug-derived-query-proof`,
  `delimiter-joined-query-proof`, and `delimiter-formatted-query-proof`.
- Add registry-backed rows that carry class, detection strategy, explanation,
  and replacement lane. The replacement lane must point consumers back to the
  public Consumer Kit surface rather than to private module constructors.
- Add `query_consumer_residue_audit(consumer_name)` as the general audit entry
  point.
- Keep `query_test_backend_residue_audit(consumer_name)` as a compatibility
  lane or wrapper so existing callers do not lose the test-backend-focused name.
- Findings must expose typed class identity, source path, matched pattern or
  detection key, line, column, finding identity, and report identity.

**Detection requirements**
- Detection must parse Rust source for the classes where syntax determines
  meaning: local struct declarations, type paths, method calls, `format!`
  invocations, and `.join(...)` calls. Text masking may still be used as a
  pre-filter, but the residue decision cannot be a naive whole-file substring
  match for broad proof-folklore classes.
- Existing exact test-backend adapter and fabricated receipt patterns may stay
  text-backed only when they are narrow symbols with low false-positive risk and
  still report line/column.
- Comment and documentation text must not count as residue.
- Ordinary debug formatting and ordinary delimiter joins must not count unless
  the syntax context is Query/proof/support/receipt/evidence-shaped.
- Raw support snapshot row use and support matrix row search must report in
  downstream consumer roots, while Query-owned implementation roots remain
  allowed owners.
- The registry must isolate detection strategy per residue class so text-backed
  exact-symbol detectors and AST-backed folklore detectors compose behind one
  public report API.

**Warnings**
- Do not turn this into a naive whole-file substring scanner for broad
  `format!("{:?}")` or `.join("||")` patterns.
- Do not make Worth own the class list. Downstream consumers choose roots and
  assert reports; WORTH Query owns residue meaning, identity, and replacement
  guidance.
- Do not collapse this into the Milestone `9.9` graph-obligation local ceremony
  audit. Graph obligation ceremony is a specialized 9.9 lane; this addendum is
  the generic 9.8 Consumer Kit residue authority.

**DX target**

```rust
use worth_query::facade::consumer_kit::query_consumer_residue_audit;

let report = query_consumer_residue_audit("worth-kernel")
    .required_root("crates/worth-kernel/src/construction")
    .evaluate()?;

report.assert_clean();

for finding in report.findings() {
    eprintln!(
        "{}:{}:{} {} -> {}",
        finding.source_path(),
        finding.line(),
        finding.column(),
        finding.residue_class().as_str(),
        finding.replacement_lane(),
    );
}
```

**Test requirements**
- One hostile test per residue class proving a typed finding is emitted.
- False-positive tests for ordinary debug formatting, ordinary delimiter joins,
  comments, docs, and Query-owned implementation roots.
- AST/syntax tests proving local struct declarations, proof-like assignments,
  proof-like returns, method calls, and type paths are detected by syntax role,
  not by incidental source substrings.
- Public-facade tests only: no private module constructors and no local proof
  assembly in the test harness.
- Reference-consumer audit coverage proving downstream roots can assert the
  report without supplying local residue classes.

**Engineering decisions**
- This addendum strengthens the existing Milestone `9.8` Consumer Kit contract;
  it does not change the Milestone `9.9` graph-obligation authority model.
- AST-backed proof-folklore detection is required in this addendum because the
  adversarial constraint is false-positive-resistant consumer proof authority,
  not merely broader substring coverage.
- The Consumer Kit owns all Query-consumption residue classes. Worth-owned
  artifact identity residue can be defended only when explicitly classified as
  domain artifact identity, never by silently bypassing Query proof surfaces.

**Open questions**
- None.

### Phase 10: Consumer Residue Certification, Support, Docs, And Reference Adoption

Close the typed residue audit as shipped infrastructure, not as an API stub. If
Phase 9 builds the audit authority, Phase 10 proves it against real consumer
roots, support/profile output, docs, and the hostile certification matrix so the
milestone does not rely on any follow-on cleanup pass.

**Relevant subsystems**
- Consumer Kit public facade exports
- `worth-kernel` reference-consumer roots
- Consumer Kit docs and AI readme guidance
- `application` support/profile reporting
- public documentation coverage
- Milestone `9.8` hostile certification matrix

**Required directory skeleton**
- `crates/worth-query/tests/consumer_residue_audit.rs`
- `crates/worth-query/tests/consumer_residue_reference_adoption.rs`
- `plans/worth-query/test-requirements.md`
- `crates/worth-query/docs/AI_README.md`
- `crates/worth-query/docs/foundations/consumer-kit.md`
- `crates/worth-query/src/application/support/report.rs`
- `crates/worth-query/src/application/support/tests/consumer_kit_closure.rs`
- `crates/worth-query/src/public_doc_coverage/tests/support.rs`

**Required proof**
- The public facade exports `query_consumer_residue_audit(...)`,
  `WORTHQueryConsumerResidueClass`, the typed finding type, and the typed report
  type.
- The old `query_test_backend_residue_audit(...)` remains usable and is proved
  to report the same runtime/test-backend residue classes through the new typed
  registry.
- A reference-consumer test runs the general audit over current downstream roots
  and proves covered Query-proof folklore is either gone or reported with typed
  findings.
- The certification matrix names every consumer-residue class and includes both
  hostile detection and false-positive certification rows.
- AI_README teaches the generic Consumer Kit residue audit as the ordinary path
  for downstream Query-proof cleanup and explicitly distinguishes it from the
  specialized Milestone `9.9` graph-obligation local ceremony audit.
- The kit surfaces enter the public docs as the ordinary consumer path, and
  every doc passage that still teaches hand-rolled reports, consumer greps,
  hand-built gap rows, or local Query-proof folklore is removed in this phase.
- Support/profile output, docs, and certification output agree exactly on the
  kit families' posture and the generic consumer-residue authority.

**Warnings**
- Do not close Phase 10 on synthetic fixture coverage alone. Fixtures prove
  class semantics; reference-consumer roots prove adoption pressure.
- Do not document this as an optional lint. It is Query's proof-consumption
  boundary audit.
- Do not let downstream consumers provide their own class registry, scanner, or
  replacement matrix.
- Do not close on kit API presence; closure is the certification matrix plus
  the adoption residue assertions passing together.
- Do not let docs teach the kit as optional ergonomics; it is the ordinary
  path, and the folklore patterns it replaces are named anti-patterns.

**Test requirements**
- Add a `Milestone 9.8 Consumer Kit Hostile Certification Matrix` to
  [test-requirements.md](./test-requirements.md) and close it in this phase.
- `cargo test -p worth-query --test consumer_residue_audit`
- `cargo test -p worth-query --test consumer_residue_reference_adoption`
- `cargo test -p worth-query milestone_9_8_consumer_kit`
- targeted docs/status audit proving Milestone `9.8`, AI_README, and Consumer
  Kit docs agree on generic residue authority and the 9.9 specialized boundary
- Combined adversarial matrix: drive report misuse, seeded bypasses, posture
  regressions, and folklore-resurrection probes in one program and require
  narrow canonical artifacts for the kit-report digest scheme, the audit
  finding set, the snapshot/pin agreement, and the adoption residue counts.
- Adversarial agreement: prove docs, support/profile rows, and certification
  output agree exactly on the kit families' posture.
- Adversarial reference-consumer evidence: the matrix publishes current
  `worth-kernel` adoption counts for covered report/digest residue,
  Query-prohibition audit residue, support-pinning residue, generic consumer
  proof-folklore residue, and defended worth-domain residues.

**Engineering decisions**
- This phase is part of Milestone `9.8` closeout, not deferred work.
- Support/profile output is authoritative for kit family closure.
- The certification matrix runs against the Milestone `9.5` raw runtime
  bootstrap so kit certification never grows private harness assembly.
- If AST-backed detection requires additional internal parser support, that
  support is in scope for 9.8 unless implementation proves it crosses the
  explicit `MENTALITY.md` major-work threshold for true blocker debt.
- Durable persisted audit archives remain out of scope because persistence is a
  store-backed concern, but source-root audit execution, typed classification,
  and reference-consumer certification are not out of scope.

**Open questions**
- None.

## Must Ship

- the declarative evidence-report composition kit over the `9.6` canonical
  evidence-identity primitive
- runtime-owned bypass enforcement: sealed/visibility-tightened seams plus
  one shipped audit artifact derived from a single prohibition registry
- the serialized, versioned support snapshot and the typed consumer pinning
  contract with build-failing drift detection
- the shipped in-memory consumer test backend with honest fail-closed support
  posture, replacing hand-implemented adapter assemblies and hand-fabricated
  receipts in consumer test suites where that residue exists
- reference-consumer adoption with deletion of `worth-kernel`'s hand-rolled
  Query-evidence report plumbing, Query-prohibition grep audit, and
  required-family/gap-row assembly in covered surfaces
- typed Consumer Kit residue audit coverage for local Query proof folklore,
  raw support-row spelunking, and fake proof strings, with public-facade
  findings that carry class identity and replacement guidance
- support/profile, docs, and hostile certification closure for the kit
  families

## Must Preserve

- the `9.6` canonical evidence-identity scheme as the only digest authority
  the kit can express
- the hard prohibitions' meaning â€” relocated into enforceable structure, not
  reworded
- one support truth: the snapshot remains a digest-bound derived projection
  of the live matrix
- `worth-kernel`'s evidence semantics through migration â€” re-expressed, never
  reduced
- the Query facade as the only consumer surface; the kit adds no second
  entry path into runtime internals

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the Milestone `9.8` certification suites added to
  [test-requirements.md](./test-requirements.md) pass with narrow
  machine-checkable artifacts
- a kit-authored report reproduces a hand-rolled report's semantics with
  canonical-scheme digests, and misuse fails typed or fails to compile
- the shipped audit detects seeded bypasses structurally, with zero textual
  false positives, from a downstream crate's own test suite
- a pinned posture regression fails exactly the pinned consumers' builds with
  typed findings
- a downstream-shaped test suite obtains a valid workspace from the shipped
  in-memory backend and exercises reads, writes, previews, and invariant
  registration with zero hand-implemented adapter traits and zero
  hand-fabricated receipts in covered backend-adoption surfaces, and every
  unbacked family fails closed
- covered `worth-kernel` surfaces carry zero remaining hand-rolled digest,
  audit, or gap-row folklore for Query-owned evidence/support/prohibition
  surfaces, with defended worth-domain residues inventoried and the deletion
  diff recorded as part of the milestone evidence
- downstream consumer roots can run a WORTH Query-owned consumer-residue audit
  and receive typed findings for fake Query-proof folklore without defining a
  local scanner, local residue class list, or local replacement matrix

## Sequencing Notes

- This milestone belongs after [milestone-9.6.md](./milestone-9.6.md), which
  supplies the evidence-identity primitive the kit is built on, and after
  [milestone-9.7.md](./milestone-9.7.md), so the kit's report, audit, and
  pinning surfaces cover the concurrency-era facade families rather than
  shipping against a surface about to be re-expressed.
- It sits before Milestone `10` in the critical path so the kit
  pressure-tests the frozen runtime-backed product surface through real
  consumer adoption before store-backed work builds on it; where staffing
  allows, its phases may overlap early Milestone `10` work, since store
  execution does not consume kit surfaces.
- Durable artifacts are out of scope: persisted snapshots, durable audit
  archives, and store-backed kit artifacts remain Milestone `10`/`11` scope.
