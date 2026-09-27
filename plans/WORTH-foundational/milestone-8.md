# Milestone 8: Performance, Layout, And Enforcement Vocabulary

## Goal

Define one shared language for layout intent, access-pattern posture,
performance-claim boundaries, breadth and allocation semantics, enforcement
layers, and cost-facing evidence so WORTH crates can describe why a structure
exists, where a claim is valid, how strong that claim is, and what work it
includes without forcing one hot-path representation, one measurement runtime,
or one benchmark harness across the workspace.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and solving the hard
infrastructure problem before feature polish. The shaping constraint is that
Milestone 8 must solve honest performance meaning and enforcement boundaries
before crates add more local performance folklore or more cost-shaped
artifacts.

### `arch_laws.md`

Protects authority-versus-derivation separation, explicit proof-bearing
strengthening, and fail-closed boundary law. The shaping constraint is that
performance vocabulary must describe authority cost, derived/support cost,
materialization cost, replay cost, and layout intent without collapsing those
surfaces into one generic report or one fake shared runtime.

### `composition_laws.md`

Protects narrow responsibility homes, named steps, and anti-bag discipline.
The shaping constraint is that layout families, claim-boundary families,
allocation postures, breadth law, enforcement layers, attachment law, and
readiness artifacts must live in separate homes rather than one broad
`performance.rs` dumping ground.

### `domain_structure_laws.md`

Protects domain topology as the structure of the code. The shaping constraint
is that this milestone must keep layout intent, access posture, performance
claims, enforcement layers, structural counters, comparison bundles, and
readiness evidence independently locatable and independently testable.

### `perf_laws.md`

Protects cost-honest surfaces, explicit locality, breadth visibility, and
mechanical observability. The shaping constraint is that Milestone 8 must make
performance claims name their real boundary, included work, excluded work,
breadth posture, and fallback/debt posture explicitly rather than relying on
elapsed time folklore.

### `dx_laws.md`

Protects organized-truth DX, explicit responsibility boundaries, visible
expensive work, object-spec-versus-builder discipline, and inspectable lowered
plans before execution. The shaping constraint is that Milestone 8 must state
in detail what the finished API, module topology, progression surfaces, and
accountability surfaces should look like so the implementation cannot drift
into ad hoc helper bags or cute but dishonest ergonomics.

### `worth_foundational_vision.md`

Protects the thesis that foundational standardizes shared meaning while
preserving crate-local representation freedom. The shaping constraint is that
Milestone 8 must standardize performance and layout vocabulary strongly enough
for cross-crate interpretation without standardizing AoS, SoA, AoSoA, sparse,
packed, or custom storage topology away.

### `worth_foundational_roadmap.md`

Protects milestone sequencing and the non-goal of a universal runtime
container. The shaping constraint is that Milestone 8 must follow Milestone 7
because performance claims need locality, replay, restoration, support, and
receipt vocabulary to attach to, and it must land before migration so crates
stop inventing incompatible performance-report dialects.

### `test-requirements.md`

Protects the shared proof bar before adopting-crate migration. The shaping
constraint is that Milestone 8 must locally prove layout-vocabulary
distinction, boundary-named performance claims, structural-counter
attachments, explicit expensive materialization seams, and representation
freedom under hostile synthetic fixtures.

### `milestone-7.md`

Protects anti-flattening descriptive law. The shaping constraint is that
Milestone 8 must preserve the same nuance for performance that Milestone 7
preserved for lineage/provenance/receipts: replay-derived, restored,
readmitted, branch-local, support-grade, transient-within-boundary, and
planned-versus-executed surfaces must all be able to carry performance meaning
without category collapse.

### `milestone-6.md`

Protects diagnostics and explanation ontology. The shaping constraint is that
performance claims must compose with diagnostic and support surfaces rather
than replacing them, and reduced-richness profiles must be able to elide
optional performance-facing descriptive surfaces without changing
authoritative outcomes.

### `milestone-6-closeout.md`

Protects machine-checkable readiness and category-complete documentation
closure. The shaping constraint is that Milestone 8 should ship with the same
kind of public-surface inventory, compile-fail boundary ownership, and
crate-doc discipline rather than leaving the performance vocabulary as an
internal-only convention.

## Existing Runtime Patterns

### `worth-query`

What to keep:

- performance is already carried as typed local meaning rather than one
  elapsed-time blob
- width, density, allocation, fallback, and complexity status already appear as
  separate seams
- live delivery and view-shape lanes already distinguish verified cost posture
  from debt posture explicitly

What to prune:

- local query-specific names such as subscription width, grouped delta, and
  view-shape cost class should not become the shared canonical names wholesale
- foundational should not adopt query's live runtime or patch engine

### `worth-store`

What to keep:

- support paths already distinguish foreground, maintenance, portability, and
  operator-facing cost surfaces
- density class, allocation scope, breadth budgets, and verified-versus-debt
  complexity surfaces are first-class
- store-global debt is explicit rather than silently admitted

What to prune:

- store-specific retention/compaction/rebuild lifecycle topology should not
  become foundational representation
- foundational should not imply one maintenance planner or one counter store

### `worth-relational`

What to keep:

- phase timing and complexity deltas are already receipt-grade execution
  evidence rather than debug-only telemetry
- authoritative mutation, publication, validation, and history cost are
  already family-distinct
- query and mutation scale work already names locality and fallback honestly

What to prune:

- relational-specific packet, reducer, and authority pipeline topology should
  not become foundational storage or execution law
- foundational should standardize the vocabulary of the claim, not the query
  planner

### `worth-runtime-bridge`

What to keep:

- provenance-rich request and receipt surfaces already carry causality,
  idempotence, loop-prevention, and compatibility context
- historical lineage topology is already distinct from generic replay slices
- bridge artifacts already behave like blind-consumer-readable evidence rather
  than producer-private runtime records

What to prune:

- bridge-local adapter/session/preview naming should not become canonical
  workspace-wide performance naming
- foundational should not imply one bridge adapter or one writeback runtime

### `worth-signal`

What to keep:

- compile-time contract, runtime policy, and counter-test enforcement are
  already separate layers
- path class, maintenance mode, authority policy, artifact policy, and
  execution strategy are already distinct seams
- lifecycle, freshness, and replay/reconstruction pressure already participate
  in support and capability surfaces

What to prune:

- signal-specific comparator and async-node topology should not become the
  shared foundational representation
- foundational should not adopt one runtime telemetry layout or one scheduler

## Why This Milestone Exists

Milestones 1 through 7 made it possible to say:

- what canonical values and locators mean
- how digests and canonical bases are formed
- what profiles govern richness and support posture
- what artifact family a boundary surface belongs to
- what branch/merge/commit transition occurred
- what diagnostics and explanation topology exists
- what lineage, provenance, receipt, support-truth, and transient boundary
  evidence mean

WORTH still lacks one shared language for the next layer of questions:

- what layout strategy a crate is relying on
- what access-pattern or maintenance posture that strategy serves
- what exact boundary a performance claim is naming
- whether a claim is compile-time, runtime-admitted, counter-backed, or support
  derived
- what breadth, locality, allocation, freshness, or fallback posture applied
- whether a cost claim is verified, explicit debt, deferred, widened, or
  rejected

Without Milestone 8, crates can continue to build serious performance
surfaces, but they will describe them in incompatible local dialects:

- one crate will call something a density class while another calls it an
  execution lane
- one crate will attach width and allocation posture to a receipt while another
  only publishes phase timing
- one crate will express replay-derived cost through support artifacts while
  another hides it inside diagnostics or helper naming
- one crate will distinguish compile-time and counter-test enforcement while
  another will only have narrative claims

That is exactly the kind of drift `worth-foundational` exists to stop.

## Adversarial Constraint

Several WORTH crates with different hot-path representations, replay and
retention models, authority boundaries, and support surfaces must be able to
attach layout intent, cost posture, breadth/allocation semantics, fallback and
debt posture, and mechanically testable performance evidence to the same
canonical boundary-facing artifacts across current, branch-local, historical,
snapshot-bound, replay-derived, restored/readmitted, planned, executed, and
transient-within-boundary contexts without:

- standardizing one in-memory representation
- collapsing authority cost, derived/support cost, replay cost, and
  materialization cost into one generic number
- hiding expensive replay, support, or materialization work behind cheap-
  looking getters
- claiming cost equivalence between AoS, SoA, AoSoA, sparse, packed, and
  custom layouts
- requiring consumers to understand producer-private telemetry or runtime
  internals to interpret the claim

## Dependencies On Earlier Milestones

Milestone 8 depends on earlier milestones mechanically, not just conceptually.

- Milestone 1 supplies aspect-native values, locators, and authoritative
  state/mask vocabulary so performance claims can attach to stable shared
  surfaces rather than stringly subjects.
- Milestone 2 supplies canonical basis, mismatch basis, digest participation,
  and blind-consumer parity so performance claims can be compared and certified
  without incidental layout drift.
- Milestone 3 supplies profile meaning and central reduced-richness posture so
  performance-facing descriptive surfaces can be elided centrally without
  changing authoritative outcomes.
- Milestone 4 supplies boundary artifact categories and materialization law so
  performance claims can name whether they apply to summaries, reports,
  artifacts, receipts, and support lanes.
- Milestone 5 supplies branch/merge/commit and planned-versus-committed
  transition law so authority cost can remain distinct from branch-local or
  planned cost.
- Milestone 6 supplies diagnostics/explanation ontology so performance claims
  can compose with support and explanatory surfaces rather than smuggling
  themselves in as ad hoc report text.
- Milestone 7 supplies locality, freshness, replay-derived, restored,
  transient, support-truth, and planned-versus-executed nuance that this
  milestone must reuse directly instead of flattening into generic
  observability.

## WORTH-Proof Dependency Boundary

Milestone 8 remains in `worth-foundational`, not `worth-proof`.

What belongs here:

- descriptive layout vocabulary
- descriptive cost-boundary vocabulary
- descriptive breadth, allocation, locality, freshness, fallback, and debt
  vocabulary for performance claims
- canonical performance claim bundles and comparison law
- attachment and materialization law for performance-facing evidence
- readiness and documentation closure for those surfaces

What may strengthen through `worth-proof`:

- proof-bearing certification that a performance claim is attached to a
  stronger authority or certified support artifact
- readmission or trust-boundary bridging for stronger performance evidence
- proof-bearing readiness artifacts over the Milestone 8 public surface

What must not move into `worth-proof`:

- ownership of the shared descriptive vocabulary itself
- ownership of runtime telemetry capture, scheduler policy, query planning, or
  storage layout
- any one crate's measurement engine or hot-path data structure

Milestone 8 uses `worth-proof` only when a descriptive performance claim is
upgraded into a stronger certified claim. Plain vocabulary, plain receipts,
plain runtime-policy claims, and plain counter-backed bundles stay local to
`worth-foundational`.

The standardized distinction for this milestone is:

- descriptive:
  - names layout intent, claim boundary, evidence strength, breadth,
    allocation, execution temperature, freshness, and fallback posture
- mechanically observable:
  - carries runtime-policy results, structural counters, and canonical
    comparison bundles without claiming stronger certification
- certified:
  - proves stronger attachment, coverage, or readiness claims through the
    shared `worth-proof` lane
- readmitted:
  - carries stronger certified performance evidence across a trust or
    current-basis boundary through the existing proof bridge

Additional boundary law:

- evidence strength is part of descriptive performance meaning
- certified and readmitted are stronger proof-bearing transport and trust
  categories layered over that meaning
- no semantic category may exist only in the proof lane without also having a
  descriptive meaning in foundational vocabulary first
- `worth-proof` may strengthen a claim; it must not become a second owner of
  performance ontology

## WORTH-Proof Standardized Lane

Milestone 8 must follow the same pattern Milestone 6 closed with: stronger
performance claims reuse `worth-proof`; descriptive performance meaning does
not migrate into a second proof substrate.

Proof-bearing surfaces standardized here:

- certified hot-path performance bundles that claim stronger operational
  posture than plain counter-backed receipts
- certified support-derived or replay-derived bundles that claim stronger
  attachment compatibility than descriptive support rows alone
- trust-boundary bridge and readmission for stronger performance evidence
- production-test readiness certification for the Milestone 8 public surface

Concrete `worth-proof` usage this milestone should freeze:

- authority-witness-based attachment for stronger performance bundles
- proof-bearing bundle carriers that wrap certified performance claims without
  changing their descriptive vocabulary
- trust-boundary bridge/readmission APIs for stronger current-basis reuse of
  certified bundles
- readiness certification APIs that freeze exact certified surfaces, hostile
  pressures, compile-fail boundaries, and adoption assumptions

Deliberately not moved into the proof kernel:

- layout intent families and access-pattern posture families
- hot-path, warm-path, cold-path, recovery-only, and support-only vocabulary
- runtime counters, budget receipts, and comparison rows
- debt, widening, rejection, replay-derived, stale-support, and
  materialization posture vocabulary

## Practical Type Targets

Milestone 8 should be designed around the following shared families.

- layout-intent families:
  - `AoS`
  - `SoA`
  - `AoSoA`
  - `Sparse`
  - `Packed`
  - `Custom`
- performance-boundary families:
  - authoritative execution
  - boundary materialization
  - replay reconstruction
  - support assembly
  - maintenance planning
  - maintenance execution
  - publication
  - delivery
  - retention/compaction
  - restore/recovery
- evidence-strength families:
  - compile-time contract
  - runtime policy admission
  - counter-backed execution receipt
  - support-derived performance claim
  - explicit debt/deferred claim
- breadth/locality posture families:
  - point-local
  - family-local batch
  - basis-local batch
  - branch-local
  - snapshot-bound
  - delta-bound
  - cross-partition/cross-region
  - portability-scope
  - operator/global debt
- allocation posture families:
  - no allocation
  - action-local
  - lifecycle-arena or arena-local
  - batch-local
  - manifest/report scoped
  - rebuild scoped
- access-pattern posture families:
  - scan-heavy
  - point-lookup
  - traversal-local
  - append-heavy
  - rebuild-capable
  - density-adaptive
- execution-temperature posture families:
  - hot-path
  - warm-path
  - cold-path
  - recovery-only
  - support-only
- freshness/retention posture families:
  - exact-basis current
  - historical retained
  - replay-derived
  - restored/readmitted
  - stale support
  - reduced retention
- fallback/debt posture families:
  - verified
  - deferred
  - debt
  - rejected
  - widened with explicit disclosure
  - fresh-freeze/rebuild/readmission required
- performance-claim attachments:
  - structural counters
  - complexity-contract names
  - included-work classes
  - excluded-work classes
  - canonical basis or digest participation
  - supporting diagnostic/support rows
- work-class families:
  - authoritative mutation work
  - validation/planning work
  - publication/delivery work
  - replay/reconstruction work
  - support/report assembly work
  - forensic/parity work

## Canonical Noun Inventory

Milestone 8 should freeze one small, reusable set of first-class nouns near the
center of the public surface. These are the primary shared toolbelt shapes the
later crate rewrites should be able to depend on directly instead of
re-inventing local performance dialects.

Minimum canonical nouns:

- `LayoutIntentClaim`
- `PerformanceClaim`
- `PolicyAdmissionReceipt`
- `CounterBackedPerformanceReceipt`
- `PerformanceBundle`
- `PerformanceComparison`
- `PerformanceReportPlan`
- `MaterializedPerformanceReport`
- `CertifiedPerformanceBundle`
- `ReadmittedPerformanceBundle`

These nouns are not the entire milestone surface. They are the smallest set of
named artifacts that should let an adopter express:

- what structural family a path was designed around
- what exact boundary a claim is naming
- what pre-execution policy or fallback posture was admitted
- what structurally counted work actually happened
- how independently produced claims compare
- what expensive report materialization will assemble before it happens
- what stronger proof-bearing or readmitted claim exists beyond plain
  descriptive or counter-backed evidence

If later implementation work starts inventing several parallel names for one of
those jobs, the burden is on the implementation to prove a genuinely new
responsibility exists rather than allowing terminology drift.

## What Makes Performance Easy

Milestone 8 should make honest performance the path of least resistance for
adopters.

That means an engineer should not need to invent local nouns for:

- lane
- claim boundary
- evidence strength
- breadth or locality posture
- allocation posture
- freshness or replay posture
- fallback or debt posture
- included-work and excluded-work disclosure
- canonical comparison
- stronger proof-bearing certification

The intended adoption experience is:

- common path:
  - declare honest descriptive meaning
  - disclose what the claim is about
  - disclose what work is counted and what is intentionally excluded
- lower lane:
  - attach canonical basis, counter evidence, comparison law, and explicit
    materialization planning
- stronger lane:
  - strengthen only when stronger certification or readmission is actually
    real

If an adopting crate still has to invent its own words for lane, basis,
fallback, width, allocation, or evidence strength after Milestone 8 lands, the
milestone did not actually make performance easy.

The minimal tangible artifact surface should be rich enough to express:

- a layout-intent artifact without claiming cost equivalence
- a compile-time contract without pretending runtime execution evidence exists
- a runtime-admitted breadth or budget receipt without pretending execution
  already happened
- an executed counter-backed receipt without pretending stronger support
  certification than it actually has
- a support-derived replay or reconstruction performance claim without
  upgrading it into authoritative current-basis cost truth
- attachment bundles that carry layout, claim boundary, evidence strength,
  breadth, allocation, freshness, and fallback posture together without
  collapsing them into one generic number

The first real artifact families should stay concrete and narrow:

- `LayoutIntentClaim`
  - answers "what representation family was this path designed around?"
  - does not claim execution evidence
- `PerformanceClaim`
  - answers "what boundary is being described, under what posture, with what
    evidence strength?"
  - does not require counters yet
- `PolicyAdmissionReceipt`
  - answers "what path was admitted, denied, widened, or deferred before
    execution?"
  - cannot masquerade as executed cost truth
- `CounterBackedPerformanceReceipt`
  - answers "what structurally counted work actually happened at execution?"
  - remains weaker than certified/readmitted proof-bearing bundles
- `PerformanceReport`
  - answers "what richer explanatory/counter/support bundle was explicitly
    materialized?"
  - must be visibly more expensive than plain claim access
- `CertifiedPerformanceBundle`
  - answers "what stronger certified performance claim passed the proof lane?"
  - must route through `worth-proof`

## Practical Boundary Scenarios

Milestone 8 is not complete unless the vocabulary can be used honestly in at
least these scenario families:

- one `worth-query` live delivery lane emits consumed width, budgeted width,
  density posture, and allocation posture as a counter-backed execution claim
  without pretending to describe relational authority cost
- one `worth-relational` commit result emits authoritative mutation timing,
  publication timing, validation timing, and complexity delta as an
  executed-boundary claim distinct from support/report materialization cost
- one `worth-store` maintenance path emits density class, breadth budget, path
  class, and verified-versus-debt complexity posture without pretending that
  foreground reads admit the same operational work
- one `worth-runtime-bridge` replay-derived or lineage-derived support lane
  emits performance posture with explicit historical/replay locality rather
  than looking like current-basis authority truth
- one `worth-signal` contract proves compile-time, runtime-policy, and
  counter-test enforcement are different evidence strengths over the same
  semantic performance posture
- one transient-within-boundary staging structure contributes to a receipt or
  support claim as a temporary cost carrier without becoming durable layout
  truth
- one hot-path operational lane proves that optional forensic, replay, parity,
  and support-expansion work are excluded by default and do not silently leak
  into the operational claim
- one cold-path support or replay lane proves that replay, reconstruction,
  parity, or expanded report assembly are included explicitly rather than being
  hidden behind a hot-looking API
- one reduced-richness profile removes optional performance-facing descriptive
  rows while leaving authoritative domain truth unchanged
- one cross-crate comparison shows AoS-backed and SoA-backed producers can emit
  the same shared performance claim meaning while preserving different internal
  storage
- one branch-local or historical lane emits performance meaning with explicit
  freshness/retention posture so stale support does not masquerade as current
  hot-path cost
- one expensive materialization lane is visibly materialized through an
  explicit builder or report API rather than a cheap-looking getter

## Naive Traps To Reject

- one universal performance envelope that tries to mean layout intent, runtime
  counters, support debt, replay cost, and authority cost all at once
- elapsed-time-only claims with no named boundary, breadth posture, or evidence
  strength
- layout vocabulary that silently implies cost equivalence between distinct
  representations
- compile-time policy claims that silently stand in for executed counter-backed
  evidence
- support-derived reconstruction claims that silently upgrade into current-
  basis authority cost truth
- replay/restoration/materialization work hidden behind cheap-looking getters
- one shared measurement runtime or container library disguised as shared
  vocabulary
- one giant helper file that owns layout, cost boundaries, attachment law,
  comparison, support, and readiness together
- debt or widened fallback expressed as ordinary verified cost posture
- transient staging, batch-local, or replay-only cost surfaces silently treated
  as durable authoritative layout truth
- a hot-path claim that omits its included-work and excluded-work classes
- a certified-looking claim minted directly from counter rows without the
  stronger proof lane
- one builder that lets callers pick contradictory combinations and only fails
  later in ad hoc runtime branches

The included-work and excluded-work rule is especially strict for high-strength
operational claims:

- no verified hot-path claim may exist without explicit included-work and
  excluded-work disclosure
- no certified hot-path bundle may exist without carrying through that
  disclosure intact
- omission is only allowed for weaker surfaces where the weakness itself is
  explicit, such as debt, deferred, rejected, or support-derived claims

## Phases

These phases are linear and mandatory. They are not parallel workstreams, not
alternative implementation styles, and not a buffet. Each phase establishes a
surface that the next phase consumes. If a later phase appears blocked, the
correct response is to complete or repair the earlier phase rather than
sidestepping it with local helpers or provisional shortcuts.

### Phase 1: Performance Primitive And Category Law

This phase is the vocabulary freeze. Nothing later in the milestone is allowed
to invent new meaning families ad hoc. The engineer should leave this phase
with a closed set of primitive types and legality rules that every later phase
must consume rather than reinterpret.

Work this phase in strict order. First, define the closed primitive families for
layout intent, performance boundary, and evidence strength. Second, define the
posture families for breadth/locality, allocation, access pattern, execution
temperature, freshness/retention, and fallback/debt. Third, define work-class
families so included work and excluded work are typed meaning, not prose. Only
after those families exist should this phase define the legality matrix that
states which combinations are valid, which are contradictory, and which require
stronger lanes later.

The implementation output from this phase should look like a set of stable,
sealed primitive modules plus compile-fail boundaries for family substitution
and contradiction. No report builders, no canonical bundles, and no
materialization helpers should exist yet. The only acceptable result is a clean
typed vocabulary with canonical ordering and equality law that later phases can
depend on without reopening the ontology.

Phase 1 is complete only when:

- the primitive families are closed and canonical
- primitive substitution and obvious contradictions fail closed
- access posture, execution temperature, and work-class vocabulary exist as
  first-class types
- blind consumers can distinguish the primitive families without producer
  folklore
- later phases can refer to these primitives without redefining them locally

### Phase 2: Claim-Boundary, Breadth, And Evidence-Strength Law

This phase builds the first real claim objects on top of the frozen primitive
families. The job here is not reporting yet. The job is to make it impossible
to talk vaguely about â€œperformanceâ€ without naming what boundary is being
described, what work is included, what work is excluded, and how strong the
claim really is.

Start by defining the core claim shapes for authoritative execution, derived or
support work, replay or materialization work, and planning or admission-only
work. Then define how included-work and excluded-work classes attach to those
claims. Then wire in breadth/locality, access posture, execution temperature,
and freshness law so a claim becomes a complete semantic statement rather than
a partial label. Only after the claim shape is complete should this phase add
compile-fail contradiction boundaries and evidence-strength boundaries.

The output from this phase should be a set of common-path claim builders and
sealed lowered claim types that already prevent the worst kinds of ambiguity:
hot-path versus support-only collapse, current-basis versus stale-support
collapse, and support-derived versus executed-evidence collapse. An engineer
should be able to construct a claim honestly at the call site, and the type
system should reject contradictory shapes before any runtime policy or counter
attachment exists.

Phase 2 is complete only when:

- every claim must name its boundary, evidence strength, and work disclosure
- breadth/locality, access posture, execution temperature, and freshness are
  carried on the claim itself
- contradictory claim shapes fail at construction time
- support-derived and executed claims remain mechanically distinct lowered
  types
- later phases can attach policy or counters without changing the claim meaning

### Phase 3: Layout Intent, Access Posture, And Allocation Law

This phase answers the â€œwhy is this laid out this way?â€ question without
standardizing the representation itself. It is intentionally after Phase 2,
because layout intent only becomes meaningful once claim boundaries and work
disclosure already exist.

Begin by defining the layout-intent families as declarative meaning, not
storage machinery. Then define the access-pattern and maintenance-posture
families that explain what the representation is optimized for. Then add
allocation-scope law so callers can distinguish no-allocation, action-local,
arena-local, batch-local, and report-scoped behavior without guessing from
implementation detail. Finally, add the anti-lie enforcement: layout intent
must not imply cost equivalence, must not imply one storage model, and must not
permit a claim to overstate what its representation proves.

The output from this phase should be concrete layout and access definition
surfaces that can be attached to claims and bundles later, plus hostile
representation-boundary tests proving that different internal layouts can emit
the same shared meaning. The engineer should leave this phase with
representation freedom preserved and with no room left for a future crate to
smuggle â€œfast because SoAâ€ or â€œshared because packedâ€ into the vocabulary.

Phase 3 is complete only when:

- layout intent is explicit, family-distinct, and non-equivalence-bearing
- access posture and allocation posture are typed and separately attachable
- representation freedom is preserved under hostile tests
- later phases can attach layout and access meaning without reopening storage
  design questions

### Phase 4: Runtime Policy, Budget, And Fallback Vocabulary

This phase introduces runtime-policy meaning, but only after claims and layout
meaning already exist. The point is to model admission, denial, widening,
deferral, and debt as their own typed lane rather than letting runtime policy
pretend it already has execution evidence.

Build this phase around explicit policy-admission surfaces. First define the
budget and admissibility vocabulary for breadth, density, locality, and
freshness-sensitive work. Then define the fallback families: verified,
deferred, widened, rejected, and debt. Then define policy-admission receipts
that lower a claim plus runtime policy facts into a visible pre-execution
artifact. These receipts must disclose included and excluded work classes for
hot, cold, support, and recovery lanes, and they must never masquerade as
executed cost truth.

The output from this phase should be explicit policy builders and lowered
policy-admission receipts plus hostile tests proving that widening and replay or
materialization expansion cannot hide inside verified narrow claims. An
engineer should be able to inspect a policy receipt and know exactly what was
admitted, what was refused, what was widened, and what stronger evidence still
does not exist yet.

Phase 4 is complete only when:

- runtime policy and budget results lower into explicit policy-admission
  receipts
- verified, deferred, debt, widened, and rejected policy outcomes remain
  distinct
- hot/cold/support/recovery policy surfaces disclose included and excluded work
- a policy receipt cannot be mistaken for executed counter-backed truth
- later phases can attach counters to policy-approved shapes without changing
  policy meaning

### Phase 5: Canonical Basis, Counter Attachment, And Comparison Law

This phase is where the milestone becomes mechanically comparable. Up to this
point the system can describe claims and policy honestly; now it must lower
those claims into canonical bundles that can carry counter specs, counter rows,
and mismatch meaning without adopting one telemetry engine.

Work this phase in sequence. First define canonical basis participation for
performance claims and bundles so equality and comparison are not producer
folklore. Then define the attachment surfaces for structural counters, contract
names, and supporting evidence rows. Then define mismatch and comparison law so
differences in boundary, freshness, evidence strength, or work disclosure can
be explained precisely. Only after the canonical bundle shape exists should
counter-backed receipts be introduced as a distinct lowered artifact.

The output from this phase should be explicit bundle builders, comparison
artifacts, and counter-backed receipts that say exactly what structurally
counted work happened. The engineer should be able to compare two bundles and
learn why they mismatch, not just that they differ. This is also the point
where raw counter bags and crate-local telemetry snapshots must be fenced out of
the shared API unless they lower through the canonical bundle lane.

Phase 5 is complete only when:

- canonical bundles exist for performance claims and counter-backed receipts
- structural counters attach through shared lowering rather than raw blobs
- comparison artifacts explain mismatch precisely, including work disclosure
- exact counter-contract testing is possible at the shared bundle surface
- later phases can materialize richer reports from canonical bundles without
  redefining comparison or counter meaning

### Phase 6: Attachment, Materialization, And Bundle Law

This phase connects the shared performance meaning to the artifact families
defined by earlier milestones and makes expensive report assembly visibly
expensive. Nothing in this phase should look like a cheap getter if it may
materialize cold-path or support-path detail.

Start by defining where claims and receipts may legally attach: summaries,
receipts, reports, artifacts, and support or certification bundles. Then define
report-request object specs and report-plan lowered forms so richer explanatory
or support-bearing output is planned before it is materialized. Then add
profile-driven elision rules so optional descriptive detail can be removed
centrally without mutating authoritative truth. Finally, define the hard
boundary law: hot-path accessors may inspect hot-path claim meaning, but only
explicit report/materialization APIs may widen into cold/support expansion.

The output from this phase should be explicit report requests, report plans, and
materialized reports, plus attachment legality tests across summaries, receipts,
reports, and support bundles. An engineer should be able to inspect a report
plan before materialization and see exactly what extra sections, rows, or
support detail will be assembled.

Phase 6 is complete only when:

- claims and receipts attach legally to earlier artifact families
- explicit report planning exists before report materialization
- reduced richness can elide optional descriptive surfaces centrally
- expensive materialization boundaries are visible in the type and method shape
- hot operational accessors cannot silently widen into cold/support expansion

### Phase 7: Production-Test Readiness

This phase is the freeze point for implementation and migration. The engineer
should not move on to documentation or downstream adoption until the public
surface, proof boundaries, hostile obligations, and finished DX topology are
machine-checkably locked.

Do this phase as an inventory and certification phase, not as a vague cleanup
pass. First inventory the exact public surface: modules, facade entrypoints,
builders, object specs, lowered receipts, report plans, reports, certified
bundles, and readiness artifacts. Then inventory the compile-fail boundaries,
hostile scenario families, canonical fixtures, and harness expansion points.
Then inventory the exact finished DX topology so migration cannot deform the
API into helper sprawl. Then freeze which surfaces are descriptive, which are
mechanically observable, which are certified, and which are readmitted. Only
after those inventories exist should the readiness artifact be certified
through the stronger proof lane.

The output from this phase should be a readiness artifact that an engineer can
use as a closure checklist and that a hostile reviewer can use to prove the
milestone is actually frozen. If a later implementer could still plausibly ask
â€œis this supposed to be in common, lowering, certified, or readiness?â€ then
this phase is not complete.

Phase 7 is complete only when:

- the public surface and DX topology are machine-checkably inventoried
- compile-fail, hostile, canonical-fixture, and readiness obligations are
  named exactly
- the milestone says exactly which surfaces require `worth-proof` and which do
  not
- adoption assumptions, non-assumptions, and residual debt are explicit
- migration can begin without reopening the public design

### Phase 8: Feature Docs, Crate-Doc Integration, And Publication Closure

This is the terminal implementation-delivery phase. The implementation is not
complete until the shipped surface has been turned into real crate-facing docs
through the `feature-doc-writer` skill and integrated into the crateâ€™s
documentation tree in the final published shape.

This phase exists to prevent the common failure mode where the milestone closes
with strong internals and readiness artifacts but without a durable
documentation system that future adopters can actually use. The engineer should
treat this as product-surface delivery, not optional cleanup.

Work this phase in strict order. First, run the `feature-doc-writer` skill
against the final shipped Milestone 8 surface rather than against the spec.
Second, create category folders under
`crates/worth-foundational/docs/` so the documentation tree is organized by
feature category rather than as a flat pile of markdown files. Third, write
exactly one feature document per real feature seam. A seam gets one primary
document, not several overlapping notes. Fourth, ensure each document follows
the skillâ€™s feature-doc standards: problem-first framing, stable entry points,
core mental model, execution model, small example, real example, inspection and
debugging guidance, anti-patterns, current limits, and related docs. Fifth, add
the resulting category folders and landing pages to the crate-facing
documentation entrypoints so the docs are actually discoverable.

The expected final documentation topology should look like an intentional
documentation system. For example:

- `crates/worth-foundational/docs/performance/README.md`
- `crates/worth-foundational/docs/performance/common-performance-claims.md`
- `crates/worth-foundational/docs/performance/policy-admission-receipts.md`
- `crates/worth-foundational/docs/performance/counter-backed-performance-receipts.md`
- `crates/worth-foundational/docs/performance/performance-bundles-and-comparison.md`
- `crates/worth-foundational/docs/performance/performance-report-planning-and-materialization.md`
- `crates/worth-foundational/docs/performance/certified-and-readmitted-performance-bundles.md`
- `crates/worth-foundational/docs/performance/performance-readiness.md`

The exact filenames may evolve, but the governing law does not: docs live in
folders by category, and each real feature gets one primary document.

Phase 8 is complete only when:

- the `feature-doc-writer` skill has been used for the final feature-doc pass
- the docs live under crate-doc category folders rather than a flat directory
- each real feature seam has exactly one primary feature document
- the landing pages and feature docs are linked into the crateâ€™s documentation
  surface
- an adopter can learn the performance vocabulary, lowering lane, stronger
  proof lane, and readiness lane from crate docs alone without reading the
  milestone spec

## Compile-Time Boundary Targets

Milestone 8 should convert the following assumptions into compile-fail
boundaries wherever visibility, sealed constructors, phantom types, or
proof-bearing wrappers can enforce them:

- a layout-intent artifact cannot satisfy an API requiring a performance claim
- a compile-time contract cannot satisfy an API requiring counter-backed
  execution evidence
- a runtime-policy admission receipt cannot satisfy APIs requiring executed
  counter-backed performance truth
- a support-derived replay or reconstruction claim cannot satisfy APIs
  requiring current-basis executed cost truth
- contradictory claim shapes such as hot-path plus support-only or
  exact-basis-current plus stale-support cannot be represented
- a cold-path or support-only claim cannot satisfy APIs requiring hot-path
  operational posture
- a widened or debt-classified claim cannot satisfy APIs requiring verified
  narrow posture
- a hot-path certified claim cannot be constructed without explicit
  included-work and excluded-work disclosure
- a generic elapsed-time blob cannot satisfy canonical performance comparison
  APIs
- a cheap-looking accessor cannot materialize an explicitly expensive report
  family
- one crate-local telemetry snapshot cannot satisfy a shared canonical
  performance bundle API without explicit lowering
- a transient-within-boundary cost carrier cannot satisfy APIs requiring
  durable layout truth
- a replay-derived or restored/readmitted performance claim cannot satisfy APIs
  requiring current-basis exact freshness
- a raw counter bag cannot bypass the canonical attachment lane for structural
  counter evidence
- a plain counter-backed receipt cannot satisfy APIs requiring stronger
  certified or readmitted performance evidence
- a certified/readmitted bundle cannot be minted without routing through the
  standardized `worth-proof` lane

## What Must Ship

- shared layout vocabulary with family-distinct layout intent categories
- shared performance-boundary vocabulary for authority, materialization,
  replay, support, maintenance, publication, delivery, and recovery surfaces
- shared evidence-strength vocabulary distinguishing compile-time, runtime
  policy, counter-backed execution, support-derived, and debt/deferred claims
- shared breadth/locality, allocation, execution-temperature,
  freshness/retention, and fallback/debt posture vocabulary
- canonical basis and comparison law for performance claims and performance
  bundles
- attachment and materialization law for performance claims over summaries,
  reports, artifacts, receipts, and support bundles
- explicit compile-fail boundaries preventing claim-strength and claim-boundary
  collapse
- explicit stronger-lane law for what does and does not require `worth-proof`
- production-test readiness and feature docs

## Semantic Guarantees

- layout and performance terms must carry one shared meaning across crates
- layout intent never implies cost equivalence between distinct
  representations
- every performance-facing claim must name the boundary where it is valid
- authority cost, derived/support cost, replay cost, and materialization cost
  remain mechanically distinct surfaces
- compile-time, runtime-policy, counter-backed, support-derived, and
  debt/deferred claims remain mechanically distinct evidence strengths
- descriptive, mechanically observable, certified, and readmitted performance
  surfaces remain explicitly distinct rather than being implied by naming
- hot-path, warm-path, cold-path, recovery-only, and support-only performance
  claims remain explicit and do not silently collapse into one generic
  operational posture
- replay-derived, restored/readmitted, historical, and stale-support
  performance claims remain explicit instead of masquerading as current-basis
  hot-path truth
- transient-within-boundary cost carriers may appear in receipts or support
  evidence without being promoted into durable layout truth
- reduced-richness profiles may elide optional performance-facing descriptive
  surfaces without changing authoritative domain outcomes
- hot-path claims exclude optional forensic/support/replay expansion by default
  unless explicitly declared otherwise
- stronger certified or readmitted performance claims must route through the
  shared `worth-proof` lane rather than crate-local witness patterns
- structural counters may attach to shared bundles without requiring one shared
  measurement runtime

## Representation Boundaries

- foundational performance vocabulary standardizes boundary meaning, not one
  storage representation, one benchmark harness, one telemetry runtime, or one
  scheduler
- crates may retain local AoS, SoA, AoSoA, sparse, packed, arena-backed, or
  custom topologies and materialize canonical performance meaning only at
  explicit boundaries
- shared attachment surfaces must not require crates to expose deep internal
  runtime state just to satisfy the canonical shape
- performance-claim comparison law must not require one counter namespace or
  one instrumentation engine
- proof-bearing artifacts may attach Milestone 8 surfaces without moving
  descriptive ownership into `worth-proof`

## Cross-Crate Comparison Non-Goals

Shared vocabulary does not imply shared performance quality.

Milestone 8 comparison law exists to standardize claim meaning and mismatch
explanation, not to prove that two crates have the same:

- slope
- constant factors
- allocation profile
- planner quality
- scheduler quality
- runtime throughput
- budget envelope
- operational temperature under one workload

What the milestone may standardize is:

- that two claims mean the same kind of thing
- that the same work classes are included or excluded
- that the same evidence strength is or is not present
- that the same freshness, fallback, and locality posture is or is not present

If two crates share the same canonical performance meaning and still have very
different cost curves, Milestone 8 is working correctly. Shared vocabulary is
not a hidden claim of equal speed.

## Must Preserve

- no universal container that pretends layout families are cost-equivalent
- no one-category performance envelope that collapses boundary, evidence
  strength, freshness, and debt posture
- no concealment of replay, restore, support, or materialization breadth
  behind cheap-looking APIs
- no forcing of `worth-relational`, `worth-query`, `worth-store`,
  `worth-runtime-bridge`, or `worth-signal` into one internal measurement or
  layout model
- no silent leakage of cold-path reconstruction, support expansion, or replay
  parity work into a hot-path operational claim
- no silent upgrade of support-derived or replay-derived performance claims
  into authoritative current-basis cost truth
- no silent upgrade of transient staging structures into durable layout truth
- no requirement that disabling optional performance-facing descriptive
  richness be implemented by touching leaf call sites throughout domain code

## Desired DX End State

An engineer should be able to predict the right surface before opening the
implementation:

- use layout vocabulary to answer "what structural representation family is
  this designed around?"
- use performance-boundary vocabulary to answer "what exact work boundary is
  this claim about?"
- use evidence-strength vocabulary to answer "how strong is the claim?"
- use breadth/allocation/execution-temperature/freshness/fallback posture
  vocabulary to answer "what operational conditions shaped the claim?"
- use proof-bearing strengthening only when stronger certification or
  readmission claims are real

A blind consumer should also be able to ask practical questions and predict the
surface family that answers them:

- "Is this about authoritative execution or report materialization?" ->
  performance-boundary vocabulary
- "Is this compile-time law, runtime admission, or executed evidence?" ->
  evidence-strength vocabulary
- "Is this narrow and verified, or widened/debt-classified?" -> breadth and
  fallback posture vocabulary
- "Is this really a hot-path claim, or is cold-path/support work included?" ->
  execution-temperature posture vocabulary
- "Does this claim apply to current truth or replay/restored/support basis?" ->
  freshness/retention posture vocabulary
- "Is this describing a durable representation or a transient cost carrier?" ->
  layout-intent versus transient-within-boundary cost evidence

The public surface should teach at least three distinct lanes:

- a common descriptive path for producing and inspecting layout and
  performance-claim primitives
- a lower lane for canonical attachment, comparison, and materialization work
- a stronger lane for proof-bearing certification, readmission, and readiness
  artifacts

The implementation should also obey the DX law that object specs encode shape
while builders encode progression:

- object specs should define closed, inspectable shapes such as:
  - `LayoutIntentDefinition`
  - `PerformanceBoundaryDefinition`
  - `WorkClassSet`
  - `PerformanceCounterSpec`
  - `PerformanceReportRequest`
  - `PerformanceReadinessInventory`
- builders should be reserved for ordered proof/progression surfaces such as:
  - constructing a `PerformanceClaim`
  - lowering a claim into a canonical bundle
  - attaching counters and evidence into a materialized report
  - certifying or readmitting stronger bundles

The finished API should also obey the DX law that friendly authoring surfaces
lower into inspectable accountability surfaces before execution or
materialization. That means:

- common-path authoring should produce:
  - intent-shaped claim builders and policy-admission builders
- lower-lane accountability should produce:
  - canonical claim bundles
  - explicit policy-admission receipts
  - explicit counter-backed receipts
  - explicit performance-report plans and materialized reports
- stronger-lane accountability should produce:
  - certified bundles
  - readmission bundles
  - readiness artifacts

The caller should be able to inspect the accountability surface before any
expensive materialization or stronger certification happens:

- what boundary is being named
- what work is included
- what work is excluded
- what locality, access, allocation, and execution-temperature posture apply
- what evidence strength exists now
- what stronger proof lane, if any, is still required

## Finished Code Shape

Milestone 8 should explicitly freeze what the finished code looks like when it
is being used successfully. The milestone is not complete if the ontology is
correct but the code still feels improvised.

### Public Module Topology

The public topology should read like a deliberate three-lane system rather than
a generic `performance` bucket:

- `worth_foundational::performance::common`
  - descriptive nouns and progression builders
- `worth_foundational::performance::lowering`
  - canonical lowering, receipts, comparison, and explicit materialization
- `worth_foundational::performance::certified`
  - stronger proof-bearing certification and readmission surfaces
- `worth_foundational::performance::readiness`
  - machine-checkable public-surface inventory and readiness closure

The common path should contain no materialization, no hidden proof minting, and
no cold-path expansion helpers. The lowering path should contain no authority to
mint stronger certified/readmitted claims. The certified path should not own
descriptive vocabulary. The readiness path should not be a generic dumping
ground for arbitrary test helpers.

### Internal Responsibility Topology

The internal module structure should preserve the semantic fracture lines so the
next correct edit is obvious:

- `performance/primitives/`
  - closed families for layout intent, boundary, evidence strength, access
    posture, execution temperature, freshness, fallback, and work classes
- `performance/claims/`
  - common-path claim builders, sealed constructors, and contradiction law
- `performance/policy/`
  - policy-admission specs and receipts
- `performance/receipts/`
  - counter-backed execution receipts and supporting lowered evidence
- `performance/reports/`
  - explicit report requests, report plans, and materialized reports
- `performance/basis/`
  - canonical basis participation, comparison, and mismatch vocabulary
- `performance/certified/`
  - proof-bearing stronger bundles and readmission
- `performance/readiness/`
  - public-surface inventory, proof inventory, hostile-pressure inventory,
    compile-fail inventory, and adoption assumptions

If the implementation instead wants one `performance.rs` or one mixed
`mod.rs` that owns primitives, builders, receipts, reports, proof, and
readiness together, the milestone should treat that as a structural failure.

### Facade Shape

The finished facade should teach correct usage in autocomplete:

```rust
use worth_foundational::performance::{
    common,
    lowering,
    certified,
    readiness,
};
```

Expected entrypoints should be narrow and semantically obvious:

- `common::claim()`
- `common::policy_admission()`
- `common::define_layout_intent(...)`
- `common::define_counter_spec(...)`
- `lowering::bundle(...)`
- `lowering::compare(...)`
- `lowering::report_request(...)`
- `certified::bundle(...)`
- `certified::readmit(...)`
- `readiness::inventory(...)`

The facade should not force callers through vague entrypoints like:

- `performance::build()`
- `performance::run()`
- `performance::helpers()`
- `performance::report()`

unless the returned type immediately narrows into a named responsibility.

### Object Specs Versus Builders

Milestone 8 should explicitly standardize which surfaces are object specs and
which are builders:

- object specs:
  - `LayoutIntentDefinition`
  - `PerformanceCounterSpec`
  - `PerformanceReportRequest`
  - `PerformanceReadinessInventory`
  - `WorkClassSet`
- builders:
  - `PerformanceClaimBuilder`
  - `PolicyAdmissionBuilder`
  - `PerformanceBundleBuilder`
  - `CertifiedPerformanceBundleBuilder`

The object-spec surfaces should make the whole shape visible at once. The
builder surfaces should encode ordered progression and next-valid-step
autocomplete. A report request should not be a long chained builder if its job
is just to define a stable shape. A claim or certification path should not be a
flat object bag if its job is to encode proof progression.

### Lowered Accountability Surfaces

DX law requires a lowered accountability surface before expensive work. For
Milestone 8 that means the code should expose explicit lowered forms such as:

- `PerformanceClaim`
- `PolicyAdmissionReceipt`
- `CounterBackedPerformanceReceipt`
- `PerformanceBundle`
- `PerformanceComparison`
- `PerformanceReportPlan`
- `MaterializedPerformanceReport`
- `CertifiedPerformanceBundle`
- `ReadmittedPerformanceBundle`

The caller should be able to inspect these lowered surfaces directly:

```rust
let bundle = lowering::bundle(claim)
    .attach_contract(query_contract)
    .attach_counter_spec(counter_spec)
    .finish()?;

bundle.boundary();
bundle.evidence_strength();
bundle.access_posture();
bundle.included_work();
bundle.excluded_work();
bundle.counter_spec();
bundle.requires_materialization();
```

The materialization path should lower first, then materialize:

```rust
let report_plan = lowering::report_request(PerformanceReportRequest {
    claim: bundle,
    include_counters: true,
    include_support_rows: true,
    include_diagnostics: false,
})?
.plan();

report_plan.claim();
report_plan.materialization_boundary();
report_plan.included_sections();
report_plan.excluded_sections();

let report = report_plan.materialize()?;
```

That is the crown molding part: the code should make the expensive boundary
visible before the expensive work starts.

### Proof-Carrying Progression

The finished code should make the performance progression teachable and hard to
misuse:

```rust
let claim = common::claim()
    .boundary_authoritative_execution()
    .counter_backed_execution()
    .hot_path()
    .point_local()
    .point_lookup()
    .exact_basis_current()
    .verified()
    .include_authoritative_mutation_work()
    .exclude_support_report_assembly_work()
    .finish()?;

let bundle = lowering::bundle(claim)
    .attach_contract(query_contract)
    .attach_counter_spec(counter_spec)
    .finish()?;

let receipt = bundle.attach_counter_rows(counter_rows)?;

let certified = certified::bundle(receipt)
    .certify_hot_path(proof_artifact)?
    .readmit_for_current_basis(current_basis_proof)?;
```

The important part is not the exact method names. The important part is that:

- authoring intent is readable
- lowered accountability is inspectable
- expensive materialization is explicit
- stronger proof progression is separate and visibly stronger

### Caller Responsibilities By Lane

The finished code should make caller responsibility changes obvious:

- common lane:
  - declare meaning
  - choose posture
  - disclose included/excluded work
- lowering lane:
  - attach canonical comparison/basis information
  - attach structural counter specs and rows
  - inspect cost/accountability before materialization
- certified lane:
  - supply stronger proof artifacts
  - opt into trust-boundary bridging and readmission
- readiness lane:
  - freeze exact public-surface and proof inventories

If a caller can accidentally cross from descriptive meaning into expensive
materialization or stronger certification without acknowledging that boundary in
the type/method shape, the milestone is incomplete.

The finished code should also look intentional at the call site. The exact
final names may shift, but the shape should look roughly like this:

```rust
use worth_foundational::performance_vocabulary_api::{
    common_path as performance,
    lower_lane,
    stronger_lane,
};
```

The common descriptive path should read like intent, not raw telemetry
assembly:

```rust
let claim = performance::claim()
    .boundary_authoritative_execution()
    .counter_backed_execution()
    .hot_path()
    .point_local()
    .action_local_allocation()
    .exact_basis_current()
    .verified()
    .finish()?;

claim.boundary();
claim.evidence_strength();
claim.execution_temperature();
claim.breadth_posture();
claim.freshness_posture();
```

Runtime-policy admission should stay visibly pre-execution:

```rust
let admission = performance::policy_admission()
    .boundary_authoritative_execution()
    .hot_path()
    .delta_bound()
    .exact_basis_current()
    .admit_verified()
    .exclude_optional_support_expansion()
    .finish()?;

admission.boundary();
admission.evidence_strength();
admission.included_work();
admission.excluded_work();
```

Support-derived or replay-derived claims should remain visibly weaker and
cost-bearing:

```rust
let support_claim = performance::claim()
    .boundary_replay_reconstruction()
    .support_derived()
    .cold_path()
    .replay_derived()
    .stale_support()
    .debt()
    .finish()?;

support_claim.boundary();
support_claim.evidence_strength();
support_claim.execution_temperature();
support_claim.freshness_posture();
support_claim.fallback_posture();
```

Explicit materialization should remain visible:

```rust
let report = lower_lane::materialization::performance_report()
    .from_claim(claim)
    .attach_contract_name("query.snapshot_explicit_targets")
    .attach_counter_rows(counter_rows)
    .declare_included_work(included_work)
    .declare_excluded_work(excluded_work)
    .materialize()?;

report.claim();
report.counter_rows();
report.included_work();
report.excluded_work();
```

The stronger lane should make proof-bearing strengthening visibly stronger:

```rust
let certified = stronger_lane::certified_bundle()
    .certify_hot_path_bundle(report_bundle, proof_artifact)?
    .readmit_for_current_basis(current_basis_proof)?;

certified.proofs();
certified.readmission_basis();
certified.certified_claim();
certified.certified_surfaces();
```

Readiness should remain a separate stronger surface rather than being bundled
into every certified claim:

```rust
let readiness = stronger_lane::readiness()
    .certify_public_surface_inventory(surface_inventory, readiness_proof)?;

readiness.certified_surfaces();
readiness.compile_fail_boundaries();
readiness.hostile_pressures();
readiness.runtime_adoption_pressures();
```

## Acceptance Evidence

- layout vocabulary tests proving AoS, SoA, AoSoA, sparse, packed, and custom
  remain family-distinct without cost-equivalence overclaim
- hostile representation-boundary tests proving distinct internal layouts can
  materialize the same shared performance meaning without shared storage
- compile-fail tests proving evidence-strength families cannot substitute for
  each other
- claim-boundary tests proving authority cost, support cost, replay cost, and
  materialization cost remain distinct
- execution-temperature tests proving hot-path, cold-path, recovery-only, and
  support-only claims remain explicit and non-substitutable
- included/excluded-work tests proving hot-path claims disclose what was
  counted and what was intentionally elided
- freshness/retention tests proving replay-derived, restored/readmitted, and
  stale-support performance claims remain explicit
- transient cost-carrier tests proving transient-within-boundary structures can
  appear in receipts/support claims without becoming durable layout truth
- reduced-richness profile tests proving optional performance-facing
  descriptive surfaces can be removed without changing authoritative domain
  truth
- hot-path elision tests proving optional forensic, replay, parity, and
  support-expansion work can be removed centrally from a hot lane
- cold-path disclosure tests proving replay/materialization/support expansion
  is included explicitly in cold/support/recovery claims
- canonical basis and comparison tests proving independently produced but
  semantically identical performance claims compare and digest the same way
- structural-counter attachment tests proving counters can ride shared bundles
  without dictating the underlying telemetry engine
- exact counter-contract tests proving named hot-path claims assert structural
  counters by exact meaning rather than elapsed-time thresholds
- expensive-materialization tests proving APIs that require broad report
  materialization make that boundary visible in type or method shape
- proof-lane tests proving certified/readmitted performance bundles cannot be
  minted outside the standardized `worth-proof` lane
- readiness certification and grouped public-surface inventory proof

## Architectural Notes

- Layout intent is not the same as performance claim boundary.
- Access-pattern posture is not the same as layout intent.
- Performance claim boundary is not the same as evidence strength.
- Evidence strength is not the same as proof law, but it must remain strong
  enough that blind consumers do not guess.
- Included-work and excluded-work classes are part of the claim meaning, not
  optional diagnostics garnish.
- Structural counters and elapsed time may both appear in a bundle, but
  elapsed time alone is not a sufficient shared performance claim.
- Milestone 8 should standardize the language that later crate-specific
  contracts attach to; it should not replace crate-specific hot-path contract
  registries.

## Sequencing Notes

- Milestone 8 belongs after Milestone 7 because performance claims need the
  replay-derived, restored/readmitted, transient, support-truth, and
  planned-versus-executed nuance that Milestone 7 standardized.
- Milestone 8 belongs before cross-crate migration and closure because the
  migrating crates need one shared language for performance meaning before
  their local dialects can converge honestly.
- Milestone 8 should land before any attempt to present one workspace-wide
  performance story, because otherwise the workspace will compare incompatible
  local claim shapes and call them the same thing.

## Explicit Non-Goals

- one shared benchmark harness for all crates
- one shared telemetry runtime or counter storage topology
- one shared hot-path container library
- one shared scheduler, planner, or execution runtime
- one shared allocation strategy
- one cost-equivalence claim across layout families
- automatic proof that a given crate's performance is "good" in absolute terms
  rather than honest in the boundary it names

## Self-Check

- Does this milestone solve a real structural problem or just package work
  cosmetically?
  - It solves the lack of canonical shared meaning for performance and layout
    claims across crates that already carry serious local cost semantics.
- Is the adversarial constraint precise and load-bearing?
  - Yes. It forces cross-crate performance meaning to survive differing
    storage, replay, support, and authority models without collapsing into one
    fake runtime.
- Does the milestone preserve crate authority boundaries?
  - Yes. Foundational owns meaning and attachment law; crates retain storage,
    planners, telemetry engines, and execution.
- Does the milestone define proof obligations, not just implementation tasks?
  - Yes. Compile-fail, hostile representation-boundary, comparison, freshness,
    transient-cost, and readiness evidence are all explicit.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  - Yes. The spec names the primitive families, attachment law, ordered phases,
    and acceptance evidence clearly enough to do so.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  - It belongs here because Milestone 7 supplies the descriptive nuance this
    milestone depends on, and Milestone 11 migration needs this language frozen
    before convergence work begins.

