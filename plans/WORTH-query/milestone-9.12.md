# Milestone 9.12: Query Public Authority Surface Cutover

## Goal

Make the public `worth-query` facade expose one authority-preserving path for
each Query capability, with raw representations, lifecycle machinery, and
certification internals unable to act as parallel authority APIs.

## Why This Milestone Exists

Milestone 9.11 closed canonical downstream basis and projection authority, but
a hostile public-surface audit found older and lower-level Query APIs that can
still mint authority-looking digests, accept raw basis identities, assert
subscription basis posture, or invoke admission machinery independently. Those
surfaces can recreate the competing-authority failures that 9.11 removed from
Worth UI.

This milestone closes Query's own remaining authority escape hatches before
store-backed execution multiplies the same contracts across a second execution
substrate.

## Governing Summaries

- `MENTALITY.md` protects adversarially correct foundations: invalid authority
  paths must become unrepresentable or uncompilable, and ordinary completion
  work must not be renamed as debt.
- `arch_laws.md` protects proof and authority continuity: representation is not
  authority, proof-bearing constructors must be sealed, and each lifecycle
  phase must consume the exact proof emitted by the prior phase.
- `composition_laws.md` protects readable semantic decomposition: facade files
  may aggregate but must not conceal multiple lifecycle implementations, and
  orchestration must name each authority transition.
- `domain_structure_laws.md` protects structural ownership: public topology
  must be narrower than internal topology, and authority, derivation,
  compatibility, diagnostics, and certification must occupy distinct spaces.
- `perf_laws.md` protects bounded pre-resolved execution: rejection precedes
  construction, proof is carried forward inside a trust boundary, and no
  convenience API may hide rediscovery or broadening cost.
- `WORTH_query_roadmap.md` protects one declared query meaning across runtime,
  store, live, and durable paths; it requires the runtime-backed public
  authority surface to close before Milestone 10 adds store-backed execution.

## Adversarial Constraint

A downstream consumer must be unable to mint, restamp, pair, or route Query
authority through a raw digest, string identity, posture enum, raw admission
request, legacy unscoped lifecycle, or certification-only artifact—even when
every supplied component is individually well-formed and its labels or digests
collide with a legitimately admitted Query artifact.

Equivalent declarative requests must converge on the same sealed Query-owned
capability. Cross-basis, cross-receipt, stale, fabricated, and phase-skipping
requests must be unrepresentable or fail typed before expensive construction or
lower-runtime contact.

## Product Decision Lock

- Public Query authority is minted only by Query-owned admission transitions.
- Digests, labels, identities, posture summaries, diagnostics, and serialized
  projections are evidence or representation; none may promote itself into
  authority.
- Each ordinary capability family has one declarative public entry path. Phase
  APIs may remain public only when they consume and return sealed successor
  types from the same lifecycle.
- Compatibility code cannot remain callable from the ordinary facade. A
  migration adapter may exist only at the narrowest internal boundary needed
  for in-repository cutover and must be deleted before closeout.
- Certification and diagnostic richness are derived consumers of operational
  receipts. They may not mint or alter operational authority.
- Store-backed and durable implementations extend these same sealed contracts;
  they do not justify retaining raw runtime-era entrypoints.

## Phase Plan

### Phase 1: Freeze The Public Authority Graph And Cutover Manifest

Status: Closed on 2026-07-13. The shipped Consumer Kit manifest classifies the
authority-bearing public constructor and transition surface, source-backed
tests discover newly added public constructors in the covered authority impls,
seeded audits reject duplicate and unclassified surfaces, and trybuild proves
the registry is read-only and its internal module is not consumer-reachable.

Freeze what the public facade currently allows before changing it. This phase
produces one reviewable map from every public authority-looking input to its
constructor, admitting transition, operational consumer, derived projection,
and intended post-cutover replacement.

**Relevant subsystems**

- `facade`
- `identity` and `identity_evolution`
- `basis_lifecycle`, `query_basis_lifecycle`, and `query_context`
- `intent_admission`, `subscription`, `preview`, and causal inspection
- Consumer Kit prohibition and support registries

**Relevant APIs**

- all `pub use` declarations in the public facade
- all public constructors for digest, identity, basis, posture, request,
  eligibility, admission, and scoped-proof types
- all public functions that cross from declaration into admission, lowering,
  execution, observation, inspection, or materialization

**Warnings**

- An export inventory is not sufficient unless it traces which values can
  reach operational authority.
- Equal names or layouts do not imply equal semantic authority; distinct
  authority and proof states must remain distinct in the manifest.
- Existing tests and examples count as consumers and can preserve a legacy
  path accidentally.

**Test requirements**

- A facade snapshot test proves every public authority-bearing constructor and
  transition is classified with one owner and one target posture.
- A seeded-residue test adds a representative raw constructor, legacy
  entrypoint, and deep export and proves the inventory fails with the exact
  unclassified surface.
- An import-graph test proves ordinary consumers cannot reach authority
  machinery outside the declared facade even when internal modules remain
  public within the crate.

**Engineering decisions**

- The manifest is machine-readable and drives later residue enforcement.
- Each surface is classified as ordinary declarative API, sealed phase API,
  read-only projection, certification-only API, internal adapter, or deletion.
- No compatibility-debt row may be accepted as the final posture of this
  runtime-backed milestone.

**Open questions**

- None.

### Phase 2: Seal Digest And Identity Authority Minting

Status: Closed on 2026-07-13. Canonical query and schema-basis authority now
originate from sealed Query-owned handles; external schema material crosses an
explicit non-authoritative token and admission boundary. Lineage,
correspondence, historical planning, replay, execution, and inspection retain
structural authority proofs instead of accepting free-standing digest or basis
labels. Compile-fail coverage rejects raw minting and private-field forgery,
collision coverage keeps equal reporting text from collapsing distinct basis
generations, and the full Query library suite passes.

Close the highest-risk edge first: externally supplied strings and domain
parts must not construct values accepted as Query authority. Query-minted
handles retain structural identity; callers receive only read-only evidence
projections where raw digest material is useful.

**Relevant subsystems**

- `identity`
- `identity_authority`
- `identity_evolution`
- evidence identity and public inspection projections

**Relevant APIs**

- public `from_domain_parts` digest constructors
- public lineage and correspondence request constructors that accept digests
- digest and identity exports from the foundation facade
- reporting accessors that expose digest labels or canonical evidence identity

**Warnings**

- Renaming a raw constructor does not seal it.
- A digest wrapped in a public tuple, builder, deserializer, or generic token is
  still mintable authority if an operational API accepts it.
- Schema or domain identifiers that legitimately originate outside Query need
  explicitly non-authoritative token types and fresh Query admission.

**Test requirements**

- Compile-fail tests prove downstream crates cannot construct canonical query,
  basis, schema-basis, historical-path, lineage, or correspondence authority
  from strings, domain parts, digest labels, or serialized projections.
- A hostile collision test supplies equal digest text from different basis
  generations and proves it cannot enter lineage, correspondence, execution,
  or inspection as authority.
- An equivalence test proves Query-minted handles reached through direct,
  fluent, replayed, and inspection paths preserve the same structural identity
  and read-only digest projection.

**Engineering decisions**

- Authority constructors and fields are sealed to the narrowest owning module.
- Operational identity-evolution APIs consume Query-minted authority handles or
  sealed comparison contexts, never free-standing digests.
- Digest projections remain available for evidence, persistence keys where
  admitted, diagnostics, and comparison reporting, but expose no promotion
  transition.

**Open questions**

- None.

### Phase 3: Collapse Query Basis Onto One Scoped Capability Lifecycle

Status: Closed on 2026-07-13. The canonical `basis_lifecycle()` declaration
surface covers current, branch, snapshot, preview, runtime snapshot, historical
snapshot, historical commit, tenant, and policy families. Query-context
admission now consumes that declaration plus Query-owned binding evidence and
returns a sealed `ScopedQueryBasisContext`; application plans, handoffs,
runtime execution, result bundles, diff contexts, certification fixtures, and
Worth UI consumers carry that scoped proof. The raw query-context request and
binding path and the overlapping `query_basis_lifecycle` surface are absent
from the public facade. Compile-fail proof rejects raw context construction and
scoped-context peeling, the compatibility-debt row is deleted, focused tests
pass, and the complete Query test graph compiles.

Converge `basis_lifecycle`, `query_basis_lifecycle`, and `query_context` onto one
ordinary declarative public path whose admitted successor is the exact scoped
basis proof required by downstream work. Remove public raw string basis
requests and unscoped execution transitions.

**Relevant subsystems**

- `basis_lifecycle`
- `query_basis_lifecycle`
- `query_context`
- basis support, inspection, and compatibility-debt registries

**Relevant APIs**

- current-head, branch-head, snapshot, commit, historical, preview, tenant, and
  policy basis declaration paths
- `bind_query_basis_context`, `admit_query_basis_context`, and
  `execute_query_basis_context`
- scoped basis admission and basis-use receipt paths

**Warnings**

- A scoped wrapper around an externally assembled unscoped artifact is not a
  completed lifecycle.
- Convenience and explicit phase APIs must share one transition, not merely
  produce similar labels or digests.
- Basis-family differences in authority, visibility, failure, or cost must not
  be flattened into a generic string-bearing context.

**Test requirements**

- Equivalent fluent and explicit declarations for every admitted basis family
  converge on the same scoped capability, use receipt, and evidence identity.
- Compile-fail tests prove raw branch, snapshot, commit, preview, tenant, and
  policy identifiers cannot skip normalization, eligibility, admission, or
  scoping.
- Cross-generation, stale-preview, inaccessible-branch, and policy-mismatch
  tests deny before execution or lower-runtime contact and produce no partial
  scoped successor.

**Engineering decisions**

- The ordinary API returns or carries a sealed scoped basis capability.
- Low-level normalize, evaluate, admit, and scope transitions remain public
  only where their types enforce the exact order and cannot be caller-minted.
- The query-context compatibility-debt row is deleted at phase closure rather
  than relabeled.

**Open questions**

- None.

### Phase 4: Replace Raw Intent Admission With Declarative Capability Handoffs

Status: Closed on 2026-07-13. Raw admission request constructors, eligibility
construction, and generic runtime admission are crate-private and absent from
the ordinary facade. Capability-owned declarations now lower through the
internal engine, including basis observation through `basis_lifecycle()`, and
compile-fail coverage proves consumers cannot assemble raw request or admitted
plan authority. Intent, domain-capability, public-DX, and no-run suites pass.

Stop exposing Query's generic admission substrate as an alternate application
API. Ordinary declarations produce sealed admission targets or typed stops;
eligibility facts and raw admission requests remain owned by the internal
admission lifecycle.

**Relevant subsystems**

- `intent_admission`
- domain capability contributions
- platform entry, effect, read, projection, inspection, and existing-truth
  intent families
- admission decision trace and lower-runtime routing

**Relevant APIs**

- `WorthQueryRawIntentAdmissionRequest`
- `WorthQueryIntentAdmissionEligibility::from_request`
- `admit_runtime_intent_request`
- intent seed and admitted-plan exports
- ordinary fluent declaration and domain-contribution entrypoints

**Warnings**

- Domains still need an ergonomic way to contribute semantic posture; sealing
  the raw engine must not force them into local pre-admission scaffolding.
- A generic public builder around the same raw request remains a parallel
  lifecycle.
- Certification fixtures must exercise the ordinary declaration lane unless
  they are explicitly inside a certification-only namespace.

**Test requirements**

- Compile-fail tests prove downstream crates cannot construct raw admission
  requests, eligibility artifacts, admitted plans, or execution handoffs.
- Parity tests prove each covered ordinary declaration and domain contribution
  reaches the same decision lattice, trace, and admitted handoff formerly
  reached through the raw request path.
- Advisory, violation, and unsupported-family tests prove no execution target
  is constructed and no lower-runtime route begins after a non-admitted result.

**Engineering decisions**

- Query exposes declarations and sealed outcomes, not its admission engine's
  intermediate assembly vocabulary.
- Domain capability contribution remains the public semantic extension seam.
- Raw requests and eligibility constructors become crate-private; any required
  test assembly moves behind narrow test-support authority.

**Open questions**

- None.

### Phase 5: Require Scoped Basis Proof For Subscription Declaration And Activation

Status: Closed on 2026-07-13. Public live-promotion constructors consume
`ScopedSubscriptionDeclarationBasis`; posture is derived only for diagnostics.
Selection, declaration, lowering, admission, activation, active-lane admission,
and the registry carry the sealed proof, while activation is derived as the
exact successor of that declaration. Equivalence and binding identities include
the scoped proof digest, so distinct branch authorities cannot share merely
because their posture labels match. Policy/tenant subscription declarations
fail closed, the subscription compatibility-debt row is deleted, and the
subscription, support, public-manifest, and compile-fail suites pass.

Replace caller-asserted subscription basis posture with Query-minted scoped
subscription declaration and activation proofs. Subscription family selection,
sharing, continuation, and delivery consume the same canonical basis lifecycle
as one-shot and live-promoted reads.

**Relevant subsystems**

- subscription declaration, admission, activation, support, and diagnostics
- live promotion and lifecycle sharing
- basis lifecycle and policy/tenant admission

**Relevant APIs**

- `QuerySubscriptionBasisPosture`
- `LiveQueryAdmissionArtifact::from_live_promotion*`
- `ScopedSubscriptionDeclarationBasis`
- `ScopedSubscriptionActivationBasis`
- subscription support and diagnostic artifacts

**Warnings**

- A posture enum describes a claim; it does not prove that Query admitted the
  claimed branch, history, preview, policy, or tenant relationship.
- Sharing keys and continuation identities must derive from the scoped proof,
  not independently from query and basis labels.
- Preview subscription isolation must not silently rebind to authoritative
  current-head truth.

**Test requirements**

- Equivalent one-shot, live-promotion, subscription-declaration, and activation
  paths preserve the same canonical query and scoped basis identity.
- Compile-fail tests prove callers cannot select `CurrentHead`, `BranchHead`,
  historical, or preview posture and feed it directly into subscription
  admission.
- Cross-basis sharing, stale continuation, preview discard, policy mismatch,
  and tenant mismatch deny without creating an active subscription or leaving
  authoritative residue.

**Engineering decisions**

- Public subscription construction consumes scoped declaration proof; active
  lifecycle construction consumes scoped activation proof.
- Basis posture remains a derived diagnostic projection where useful.
- The subscription compatibility-debt row is removed when all ordinary and
  certification callers use the scoped lifecycle.

**Open questions**

- None.

### Phase 6: Bind Causal Inspection To Scoped Inspection Basis

Status: Closed on 2026-07-13. `QueryObservationReceipt` now binds the scoped
inspection proof supplied at its conversion boundary, and
`CausalInspection::for_observation` requires the same proof explicitly. A
cross-basis pair returns typed `BasisMismatch` before anchoring, evidence
resolution, admission, or bridge assembly; the scoped digest also participates
in receipt and request identity. Raw request assembly is crate-private, the
causal compatibility-debt row is deleted, discovery docs teach only the scoped
path, and causal, public-DX, manifest, and receipt-only compile-fail suites pass.

Make causal inspection consume `ScopedInspectionBasis` alongside its Query
observation anchor. Observation receipts remain causal evidence, but they may
not independently authorize inspection against a reconstructed or ambient
basis.

**Relevant subsystems**

- runtime causal inspection
- query observation receipts and causal anchors
- cross-runtime explanation admission and materialization
- inspection basis lifecycle

**Relevant APIs**

- causal inspection request builders
- `QueryObservationReceipt`
- `ScopedInspectionBasis`
- causal explanation envelopes and inspection denials

**Warnings**

- An authentic observation receipt can still be stale or paired with the wrong
  inspection scope.
- Inspection materialization must not reopen lower-runtime authority to infer a
  missing basis.
- Redaction and diagnostic detail are derived policy; neither may change the
  admitted inspection authority.

**Test requirements**

- Direct, replayed, redacted, and narrowed inspection of the same observation
  and scoped basis produces equivalent causal identity and authority binding.
- Cross-basis receipt pairing, stale scope, missing evidence, and policy
  mismatch produce typed denial or advisory results before causal envelope
  assembly.
- Compile-fail tests prove a raw observation receipt cannot construct an
  operational causal inspection request without scoped inspection proof.

**Engineering decisions**

- Observation receipts anchor causality; scoped inspection basis authorizes
  inspection. Neither substitutes for the other.
- The causal compatibility adapter becomes internal during migration and is
  deleted before phase close.
- Cold diagnostic richness remains downstream of the admitted inspection
  artifact.

**Open questions**

- None.

### Phase 7: Carry Scoped Preview Basis Through Drift And Execution

Status: Closed on 2026-07-13. Preview-live admission now derives the canonical
`basis_lifecycle` observation proof and seals it with the session/live
component into `ScopedPreviewLiveSessionPlanBinding`. Execution envelopes,
maintenance, drift denial, maintained state, and explicit rebind artifacts all
carry that scoped binding; rebind performs a fresh admission and structural
equality, not report-digest comparison, determines maintenance. The legacy
live component is crate-private and absent from the facade, the compatibility
registry is empty, and preview, certification, manifest, and drift/execution
compile-fail suites pass.

Complete the preview-live cutover so drift assessment, execution envelopes,
explicit rebind, promotion comparison, and discard all consume the scoped
preview-live binding rather than recovering scope from the older session-plan
binding.

**Relevant subsystems**

- preview session query contexts
- preview-live admission, drift, rebind, promotion, and discard
- live execution envelopes and subscription preview isolation

**Relevant APIs**

- `PreviewLiveSessionPlanBinding`
- `ScopedPreviewLiveSessionPlanBinding`
- `assess_preview_live_drift`
- preview-live execution-envelope constructors

**Warnings**

- Wrapping and later unwrapping the scoped binding before operational use loses
  the proof exactly where it matters.
- Drift decisions must compare structural preview authority and generation, not
  display labels or digest equality.
- Explicit rebind is a new admission transition, not mutation of an existing
  proof.

**Test requirements**

- Preview execution, live maintenance, drift assessment, and promoted-result
  comparison preserve one scoped preview identity across equivalent paths.
- Stale generation, discarded preview, mismatched session, and unauthorized
  rebind tests deny before execution and leave no authoritative live residue.
- Compile-fail tests prove the legacy preview binding cannot enter drift or
  execution without scoped admission.

**Engineering decisions**

- Operational preview-live APIs consume the scoped binding directly.
- The legacy binding may remain only as an internal phase component while the
  scoped successor is assembled; it is not exported as an ordinary artifact.
- The preview compatibility-debt row is deleted at phase closure.

**Open questions**

- None.

### Phase 8: Contract The Facade Around Stable Capabilities

Status: Closed on 2026-07-13. The public facade is split into explicit
foundation, policy, runtime, application, Consumer Kit, and certification
namespaces; ordinary exports no longer mirror certification or migration
topology. The public-authority manifest, golden pass fixtures, and seeded
compile-fail fixtures certify the contracted surface and the separate tooling
lane.

Replace the facade's mirror of internal topology with a narrow stable product
surface. Separate ordinary developer capabilities from certification,
inspection projection, migration tooling, and internal lifecycle machinery.

**Relevant subsystems**

- `facade`
- support and capability registries
- certification bundles, migration audits, phase manifests, and diagnostics
- public API compile snapshots

**Relevant APIs**

- `exports_foundation.rs`
- `exports_policy.rs`
- `exports_runtime.rs` and runtime phase export modules
- all public reexports classified by the Phase 1 manifest

**Warnings**

- Moving the same exports into another barrel file does not contract the API.
- Certification needs do not justify exposing operational constructors to
  ordinary consumers.
- Facade contraction must preserve daily-driver ergonomics for admitted
  capabilities rather than forcing deep imports.

**Test requirements**

- A golden public API snapshot proves the ordinary facade exposes only
  declarations, sealed capabilities, typed stops, operational receipts, and
  read-only inspection projections required by supported workflows.
- Seeded deep-export and certification-leak tests prove internal lifecycle,
  migration, manifest, and authority-construction surfaces cannot be imported
  by an ordinary downstream crate.
- Golden DX transcripts prove representative read, live, subscription,
  workflow, projection-consumption, basis, and inspection tasks remain no more
  ceremonious than their pre-cutover ordinary paths.

**Engineering decisions**

- Ordinary product API and certification/tooling API are distinct namespaces
  or crates with explicit dependency direction.
- Internal module visibility shrinks independently of whether a type is
  reexported.
- Public API growth requires an allowlisted capability owner and a facade
  snapshot update.

**Open questions**

- None.

### Phase 9: Cut Over Consumers And Install Permanent Prohibitions

Status: Closed on 2026-07-13. In-repository Query fixtures and runtime-backed
Worth UI consumers use the explicit capability facades, the parallel
`query_basis_lifecycle` implementation and its adapters are deleted, and the
Consumer Kit prohibition, compile-fail, and residue registries cover the
removed authority families. Discovery documentation teaches only the surviving
declarative paths.

Migrate every in-repository caller to the sealed declarative paths, delete
compatibility adapters and legacy fixtures, then make resurrection of each
removed seam fail mechanically in both Query and downstream consumer builds.

**Relevant subsystems**

- all worth-query tests, examples, compile fixtures, docs, and AI orientation
- Worth UI and other reference consumers
- Consumer Kit prohibition registry, compile-fail manifest, and residue audit
- support snapshots and capability profiles

**Relevant APIs**

- all surfaces marked `internal adapter` or `deletion` in Phase 1
- hard prohibition registry and downstream bypass audit
- public support and compatibility-debt registries

**Warnings**

- A legacy API is not removed while a golden fixture, example, test helper, or
  documentation snippet still teaches it.
- Source-text grep alone is insufficient enforcement; visibility and compile
  boundaries are primary.
- Migration aliases and deprecated constructors remain callable competing
  authority and therefore cannot survive closeout.

**Test requirements**

- Compile-fail suites cover raw digest minting, raw basis identities, unscoped
  query context, raw admission requests, posture-authored subscription,
  receipt-only causal inspection, legacy preview execution, and deep facade
  imports.
- Seeded consumer-residue tests prove each prohibited seam is detected in
  production, tests, examples, fixtures, docs, and generated public API
  snapshots.
- Reference-consumer parity tests prove the migrated paths preserve canonical
  results, receipts, authority identities, support posture, and bounded-work
  counters.

**Engineering decisions**

- Each removed seam gains a named prohibition row with the supported
  replacement and enforcement tier.
- Compatibility-debt registry rows for query context, subscription, causal
  inspection, and preview are removed, not retained as historical instruction.
- Historical explanation belongs in closeout or migration records, not the
  ordinary discovery documentation.

**Open questions**

- None.

### Phase 10: Certify One Public Authority Surface And Close The Milestone

Status: Closed on 2026-07-13. The digest-bearing Milestone 9.12 certification
bundle closes manifest, facade, prohibition, residue, intent-admission,
projection-consumption, hostile authority, and real Worth UI adoption evidence.
Public consumer-shaped certification proves one scoped authority chain, and
trybuild sabotage fixtures prove the removed families remain mechanically
unavailable.

Run hostile end-to-end certification across the complete cutover and prove the
remaining public surface cannot create a second authority path under
collision, replay, stale generation, cross-family pairing, or facade-growth
pressure.

**Relevant subsystems**

- query normalization, basis, admission, execution, subscription, preview,
  inspection, projection consumption, and identity evolution
- support/profile truth, Consumer Kit, public facade, docs, and closeout
- runtime-backed reference consumers

**Relevant APIs**

- the final ordinary public facade
- public support and inspection projections
- prohibition, residue, public API, and compatibility-debt registries
- milestone certification bundle

**Warnings**

- Passing behavioral tests is insufficient if a forbidden constructor remains
  callable.
- Certification must use public consumer-shaped code, not privileged internal
  test assembly.
- Store-backed extension points must be typed and fail closed without claiming
  Milestone 10 behavior early.

**Test requirements**

- A hostile authority matrix crosses every admitted capability with equal
  labels, equal digest text, different generations, different receipts,
  different policy/tenant scope, stale proofs, and replayed artifacts; only
  structurally identical admitted authority may converge.
- An end-to-end parity test proves declarative request through scoped admission,
  execution or observation, receipt, inspection, and evidence projection
  preserves one canonical authority chain without rediscovery.
- A sabotage test reintroduces one constructor or export from every removed
  family and proves facade snapshot, compile-fail, prohibition, residue, or
  compatibility-debt certification fails locally.
- Exact counters prove denials precede expensive construction and unrelated
  workspace, history, subscription, or consumer state does not affect
  admission work.

**Engineering decisions**

- Closure requires exact-zero callable legacy seams and exact-zero ordinary
  facade exports classified as internal, compatibility, or certification-only.
- Certification emits one digest-bearing bundle derived from the manifest,
  public API snapshot, prohibition registry, support matrix, hostile matrix,
  and reference-consumer adoption evidence.
- No phase may be declared closed through documentation-only posture.

**Open questions**

- None.

## Must Ship

- sealed Query-owned identity and digest authority construction
- one scoped public basis lifecycle across ordinary Query contexts
- declarative intent admission with internal raw request machinery
- scoped subscription, causal inspection, and preview-live follow-on paths
- a contracted ordinary facade separated from certification and migration
  machinery
- complete consumer adoption, named prohibitions, compile-fail coverage,
  residue enforcement, public API snapshots, and hostile certification

## Must Preserve

- Query owns expression, admission, public capability, receipt, and result-shape
  contracts without stealing relational, bridge, signal, store, policy, tenant,
  or downstream-domain authority
- canonical query meaning and runtime-backed behavioral parity across direct,
  fluent, explicit phase, live, subscription, preview, replay, and inspection
  paths where those paths are admitted
- evidence and diagnostics remain useful read-only projections
- ordinary developer ergonomics remain declarative and do not expose lifecycle
  machinery merely to reduce implementation work inside Query
- rejection precedes expensive construction and lower-runtime contact

## Acceptance Evidence

Milestone 9.12 is complete only when `worth-query` can prove:

- no downstream crate can mint operational query, basis, schema-basis,
  historical, lineage, or correspondence authority from raw representation
- no raw or unscoped query-context, intent-admission, subscription, causal
  inspection, or preview-live entrypoint remains callable from the ordinary
  facade
- every admitted ordinary declaration reaches one sealed successor chain and
  every invalid relationship produces one typed stop with no partial successor
- the compatibility-debt registry contains no runtime-backed rows covered by
  this milestone
- the public facade snapshot contains no internal lifecycle, migration,
  certification, or authority-construction leaks
- reference consumers preserve canonical results, receipts, authority identity,
  support posture, and complexity counters after deleting legacy usage
- sabotage and compile-fail suites make every removed authority path
  mechanically non-resurrectable

## Closeout Verification

Closed on 2026-07-13 with the following repository evidence:

- `cargo test -p worth-query --tests --quiet` passes the complete package test
  surface, including all integration and trybuild binaries.
- `cargo test -p worth-query --test milestone_nine_twelve_hostile_certification
  --quiet` passes the public authority-chain and real Worth UI adoption lanes.
- `cargo test -p worth-query --test public_authority_surface_compile_fail
  --quiet` passes all thirteen rejection fixtures and both golden facade
  fixtures.
- `cargo test -p worth-server --test query_dependency_audit --quiet` passes the
  downstream server dependency audit after facade contraction.
- `cargo check --workspace --quiet` passes at the repository root, and
  `cargo check --workspace --quiet` passes in `workspaces/worth-ui`.
- `cargo test -p worth-query --doc --quiet`, `cargo fmt --all -- --check`, and
  `git diff --check` pass.
- Milestone-owned registry files comply with the workspace 400-line cap. The
  repository-wide line-cap audit still reports pre-existing, separately owned
  backlog outside this cutover and is not represented as Milestone 9.12 debt.

## Store Dependency

This milestone is not blocked on `worth-store`. Store-backed source admission,
durable identity reload, persisted inspection artifacts, and restart-stable
subscription or preview continuation remain Milestones 10 and 11 scope. Those
milestones must extend the sealed capability types and may not restore raw
constructors or parallel lifecycle entrypoints.

## Sequencing Notes

This milestone belongs immediately after Milestone 9.11 because 9.11 proves
the canonical downstream authority product while 9.12 removes remaining Query
facade paths capable of bypassing that product or recreating authority by
representation. It belongs before Milestone 10 because store-backed execution
must inherit one sealed runtime-backed authority surface rather than multiply
legacy and canonical paths across two substrates.

Phases are intentionally ordered. The inventory freezes the problem; identity
minting and basis admission close the foundational authority edges; raw intent
admission closes the generic bypass; subscription, inspection, and preview
consume the sealed basis proofs; facade contraction follows real lifecycle
closure; consumer deletion and hostile certification close last.
