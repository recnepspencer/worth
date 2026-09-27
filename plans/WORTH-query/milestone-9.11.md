# Milestone 9.11: Declarative Downstream Basis Authority And Consumer DX

Status:
Closed on 2026-07-12. The implementation and verification record is
[milestone-9.11-closeout.md](./milestone-9.11-closeout.md).

## Goal

Make Query produce one canonical, proof-bearing downstream authority artifact
that indivisibly binds the scoped Query basis, projection contract, consumption
receipt, source lineage, settlement posture, and admitted typed facts. Give
downstream runtimes a declarative, fluent surface for requesting that artifact
without learning Query's internal lifecycle choreography or reconstructing
authority from strings, digests, independently pairable receipts, or local
compatibility scans.

## Why This Milestone Exists

Milestones `9.3.2` and `9.3.4` established phase-typed basis capabilities and
declared projection consumption. Milestone `9.8` established Query-owned
consumer proof. Those foundations are individually sound, but a serious
downstream runtime must still carry and pair several separately valid artifacts
to prove one consumption event. Worth UI exposed the consequence: typed pieces
could be copied, reduced to evidence projections, paired across bases, or
restamped by a consumer even though each local API appeared reasonable.

Milestone `9.10` makes declarative graph access safer and easier than manual
graph/index folklore. This milestone applies the same product standard to basis
and projection authority before store-backed execution multiplies the number of
sources, reload postures, and downstream consumers that must preserve it.

## Governing Summaries

- `MENTALITY.md` protects production architecture from MVP-shaped shortcuts.
  The milestone must make cross-basis recombination unrepresentable, build the
  canonical artifact before convenience APIs, and certify architecture rather
  than merely demonstrate behavior.
- `arch_laws.md` protects autonomous authority, exact identity boundaries, and
  phase-typed observation. Query must preserve identity authority instead of
  accepting labels or digests as latent proof, and every boundary must expose
  operation-owned counters.
- `composition_laws.md` protects navigable, single-purpose implementation
  structure. Lifecycle, authority product, declaration DX, projection,
  inspection, and certification must remain separately named neighborhoods;
  no generic consumer helper pile may own semantic decisions.
- `domain_structure_laws.md` protects domain vocabulary and ownership-shaped
  topology. Types and modules must say whether they own intent, admission,
  authority, derived evidence, denial, or consumption rather than hiding these
  distinctions behind generic basis utilities.
- `perf_laws.md` protects bounded hot paths through explicit cost models and
  exact counters. Admission must be proportional to the declared contract and
  consumed fact width, never to unrelated workspace, basis, or consumer state.
- `WORTH_query_roadmap.md` protects Query as the ordinary platform entry and
  forbids downstream pseudo-Query layers. This milestone closes the remaining
  gap between basis/projection primitives and a product-grade downstream
  authority handoff before Milestone `10` adds store-backed sources.

## Adversarial Constraint

Given two Query evaluations whose labels, rendered digests, fact values, target
identities, or projection shapes can collide while their basis generation,
source lineage, contract, settlement, or consumption receipt differs, no
downstream crate may construct, clone into independent authority pieces, or
successfully admit a hybrid artifact. The ordinary declaration-to-authority
path must remain deterministic and bounded by `O(declared_requirements +
consumed_facts)` regardless of unrelated workspace size, historical basis
count, or consumer graph size. Evidence and inspection projections must never
be promotable back into operational authority.

## Product Decision Lock

- The canonical product is a Query-owned, sealed, move-only downstream
  authority artifact. It is not a tuple, DTO, digest bundle, or public builder
  target.
- Consumers declare required meaning; they do not choreograph normalize,
  eligibility, admission, scoping, projection binding, and receipt matching.
- Query performs all compatibility and settlement decisions before producing
  the artifact. Consumers may narrow it through typed ports but may not reopen
  or restamp Query authority.
- The fluent DX and the explicit phase API are two views over one transition
  implementation and one denial taxonomy. Convenience may not create a second
  semantic path.
- `basis_lifecycle` remains the core phase model. The overlapping
  `query_basis_lifecycle` public story must converge behind one curated facade;
  compatibility aliases may exist only during the milestone and must be
  deleted before closeout.
- Worth UI is the reference demanding consumer. The milestone does not close
  until its Query binding path consumes the canonical artifact and deletes
  local reconstruction folklore.

## Phase Plan

### Phase 1: Downstream Authority Closure Contract

Freeze the exact semantic relationship the milestone will make mechanical.
Inventory every Query artifact a downstream runtime currently has to pair,
every public or crate-visible constructor that can mint equivalent-looking
authority, and every consumer-side reconstruction seam. Produce one closure
contract and residue registry before introducing the replacement type.

**Relevant subsystems**

- `basis_lifecycle`, `query_basis_lifecycle`, and facade exports
- `projection_consumption` contracts, receipts, sources, facts, and identity
- Consumer Kit boundary audit, prohibition registry, and residue audit
- downstream adapters in Worth UI Query Binding

**Relevant APIs**

- `AdmittedBasisCapability`, scoped basis proofs, and `BasisUseReceipt`
- `MaterializedProjectionContract`, `ProjectionConsumptionReceipt`, and
  `ConsumedProjectionFactSet`
- Query evidence identity and inspection projection surfaces
- public and crate-visible constructors for basis-, source-, and receipt-like
  artifacts

**Warnings**

- Do not define the new artifact as the union of whatever fields current
  consumers happen to copy. Start from authority and compatibility law.
- A typed scalar, newtype digest, or private field is not sufficient when the
  enclosing artifact can still be independently paired with a foreign basis.
- Inventory derived inspection projections separately from operational
  authority; identical representations do not imply identical trust.

**Test requirements**

- Seed two evaluation paths with equal rendered labels and equal fact values
  but different basis/source/receipt lineage; prove the closure contract marks
  every hybrid pairing invalid.
- Run a residue audit over Query and Worth UI that identifies every raw string,
  digest selector, public constructor, compatibility rescan, tuple handoff,
  and independently pairable basis/receipt surface in scope.
- Prove replay of the same admitted evaluation produces an equivalent closure
  contract and stable canonical requirement order.

**Engineering decisions**

- Name each required structural component and its owning subsystem.
- Define which identities require exact structural equality, which are ordered,
  and which are derived evidence only.
- Register the old surface inventory as deletion obligations, not optional
  migration notes.

**Open questions**

- Resolved: `WorthQueryConsumedProjectionAuthority` is the frozen public
  product name. Worth UI adoption and facade-only compile transcripts proved
  the name outside the internal lifecycle vocabulary.

### Phase 2: Canonical Consumed Projection Authority

Implement the sole Query-owned artifact that binds the exact scoped basis,
projection contract, consumption receipt, source authority, settlement posture,
fact set, structural counters, and evidence projection produced by one admitted
consumption. Construction occurs only inside the owning Query transition and
cannot be reproduced from getters.

**Relevant subsystems**

- basis scoping and use-receipt emission
- projection-consumption eligibility, extraction, and receipt transitions
- source-reference and evidence-identity ownership
- downstream authority denial and inspection projection

**Relevant APIs**

- `WorthQueryConsumedProjectionAuthority<C>` working generic product
- `ConsumedProjectionAuthorityDenial`
- borrowed, capability-shaped target/source/fact observation ports
- derived `ConsumedProjectionAuthorityEvidence` and exact operation counters

**Warnings**

- The artifact must not implement `Clone` when cloning would create two
  independently consumable authority tokens. Cloneable observation should use
  an opaque shared reference or derived evidence view.
- Do not expose `into_parts`, public field constructors, digest lookup, or a
  general conversion from evidence back to authority.
- Genericity belongs only at the declared consumer-contract boundary. Do not
  erase fact family, settlement, or basis meaning into an open bag.
- A successful artifact must retain the exact Query-owned source objects where
  they carry authority; labels and digests remain reporting projections.

**Test requirements**

- Prove cross-basis, cross-generation, cross-contract, cross-source, and
  cross-receipt substitution cannot construct the artifact, including compile-
  fail cases for public and sibling-crate callers.
- Prove two structurally different authorities remain unequal even when every
  exposed reporting digest is forced to collide in a hostile fixture.
- Prove deterministic replay yields structurally equivalent authority and
  evidence while a stale or partial settlement returns a typed denial without
  a partial authority artifact.
- Assert exact counters for zero-fact, one-fact, mixed-family, maximum admitted
  width, and early-denial cases.

**Engineering decisions**

- Authority equality is structural and Query-owned; evidence identity is a
  derived projection.
- Denial carries the admitted inputs and exact failed relationship needed for
  inspection but no constructible successor.
- The artifact owns one canonical order for declared requirements and consumed
  facts so downstream identity does not depend on map iteration.

**Open questions**

- Resolved: the authority is move-only and does not implement `Clone`. Borrowed
  observation and derived evidence support inspection without creating a
  second consumable authority token.

### Phase 3: Declarative Consumer Contract And Fluent DX

Make the correct authority path the shortest and most discoverable path.
Consumers declare the meaning they require in domain vocabulary; Query derives
the lifecycle work, verifies support and settlement, and returns one authority
or one typed denial. Preserve an explicit phase API for advanced composition,
but implement both surfaces through the same transition core.

**Relevant subsystems**

- facade DX and declaration authoring
- projection-consumption contract binding
- basis intent selection, support discovery, and scoped admission
- compile diagnostics, rustdoc examples, recipes, and AI orientation

**Relevant APIs**

- `ProjectionAuthorityContract::declare()` working entry
- contract methods such as `require_target_identity`,
  `require_source_authority`, `require_settled_consumption`,
  `require_basis_generation`, and typed fact-family requirements
- `consume_projection_authority(contract)` on supported Query result artifacts
- named presets only where they encode stable Query concepts, not consumer-
  specific policy

**Warnings**

- Do not make users select lifecycle phases, provide redundant identity fields,
  or repeat information Query can derive from the source artifact.
- Do not use a bag-shaped builder accepting arbitrary strings, generic metadata,
  or boolean flags. Methods must correspond to closed semantic requirements.
- Do not return a partially filled authority with runtime `Option` fields for
  required contract members. Missing required meaning is a typed denial.
- Fluent chaining must not hide support posture, warnings, or cost. The outcome
  must expose exact admission and extraction counters.
- Presets must expand to inspectable canonical declarations; no preset may own
  a second implementation or invisible defaults.

**Test requirements**

- Prove fluent, explicit phase, and canonical serialized declaration paths
  lower to equivalent contracts, authorities, denials, counters, and evidence.
- Compile-fail tests must reject contradictory requirements, post-seal mutation,
  authority construction from evidence, and attempts to request unsupported
  fact families without handling typed support posture.
- Golden DX tests must cover the shortest ordinary path, an advanced mixed-fact
  path, a stale-basis denial, an unsupported-family denial, and IDE-visible
  error types without requiring internal Query imports.
- Measure declaration and admission work against requirement width and fact
  width; unrelated available fact families and workspace rows must not change
  counters.
- Run an AI-agent usability trial: from facade docs alone, an agent must choose
  the authority API rather than local receipt pairing, and its result must pass
  the prohibition audit.

**Engineering decisions**

- Use typestate only where it removes invalid ordering or contradictory states;
  do not force users through ceremonial generic parameters they cannot reason
  about.
- Prefer domain verbs and result-attached methods over free-function pipelines.
- Expose one compact `explain_denial()`/inspection projection without replacing
  typed matching through the denial enum.
- Provide migration diagnostics that name the replacement contract method when
  an old decomposed surface is used during the cutover window.
- Document three layers explicitly: five-line ordinary path, declarative
  contract reference, and advanced lifecycle mechanics.

**Open questions**

- Resolved: result-attached `consume_projection_authority(...)` is the ordinary
  verb. It returns `ProjectionAuthorityOutcome`, making Query construction and
  typed non-admission explicit.

### Phase 4: Ordinary Query Integration And Facade Convergence

Wire the canonical product and declarative contract into every supported
ordinary source: read results, write receipts, Query-context artifacts,
retained/live artifacts where admitted, and later store-facing extension
points. Converge the overlapping lifecycle vocabularies behind one facade and
one transition implementation.

**Relevant subsystems**

- Query facade exports and workspace result surfaces
- basis and query-basis lifecycle adapters
- projection consumption extraction families
- retained/live artifact consumption and support matrix

**Relevant APIs**

- result-attached `consume_projection_authority` entry points
- curated facade exports for basis intent, authority contracts, outcomes, and
  inspection
- compatibility adapters from currently admitted lifecycle sources
- support rows for every source/fact/settlement combination

**Warnings**

- Do not create one authority implementation per source artifact. Source
  extraction may vary; admission and canonical construction may not.
- Do not declare every visible source admitted. Deferred and unsupported
  neighbors must remain typed and fail before construction.
- Do not retain two public `AdmittedBasisCapability` mental models with
  ambiguous import paths.
- Existing lower-runtime authority must remain owned by its source runtime;
  Query binds its receipt rather than copying or promoting it.

**Test requirements**

- For every admitted source, prove the fluent and explicit paths produce
  equivalent canonical authority and that replay is source-order independent
  where the contract declares order irrelevant.
- Prove unsupported retained, temporal, async, store-backed, and durable
  neighbors fail through their named support posture without fallback to raw
  facts or old lifecycle adapters.
- Run facade-only compile tests demonstrating ordinary use without internal
  module imports, plus negative tests for old ambiguous imports after removal.
- Assert source extraction and authority admission counters remain additive and
  do not rescan the basis or fact set at each adapter boundary.

**Engineering decisions**

- One internal transition owns normalization through authority construction;
  adapters contribute typed source witnesses only.
- Public docs and autocomplete lead with the result-attached declarative path;
  lifecycle internals remain available only where they are an intentional
  advanced surface.
- Support reports identify authority-product availability independently from
  raw projection-consumption availability.

**Open questions**

- Resolved: the curated facade retains the declarative contract, outcome,
  sealed authority, denials, support, inspection, counters, and certification.
  Independently pairable completion and attempt types are absent from the
  facade and covered by compile-fail tests.

### Phase 5: Consumer Enforcement And Worth UI Cutover

Turn Consumer Kit into the mechanical adoption boundary and migrate Worth UI
through the real public facade. Worth UI must receive one Query authority,
combine it once with graph/host authority at its own owning boundary, and
delete every local mechanism that reopens Query basis meaning.

**Relevant subsystems**

- Consumer Kit prohibition registry, boundary audit, evidence reporting, test
  backend, and consumer residue audit
- `worth-ui-query-binding`
- Worth UI measurement basis, graph constraint admission, scroll ownership,
  portal anchoring, activation, and receipt publication
- cross-crate compile-fail and architectural certification harnesses

**Relevant APIs**

- consumer declaration of required Query authority families
- Query-owned adoption manifest and residue report
- Worth UI graph admission consuming the sealed Query authority
- consumer-facing test fixtures minted only through a real in-memory Query
  workspace

**Warnings**

- Do not wrap the new artifact in a Worth UI mirror type that copies its fields.
  Worth UI may retain it opaquely or consume typed ports.
- Do not preserve old digest/string selectors as deprecated fallbacks. A
  compatibility route that can mint authority defeats the milestone.
- The graph/host/Query aggregate is Worth UI authority, but its Query member
  remains the exact Query-owned artifact; Worth UI cannot restamp it.
- Test support may not construct Query authority directly or bypass Query
  settlement to make fixtures convenient.

**Test requirements**

- Prove Worth UI rejects equal-text foreign Query authority, cross-generation
  basis substitution, stale/partial settlement, contradictory host/Query
  ownership, and missing required facts before graph planning or receipt
  publication.
- Prove the ordinary Query -> measurement -> graph -> scroll/portal ->
  activation path preserves the exact authority and produces deterministic
  locality, counters, evidence, and receipt identity across replay.
- Compile-fail tests must reject consumer construction, field extraction for
  reconstruction, digest promotion, decomposed basis arguments, and test-only
  authority minting.
- Residue audits must report zero local Query basis scans, raw source keys,
  copied receipt identities, consumer-owned compatibility checks, or direct
  internal Query imports.
- Under unrelated Query workspace and Worth UI graph growth, admission work
  remains bounded by the declared authority/fact width plus the selected graph
  neighborhood, with exact separate counters.

**Engineering decisions**

- Worth UI cutover is part of product acceptance, not an example after Query is
  declared complete.
- Consumer Kit owns the generic prohibition vocabulary; Worth UI contributes
  its source inventory and domain-specific forbidden seams declaratively.
- Cross-crate failures retain the Query denial and add Worth UI phase context
  without flattening either taxonomy into strings.

**Open questions**

- Resolved: Worth UI remains the demanding reference consumer, while
  facade-only golden transcripts and the public-bridge reader lane provide the
  consumer-neutral shape proof without adding a synthetic second runtime.

### Phase 6: Architectural Certification And Legacy Deletion

Close the milestone by proving the new path under hostile composition and
removing every authority-capable predecessor. Certification must distinguish
semantic authority, derived inspection, support posture, bounded work, DX
quality, and consumer adoption rather than compressing closure into passing
unit tests.

**Relevant subsystems**

- Query certification and support matrix
- Consumer Kit adoption/residue certification
- facade documentation, recipes, AI orientation, and public API audits
- Worth UI end-to-end certification

**Relevant APIs**

- `ConsumedProjectionAuthorityCertificationBundle`
- authority-product support and closure rows
- DX transcript inventory and compile-fail manifest
- consumer adoption and deletion receipts

**Warnings**

- Deprecation is not deletion when an old API can still create or pair
  operational authority.
- Passing behavioral tests does not prove that evidence cannot be promoted,
  that two lifecycle paths cannot drift, or that a consumer cannot bypass the
  facade.
- Documentation must not teach the explicit lifecycle choreography as the
  ordinary path merely because it is architecturally interesting.

**Test requirements**

- Run a hostile matrix spanning source families, basis families, generation
  drift, equal-looking collisions, partial and denied settlements, replay,
  retained/live support posture, and cross-consumer transfer; every case must
  produce one authority or one typed denial with no partial successor.
- Prove deletion with zero-match source audits and compile failures for every
  Phase 1 legacy obligation, including old facade exports and compatibility
  aliases.
- Prove fluent DX, explicit phase DX, docs examples, and AI-generated usage all
  reach the same canonical transition and certification identity.
- Assert exact complexity counters and slopes across requirement width, fact
  width, unrelated workspace growth, historical basis growth, and consumer
  graph growth.
- Run the complete Query and Worth UI suites, facade/public-boundary audits,
  line-cap checks, formatting, deterministic replay, and documentation/API
  agreement checks.

**Engineering decisions**

- Closure requires zero authority-capable legacy seams. Reporting-only legacy
  projections may remain only when named honestly and mechanically unable to
  re-enter authority.
- The support matrix gains a distinct row for declarative downstream authority,
  not a loose amendment to projection-consumption support.
- The final documentation order is ordinary fluent path, contract reference,
  denial/inspection, advanced lifecycle, then migration history.

**Open questions**

- None.

## Implementation Closeout

All six phases are closed. The shipped surface includes:

- the sealed, move-only `WorthQueryConsumedProjectionAuthority`
- `ProjectionAuthorityContract`, typed `ProjectionAuthorityOutcome`, and
  fail-closed terminal JSON contract replay
- one canonical transition shared by fluent and explicit lifecycle paths
- a distinct authority-product support matrix and five-axis exact complexity
  evidence
- Consumer Kit residue classification, Worth UI adoption proof, and a sealed
  four-obligation deletion receipt bound to the audited source inventory
- compile-fail removal of independently pairable legacy types from the curated
  facade
- ordinary-first feature docs, a downstream-runtime recipe, AI orientation,
  and executable documentation/API agreement checks

The authoritative artifact and verification inventory is recorded in
[milestone-9.11-closeout.md](./milestone-9.11-closeout.md).

## Must Ship

- one sealed Query-owned consumed-projection authority artifact
- one closed declarative consumer contract and typed denial family
- result-attached fluent DX backed by the same transition as the explicit API
- one curated facade story for basis and downstream authority
- Query-owned inspection, counters, support posture, and certification
- Consumer Kit prohibition, adoption, residue, and compile-fail proof
- complete Worth UI adoption with local Query-authority reconstruction deleted

## Must Preserve

- Query owns Query basis, projection admission, consumption, and source lineage
- relational, bridge, signal, store, and consumer runtimes retain their own
  truth and execution authority
- evidence, labels, and digests remain derived and non-promotable
- unsupported and deferred source families fail closed through support posture
- explicit lifecycle APIs remain semantically identical to fluent DX where
  retained
- hot-path cost remains visible, exact, and bounded

## Acceptance Evidence

- cross-basis hybrid authority is unrepresentable or uncompilable
- every admitted source produces exactly one canonical authority artifact
- every failed relationship produces one typed denial and no partial successor
- fluent, explicit, replayed, and serialized declaration paths are equivalent
- Worth UI passes ordinary scroll and portal allocation through the artifact
  with exact source authority and no local basis reconstruction
- Consumer Kit and source residue audits report zero authority-capable legacy
  seams
- exact complexity proofs hold under unrelated Query and consumer growth
- facade docs, recipes, AI orientation, support matrix, compile-fail manifest,
  and implementation agree

## Sequencing Notes

This milestone belongs after `9.10` because Query has already established the
standard that declarative admission must replace consumer-owned access
folklore. It belongs before `10` because store-backed execution, historical
parity, and later durable reload would otherwise multiply the same decomposed
basis handoff across additional sources and persistence postures.

Phases are intentionally ordered. Phase 1 freezes the closure relationship;
Phase 2 makes it canonical; Phase 3 makes it pleasant; Phase 4 makes it
ordinary; Phase 5 proves it survives a demanding consumer; Phase 6 deletes the
old architecture and certifies the product. DX work may prototype during Phase
2, but no public fluent surface freezes before the canonical artifact and
denial taxonomy exist.
