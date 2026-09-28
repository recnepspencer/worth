# Milestone 3.8 Engineering Spec: Allocation Receipts, Incremental Replanning, Scroll, Portal, And Continuous Interaction Measurement

> Terminology: ledger lifecycle and receipt ledgers in this specification are
> product-domain runtime behavior, not QA, proof, closure, or phase ledgers.
> The historical-ledger retirement policy does not apply to them.

> **Status:** Closed — Phases 1-22 implemented and milestone-native hostile
> certification complete
>
> **Roadmap parent:** [worth_ui_roadmap.md](./worth_ui_roadmap.md)
>
> **Primary prerequisites:**
> `Milestone 3.6a Measurement Vocabulary, Basis Admission, And Host Evidence Boundaries`
> `Milestone 3.6b Allocation Neighborhood Planning And Constraint Propagation`
> `Milestone 3.7 Structural Runtime Cleanup And Boundary Closeout`
> `Worth Query Milestone 9.13 Declarative Query Experience And Installed Domain Authority`
>
> **Follow-on sequence:** `Milestone 3.9 execution-plan lowering, equivalence, and frame-cost surfaces`
>
> **Primary architectural driver:** turn admitted measurement basis and allocation planning into committed allocation truth that survives resize, drag, scroll, portal, and high-frequency stream pressure without broad churn or host-owned layout folklore.

## Goal

Freeze committed allocation truth and continuous-measurement semantics as a
runtime-owned lane.

Milestone 3.8 is complete when Worth UI can take admitted
`UiMeasurementBasis`, admitted `UiAllocationNeighborhood`, and `plan_allocation`
output; own the frame-dispatch boundary that turns admitted host, interaction,
and Query observations into sealed stream frames; classify those frames into
typed invalidation families; replan only the affected allocation neighborhood;
and commit explicit `UiAllocationReceipt` artifacts with declared freshness,
identity, reuse, denial, and inspection posture.

Because Query Milestone 9.13 changed the ordinary downstream contract while
3.8 was in progress, completion also requires Worth UI to carry
runtime-installed domain authority and Foundational-native projection values
through one binding-owned edge. Updating imports without deleting
consumer-local Query mirrors, reconstructed identity digests, or detached
bootstrap paths does not satisfy this requirement.

This milestone closes the first half of churn-heavy runtime geometry truth:

- what the runtime means by candidate allocation versus committed allocation
- what an allocation receipt is and what facts it carries
- how allocation receipt identity, generation, equivalence, and reuse work
- how resize, drag, scroll, portal-anchor, viewport, and content-growth
  changes enter as typed observations rather than host-local meaning
- how high-frequency streams map to preview, projection, derived-truth, and
  durable commit cadences
- how incremental replanning stays neighborhood-bounded under pressure

It does not close:

- full execution-plan lowering and frame-cost surface finalization from 3.9
- mounted receipt execution and paint-time geometry truth beyond allocation
  receipt commitment
- service-complete drag/drop or scroll UX above the allocation/runtime lane
- visual snapshot or human-inspector product surfaces
- later rebind, diagnostics, or AI-tooling milestones that consume these
  receipts more broadly
- generic application-local debounce policy as a replacement for typed stream
  semantics

## Why This Milestone Exists

3.6a and 3.6b deliberately stopped before committed allocation truth.

3.6a froze typed host measurement intake, `UiMeasurementResult`, and
`UiMeasurementBasis`. 3.6b froze allocation neighborhood admission and the
deterministic planning kernel that culminates in `plan_allocation`. 3.7 then
cleaned the runtime topology so future receipt and churn work would extend
owned lanes instead of reopening giant facades, helper swamps, or
certification-as-production-law.

That means Worth UI now has:

- typed measurement requests and host observations
- admitted measurement basis
- admitted allocation neighborhoods
- deterministic allocation planning
- identity-match and impact-narrowing work from replacement
- a cleaned public facade and proof-flow grammar for 3.8 to consume

What it does not yet have is the actual committed allocation runtime contract.

Without this milestone, the runtime would still be missing the hard answers to
the questions that matter under pressure:

- what exactly is committed allocation truth?
- what differentiates candidate allocation from committed receipt state?
- when is a prior receipt reusable, partially reusable, or denied?
- what typed invalidation family owns viewport churn, splitter drag, scroll
  extent change, portal-anchor movement, or content growth?
- which changes are preview-only and which are durable?
- what cadence is legal for stream ingress, local projection, derived receipt
  recompute, and durable commit?
- how can the runtime prove that only the affected allocation neighborhood was
  replanned?

If those answers stay implicit, later work will encode them in host adapters,
gesture handlers, local resize helpers, per-feature debounce rules, or broad
replan fallbacks. That would make `UiAllocationReceipt` look real while its
truth basis remains folklore.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.8 must start from hostile
  stream pressure, not from static layout demos.
- `arch_laws.md`
  protects authority/derivation separation and proof-bearing phase transitions.
  3.8 must keep stream ingress, candidate planning, committed allocation
  receipts, and downstream mounted behavior as distinct artifacts.
- `composition_laws.md`
  protects named semantic steps. 3.8 must not collapse stream classification,
  invalidation selection, receipt reuse, denial, and inspection into one giant
  replanning function.
- `domain_structure_laws.md`
  protects responsibility-shaped topology. 3.8 must give typed homes to
  allocation receipts, invalidation families, stream cadence policy, scroll
  semantics, portal semantics, counters, and certification.
- `perf_laws.md`
  protects bounded breadth, locality, and explicit equivalence contracts. 3.8
  must prove neighborhood-bounded replanning and receipt reuse without
  heuristic debounce folklore.
- `docs/coding-guidelines/dx_laws.md`
  protects organized truth as developer experience. The correct Query-bound
  UI path must be the shortest declarative path, invalid next acts must be
  absent from autocomplete, and advanced evidence must remain inspectable
  without making ordinary composition ceremonial.
- `worth_ui_roadmap.md`
  protects sequence. 3.8 must consume cleaned 3.7 surfaces after 3.6a/3.6b
  planning exists and before 3.9 execution-plan lowering broadens runtime cost
  surfaces.
- `workspaces/worth-ui/docs/worth-ui-readme.md`
  protects the actual Worth UI runtime stack. 3.8 must keep layout meaning in
  Worth UI runtime, host code in observation/mechanics lanes, and Query-backed
  facts in Query-owned public lanes.
- `worth-ui-dsl-vision.md`
  protects explicit layout operators and semantic lanes. 3.8 must give
  `scroll`, `portal_anchor`, `split`, `mosaic`, and related layout operators
  real runtime allocation semantics instead of implicit parent magic.
- `ai-diagnostics.md`
  protects one shared evidence substrate. 3.8 receipts, denials, freshness
  posture, and replan explanations must surface through typed evidence and
  inspection, not a separate layout-debug folklore lane. 3.8 must also
  preserve receipt-backed geometry and allocation explanation in a form that
  later AI visual-critique surfaces can consume for alignment, spacing,
  symmetry, and visual-drift evaluation instead of diagnosing only runtime
  correctness.
- `crates/worth-query/docs/AI_README.md`
  protects Query-owned basis, projection consumption, inspection, and
  cross-runtime explanation. 3.8 must consume installed-handle declarations,
  `WorthQueryProjectionOutcome`, retained consumed-projection authority, and
  `workspace.inspections().inspect(...)` only through the admitted Query/Worth
  binding seam when Query-backed content still affects allocation truth.
- `WORTH-query/milestone-9.13-closeout.md`
  protects runtime-installed domain handles and Foundational-native values as
  the only ordinary Query product path. 3.8 must preserve that authority
  through UI binding, allocation ingestion, inspection, and rebind rather than
  reconstructing it from reporting projections or local digests.

## Adversarial Constraint

3.8 must survive this hostile condition:

> A running Worth UI app contains nested mosaics, local composition regions,
> scroll-owned inspectors, portal-anchored dropdowns, viewport-relative shell
> regions, autosizing text input, Query-backed streaming collections, and
> user-resizable splitters. While a user types on every character, drags a
> splitter at pointer frequency, resizes the viewport continuously, receives
> streaming Query updates, and opens/closes anchored portals, the runtime must
> preserve exact source fact freshness, keep committed allocation truth
> separate from preview/candidate state, replan only the affected allocation
> neighborhood, explain every reuse or denial decision, and avoid broad
> unrelated replanning or host-owned semantic fallback.

The failure modes this milestone must prevent are:

- every keystroke becoming a global layout rewrite
- every pointer delta becoming a durable resize mutation
- every scroll tick becoming authoritative layout truth
- candidate allocation leaking into host behavior or inspection as if it were
  committed truth
- receipt reuse without a precise equivalence contract
- broad "something changed" invalidation replacing typed family intake
- host adapters deciding scroll, portal, or resize semantics locally
- inspection being unable to explain why a receipt exists, why reuse was
  denied, or why a neighborhood was selected

## Product Decision Lock

- `UiAllocationReceipt` is a real runtime truth artifact, not a convenience
  summary over host geometry.
- Candidate allocation results and committed allocation receipts are separate
  categories with separate identity, lifecycle, and inspection posture.
- 3.8 will not use one generic "commit" concept. Stream ingress, local
  projection, derived allocation receipt commit, and durable state mutation are
  distinct runtime acts.
- High-frequency streams are governed by typed commit strategies, not generic
  debounce helpers.
- The runtime, not a host callback or test harness, owns frame collection,
  deterministic frame close, stream-policy resolution, and transition dispatch.
  Host and Query-facing boundaries may submit admitted source facts only.
- Scroll-owned and portal-anchored behavior remain runtime-owned semantics.
  Host adapters report observations and consume receipts; they do not decide
  allocation truth.
- Incremental replanning is only valid when backed by typed invalidation
  families and mechanical affected-neighborhood proof.
- Allocation invalidation, neighborhood selection, and receipt recompute must
  consume existing UI authority-graph indexes, touched-graph consequences, and
  admitted Query projection-consumption lanes. 3.8 must not invent a second
  allocation-local dependency graph or consumer-owned proof table.
- 3.8 must consume the 3.7 cleaned facade/proof-flow shape and must not reopen
  broad root facades, dumping-ground modules, or certification support as
  production law.
- `worth-ui-query-binding` is the single UI/Query semantic edge. It may retain
  and narrow Query-owned authority into UI-specific admitted artifacts, but it
  may not rebuild Query meaning, identity, basis, lifecycle, or value families.
- A Query-enabled UI runtime must receive an installed, runtime-affine binding
  capability. Generic bootstrap without installed authority is not a valid
  Query-enabled production path.
- Consumer-authored strings, terminal reporting projections, debug formatting,
  and local hashes are never Query identity or admission authority.
- Enforcement must stay narrow and generative: dependency direction,
  constructor visibility, proof-carrying APIs, and a small number of sabotage
  tests. The plan must not grow a bureaucratic file-by-file permission list.

## Contract Appendix

This appendix freezes the minimum executable law that must exist before 3.8
implementation broadens.

### Post-9.13 Query Edge Law

The ordinary production direction is:

```text
owning Query workspace
-> runtime-installed Worth UI domain handle
-> worth-ui-query-binding admitted declaration/consumption artifacts
-> Worth UI runtime allocation and inspection consumers
```

The owning Query workspace installs `worth_ui_domain_package()` with
`worth_ui_native_aspect_contracts()` and mints the runtime-affine installed
handle. `worth-ui-query-binding` turns handle-bound declarations and consumed
projection outcomes into UI-specific proof-bearing artifacts. Worth UI runtime
consumes those artifacts; it does not independently interpret Query support,
basis, result shape, live posture, or identity.

The boundary is enforced by four load-bearing rules:

1. production `worth-ui-runtime` does not depend directly on `worth-query`;
2. Query-enabled runtime composition requires a binding artifact minted from
   the installed Worth UI domain handle;
3. no reporting projection, string, or consumer-local digest can satisfy an
   API requiring Query authority or equivalence identity;
4. certification may construct hostile Query worlds, but production and test
   support cannot mint admitted UI binding authority.

The enforcement source is the canonical workspace boundary configuration plus
compiler visibility and compile-fail fixtures. Generated agent context may
explain the edge, but no second handwritten allowlist may compete with the
machine contract.

### Agent Default Path And DX Target

The exact final names may change during implementation; this grammar may not:

```rust
let mut query_workspace = app_query_runtime()
    .aspect_contracts(worth_ui_native_aspect_contracts())?
    .domain_package(worth_ui_domain_package())?
    .workspace("worth-ui-app")?;

let installed_ui = query_workspace.domain(WorthUiDomainEntry)?;

let measurement_view = installed_ui
    .measurements()?
    .bind_ui_view(UiViewBindingIdentity::new("inspector.measurements")?)?;

let app = WorthUi::builder()
    .with_query_bindings(installed_ui.ui_bindings())
    .register_view_binding(measurement_view)
    .freeze()?;
```

The common path must express domain installation once, derive bindings from
that installed capability, and register one coherent binding artifact. It must
not require callers to separately assemble capability status, result-shape
digest, basis posture, live compatibility, or composition-profile digest.

Query-free Worth UI apps remain ceremony-free. They may use the ordinary
builder without Query setup. The moment a Query-bound declaration is
registered, however, freeze/launch must require the installed binding
capability through type progression or a typed admission result. A runtime
boolean or late panic is insufficient.

Advanced callers may inspect the binding plan, consumed projection contract,
invalidation footprint, and lifecycle posture before launch. That advanced
surface is observation over the same artifact, not a second construction path.

### Receipt Commit Grammar

Committed allocation receipt production is a runtime-owned state transition,
not a caller-owned formatting step.

Only the post-planning runtime seam may commit an allocation receipt. Ordinary
consumers, host adapters, services, and inspection code may inspect committed
receipts, but they may not mint or promote candidate allocation into committed
truth.

Required inputs:

- admitted `UiAllocationPlan`
- admitted `UiMeasurementBasis`
- admitted `UiAllocationNeighborhood` or admitted neighborhood set
- committed declaration/graph identity context
- declared coordinate/ownership posture

Forbidden inputs:

- raw host geometry state
- preview-only interaction state as if it were durable truth
- synthetic neighborhood artifacts built outside admitted neighborhood lanes
- caller-owned digest strings or helper-owned reuse verdicts

Receipt commit preconditions:

- plan, basis, and neighborhood-set generations must be mutually compatible
- coordinate/ownership posture must be admitted for the target operator family
- required invalidation-family and stream-policy resolution must already be
  complete

Receipt commit outputs:

- committed `UiAllocationReceipt`
- companion `UiAllocationReceiptReport`
- committed `UiAllocationReplanTransaction`
- typed reuse, partial-reuse, or denial evidence
- named counters for classification, neighborhood selection, reuse, widening,
  and commit cadence

Receipt commit denial seed set:

- stale measurement basis at commit
- stale plan generation at commit
- neighborhood generation mismatch
- unsupported coordinate/ownership posture
- impossible reuse contract
- illegal stream-policy combination
- forbidden root-collapse widening

Receipt commit laws:

- receipts are committed per admitted neighborhood
- neighborhood-set selection is carried by `UiAllocationReplanTransaction`, not
  by receipt identity itself
- commit is idempotent for replay of the same admitted plan generation, basis
  generation, neighborhood generation, and coordinate/ownership posture
- commit may deny; it is not a pure formatter over planning output
- committed does not mean current

Idempotence includes both:

- value idempotence: replay returns the same receipt identity and the same
  receipt/report outcome
- side-effect idempotence: replay does not append duplicate committed evidence,
  duplicate counter effects, or duplicate denial artifacts for the same
  admitted commit input set

### Preview Candidate Versus Committed Receipt Consumption

Preview is not a freshness posture on a committed receipt.

3.8 must distinguish two separate host-consumable geometry lanes:

- committed receipt consumption
  - host paint may consume a committed `UiAllocationReceipt` only when its
    companion `UiAllocationReceiptReport` posture is admitted for paint
- preview candidate consumption
  - host paint may consume an explicitly admitted preview-candidate artifact
    that is distinct from committed receipt truth

The following coercions are forbidden:

- candidate preview artifact -> committed receipt
- committed receipt -> preview candidate artifact
- preview-candidate consumption through APIs that require committed truth

Preview cadence may run at pointer or interaction frequency.
Committed receipt cadence remains governed by the admitted stream policy and may
not be smuggled through report mutation.

### Receipt Identity, Freshness, And Partial Reuse

`UiAllocationReceipt` identity is composed from admitted fields first:

- declaration/graph identity
- neighborhood identity
- measurement-basis generation
- coordinate/ownership posture
- equivalence-class identity

`UiAllocationReplanTransaction` identity is composed separately from:

- primary neighborhood identity
- ordered neighborhood-set membership
- widen reason ids
- policy-merge verdict
- transaction generation

Any digest is derived cache/support material only. A digest may not be the sole
identity authority.

Freshness does not live as an open-ended posture bag on the receipt itself.

- `UiAllocationReceipt` carries committed truth, identity, generation,
  equivalence basis, and committed outcome.
- `UiAllocationReceiptReport` carries freshness, lag posture, stream policy
  used, widening reasons, and bounded-staleness explanation.
- `UiAllocationReplanTransaction` carries ordered neighborhood membership,
  primary neighborhood, widen reasons, merge/composition verdict, and
  transaction-local boundedness evidence.

`equivalence-class identity` means the admitted operator-family-specific
allocation equivalence basis used for receipt reuse.

At minimum it must be derived from:

- declaration/operator family
- admitted coordinate/ownership posture
- neighborhood-local constraint shape that planning declares reuse-relevant
- any admitted operator-family reuse discriminator frozen by the planning lane

It may not be a caller-owned digest string or helper-owned synthetic category.

Freshness postures must include at least:

- `current`
- `coalescing`
- `stale_but_bounded`
- `recompute_pending`

Ordinary law:

- host paint may consume committed receipts whose companion report posture is
  `current`, `coalescing`, or `stale_but_bounded`
- later execution lowering may consume only receipts whose companion report is
  `current` or `stale_but_bounded` under an admitted policy that explicitly
  allows bounded lag; that allowance is carried by the report and cannot be
  minted or widened by the consumer
- `recompute_pending` is not an execution-lowering input

`recompute_pending` lives on append-only report lineage artifacts, not by
mutating committed receipt truth in place. A denial is an attempted-transition
outcome on transaction lineage, not a freshness posture of the prior committed
receipt.

If a commit attempt denies:

- no new committed receipt is produced
- a new denial-bearing report/evidence artifact is appended against the prior
  committed receipt lineage or transaction lineage, according to the denial
  family
- the prior committed receipt retains its prior paintability posture, or moves
  only to `stale_but_bounded` or `recompute_pending` through an independently
  admitted safety rule; denial alone may not revoke prior admitted truth

If recompute is pending:

- no preview candidate is silently upgraded
- the runtime appends a new report/evidence artifact describing the pending
  posture and the blocked next act

Freshness posture transitions must be explicit:

| Current Posture | Driving Event | Next Posture |
| --- | --- | --- |
| `current` | admitted coalescing policy accepts delayed recompute | `coalescing` |
| `current` | later basis/observation invalidates but bounded prior receipt remains paintable | `stale_but_bounded` |
| `current` | replacement/remeasure required before next committed receipt | `recompute_pending` |
| `coalescing` | admitted coalescing lag remains within its report-carried bound | `stale_but_bounded` |
| `coalescing` | bounded lag exceeded before recompute | `recompute_pending` |
| `stale_but_bounded` | bounded lag no longer admissible | `recompute_pending` |

A successful recompute commits a new receipt and a new `current` report. It
does not mutate a `coalescing`, `stale_but_bounded`, or `recompute_pending`
report back to `current`; those report-lineage artifacts remain immutable.

Partial reuse is admitted in one narrow form in 3.8:

- `structure_reuse_leaf_remeasure`

This means:

- declaration/graph identity is preserved
- neighborhood identity is preserved
- coordinate/ownership posture is preserved
- structure-level allocation topology is reusable
- one or more leaf/intrinsic measurement facts changed, so leaf remeasure is
  required before full committed receipt replacement

During `structure_reuse_leaf_remeasure`:

- the prior committed receipt remains the only committed receipt
- any preserved structure paintability must be expressed through committed
  receipt + report posture, not through silent partial commit
- leaf remeasure pending is represented through `recompute_pending` or
  `stale_but_bounded` according to the admitted freshness law for the family

All other partial-reuse shapes are denied in 3.8 unless later milestones admit
them explicitly.

### Replan Unit And Neighborhood-Set Law

The replan unit in 3.8 is an ordered set of admitted neighborhoods, not a
single implicit neighborhood.

Neighborhood selection must be derived from admitted UI graph topology,
touched-graph consequences, retained dependency indexes, and admitted
projection-consumption facts. It may not be rediscovered from recursive host
geometry walks, renderer-local membership maps, or allocation-local helper
graphs.

Every replan must classify:

- primary neighborhood
- widened neighborhood set, if any
- typed widen reason per added neighborhood
- whether root/page-wide widening was denied, counted, or admitted by explicit
  exceptional policy

Silent root collapse is forbidden.

Root semantics must distinguish:

- legitimate root primary neighborhood
  - root or shell scope is the admitted primary neighborhood for the change
- counted root widen
  - a non-root primary neighborhood widens to root/shell through an explicit
    typed widen reason and counter
- forbidden root fallback
  - implementation collapses to root because locality proof failed

Only the second case increments root-widen counters. The third case must deny.

Neighborhood-set counters must include at least:

- set cardinality
- widen reason histogram
- root-widen attempts
- neighborhoods reused
- neighborhoods replanned

Neighborhood-set order is deterministic and must be derived by:

1. primary neighborhood first
2. widened neighborhoods in typed widen-discovery order
3. stable identity ordering as the tie-breaker when multiple neighborhoods are
   discovered in the same widen class

That ordering participates in transaction evidence and replay determinism.

Neighborhood-set commit is atomic. Before mutating committed truth, the runtime
must preflight the complete ordered set against its expected generation vector,
policy verdict, reuse legality, coordinate posture, and locality proof. It then
either publishes every receipt, report, transaction artifact, counter effect,
and evidence reference as one outcome, or publishes no receipt and no partial
counter/evidence effect while appending one typed transaction denial.

`UiAllocationReplanTransaction` is the commit authority for a neighborhood set.
Its idempotency key includes runtime-instance generation, allocation frame
epoch, resolved policy verdict, ordered neighborhood identities, and expected
generation vector. Replay of the same key returns the prior complete outcome
without duplicate evidence or counters.

Disjoint transaction sets may commit independently. Overlapping sets must merge
before preflight or serialize by stable neighborhood-set order; they may never
partially interleave publication.

Widen reasons must come from a closed seed set in 3.8:

- `constraint_propagation_crossing`
- `shared_ancestor_requirement`
- `portal_layer_span`
- `scroll_owner_containment`
- `policy_merge_escalation`
- `viewport_shell_requirement`
- `measurement_basis_reach`

Additional widen reasons are denied until explicitly admitted.

### Stream Policy Composition Table

Each stream family is resolved by runtime-owned policy tables, not ad hoc app
code. Declarations may select from admitted policy families only where the
runtime says override is legal.

Default authority:

- runtime default policy table owns the family-to-policy mapping
- operator/declaration-level override is legal only where the runtime marks the
  family override-admissible
- numeric cadence budgets live on the runtime policy table, not in test-only
  fixtures

Each admitted policy family must declare at least:

- maximum committed receipt count per `N` ingress events
- maximum durable semantic-state mutations per `N` ingress events
- evidence cadence rule
- whether the family may enter `coalescing` or `stale_but_bounded`
- whether the family is override-admissible

`SourceLane`, `UiAllocationStreamFamily`, and
`UiAllocationInvalidationFamily` are distinct closed classifications. Every
sealed entry carries runtime-instance generation, source identity, source
generation, monotonic source sequence, source/evidence reference, and a
deduplication key. Within-source order is sequence order; cross-source ties use
the frozen source-lane rank and stable source identity. Duplicate and sequence
gap handling are typed policy inputs, not callback-order folklore.

Concurrent same-frame policy law:

- resize preview + durable resize commit: preview resolves first, durable
  mutation follows policy
- viewport extent + splitter drag: viewport extent does not inherit splitter
  durable policy by default
- typing + Query growth: semantic field commit and Query ingress may both occur,
  but allocation invalidation resolves through typed family merge and
  neighborhood-set selection
- portal anchor movement + scroll extent change: merge into typed
  multi-neighborhood replan selection, not silent root widening

Illegal combinations must deny at admit-time or runtime commit-time with typed
denials. They may not fall back to generic debounce or whole-page replanning.

The merge table is total and symmetric over admitted family pairs, with exact
numeric budgets for every admitted family declared in the table rather than
left to implementation selection. Its merge operator must prove associative,
commutative, and idempotent behavior; otherwise the runtime uses the defined
canonical left-fold over the sealed entry order and records every intermediate
verdict.

Every pair must map to exactly one of:

- merge to one composed family verdict
- remain distinct and co-select one transaction set
- typed denial

For frames containing three or more families, evaluation order is:

1. declaration change
2. Query fact change
3. host measurement replacement
4. viewport extent change
5. durable resize change
6. resize preview delta
7. scroll-owned extent change
8. portal-anchor movement
9. content growth/shrink

That order is the n-way merge law for deterministic replay.

Evidence cadence follows commit cadence, not raw ingress cadence.
Coalesced windows must emit one evidence artifact per committed window, plus
ingress sample counts through counters.

### Frame Dispatch And Ingress Lifecycle Law

An admitted stream frame is a runtime-owned transition boundary, not a bag that
host callbacks may fill and flush at will.

The ordinary path is:

`admitted source fact -> source-to-frame gateway -> open frame collector ->
sealed admitted stream frame -> policy resolution -> allocation transition ->
typed outcome/evidence envelope`

The runtime owns exactly one dispatcher lifecycle per active runtime instance.
It owns frame epoch allocation, collector replacement, deterministic frame
close, and handoff into the stream-resolved allocation transition. No host
adapter, renderer, gesture helper, Query callback, inspection surface, or test
fixture may close a frame, invoke policy resolution, or commit a receipt.

Source-to-frame gateways are narrow admission boundaries. They accept only
already admitted source truth:

- host measurement observations through the measurement lane
- settled Query projection facts through the binding-consumption lane
- admitted transient interaction state through the interaction lane
- admitted durable resize input through the durable-state lane

They do not decide invalidation family, policy, locality, receipt reuse, or
commit eligibility. Those decisions begin only after frame sealing.

Frame lifecycle is an explicit state machine:

`Paused -> Open(epoch) -> Closing(epoch, next_epoch_reserved) ->
Sealed(epoch) -> Dispatched(epoch)`

`Paused` is entered only during launch, replacement pause, shutdown, or a
typed fatal dispatcher denial. `Dispatched(epoch)` opens only the next reserved
epoch; no state may reopen or mutate a sealed epoch.

The runtime executor is the sole linearization authority. In 3.8 it is an
explicitly single-threaded runtime pump over a bounded runtime-owned mailbox;
host and Query callbacks submit capability-shaped messages but never borrow or
mutate the collector. A host paint/tick boundary may be an admitted boundary
fact, but it does not allocate the epoch or close the frame. The runtime pump
closes an epoch through its one named dispatch act.

Submission outcomes are typed and carry the accepted epoch plus assigned
sequence, duplicate identity, backpressure watermark/retry epoch, or terminal
denial. Every accepted ingress belongs to exactly one epoch. Ingress observed
after `Closing` begins is assigned to the next reserved epoch or returns typed
backpressure/denial; it is never folded into the closing frame silently.

Transport capacity is a Phase 3 mailbox/collector law, distinct from later
semantic per-family cadence policy. Overflow may backpressure or deny with
source identity and counters, but may not silently discard source truth.
Duplicate/retry handling is idempotent: the same ingress key has one accepted
sequence and one counter/evidence effect.

Dispatcher lifetime is tied to runtime-instance generation. Launch creates its
initial `Open` epoch; replacement pause seals or explicitly disposes queued
ingress through typed outcomes; shutdown denies new ingress and drains or
disposes the bounded queue by declared policy; epoch exhaustion is terminal.
`UiAllocationFrameEpoch` must either replace `WorthUiRuntimeFrameEpoch` or
carry an explicit causal bridge to it; two unrelated epoch authorities are
forbidden.

Only `Dispatched(epoch)` returns one `UiAllocationFrameTransitionOutcome`
carrying the resolved frame plan, later transition result, receipt/report or
denial lineage, and boundary counters. Consumers do not reconstruct an outcome
from collector or host state.

The dispatcher may batch only across a semantically honest frame boundary. It
must expose ingress count, frame count, late-ingress count, queue/overflow
denials, family cardinality, resolved transaction-set cardinality, receipt
count, and evidence count so frame-cost claims remain mechanically auditable.

### Named Step Surface

3.8 must ship named typed steps, even when some remain internal runtime seams
rather than public facade methods:

- `classify_stream_event(...)`
- `classify_allocation_invalidation(...)`
- `select_replan_neighborhoods(...)`
- `evaluate_allocation_receipt_reuse(...)`
- `commit_allocation_receipt(...)`
- `inspect_allocation_receipt(...)`

These names may evolve, but the step boundaries may not collapse into one giant
replan function.

Each step must own a typed artifact boundary:

- `classify_stream_event(...)` -> `UiStreamClassification`
- `classify_allocation_invalidation(...)` -> `UiAllocationInvalidation`
- `select_replan_neighborhoods(...)` -> `UiAllocationReplanTransaction`
- `evaluate_allocation_receipt_reuse(...)` -> `UiAllocationReuseDecision`
- `commit_allocation_receipt(...)` -> `UiAllocationReceipt` or
  `UiAllocationCommitDenial`
- `inspect_allocation_receipt(...)` -> `UiAllocationInspectionArtifact`

### Generation, Concurrency, And Supersession Law

3.8 must define generation compatibility as a real runtime contract.

At minimum:

- measurement basis owns a basis generation
- each admitted neighborhood owns a neighborhood generation
- each candidate plan owns a plan generation tied to its basis + neighborhood
- each replan transaction owns a transaction generation

Commit legality is compare-and-swap against the latest admitted generation set
for the target neighborhood.

If stream ingress lands mid-plan:

- the in-flight plan is compared against basis and neighborhood generation at
  commit time
- stale generation mismatch denies committed replacement for that receipt
  attempt
- the newer ingress is classified as a later invalidation/replan act, not
  folded silently into the stale plan

If overlapping neighborhood transactions exist in the same frame:

- they must be merged before commit when overlap is admitted and mergeable
- otherwise later-generation supersession must deny the stale overlapping
  commit attempt with typed evidence

Replay determinism depends on this law and may not invent ordering locally.

### Query Settlement To Allocation Posture Law

Query-backed allocation facts must map through admitted settlement states.

At minimum:

- settled projection consumption -> eligible for ordinary invalidation and
  commit
- partial projection settlement -> `stale_but_bounded`,
  `recompute_pending`, or typed denial according to the admitted stream policy
- failed or unsupported projection settlement -> typed denial

Query-backed layout facts may be allocation-relevant only when the binding lane
admits them as such. Typical admitted categories include:

- collection cardinality
- ordering that changes visible neighborhood membership
- intrinsic-size-relevant projected fields

All other Query changes are not implicitly allocation-invalidating.

### Geometry Evidence Minimum Schema

Allocation geometry evidence must freeze a minimum schema in 3.8.

At minimum, ordinary allocation inspection must be able to return:

- axis-aligned bounds in admitted coordinate space
- parent edge refs as `(target_id, axis, delta)`
- sibling edge refs as `(target_id, axis, delta)`
- anchor posture enum plus anchor target id when present
- spacing/group relationship ids when allocation established them
- `not_known_at_allocation` markers for baseline/alignment or perceptual facts
  the allocation lane did not establish

This keeps later AI critique honest about runtime-semantic versus
paint/perceptual knowledge.

### Portal Identity Rule

Portal anchor identity must be explicit in 3.8.

At minimum:

- rect-value changes with the same anchor target id and coordinate-space
  posture preserve anchor identity
- anchor target id change replaces anchor identity
- coordinate-space posture change replaces anchor identity
- stale or generation-mismatched anchor evidence denies reuse before any
  committed replacement receipt is produced

Phase 11 tests asserting anchor identity must certify against this rule.

### Anti-Bypass Type Fence Law

Anti-bypass is part of the appendix contract, not only Phase 22 certification.

3.8 must enforce at least:

- committed receipt constructors sealed or crate-private to the commit lane
- invalidation family constructors closed to admitted runtime lanes
- preview-candidate artifacts distinct from committed receipt artifacts at the
  type level
- test-support builders unable to mint committed receipts directly
- certification and inspection crates unable to bypass the post-planning commit
  seam

Zero candidate-to-committed coercion and zero untyped invalidation paths must
be guaranteed structurally, not only asserted narratively.

### Virtualization And Content-Totality Default

3.8 does not admit virtualization or offset-sensitive allocation as ordinary
behavior.

For 3.8:

- allocation is defined over bounded admitted content neighborhoods
- scroll offset is projected interaction only unless a later admitted operator
  family reopens it
- windowed/virtualized allocation is deferred beyond 3.8 and must not be
  smuggled into scroll semantics or hostile certification assumptions

### Complexity And Evidence-Cadence Envelope

3.8 must declare boundedness in work-shape terms, not only prose.

At minimum:

- stream classification is `O(1)` per admitted ingress event
- invalidation-family classification is `O(1)` per admitted classified event
- neighborhood selection is bounded by retained locality indexes and may not
  degrade to recursive whole-graph scans
- reuse evaluation is bounded by receipt identity/equivalence fields plus
  admitted neighborhood-local evidence
- plan execution cost must be expressible as work over the selected
  neighborhood set, not rediscovery over the full UI graph

Evidence cadence follows commit cadence:

- pointer-rate ingress may increment counters every sample
- coalesced or thresholded commit policies emit one committed evidence set per
  committed window or terminal act
- replay and certification must be able to prove both ingress counts and
  committed evidence counts without allocating one evidence artifact per raw
  pointer delta

### Allocation Inspection And Visual-Critique Readiness

3.8 is not the milestone that ships the full visual-evaluation product
surface, but it must preserve the runtime evidence shape that makes later AI
critique honest.

Allocation inspection in 3.8 must therefore support both:

- diagnostic why-questions about committed allocation truth
- critique-readiness about visible geometry relationships that allocation
  establishes

Ordinary local inspection should be able to answer at least:

- why this allocation receipt exists
- which invalidation family or stream family produced it
- why reuse, partial reuse, or denial occurred
- which neighborhood or neighborhood set was selected
- what coordinate and ownership posture was committed
- which visible geometry facts were established that later visual evaluation
  may consume

The receipt/report/evidence lane must preserve inspectable geometry-facing
facts such as:

- allocated bounds
- anchor and coordinate-space posture
- sibling and parent-relative edges
- group-local sizing and spacing relationships
- baseline- or alignment-relevant structure when known at allocation time
- whether a visible relationship is runtime-semantic allocation truth versus a
  later paint/perceptual concern

3.8 must not require later AI critique tooling to reconstruct allocation
meaning from screenshots, host-local rectangles, or logs when receipt-backed
geometry truth already exists.

3.8 should also reserve allocation-facing evidence homes for later visual
evaluation consumers, for example:

- `UiAllocationInspectionArtifact`
- `UiAllocationNeighborhoodSelectionReport`
- `UiAllocationReuseDecision`
- `UiAllocationFreshnessReport`
- `UiAllocationCommitDenial`
- `UiAllocationCounterReport`
- allocation-linked geometry evidence that later visual-evaluation milestones
  can join to alignment, spacing, symmetry, and overlay artifacts

The law is:

- 3.8 closes committed allocation truth
- later visual-evaluation milestones may add stronger critique artifacts
- but they must not need to invent a second geometry truth because 3.8 failed
  to preserve the first one

### Certification Oracles

3.8 certification must prove at least:

- zero untyped invalidation paths
- zero candidate-to-committed coercion paths
- root-widen attempts are counted and typed
- maximum neighborhood-set cardinality is asserted per hostile scenario
- maximum committed receipt count is asserted per named stream-policy scenario
- partial reuse is either `structure_reuse_leaf_remeasure` or typed denial
- ordinary receipt inspection can explain local why/what/where without
  requiring causal inspection escalation
- allocation evidence preserves enough receipt-backed geometry truth that later
  AI critique can evaluate alignment/spacing/symmetry from runtime semantics
  instead of screenshot-only reconstruction

## Final Structural Homes

3.8 is cross-cutting enough that its final authority homes are recorded here.
This is the implemented topology, with unchanged sibling surfaces elided.

```text
workspaces/worth-ui/crates/
  worth-ui-query-binding/
    src/
      installed_domain/
        capability.rs
        rebind.rs
        workspace.rs
      declaration/
        binding_contract_identity.rs
        definition.rs
        identity.rs
        installed_view.rs
      consumption/
        installed_projection.rs
      prerequisites/
        allocation/query_allocation_invalidation_basis.rs
        basis/query_authority_handle.rs
        basis/query_basis_authority.rs
        basis/query_lane.rs
        measurement/query_measurement_fact_receipt.rs
        measurement/query_measurement_fact_settlement.rs
      domain_marker.rs
      domain_package.rs
      installed_measurements.rs
      native_aspect_contracts.rs

  worth-ui-runtime/
    src/
      graph/
        allocation_neighborhood/
          mod.rs
          admission/
            admitted_constraint_basis.rs
            catalog_basis.rs
            operator_contract.rs
          membership/
            equivalence.rs
            membership.rs
            membership_rule.rs
          replan_selection/
            consequences.rs
            neighborhood_selection.rs
            portal_consequence.rs
            scroll_consequence.rs
            selector.rs
          constraint_authority/
            ...admitted derivation and integration surfaces...

      runtime/
        allocation_frame_dispatch/
          ...frame lifecycle and capability-shaped gateways...
        invalidation_narrowing/
          ...typed classification, graph narrowing, and authority...
        stream_policy/
          ...cadence, ordering, composition, and closed families...

        allocation_receipt/
          mod.rs
          committed_truth/
            committed_allocation.rs
            committed_evidence.rs
            committed_lowering_input.rs
            committed_receipt.rs
            geometry_evidence.rs
          ledger_lifecycle/
            ledger_state.rs
            receipt_ledger.rs
            receipt_ledger_entry.rs
          report_freshness/
            allocation_counters.rs
            consumer_admission.rs
            receipt_generation.rs
            receipt_identity.rs
            receipt_report.rs
          reuse/
            preview_candidate.rs
            preview_isolation.rs
            receipt_budget.rs
            reuse_verdict.rs
          transaction/
            denial_taxonomy.rs
            receipt_commit.rs
            replan_transaction.rs
            transaction_outcome.rs

      evidence/
        planning/
          allocation_geometry_inspection.rs
          denied_replan_inspection.rs
          receipt_inspection.rs

  worth-ui-inspection/
    src/
      allocation/
        mod.rs
        attempt_projection.rs
        evidence_ref.rs
        geometry_projection.rs
        receipt_projection.rs

  worth-ui-certification/
    src/
      allocation/
        mod.rs
        allocation_closeout_certification.rs
      topology/
        allocation_closeout_anti_bypass_audit.rs
```

### Directory Laws

- `graph/allocation_neighborhood/` owns neighborhood identity, ordered-set
  shape, widen reasons, and locality proof only.
- `runtime/invalidation_narrowing/` owns typed family classification and
  admitted graph-target narrowing only.
- `runtime/allocation_frame_dispatch/` owns runtime lifecycle, transport
  capacity, epoch/sequence allocation, immutable sealing, and one-shot dispatch
  only; its `gateway/` children own capability-shaped source submission only.
- `runtime/stream_policy/` owns cadence, ordering, and merge
  legality only; it returns a resolved frame plan and never closes frames or
  commits receipts.
- `runtime/allocation_receipt/committed_truth/` owns committed receipt truth.
- `runtime/allocation_receipt/report_freshness/` owns freshness admission and
  allocation counter reports.
- `runtime/allocation_receipt/transaction/` owns atomic transaction and typed
  denial truth.
- `evidence/planning/` projects runtime-owned receipt evidence into inspection
  types; it does not own operational authority.
- `worth-ui-inspection/src/allocation/` projects evidence; it does not mint it.
- `worth-ui-certification/src/allocation/` proves behavior; it does not own it.
- `worth-ui-test-support/src/allocation/` builds fixtures; it does not become a
  production API lane.

### Phase Mapping

- Phase 1 and Phase 2 land primarily in `runtime/allocation_receipt/` and
  `evidence/planning/`
- Phase 3 lands primarily in `runtime/allocation_frame_dispatch/`
- Phase 4 lands primarily in `runtime/allocation_frame_dispatch/gateway/` and
  admitted host/Query/interaction boundary ports
- Phase 5 lands primarily in `runtime/stream_policy/` and
  `runtime/invalidation_narrowing/` as a resolved frame plan with typed
  invalidation artifacts only
- Phase 6 lands primarily in `runtime/invalidation_narrowing/` and admitted
  graph/Query narrowing surfaces
- Phase 7 lands primarily in `graph/allocation_neighborhood/`, planning, and
  the transaction-owned atomic allocation transition seam
- Phases 8 through 11 land in the typed viewport, drag-resize, scroll-owned,
  and portal consequences below `runtime/invalidation_narrowing/`, with graph
  locality owned by `graph/allocation_neighborhood/`.
- Phase 12 lands in workspace boundary configuration, crate manifests,
  `worth-ui-query-binding`, and Query-edge certification
- Phase 13 lands in `worth-ui-query-binding/installed_domain/` plus Worth UI
  builder, launch, replacement, and shutdown lifecycle propagation
- Phase 14 lands in `worth-ui-query-binding/declaration/` plus the public
  capability-registration facade
- Phase 15 lands in `worth-ui-query-binding/consumption/` plus the allocation
  frame Query gateway and measurement-fact consumers
- Phase 16 deletes consumer-local Query mirrors and reconstructed identity
  digests from runtime capability, source, replacement, and admission lanes
- Phase 17 lands in the Worth UI public facade and
  `workspaces/worth-ui/docs/worth-ui-readme.md`
- Phase 18 restructures oversized allocation responsibility neighborhoods and
  installs topology, naming, residue, and agent-context enforcement
- Phase 19 lands primarily in `worth-ui-inspection/src/allocation/`
- Phase 20 lands primarily in
  `runtime/allocation_receipt/report_freshness/` and the execution-consumer
  admission boundary
- Phase 21 lands primarily in
  `runtime/allocation_receipt/report_freshness/allocation_counters.rs`,
  `runtime/allocation_receipt/transaction/denial_taxonomy.rs`, and the receipt
  inspection projection
- Phase 22 lands primarily in `worth-ui-certification/src/allocation/`

### Anti-Sprawl Rules

- Do not create `src/allocation/` as an omnibus folder under `worth-ui-runtime`.
- Do not create `allocation_helpers.rs`, `allocation_support.rs`, or
  `allocation_utils.rs` as overflow bins.
- Do not put receipt truth in `allocation_planning/`.
- Do not put locality proof in host-observation files.
- Do not put production logic in inspection or certification modules.
- Do not let `mod.rs`, `lib.rs`, or facade files carry classification, reuse,
  freshness, or counter logic.
- Do not replace direct Query imports with binding-crate re-exports of the same
  raw Query topology. The binding crate must publish UI-specific admitted
  capabilities and outcomes.
- Do not create `query_reference`, `query_support`, or `query_context` as bags
  for locally mirrored Query state. Name the declaration, consumption,
  lifecycle, or inspection responsibility instead.
- Do not retain `helpers.rs`, `types.rs`, `model.rs`, or same-level directories
  with more than ten production files when the contained authority or lifecycle
  axes can be named honestly.

## Phase Plan

### Phase 1: Allocation Truth Boundary

This phase freezes the ontology for allocation work before any receipt or churn
code lands.

3.8 must distinguish at least these categories:

- `ephemeral stream event`
- `local projected interaction state`
- `candidate allocation result`
- `preview allocation posture`
- `committed allocation receipt`
- `durable semantic state`

That is the minimum separation needed to keep typing, drag, resize, scroll,
and portal churn from collapsing into one false "layout changed" lane.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/replacement/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/measurement/`

**Relevant APIs**
- `runtime.plan_allocation(...)`
- `UiMeasurementBasis`
- `UiAllocationNeighborhood`
- `UiMeasurementResult`
- `UiHostObservation`
- Worth UI inspection facade

**Warnings**
- Do not let `plan_allocation(...)` output masquerade as already-committed
  allocation truth.
- Do not let host-facing geometry observations become committed allocation
  without runtime receipt commitment.
- Do not blur durable UI state mutation with allocation receipt recompute.

**Test requirements**
- Adversarial equivalence test: the same `UiMeasurementBasis` and
  `UiAllocationNeighborhood` produce the same candidate allocation result
  regardless of prior preview-only interaction churn, with identical candidate
  identity, neighborhood identity, and committed-ineligible posture.
- Adversarial rejection test: preview drag/IME/scroll state cannot be consumed
  by APIs that require committed allocation truth, and the failure must occur
  through typed API rejection or compile-time boundary denial rather than
  silent coercion.

**Engineering decisions**
- Introduce explicit type/category separation between candidate allocation and
  committed allocation receipt artifacts.
- Treat preview allocation as candidate allocation under an explicit preview
  posture, not as a silently separate truth lane.
- Keep receipt commitment downstream of admitted planning, not inside host
  translation or gesture-local code.
- Preserve 3.7 proof-flow ordering:
  `measurement basis admit -> neighborhood admit -> plan_allocation -> 3.8 receipt commit`.

**Open questions**
- Resolved in Contract Appendix:
  `Preview Candidate Versus Committed Receipt Consumption`,
  `Anti-Bypass Type Fence Law`.

### Phase 2: Allocation Receipt Identity, Generation, And Reuse Contract

This phase freezes what `UiAllocationReceipt` is, which fields carry identity,
and which fields are explanatory only.

The receipt must answer:

- which declaration/graph identity it belongs to
- which admitted neighborhood and measurement basis generation it consumed
- what coordinate/ownership semantics it committed
- what equivalence basis justifies reuse
- whether a result is full reuse, `structure_reuse_leaf_remeasure`, or denial
- what explanatory evidence shows why reuse was admitted or denied

It must not be a host-shaped geometry dump.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/measurement/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/launch/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/replacement/`

**Relevant APIs**
- `UiAllocationPlan`
- `UiAllocationReceipt`
- `UiMeasurementBasis`
- `WorthUiQueryPrerequisiteEvidence`
- `WorthUiQueryAuthorityHandle`
- binding-owned consumed-projection settlement

**Warnings**
- Receipt identity that is too coarse will over-invalidate and destroy reuse.
- Receipt identity that is too host-shaped or too fine will make reuse brittle
  and cross-frame reasoning unstable.
- Reuse predicates hidden in helper code are forbidden; reuse must be backed by
  an explicit equivalence contract.

**Test requirements**
- Adversarial equivalence test: receipt reuse succeeds only when identity,
  generation, ordering, and admitted equivalence basis match exactly, with
  receipt identity and reuse verdict identical across replay.
- Adversarial partial-reuse test: preserved structure with changed leaf
  intrinsic evidence admits only `structure_reuse_leaf_remeasure` and not full
  receipt reuse, and inspection must expose the exact preserved structure ids
  and exact leaf identities requiring remeasure.
- Adversarial denial test: mismatched neighborhood generation, basis lineage,
  or coordinate ownership denies reuse with typed denial rather than silent
  fallback, and no committed replacement receipt may be produced.

**Engineering decisions**
- Split receipt facts into identity-bearing fields and explanatory fields.
- Compose receipt identity from admitted fields first; keep any digest as a
  derived cache key only.
- Carry measurement-basis generation and neighborhood identity forward so later
  rebind and execution work can reason from receipts without re-querying
  planning internals.
- Require reuse and denial evidence to be inspectable on the receipt-first
  lane before broader cross-runtime explanation.

**Open questions**
- Resolved in Contract Appendix:
  `Receipt Commit Grammar`,
  `Receipt Identity, Freshness, And Partial Reuse`,
  `Generation, Concurrency, And Supersession Law`.

### Phase 3: Runtime Frame Dispatch Ownership

This phase gives continuous allocation work a sealed dispatcher lifecycle
before source gateways, framework scheduling, or stream policy are allowed to
claim effect.

The dispatcher owns the open-frame collector, epoch progression, deterministic
internal close transition, immutable frame sealing, and a move-only handoff
artifact for the later stream transition. It is the only owner permitted to
turn already-admitted ingress into a sealed frame. Phase 3 establishes that
authority; it does not yet wire a real host event loop, source gateway, or
stream-policy consumer to it.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/launch/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_frame_dispatch/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/`

**Relevant APIs**
- `UiAllocationFrameEpoch`
- `UiAllocationFrameDispatcher`
- `UiAdmittedAllocationStreamFrame`
- `UiAllocationFrameTransitionOutcome`
- dispatcher-owned sealed-frame handoff vocabulary

**Warnings**
- Do not expose collector flush or policy resolution as a host/renderer helper.
- Phase 3 must not invent a proxy framework loop, public close capability, or
  synthetic production caller merely to prove later Phase 4 integration.
- Phase 3 must not self-acknowledge or self-consume its handoff merely to
  simulate later Phase 5 stream-policy consumption.
- Do not conflate a sealed frame with its later policy decision or allocation
  outcome.

**Test requirements**
- Adversarial replay test: the same already-admitted ingress set, submitted
  through the dispatcher test-support seam in every permitted arrival order
  within one epoch, seals to the same ordered frame and produces the same
  dispatch identity and counters.
- Adversarial lifecycle test: duplicate close and reentrant dispatch are typed
  denials, and a sealed epoch cannot be dispatched twice or mutated after close.
- Adversarial late-ingress test: ingress racing frame close is assigned to the
  next epoch or denied by declared capacity law, never silently merged into the
  closing outcome.
- Adversarial boundary test: no non-test source gateway, host event-loop owner,
  or stream-policy consumer is claimed or fabricated in this phase; the
  dispatcher exposes only the narrow sealed-frame handoff required by Phases 4
  and 5.

**Engineering decisions**
- Add a dedicated `allocation_frame_dispatch` responsibility rather than
  placing lifecycle state in stream policy, host observation, or launch facade.
- Keep the dispatcher as the source of the one ordinary frame-transition
  artifact; collectors and policy internals remain non-authoritative support.
- Use a bounded, runtime-owned collector footprint with explicit epoch and
  ingress counters.
- Phase 4 owns production ingress plus the concrete framework-loop scheduling
  seam. Phase 5 owns mandatory sealed-frame consumption and acknowledgment.

**Open questions**
- Resolved in Contract Appendix: `Frame Dispatch And Ingress Lifecycle Law`.

### Phase 4: Source-To-Frame Admission Gateways

This phase connects existing admitted source lanes and the concrete framework
turn to the runtime dispatcher without promoting host mechanics or Query
callbacks into allocation owners.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_frame_dispatch/gateway/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/launch/`
- `workspaces/worth-ui/crates/worth-ui-query-binding/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/`

**Relevant APIs**
- `UiHostObservation`
- `UiMeasurementBasis`
- `UiProjectionFactReceipt`
- `WorthUiTransientInteractionState`
- `WorthUiAdmittedDurableResizeInput`
- `runtime.submit_admitted_allocation_ingress(...)`
- framework-owned allocation-frame turn scheduler

**Warnings**
- A gateway may classify only the source-backed ingress family it is admitted
  to submit; it may not select invalidation, policy, receipt reuse, or commit.
- Query callbacks must submit settled projection facts through the binding lane,
  never payload caches or locally reconstructed Query state.
- Do not make one generic event API that accepts raw host geometry, arbitrary
  source labels, or caller-owned frame identifiers.
- The framework-loop owner may close and dispatch a turn, but it must hand the
  sealed result forward unchanged; Phase 4 must not classify policy or
  acknowledge downstream consumption on its own.

**Test requirements**
- Adversarial anti-bypass test: host, renderer, gesture, Query, and fixture
  callers cannot close a frame, invoke policy resolution, or commit a receipt;
  only their narrow gateway submission is available.
- Adversarial source-settlement test: unsupported Query settlement and stale
  host measurement admission deny before they enter a frame; partial Query
  settlement enters only as an explicitly typed partial-settlement fact for
  later policy resolution, with exact gateway evidence retained.
- Adversarial boundedness test: a pointer-rate interaction stream and a burst
  of Query facts remain bounded by per-family ingress capacity without losing
  the declared source-order or silently widening work.
- Adversarial ownership test: a concrete non-test framework-turn owner, not a
  callback or host facade, exclusively invokes the dispatcher close/pump seam;
  unsupported absence of a downstream Phase 5 consumer is a typed handoff
  backpressure result rather than a self-acknowledged success.

**Engineering decisions**
- Gateways consume already admitted source truth and append to the dispatcher;
  they do not recreate source admission or dependency graphs.
- Each gateway preserves source generation and ordering posture into the sealed
  frame so later policy and transaction work never infer it from callback order.
- Make raw collector mutation crate-private and expose only capability-shaped
  submission ports from the runtime facade.
- Move all ordinary nonempty-frame proof off test-only ingress construction and
  through the production gateways and framework-owned turn seam.

**Open questions**
- Resolved in Contract Appendix: `Frame Dispatch And Ingress Lifecycle Law`,
  `Query Settlement To Allocation Posture Law`.

### Phase 5: Stream Classification And Typed Invalidation Plan

This phase freezes the semantic contract for continuous streams, installs the
mandatory consumer of the Phase 4 sealed-frame handoff, and lowers each sealed
dispatcher-owned frame into a resolved frame plan. It classifies source
and stream families and produces typed invalidation artifacts, but it does not
select neighborhoods, plan allocation, or commit receipts.

3.8 must not force one generic "commit" meaning across:

- pointer drag streams
- resize streams
- scroll streams
- text/IME streams
- streaming Query/data updates
- host measurement churn

Each stream family must declare:

- ordering semantics
- collapse/coalesce policy
- latest-wins legality or illegality
- preview/derived/durable commit targets
- freshness posture and bounded lag
- concurrent same-frame merge/composition law

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_frame_dispatch/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/activation/`
- `workspaces/worth-ui/crates/worth-ui-query-binding/`

**Relevant APIs**
- `UiAdmittedAllocationStreamFrame`
- `UiResolvedAllocationFramePlan`
- `UiAllocationInvalidation`
- `runtime.resolve_allocation_frame(...)`
- `UiHostObservation`
- `UiMeasurementRequest::viewport_extent(...)`
- `UiMeasurementRequest::portal_anchor_rect(...)`
- `UiMeasurementRequest::scroll_container_viewport(...)`
- `WorthUiQueryMeasurementFactSettlement`
- Worth UI inspection facade

**Warnings**
- Generic debounce is not an acceptable semantic contract.
- "We cannot afford to recompute every event" must not become "we lost source
  truth."
- Per-character semantic commit is legal for some fields and illegal as a
  global allocation commit default.
- Do not leave policy resolution as a pull-only helper; the sealed dispatcher
  frame is its mandatory ordinary input, it must acknowledge consumption only
  after accepting that exact move-only handoff, and the resolved frame plan is
  its only output in this phase.

**Test requirements**
- Adversarial cadence test: per-character typing may commit semantic field
  value each event while the resolved allocation frame plan remains
  threshold-bounded, with declared input, frame-plan, and later receipt budget
  contracts for `N` semantic input events under each admitted policy family.
- Adversarial stream-policy test: latest-wins, coalesced-window, and
  terminal-commit policies reject illegal collapse of stream families whose
  intermediate states matter semantically, with typed denial of the exact
  illegal policy combination.
- Adversarial multi-stream test: same-frame typing + Query growth + resize
  preview resolves through the policy composition table into typed
  invalidation artifacts and a deterministic frame plan instead of a hidden
  priority hack.

**Engineering decisions**
- Introduce explicit stream policy families such as immediate latest-wins,
  coalesced window, threshold-triggered, terminal commit, and priority-split
  commit.
- Represent freshness/posture on a companion `UiAllocationReceiptReport`
  rather than lying that committed always means current.
- Keep stream ingress truth separate from downstream allocation receipt
  commitment.
- Resolve concurrent same-frame stream families through one runtime-owned merge
  table rather than per-feature policy code.
- Return the resolved policy, typed invalidation set, preview legality, and
  cadence/evidence verdict together as one immutable resolved frame plan.

**Open questions**
- Resolved in Contract Appendix:
  `Preview Candidate Versus Committed Receipt Consumption`,
  `Stream Policy Composition Table`,
  `Complexity And Evidence-Cadence Envelope`.

### Phase 6: Invalidation Family Closure And Graph Narrowing

This phase closes the invalidation vocabulary and turns the Phase 5 resolved
frame plan into graph- and Query-backed invalidation facts without selecting a
neighborhood set or committing allocation.

Phase 6 freezes the invalidation family skeleton and typed evidence shape.
Phases 8 through 11 finalize domain-specific semantics for resize, scroll, and
portal families without reopening the existence of the family itself.

At minimum, 3.8 needs typed invalidation families for:

- viewport extent change
- durable local resize change
- resize preview delta
- content growth/shrink
- scroll-owned extent change
- portal-anchor movement
- Query-backed measurement fact change
- host measurement result replacement

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/measurement_invalidation.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/graph/allocation_neighborhood/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/replacement/`
- `workspaces/worth-ui/crates/worth-ui-query-binding/`

**Relevant APIs**
- `UiHostObservation`
- `UiMeasurementResult`
- `UiMeasurementBasis`
- binding-owned consumed-projection settlement
- `workspace.inspections().inspect(...)`

**Warnings**
- Family closure must happen before incremental replanning code lands, not
  after.
- Query-backed layout facts must enter through admitted projection consumption,
  not through local caches or host-local payload copies.
- Host observations may trigger invalidation, but may not redefine the
  invalidation taxonomy.
- Host observations may target existing graph nodes, neighborhoods, and
  receipts, but they may not create new semantic dependency edges, touched
  graph membership, or consumer-owned authority lanes on their own.

**Test requirements**
- Adversarial classification test: every supported host/Query allocation input
  maps to one typed invalidation family with no generic fallback bucket, and
  enum or artifact closure tests prove no untyped family can be constructed.
- Adversarial denial test: unsupported or ambiguous invalidation origins
  produce typed denial artifacts instead of broad replan fallback, and no root
  or whole-page replan may occur in the denied path.

**Engineering decisions**
- Model invalidation families as explicit runtime artifacts or enums with
  family-native evidence, not booleans.
- Tie each invalidation family to its allowed stream policies and affected
  neighborhood selection grammar.
- Preserve origin distinctions across declaration change, Query fact change,
  host observation, and interaction preview paths.
- Query-backed fact changes invalidate allocation only after admitted
  projection-consumption settlement; partial projection posture may yield typed
  stale-basis or deferred-allocation denial rather than silent recompute.
- Require allocation invalidation to consume existing authority-graph indexes,
  touched-graph consequences, and admitted projection-consumption evidence
  rather than introducing a parallel allocation-specific dependency engine.

**Open questions**
- Resolved in Contract Appendix:
  `Query Settlement To Allocation Posture Law`,
  `Anti-Bypass Type Fence Law`,
  `Replan Unit And Neighborhood-Set Law`.

### Phase 7: Affected-Neighborhood Replan Selection

This phase turns "only replan the affected neighborhood" into a mechanical
runtime proof and consumes the resolved frame plan plus closed invalidation
facts to perform the first ordinary allocation transition.

3.8 must define:

- the smallest legal incremental replan unit
- how invalidation families narrow to that unit
- when a neighborhood may be reused
- when a neighborhood must widen
- how widening is explained and counted
- how multi-neighborhood replan sets are ordered and certified
- how complete-set preflight, planning, receipt commitment, and transaction
  publication remain atomic

This phase must build directly on 3.6b neighborhood admission and 3.7
replacement/impact narrowing instead of inventing a second selection path.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/graph/allocation_neighborhood/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/replacement/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/launch/`

**Relevant APIs**
- `UiAllocationNeighborhood`
- `UiMeasurementBasis`
- `runtime.plan_allocation(...)`
- `UiResolvedAllocationFramePlan`
- `UiAllocationFrameTransitionOutcome`
- identity-match report surfaces from replacement
- Worth UI inspection facade

**Warnings**
- Replan selection must not silently widen to page/root scope because the
  implementation cannot classify a case.
- The neighborhood basis used for receipt reuse and for replan widening must be
  inspectable and identical across certification and production surfaces.
- Do not recompute locality from host geometry when admitted neighborhood and
  impact-narrowing artifacts already exist.
- Do not derive affected neighborhoods from consumer-owned recursive scans,
  renderer-local tree position, or helper-owned dependency maps when the UI
  authority graph, touched-graph consequences, and retained indexes already
  own the locality proof.
- Do not commit one neighborhood from a selected set before every member has
  passed generation, policy, reuse, and locality preflight.

**Test requirements**
- Adversarial locality test: local content growth or splitter movement replans
  only the admitted affected neighborhood while unrelated sibling and shell
  regions remain untouched, with asserted affected neighborhood ids, asserted
  unaffected neighborhood ids, and exact expected neighborhood-set cardinality.
- Adversarial widening test: when classification proves the neighborhood is
  insufficient, the runtime widens through a typed reason and counter rather
  than by hidden fallback, with exact widen reason ids and exact widen count
  asserted for the scenario.
- Adversarial set test: portal-anchor movement plus scroll-owned extent change
  yields a typed neighborhood set with counted widen reasons instead of silent
  root collapse, with an asserted primary neighborhood id, ordered
  neighborhood-set membership, and zero root-widen events.
- Adversarial atomicity test: a generation or reuse denial for any selected
  neighborhood publishes no replacement receipt, counter, or evidence for any
  member; replay of the transaction idempotency key returns the exact prior
  complete outcome.

**Engineering decisions**
- Reuse 3.6b neighborhood grammar and 3.7 impact narrowing as the sole inputs
  to replan selection.
- Model the replan unit as an ordered admitted neighborhood set with a primary
  neighborhood plus typed widen-to-set reasons.
- Make widening reasons first-class evidence so certification can assert that
  broad replans are exceptional and explained.
- Keep receipt reuse proof and neighborhood selection proof aligned so later
  execution can consume one honest story.
- Freeze `UiCommittedAllocationLoweringInput` as the 3.8-to-3.9 handoff: a
  committed receipt, execution-admissible companion report, transaction
  identity, and allocation-frame epoch. 3.9 may lower only from this wrapper
  and may not reinterpret preview, policy, freshness, locality, or denial.
- Keep neighborhood selection downstream of runtime graph truth and Query
  consumption truth; 3.8 may specialize allocation invalidation, but it may
  not become a second graph-authority or consumer-proof lane.

**Open questions**
- Resolved in Contract Appendix:
  `Replan Unit And Neighborhood-Set Law`,
  `Generation, Concurrency, And Supersession Law`.

### Phase 8: Continuous Viewport Resize Semantics

This phase freezes viewport resize as its own allocation pressure family.

Viewport resize is not the same thing as drag-driven durable resize mutation.
It is host extent observation plus derived allocation invalidation and receipt
recompute policy.

3.8 must explicitly define:

- how viewport extent observations enter the runtime
- which invalidation family owns viewport resize
- what receipt recompute cadence is legal under continuous viewport churn
- how viewport-local replanning stays neighborhood-bounded

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/activation/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/measurement/`

**Relevant APIs**
- `UiHostObservation`
- `UiMeasurementRequest::viewport_extent(...)`
- `UiMeasurementBasis`
- `runtime.plan_allocation(...)`
- Worth UI inspection facade

**Warnings**
- Viewport resize must not inherit splitter-drag durable mutation policy by
  default.
- Continuous viewport churn must not force broad whole-page replanning when the
  neighborhood proof remains local.
- Do not hide viewport resize cost by mutating committed receipt truth in
  place.
- Viewport observations may invalidate existing admitted neighborhoods, but
  they may not author new graph structure or dependency ownership outside the
  runtime graph lane.

**Test requirements**
- Adversarial viewport test: continuous window-edge resize keeps source
  observations exact while committed receipt churn stays policy-bounded and
  neighborhood-local, with an asserted maximum committed receipt count for `N`
  viewport samples under the named policy.
- Adversarial locality test: shell viewport change may replan affected shell or
  viewport-relative neighborhoods without disturbing unrelated local regions,
  with asserted touched versus untouched neighborhood ids and zero silent
  page-root collapse.

**Engineering decisions**
- Keep viewport resize policy distinct from drag-driven durable resize policy by
  default.
- Treat viewport resize as extent observation plus derived allocation replan,
  not as a durable semantic-state mutation lane.
- Record the viewport receipt-commit strategy in inspection evidence so later
  diagnostics can explain bounded lag or immediate recompute posture.

**Open questions**
- Resolved in Contract Appendix:
  `Stream Policy Composition Table`,
  `Replan Unit And Neighborhood-Set Law`,
  `Complexity And Evidence-Cadence Envelope`.

### Phase 9: Drag Preview And Durable Resize Commit

This phase freezes drag-driven preview and durable resize semantics so the
runtime can render smooth interaction without treating every pointer delta as
durable allocation truth.

3.8 must explicitly separate:

- preview allocation update
- committed durable resize state mutation
- allocation receipt recompute cadence
- mounted follow-on consumer cadence

This phase covers splitter drag and local resize handles as the hostile
continuous-interaction cases.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/graph/allocation_neighborhood/constraint_durable_resize_input.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/planning/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/activation/`

**Relevant APIs**
- `UiHostObservation`
- `UiMeasurementBasis`
- `runtime.plan_allocation(...)`
- Worth UI inspection facade

**Warnings**
- Preview motion must not overwrite durable resize state unless the declared
  stream policy says so.
- "Terminal on release" and "throttled durable commit" are semantics, not UI
  hacks; they must be modeled as runtime policy.
- Do not hide per-frame preview cost by mutating committed receipt truth in
  place.
- Drag preview may influence allocation recompute only through admitted
  runtime-owned invalidation and neighborhood lanes, not through gesture-local
  helper ownership.

**Test requirements**
- Adversarial drag test: 300 pointer deltas may update preview geometry while
  durable splitter state and committed allocation receipt cadence remain bounded
  and explicit, with asserted maximum durable resize mutations and asserted
  maximum committed receipt count for the 300-sample stream.
- Adversarial durable-commit test: release-time or threshold-triggered durable
  mutation updates semantic resize state only through the admitted stream
  policy, never as a side effect of preview rendering, with preview-only runs
  asserting zero durable mutations.

**Engineering decisions**
- Treat preview allocation as a separate runtime posture from committed
  `UiAllocationReceipt`.
- Allow stream-family-specific durable commit policies instead of one global
  rule.
- Record the commit strategy used for a drag/resize family in inspection
  evidence so later diagnostics can explain why durable state lagged or
  updated.

**Open questions**
- Resolved in Contract Appendix:
  `Preview Candidate Versus Committed Receipt Consumption`,
  `Stream Policy Composition Table`,
  `Generation, Concurrency, And Supersession Law`.

### Phase 10: Scroll-Owned Allocation Semantics

This phase closes scroll-owned layout truth as a runtime lane rather than an
adapter behavior.

3.8 must define:

- what `scroll-owned` means in planning and receipt terms
- who owns scroll extent truth
- what counts as observation versus durable semantic state
- which extent changes are allocation-relevant
- how scroll position, viewport, and content extent participate in invalidation

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/graph/allocation_neighborhood/constraint_scroll_owner_planning_input.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/measurement_request_boundary.rs`
- `workspaces/worth-ui/crates/worth-ui-host-contract/src/runtime/measurement_request/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/measurement/`

**Relevant APIs**
- `UiMeasurementRequest::scroll_container_viewport(...)`
- `UiHostObservation`
- `UiMeasurementResult`
- `UiMeasurementBasis`
- `UiAllocationReceipt`

**Warnings**
- Scroll position samples are not automatically durable layout truth.
- Host adapters may report viewport/extent observations, but may not decide
  scroll-owned allocation semantics.
- Scroll extent changes must not broaden to whole-page replans unless the
  neighborhood proof says they must.
- Scroll observation must flow through existing graph/index/receipt locality
  surfaces rather than opening a host-owned scroll dependency lane.

**Test requirements**
- Adversarial locality test: streaming list growth changes scroll-owned extent
  and replans only the list-owned allocation neighborhood, not unrelated page
  chrome or sibling mosaics, with asserted affected neighborhood ids and zero
  unrelated neighborhood churn.
- Adversarial denial test: unsupported or contradictory scroll ownership emits
  typed allocation denial instead of host-local fallback behavior.
- Adversarial offset-insensitivity test: when offset-sensitive allocation is
  not admitted, pure scroll-offset changes produce zero allocation invalidation
  events and zero committed receipt replacements.

**Engineering decisions**
- Keep `scroll-owned` as a declared runtime-owned coordinate/extent posture,
  not a rendering convenience flag.
- Freeze 3.8 law: scroll extent invalidates allocation when content extent or
  scroll-container viewport extent changes; scroll offset is projected
  interaction by default and does not invalidate allocation unless a later
  admitted operator family explicitly says so.
- Separate scroll extent invalidation from scroll position observation so the
  runtime can keep source truth exact without overcommitting allocation churn.
- Preserve receipt explanation for why an extent change did or did not require
  committed replanning.

**Open questions**
- Resolved in Contract Appendix:
  `Virtualization And Content-Totality Default`,
  `Query Settlement To Allocation Posture Law`,
  `Replan Unit And Neighborhood-Set Law`.

### Phase 11: Portal-Anchored Allocation Semantics

This phase closes portal anchoring as runtime-owned allocation truth instead of
floating host glue.

3.8 must define:

- anchor identity
- anchor coordinate space
- anchor observation intake
- portal placement invalidation rules
- anchor movement versus portal durable outcome semantics

This is where anchor movement caused by drag, resize, scroll, or content growth
must remain mechanically explainable.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/graph/allocation_neighborhood/constraint_portal_anchor_planning_input.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/measurement_request_boundary.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/services/`
- `workspaces/worth-ui/crates/worth-ui-host-contract/src/runtime/measurement_request/`

**Relevant APIs**
- `UiMeasurementRequest::portal_anchor_rect(...)`
- `UiHostObservation`
- `UiMeasurementBasis`
- `UiAllocationReceipt`
- `workspace.inspections().inspect(...)`

**Warnings**
- Portal placement must not become adapter-owned because the anchor rect comes
  from host observation.
- Anchor movement invalidation must be typed and inspectable, not inferred from
  a changed floating rectangle alone.
- Portal anchoring cannot silently bypass neighborhood selection just because
  it spans layers.
- Portal anchor movement may widen across admitted neighborhoods, but it may
  not bypass graph-owned locality proof or create a second host-owned portal
  membership graph.

**Test requirements**
- Adversarial anchor-movement test: moving an anchored control through scroll
  or drag updates only the portal-owned affected allocation neighborhood and
  keeps unrelated regions untouched, with asserted portal neighborhood-set
  membership and asserted unaffected neighborhood ids.
- Adversarial denial test: broken, stale, or generation-mismatched portal
  anchors deny committed receipt reuse with typed evidence instead of host-local
  snap correction, and denial must occur before any committed replacement
  receipt is produced.
- Adversarial anchor-identity test: equivalent anchor movement preserves or
  replaces receipt identity only according to the named portal identity rule,
  with the expected identity transition asserted explicitly.

**Engineering decisions**
- Treat portal anchor identity as a first-class allocation input artifact.
- Keep portal anchor observations in the host observation lane and convert them
  into allocation semantics only through runtime admission.
- Freeze 3.8 law: portal anchor observation invalidates the portal-owned
  neighborhood set; portal placement outcome is derived allocation truth, not
  host glue.
- Make portal placement reuse and denial explainable through receipt and causal
  inspection surfaces.

**Open questions**
- Resolved in Contract Appendix:
  `Portal Identity Rule`,
  `Replan Unit And Neighborhood-Set Law`,
  `Generation, Concurrency, And Supersession Law`.

### Phase 12: Query/UI Authority Map And Dependency Edge

This phase freezes the post-9.13 authority map before any consumer API is
rewritten. It makes the intended dependency direction executable and names the
few places allowed to translate Query meaning into Worth UI meaning.

**Relevant subsystems**
- `workspaces/worth-ui/Cargo.toml` and crate manifests
- `tools/boundary-check/config/road1.toml`
- `workspaces/worth-ui/crates/worth-ui-query-binding/`
- `workspaces/worth-ui/crates/worth-ui-runtime/`
- `workspaces/worth-ui/crates/worth-ui-certification/`

**Relevant APIs**
- `worth_query::facade::{domain, read, live, inspection, runtime}`
- `WorthQueryInstalledDomainHandle<WorthUiDomainEntry>`
- `WorthQueryProjectionOutcome`
- current Worth UI Query-facing facade exports

**Warnings**
- Moving a direct Query import behind a binding-crate re-export is topology
  laundering, not boundary closure.
- Do not prohibit Foundational-native value types merely because Query also
  uses them; the rule protects Query-owned authority and lifecycle, not textual
  crate-name purity.
- Certification needs explicit authority to construct hostile Query worlds,
  but that exception must not leak into production or ordinary test support.
- Avoid per-file allowlists. The enforceable unit is responsibility and crate
  direction unless a narrower exception is structurally unavoidable.

**Test requirements**
- Adversarial topology test: production `worth-ui-runtime` and the public
  `worth-ui` facade compile without a direct `worth-query` dependency, while
  `worth-ui-query-binding` remains the only production Query consumer and
  certification remains an explicit test-only exception.
- Adversarial sabotage test: adding a direct runtime Query dependency, a raw
  Query facade re-export, or a second Query translation crate fails the named
  boundary checker and reports the allowed path.
- Adversarial completeness test: every current production Query import is
  classified as installed-domain declaration, projection consumption,
  lifecycle, identity retention, inspection projection, or residue to delete;
  no unclassified edge may survive the phase.

**Engineering decisions**
- Make `worth-ui-query-binding` the single semantic edge from Query into Worth
  UI runtime.
- Keep Query certification construction in `worth-ui-certification` behind
  explicit test/certification posture rather than weakening the ordinary edge.
- Generate per-crate agent context from the canonical boundary configuration
  so agents see the legal dependency direction where they work.
- Prefer a positive compile-pass transcript of the ordinary path plus a small
  sabotage matrix over a growing blacklist of historical names.

**Open questions**
- None.

### Phase 13: Installed Worth UI Domain Composition And Lifecycle

This phase connects the 9.13 installed-domain capability to the real Worth UI
builder and runtime lifecycle. Query-free UI remains simple; Query-enabled UI
cannot launch from an authority-free bootstrap.

The canonical artifact may retain stable binding declarations, but the active
runtime must separately retain the runtime-affine installed capability and its
installation generation. Semantic declaration identity and runtime authority
are not interchangeable.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/installed_domain/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/entry/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/launch/`
- replacement, rebind, preservation, shutdown, and inspection lifecycle lanes

**Relevant APIs**
- `worth_ui_domain_package()`
- `worth_ui_native_aspect_contracts()`
- `WorthUiQueryExt`
- proposed installed UI binding capability and builder attachment surface
- `WorthUiBuilder::freeze(...)` and runtime launch progression

**Warnings**
- Do not make Worth UI mint Query runtime authority or own the Query workspace.
  The owning Query workspace installs; Worth UI receives the admitted
  capability.
- Do not store a reporting domain key or installation-generation integer as a
  substitute for the runtime-affine handle.
- Do not make Query setup mandatory for Query-free apps.
- Do not accept Query-bound declarations and defer missing installation to a
  late panic or host callback.

**Test requirements**
- Adversarial lifecycle test: launch, replacement, rebind, preservation, and
  shutdown all propagate the same installed capability; adding a new lifecycle
  fork without propagation produces a compile failure or exhaustive-match
  failure.
- Adversarial authority test: foreign-runtime, stale-generation, wrong-world,
  and detached diagnostic representations cannot activate Query-bound UI and
  perform zero allocation planning or frame ingress work.
- Adversarial parity test: Query-free apps retain the same builder ergonomics
  and runtime behavior, while registering a Query-bound declaration without an
  installed capability yields one typed, actionable admission result.

**Engineering decisions**
- Replace generic Query-enabled `WorthUiQueryBindingSubsystem::bootstrap()`
  with construction that consumes a binding-owned installed capability.
- Preserve an explicit Query-free runtime state rather than using `Option` as
  an ambiguous authority marker throughout the runtime.
- Carry runtime affinity and installation generation through one opaque
  binding-owned artifact; do not duplicate those fields in UI runtime structs.
- Make the builder error or typestate name the exact missing act: install the
  Worth UI Query domain and attach its bindings.

**Open questions**
- The exact public type and method names remain implementation decisions. The
  installed-authority grammar and lifecycle propagation are locked.

### Phase 14: Declarative Query Binding Definition And Agent DX

This phase replaces scattered Query posture assembly with one semantic binding
definition. The common API states what UI view is desired; the binding layer
derives capability, result-shape, basis, live, projection, invalidation, and
lifecycle requirements from the installed declaration.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/declaration/`
- Worth UI view-binding capability registration
- source lowering and canonical artifact binding declarations
- public Worth UI facade compile transcripts

**Relevant APIs**
- installed-domain read/live declarations from `WorthUiQueryExt`
- proposed `WorthUiQueryViewBinding` or equivalent semantic definition
- `ViewBindingDescriptor`
- `WorthUiBuilder::register_view_binding(...)`

**Warnings**
- A mega-builder with many optional `with_query_*` calls is the current wiring
  model in a friendlier coat.
- UI-owned presentation choices may accompany a binding, but they must not
  redefine Query support, result, basis, live, or projection semantics.
- Do not put runtime-affine authority into serializable canonical source
  artifacts; bind stable declaration identity to installed runtime authority at
  the admitted lifecycle boundary.
- Avoid clever macros when ordinary definition functions and typed artifacts
  provide sufficient inference and diagnostics.

**Test requirements**
- Adversarial convergence test: file-authored and Rust-authored definitions of
  the same installed Query view lower to the same canonical UI binding identity,
  consumption contract, invalidation footprint, and lifecycle requirement.
- Adversarial construction test: callers cannot independently assemble or
  mismatch capability status, result shape, basis posture, live compatibility,
  or composition identity; invalid combinations are unrepresentable or fail at
  the definition boundary.
- Adversarial DX transcript: an external consumer installs the Worth UI domain,
  derives one measurement view, registers it in one coherent call, inspects its
  plan, and launches without deep imports or manual digest plumbing.

**Engineering decisions**
- Replace `QueryViewCapabilityReference`, `QueryResultShapeReference`,
  `QueryBasisPostureReference`, and `QueryLiveCompatibility` as public assembly
  inputs with one binding-owned admitted definition.
- Use an object/spec shape for the view's UI presentation and a builder only
  where Query proof progression genuinely occurs.
- Make the ordinary definition expose an inspection surface for its declared
  facts, invalidation scope, lifecycle, and expected value shape.
- Write compiler errors and typed denials as routing help: they should name the
  correct installed-domain or binding method rather than only say forbidden.

**Open questions**
- Exact final naming remains open until implementation validates inference and
  autocomplete. The one-definition grammar is not open.

### Phase 15: Native Projection Consumption And Allocation Handoff

This phase carries Foundational-native values and the exact Query-minted
projection authority through the UI binding edge into allocation ingress. The
binding may refine native values into UI measurement meaning, but it may not
stringify, reparse, or reconstruct value authority.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/consumption/`
- Query projection-consumption native views and refinement APIs
- Worth UI measurement evidence and allocation frame Query gateway
- scroll-owned content extent and Query-backed measurement invalidation

**Relevant APIs**
- `WorthQueryProjectionOutcome`
- `WorthQueryConsumedProjectionAuthority`
- Foundational `AspectValue`, `CanonicalF32`, and exact native refinements
- `WorthUiQueryMeasurementFactSettlement`
- `WorthUiQueryProjectionSubmission`

**Warnings**
- Float bits, JSON-shaped values, debug strings, or coarse local scalar enums
  must not become an intermediate authority representation.
- Projection warnings and partial settlement must remain attached to the same
  consumed authority rather than being flattened into UI booleans.
- Do not rescan all projected fields when the installed declaration names the
  exact measurement facts required by the binding.
- The allocation gateway accepts the binding-owned settlement; it does not
  import Query types to interpret them again.

**Test requirements**
- Adversarial native-value parity test: scalar, struct, absence, and refinement
  behavior observed through Worth UI matches the Query native projection view
  exactly, including canonical float behavior and unsupported-family denial.
- Adversarial authority-continuity test: the installed domain authority,
  projection consumption authority, native value contract, settlement, and
  allocation ingress remain one traceable chain; copied values or receipts
  without the authority cannot enter the frame.
- Adversarial boundedness test: consuming `k` declared measurement facts performs
  `O(k)` refinement with exact counters and does not grow with unrelated rows,
  fields, domains, or UI bindings.

**Engineering decisions**
- Store native canonical carriers, not representation bits or JSON-like
  intermediates, wherever the value itself participates in UI meaning.
- Let `worth-ui-query-binding` own the only Query-to-UI fact refinement and
  settlement mapping.
- Pass one opaque settlement into allocation frame dispatch; runtime policy may
  classify its UI allocation consequences but may not reinterpret Query value
  legality.
- Retain exact warning, partial, source, basis, and contract evidence for local
  inspection without making those projections new authority.

**Open questions**
- None.

### Phase 16: Eliminate Query Mirrors And Reconstructed Digest Authority

This phase deletes the pre-9.13 consumer-local Query model and replaces every
operational use of formatted or rehashed Query identity with carried typed
authority or a binding-owned canonical identity.

UI-specific presentation and allocation classifications may remain when they
express genuinely new UI meaning. A type survives only if deleting Query from
its name would still leave an independently meaningful UI responsibility.

**Relevant subsystems**
- view-binding capability descriptors and source binding semantics
- runtime dependency hooks, candidate admission, replacement, and rebind
- measurement basis compatibility and projection receipt indexing
- facade graph Query re-exports and inspection fingerprints

**Relevant APIs**
- current `Query*Reference` types
- `WorthUiRuntimeDependencyHook::digest_basis()`
- `WorthUiQuerySupportReceipt`
- `UiQueryMeasurementBasisAuthority`
- Query-owned typed identity, basis, contract, and receipt surfaces

**Warnings**
- Renaming a mirror or moving its hash into `worth-ui-query-binding` does not
  remove competing authority.
- Query reporting projections may be displayed and logged, but may not order,
  admit, compare, reuse, rebind, or key operational state.
- Do not delete legitimate UI-owned posture such as denial presentation or
  allocation relevance merely because it is adjacent to Query.
- A diagnostic fingerprint must be named and typed as diagnostic, and changing
  it must have zero operational effect.

**Test requirements**
- Adversarial identity test: terminal reporting projections, debug formatting,
  copied digest strings, and locally hashed Query metadata cannot satisfy
  candidate admission, replacement equivalence, binding rebind, or allocation
  authority requirements.
- Adversarial derived-state test: all UI Query indexes, inspection summaries,
  and diagnostic fingerprints can be destroyed and rebuilt from retained
  binding authority and canonical UI artifacts without changing operational
  outcomes.
- Adversarial residue test: source scanning and API snapshots report zero
  consumer-local Query mirror types, zero operational `digest_basis()` chains,
  and zero local hashing of Query reporting projections.

**Engineering decisions**
- Remove the four current `Query*Reference` assembly types and replace their
  operational consumers with the admitted binding definition from Phase 14.
- Replace locally folded Query support receipts with typed binding contract
  identity and posture carried from the binding edge.
- Use typed `BasisDigest` or the stronger retained basis authority consistently;
  do not keep parallel `Box<str>` basis variants.
- Keep derived index keys visibly non-authoritative and require the retained
  authority alongside them for every operational lookup result.

**Open questions**
- None.

### Phase 17: Public Facade And Discovery Closure

This phase makes the post-9.13 path the only path agents and application authors
can discover. It removes public invalid constructors, stale aliases, and
historical setup from ordinary documentation rather than teaching callers how
to avoid them.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui/src/facade/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/`
- external compile-pass and compile-fail transcripts
- `workspaces/worth-ui/docs/worth-ui-readme.md`

**Relevant APIs**
- Worth UI builder and Query binding facade
- `ViewBindingDescriptor`
- runtime support inventory facade
- installed-domain and native-value discovery examples

**Warnings**
- Documentation is not the enforcement layer, but stale discovery reliably
  teaches agents to recreate deleted architecture.
- Do not preserve public constructors solely so certification can prove that
  their products are rejected; certification can use hostile fixtures outside
  the ordinary facade.
- Do not document historical methods as banned alternatives. Ordinary
  discovery teaches only the current positive path.
- Avoid giant facade barrels that merely conceal lower topology.

**Test requirements**
- Adversarial facade test: an external consumer can complete the installed
  domain, declarative binding, native projection, allocation settlement, and
  inspection journey using only admitted facade modules.
- Adversarial prohibition test: the deprecated phase-specific support alias,
  public pseudo-Query constructor, raw Query graph re-exports, and detached
  Query bootstrap are absent from public API snapshots and fail compile probes.
- Adversarial documentation test: every advertised stable example compiles,
  every relative link resolves, and source scanning finds no legacy setup,
  mirror-type, manual-digest, or JSON-shaped Query guidance.

**Engineering decisions**
- Remove `PHASE3_RUNTIME_SUPPORT_INVENTORY` and migrate all tests to the stable
  inventory name.
- Remove `local_pseudo_query_for_diagnostics` from the public production
  descriptor; hostile construction belongs in certification support.
- Expose one cohesive Query binding namespace from the Worth UI facade and keep
  lower Query topology private.
- Update the UI discovery document in the same change as the public cutover so
  API and teaching cannot drift.

**Open questions**
- None.

### Phase 18: Responsibility Topology And Agent Guardrails

This phase makes the next correct edit spatially obvious. It restructures the
allocation and Query-binding neighborhoods around authority and lifecycle, then
installs narrow structural checks so the tree does not relapse under later 3.8
and 3.13 work.

**Relevant subsystems**
- `runtime/allocation_receipt/`
- `graph/allocation_neighborhood/`
- `runtime/invalidation_narrowing/`
- `worth-ui-query-binding/src/prerequisites/` and its post-cutover successors
- Worth UI topology certification and generated agent context

**Relevant APIs**
- module/facade exports for allocation truth, transaction, reuse, and reports
- neighborhood admission, constraint authority, and replan selection
- boundary checker and structural inventory audits

**Warnings**
- File count alone is not the law. Split by receipt truth, transaction
  lifecycle, reuse/freshness, neighborhood admission, constraint authority, and
  replan selection because those responsibilities have distinct structural
  fate.
- Do not replace a flat directory with ceremonial folders that mix the same
  axes again.
- Structural checks should reject bucket names and boundary bypasses, not freeze
  every current filename forever.
- Generated agent context explains the canonical homes; it must not become a
  second manually edited architecture document.

**Test requirements**
- Adversarial navigability test: each allocation/Query responsibility maps to
  one production home, no immediate production directory exceeds ten files
  without an explicit responsibility-based exemption, and no `helpers.rs`,
  `types.rs`, `model.rs`, `common.rs`, or `support.rs` bucket remains.
- Adversarial dependency test: moving receipt truth into planning, locality
  proof into host code, Query translation into runtime, or production behavior
  into inspection/certification fails the topology audit.
- Adversarial deletion test: deprecated aliases, mirror types, and hostile-only
  constructors can be deleted with their responsibility without preserving
  fragments in unrelated modules.

**Engineering decisions**
- Organize allocation receipt code by committed truth, transaction, reuse,
  report/freshness, and ledger lifecycle.
- Organize allocation neighborhood code by admission, membership, constraint
  authority, replan selection, and activation handoff.
- Keep one canonical machine boundary contract and regenerate agent context
  after topology changes.
- Treat this phase as enabling infrastructure for Phases 19-22 and later 3.13,
  not as an aesthetic cleanup pass.

**Open questions**
- None.

### Phase 19: Allocation Inspection Surface

This phase closes the local inspection surface needed to keep 3.8 from
becoming an authority black box.

Allocation receipts and replan outcomes must be able to explain:

- why this receipt exists
- which basis, neighborhood, and stream family produced it
- why reuse was admitted or denied
- why the neighborhood was selected or widened
- which geometry relationships were committed for later AI critique and visual
  evaluation lanes

Ordinary receipt/report inspection must answer those questions locally.
Cross-runtime causal inspection is escalation for broader cross-runtime why, not
the default allocation debugger.

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-inspection/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/`

**Relevant APIs**
- Worth UI inspection facade
- `workspace.inspections().inspect(...)`
- `WorthUiQueryMeasurementFactSettlement`
- `UiAllocationReceipt`

**Warnings**
- Logs are not the evidence contract.
- Do not make later AI visual critique depend on screenshot archaeology when
  allocation receipts already know the geometry relationships that produced the
  visible result.
- Ordinary allocation inspection must not require causal escalation just to
  answer local receipt questions.

**Test requirements**
- Adversarial explanation test: for any denied or widened replan, inspection
  can name the invalidation family, neighborhood basis, reuse denial reason,
  and resulting receipt posture, and must return exact evidence refs for the
  invalidation artifact, neighborhood-selection artifact, and reuse/denial
  artifact.
- Adversarial escalation-boundary test: ordinary receipt/report inspection
  answers local allocation why without requiring Query workspace inspection
  unless the question crosses runtime boundaries, and the ordinary response must
  include the exact local fields required by the inspection contract.
- Adversarial critique-readiness test: AI-facing inspection can identify the
  receipt-backed bounds, spacing relationships, anchor posture, and relevant
  geometry evidence for a selected region without scraping logs or relying on a
  screenshot-only explanation path, with exact geometry evidence refs returned.

**Engineering decisions**
- Preserve allocation-linked geometry evidence as first-class inspection output
  so later visual-evaluation milestones can critique real runtime truth rather
  than inferring structure back from pixels.
- Keep ordinary allocation inspection local, typed, and receipt-backed before
  invoking broader cross-runtime explanation tools.

**Open questions**
- Resolved in Contract Appendix:
  `Allocation Inspection And Visual-Critique Readiness`,
  `Geometry Evidence Minimum Schema`.

### Phase 20: Allocation Freshness And Lag Posture

This phase closes the freshness contract for committed allocation truth.

3.8 must define:

- where freshness and lag posture live
- which freshness postures are admitted
- which consumers may use which postures
- when committed truth is current versus stale-but-bounded versus denied

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/`
- `workspaces/worth-ui/crates/worth-ui-inspection/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_receipt/report_freshness/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/execution/host_lanes/`

**Relevant APIs**
- `UiAllocationReceipt`
- `UiAllocationReceiptReport`
- Worth UI inspection facade

**Warnings**
- Committed does not mean current.
- Freshness must not be reconstructed from timing heuristics or logs.
- Execution consumers must not silently treat `recompute_pending` as valid
  committed geometry truth; attempted-transition denial is inspected through
  transaction lineage, not as a receipt posture.

**Test requirements**
- Adversarial posture test: inspection distinguishes receipt `current`,
  `coalescing`, `stale_but_bounded`, and `recompute_pending` from separate
  preview-candidate artifacts and attempted-transition denials, with the
  expected posture or outcome asserted per scenario.
- Adversarial consumer-boundary test: host paint and downstream execution obey
  the admitted freshness-consumption rules rather than sharing one permissive
  posture, with explicit assertions for which postures each consumer may and
  may not consume. Topology assembly must accept
  `UiCommittedAllocationLoweringInput`, never a raw receipt, so a caller cannot
  bypass or widen freshness admission.

**Engineering decisions**
- Place freshness and lag posture on companion `UiAllocationReceiptReport`
  while leaving receipt identity, equivalence, and generation on
  `UiAllocationReceipt`.
- Derive bounded-stale execution legality from the report-carried lag bound;
  expose no public constructor or caller-supplied lag allowance.
- Keep freshness as a typed runtime contract, not a presentation-layer summary.

**Open questions**
- Resolved in Contract Appendix:
  `Receipt Identity, Freshness, And Partial Reuse`,
  `Preview Candidate Versus Committed Receipt Consumption`.

### Phase 21: Allocation Counters And Denial Taxonomy

This phase closes the boundedness proof and denial vocabulary for allocation
churn.

3.8 must define:

- which counters are mandatory
- which denial families are mandatory
- how counters and denials attach to receipts, neighborhood selection, and
  stream policy outcomes

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_receipt/report_freshness/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_receipt/transaction/`
- `workspaces/worth-ui/crates/worth-ui-certification/`
- `workspaces/worth-ui/crates/worth-ui-inspection/`

**Relevant APIs**
- Worth UI inspection facade
- `UiAllocationReceipt`
- `UiAllocationReceiptReport`
- `UiMeasurementResult`

**Warnings**
- Counters must explain work performed, not just elapsed time.
- Denials must stay typed: generation mismatch, unsupported scroll ownership,
  broken portal anchor, impossible reuse, stale host evidence, and similar
  cases may not collapse into one generic fallback.
- Counter surfaces must not live only in certification helpers while production
  runtime truth stays opaque.

**Test requirements**
- Adversarial boundedness test: counters prove invalidation classification,
  neighborhood selection, replanned neighborhoods, reused receipts, denied
  reuse, and churn-burst handling under hostile stream pressure, with exact
  counter names and per-scenario maximum values asserted.
- Adversarial denial test: mandatory denial families surface as typed evidence
  with stable cause-sensitive identities and attached allocation causes;
  distinct reuse-denial reasons must retain their exact reason and must not
  collide. Certification must assert zero generic fallback-denial artifacts.

**Engineering decisions**
- Add allocation-specific denial taxonomy instead of generic measurement
  failure buckets.
- Expose counters at explicit measurement/allocation boundaries so 3.9 can
  inherit real surfaces instead of reconstructing them.

**Open questions**
- Resolved in Contract Appendix:
  `Receipt Commit Grammar`,
  `Replan Unit And Neighborhood-Set Law`,
  `Complexity And Evidence-Cadence Envelope`.

### Phase 22: Runtime Integration And Certification Closeout

This phase closes the actual runtime path and certification program for 3.8.

It is not enough to define receipt, invalidation, and stream semantics
separately. The milestone must prove that the committed allocation lane fits
the existing proof-flow grammar:

`measurement basis admit -> neighborhood admit -> plan_allocation -> allocation receipt commit -> downstream execution consumers`

**Relevant subsystems**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/allocation_receipt/`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/tests/activation/`
- `workspaces/worth-ui/crates/worth-ui-certification/`
- `workspaces/worth-ui/crates/worth-ui-test-support/`

**Relevant APIs**
- `runtime.plan_allocation(...)`
- `UiAllocationReceipt`
- Worth UI inspection facade
- `WorthUiQueryMeasurementFactSettlement`
- `workspace.inspections().inspect(...)`

**Warnings**
- Do not let certification support or test fixtures become the production lane.
- Do not treat broad whole-page replanning as acceptable just because the
  resulting screen looks right.
- Do not close the milestone while `3.9` would still need to reopen allocation
  truth, receipt identity, or stream semantics.
- Do not certify a path where allocation invalidation or locality proof is
  derived from consumer-owned scans, host-owned dependency state, or Query
  internals that bypass admitted projection-consumption and graph lanes.

**Test requirements**
- Adversarial integration test: one hostile workbench scenario combines typing,
  streaming data growth, splitter drag, viewport resize, scroll extent change,
  and portal anchor movement while preserving typed source truth and bounded
  allocation replanning, with asserted maximum committed receipt count, maximum
  neighborhood-set cardinality, and zero silent root-widen events.
- Adversarial anti-bypass test: attempts to derive allocation truth from host
  adapter state, local helper caches, or synthetic neighborhood construction
  fail compile-time, certification-time, or through typed denials, with zero
  host-derived committed receipt paths admitted in certification.
- Adversarial oracle test: certification asserts zero untyped invalidation
  paths, zero candidate-to-committed coercions, bounded receipt-commit counts
  under named policies, counted root-widen attempts, zero generic fallback
  denials, and deterministic replay of the same hostile event batch.

**Engineering decisions**
- Route all certification through `worth-ui-test-support` and facade/certify
  consumers rather than deep runtime internals.
- Treat 3.8 as the closeout of committed allocation truth before 3.9 lowers
  execution plans over it.
- Keep the 3.7 non-reopen rules explicit in certification scenarios.

**Open questions**
- Resolved in Contract Appendix:
  `Generation, Concurrency, And Supersession Law`,
  `Anti-Bypass Type Fence Law`,
  `Complexity And Evidence-Cadence Envelope`.

## Must Ship

- one production installed-domain composition path from the owning Query
  workspace into Worth UI builder, launch, replacement, rebind, and shutdown
- one declarative Query view-binding definition that derives Query support,
  result-shape, basis, live, projection, invalidation, and lifecycle posture
  without scattered consumer assembly
- `worth-ui-query-binding` as the mechanically enforced single production
  semantic edge from Query into Worth UI runtime
- Foundational-native projection values preserved through Query consumption,
  UI measurement settlement, and allocation ingress without JSON, float-bit,
  or coarse local value authority
- zero consumer-local Query mirror types and zero operational identity derived
  from reporting projections, debug formatting, or consumer-authored digests
- current positive discovery and external compile transcripts for Query-free and
  Query-enabled Worth UI composition
- responsibility-shaped allocation and Query-binding topology with no bucket
  modules or broad same-level production directories left for later agents
- `UiAllocationReceipt` as a committed runtime truth artifact with explicit
  identity, generation, reuse, and denial posture
- `UiAllocationReceiptReport` as the companion freshness, lag, widening,
  stream-policy, and counter posture artifact
- explicit distinction between stream ingress, local projected interaction
  state, candidate allocation result, preview allocation posture, committed
  allocation receipt, and durable semantic state
- runtime-owned frame dispatcher, immutable admitted stream frame, and
  capability-shaped source-to-frame gateways before stream policy may cause an
  ordinary allocation transition
- typed stream commit-strategy contract for continuous interaction and
  continuous data/measurement families
- same-frame stream policy composition table and override legality rules
- typed invalidation family taxonomy for viewport, resize, drag, content
  growth, scroll-owned extent, portal-anchor, Query-backed measurement fact,
  and host measurement replacement paths
- ordered neighborhood-set replan selection with typed widen reasons and
  forbidden silent root collapse
- affected-neighborhood replan selection built on admitted 3.6b neighborhood
  artifacts and 3.7 impact narrowing
- narrow partial-reuse support through `structure_reuse_leaf_remeasure`
- runtime-owned semantics for `viewport-relative`, `scroll-owned`, and
  `portal-anchored` committed allocation behavior
- allocation-specific inspection evidence, freshness posture, denial taxonomy,
  and boundedness counters
- allocation-linked geometry evidence sufficient for later AI critique of
  alignment, spacing, symmetry, and visual-drift questions without rebuilding
  layout truth from screenshots
- certification scenarios proving 3.8 did not become host-owned or broad-churn
  folklore

## Must Preserve

- Query runtime installation, domain, basis, projection, lifecycle, and native
  value authority without UI-local reinterpretation
- ceremony-free Query-free Worth UI composition and one coherent declarative
  act for Query-enabled composition
- 3.7 cleaned facade/proof-flow topology and anti-bypass fences
- host adapters as mechanics/observation only, never allocation-truth owners
- Query-owned basis, inspection, and projection-consumption semantics
- admitted measurement basis and neighborhood artifacts from 3.6a/3.6b as the
  only honest start-here path
- authority/derivation separation between source facts, candidate plans,
  committed receipts, and mounted follow-on consumers
- receipt-first and evidence-first explanation rather than logs or pixel-only
  guesswork
- AI-critique readiness through receipt-backed geometry and relationship
  evidence, not screenshot-only design judgment
- bounded locality rather than whole-page fallback as the ordinary replanning
  path

## Acceptance Evidence

- `worth-ui-runtime` has no direct production `worth-query` dependency, and a
  sabotage dependency or raw Query facade re-export fails the canonical
  boundary checker
- a real external consumer installs the Worth UI Query domain, derives a
  measurement binding from the installed handle, attaches it to a Worth UI app,
  consumes native projection facts, enters allocation through the binding-owned
  settlement, and inspects the resulting evidence without deep imports
- foreign-runtime, stale-generation, copied-digest, reporting-projection, and
  raw-native-value attempts perform zero UI allocation or replacement work
- Query-free apps require no placeholder Query capability, while Query-bound
  declarations cannot freeze or launch without the installed binding
- residue certification reports zero Query mirror types, zero operational local
  digest reconstruction, zero deprecated phase aliases, zero invalid public
  pseudo-Query constructors, and zero bucket-named production modules
- `UiAllocationReceipt` exists as a typed committed artifact and can be
  inspected without reading host-local geometry state
- `commit_allocation_receipt(...)` or its final named seam is idempotent under
  replay of the same admitted plan/basis/neighborhood generations and denies
  stale-basis commit attempts with typed evidence
- viewport resize enters through `UiMeasurementRequest::viewport_extent(...)`
  and `UiHostObservation`, enters a runtime-owned admitted stream frame, and
  replans only the affected allocation
  neighborhood
- scroll-owned extent changes enter through
  `UiMeasurementRequest::scroll_container_viewport(...)` and remain
  runtime-owned
- portal-anchor changes enter through
  `UiMeasurementRequest::portal_anchor_rect(...)` and remain runtime-owned
- per-character typing may commit semantic field truth each event while
  allocation churn remains threshold- and neighborhood-bounded
- same admitted source facts replay to the same sealed frame, policy decision,
  transition outcome, and allocation evidence regardless of permitted arrival
  order; late ingress is assigned or denied by explicit epoch law
- splitter drag preview can update continuously without rewriting durable
  resize truth every pointer delta unless the declared stream policy requires it
- Query-backed content growth participates through installed-handle
  declarations, retained consumed-projection authority, binding-owned
  measurement settlement, and Query workspace inspection rather than local
  pseudo-Query caches
- ordinary receipt/report inspection can explain local reuse, widening,
  freshness, and denial without causal escalation
- AI-facing inspection can expose receipt-backed geometry relationships for a
  target region so later visual critique can reason about alignment, spacing,
  anchor posture, and allocation-local symmetry without reconstructing layout
  truth from host internals
- cross-runtime explanations for allocation denial or widening can route
  through the Query workspace inspection capability when ordinary receipt
  inspection is insufficient
- certification asserts zero untyped invalidation paths, zero
  candidate-to-committed coercions, counted root-widen attempts, asserted
  neighborhood-set cardinality, and bounded receipt-commit cadence under
  hostile churn

## Closeout Evidence

- the complete `worth-ui-runtime` library suite passes all 818 tests
- the milestone closeout suite proves local receipt and denied-attempt
  inspection, all four freshness postures, distinct paint/execution admission,
  mandatory boundedness counters, and the closed denial taxonomy
- one combined hostile workbench drives typing, Query growth, resize preview,
  durable resize, viewport observation, scroll observation, and portal movement
  through production framework turns; replay is deterministic, every
  transition remains exactly one neighborhood and one committed receipt, and
  root-widen attempts remain zero
- public certification consumes the runtime-origin fixture through
  `worth-ui-test-support`, asserts exact receipt and neighborhood bounds, and
  proves committed receipt execution consumption
- allocation anti-bypass certification finds no host-owned receipt minting,
  synthetic neighborhood construction, or consumer-owned allocation cache
  authority
- Road 1 boundary topology and generated agent-context checks pass, and every
  Rust file changed by this closeout remains within the 400-line limit

## Sequencing Notes

- Phases 1-11 were implemented before the Query 9.13 cutover. Phases 12-18 now
  replace the interim consumer model, and Phases 19-22 close inspection,
  freshness, boundedness, denial, integration, and certification.
- Phases 12-18 are intentionally inserted before allocation inspection,
  freshness, counters, and certification. Those later surfaces would otherwise
  make pre-9.13 mirrors and reconstructed identity part of the durable 3.8
  contract.
- The 3.8 cutover closes the narrow Query authority required for committed
  allocation truth. Milestone 3.13 still broadens general product-facing Query
  bindings, collections, live surfaces, and richer UI posture; it must consume
  this edge rather than reopen it.
- This milestone belongs after 3.6a because committed allocation truth must
  consume admitted measurement basis instead of inventing a second evidence
  lane.
- It belongs after 3.6b because incremental replanning must consume admitted
  allocation neighborhoods and deterministic planning rather than improvising
  locality during churn.
- It belongs after 3.7 because receipts and stream pressure would otherwise
  stack on top of broad facades, helper sprawl, and unclear proof-flow
  topology.
- It belongs before 3.9 because execution-plan lowering must consume committed
  allocation truth, dispatcher-owned ordinary stream transitions, explicit
  receipt reuse, and boundedness counters rather
  than rediscovering those semantics during execution.
- It should not be split into "nice layout receipts" and "stream handling"
  because the scaling boundary is precisely the interaction between committed
  allocation truth and churn-heavy stream pressure.
