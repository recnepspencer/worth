# Milestone 6: Diagnostics And Explanation Ontology

## Goal

Define one shared diagnostics and explanation language so WORTH crates can
report why something happened, why something was denied, what evidence exists,
what evidence is missing, and what detail was retained, deferred, redacted, or
reconstructed without inventing local diagnostics folklore or letting
descriptive artifacts impersonate authority.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and treating explanation as a
production contract rather than an afterthought. The shaping constraint is that
Milestone 6 must solve the hostile case first: partial evidence, stale replay,
policy-redacted detail, deferred reconstruction, and family-distinct denials
must all remain typed and self-describing instead of collapsing into debug
strings.

### `arch_laws.md`

Protects authority-versus-description separation, self-describing envelopes,
explicit seams, and proof-bearing strengthening only where stronger claims are
real. The shaping constraint is that diagnostics must stay descriptive,
receipts and authority artifacts must stay authoritative, and explanation
bundles must never fake missing evidence or stronger claims through placeholder
fields.

### `composition_laws.md`

Protects responsibility-shaped files, named boundaries, and narrow helpers. The
shaping constraint is that diagnostic primitives, outcome topology,
materialization/delivery law, bundle law, canonical-basis participation, and
readiness evidence must live in separate responsibility homes rather than one
large diagnostics bag.

### `domain_structure_laws.md`

Protects structure as domain topology rather than convenience filing. The
shaping constraint is that support posture, explanation rows, evidence
availability, redaction posture, denial artifacts, and diagnostic bundles must
be independently locatable and testable.

### `perf_laws.md`

Protects cost-honest surfaces, explicit planning/materialization seams, and
cold-path richness. The shaping constraint is that retained-hot,
deferred-cold, reconstructable, and unavailable diagnostics must be explicit
postures; no report or explanation API may hide rescans, replay rebuilds, or
broad diagnostic assembly behind a cheap-looking accessor.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` owns shared truth-adjacent
boundary vocabulary while preserving crate-local execution and storage. The
shaping constraint is that Milestone 6 must standardize diagnostics meaning,
not one diagnostics store, one report layout, one index, or one replay engine.

### `worth_foundational_roadmap.md`

Protects sequencing. The shaping constraint is that diagnostics follows values,
canonicalization, profiles, boundary artifacts, and transitions, and must land
before lineage/provenance/receipt deepening and migration closure. Milestone 6
must therefore consume existing artifact, profile, and transition law rather
than reopening them.

### `test-requirements.md`

Protects local hostile proof before adopting-crate migration. The shaping
constraint is that Milestone 6 must prove canonical diagnostic identity,
outcome-family separation, blind-consumer explanation, profile-richness
honesty, compile-fail boundaries, and attachment compatibility locally. It also
means the suite must attack ambient basis choice, hidden strategy influence,
thin diagnostic bundles, missing hostile coverage, report overclaiming, and
cheap convenience helpers that bypass inspectable explanation seams.

### `milestone-5.md`

Protects the newly standardized transition nouns and proof-bearing transition
surfaces. The shaping constraint is that diagnostics for merge, commit, branch,
discard, and receipt behavior must consume those canonical transition artifacts
instead of recreating transition meaning locally.

### `milestone-5-closeout.md`

Protects the fact that branch-local, merge, committed-authority, receipt,
bundle, locator, canonical-basis, profile-reuse, and current-basis lanes are
already implemented and locally certified. The shaping constraint is that
Milestone 6 must attach explanation to those surfaces without re-deciding their
authority or flattening their distinctions.

## Existing Runtime Patterns

Milestone 6 is intentionally shaped by the hard diagnostics surfaces already
shipped elsewhere in WORTH.

### `worth-signal`

What to keep:

- diagnostics and history are different questions: "why did this happen?" is
  not "what happened over time?"
- retained artifacts, reconstructed artifacts, and authoritative compact truth
  are distinct
- diagnostics availability is explicit; a forensic request can return
  retained, reconstructed, or unavailable posture instead of pretending
  everything is always hot
- diagnostics policy is a typed plan with retained-only, budgeted expansion,
  forensic-expansion budget, and deny-cold-expansion lanes
- failures and rollbacks keep phase, execution, rollback, and epoch context
  structured instead of prose-only
- compare and inspect surfaces are grouped by job, not one universal dump

What to prune:

- signal-specific naming and graph-centric grouping should not become
  foundational vocabulary wholesale
- foundational should standardize the boundary meanings, not Signal's whole
  public diagnostics API tree

### `worth-relational`

What to keep:

- diagnostic scopes, artifact kinds, delivery classes, and determinism
  expectations are explicit
- diagnostics profiles make hot/deferred/reconstructable policy visible and can
  disable optional artifacts without changing truth
- merge execution diagnostics keep structural summary, policy proof boundary,
  applied policies, record classes, and aspect rows explicit
- preparation failures classify planning-proof insufficiency, packet overlap,
  identity conflict, fallback-to-serial, and isolation problems distinctly
- rejection artifacts remain typed and scope-bearing
- canonical ordering for validation observations is asserted directly

What to prune:

- relational-local codes and scope names are too wide to become foundational
  names directly
- foundational should capture shared explanation law, not relational's full
  domain-specific code catalog

### `worth-runtime-bridge`

What to keep:

- explanation artifacts are digest-bearing and self-describing
- bridge policy reports show a strong row pattern: contract, lowered policy,
  provenance entries, replay bundle, canonical basis, digest
- merge explanations preserve decision logs, blocked stage, denial class, and
  counters
- historical failure records keep selector, branch, optional commit/snapshot,
  typed failure class, and counters together
- preview execution, discard, promotion, and replay explanation remain
  different artifacts with different authority stories
- the facade exposes explanation by record type rather than one generic blob

What to prune:

- bridge-local family names and record proliferation should not become the
  foundational ontology itself
- foundational should borrow the explanation grammar, not every bridge subtype

### `worth-query`

What to keep:

- support reports and diagnostic bundles are sealed and typed; callers cannot
  fabricate them
- hostile coverage is a real closure requirement, not nice-to-have metadata
- support cannot overclaim durable/store-backed meaning that was not certified
- diagnostic assembly must not rescan or rediscover semantics after the
  canonical artifacts already exist
- family-distinct support and denial posture remain distinct even when lower
  bridge families are shared
- bridge parity and manual witness patterns make "same semantic thing" an
  explicit proof artifact rather than narrative prose

What to prune:

- query-family-specific classes and runtime-certification matrices should stay
  in Query
- foundational should standardize the support/explanation grammar, not Query's
  subscription taxonomy

### `worth-topo`

What to keep:

- support and runtime-posture matrices treat omission as a construction bug,
  not an implicit denial
- family support, lane support, and closeout posture are distinct surfaces
- locality claims are explicit enough to compare claimed scope versus executed
  scope and record mismatch as first-class debt
- repeated rediscovery is counted and denied explicitly rather than tolerated
  as hidden explanation work
- derived diagnostics are anchored to concrete basis evidence, touched aspects,
  fallback counts, and mutation origin instead of free-form topology prose
- hostile certification categories can be `Certified` or `Partial`, with named
  gap labels rather than vague â€œnot done yetâ€ folklore
- failure locality and widened fallout are tracked separately from the primary
  rejection class

What to prune:

- topology-family, lane, and hostile-suite taxonomies are worth-topo-owned and
  should not become foundational names
- foundational should borrow the matrix/gap/locality grammar, not the
  topology-domain scenario catalog

## Why This Milestone Exists

Milestones 1 through 5 already closed:

- aspect-native truth vocabulary
- canonical basis and digest-honest equality
- profile meaning and reduced-richness law
- boundary artifact categories and materialization seams
- branch/merge/commit authority-transition language

They did not yet answer the next shared boundary question:

- what is a diagnostic code, scope, severity, and artifact kind
- what is an accepted/advisory/denied/unsupported/deferred/mismatch outcome
- what does it mean for evidence to be hot, deferred, reconstructable, or
  unavailable
- how does a blind consumer tell whether missing detail was redacted, not
  retained, denied by policy, or simply never existed
- how do support reports, explanation rows, failure bundles, and comparison
  bundles stay descriptive without impersonating authority
- how do reduced-richness profiles change breadth without changing truth

Without Milestone 6, every adopting crate can still answer those questions with
its own diagnostics dialect. That would repeat the exact problem
`worth-foundational` exists to solve.

This milestone therefore owns the shared explanation boundary before Milestone
7 deepens provenance and receipts and before Milestone 11 migrations ask real
runtimes to converge on one diagnostics language.

It also deliberately lays groundwork for Milestone 7. If diagnostics does not
already distinguish explanation from provenance-readiness, retained evidence
from reconstructed evidence, domain denial from policy denial from structural
construction bug, and certified coverage from partial-with-named-gaps, then
Milestone 7 will be forced to recover those meanings after ambiguity already
shipped.

## Adversarial Constraint

Several WORTH crates with different runtime models, retention policies,
authority boundaries, replay shapes, support matrices, preview/branch flows,
and strategy-bearing transitions must be able to describe the same diagnostic
fact, denial, advisory, missing-evidence posture, and explanation bundle with
one canonical meaning everywhere, while preserving:

- whether the artifact is purely descriptive
- whether evidence is retained hot, deferred cold, reconstructable, redacted,
  or unavailable
- whether an outcome is success, advisory, denial, unsupported, deferred,
  mismatch, partial, or violation
- whether a support claim is truly certified or merely deferred or denied
- whether a transition or policy fact came from retained evidence or replay
  reconstruction
- whether branch/preview/discard paths remained non-authoritative
- whether an omitted support/explanation row is a real denial or a producer
  construction bug
- whether the explanation stayed localized or widened beyond the declared
  subject or scope
- whether the diagnostic row is explanation of a decision versus provenance of
  evidence origin
- whether a failure means domain denial, policy denial, evidence absence, or a
  construction/integrity breach in the reporting surface itself
- whether a partially complete artifact carries explicit named gaps instead of
  bluffing completeness

This milestone fails if:

- diagnostics can mutate or redefine authoritative outcome meaning
- missing evidence, redacted evidence, and unsupported evidence collapse into
  one generic `None`
- reduced-richness profiles alter truth instead of only diagnostic breadth
- report/support bundles fake evidence through placeholder ids, zero digests,
  or prose-only recovery
- family-distinct denials or advisories collapse because lower-runtime surfaces
  happened to share a bridge family or record shape
- explanation requires rescanning broad runtime state rather than consuming the
  canonical artifacts already named at the boundary
- hot, deferred, reconstructable, and unavailable evidence are not
  distinguishable
- preview discard or non-authoritative closeout can read like committed
  authority evidence
- diagnostic rows are not canonical enough for a blind consumer to interpret
  them without producer folklore
- support or coverage matrices treat omission as implicit denial instead of a
  construction error
- explanation breadth or fallback debt silently widens beyond the declared
  locality/scope without becoming structured evidence
- explanation rows and evidence-origin rows collapse together so later
  provenance work cannot tell â€œwhy this was reportedâ€ from â€œwhere this came
  fromâ€
- partial diagnostics or certification surfaces claim simple admitted/denied
  status without explicit named gaps

## Dependencies On Earlier Milestones

Milestone 6 is downstream of earlier foundational work and must reuse it
explicitly.

### Milestone 2: Canonicalization

Milestone 2 remains the owner of canonical basis, digest slots, and
comparison/current-basis law. Milestone 6 may add diagnostics domains, entry
kinds, and row/bundle basis builders, but it must not invent a second
canonicalization dialect.

Use Milestone 2 for:

- canonical identity of diagnostic rows, bundles, and support reports
- deterministic row ordering and digest-preparation
- mismatch/equivalence basis for diagnostics comparison surfaces

Milestone 6 is not complete if:

- diagnostic rows or bundles invent a second digest/canonicalization dialect
- comparison or mismatch explanation depends on local ordering instead of the
  Milestone 2 canonical basis lane

### Milestone 3: Profiles

Milestone 3 remains the owner of profile meaning and reduced-richness law.
Milestone 6 must consume those profiles rather than reinterpreting them.

Use Milestone 3 for:

- diagnostic richness tiers and support/certification posture attachment
- central suppression of optional detail
- legality of attaching support/forensic/certification profiles to specific
  diagnostic surfaces

Milestone 6 is not complete if:

- reduced-richness diagnostic behavior is implemented as local ad hoc policy
  branching instead of Milestone 3 profile attachment/materialization law
- support/certification posture meaning drifts from the canonical profile
  families already standardized

### Milestone 4: Boundary Artifacts

Milestone 4 remains the owner of `Summary`, `Report`, `Artifact`, `Receipt`,
bundle law, materialization seams, delivery/availability law, and current-basis
attachments.

Use Milestone 4 for:

- diagnostic summaries versus reports versus bundles
- explicit materialization/delivery seams for explanation artifacts
- bundle legality and attachment points

Milestone 6 is not complete if:

- support reports, explanation bundles, failure bundles, or certified bundles
  reopen category law that Milestone 4 already standardized
- diagnostic materialization hides delivery/availability seams that Milestone 4
  already made explicit

### Milestone 5: Transitions

Milestone 5 remains the owner of branch/merge/commit authority meaning.
Milestone 6 must explain those surfaces, not recreate them.

Use Milestone 5 for:

- branch-local, merge, committed-authority, receipt, and discard diagnostic
  subjects
- transition locators, basis, and strategy-bearing facts where those shaped the
  diagnostic explanation
- trust-boundary/current-basis reuse where stronger transition claims already
  exist

Milestone 6 is not complete if:

- diagnostics for branch/merge/commit surfaces recreate transition meaning
  locally instead of consuming the Milestone 5 transition vocabulary
- diagnostics attached to committed-authority, receipt, or current-basis
  artifacts weaken or replace the existing Milestone 5 `worth-proof` lane

### Milestone 7: Provenance And Receipts Preparation

Milestone 7 is not implemented yet, but Milestone 6 must leave it solid
groundwork rather than cleanup debt.

Milestone 6 is not complete unless it prepares Milestone 7 by:

- separating explanation rows from future provenance rows explicitly now
- carrying evidence-reference posture strongly enough that later provenance can
  say whether evidence was retained, reconstructed, summarized, redacted, or
  absent
- distinguishing domain denial, policy denial, evidence absence, and
  construction/integrity breach now instead of making Milestone 7 rediscover
  them
- treating partial-but-honest coverage as a first-class pattern through named
  gaps so later receipt and certification work can reuse it

## WORTH-Proof Dependency Boundary

Milestone 6 is mostly descriptive ontology. It does not move ordinary
diagnostics into the proof kernel.

The mandatory rule is:

- plain diagnostics vocabulary stays local to `worth-foundational`
- proof-bearing artifacts from earlier milestones may attach diagnostics and
  explanation bundles
- any stronger claim that a diagnostic bundle is certified, current-basis, or
  readiness-complete must reuse the existing `worth-proof::Artifact` lanes
  rather than local booleans or wrapper folklore
- milestone readiness remains proof-bearing

`worth-proof` is mandatory for:

- Milestone 6 production-test readiness
- any certified diagnostic artifact that claims stronger current-basis or
  readiness posture beyond plain description
- trust-boundary weakening/readmission if a diagnostic bundle later exposes a
  stronger current-basis lane
- any diagnostic/support/explanation bundle that claims proof-bearing
  certification rather than plain descriptive coverage
- any Milestone 5 committed-authority, receipt, or current-basis transition
  artifact that carries Milestone 6 diagnostics while preserving its already
  stronger proof-bearing posture

`worth-proof` is forbidden for:

- plain diagnostic codes, scopes, severities, and artifact kinds
- plain outcome/advisory/denial vocabulary
- plain explanation rows and availability/delivery classes
- plain support reports and descriptive bundles

The operating rule is:

`worth-foundational` defines what diagnostics and explanation mean.`
`worth-proof` only carries stronger certification/current-basis claims when`
`those claims are real.`

Milestone 6 is therefore not allowed to improvise a second proof substrate for
diagnostics. If a surface needs stronger certification, current-basis, or
trust-boundary readmission meaning, it must reuse the existing
`worth-proof::Artifact` lane from earlier milestones. If it is only
descriptive, it must remain plain foundational vocabulary.

## Practical Type Targets

The implementation may choose better names, but these responsibilities must
exist concretely somewhere:

```rust
pub struct FoundationalDiagnosticCodeId { /* private fields */ }
pub struct FoundationalDiagnosticScopeId { /* private fields */ }

pub enum FoundationalDiagnosticSeverity {
    Info,
    Advisory,
    Warning,
    Denial,
    Failure,
    Violation,
}

pub enum FoundationalDiagnosticArtifactKind {
    Summary,
    Report,
    FailureBundle,
    ComparisonBundle,
    SupportReport,
    ExplanationBundle,
}

pub enum FoundationalDiagnosticDeliveryClass {
    MustBeHot,
    CanDefer,
    ReconstructableFromReplay,
    UnavailableByPolicy,
}

pub enum FoundationalDiagnosticAvailability {
    RetainedHot,
    DeferredCold,
    Reconstructable,
    Redacted,
    Unavailable,
}

pub enum FoundationalDiagnosticOutcomeKind {
    Accepted,
    Advisory,
    Denied,
    Unsupported,
    Deferred,
    Partial,
    Mismatch,
    Violation,
}

pub enum FoundationalDiagnosticAbsenceCause {
    NotRetained,
    RedactedByPolicy,
    UnsupportedSurface,
    ReconstructionDenied,
    MissingEvidence,
}

pub enum FoundationalDiagnosticBreachClass {
    ConstructionBug,
    IntegrityMismatch,
    CoverageOmission,
    CanonicalizationViolation,
}

pub enum FoundationalDiagnosticDenialClass {
    DomainDenied,
    PolicyDenied,
    UnsupportedDenied,
    EvidenceUnavailableDenied,
}

pub enum FoundationalDiagnosticEvidencePosture {
    RetainedDirect,
    Reconstructed,
    Summarized,
    Redacted,
    AbsentExpected,
}

pub struct FoundationalDiagnosticSubject { /* private fields */ }
pub struct FoundationalDiagnosticLocator { /* private fields */ }
pub struct FoundationalDiagnosticEvidenceDigest { /* private fields */ }

pub struct FoundationalDiagnosticSemanticLabelSet { /* private fields */ }

pub enum FoundationalDiagnosticRowFamily {
    Decision,
    Failure,
    Comparison,
    SupportEvidence,
    ProvenanceReadyEvidenceOrigin,
}

pub struct FoundationalDiagnosticRow {
    /* code, scope, severity, subject, locator, outcome, message class,
       availability, evidence digest, semantic labels, counters */
}

pub struct FoundationalDiagnosticDecisionRow { /* private fields */ }
pub struct FoundationalDiagnosticFailureRow { /* private fields */ }
pub struct FoundationalDiagnosticComparisonRow { /* private fields */ }
pub struct FoundationalDiagnosticProvenanceReadyRow { /* private fields */ }

pub struct FoundationalDiagnosticSupportReport { /* private fields */ }
pub struct FoundationalDiagnosticExplanationBundle { /* private fields */ }
pub struct FoundationalDiagnosticDeniedBundle { /* private fields */ }
pub struct FoundationalDiagnosticAdmittedBundle { /* private fields */ }

pub struct FoundationalDiagnosticCounterSnapshot { /* private fields */ }
pub struct FoundationalDiagnosticAssemblyReceipt { /* private fields */ }
pub struct FoundationalDiagnosticCoverageReceipt { /* private fields */ }

pub enum FoundationalDiagnosticGapClass {
    CoverageGap,
    EvidenceGap,
    HostileCoverageGap,
    LocalityGap,
    ConstructionGap,
}

pub enum FoundationalDiagnosticCoverageClass {
    HappyPathOnlyDenied,
    HostileCoveragePresent,
    CoverageIncompleteDenied,
    PartialWithNamedGaps,
}

pub struct FoundationalDiagnosticNamedGap {
    /* private fields */
}

pub struct FoundationalDiagnosticCanonicalBasisReady<T> { /* private fields */ }
```

These sketches imply concrete constraints:

- code, scope, severity, artifact kind, and availability remain separate
  concepts even if their storage is similar
- rows point at typed locators and subjects rather than producer-private string
  paths
- if a generic `FoundationalDiagnosticRow` exists at all, it is only an
  internal normalization surface; public APIs must expose family-distinct row
  types and must not let callers satisfy them with one shared body shape plus a
  `family` field
- support reports, explanation bundles, and denial bundles remain different
  categories
- evidence presence and evidence absence are both structured
- explanation rows and provenance-ready evidence-origin rows remain distinct
- denial class and breach class remain distinct
- partial coverage is a first-class typed state with named gaps, not an ad hoc
  string note
- every named gap must carry typed gap class, typed subject or locator, and
  typed closure posture; a human-readable label alone is not enough
- provenance-ready evidence-origin rows remain descriptive-only and must not
  directly satisfy future provenance or receipt APIs

## Naive Traps To Reject

- using free-form strings as the primary explanation surface
- using `Option` alone to represent redacted, missing, not-retained, and
  reconstructable evidence
- overclaiming durable, replay-safe, or certified support when only a runtime-
  local hot artifact exists
- allowing reduced-richness profiles to alter authoritative transition meaning
- letting the same report type stand in for support, failure, denial, and
  certified coverage
- forcing blind consumers to rescan runtime state or reconstruct semantics from
  logs
- using placeholder ids or zero digests when evidence is absent
- flattening preview discard, preview promotion, committed receipt, and report
  explanation into one "diagnostic event" bag
- letting diagnostic assembly rediscover strategy/basis semantics that earlier
  canonical artifacts already fixed
- treating missing family/lane/support rows as soft denials instead of a
  producer bug
- allowing locality-claim mismatch, widened fallout, or repeated rediscovery
  to survive only as counters with no typed explanation surface
- using one denial surface for domain refusal, policy refusal, unsupported
  scope, evidence absence, and structural reporting corruption
- making â€œpartial but honestâ€ bundles through comments or strings instead of
  one typed named-gap lane

- using one generic diagnostic row with a `family` tag and optional payload
  fields as the public API for decision, failure, comparison, support, and
  provenance-ready surfaces
- representing named gaps as free-form labels without typed gap family,
  subject, and closure posture
- letting certified, denied, and partial-with-named-gaps coverage collapse into
  one boolean plus optional comment

## Phases

These phases are implementation order, not a buffet. Each phase must leave a
concrete production responsibility home, a minimum public surface, and hostile
proof boundaries that the next phase is allowed to consume.

An engineer implementing this milestone should treat each phase as a gate:

- finish the named production surface for the current phase
- finish the compile-fail and hostile-proof bar for the current phase
- stop and verify that the next phase can consume the new surface without
  reopening primitive law from below
- do not begin the next phase by "temporarily" inventing categories, row
  shapes, bundle status, or proof posture that the current phase was supposed
  to freeze first

If a later phase reveals that an earlier phase needs correction, that
correction should be made as alignment work in service of the later phase. It
does not turn the milestone into a buffet or allow engineers to skip phase
gates opportunistically.

| Phase | What must be true before the next phase may begin |
| --- | --- |
| Phase 1 | Canonical diagnostic primitives and category law exist, and engineers can no longer confuse code/scope/severity/artifact-kind/delivery/availability. |
| Phase 2 | Outcome, absence, and explanation-row topology exists, and consumers can distinguish advisory, denial, missing evidence, redaction, replay reconstruction, and unsupported surfaces structurally. |
| Phase 3 | Materialization, support-report, and profile-richness law exist, and no caller can hide hot/deferred/reconstructable/unavailable boundaries behind cheap accessors. |
| Phase 4 | Canonical basis, locator, comparison, and blind-consumer bundle law exist, and reports/bundles can be digested and interpreted without producer folklore or rescans. |
| Phase 5 | Certified diagnostic/support/explanation bundle law exists, including hostile coverage, denial families, and attachment compatibility with transition and artifact surfaces. |
| Phase 6 | Production-test readiness evidence exists with exact certified surfaces, hostile pressures, compile-fail inventories, assumptions, non-assumptions, and debt. |
| Phase 7 | Developer-facing crate documentation exists for the shipped diagnostics surface, is added to crate docs, and matches the real implemented DX rather than milestone-plan prose. |

### Phase 1: Diagnostic Primitive And Category Law

Freeze the shared primitive nouns before any bundle or report work begins.
Engineers should finish this phase able to point at one canonical diagnostic
code/scope/severity/artifact-kind/delivery/availability language and explain
why each is not interchangeable with the others.

This phase is the floor. Do not start row topology, report construction, or
support/certification surfaces until these primitive families are mechanically
 frozen. The point is to remove the temptation to let later rows or bundles
 invent primitive meaning ad hoc.

Practical implementation order:

1. Define typed code, scope, severity, artifact-kind, delivery-class, and
   availability vocabulary.
2. Define the minimum legality law connecting artifact kinds to delivery and
   availability classes.
3. Add compile-time boundaries preventing obvious primitive substitution.
4. Add canonical ordering rules for primitive sets and labels.
5. Freeze denial-versus-breach and evidence-posture as separate primitive
   families rather than later bundle decoration.

Phase 1 is complete only when:

- diagnostic codes/scopes/severities/artifact kinds are typed and canonical
- delivery class and availability are explicit and non-substitutable
- primitive equality/canonicalization does not depend on insertion order or
  string formatting
- generic "severity plus string" fallback construction is impossible or
  fail-closed
- denial class, breach class, and evidence posture are distinct primitive
  meanings rather than ad hoc row payload fields

Acceptance evidence:

- primitive parity tests across independent producers
- compile-fail tests preventing primitive substitution
- hostile tests proving delivery and availability stay explicit rather than
  ambient
- parity tests proving denial class, breach class, and evidence posture cannot
  collapse

### Phase 2: Outcome, Absence, And Explanation Row Topology

Build the shared row and outcome grammar after the primitive nouns are frozen.
This phase must make success/advisory/denial/unsupported/deferred/mismatch/
violation meaning concrete, and it must do the same for missing evidence,
redaction, and reconstruction posture.

This phase consumes Phase 1 primitives and is not allowed to reopen them. The
implementation job here is to create the first honest row topology engineers
can build on later. Do not start support reports, canonical bundles, or
coverage classes until family-distinct row law, absence law, and row-family
legality are closed.

Practical implementation order:

1. Define outcome families and absence causes.
2. Define diagnostic subjects and typed locators that rows can point at.
3. Define structured row types for decision, failure, comparison, and support
   evidence.
4. Freeze row ordering, required fields, and semantic-label carriage.
5. Add explicit room for locality, widened-fallout, and construction-bug
   postures where the producer cannot honestly collapse them into denial.
6. Separate explanation rows from provenance-ready evidence-origin rows so
   later Milestone 7 provenance law can attach without reopening diagnostics.
7. Make row-family legality mechanical enough that public APIs cannot accept
   one generic row shape plus optional payloads where the type system already
   knows the family.

Phase 2 is complete only when:

- blind consumers can distinguish accepted, advisory, denied, unsupported,
  deferred, partial, mismatch, and violation outcomes mechanically
- blind consumers can distinguish not-retained, redacted, unsupported,
  reconstruction-denied, and missing-evidence absence mechanically
- rows point at typed subjects/locators, not producer-private strings
- preview/branch/merge/commit explanations can name non-authoritative versus
  authoritative subjects without re-deciding authority
- locality mismatch, widened fallout, and construction-bug omission can be
  represented without being flattened into ordinary denial
- evidence-origin posture is structurally visible on the row where it matters
- explanation rows and provenance-ready rows are not one generic row shape with
  optional semantics hidden in payloads
- row-family legality is strong enough that callers cannot satisfy
  decision/failure/comparison/support/provenance-ready lanes with one shared
  public row wrapper when the family is statically known

Acceptance evidence:

- topology tests proving outcome families remain distinct
- absence-cause tests proving `None`-style collapse is impossible
- locator/subject parity tests across independent producers
- compile-fail tests preventing raw string locator substitution where the API
  can enforce it
- tests proving omitted matrix rows are construction bugs rather than implied
  denial
- tests proving explanation rows and provenance-ready rows remain distinct
- compile-fail tests preventing public generic-row substitution for
  family-distinct row APIs

### Phase 3: Materialization, Support Report, And Profile-Richness Law

Add the real materialization and support-report surfaces only after rows and
outcomes exist. This phase owns the hot/deferred/reconstructable/unavailable
boundary and the reduced-richness rule that breadth may change without truth
changing.

This phase is where explanation stops being just vocabulary and becomes an
expensive, inspectable boundary. Do not begin canonical bundle work or
certified coverage work until support reports, explanation bundles, profile
attachment, and named-gap partiality are already honest on their own terms.

Practical implementation order:

1. Define support-report and explanation-bundle categories.
2. Define materialization receipts/counter snapshots and attach profile-driven
   richness decisions explicitly.
3. Define availability transitions for retained, deferred, reconstructable,
   redacted, and unavailable evidence.
4. Add denial/posture vocabulary for overclaiming retained or certified support.
5. Make repeated rediscovery, row-scan fallback, and whole-view fallback
   explicit debt or denial postures rather than invisible implementation
   choices.
6. Add named-gap vocabulary for partial-but-honest support and diagnostic
   surfaces so partial closure never has to fake binary status.
7. Freeze the minimum named-gap shape so later code cannot get away with a
   free-form label where typed gap class, subject, and closure posture are
   already known.

Phase 3 is complete only when:

- no report/explanation API hides whether detail was retained hot, deferred,
  reconstructed, redacted, or unavailable
- support reports cannot overclaim durable, replay-safe, or certified support
  not actually present
- reduced-richness profiles remove only optional detail
- diagnostic assembly cannot silently rescan broad runtime state instead of
  consuming already-canonical evidence
- repeated rediscovery and widened fallback breadth are either denied or named
  as structured debt/counter evidence
- partial support and diagnostic closure can be expressed honestly with named
  gaps instead of bluffing full support or degrading into generic denial
- named gaps carry typed gap class, typed subject or locator, and typed closure
  posture rather than only prose labels

Acceptance evidence:

- richness hostility tests proving truth stays unchanged
- support overclaim rejection tests
- reconstruction/unavailability tests
- misuse-pressure tests attacking cheap convenience helpers and hidden rescans
- fallback-debt and repeated-rediscovery tests proving those costs remain
  explicit
- partial-support and named-gap tests
- named-gap shape tests proving labels alone are insufficient

### Phase 4: Canonical Basis, Comparison, And Blind-Consumer Bundle Law

Once support/materialization law is stable, lower diagnostic rows and bundles
through canonical basis and freeze blind-consumer interpretation rules.

This phase is only about making already-honest rows and bundles canonical,
portable, and blind-consumer readable. It is not allowed to hide unfinished
support/materialization law behind canonicalization work. If a bundle still
needs producer folklore to explain itself, Phase 3 is not done yet.

Practical implementation order:

1. Add diagnostics domains and entry kinds to the Milestone 2 canonicalization
   lane.
2. Define canonical row/bundle ordering and basis participation.
3. Add comparison bundles and mismatch-basis surfaces for diagnostics.
4. Define blind-consumer bundle interpretation rules and exact evidence-floor
   expectations.
5. Define how partial certification or partial hostile coverage is represented
   with named gap labels instead of vague bundle status.
6. Freeze evidence-reference posture participation in canonical basis so
   retained versus reconstructed versus summarized evidence remains replay-safe.
7. Freeze canonical participation for named-gap structure so partial coverage
   cannot reorder or selectively omit gap meaning across independent producers.

Phase 4 is complete only when:

- semantically identical diagnostic rows/bundles digest the same way across
  independent producers
- comparison bundles preserve mismatch basis explicitly
- bundles remain interpretable without producer-private state
- missing evidence is represented structurally, not via placeholder values
- partial coverage and named gaps are structurally representable without
  pretending the bundle is either fully certified or fully denied
- evidence-reference posture is canonical enough for later provenance and
  receipt work to reuse without reinterpretation
- named-gap meaning is canonical enough that blind consumers can compare
  partial-with-named-gaps bundles without producer-private label folklore

Acceptance evidence:

- canonical-basis parity tests
- diagnostic comparison/mismatch tests
- blind-consumer interpretation tests
- exact row-order and bundle-order tests
- partial-coverage and named-gap interpretation tests
- evidence-posture parity tests
- named-gap canonical-order and parity tests

### Phase 5: Certified Bundle And Attachment Compatibility Law

Only after canonical bundle law exists should the milestone add certified
coverage surfaces and attachment compatibility to transitions, boundary
artifacts, and stronger proof-bearing artifacts.

This phase is where stronger claims finally appear, so it must be especially
strict about not re-deciding ordinary diagnostics meaning locally. The
implementation job is to attach certification, hostile coverage, proof-lane
reuse, and provenance-ready hooks to already-canonical descriptive bundles.
Do not start readiness closeout until certified-versus-partial-versus-denied
coverage is mechanically closed and attachment legality is proven.

Practical implementation order:

1. Define admitted/denied/support/certified diagnostic bundles and coverage
   classes.
2. Define attachment law for boundary-artifact and transition surfaces.
3. Add fail-closed denials for fake coverage, missing hostile rows, missing
   source digests, or category collapse.
4. Reuse existing `worth-proof::Artifact` lanes only where bundles claim
   stronger certification/current-basis meaning.
5. Freeze distinction between certified, partial-with-named-gaps, and denied
   coverage so closeout does not collapse them.
6. Add explicit provenance-ready attachment hooks rather than forcing Milestone
   7 to tunnel provenance semantics through explanation-only bundles.
7. Define coverage-matrix legality strongly enough that when required row
   families are statically known, omitted required rows and illegal
   certified-versus-partial construction are unrepresentable or uncompilable,
   not merely reviewer-visible.

Phase 5 is complete only when:

- hostile coverage is mandatory for certified bundles
- attachment to transition and artifact surfaces preserves category honesty
- report/support/explanation/failure bundles cannot impersonate each other
- stronger certified/current-basis bundles reuse existing proof lanes instead
  of local pseudo-proof wrappers
- partial coverage cannot masquerade as certified coverage unless its gaps are
  explicitly named and typed
- provenance-ready attachment hooks exist without collapsing provenance into
  explanation or diagnostics into receipts
- coverage construction is strong enough that a bundle cannot claim
  `HostileCoveragePresent` or equivalent certified posture while omitting
  required family rows or surfacing only gap labels without typed gap records

Acceptance evidence:

- hostile coverage omission tests
- attachment compatibility tests
- compile-fail tests preventing fake certified bundle construction
- proof-lane tests proving plain diagnostics stay descriptive while stronger
  certification lanes reuse `worth-proof`
- hostile tests proving partial coverage without named gaps is rejected
- attachment tests proving provenance-ready hooks do not redefine explanation
  rows as provenance rows
- compile-fail or typestate tests preventing illegal certified/partial coverage
  matrix construction where the required row families are statically known

### Phase 6: Production-Test Readiness

Close the milestone with the same standard used for Milestones 3 through 5: a
concrete, proof-bearing readiness artifact with exact certified surfaces and
exact evidence rows.

This phase is not an afterthought and not a summary document pass. The
implementation job is to freeze the exact closure contract downstream runtimes
and Milestone 7 may rely on. If any certified surface, hostile pressure,
compile-fail boundary, or residual debt item is still discoverable only by code
archaeology, the milestone is not ready to close.

Practical implementation order:

1. Inventory every certified diagnostics surface and every hostile pressure.
2. Inventory every compile-fail boundary and every canonical golden artifact.
3. Freeze assumptions, non-assumptions, runtime-adoption failure pressures, and
   harness expansion points.
4. Certify readiness through the chosen proof lane.

Phase 6 is complete only when:

- every certified surface has one concrete evidence row
- every hostile pressure has one concrete owning test path
- the readiness artifact names what downstream runtimes may and may not assume
- the milestone has a stable human-facing closeout and machine-facing readiness
  artifact

Acceptance evidence:

- readiness certification tests
- compile-fail tests for readiness-only stronger claims
- golden inventory verification
- exact assumptions/non-assumptions/debt inventory checks

### Phase 7: Feature Documentation And Crate-Docs Integration

Close the milestone with real developer-facing documentation after the
implementation and readiness surface are stable. This phase exists so the
finished diagnostics API is taught as an actual crate capability rather than
remaining discoverable only through milestone specs, closeout docs, and code
archaeology.

This phase must use the `feature-doc-writer` skill when producing the final
developer-facing documentation surface.

Practical implementation order:

1. Identify the stable public diagnostics/explanation surfaces that shipped in
   Phases 1 through 6.
2. Write feature documentation that teaches the common path, advanced plan
   path, partial-with-named-gaps path, certified path, and proof-lane
   boundaries using the real crate API.
3. Add that documentation to the crateâ€™s developer-facing docs in the
   appropriate documentation home instead of leaving it only in milestone or
   closeout files.
4. Verify that examples, terminology, and guarantees match the implemented
   facade and not earlier draft naming from the milestone plan.
5. Cross-check the new crate docs against the readiness artifact so certified
   claims, non-assumptions, and residual debt are not overstated.

Phase 7 is complete only when:

- the shipped diagnostics surface is documented in crate-facing docs rather
  than only milestone planning docs
- the documentation teaches the real DX lanes the milestone standardized:
  common explanation, inspectable planning, explicit availability/absence,
  partial-with-named-gaps, and stronger certified/readmitted paths
- the documentation makes descriptive-versus-stronger-proof-bearing boundaries
  explicit
- the documentation matches the actual public API and crate topology
- downstream engineers can learn how to use the milestone from crate docs
  without first reading milestone-planning prose

Acceptance evidence:

- published crate-doc additions for the diagnostics/explanation surface
- doc examples checked against the real public API
- terminology parity check against the final readiness artifact and closeout
- explicit confirmation that `feature-doc-writer` was used for the final
  feature-doc pass

## What Must Ship

- typed diagnostic primitives for code, scope, severity, artifact kind,
  delivery class, and availability
- typed outcome families for accepted, advisory, denied, unsupported,
  deferred, partial, mismatch, and violation
- typed absence causes for not-retained, redacted, unsupported,
  reconstruction-denied, and missing-evidence
- typed diagnostic subjects and locators
- structured diagnostic rows for decision, failure, comparison, and support
  evidence
- explicit evidence-posture vocabulary
- explicit denial-class versus breach-class vocabulary
- explicit named-gap vocabulary for partial-but-honest surfaces
- support reports and explanation bundles with explicit evidence-floor law
- profile-aware materialization, availability, and support posture
- canonical-basis participation for rows and bundles
- comparison/mismatch bundles for diagnostics parity work
- certified bundle and hostile-coverage vocabulary
- proof-bearing readiness artifact
- developer-facing crate documentation for the shipped diagnostics surface

## Semantic Guarantees

- diagnostics remain descriptive and do not redefine authoritative truth
- the same code/scope/severity/artifact-kind/outcome combination means one
  thing everywhere
- missing, redacted, unsupported, deferred, reconstructable, and unavailable
  evidence remain distinct
- support, failure, comparison, explanation, and certified-coverage bundles
  remain distinct categories
- profile-richness changes diagnostic breadth only
- blind consumers can interpret rows and bundles without producer-private state
- explanation and provenance-ready evidence-origin remain distinct semantic
  surfaces
- partial-but-honest status remains explicit rather than collapsing into binary
  admitted/denied folklore

## Representation Boundaries

- crates remain free to retain diagnostics in AoS, SoA, AoSoA, sparse, packed,
  or custom forms
- foundational diagnostics standardize boundary meaning, not one diagnostics
  store, one evidence index, or one replay engine
- foundational diagnostics do not own transition meaning, query-family
  taxonomies, graph internals, or bridge record layout
- support/certification bundles standardize semantics, not one QA harness or
  one runtime certification registry

## Must Preserve

- diagnostics stay descriptive, not authoritative
- reduced-richness profiles cannot change truth
- hot/deferred/reconstructable/unavailable boundaries stay explicit
- preview discard and other non-authoritative closeout evidence cannot
  masquerade as committed authority or authoritative receipts
- canonical bundle interpretation never depends on producer folklore, placeholder
  ids, or broad rescans
- denial, breach, and absence remain distinguishable
- partial coverage cannot masquerade as closure

## Desired DX End State

Milestone 6 should not finish as "some enums and a report struct." It should
finish as a layered explanation surface where the common path reads like
"explain this thing under this profile," the lower path exposes evidence
availability and assembly cost, and the stronger certified lane is visibly
stronger.

The finished code should read like intent at the top, inspectable plan in the
middle, and visibly stronger certification only where stronger claims are
real. The API should not make callers assemble row bags manually or recover
meaning from debug strings.

The common path should feel like asking for explanation at a named semantic
target:

```rust
let report = diagnostics::support_report()
    .for_transition(&commit_receipt)
    .under_profile(profile)
    .materialize()?;

report.subject();
report.coverage_class();
report.rows();
report.counter_snapshot();
```

```rust
let bundle = diagnostics::explain()
    .for_transition(&merge_verdict)
    .at(locator)
    .under_profile(profile)
    .materialize()?;

bundle.outcome_kind();
bundle.availability();
bundle.subject();
bundle.rows();
bundle.counter_snapshot();
bundle.explain();
```

The advanced path should keep cost, locality, and evidence posture explicit
before materialization:

```rust
let plan = diagnostics::explain()
    .for_transition(&committed)
    .at(locator)
    .under_profile(profile)
    .plan()?;

plan.delivery_class();
plan.availability();
plan.subject();
plan.locality_claim();
plan.widened_fallout_posture();
plan.evidence_posture();
plan.retained_evidence_count();
plan.reconstructable_evidence_count();
plan.redacted_evidence_count();
plan.coverage_class();
plan.named_gaps();
plan.counter_snapshot();
plan.explain();
```

The row-level surface should feel family-distinct and typed, not like one
generic event body:

```rust
for row in bundle.rows() {
    row.code();
    row.scope();
    row.subject();
    row.locator();
    row.outcome_kind();
}

for row in bundle.decision_rows() {
    row.denial_class();
    row.evidence_posture();
}

for row in bundle.provenance_ready_rows() {
    row.evidence_origin_locator();
    row.evidence_posture();
}
```

The absence and availability path should make hostile missing-detail cases
beautifully explicit instead of collapsing into `Option`:

```rust
match bundle.availability() {
    RetainedHot(hot) => inspect(hot),
    DeferredCold(cold) => request_materialization(cold)?,
    Reconstructable(recipe) => rebuild(recipe)?,
    Redacted(redaction) => explain_redaction(redaction),
    Unavailable(cause) => explain_unavailability(cause),
}
```

Partial-but-honest support should feel like a first-class descriptive state,
not a disappointed boolean:

```rust
let support = diagnostics::support_report()
    .for_transition(&discard_receipt)
    .under_profile(profile)
    .materialize()?;

match support.coverage_class() {
    HostileCoveragePresent(coverage) => use_certified_support(coverage),
    PartialWithNamedGaps(gaps) => {
        for gap in gaps {
            gap.gap_class();
            gap.subject();
            gap.closure_posture();
        }
    }
    CoverageIncompleteDenied(denial) => explain_denial(denial),
    HappyPathOnlyDenied(denial) => explain_denial(denial),
}
```

The support/certification path should make stronger claims visibly stronger:

```rust
let certified = diagnostics::certified_bundle()
    .for_transition(&receipt)
    .under_profile(profile)
    .with_hostile_coverage(hostile_rows)
    .materialize()?;

certified.coverage_class();
certified.proofs();
certified.rows();
```

The stronger path should also make trust-boundary weakening and readmission
visible instead of ambient:

```rust
let bridged = certified.bridge_trust_boundary();

let readmitted = diagnostics::certified_bundle()
    .readmit_with_authority(bridged, authority_witness)?;
```

The API should make several bad shapes impossible or at least obviously wrong:

- no `Report::new(code, message, true, false)` positional construction
- no generic `DiagnosticEvent { status, severity, body }` bags that stand in
  for support/failure/comparison/explanation/certified bundles
- no public "one row plus family enum" authoring lane that can stand in for
  decision/failure/comparison/support/provenance-ready rows
- no ambient reconstruction or redaction decisions hidden behind plain
  `Option`
- no support-report helpers that silently overclaim durable/certified meaning
- no explanation helpers that rescan broad runtime state after canonical
  evidence already exists
- no "partial support" or "partial coverage" construction that accepts only a
  string note where a typed named-gap record is required
- no public API that forces callers to build family rows, named gaps, or
  coverage matrices by hand from primitive strings and booleans when the
  framework already knows the semantic slot

When this milestone is done well, engineers should naturally write code that:

- names the diagnostic subject explicitly
- chooses a profile explicitly
- plans before expensive explanation work when cost or breadth matters
- observes retained versus reconstructed versus redacted versus unavailable
  evidence structurally
- handles partial coverage as a real state with typed gaps
- uses a visibly stronger lane when certification or current-basis claims are
  actually being made

## Acceptance Evidence

- diagnostic primitive parity tests
- compile-fail tests preventing primitive/category substitution
- outcome-topology tests proving accepted/advisory/denied/unsupported/deferred/
  partial/mismatch/violation remain distinct
- absence-cause tests proving redacted/not-retained/missing-evidence do not
  collapse
- profile-richness hostility tests proving breadth changes do not change truth
- support overclaim rejection tests
- hot/deferred/reconstructable/unavailable materialization tests
- blind-consumer interpretation tests
- canonical-basis parity tests across independent producers
- certified-bundle hostile-coverage tests
- attachment compatibility tests for boundary artifacts and transitions
- generic-row collapse rejection tests
- named-gap structure and certified-versus-partial coverage legality tests
- readiness artifact tests covering certified surfaces, hostile pressures,
  compile-fail boundaries, assumptions, non-assumptions, and debt
- developer-facing crate-doc verification for the shipped diagnostics surface

## Architectural Notes

The implementation should preserve distinct diagnostics responsibility homes. A
likely shape is:

```text
crates/worth-foundational/src/
  diagnostics/
    vocabulary/
    outcomes/
    materialization/
    bundles/
    basis/
    readiness/
```

Public exports should remain facade-controlled. The root may exist, but it must
not become an unnamed bucket where primitives, row topology, support reports,
comparison bundles, and readiness reporting collapse into one file.

## Sequencing Notes

Milestone 6 belongs immediately after Milestone 5 because diagnostics needs the
new branch/merge/commit vocabulary before it can explain those outcomes
honestly.

It must remain after Milestone 4 because it consumes artifact categories,
bundle law, materialization seams, and delivery/availability meaning.

It must remain before Milestone 7 because provenance, lineage, and receipt
deepening need one diagnostics and explanation ontology to attach to instead of
reinventing local support/report dialects.

## Explicit Non-Goals

- one diagnostics runtime or one diagnostics store
- one replay engine or one retained-evidence index
- one support/certification harness runtime
- one query-family, graph, bridge, or relational diagnostics taxonomy
- full provenance and lineage ontology
- full cross-runtime causal inspection workflow for every adopting crate
- replacing Milestone 2 canonicalization, Milestone 3 profile law, Milestone 4
  artifact law, or Milestone 5 transition law

## Self-Check

- Does this milestone solve a real structural problem rather than packaging
  work cosmetically? Yes. It closes the shared explanation boundary that
  runtimes already need and currently describe in incompatible local dialects.
- Is the adversarial constraint precise and load-bearing? Yes. It attacks
  hidden rescans, missing-evidence collapse, hot/deferred confusion,
  overclaimed support, redaction ambiguity, and fake stronger claims.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns shared diagnostics meaning while runtimes keep
  capture, storage, replay, and domain-specific semantics.
- Does the milestone define proof obligations rather than implementation
  chores? Yes. Closure requires hostile richness tests, category hostility,
  blind-consumer interpretation, coverage hostility, compile-fail boundaries,
  and readiness evidence.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to primitives, rows, materialization,
  canonicalization, certified bundles, and readiness.
- Does the milestone belong in this roadmap sequence? Yes. It depends on
  canonicalization, profiles, boundary artifacts, and transitions, and it is a
  prerequisite for provenance/lineage deepening and migration closure.
