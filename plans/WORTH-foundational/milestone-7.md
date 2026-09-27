# Milestone 7: Lineage, Provenance, And Receipt Vocabulary

## Goal

Define one shared language for lineage, provenance, receipts, support-truth,
and planned-versus-executed boundary explanation so WORTH crates can describe
continuity, derivational context, completed authority/effectful boundaries,
support-grade evidence, and degraded or partial recovery one way everywhere
without collapsing them into metadata folklore, event-log theater, or one
ambiguous artifact envelope.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and solving the hostile case
before the happy path. The shaping constraint is that Milestone 7 must solve
ambiguous continuity, branch-local divergence, replay-derived support, partial
evidence, denied reconstruction, and completed-versus-planned boundary honesty
first instead of assuming a simple linear history.

### `arch_laws.md`

Protects authority-versus-description separation, self-describing boundary
artifacts, explicit proof-bearing strengthening, and honesty about what a
surface can and cannot claim. The shaping constraint is that lineage,
provenance, receipts, and support-truth must remain typed and composable
without letting descriptive surfaces impersonate authority or proof.

### `composition_laws.md`

Protects responsibility-shaped files, explicit named steps, and narrow helpers.
The shaping constraint is that locality vocabulary, lineage families,
provenance layers, receipt classes, support-truth bundles, attachment law, and
readiness evidence must live in separate responsibility homes rather than one
large history/provenance dump.

### `domain_structure_laws.md`

Protects structure as domain topology rather than convenience filing. The
shaping constraint is that branch divergence, replay provenance, receipt
attestation, degraded recovery, support parity, and readmission boundaries must
be independently locatable and testable.

### `perf_laws.md`

Protects cost-honest surfaces, explicit locality, and planning/materialization
separation. The shaping constraint is that replay-derived, snapshot-bound,
branch-local, historical, comparison-paired, restored, and reconstructed
surfaces must remain explicit postures; no cheap-looking accessor may hide
history scans, replay rebuilds, checkpoint walks, or broad provenance
assembly.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` owns shared boundary meaning
while preserving crate-local execution and storage. The shaping constraint is
that Milestone 7 must standardize lineage/provenance/receipt/support-truth
meaning, not one journal layout, one event store, one executor, one replay
engine, or one persistence model.

### `worth_foundational_roadmap.md`

Protects sequencing. The shaping constraint is that Milestone 7 follows
boundary artifacts, transitions, and diagnostics, and must therefore consume
artifact taxonomy, current-basis law, branch/merge/commit vocabulary, and
provenance-ready diagnostic distinctions rather than reopening them.

### `test-requirements.md`

Protects local hostile proof before adopting-crate migration. The shaping
constraint is that Milestone 7 must locally prove category separation,
planned-versus-executed law, blind-consumer interpretability, digest and basis
attachment honesty, reduced-richness preservation, degraded recovery honesty,
compile-fail boundaries, and proof-composable strengthening before any runtime
crate is allowed to depend on it.

### `milestone-4.md`

Protects boundary artifact categories, materialization law, canonical-basis
participation, and current-basis proof-bearing readmission. The shaping
constraint is that Milestone 7 must attach lineage/provenance/receipt/support
meaning to those artifact families instead of inventing its own independent
artifact taxonomy.

### `milestone-5.md`

Protects typed branch-local, merge, committed-authority, receipt, locator, and
current-basis vocabulary. The shaping constraint is that Milestone 7 must
consume canonical parentage, deltas, merge outcomes, strategy basis, and
commit receipts instead of rediscovering transition law locally.

### `milestone-6.md`

Protects diagnostics/explanation distinctions that Milestone 7 now depends on.
The shaping constraint is that provenance-ready rows, evidence-retained versus
reconstructed posture, denial/absence/breach distinction, and named-gap
partiality must be reused directly instead of being flattened back into
receipt or support prose.

## Existing Runtime Patterns

Milestone 7 is intentionally shaped by the hard boundary surfaces already
shipped in other WORTH crates.

### `worth-query`

What to keep:

- identity evolution results are family-distinct: exact continuity, plural
  successors, advisory candidates, ambiguity, break, and denial are not one
  result bag
- basis locality is explicit: branch-local, historical, comparison, and replay
  contexts are named instead of ambient
- correspondence is descriptive while identity continuity remains the stronger
  claim
- branch-local divergence and promotion posture are explicit instead of being
  inferred from timestamps or ids

What to prune:

- query-specific dialect around correspondence planning and local naming should
- not become foundational vocabulary wholesale
- foundational should not adopt query's executor or index assumptions

### `worth-store`

What to keep:

- support-grade truth can be authoritative for its declared support role while
  still remaining subordinate to stronger authority surfaces
- degraded recovery, rebuild, quarantine, checkpoint, resume, and restart
  postures are first-class and typed
- support parity, residual debt, and machine-checkable closeout matter as much
  as the happy-path persistence story
- support publication receipts and restore/readmission surfaces are distinct

What to prune:

- store-specific persistence layout, checkpoint topology, and cursor storage
  should not become foundational representation
- foundational should not imply one durable-runtime or one support publisher

### `worth-relational`

What to keep:

- history shape, merge ontology, planning, and execution are separate families
- branch-local and authority-bearing merge surfaces must remain distinct
- lowered decisions such as execute, block, and reject are clearer than one
  generic lifecycle result
- causal and policy context are inputs to explanation and receipt, not
  substitutes for them

What to prune:

- relational-specific dataflow, planner, and merge-policy topology should not
  become foundational layout
- foundational should standardize the boundary meaning, not one merge runtime

### `worth-runtime-bridge`

What to keep:

- source declaration, capability admission, planning, packet materialization,
  retained failure, replay, and support publication are separate phases
- read-side source provenance is a first-class protocol, not a comment
- packets and materialized reads need completed-boundary receipts, not just
  "attempted" summaries
- restored and replay-safe source records must remain basis-explicit

What to prune:

- bridge-local preview/writeback/session naming should not become the shared
  language
- foundational should not imply one adapter or bridge execution model

### `worth-signal`

What to keep:

- history, lineage, replay, merge, diagnostics, and provenance-retention are
  related but not identical questions
- user-facing simple front doors can exist on top of rich basis, snapshot,
  branch, and replay machinery
- support and diagnostics richness can be budgeted without changing
  authoritative outcomes
- provenance-retention policy is explicit instead of silently dropping context

What to prune:

- signal-specific graph/runtime vocabulary should not become foundational law
- foundational should not adopt one runtime history container or one replay
  planner

## Why This Milestone Exists

Milestones 4, 5, and 6 made it possible to say:

- what category an artifact belongs to
- what transition or authority boundary happened
- what diagnostics exist, what evidence exists, and what is missing

But WORTH still lacks one shared language for the next layer of questions:

- what continuity claim is being made
- under what basis and locality was that claim or artifact produced
- what completed boundary actually happened versus what was only planned
- what support-grade or certification-grade truth exists around that boundary
- what replay, restore, checkpoint, or degraded-recovery posture applies

Without Milestone 7, each crate can still implement those ideas locally, but
cross-crate consumers will have to relearn them from dialect, field names, and
private assumptions. That is exactly the kind of folklore `worth-foundational`
exists to eliminate.

## Adversarial Constraint

Multiple WORTH crates with different storage, replay, persistence, and
execution models must be able to attach lineage, provenance, receipts,
support-truth, and degraded-recovery explanation to the same canonical
boundary-facing artifacts across branch-local, historical, snapshot-bound,
comparison-paired, replay-derived, and restored/readmitted contexts without:

- collapsing continuity, provenance, and completed-boundary evidence into one
  envelope
- hiding replay/reconstruction breadth behind cheap-looking APIs
- confusing support-grade truth for stronger authority truth
- letting planned intent impersonate executed reality
- requiring consumers to know producer-private storage or runtime internals

## Dependencies On Earlier Milestones

Milestone 7 depends on earlier milestones mechanically, not just conceptually.

- Milestone 1 supplies aspect-native values, locators, masks, authoritative
  state, and compatibility lowering so provenance and receipt attachments can
  name stable shared vocabulary instead of strings.
- Milestone 2 supplies canonical basis, mismatch basis, export bundles, digest
  participation, and grouped public lanes so lineage/provenance/receipt/support
  artifacts can participate in stable boundary identity and blind-consumer
  parity.
- Milestone 3 supplies profiles, attachment legality, materialization, and
  certified/readmitted profile posture so richness, elision, and support-truth
  can be typed instead of ambient policy.
- Milestone 4 supplies boundary artifact categories and current-basis
  readmission so lineage/provenance/receipt attachments have a lawful target.
- Milestone 5 supplies branch-local, merge, committed-authority, delta,
  parentage, receipt, and current-basis transition nouns.
- Milestone 6 supplies provenance-ready diagnostics rows, evidence posture,
  named-gap partiality, and explanation/support honesty that Milestone 7 must
  reuse instead of bypassing.

## WORTH-Proof Dependency Boundary

Milestone 7 remains in `worth-foundational`, not `worth-proof`.

What belongs here:

- descriptive lineage vocabulary
- descriptive provenance vocabulary
- descriptive receipt vocabulary
- support-truth and certification-summary vocabulary
- locality, planned-versus-executed, degraded-recovery, and residual-debt
  postures
- attachment and materialization law for those surfaces

What may strengthen through `worth-proof`:

- proof-bearing certification that a lineage/provenance/receipt/support bundle
  is attached to a stronger authority artifact
- readmission of current-basis or support bundles across trust boundaries
- proof-bearing readiness artifacts and support closeout artifacts

What must not move into `worth-proof`:

- ownership of the shared descriptive vocabulary itself
- crate-local execution, persistence, replay, or history runtime behavior

## Practical Type Targets

Milestone 7 should be designed around the following shared families.

- locality primitives:
  - current
  - branch-local
  - historical
  - comparison-paired
  - snapshot-bound
  - replay-derived
  - restored/readmitted
- lineage outcome families:
  - singular continuity
  - plural successor/predecessor
  - merge successor
  - branch-local replacement
  - restored continuity
  - reconstructed equivalence
  - named-gap partial continuity
  - withheld/redacted continuity
  - transient-within-boundary closure
  - advisory correspondence candidate
  - ambiguity
  - identity break
  - denial
- provenance layers:
  - source basis
  - authority path
  - strategy basis
  - profile basis
  - comparison basis
  - canonical/digest basis
  - retention/freshness posture
  - support-context attachments
- receipt families:
  - admission
  - planning
  - execution
  - publication
  - restoration
  - support publication
  - checkpoint/resume
  - denial/blocked closeout
- support-truth families:
  - evidence bundles
  - certification summaries
  - parity artifacts
  - complexity/counter evidence
  - degraded-recovery reports
  - stale-basis disclosures
  - transient lifecycle evidence
  - residual-debt statements
- composition/lifecycle families:
  - symbolic resolution maps
  - family-distinct lifecycle outcomes
  - branch-divergence posture
  - promotion posture
  - object-level versus locator-level continuity attachments
  - same-family or mixed-authority closeout receipts

The minimal tangible artifact surface should be rich enough to express:

- a lineage claim artifact without provenance or receipt substitution
- a provenance artifact without lineage or receipt substitution
- an executed-boundary receipt artifact without planning substitution
- a support-truth artifact without stronger authority substitution
- attachment bundles that can carry these families together without collapsing
  them into one generic record

## Practical Boundary Scenarios

Milestone 7 is not complete unless the vocabulary can be used honestly in at
least these scenario families:

- a branch-local entity is promoted into globally admitted continuity with
  explicit pre-promotion branch divergence, promotion posture, committed
  receipt, and post-promotion lineage
- a replay-derived support bundle reconstructs likely continuity from retained
  snapshots and replay slices while remaining explicitly replay-derived and
  support-grade
- a checkpoint restore readmits identity with enough evidence to distinguish
  direct restored continuity from reconstructed equivalence
- a blocked or denied execution still emits typed planning/closeout/support
  receipts without impersonating execution truth
- a create-then-delete or introduce-then-retire participant is preserved as
  transient executed/support evidence without becoming durable lineage
- a pointer/path/field relocation is expressed as locator-level continuity
  without pretending the whole object changed identity
- a profile-elided or redacted result preserves authoritative truth while
  resolving predecessor/successor detail into named-gap, withheld, or denied
  continuity posture
- a stale snapshot or reduced retained basis still yields usable support-truth
  with explicit freshness and retention disclosure
- the same canonical boundary artifact carries provenance, receipt, digest
  basis, and diagnostics attachment without category collapse

## Naive Traps To Reject

- one universal provenance envelope that tries to mean lineage, origin,
  parentage, support, replay, and receipt all at once
- receipt surfaces that record intention or attempted work but not completed
  boundary truth
- lineage reduced to generic event logging or timestamp chains
- replay-derived continuity silently upgraded into retained or directly attested
  continuity
- restored/readmitted identity silently treated as true continuity when only
  reconstructed equivalence exists
- partial, elided, or redacted lineage expressed as complete continuity instead
  of named-gap, withheld, or denied posture
- transient-within-boundary participants silently treated as durable lineage
  continuity
- provenance that is only meaningful if you already know the producer runtime
- support artifacts built from stale or reduced basis without explicit freshness
  or retention disclosure
- support-grade truth pretending to be stronger authority truth
- replay/restoration/build-from-checkpoint hidden behind cheap-looking getters
- "best match" or "maybe successor" bags that collapse ambiguity and denial
- locator-level continuity silently folded into object-level continuity
- one required persistence or executor model
- one giant helper file that owns locality, lineage, provenance, receipts,
  support, and readiness together

## Phases

These phases are implementation order, not a buffet. Each phase must leave a
production responsibility home, a minimum public surface, and a hostile proof
bar that the next phase is allowed to consume.

An engineer implementing this milestone should follow this rhythm:

1. finish the named production surface for the current phase
2. finish the compile-fail and hostile runtime proof for the current phase
3. stop and verify that the next phase can consume that surface without
   reopening lower-level law
4. only then move on

If a later phase reveals a flaw in an earlier phase, fix the earlier phase as
alignment work. Do not treat that as permission to skip the gate or start
building later-phase bundles on unstable lower law.

### Phase 1: Category, Locality, And Role Primitives

Freeze the primitive nouns first. This phase exists to remove every excuse to
smuggle lineage, provenance, receipt, support-truth, locality, or
planned-versus-executed meaning through generic labels later.

Practical implementation order:

1. Define typed category primitives for:
   - lineage
   - provenance
   - receipt
   - support-truth
2. Define typed locality primitives for:
   - current
   - branch-local
   - historical
   - comparison-paired
   - snapshot-bound
   - replay-derived
   - restored/readmitted
3. Define typed role/severity-like primitives for:
   - planned versus executed
   - support-grade versus stronger-authority-adjacent descriptive role
   - freshness/retention posture at the primitive floor, not as later bundle
     decoration
4. Define the minimum legality law connecting category, locality, and role
   posture.
5. Add compile-fail boundaries preventing primitive substitution.
6. Add canonical ordering and equality law for primitive labels and sets.

Do not begin provenance layering, receipt families, or lineage outcomes until
these primitives are mechanically frozen.

Phase 1 is complete only when:

- category substitution is fail-closed
- locality posture is typed and canonical
- planned-versus-executed is a first-class primitive seam
- freshness/retention posture is available without inventing provenance bags
- blind consumers can distinguish the primitive families without producer
  folklore

### Phase 2: Provenance Layering And Freshness Law

Build provenance next, on top of Phase 1 primitives. The goal of this phase is
to make â€œunder what basis, path, and retained context was this produced?â€
answerable without yet solving continuity or executed-boundary truth.

Practical implementation order:

1. Define typed provenance layers for:
   - source basis
   - authority path
   - strategy basis
   - profile basis
   - comparison basis
   - canonical/digest basis
   - support-context attachments
2. Attach retention/freshness posture to provenance in a way that cannot be
   omitted accidentally.
3. Define the minimum provenance construction path for current, historical,
   replay-derived, and restored/readmitted contexts.
4. Add compile-fail boundaries so provenance cannot satisfy lineage or receipt
   APIs.
5. Add parity tests proving the same provenance meaning canonicalizes
   identically across independent producers.

Do not begin receipt law or continuity outcomes until provenance can already
name context, basis, and freshness honestly.

Phase 2 is complete only when:

- provenance explains basis and context without impersonating stronger proof
- stale or reduced-basis posture is structurally visible
- replay-derived and restored/readmitted provenance are distinct from ordinary
  current provenance
- blind consumers can inspect provenance without producer-private state

### Phase 3: Receipt Families And Closeout Truth

Only after provenance is stable should executed-boundary and closeout truth be
frozen. This phase answers â€œwhat actually happened?â€ and separates that from
planning, denial, or support followthrough.

Practical implementation order:

1. Define typed receipt families for:
   - admission
   - planning
   - execution
   - publication
   - restoration
   - support publication
   - checkpoint/resume
   - denial/blocked closeout
2. Define the receipt construction path so executed receipts must carry enough
   completed-boundary evidence and cannot be hand-assembled from generic bags.
3. Define typed no-op, blocked, and denied closeout receipts that remain
   distinct from execution receipts.
4. Add room for transient participants that opened and closed within one
   executed boundary without promoting them into continuity yet.
5. Add compile-fail boundaries preventing planning receipts from satisfying
   executed receipt APIs.

Do not begin continuity families until completed-boundary truth is already
mechanically frozen.

Phase 3 is complete only when:

- planning and execution receipts are non-substitutable
- blocked/denied/no-op closeout remains typed instead of boolean
- transient within-boundary lifecycle can be preserved as executed evidence
- blind consumers can inspect completed-boundary truth without inferring from
  diagnostics or support alone

### Phase 4: Lineage, Continuity, And Divergence Outcomes

Only after completed-boundary truth exists should the milestone define
continuity claims. This phase answers â€œwhat survived, split, merged,
reconstructed, diverged, or failed to continue?â€

Practical implementation order:

1. Define typed lineage outcome families for:
   - singular continuity
   - plural successor/predecessor
   - merge successor
   - branch-local replacement
   - restored continuity
   - reconstructed equivalence
   - named-gap partial continuity
   - withheld/redacted continuity
   - transient-within-boundary closure
   - advisory correspondence candidate
   - ambiguity
   - identity break
   - denial
2. Define branch-divergence and promotion posture as distinct attachments rather
   than generic flags.
3. Define replay-derived continuity law so it cannot silently upgrade into
   retained/directly attested continuity.
4. Define restored/readmitted law so direct restored continuity cannot be
   confused with reconstructed equivalence.
5. Define partial/redacted/elided lineage law so named-gap, withheld, and
   denied posture are explicit.
6. Add compile-fail boundaries across the strongest continuity substitutions.

Do not begin support-truth bundles until continuity meaning is already stable.

Phase 4 is complete only when:

- branch-local divergence and promotion posture are explicit
- replay-derived continuity remains visibly weaker than retained continuity
- reconstructed equivalence cannot impersonate restored continuity
- partial, withheld, and denied continuity are structurally distinct
- transient participants do not become durable lineage nodes

### Phase 5: Support-Truth, Recovery, And Degraded Operation

Only after continuity and receipts are frozen should support-grade truth be
built. This phase answers â€œwhat support evidence exists, how strong is it, how
fresh is it, and what degraded or recovery posture applies?â€

Practical implementation order:

1. Define typed support-truth families for:
   - evidence bundles
   - certification summaries
   - parity artifacts
   - degraded-recovery reports
   - stale-basis disclosures
   - transient lifecycle evidence
   - residual-debt statements
2. Define checkpoint/resume, reconstruction denial, rebuild, and quarantine
   posture explicitly.
3. Define support publication and support closeout meaning on top of receipt and
   provenance law rather than duplicating them.
4. Define the support construction path so stale or reduced-basis support
   artifacts cannot be constructed without explicit disclosure.
5. Add compile-fail boundaries preventing support-grade artifacts from
   satisfying stronger authority or proof APIs.

Do not begin canonical attachment bundles until support truth and degraded
recovery honesty are already closed.

Phase 5 is complete only when:

- support-grade truth is authoritative for its declared support role only
- stale or reduced-basis support remains visibly limited
- recovery, rebuild, quarantine, and residual debt are typed
- support publication and support closeout do not impersonate stronger
  authority boundaries

### Phase 6: Attachment, Canonical Participation, And Materialization

Only after all four semantic families are stable should the milestone define
how they travel together. This phase answers â€œhow do these artifacts attach to
real boundary surfaces, participate in canonical/digest identity, and materialize
under profiles?â€

Practical implementation order:

1. Define attachment law for:
   - boundary artifacts
   - transition artifacts
   - diagnostics bundles
   - canonical basis and digest participation
2. Define object-level versus locator-level continuity attachments as separate
   legal shapes.
3. Define profile-driven materialization/elision for optional descriptive
   richness.
4. Define current-basis and support readmission boundaries for stronger lanes.
5. Add compile-fail boundaries preventing wrong-family attachments and
   reentry/readmission bypass.
6. Add cross-surface scenario tests proving one canonical boundary artifact can
   carry provenance, receipt, lineage, support, and diagnostics without
   category collapse.

Do not begin readiness closeout until the public artifact shapes and grouped
lanes are frozen.

Phase 6 is complete only when:

- the same target artifact can legally carry the right family mix without
  collapse
- object-level and locator-level continuity are both supported and distinct
- reduced richness can remove optional surfaces without changing authority
  truth
- stronger readmission lanes remain visibly stronger

### Phase 7: Production-Test Readiness

Only after the real production surface exists should the milestone freeze the
machine-checkable readiness contract. This phase is not implementation fluff;
it is the proof that the surface is stable enough for adoption.

Practical implementation order:

1. Freeze the exact public surface inventory.
2. Freeze compile-fail boundary ownership and scenario-family ownership.
3. Freeze canonical golden artifacts, property seeds, and harness expansion
   points.
4. Freeze runtime adoption failure pressures, assumptions, non-assumptions, and
   residual debt.
5. Certify the readiness artifact through the stronger proof lane.

Phase 7 is complete only when:

- the public surface is machine-checkably inventoried
- compile-fail, hostile, property, and scenario obligations are named exactly
- runtime adoption pressures are explicit rather than implied
- residual debt is explicit and bounded

### Phase 8: Feature Docs And Crate-Doc Integration

Only after readiness is frozen should the crate-facing docs be written. This
phase must describe the shipped surface, not speculate about the planned one.

Practical implementation order:

1. Write one milestone landing page plus one capability doc per real seam.
2. Document the grouped common path, lower lane, and stronger lane with real
   examples.
3. Document edge cases that are easy to lose to history:
   - replay-derived continuity
   - reconstructed equivalence
   - blocked closeout
   - transient lifecycle
   - locator-level continuity
   - stale support basis
4. Run a hostile doc omissions pass so no real surface is recoverable only from
   code.

Phase 8 is complete only when:

- the docs match the implemented DX and proof boundaries
- the public surface is history-safe
- later adopters do not need code archaeology to discover the meaning surface

### Phase 9: `feature-doc-writer` Closeout And Crate-Docs Registration

Only after the feature seams are fully implemented and verified should the
milestone run the final documentation closeout. This phase exists so the docs
are not treated as optional polish or one large catch-all narrative page.

This phase must explicitly use the `feature-doc-writer` skill and must produce
crate-facing docs that are organized by category folder, with one document per
real feature seam.

Practical implementation order:

1. Create one milestone documentation folder under
   `crates/worth-foundational/docs/` named for the shipped category.
2. Add one landing-page `README.md` for the milestone category.
3. Add one feature document per real capability seam rather than one giant
   milestone dump or mixed concept/guide/reference soup.
4. Ensure the docs cover:
   - the common path
   - the lower lane
   - the stronger lane
   - the important edge cases that would otherwise be lost to history
5. Add the new folder to crate-facing documentation entrypoints so the docs are
   actually discoverable from the crate docs rather than orphaned on disk.
6. Run a hostile omissions pass to make sure no implemented surface is
   preserved only in code or tests.

The intended documentation topology should look like:

```text
crates/worth-foundational/docs/
  <milestone-7-category>/
    README.md
    <feature-1>.md
    <feature-2>.md
    <feature-3>.md
    ...
```

Phase 9 is complete only when:

- the `feature-doc-writer` pass has been run against the shipped Milestone 7
  surface
- the docs live in a category folder rather than a flat pile
- each real feature seam has exactly one primary documentation home
- the docs are linked into the crate-facing documentation surface
- the docs have passed a hostile omissions review so future adopters do not
  need code archaeology to recover milestone meaning

## Compile-Time Boundary Targets

Milestone 7 should convert the following assumptions into compile-fail
boundaries wherever visibility, sealed constructors, phantom types, or
proof-bearing wrappers can enforce them:

- a provenance artifact cannot satisfy an API requiring a lineage claim
- a lineage artifact cannot satisfy an API requiring provenance or receipt
- a planning receipt cannot satisfy an executed-boundary receipt API
- a support-truth artifact cannot satisfy stronger authority or stronger proof
  APIs
- a replay-derived continuity artifact cannot satisfy APIs requiring retained
  or directly attested continuity
- a reconstructed-equivalence artifact cannot satisfy APIs requiring restored
  direct continuity
- a branch-local continuity artifact cannot satisfy globally admitted
  continuity APIs without explicit promotion/readmission
- a locator-level continuity attachment cannot satisfy object-level continuity
  APIs
- a generic replay slice or event-history record cannot satisfy typed
  lineage/provenance/receipt APIs
- a stale or reduced-basis support artifact cannot be constructed without an
  explicit freshness/retention disclosure posture
- current-basis or support reentry across trust boundaries cannot bypass the
  proof-bearing readmission lane

## What Must Ship

- shared lineage vocabulary with family-distinct continuity outcomes
- shared provenance vocabulary with explicit basis locality and authority path
- shared receipt vocabulary for completed effectful or authority-bearing
  boundaries
- shared support-truth and certification-summary vocabulary
- explicit planned-versus-executed boundary seams
- explicit replay-derived, snapshot-bound, historical, branch-local, and
  restored/readmitted locality posture
- explicit replay-versus-history relationship rules so lineage artifacts do not
  masquerade as replay slices or generic event histories
- explicit branch-divergence and promotion posture vocabulary
- attachment and materialization law for boundary artifacts, transitions,
  diagnostics, and current-basis surfaces
- proof-composable strengthening and readmission boundaries
- production-test readiness and feature docs

## Semantic Guarantees

- lineage, provenance, receipt, and support-truth remain distinct categories
  with stable meanings
- provenance explains context and basis; it does not replace proof law
- receipts attest completed effectful or authority-bearing boundaries, not
  merely attempted or planned work
- branch-local, historical, replay-derived, and restored/readmitted postures
  remain explicit
- replay-derived continuity never silently upgrades into retained or directly
  attested continuity
- restored/readmitted identity distinguishes direct continuity from
  reconstructed equivalence
- partial, elided, or redacted lineage resolves into explicit named-gap,
  withheld, or denied posture instead of looking complete
- replay/history relationship remains explicit: replay slices, event histories,
  and lineage continuity artifacts are related but not interchangeable
- support and certification artifacts remain derived proof-of-truth surfaces
  rather than being confused for stronger authority state
- stale or reduced-basis support artifacts disclose freshness/retention limits
  explicitly
- profile-driven richness reduction may elide optional descriptive surfaces but
  must not change authoritative truth
- transient-within-boundary participants may appear in executed-boundary or
  support-truth evidence without being promoted into durable lineage continuity

## Representation Boundaries

- crates retain their own storage, replay, persistence, journal, and executor
  topology until canonical materialization boundaries
- foundational lineage/provenance/receipt vocabulary does not imply one event
  record layout or one support bundle layout
- foundational support-truth vocabulary does not imply one QA harness or one
  persistence model
- proof-bearing artifacts may attach these descriptive surfaces without moving
  their descriptive ownership into `worth-proof`

## Must Preserve

- no generic history envelope that makes continuity, derivation, and attested
  execution indistinguishable
- no implied equivalence between support-grade truth and stronger authority
  truth
- no replay or restoration API that hides breadth or cost
- no replay slice or generic history record may silently stand in for a lineage
  continuity artifact
- no transient participant may be silently upgraded into a durable successor,
  predecessor, or surviving lineage node
- no reconstructed equivalence may silently upgrade into direct restored
  continuity
- no stale or reduced-basis support artifact may look fresh, complete, or
  stronger than it is
- no locator-level continuity may be silently rewritten as object-level
  continuity
- no branch-local or advisory result that silently upgrades into authoritative
  continuity
- no producer-private interpretation requirement for blind consumers

## Desired DX End State

An engineer should be able to predict the right surface before opening the
implementation:

- use lineage vocabulary to answer continuity questions
- use provenance vocabulary to answer basis/context/path questions
- use receipt vocabulary to answer completed-boundary questions
- use support-truth vocabulary to answer support, parity, recovery, and debt
  questions
- use profile/materialization lanes to control richness and delivery posture
- use proof-bearing strengthening only when stronger claims are real

A blind consumer should also be able to ask practical questions and predict the
surface family that answers them:

- "Did this identity survive?" -> lineage
- "Under what basis and retention posture was this conclusion produced?" ->
  provenance
- "What actually executed versus what was only planned or denied?" -> receipt
- "How strong is this support claim, and what debt or staleness does it
  carry?" -> support-truth
- "Is this about the object itself or only a pointer/path/field relation?" ->
  locator-level versus object-level continuity

The public surface should teach at least three distinct lanes:

- a common descriptive path for producing and inspecting typed lineage,
  provenance, receipt, and support artifacts
- a lower lane for direct canonical attachment, comparison, and digest
  participation work
- a stronger lane for proof-bearing certification, current-basis readmission,
  and readiness artifacts

The finished code should also look intentional at the call site. The exact
final names may shift, but the shape should look roughly like this:

```rust
use worth_foundational::boundary_evidence_api::{
    common_path as evidence,
    lower_lane,
    stronger_lane,
};
```

The common descriptive path should read like intent, not like raw record
assembly:

```rust
let provenance = evidence::provenance()
    .current(current_basis)
    .authority_path(authority_path)
    .profile_basis(profile_basis)
    .canonical_digest_basis(digest_basis)
    .retention(retention_posture)
    .finish()?;

let receipt = evidence::receipt()
    .execution()
    .for_boundary(committed_transition)
    .with_provenance(&provenance)
    .publish()?;

receipt.kind();
receipt.completed_boundary();
receipt.provenance();
receipt.locality();
receipt.explain();
```

Branch-local promotion should be explicit at the call site instead of ambient:

```rust
let promoted = evidence::lineage()
    .branch_local(candidate_identity)
    .with_branch_divergence(divergence_posture)
    .promoted_by(&receipt)
    .into_globally_admitted_continuity()?;

promoted.lineage_kind();
promoted.pre_promotion_posture();
promoted.promotion_posture();
promoted.executed_receipt();
promoted.provenance();
```

Replay-derived support should remain visibly weaker and cost-bearing:

```rust
let support = evidence::support()
    .replay_derived()
    .from_snapshot(snapshot_basis)
    .from_replay_slice(replay_basis)
    .with_freshness(stale_support_posture)
    .reconstruct_likely_continuity(subject_identity)?
    .materialize()?;

support.support_grade();
support.freshness_posture();
support.recovery_posture();
support.named_gaps();
support.parity_artifacts();
support.residual_debt();
```

Blocked or denied boundaries should still materialize the right receipt family
without impersonating execution:

```rust
let blocked = evidence::receipt()
    .blocked_closeout()
    .for_plan(planned_boundary)
    .with_provenance(&provenance)
    .with_diagnostics(blocked_diagnostics)
    .publish()?;

blocked.kind();
blocked.completed_boundary(); // closeout happened
blocked.did_execute();        // false
blocked.denial_cause();
blocked.support_rows();
```

Transient participants should stay visible without becoming durable lineage:

```rust
let transient = evidence::support()
    .transient_lifecycle()
    .opened_and_closed_within(&receipt)
    .for_subject(transient_identity)
    .record()?;

transient.lifecycle_kind();
transient.opening_boundary();
transient.closing_boundary();
transient.promotes_to_durable_lineage(); // false
```

Locator-level continuity should be first-class instead of being smuggled into
object-level continuity:

```rust
let relocation = evidence::lineage()
    .locator_level()
    .from_locator(old_pointer_locator)
    .to_locator(new_pointer_locator)
    .under_object(stable_object_identity)
    .attested_by(&receipt)
    .materialize()?;

relocation.continuity_scope(); // locator-level
relocation.object_identity();
relocation.from_locator();
relocation.to_locator();
```

The lower lane should expose exact artifact-level construction and canonical
attachment without pretending to be the common path:

```rust
let canonical = lower_lane::attachment::bundle()
    .attach_lineage(promoted)
    .attach_provenance(provenance)
    .attach_receipt(receipt)
    .attach_support(support)
    .under_digest_basis(digest_basis)
    .materialize()?;

canonical.member_categories();
canonical.digest_participation();
canonical.canonical_basis_entries();
canonical.explain();
```

The stronger lane should make proof-bearing strengthening visibly stronger:

```rust
let certified = stronger_lane::readiness()
    .certify_support_bundle(canonical, proof_artifact)?
    .readmit_for_current_basis(current_basis_proof)?;

certified.proofs();
certified.readmission_basis();
certified.certified_surfaces();
certified.runtime_adoption_pressures();
```

The goal is that callers can stay on the common path for normal descriptive
work, drop to the lower lane when canonical attachment and basis participation
matter, and enter the stronger lane only when proof-bearing certification or
readmission is actually required.

## Acceptance Evidence

- hostile category-separation tests proving lineage, provenance, receipt, and
  support-truth cannot substitute for each other
- materialization tests across branch-local, historical, snapshot-bound,
  replay-derived, and restored/readmitted locality
- compile-fail tests proving planned artifacts cannot satisfy executed-receipt
  APIs and support-grade artifacts cannot satisfy stronger authority APIs
- replay-versus-history tests proving lineage continuity does not collapse into
  replay slices or generic event histories
- restored-versus-reconstructed tests proving readmitted identity can
  distinguish direct continuity from reconstructed equivalence
- partial-lineage tests proving elided, redacted, or missing lineage resolves
  into named-gap, withheld, or denied posture
- stale-support tests proving support artifacts disclose reduced or stale basis
  explicitly
- branch-divergence and promotion-posture tests proving branch-local truth does
  not silently upgrade into globally admitted continuity
- transient lifecycle tests proving create-then-delete or introduce-then-retire
  activity can be preserved as executed/support evidence without appearing as
  durable lineage continuity
- locator-versus-object continuity tests proving pointer/path/field continuity
  can be expressed without pretending the whole object changed identity
- blind-consumer tests proving artifacts remain interpretable without
  producer-private state
- scenario tests covering every practical boundary scenario listed above rather
  than only isolated unit-shape proofs
- reduced-richness profile tests proving optional descriptive surfaces can be
  removed without changing authoritative truth
- canonical/digest attachment tests proving lineage/provenance/receipt/support
  participation is stable and basis-honest
- readiness certification and grouped public-surface inventory proof

## Architectural Notes

- Diagnostics and provenance must compose, but provenance is not just
  diagnostics renamed.
- Receipts and support-truth must compose, but support-truth is not merely
  "receipt plus notes."
- Lineage continuity should be able to attach to current-basis transitions,
  boundary artifacts, or diagnostics bundles without forcing one storage model.
- If any adopting crate needs one giant artifact envelope to make the milestone
  usable, the milestone has probably been underspecified and should be expanded
  instead of papered over.

## Sequencing Notes

- Milestone 7 should be implemented before Milestone 8 because layout and
  performance vocabulary may need to attach to support-truth, replay, and
  degraded-recovery surfaces.
- Milestone 7 should land before final migration/closure work because adopting
  crates need this shared language to stop carrying local lineage/provenance
  folklore.

## Explicit Non-Goals

- one shared event store
- one replay executor
- one persistence schema
- one diagnostics runtime
- one support or QA harness
- one merge or identity-resolution engine

## Self-Check

- Does this milestone standardize shared meaning rather than shared storage?
- Are lineage, provenance, receipt, and support-truth still mechanically
  distinct?
- Can a blind consumer interpret the artifact without producer-private state?
- Are replay, restore, checkpoint, and degraded-recovery postures explicit?
- Do receipts attest completed boundaries rather than intent?
- Can richer descriptive surfaces be elided centrally without changing
  authoritative outcomes?
- Would `worth-query`, `worth-store`, `worth-relational`,
  `worth-runtime-bridge`, and `worth-signal` all be able to map their local
  Milestone 7 concepts onto this vocabulary without being forced into one
  executor or persistence model?
