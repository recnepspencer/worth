# Milestone 7 Closeout: Lineage, Provenance, And Receipt Vocabulary

Date: 2026-05-19

## Status

Milestone 7 is implementation-complete for `worth-foundational` through
Phase 9.

The crate now owns the shared boundary-evidence language for category and
locality primitives, provenance layering and freshness, receipt families and
closeout truth, lineage continuity/divergence outcomes, support-truth and
degraded recovery, mixed-family attachment materialization and canonical/digest
participation, proof-bearing readiness closure, and crate-facing feature docs
for the shipped Milestone 7 surface.

## Completed Surface

- Typed boundary-evidence category, locality, role, and freshness primitives
  now exist and remain mechanically non-substitutable.
- Provenance is now basis-explicit, freshness-explicit, and blind-consumer
  readable across boundary-artifact and transition-rooted source bases.
- Planning, executed, support-publication, restoration, checkpoint/resume, and
  closeout receipt families are now family-distinct, with blocked and denied
  closeout preserved as completed-boundary truth without impersonating
  execution.
- Lineage outcomes now remain typed across attested continuity, replay-derived
  continuity, restored continuity, reconstructed equivalence, branch-local
  replacement, promotion posture, partial continuity, withheld/redacted
  continuity, ambiguity, identity break, denial, and transient-within-boundary
  closure.
- Support-truth is now support-grade instead of authority-shaped, with typed
  recovery postures, basis disclosures, residual debt, transient lifecycle
  evidence, and degraded closeout surfaces.
- Mixed-family attachment bundles can now legally carry lineage, provenance,
  receipt, support, and diagnostics together on one target artifact while
  keeping object-level versus locator-level continuity explicit.
- Attachment bundles now participate in canonical basis and digest derivation
  honestly, and both current-basis and support-basis readmission remain
  explicit stronger lanes instead of ambient upgrades.
- Boundary-evidence readiness now exists as a proof-bearing artifact with exact
  certified surfaces, hostile pressures, compile-fail boundaries, golden
  artifacts, property seeds, harness expansion points, grouped public-surface
  inventory, documentation inventory, runtime assumptions, non-assumptions, and
  residual debt.
- Crate-facing Milestone 7 docs now exist under
  [crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth),
  with one landing page and one feature doc per shipped capability seam, plus a
  crate docs entrypoint at
  [crates/worth-foundational/docs/README.md](../../crates/worth-foundational/docs/README.md).

## Phase Crosswalk

### Phase 1: Category, Locality, And Role Primitives

Shipped homes:

- [primitives.rs](../../crates/worth-foundational/src/boundary_evidence/primitives.rs)
- [front_doors/mod.rs](../../crates/worth-foundational/src/boundary_evidence/front_doors/mod.rs)
- [common_path.rs](../../crates/worth-foundational/src/boundary_evidence_api/common_path.rs)
- [lower_lane/primitives.rs](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane/primitives.rs)
- [primitives.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/primitives.rs)
- [ui/boundary_evidence/primitives](../../crates/worth-foundational/tests/ui/boundary_evidence/primitives)

What closed:

- typed category and locality definitions
- typed role/descriptive/freshness posture floor
- minimum legality without pre-solving later provenance or receipt law
- compile-time non-substitution for primitive families

### Phase 2: Provenance Layering And Freshness Law

Shipped homes:

- [provenance/mod.rs](../../crates/worth-foundational/src/boundary_evidence/provenance/mod.rs)
- [layers.rs](../../crates/worth-foundational/src/boundary_evidence/provenance/layers.rs)
- [source_basis.rs](../../crates/worth-foundational/src/boundary_evidence/provenance/source_basis.rs)
- [artifact.rs](../../crates/worth-foundational/src/boundary_evidence/provenance/artifact.rs)
- [provenance_front_doors.rs](../../crates/worth-foundational/src/boundary_evidence/provenance_front_doors.rs)
- [lower_lane/provenance.rs](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane/provenance.rs)
- [provenance.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/provenance.rs)
- [ui/boundary_evidence/provenance](../../crates/worth-foundational/tests/ui/boundary_evidence/provenance)

What closed:

- source-basis, authority-path, strategy/profile/comparison/canonical basis,
  and support-context layering
- family-distinct boundary-artifact versus transition provenance roots
- mandatory freshness posture
- canonical support-context ordering and deduplication

### Phase 3: Receipt Families And Closeout Truth

Shipped homes:

- [receipts/mod.rs](../../crates/worth-foundational/src/boundary_evidence/receipts/mod.rs)
- [artifact.rs](../../crates/worth-foundational/src/boundary_evidence/receipts/artifact.rs)
- [receipt_front_doors.rs](../../crates/worth-foundational/src/boundary_evidence/receipt_front_doors.rs)
- [lower_lane/receipts.rs](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane/receipts.rs)
- [receipts.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/receipts.rs)
- [ui/boundary_evidence/receipts](../../crates/worth-foundational/tests/ui/boundary_evidence/receipts)

What closed:

- planning-versus-completed-versus-executed receipt strength
- typed blocked and denied closeout dispositions
- support publication, restoration, and checkpoint/resume receipt families
- replay/history masquerade rejection at the compile-fail boundary

### Phase 4: Lineage, Continuity, And Divergence Outcomes

Shipped homes:

- [lineage/mod.rs](../../crates/worth-foundational/src/boundary_evidence/lineage/mod.rs)
- [artifact.rs](../../crates/worth-foundational/src/boundary_evidence/lineage/artifact.rs)
- [lineage_front_doors.rs](../../crates/worth-foundational/src/boundary_evidence/lineage_front_doors.rs)
- [lower_lane/lineage.rs](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane/lineage.rs)
- [lineage.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/lineage.rs)
- [ui/boundary_evidence/lineage](../../crates/worth-foundational/tests/ui/boundary_evidence/lineage)

What closed:

- typed continuity, divergence, and denial families
- replay-derived versus attested continuity separation
- restored continuity versus reconstructed equivalence separation
- branch-local replacement and promotion posture law
- transient-within-boundary lifecycle staying out of durable continuity

### Phase 5: Support-Truth, Recovery, And Degraded Operation

Shipped homes:

- [support/mod.rs](../../crates/worth-foundational/src/boundary_evidence/support/mod.rs)
- [definitions.rs](../../crates/worth-foundational/src/boundary_evidence/support/definitions.rs)
- [artifact.rs](../../crates/worth-foundational/src/boundary_evidence/support/artifact.rs)
- [support_front_doors.rs](../../crates/worth-foundational/src/boundary_evidence/support_front_doors.rs)
- [lower_lane/support.rs](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane/support.rs)
- [support.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/support.rs)
- [ui/boundary_evidence/support](../../crates/worth-foundational/tests/ui/boundary_evidence/support)

What closed:

- support publication and support closeout families
- degraded recovery posture and basis-disclosure law
- residual debt as required support-grade meaning
- transient lifecycle evidence as support-grade rather than durable lineage

### Phase 6: Attachment, Canonical Participation, And Materialization

Shipped homes:

- [attachments/mod.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/mod.rs)
- [target.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/target.rs)
- [continuity.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/continuity.rs)
- [descriptive.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/descriptive.rs)
- [bundle.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/bundle.rs)
- [materialization.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/materialization.rs)
- [readmission.rs](../../crates/worth-foundational/src/boundary_evidence/attachments/readmission.rs)
- [attachment_front_doors.rs](../../crates/worth-foundational/src/boundary_evidence/attachment_front_doors.rs)
- [stronger_lane/readmission.rs](../../crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs)
- [attachments.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/attachments.rs)
- [ui/boundary_evidence/attachments](../../crates/worth-foundational/tests/ui/boundary_evidence/attachments)

What closed:

- typed attachment targets and continuity scope
- profile-governed materialization
- mixed-family canonical basis and digest participation
- current-basis and support-basis readmission across trust boundaries

### Phase 7: Production-Test Readiness

Shipped homes:

- [readiness/mod.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/mod.rs)
- [authority.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/authority.rs)
- [certification.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/certification.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/vocabulary.rs)
- [inventory.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/inventory.rs)
- [report.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/report.rs)
- [grouped_surface.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/grouped_surface.rs)
- [readiness.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/readiness.rs)
- [ui/boundary_evidence/grouped_surface](../../crates/worth-foundational/tests/ui/boundary_evidence/grouped_surface)
- [ui/boundary_evidence/readiness](../../crates/worth-foundational/tests/ui/boundary_evidence/readiness)

What closed:

- exact certified-surface inventory
- exact grouped public-surface inventory
- exact compile-fail boundary inventory
- exact golden-artifact, property-seed, harness-expansion, assumption,
  non-assumption, residual-debt, and phase-gate inventories

### Phase 8: Feature Docs And Crate-Doc Integration

Shipped homes:

- [docs/README.md](../../crates/worth-foundational/docs/README.md)
- [lineage-provenance-receipts-and-support-truth/README.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/README.md)
- [primitive-categories-locality-and-role-postures.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/primitive-categories-locality-and-role-postures.md)
- [provenance-layering-and-freshness.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/provenance-layering-and-freshness.md)
- [receipts-and-closeout-truth.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/receipts-and-closeout-truth.md)
- [lineage-continuity-divergence-and-promotion.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/lineage-continuity-divergence-and-promotion.md)
- [support-truth-recovery-and-degraded-operation.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md)
- [attachment-materialization-canonical-participation-and-readmission.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/attachment-materialization-canonical-participation-and-readmission.md)
- [grouped-public-lanes-and-stronger-readiness.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/grouped-public-lanes-and-stronger-readiness.md)
- [boundary-evidence-production-readiness.md](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/boundary-evidence-production-readiness.md)

What closed:

- one landing page plus one feature doc per shipped Milestone 7 seam
- grouped common path, lower lane, and stronger lanes documented with real
  examples
- key edge cases made history-safe instead of recoverable only from code

### Phase 9: `feature-doc-writer` Closeout And Crate-Docs Registration

Shipped homes:

- [milestone-7.md](./milestone-7.md)
- [docs/README.md](../../crates/worth-foundational/docs/README.md)
- [lib.rs](../../crates/worth-foundational/src/lib.rs)
- [readiness/inventory.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/inventory.rs)
- [readiness/report.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/report.rs)
- [readiness.rs test](../../crates/worth-foundational/tests/certification/boundary_evidence/readiness.rs)

What closed:

- `feature-doc-writer`-shaped docs closeout against the shipped surface
- docs inventory frozen into the machine-checkable readiness artifact
- crate-root discoverability of the milestone docs surface

## Grouped Public Surface

Milestone 7 ships the grouped public lane at:

- [boundary_evidence_api::common_path](../../crates/worth-foundational/src/boundary_evidence_api/common_path.rs)
- [boundary_evidence_api::lower_lane](../../crates/worth-foundational/src/boundary_evidence_api/lower_lane)
- [boundary_evidence_api::stronger_lane](../../crates/worth-foundational/src/boundary_evidence_api/stronger_lane)

The readiness artifact freezes the exact grouped surface inventory and the
exact docs inventory that explain it.

## Test-Requirements Mapping

Milestone 7 now satisfies the local proof bar for lineage/provenance/receipt
vocabulary in
[plans/worth-foundational/test-requirements.md](./test-requirements.md)
before adopting-crate migration.

What is proved locally:

- producer diversity across independent construction orders
- consumer blindness for boundary-evidence artifacts
- authority separation across planning, execution, closeout, support, and
  readmission lanes
- ordering hostility for provenance and mixed-family attachments
- category hostility across lineage, provenance, receipts, and support-truth
- misuse pressure around replay/history masquerade, promotion denial,
  closeout-versus-executed confusion, stale support basis, and unbridged
  readmission

Primary evidence homes:

- [tests/certification/boundary_evidence](../../crates/worth-foundational/tests/certification/boundary_evidence)
- [tests/ui/boundary_evidence](../../crates/worth-foundational/tests/ui/boundary_evidence)
- [readiness/report.rs](../../crates/worth-foundational/src/boundary_evidence/readiness/report.rs)

## Remaining Debt

Milestone 7 closes with explicit, bounded residual debt only:

- adopting-runtime parity is still deferred to real crate migrations
- runtime-specific history/journal taxonomy remains crate-local
- real runtime support-bundle persistence topology remains crate-local

No local foundational implementation debt remains open for the shipped
Milestone 7 surface.

## Verification

The final broad gap-close pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational --test certification boundary_evidence::readiness -- --nocapture
cargo test -p worth-foundational
git diff --check
```

That pass completed cleanly, aside from the existing harmless LF/CRLF warnings
on already-touched files.
