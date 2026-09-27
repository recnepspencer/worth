# Milestone 9.3.8 Engineering Spec: Query-As-Beginning Platform Entry

> **Status:** Shipped through Phase 35 on 2026-05-27; reopened for planned
> collaboration-entry extension Phases 36 through 41
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.7.md](./milestone-9.3.7.md)
>
> **Prior closeout:** [milestone-9.3.7-closeout.md](./milestone-9.3.7-closeout.md)

## Goal

Make `worth-query` the true first-class platform entry for serious downstream
domain work so a domain can enter WORTH once through Query and then progress
through declaration, legality, proof-bearing admission, foundational
description, authority routing, preparation, continuation, inspection, and
certification without rebuilding a local pseudo-Query seam above
`worth-relational`, `worth-runtime-bridge`, `worth-signal`, `worth-proof`, or
`worth-foundational`.

## Adversarial Constraint

A geometry-kernel-grade domain must be able to declare serious domain meaning,
prove declaration legality, carry provenance/support/explanation posture, and
prepare for later lower-authority routing through one Query-owned lifecycle.
Alternate authoring paths, alternate diagnostics richness, and alternate
checked/proof lanes must converge to the same canonical declaration meaning,
while wrong domain families, wrong handles, wrong verbs, and illegal
progression must fail before later routing, continuation, or runtime contact
can occur.

## Governing Summaries

- `MENTALITY.md`: the hard problem is the missing platform-entry boundary, not
  a missing helper. The first eight phases must build the boundary law before
  any convenience lowering.
- `arch_laws.md`: each phase must own a real artifact boundary and make the
  wrong thing unrepresentable, uncompilable, or explicitly denied.
- `composition_laws.md`: entry DX, canonical artifact formation, legality,
  proof progression, and foundational description are separate responsibilities
  and must stay separate in both the spec and tree shape.
- `domain_structure_laws.md`: Query-owned entry modules must remain visibly
  distinct from later relational-, bridge-, and signal-backed routing modules.
- `perf_laws.md`: declaration entry may not hide repeated canonicalization,
  repeated digest derivation, or repeated eligibility reconstruction behind
  cheap-looking common-lane APIs.
- `worth_query_vision.md`: Query is the daily-driver import and must own the
  public grammar even when lower crates remain authoritative.
- `worth_query_roadmap.md`: `9.3.8` now stands in the critical path before the
  runtime stabilization gate, so its early phases must be stable enough to
  anchor later routing and continuation work.
- `test-requirements.md`: every early phase needs typed denial lanes, parity
  lanes, compile-fail boundaries, and canonical bundle assertions rather than
  only happy-path examples.
- `milestone-9.3.6.md`: Query should reuse the route-plan / receipt / envelope
  lifecycle discipline even before actual lower-authority routing begins.

## Phase Authoring Rules

Every phase in this milestone is a boundary spec, not an implementation note.
That means each phase must carry:

- required artifacts
- locked references
- requirements
- compile-time enforcement
- acceptance evidence
- a `DX target` section

Documentation is not automatically a final-phase concern. Instead:

- if a phase changes or introduces a discoverable public surface, that phase
  must carry a `Documentation obligation`
- if a phase is purely internal and does not yet change the discoverable public
  product, it does not need phase-local docs work
- the final docs phase exists to close coverage, goldens, and teaching
  completeness, not to defer all documentation thought until the end

The same rule applies to DX:

- every phase must state the user-facing DX target for the boundary it closes
- later ergonomics phases may improve the experience substantially, but earlier
  phases are still required to describe the intended feel of the surface they
  introduce

## Decisions Locked Now

The following are not phase-local implementation details. They are milestone
shape decisions and must not be rediscovered later:

- `9.3.8` is one end-to-end platform-entry milestone, not three separate
  declaration/preparation/runtime mini-products.
- the ordinary public import path remains `worth_query::facade`; lower crates
  stay authoritative but are not the daily-driver entry surface.
- the primary ordinary lane must be typed and domain-first. Raw string domain
  ids may exist as compatibility or lower lanes, but they are not allowed to be
  the main public entry model.
- hidden ambient context may not creep back in through convenience APIs.
  Domain configuration, route intent, truth context, continuation mode,
  grouped declaration semantics, support/readiness queries, and recovery paths
  must all have typed input families underneath any ordinary-lane sugar.
- one canonical declaration identity must exist, and it must derive from
  foundational canonicalization rather than host-local hashing or lane-local
  builder shape.
- configured domain handles own stable admitted operating context only.
  Dynamic operation eligibility, exact preview/historical/runtime basis
  binding, and declaration-specific meaning must remain later-phase concerns.
- configured domain handle identity must include all typed stable operating
  context that changes the admitted world; equivalent contexts must converge
  canonically and materially different admitted worlds must diverge.
- one admitted declaration must yield one Query-owned route plan. Multiple
  lower-authority consequences are only allowed when the retained declaration
  family taxonomy explicitly admits mixed-authority or a later richer
  route-multiplicity posture; accidental widening is forbidden.
- caller-owned runtime builder assembly and caller-owned basis/preflight/
  historical-binding choreography are in-scope defects for this milestone, not
  later polish work.
- every admitted route posture that reaches the receipt boundary must yield one
  Query-owned boundary receipt artifact, and every covered crossing must yield
  one Query-owned boundary envelope.
- `9.3.8` platform-entry lifecycle and `9.3.7`
  domain-capability-contribution lifecycle are sequential phases of one Query
  pipeline. Admitted declarations, route plans, and boundary envelopes
  produced by `9.3.8` must be valid binding targets for `9.3.7`
  contribution authoring without forcing the caller through a second public
  entry grammar or adapter-shaped conversion story.
- the lane hierarchy is mandatory: ordinary -> checked -> proof -> raw. No
  phase may collapse directly from ordinary into raw internals.
- support matrix breadth, crossing inventory breadth, docs coverage breadth,
  and certification bundle breadth must converge on the same live surface.
- relational remains the authority for truth semantics, bridge remains the
  authority for continuation/coordination semantics, signal remains the
  authority for derived execution semantics, `worth-proof` remains the owner of
  progression law, and `worth-foundational` remains the owner of shared
  boundary vocabulary.
- neighborhood and batch declaration surfaces are semantic grouping surfaces,
  not convenience `Vec<T>` overloads.

## Questions That May Stay Open Temporarily

The following may be decided phase-by-phase as long as they do not violate the
locked decisions above:

- exact Rust type names, trait names, and module names
- exact ordinary-lane verbs for each helper family
- whether a given internal phase needs one file or several modules
- the exact formatting of inspection/readiness reports
- the exact set of representative family examples used in docs and goldens
- whether adjacent phases should merge or split after hostile implementation QA,
  so long as the underlying boundary truths stay intact

## Phase Plan

1. **Phase 1: Public Domain Entry Boundary**
   Query gets one first-class typed domain entry surface. This is the moment domains stop starting outside the platform.

2. **Phase 2: Configured Domain Handle As Admitted Operating Context Boundary**
   Typed configured domain handles, their stable operating-context contract, and their validation/admission rules become explicit and sealed. This prevents stringly or bag-shaped domain entry while freezing one admitted operating world that later declaration work can honestly depend on.

3. **Phase 3: Canonical Declaration Artifact Boundary**
   Every domain declaration entering Query canonicalizes into one authoritative declaration artifact family. This is the "one meaning, one identity" boundary.

4. **Phase 4: Declaration Family Taxonomy Boundary**
   Query freezes the typed declaration family taxonomy: downstream semantic
   family identity plus Query-owned primary authority posture and modifiers.
   This stops generic-bag drift and prevents later phases from rediscovering
   family meaning from labels or payload folklore.

5. **Phase 5: Compile-Time Capability Matrix Boundary**
   Query freezes the hybrid family capability boundary: structural witness
   availability becomes compile-time wherever posture is type-known, while
   family admission that still depends on support/config posture remains a
   typed checked-lane outcome. This is where wrong witness surfaces disappear,
   wrong families and wrong handles fail honestly, and later phases stop
   pretending runtime support facts are compile-time truths.

6. **Phase 6: Declaration Legality Boundary**
   Query proves the declaration is structurally legal inside an already
   admitted operating world and for an already capability-admitted family.
   This is the declaration-side equivalent of Query's existing legality gates.

7. **Phase 7: Proof-Bearing Declaration Progression Boundary**
   Request, review, legality-cleared eligibility, admission, stale, rebind,
   denial, and stronger forms become a real `worth-proof` progression. This
   is where declaration entry stops being "typed helpers" and becomes a real
   proof chain over already retained family capability posture.

8. **Phase 8: Foundational Description Boundary**
   Provenance, support, explanation, receipts, reports, summaries, artifacts,
   and equivalence vocabulary become first-class through
   `worth-foundational`. This is where declaration entry gains a shared
   descriptive language over declaration identity, family capability posture,
   retained progression truth, and retained admitted-world identity instead of
   local dialects.

9. **Phase 9: Query Route-Plan Boundary**
   Query owns an explicit route plan for every admitted declaration. This is
   where the platform decides which lower authority families are actually in
   play by consuming retained family taxonomy and capability posture rather
   than reopening family meaning from labels or payload heuristics.

10. **Phase 10: Query Boundary-Receipt Boundary**
    Query records route-backed crossing posture as a Query-owned boundary
    receipt artifact. This is where declaration lowering becomes inspectable
    and certifiable as a public event, with retained progression posture,
    retained admitted-world identity, and deferred/denied/failed crossing
    posture carried forward instead of hidden as internal implementation
    detail.

11. **Phase 11: Query Boundary-Envelope Boundary**
    Query wraps route plan, receipt, foundational evidence, and denial topology into one self-describing declaration boundary envelope. This is the public crossing artifact boundary.

12. **Phase 12: Relational Truth-Routing Boundary**
    Query can now lower declaration families that are really about authoritative truth into relational-owned surfaces. This is where identity, lineage, invariants, history, merge/strategy truth become first-class routed declaration consequences.

13. **Phase 13: Bridge Continuation-Routing Boundary**
    Query can now lower declaration families that are really about continuation/coordination into bridge-owned surfaces. This is where preview, truth-view, basis, subscription, writeback, and cross-runtime continuity become first-class routed consequences.

14. **Phase 14: Signal Compatibility Boundary**
    Query freezes the exact continuation contract for later derived execution without yet pretending to execute through Signal here. This preserves a clean path into invalidation/recompute/observation later without semantic reset.

15. **Phase 15: Seam Classification Boundary**
    Every declaration-entry crossing gets classified as canonical reuse, Query boundary adapter, compatibility debt, deferred neighbor, or forbidden duplicate. This is where we stop tolerating ambiguous seams.

16. **Phase 16: Concrete Crossing Inventory Boundary**
    The milestone gains an executable inventory of all covered declaration-entry crossings. This is where we stop speaking in abstract families and name the actual public/internal seams.

17. **Phase 17: Cross-Authority Inspection Boundary**
    Query gets one unified inspection surface over declaration, route plan, receipt, envelope, relational lowering, bridge lowering, and later signal compatibility posture. This is where the platform becomes understandable.

18. **Phase 18: Support And Readiness Boundary**
    Query gets a first-class support/readiness story for declaration families and their routed authority posture. This is where "is this admitted/supported/deferred/denied here?" becomes a public product surface, with progression deferral, stale, and rebind posture kept distinct from earlier admission and legality denials.

18. **Phase 18 Addendum: 9.3.7 Composition Lock**
    Query locks the declaration-entry composition law with `9.3.7` so unified
    inspection and readiness/support become one public story when
    declaration-scoped contribution evidence is present.

19. **Phase 19: Admitted-Orchestration Law Boundary**
    Query becomes the admitted orchestrator for the ordinary surface without
    becoming a shortcut around proof. This phase locks the law that the
    domain-first ordinary surface is one view over the same canonical
    pipeline.

20. **Phase 20: Orchestration Artifact Model Boundary**
    Query defines typed orchestration inputs, plans, outcomes, transcripts,
    exposure levels, and denial/refusal families. This is where orchestration
    becomes a certifiable artifact-producing subsystem rather than a bag of
    helpers.

21. **Phase 21: Public Orchestration Verb Grammar Boundary**
    Query freezes the front-door verb grammar for ordinary, checked, and
    proof-visible orchestration so the ergonomic surface stays singular instead
    of fragmenting into too many equal entrypoints.

22. **Phase 22: Canonical Sequencing Automation Boundary**
    Query specifies exactly how admitted world, declaration, legality,
    progression, foundational description, route planning, receipt issuance,
    and envelope construction may be sequenced automatically, and where that
    automation must refuse.

23. **Phase 23: Artifact Materialization And Cost Policy Boundary**
    Query locks default artifact richness, lane-visible materialization policy,
    and explicit expensive-work gates so the ordinary surface never hides
    meaningful execution cost or changes declaration truth when publication
    gets richer.

24. **Phase 24: Route / Receipt / Envelope Orchestration Boundary**
    Query automates route planning, receipt issuance, and envelope construction
    while preserving those products as first-class public artifacts rather
    than laundering them away behind orchestration.

24a. **Phase 24 Addendum: Shared Binding Continuity Extraction**
    Query extracts one shared retained target-binding substrate immediately
    after the shipped Phase 24 product lane so `9.3.7` contribution targets
    and progressed route/receipt/envelope orchestration stop living on
    parallel binding seams.

24b. **Phase 24 Addendum: Aspect Contract And Granularity Extraction**
    Query retrofits aggressive aspect-aware contracts across the already-closed
    declaration-entry, routing, compatibility, materialization, and progressed
    product boundaries so binding, scope enforcement, permission narrowing,
    invariant posture, and later continuation all inherit fine-grained
    semantic slices instead of only artifact digests and family posture.

25. **Phase 25: Typed Binding / Extractor / Resolver Boundary**
    Query broadens the already-shipped shared retained binding substrate into
    the larger extractor / resolver / capability-witness system for the new
    platform surface: typed extractors, retained-artifact resolvers,
    aspect-aware family-scoped binding contracts, and capability witnesses that
    make declarative entry paths compact without ambient DI, decorator magic,
    or hidden authority crossings.

26. **Phase 26: Denial-Preserving Ordinary Outcome Boundary**
    Typed denials, unsupported posture, deferred posture, stale/rebind
    requirements, and authority-transition constraints become first-class
    ordinary outcomes instead of escape hatches from the ergonomic surface.
    This phase now ships and future phases must project onto that one shared
    ordinary outcome family rather than inventing local ordinary vocabularies.

27. **Phase 27: Runtime / Workspace / Basis Continuation Boundary**
    Query removes caller-owned runtime builder, workspace entry, and
    basis-binding choreography for admitted supported families by compiling
    continuation onto the shared binding substrate while keeping
    prepared-vs-executed posture, truth context, and basis sensitivity honest.
    It must also project any concise continuation outcomes onto the now-shipped
    ordinary outcome layer instead of creating continuation-local terminal
    result families.

28. **Phase 28: Signal Compatibility Orchestration Boundary**
    Query composes Phase 14 signal compatibility into ordinary orchestration
    without erasing execution family, required basis families, or typed
    compatibility denials. The concise signal-facing surface must therefore
    compile onto the shipped ordinary outcome family instead of inventing a
    signal-only convenience error lane. This phase now ships as the public
    signal-facing composition seam between retained compatibility and prepared
    continuation.

29. **Phase 29: Contribution-Composed Orchestration Boundary**
    Query composes `9.3.8` declaration-entry orchestration with `9.3.7`
    contribution authoring through the same shared binding substrate so
    callers do not have to cross a second public grammar seam or a second
    binding world. It must also preserve one shared ordinary outcome story over
    entry, contribution, and composed orchestration posture. This phase now
    ships as the public declaration-plus-contribution composition seam.

30. **Phase 30: Orchestration Inventory And Transcript Boundary**
    Query synchronizes live orchestration verbs, transcripts, support/readiness
    rows, docs, goldens, and certification so the ergonomic layer remains
    structurally honest under growth. That inventory must now explicitly track
    ordinary outcomes as one shipped public layer, not just verbs and proof
    transcripts. This phase now ships as the canonical orchestration-surface
    registry and anti-drift audit boundary over the admitted orchestration
    families.

31. **Phase 31: Denial And Recovery UX Boundary**
    Typed denials, progression deferral, stale/rebind outcomes, failed
    progression, fallback guidance, and route-sensitive explanations become
    product-quality. This is where failure becomes as usable as success, but it
    must widen the shipped ordinary outcome family rather than replacing it,
    and it must register its public recovery/denial lanes against the shipped
    orchestration inventory/audit boundary instead of creating uncatalogued
    helper surfaces. This phase now ships as the public recovery boundary over
    the ordinary, checked, and proof-visible stop surfaces.

32. **Phase 32: Family-Specific Ergonomics Boundary**
    The public lane gets the domain-shaped helpers for the major declaration
    families as projections over the shared binding substrate instead of
    forcing generic entry patterns everywhere. This is where the surface
    starts feeling native to real domain work. Helper families must continue to
    return the shared ordinary outcome story on concise lanes, and this phase
    now ships as the public family-helper projection seam over the admitted
    handle.

33. **Phase 33: Neighborhood Authoring DX Boundary**
    Query supports meaningful groups of declarations as first-class authoring
    units through the same shared binding substrate. This matters a lot for
    geometry because real work often happens in local neighborhoods, not
    isolated single declarations. Group-level ordinary results must extend the
    shared ordinary outcome family rather than creating grouped-only terminal
    shortcuts, and grouped ergonomics must compose onto the now-shipped family
    helper seam instead of introducing a separate grouped-helper grammar.

34. **Phase 34: Public Documentation And Golden Teaching Boundary**
     The docs, examples, and goldens all teach the exact public path honestly.
     This is where we make sure the platform is discoverable, not just
     implemented, including one explicit binding story instead of local helper
     folklore. This phase now also ships the public-doc-coverage registry and
     anti-drift audit, so family helpers, grouped authoring, and the ordinary
     orchestration families all live under one published docs/goldens coverage
     boundary instead of being tracked by scattered page links and examples.

35. **Phase 35: Certification And Closeout Boundary**
    Compile-fail boundaries, parity tests, hostile certification,
    route/receipt/envelope digests, binding-substrate parity proofs, and
    end-to-end convergence proofs close the milestone. This is where the
    whole seam becomes production-grade rather than plausible, including
    helper-vs-generic parity and helper-family gating as closure-grade proof
    obligations rather than helper-local niceties.

36. **Phase 36: Collaborative Strategy Admission Boundary**
    Query adds one stable public strategy-admission seam over relational
    merge/commit strategy families, signal merge/invalidation/delivery
    strategy families, and bridge writeback/promotion families so serious
    downstream domains stop reopening lower-runtime registries and host-local
    compatibility glue.

37. **Phase 37: Collaborative Strategy Witness And Compatibility Boundary**
    Query adds one retained, proof-bearing collaborative strategy witness over
    admitted strategy families plus basis/authority/policy posture so later
    collaboration phases classify against one canonical retained strategy story
    instead of three loosely-related lower-runtime surfaces.

38. **Phase 38: Collaborative Strategy Inspection And Preparation Boundary**
    Query adds one strategy-facing inspection and preparation seam that can
    explain admitted collaborative strategy posture and prepare the next lower
    merge/writeback/inspection handoff without pretending to execute the lower
    authorities directly.

39. **Phase 39: Collaborative Authority Witness Boundary**
    Query adds one stable collaboration-facing witness surface over the
    already-planned preview-session, branch/workspace basis, lineage,
    tenant/policy, and mutation-evidence groundwork so downstream domains do
    not have to mint local pseudo-Query authority bags just to talk about
    collaborative posture.

40. **Phase 40: Collaborative Readmission And Conflict Classification Boundary**
    Query adds one typed readmission and conflict-classification seam over the
    retained collaborative witness so replayable, stale, rebind-required,
    authority-mismatch, merge-inspection-required, and semantic-conflict
    posture become ordinary Query-facing truth instead of host-local merge
    folklore.

41. **Phase 41: Collaborative Recovery And Inspection Boundary**
    Query adds one operator-facing recovery and inspection seam that exposes
    collaborative next-step posture over the retained witness and conflict
    classification without stealing merge, writeback, or durable workflow
    authority from later roadmap milestones.

## Shared Binding Continuity Lock

The shared retained target-binding core lands immediately after shipped
Phase 24, and Phase 25 broadens that core into the larger extractor/resolver
system. Binding continuity is therefore not a future hope; it is a locked
substrate rule from the moment Phase 24 closes.

From Phase 25 onward, every future ergonomic widening in `9.3.8` must obey
these rules:

- Query may have only one public proof-bearing binding substrate for this
  milestone family
- the `9.3.7` typed contribution target-binding family must be generalized
  into that shared substrate rather than preserved as a parallel contribution-
  only binding world
- declaration-context extraction, contribution authoring, route/receipt/
  envelope product binding, continuation preparation, family helpers, and
  grouped authoring must all compile onto that one binding substrate
- no later phase may introduce a continuation-local extractor vocabulary, a
  helper-local resolver vocabulary, or a grouped-authoring-local binding
  vocabulary
- binding explanation, denial, and authority posture must remain projections
  of the same canonical Query truth rather than local convenience summaries
- any future DX work that appears to "inject" context must still be explicit
  in the types, witnesses, and proof-bearing artifacts; there is no ambient
  dependency-injection container hiding behind the facade

If a later phase needs a new binding form, it must extend the shared
substrate. It may not fork it.

## Aspect Contract And Granularity Lock

The declaration-entry milestone also inherits a second immediate retrofit
obligation after the shipped Phase 24 and Phase 24a work: the already-closed
Phase 5-14 and Phase 23-24 boundaries must gain one explicit aspect-aware
contract story before later extractor/resolver ergonomics widen the public
binding surface.

This is not optional polishing. It protects four things the surrounding stack
already treats as first-class:

- `worth-relational` already owns aspect-filtered reads, projection
  `required_aspects()`, and aspect-sensitive historical/materialized access
- `worth-runtime-bridge` already owns explicit aspect mapping and ambiguity
  failures
- `worth-signal` already owns aspect-version reads, aspect masks, and
  produced/dependency aspect contracts
- `worth-query` runtime/computed/effect/view surfaces already use aspects for
  projection, invalidation narrowing, produced-state contracts, and
  authority-lane inspection

From this addendum onward, every future widening in `9.3.8` must therefore
obey these rules:

- declaration-entry binding may not stay artifact-digest-only when a narrower
  aspect contract is the real semantic discriminator
- route, receipt, envelope, relational-routing, bridge-routing, and
  signal-compatibility surfaces must be able to say which semantic slices they
  require, preserve, publish, or deny
- scope, permission, policy masking, and invariant enforcement may use family
  posture and admitted-world posture, but they must also be able to narrow by
  declared aspect contract where that is the real truth boundary
- cheap-looking ordinary APIs may not silently widen to broad whole-artifact
  semantics when later Query, relational, bridge, or signal layers already
  operate on narrower aspect contracts
- ambiguity resolution must prefer aspect fit, aspect coverage, and explicit
  incompatibility over folklore ordering of candidate sources
- later extractor/resolver ergonomics may automate context gathering, but they
  may not bypass aspect-sensitive denial, masking, or invariant posture

If a later phase needs finer semantic granularity, it must extend this shared
aspect contract story. It may not create local aspect vocabularies per helper,
per continuation mode, or per authority family.

## Phase Specifications

### Phase 1: Public Domain Entry Boundary

This phase establishes one Query-owned front door for serious domain work.
After it closes, a downstream domain is no longer expected to begin in a local
crate-specific declaration world and later "hand something to Query."

**Required Query artifacts**

- one ordinary-lane entry surface rooted under `worth_query::facade`
- one checked-lane entry outcome for typed denial and deferred posture
- one proof-lane root that later phases can strengthen without changing
  ordinary entry meaning
- one entry-surface support snapshot so callers can inspect what this Query
  build admits before authoring domain declarations

**Locked local surfaces**

- `worth_query::facade::WORTHQueryApplicationFacade`
- `worth_query::facade::WORTHQueryConfig`
- `worth_query::facade::WORTHQuerySupportMatrix`
- `worth_query::facade::WORTHQuerySupportReport`
- `worth_query::facade::WORTHQueryCapabilityResolution`

**Requirements**

- the ordinary lane must expose one obvious domain-first entry grammar and may
  not start from raw `String`, JSON, or map-shaped bags as the primary public
  authoring model
- Query owns the generic entry capability, progression reserve, and
  support/readiness posture; downstream crates own concrete domain identity
  through downstream-implemented marker types
- this phase may publicly commit to a generic marker trait, but it may not
  define or export Query-owned concrete domains now or later
- the public import path must stay inside `worth_query::facade`; no early
  phase may require direct imports from `worth-proof`, `worth-foundational`,
  `worth-relational`, `worth-runtime-bridge`, or `worth-signal`
- the phase must reserve named checked and proof continuations even if later
  phases define their full strengthening rules
- the public entry surface must emit support/readiness posture through Query's
  existing facade/support vocabulary rather than inventing a second readiness
  dialect

**DX target**

- the ordinary lane should feel like "enter the spatial/topology/domain world
  through Query" rather than "construct a Query substrate object"
- the first public touchpoint should make it obvious where serious domain work
  starts without making the user think about lower crates

**Documentation obligation**

- yes; this phase introduces the first discoverable platform-entry surface and
  therefore needs an explicit public entry doc and golden transcript once it
  lands

**Compile-time enforcement**

- direct construction of raw domain-entry artifacts outside Query-owned front
  doors must be impossible or crate-private
- ordinary entry must be anchored on downstream-owned typed domain markers or
  equivalent downstream-owned typed entry fronts, not unvalidated free-form
  strings as the primary lane
- `worth_query::facade` may export the generic marker contract, but it may not
  export real downstream domain nouns as if they were Query-owned concepts

**Acceptance evidence**

- golden DX transcripts for ordinary, checked, and proof entry roots
- compile-fail fixtures proving that internal entry constructors are not public
- compile-fail fixtures proving fake Query-owned concrete domain markers are not
  available from the facade
- parity assertions showing entry support/readiness inspection stays identical
  across equivalent ordinary and checked entry roots

**Open questions before implementation**

- the ordinary entry root is a method on `WORTHQueryApplicationFacade`; checked
  and proof siblings also live on the facade but do not displace the ordinary
  lane as the primary front door
- the public participation contract in this phase is a downstream-implemented
  `WORTHQueryDomainEntryMarker` trait; Query does not define concrete domain
  marker types
- checked entry posture uses a typed checked-entry family immediately rather
  than collapsing into `Result<T, E>` or waiting for raw declaration-phase
  strengthening later
- the minimum entry support/readiness snapshot in this phase is capability
  families plus config section posture and one stable digest

### Phase 2: Configured Domain Handle As Admitted Operating Context Boundary

This phase freezes the first Query-owned configured domain context that later
declaration work is allowed to depend on. The configured handle is not merely a
typed config wrapper. It is one admitted operating world composed from a
downstream-owned domain marker plus a downstream-owned operating-context input,
with Query owning the draft, validated, admitted, and checked lifecycle around
that pair.

**Required Query artifacts**

- one downstream-implemented operating-context contract for stable admitted
  regime inputs
- one Query-owned configured-handle draft family
- one Query-owned validated configured-handle family
- one Query-owned admitted configured-handle family
- one checked configured-handle denial family
- one stable digest for configured-handle identity across marker, operating
  context, and validated Query config posture

**Locked local surfaces**

- `worth_query::facade::WORTHQueryConfig`
- `worth_query::facade::ValidatedWORTHQueryConfig`
- `worth_query::facade::WORTHQueryConfigSectionFamily`
- `worth_query::facade::WORTHQueryConfigSectionResolution`
- `worth_query::facade::WORTHQueryCapabilityRegistry`
- `worth_query::facade::WORTHQuerySupportMatrix`
- `worth_query::facade::WORTHQuerySupportReport`

**Requirements**

- Query owns the configured-handle lifecycle; downstream crates own concrete
  domain markers and concrete operating-context values
- the operating-context contract in this phase must stay declarative and narrow:
  it may name required capability families, required config sections, and one
  stable context-identity digest, but it may not smuggle in host-local policy
  callbacks, invariant callbacks, or builder-side ambient state
- the configured handle must represent stable admitted operating regime only.
  Allowed content includes policy/access class identity, invariant regime
  identity, assumption/tolerance regime identity, collaborator/tenant-like
  operating class identity when it changes the admitted world, and similar
  stable regime posture
- the configured handle must not carry declaration-specific meaning,
  per-operation trigger dependency graphs, exact preview/historical/runtime
  basis binding, raw collaborator/tenant/branch ids as Query authority, or bool
  shortcut flags such as `can_edit`, `preview`, or `physics_enabled`
- every configured handle must name the Query capability families and config
  sections that materially affect its admission
- handle construction must separate draft configured handle, validated
  configured handle, admitted configured handle, and checked denial posture
- this phase may reserve later relational/bridge/signal routing posture on the
  handle, but may not yet perform lower-authority routing
- configured-handle identity must include marker identity, all typed stable
  operating-context identity, required capability families, required config
  sections, operating-context digest, and validated Query config digest
- validation must remain Query-owned and structural only. It may canonicalize
  capability families, canonicalize required config sections, verify
  capability-to-section coverage, and stabilize identity; it may not pretend to
  evaluate downstream declaration semantics
- admission must deny early when required capability families are deferred or
  unsupported, when required config sections are disabled, or when the
  operating context asks for a regime Query cannot honestly admit yet
- canonicalization, digest derivation, and admission classification must happen
  once per configured-handle lifecycle and later phases must consume the
  validated/admitted proof-bearing handle rather than rediscovering those facts
  from raw marker/context inputs

**DX target**

- handles should read like stable admitted domain contexts, not service
  locators or raw config bags
- normal users should understand "I have a configured domain handle inside one
  operating regime" without needing to learn subsystem ownership details first

**Documentation obligation**

- yes; typed handles and their operating-context posture are public and need
  explicit usage guidance

**Compile-time enforcement**

- unvalidated configured handles may not masquerade as admitted configured
  handles
- Query-owned configured-handle wrappers must be sealed against downstream
  implementation even though the marker and operating-context contracts remain
  downstream-owned and open
- raw ids, bool shortcuts, and callback-shaped operating-context shortcuts may
  not become ordinary public Query authority lanes

**Acceptance evidence**

- compile-fail coverage for illegal direct handle construction
- checked-lane tests showing unsupported capability families deny handle
  admission before declaration authoring begins
- digest parity tests showing equivalent stable operating contexts yield one
  canonical configured-handle identity while materially different admitted
  worlds diverge
- adversarial tests proving stable regime inputs do not leak through ambient
  builder-side state once the typed operating-context input family exists
- denial tests proving deferred, unsupported, and invalid-context posture stop
  before declaration authoring exists
- counter or proof tests proving capability-family canonicalization,
  config-section canonicalization, digest derivation, and admission
  classification are not repeatedly rediscovered across the configured-handle
  lifecycle

**Open questions before implementation**

- should the operating-context contract stay as one shared generic contract or
  allow family-specific extensions while preserving one canonical lowering path?
- which stable regime facts belong in configured-handle identity from day one:
  policy/access class, invariant regime, assumption/tolerance regime,
  collaborator scope class, or a narrower initial subset?
- should admitted configured handles expose partial-support posture directly, or
  only through support/readiness inspection surfaces?

### Phase 3: Canonical Declaration Artifact Boundary

This phase creates the one authoritative declaration artifact family for domain
entry. From this point onward, alternate authoring paths may exist, but they
must canonicalize into one declaration meaning.

**Required Query artifacts**

- one raw declaration input family
- one canonical declaration artifact family
- one canonical declaration identity/digest
- one declaration comparison/equivalence surface for certification lanes

**Locked external surfaces**

- `worth_foundational::facade::CanonicalBasisFrontDoor`
- `worth_foundational::facade::prepare_canonical_basis_bundle`
- `worth_foundational::facade::derive_canonical_digest`
- `worth_foundational::facade::compare_canonical_basis`
- `worth_foundational::facade::CanonicalEquivalenceBasis`

**Requirements**

- canonical declaration formation must consume an admitted configured domain
  handle plus declaration-local input; it may not restart from bare domain
  markers or bypass the admitted operating-world proof established in Phase 2
- every domain declaration admitted by Query must lower into one canonical
  basis bundle before later legality, proof progression, or foundational
  description
- declaration identity must be derived from canonical basis preparation rather
  than host-local hashing rules
- declaration identity must compose handle-rooted admitted-world identity with
  declaration-local meaning; later phases may extend the proof chain around
  that identity, but they may not redefine the configured-handle contribution
- the ordinary lane may expose domain-first authoring verbs, but all authoring
  paths must converge before they can claim authoritative declaration meaning
- no later phase may redefine declaration identity or introduce a second
  authoritative declaration AST

**DX target**

- declaration authoring should feel flexible, but declaration identity should
  feel inevitable once meaning is fixed
- users should not need to care which authoring path they took in order to
  understand the resulting declaration artifact

**Documentation obligation**

- yes; canonicalization and equivalence are public meaning rules and cannot be
  left implicit

**Compile-time enforcement**

- canonical declaration artifacts must not be publicly mintable without
  passing through Query-owned canonicalization front doors
- comparison surfaces must consume canonical declaration artifacts or declared
  basis inputs, not arbitrary downstream structs

**Acceptance evidence**

- canonical digest parity across at least two semantically equivalent builder
  paths
- inequality assertions for intentionally different declaration meaning
- parity tests proving identical declaration-local input rooted in distinct
  admitted operating worlds does not collapse when the admitted world changes
  canonical meaning
- compile-fail coverage for direct artifact minting outside canonical front
  doors

**Open questions before implementation**

- should raw declaration input and canonical declaration artifacts be distinct
  public types in ordinary lanes, or should ordinary lanes mostly hide the raw
  form behind front doors?
- does family membership live structurally inside the canonical declaration
  artifact, or alongside it as a tightly-bound paired artifact before Phase 4
  fully freezes taxonomy?
- what comparison/equivalence helpers should be public now versus reserved for
  certification and inspection surfaces later?

### Phase 4: Declaration Family Taxonomy Boundary

This phase freezes the family system so later phases can gate verbs, legality,
progression, and lower-authority routing structurally instead of by informal
branching.

**Required Query artifacts**

- one sealed declaration family taxonomy
- one mapping from family to target authority class
- one mapping from family to later neighborhood/batch eligibility posture
- one public family inspection surface

**Locked authority references**

- `worth_relational::facade` reserved as the later authority for
  truth-bearing families
- `worth_runtime_bridge::facade` reserved as the later authority for
  continuation-bearing families
- `worth_signal::facade` reserved as the later authority for derived-execution
  compatible families

**Requirements**

- at minimum, the taxonomy must distinguish descriptive-only, relational-truth,
  bridge-continuation, mixed-authority, signal-compatible, and
  neighborhood-capable families
- the family system must be public enough for inspection and certification, but
  sealed enough that downstream code cannot invent unsupported family tags
- later phases may refine a family's internal states, but may not silently
  widen a family's authority class without a spec update
- family membership must become part of canonical declaration meaning before
  legality or proof progression begins

**DX target**

- the family model should make the surface feel more intentional, not more
  bureaucratic; the user should feel that Query knows what kind of declaration
  they are making
- wrong-family paths should feel absent, not merely "discouraged"

**Documentation obligation**

- yes; family distinctions are public product semantics and need explicit docs

**Compile-time enforcement**

- verbs unavailable to a family must be absent from the surface, not merely
  denied late at runtime
- families that are intentionally unsupported for a given handle must fail
  handle-plus-family composition before later legality or proof progression

**Acceptance evidence**

- compile-fail fixtures proving unavailable verbs are not callable on the wrong
  family
- certification bundles proving family tags participate in canonical
  declaration identity
- checked-lane parity showing equivalent declarations in the same family
  converge while cross-family declarations diverge

**Open questions before implementation**

- should declaration family taxonomy be modeled as a sealed enum, sealed trait
  family, or another typed representation that still supports compile-time verb
  gating cleanly?
- which family distinctions need first-class public names immediately versus
  being internal subcategories under the required top-level taxonomy?
- how should neighborhood-capable posture relate to family identity without
  prematurely collapsing grouped semantics into family tags alone?

### Phase 5: Compile-Time Capability Matrix Boundary

This phase makes family-taxonomy-aware structural witness availability and
family-specific admission posture explicit and machine-checkable.

It is intentionally hybrid:

- structurally wrong witness surfaces become compile-time absences wherever the
  wrong combination is knowable from admitted handle type, declaration input
  type, and retained family posture tags
- family admission that still depends on support snapshot or config posture
  remains a typed checked-lane outcome instead of fake compile-time magic

**Required Query artifacts**

- one sealed capability matrix for `admitted handle x retained family taxonomy x
  witness surface`
- one checked-lane outcome family for support/config/data-dependent denials
  that remain after structural compile-time gating
- one support-matrix projection that exposes the same posture publicly

**Locked local surfaces**

- `worth_query::facade::WORTHQuerySupportMatrix`
- `worth_query::facade::WORTHQuerySupportReport`
- `worth_query::facade::WORTHQueryCapabilityRegistry`
- `worth_query::facade::WORTHQueryCapabilityResolution`

**Requirements**

- if a retained family witness surface is invalid by retained primary
  authority posture, retained modifier posture, or other type-visible family
  posture alone, it must be unrepresentable or uncompilable
- if a family admission decision depends on support snapshot or config posture,
  it must remain a typed checked-lane outcome rather than being mislabeled as
  compile-time fact
- only support/config/data-dependent, policy-dependent, or
  declaration-content-dependent failures may survive into checked-lane denial
  outcomes
- family visibility may depend on admitted operating-world proof from Phase 2;
  taxonomy is global, but public family availability need not be
- compile-time gating must consume retained canonical declaration family proof
  from Phase 4 rather than rediscovering family meaning from semantic family
  keys, payload shape, or host-local routing heuristics
- the matrix must be inspectable through Query's public support/readiness
  surfaces so ordinary users can understand why something is unavailable
- the matrix must reserve later authority-routing eligibility columns for
  relational, bridge, and signal without performing the routing yet

**DX target**

- unsupported combinations should mostly disappear from the surface before the
  user can make a mistake
- when something is unavailable, the checked lane should explain that clearly
  without making the user feel they fell into internals

**Documentation obligation**

- yes; the support/readiness story is public and needs to be taught where it
  affects public entry choices

**Compile-time enforcement**

- wrong-family-taxonomy verbs, wrong-handle verbs, and wrong-target verbs must
  have UI compile-fail coverage
- wrong operating-world verbs and wrong continuation-mode verbs must also have
  compile-fail coverage wherever the mismatch is structurally knowable
- support-matrix posture may not disagree with actual method presence on the
  public lane

**Acceptance evidence**

- compile-fail suites for illegal handle/family-taxonomy/witness combinations
- parity tests between support-matrix rows and real public method presence
- checked-lane denial tests proving only support/config/content-dependent and
  policy-dependent failures survive past structural compile-time gating

**Open questions before implementation**

- should the capability matrix be materialized as explicit Rust marker/state
  types, generated code, table-driven compile-time traits, or a hybrid?
- which capability decisions are truly compile-time knowable versus requiring a
  checked-lane runtime/config denial even after type selection?
- how should support-matrix rows map back to the capability matrix so docs and
  compile-fail fixtures stay synchronized?

### Phase 6: Declaration Legality Boundary

This phase proves the declaration is structurally legal before it can enter the
stronger proof-bearing progression. It is the declaration-entry equivalent of
Query's earlier legality gates, and it must consume retained family-taxonomy
proof plus retained Phase 5 capability posture rather than reclassifying
declarations ad hoc.

**Required Query artifacts**

- one legality input artifact family over canonical declaration artifacts
- one legality verdict family with non-binary denial causes
- one legality evidence artifact that later proof phases can consume

**Locked external surfaces**

- `worth_foundational::facade::evaluate_boundary_role_claim_legality`
- `worth_foundational::facade::evaluate_boundary_surface_disposition_legality`
- `worth_foundational::facade::FoundationalBoundaryRoleClaim`
- `worth_foundational::facade::FoundationalBoundarySurfaceDisposition`
- `worth_foundational::facade::FoundationalBoundaryDecisionCause`

**Requirements**

- legality must consume canonical declaration artifacts rooted in admitted
  configured handles; it may not evaluate declarations in a worldless vacuum
- legality must run after canonical declaration formation and family taxonomy
  freeze, but before proof-bearing admission and foundational materialization
- legality must consume the retained semantic family key plus Query-owned
  taxonomy posture from Phase 4; it may not rediscover authority class,
  grouped posture, or signal posture from payload inspection or route-local
  heuristics
- legality must consume retained Phase 5 family capability/admission posture
  instead of re-running support gating or pretending support/config denials
  belong to legality
- legality must distinguish unsupported structure, illegal authority claims,
  illegal disposition claims, and later-content-dependent denials rather than
  collapsing all failure into one generic rejection
- later phases must consume legality evidence instead of recalculating the same
  role/disposition judgments from scratch
- legality must remain Query-owned even when it delegates specific boundary
  legality vocabulary to `worth-foundational`
- legality must stop at structural legality. Dynamic per-operation eligibility,
  live truth predicates, preview/historical basis sensitivity, and runtime
  trigger conditions remain later-phase concerns even when the admitted
  operating world constrains the legal space

**DX target**

- legality should feel like an honest gate, not arbitrary framework rejection
- the user should be able to tell whether they made an illegal declaration or
  simply asked for something unsupported

**Documentation obligation**

- yes; legality versus unsupported posture is a public distinction and needs
  explicit examples

**Compile-time enforcement**

- legality evidence artifacts must not be publicly constructible without a real
  legality pass
- checked-lane outcomes must preserve typed denial variants instead of
  degrading to `Result<T, String>`

**Acceptance evidence**

- hostile legality suites with equivalent declarations across alternate builder
  paths
- typed-failure assertions for role/disposition violations
- proofs that later phases consume legality evidence rather than re-running
  structural legality ad hoc

**Open questions before implementation**

- should legality evidence be a standalone artifact family or a strengthening
  wrapper around canonical declarations?
- what is the exact split between legality denials that belong here versus
  capability/readiness denials that should stay in Phase 5 or Phase 18?
- how much foundational legality vocabulary should be re-exposed directly in
  Query types versus wrapped in Query-owned denial/readout families?

### Phase 7: Proof-Bearing Declaration Progression Boundary

This phase turns declaration entry into a real `worth-proof` progression rather
than a chain of typed helper structs.

**Required Query artifacts**

- one declaration recipe/progression family rooted in `worth-proof`
- one checked outcome family that surfaces `ProofOutcomeKind` honestly
- one strengthening path from raw declaration input to admitted declaration

**Locked external surfaces**

- `worth_proof::facade::create`
- `worth_proof::facade::recipe`
- `worth_proof::facade::sym`
- `worth_proof::facade::gate_ready`
- `worth_proof::facade::ready_now`
- `worth_proof::facade::proof_flow`
- `worth_proof::facade::ProofFlow`
- `worth_proof::facade::ProofOutcome`
- `worth_proof::facade::ProofOutcomeKind`
- `worth_proof::facade::RecipeStageKind`

**Requirements**

- the phase sequence must at minimum represent declaration request,
  legality-cleared declaration, review/eligibility, admitted declaration, and
  typed stale/rebind/denied branches where applicable
- the proof chain must begin from declarations that already retain Phase 4
  family taxonomy proof, Phase 5 family capability posture, and Phase 6
  legality evidence plus the retained explicit legality contract; progression
  may strengthen or deny that state, but it may not fall back to pre-Phase-5
  generic declaration meaning or silently rerun legality from taxonomy/capability
  folklore alone
- the proof chain must inherit admitted configured-handle identity and
  operating-world proof from Phase 2 rather than re-binding it as ambient
  metadata on later declaration stages
- Query may wrap proof surfaces for DX, but it may not replace
  `worth-proof` with a Query-local typestate imitation
- checked outcomes must preserve the next lower proof truth so callers can
  branch honestly without dropping immediately to raw proof APIs
- proof-bearing declaration artifacts must be stable inputs to later
  foundational description and route-plan phases
- progression must consume retained legality evidence as the source of
  structural-legality truth; it may not treat "capability-admitted" as if that
  already implied legality-cleared progression eligibility

**DX target**

- ordinary users should benefit from proof-backed progression without having to
  think in proof jargon
- advanced users should be able to drop to checked/proof lanes and still feel
  like they are in one coherent progression model

**Documentation obligation**

- yes; the ordinary/checked/proof relationship is a core public teaching
  concern

**Compile-time enforcement**

- stronger declaration states must be unWORTHable outside Query-owned and
  `worth-proof`-owned strengthening APIs
- out-of-order progression must be uncompilable or sealed behind typed denial
  outcomes

**Acceptance evidence**

- parity suites showing ordinary and proof-lane progression converge to the
  same admitted declaration meaning
- proof-chain parity tests proving declaration progression preserves admitted
  configured-handle identity rather than re-deriving it from raw inputs
- compile-fail tests for illegal out-of-order strengthening
- checked-lane tests proving stale, rebind, and denial variants remain typed
  and inspectable

**Open questions before implementation**

- what is the smallest proof-stage family that still captures the required
  declaration request, legality-cleared, reviewed, admitted, stale, rebind, and
  denied distinctions honestly?
- should the ordinary lane surface one main checked outcome family throughout
  progression, or should each major phase expose its own checked outcome type?
- where should phase-local convenience wrappers stop so Query does not
  accidentally imitate `worth-proof` instead of reusing it?

### Phase 8: Foundational Description Boundary

This phase gives domain entry one shared descriptive language for provenance,
support, explanation, attachments, receipts, and canonical evidence bundles.

**Required Query artifacts**

- one declaration-entry boundary evidence bundle
- one provenance surface for declaration origin and carry-forward posture
- one support surface for declaration-time support/readiness disclosure
- one receipt/report/summary distinction for the entry lifecycle
- one canonical digest derivation surface for foundational evidence bundles
- one legality-description surface for retained legality evidence, explicit
  legality contracts, and typed legality denials
- one progression-description surface for retained progression outcome truth,
  progression digests, and retained admitted-world identity
- one typed foundational-evidence input family over legality evidence, legality
  denials, admitted progression, and non-success progression outcomes
- one materialization-profile surface that preserves retained declaration truth
  while allowing descriptive-richness reduction without semantic drift

**Locked external surfaces**

- `worth_foundational::facade::FoundationalBoundaryEvidenceFrontDoor`
- `worth_foundational::facade::FoundationalBoundaryEvidenceProvenanceFrontDoor`
- `worth_foundational::facade::FoundationalBoundaryEvidenceSupportFrontDoor`
- `worth_foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
- `worth_foundational::facade::prepare_boundary_evidence_attachment_bundle_for_canonical_basis`
- `worth_foundational::facade::derive_boundary_evidence_attachment_bundle_digest`
- `worth_foundational::facade::FoundationalBoundaryEvidenceAttachmentBundle`
- `worth_foundational::facade::FoundationalBoundaryEvidenceProvenanceArtifact`
- `worth_foundational::facade::FoundationalBoundaryEvidenceSupportAttachment`
- `worth_foundational::facade::FoundationalBoundaryEvidenceReceiptKind`

**Requirements**

- declaration entry must speak in foundational receipt/report/support/provenance
  vocabulary rather than inventing a Query-local descriptive dialect
- the phase must materialize enough foundational evidence that later route
  planning, receipts, and envelopes can compose it rather than recreate it
- support/readiness disclosures emitted here must line up with Query's public
  support-matrix story from Phases 1, 2, and 5
- foundational evidence must be able to describe retained family capability
  posture and family-admission truth where those facts are part of the public
  declaration story rather than leaving later phases to rediscover them
- foundational evidence must also be able to describe retained legality
  evidence, explicit legality-contract posture, and typed legality denials
  where those facts are part of the public declaration story
- foundational evidence must also be able to describe retained progression
  truth, `ProofOutcomeKind`, progression digests, and retained admitted
  operating-world identity rather than treating legality evidence as the final
  strengthened artifact
- foundational evidence must support both legality-only descriptive lanes and
  progression-bearing descriptive lanes; later phases may not assume that all
  declaration evidence already came from admitted progression
- the ordinary descriptive lane should default to a full descriptive
  materialization profile, while checked/proof lanes may select leaner
  materialization profiles without changing the retained declaration truth being
  described
- foundational descriptive receipts produced here are not yet the Query-owned
  crossing receipts from Phase 10; later phases must compose or wrap them
  rather than aliasing them as equivalent products
- foundational evidence digests must become part of later certification bundles
  and parity suites

**DX target**

- provenance, support, and receipt language should make the system easier to
  trust rather than more abstract
- users should feel like they can explain what happened without learning a
  second private vocabulary

**Documentation obligation**

- yes; this is the first phase where the shared descriptive language becomes a
  real public contract

**Compile-time enforcement**

- foundational evidence bundles must be built through Query-owned and
  foundational front doors, not by public struct literals
- receipt/report/summary surfaces must remain distinct artifact families, not
  aliases over one bag-shaped payload

**Acceptance evidence**

- canonical digest parity for equivalent foundational evidence bundles
- compile-fail tests for direct public construction of evidence artifacts
- parity suites proving ordinary, checked, and proof declaration paths yield
  the same foundational provenance/support/receipt meaning when semantically
  equivalent
- hostile suites proving legality-only descriptive lanes diverge when admitted
  world identity changes and non-success progression outcomes do not collapse
  into admitted-progression evidence

**Open questions before implementation**

- what is the minimum foundational evidence bundle that later route plans,
  receipts, and envelopes can consume without forcing re-materialization?
- should provenance, support, and receipt/report/summary surfaces be separate
  top-level artifacts from the start or grouped behind one declaration-entry
  evidence bundle with typed views?
- how should foundational evidence digests compose with canonical declaration
  digests so later certification can compare both cleanly?

### Phase 9: Query Route-Plan Boundary

This phase introduces the first explicit crossing artifact for domain entry:
the Query-owned route plan that decides which lower-authority families are in
play for an admitted declaration and why, using admitted progression proof,
matching foundational evidence, retained declaration family taxonomy proof,
and retained Phase 5 capability posture rather than family-label folklore.

**Required Query artifacts**

- one declaration route-plan artifact family
- one route-plan input family over admitted declaration progression plus
  matching foundational evidence bundles
- one route-plan classification vocabulary for later relational, bridge,
  signal-compatible, mixed, deferred, and forbidden routes
- one route-plan digest and inspection surface
- one typed route-intent input family for declaration families whose routing
  posture is caller-meaningful rather than purely inferred
- one explicit route-set artifact family so zero, one, or many valid routes
  remain first-class public truth
- one route-plan explanation surface for why route segments were admitted and
  why deferred or denied outcomes occurred
- one typed route-plan denial-cause vocabulary rather than free-form denial
  strings

**Locked local and adjacent references**

- Query must reuse the route-plan posture established in
  [milestone-9.3.6.md](./milestone-9.3.6.md)
- route planning must consume outputs from Phase 7 admitted declarations and
  Phase 8 foundational evidence bundles
- reserved authority targets remain `worth_relational::facade`,
  `worth_runtime_bridge::facade`, and `worth_signal::facade`

**Requirements**

- every admitted declaration family taxonomy posture must lower into exactly
  one explicit Query route-plan artifact before any lower-authority receipt or
  continuation can be claimed
- route planning must start from admitted progression proof plus matching
  foundational evidence; legality-only truth, canonical declarations alone,
  and family-support posture alone must not masquerade as ordinary-lane
  route-admitted inputs
- a route plan may reference more than one lower authority family only when the
  retained declaration family taxonomy is explicitly typed as mixed-authority
  or another later route-multiplicity posture that the Query taxonomy admits by
  spec; otherwise one declaration maps to one lower-authority family or one typed
  deferred/forbidden posture
- route plans must distinguish "no lower authority yet", "relational-backed",
  "bridge-backed", "mixed-authority", "signal-compatible later", "deferred
  neighbor", and "forbidden" as separate public postures
- route planning must preserve whether continuation participation is required,
  optional, deferred, or absent whenever that distinction is part of the
  retained declaration family's typed posture; Query must not collapse those
  cases into one coarse "mixed-authority" bucket
- route planning must consume retained semantic family identity and retained
  Query taxonomy posture directly from canonical declaration artifacts, and it
  should prefer retained capability posture or typed witness-bearing forms from
  Phase 5 wherever those already make a structural route question explicit; it
  may not infer lower-authority class from family labels alone or from
  declaration payload structure
- route planning must consume retained legality/progression proof rather than
  re-run structural legality or assume taxonomy/capability posture is
  sufficient proof of route eligibility
- legality-only or non-success progression descriptive evidence from Phase 8 is
  valuable for inspection and denial UX, but it must not masquerade as
  route-admitted declaration proof
- route planning must consume admitted operating-world proof from Phase 2 plus
  admitted declaration proof from earlier phases; route intent may narrow or
  select crossings within that world, but it may not redefine the world itself
- matching handle identity, operating-context identity, declaration digest,
  progression digest, and admitted evidence class are all part of the route-
  plan trust boundary and must deny or fail closed on mismatch
- route planning must remain Query-owned orchestration, not a restatement of
  lower-crate semantics
- route plans must be derived once from admitted declaration meaning and
  foundational evidence; later phases may consume them but not re-decide them
- declarations that can continue into Query runtime must carry one explicit
  runtime-continuation preparation posture in the route plan rather than
  leaving runtime builder assembly or basis-binding choreography to caller-owned
  code
- where callers are allowed to distinguish truth-only, continuation-bearing, or
  mixed-authority intent, that distinction must travel through a typed
  route-intent input family rather than ambient method choice or comment-level
  convention
- route intent and operating context are distinct typed inputs. The former asks
  what crossing is desired from within an admitted world; the latter defines the
  admitted world itself
- route-planning denial topology must be typed and public. At minimum, wrong
  admitted world, evidence mismatch, required intent, forbidden intent,
  intent/contract conflict, no allowed routes, and forbidden route
  combinations must stay distinguishable
- stale and rebind posture remain Phase 7 concerns. Phase 9 must not reinvent
  them as new ordinary-lane route facts; mismatched retained inputs should
  deny or fail on route-plan integrity instead
- route plans must remain explicit about route multiplicity and continuation
  participation for geometry-style or other multi-seam domains whose later
  routing consequences cannot be described honestly by a single yes/no mixed
  flag alone

**DX target**

- users should be able to understand "where Query plans to send this" without
  having to know how to call lower crates directly
- route plans should make the system feel legible, not expose implementation
  clutter

**Documentation obligation**

- yes; route plans are public inspection artifacts and need concise teaching

**Compile-time enforcement**

- lower-authority receipt/envelope APIs must not accept raw declarations in
  place of route plans
- route-plan artifact construction must remain behind Query-owned front doors
- route planning must not accept raw declaration input, canonical declarations,
  or legality evidence directly on the ordinary success lane
- caller route intent must remain a typed family, not raw strings or ad hoc
  enums threaded through convenience methods

**Acceptance evidence**

- parity suites showing equivalent admitted declarations produce the same route
  plan
- inequality assertions for declarations whose retained family taxonomy or
  authority posture differs intentionally
- typed-denial tests covering route-intent and admitted-world integrity
  failures
- compile-fail coverage proving canonical declarations and legality evidence
  cannot satisfy route-planning APIs directly
- compile-fail coverage proving later receipt/envelope phases cannot skip the
  route-plan artifact
- representative continuation tests proving runtime-capable route plans encode
  enough preparation posture to remove caller-owned runtime assembly sequencing
  from downstream domain code

**Open questions before implementation**

- should route intent be a standalone typed input family, a property of certain
  declaration families, or both with one canonical lowering path?
- what is the smallest route-plan artifact shape that can still carry runtime-
  continuation preparation posture without becoming a second envelope too early?
- should continuation participation and route multiplicity remain simple public
  postures on route plans, or should later phases promote one or both into
  richer route-contract artifacts once mixed-authority domains need more than a
  coarse flag?
- how should deferred and forbidden route rows be represented so they stay
  certifiable without imitating successful lower-authority routes?

### Phase 10: Query Boundary-Receipt Boundary

This phase makes every real declaration-entry crossing produce a Query-owned
boundary receipt rather than an invisible implementation transition.

**Required Query artifacts**

- one boundary-receipt input family keyed by retained route truth
- one boundary-receipt artifact family keyed by planned, deferred, denied, or
  failed route posture
- one receipt-class and receipt-kind vocabulary
- one receipt explanation surface
- one receipt integrity/digest surface
- one checked-lane receipt outcome family plus typed receipt-boundary denials

**Locked external surfaces**

- `worth_foundational::facade::boundary_receipt_definition`
- `worth_foundational::facade::FoundationalBoundaryReceiptSurface`
- `worth_foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
- `worth_foundational::facade::FoundationalBoundaryEvidenceReceiptKind`

**Requirements**

- every admitted crossing covered by this milestone must emit one Query-owned
  receipt artifact even when the lower authority also emits its own local
  evidence
- receipt construction must start from retained route truth; raw declaration
  input, canonical declarations alone, legality evidence alone, and
  foundational evidence alone must not masquerade as ordinary-lane
  receipt-admitted inputs
- planned, deferred, denied, and failed route posture must all remain
  first-class receipt inputs and first-class public receipt outcomes rather
  than collapsing non-success crossings into silent absence
- receipts must say what crossed, why it was admitted, which route-plan row it
  followed, and which authority family it touched
- receipts must preserve route-plan explanation and typed route-denial causes
  whenever the crossing stays deferred or denied, rather than flattening Phase
  9 route reasoning into generic closeout prose
- receipts must preserve receipt-boundary-specific denial causes when the route
  plan was valid but receipt issuance was unsupported or materially mismatched;
  receipt-boundary denial must not be flattened into route denial folklore
- receipts must retain whether the crossing came from legality-cleared
  declaration evidence and which explicit legality contract or legality-denial
  posture governed the crossing boundary
- receipts must retain whether the crossing came from admitted progression,
  deferred progression, stale progression, rebind-required progression, or
  failed progression instead of flattening progression posture into generic
  success/failure folklore
- receipts must preserve retained handle identity and retained operating-context
  identity whenever those are part of the crossing's admitted-world proof
- Query boundary receipts must compose Phase 8 foundational descriptive
  receipts and evidence rather than re-materializing declaration provenance,
  support, or descriptive receipt meaning from scratch
- Query boundary receipts must remain distinct from Phase 8 foundational
  descriptive receipts even when they carry overlapping descriptive facts
- where a retained descriptive receipt already exists from Phase 8, Phase 10
  should carry it forward as descriptive basis rather than regenerating a
  second descriptive receipt story
- deferred-neighbor and forbidden crossings must still produce Query-owned
  typed receipt/denial posture rather than silent absence
- receipt kinds and classes must stay explicit and typed; covered, deferred,
  denied, and failed crossing posture may not collapse into one generic
  success/failure story
- successful receipt materialization must fail closed when the retained route
  kind is not yet supported as a successful crossing kind; unsupported
  crossing kinds must become typed receipt denial rather than accidental
  pseudo-success
- receipts must be inspectable without lower-crate spelunking
- runtime continuation receipts must record whether workspace/runtime
  preparation and current/historical basis binding were Query-owned outcomes
  rather than caller-owned choreography
- receipt digest identity must be explicit and topology-sensitive. At minimum,
  retained handle identity, retained operating-context identity, declaration
  identity, retained progression posture, retained route-plan posture,
  receipt class, receipt kind, and foundational evidence bundle identity must
  participate so equivalent retained truth converges and distinct deferred,
  denied, failed, or world-divergent crossings do not collapse

**DX target**

- every crossing should leave behind something the user can point to and say
  "this is what happened"
- receipt language should feel operational and concrete, not ceremonial

**Documentation obligation**

- yes; receipts are public-facing artifacts and need examples

**Compile-time enforcement**

- route-plan consumers that claim successful crossing must return a receipt
  artifact family, not `()` or free-form text
- receipt kinds must be typed rather than host strings
- ordinary-lane receipt APIs must not accept legality evidence or foundational
  evidence directly in place of retained route truth
- direct construction of receipt artifacts must remain sealed behind
  Query-owned front doors

**Acceptance evidence**

- parity suites showing equivalent route plans emit equivalent receipts
- typed denial assertions for denied and deferred crossing kinds
- typed denial assertions for receipt-boundary-specific unsupported or
  mismatched receipt kinds
- compile-fail coverage proving public crossing APIs cannot claim success
  without a receipt artifact
- compile-fail coverage proving route-truth-bypassing inputs cannot satisfy
  receipt APIs directly
- continuation tests proving runtime-facing receipts expose runtime preparation
  and basis-binding admission as first-class receipt facts

**Open questions before implementation**

- should boundary receipts be one unified typed family with variants or several
  tightly-related receipt families keyed by crossing class?
- how much lower-authority detail belongs in a receipt versus being deferred to
  envelopes and inspection surfaces?
- what is the minimum receipt identity needed so equivalent crossings converge
  without erasing meaningful route differences?

### Phase 11: Query Boundary-Envelope Boundary

This phase wraps retained receipt truth into one self-describing public
envelope that still preserves the retained foundational evidence and retained
route truth carried by that receipt.

**Required Query artifacts**

- one declaration boundary-envelope family
- one envelope digest surface
- one envelope inspection surface
- one envelope comparison basis for certification
- one receipt-backed public envelope lane over issued, deferred, denied, and
  failed receipt truth

**Locked external surfaces**

- envelope posture must follow the public crossing lifecycle taught by
  [milestone-9.3.6.md](./milestone-9.3.6.md)
- foundational attachments and receipts must reuse Phase 8 surfaces rather than
  introducing a second descriptive bundle format
- the public envelope lane is receipt-backed only; legality evidence,
  foundational evidence, and route plans do not satisfy the ordinary envelope
  boundary directly

**Requirements**

- envelopes must contain, at minimum: declaration identity, family posture,
  admitted-world identity, evidence-origin posture, route-plan posture when
  present, receipt posture, and foundational evidence posture
- envelopes must preserve retained legality evidence and the explicit
  legality-contract posture rather than flattening legality into one generic
  "legal/illegal" note
- envelopes must preserve whether their descriptive evidence came from a
  legality-only lane, an admitted progression lane, or a non-success
  progression lane rather than flattening Phase 8 evidence origin
- envelopes must preserve route-plan explanation and typed route-denial causes
  whenever routing did not produce a planned crossing
- envelopes must also preserve receipt explanation, receipt class/kind, and
  typed receipt-boundary denial causes whenever Phase 10 did not issue a
  covered crossing receipt
- envelopes must start from retained receipt truth on the public lane and may
  not accept raw declarations, canonical declarations, legality evidence,
  foundational evidence alone, or route plans directly as ordinary envelope
  inputs
- the envelope input family must preserve checked receipt truth as a real input
  form rather than forcing callers to destructure checked receipt outcomes
  themselves before entering Phase 11
- envelopes must become the primary public artifact for later inspection,
  diagnostics, and certification
- lower crates may remain authoritative for their own evidence, but Query must
  own the envelope that composes those facts into one public crossing story
- envelopes must remain self-describing enough to support hostile certification
  without ambient reconstruction
- runtime-capable envelopes must make runtime/workspace preparation and
  current/historical basis-binding outcomes visible without requiring
  downstream glue code to reconstruct those facts

**DX target**

- envelopes should feel like the one thing a user can inspect or pass around to
  understand the crossing
- users should not need to mentally assemble route plan + receipt + evidence by
  hand

**Documentation obligation**

- yes; envelopes are a major public artifact and need explicit docs

**Compile-time enforcement**

- later inspection/certification surfaces must consume envelopes or stronger
  derived artifacts, not ad hoc tuples
- public envelope construction must stay behind Query-owned front doors
- public Phase 11 surface area must remain honest: only denial and failure
  families that the boundary can actually produce should be exported or
  documented as public envelope artifacts

**Acceptance evidence**

- canonical digest parity for equivalent envelopes across alternate ordinary
  and proof lanes
- inequality assertions for envelopes whose route-plan or receipt posture
  differs semantically
- compile-fail coverage for direct envelope minting outside Query-owned
  boundaries
- compile-fail coverage proving route plans cannot satisfy the public envelope
  lane without receipt truth
- continuation tests proving runtime-capable envelopes can explain runtime
  preparation and basis-binding outcomes without consulting local runtime glue

**Open questions before implementation**

- should envelopes store foundational evidence by value, by digest plus lookup
  reference, or through typed attachment handles?
- how much route/receipt detail should be normalized into the envelope itself
  versus accessed through envelope-backed inspection views?
- what comparison basis should the envelope use so certification can compare
  semantically equivalent crossings across ordinary and proof lanes cleanly?

### Phase 12: Relational Truth-Routing Boundary

This phase turns relational-backed declaration families into real public
Query-mediated crossings rather than future intentions.

**Required Query artifacts**

- one envelope-backed relational-routing input family
- one Query-owned relational truth-routing artifact family
- one checked relational-routing outcome family over covered, deferred,
  denied, and failed envelope truth
- one relational support/readiness posture row set

**Locked external surfaces**

- `worth_relational::facade::history`
- `worth_relational::facade::grouped_truth`
- `worth_relational::facade::commit_strategies`
- `worth_relational::facade::runtime`
- `worth_relational::facade::bridge::RuntimeBridgeRelationalSource`

**Requirements**

- the public Phase 12 lane must start from retained envelope truth only; it
  may not accept legality evidence, foundational evidence, route plans, or
  receipts directly as ordinary relational-routing inputs
- truth-bearing declaration families must lower through Query into explicit
  relational-backed bindings rather than making callers import relational
  directly
- this phase must explicitly cover relational families for identity, lineage,
  historical truth, invariant-bearing truth, grouped/materialized truth, and
  strategy-backed truth where those meanings belong to relational authority
- mixed relational+bridge declaration families must still route in Phase 12,
  but this phase may lower only the relational slice and must preserve that
  mixed-origin posture explicitly for later bridge continuation work
- Query may compile onto relational APIs, but it may not redefine truth,
  lineage, invariant, or commit-strategy semantics locally
- relational routing must be admitted-handle-bound: retained envelope truth
  from a different admitted handle or a different admitted operating world
  must deny at the Query boundary rather than silently lowering
- relational-routing artifacts must preserve which relational authority family
  was reached, which truth claim was routed, and whether the routed posture
  was exclusive relational truth or the relational slice of mixed authority
- support/readiness rows in this phase must expose at least declaration
  family, relational truth claim, relational authority family, support
  status, and reason

**DX target**

- relational-backed declarations should still feel like Query features, not
  like accidental wrappers over relational internals
- users should gain relational power without being forced to think in
  relational-first API choreography

**Documentation obligation**

- yes; once Query exposes relational-backed families, the docs need to teach
  what truth claims those families actually make

**Compile-time enforcement**

- declaration families that are not relational-truth-bearing must not expose
  relational-routing verbs or route-plan outcomes
- declaration families whose primary authority is mixed but relational-truth-
  capable must keep the common relational-routing lane available
- ordinary relational-routing helpers must remain unavailable to families that
  are not relational-truth-capable, even if they are otherwise admitted for
  declaration progression and envelope production
- route-plan rows marked relational-backed must only lower through the named
  relational facade families, not internal crate-private shortcuts

**Acceptance evidence**

- parity suites showing equivalent relational-backed declarations converge to
  the same relational-routing digest from both the explicit and common lanes
- hostile tests proving mixed-authority families still keep the common
  relational lane while lowering only the relational slice
- hostile tests proving explicit relational routing rejects retained envelope
  truth from the wrong admitted handle or wrong admitted world
- compile-fail coverage for wrong-family relational routing
- support/readiness tests proving relational-backed posture is public and
  accurate before runtime execution

**Open questions before implementation**

- the public surface is one relational-routing family with typed truth-claim
  and authority-family submodes rather than several sibling public families
- the ordinary lane remains admitted-handle-owned and envelope-backed rather
  than exposing relational-first choreography
- mixed-authority continuation remains intentionally incomplete here; Phase 13
  must consume the retained mixed-origin posture without reopening Phase 12
  routing meaning

### Phase 13: Bridge Continuation-Routing Boundary

This phase turns bridge-continuation declaration families into one
envelope-backed, Query-mediated bridge continuation-routing boundary, and it
must consume retained taxonomy proof instead of reinterpreting declarations by
name or payload.

**Required Query artifacts**

- one bridge-routing declaration family set
- one envelope-backed bridge-routing input family spanning issued, deferred,
  denied, failed, and checked envelope truth
- one Query-owned bridge continuation-routing artifact family with checked
  `Routed`, `Deferred`, `Denied`, and `Failed` outcomes
- one bridge support/readiness posture row set
- one Query-owned runtime continuation/preparation artifact family for bridge-
  routed declarations that continue into runtime-backed workspaces or
  basis-bound reads
- one typed continuation-request family covering runtime, workspace, preview,
  truth-view, current-snapshot, and historical continuation modes
- one typed truth-context family covering current, historical, and preview
  worlds for continuation-bearing declarations

**Locked external surfaces**

- `worth_runtime_bridge::facade::RuntimeBridge`
- `worth_runtime_bridge::facade::BridgeRouteRequest`
- `worth_runtime_bridge::facade::BridgeTruthViewEvaluationRequest`
- `worth_runtime_bridge::facade::BridgeSpeculativeSessionRequest`
- `worth_runtime_bridge::facade::BridgeSpeculativePromotionRequest`
- `worth_runtime_bridge::facade::HistoricalEvaluationDeclaration`
- bridge subscription and writeback facade exports re-exported from
  `worth_runtime_bridge::facade`

**Requirements**

- continuation-bearing declaration families must lower through Query into
  explicit bridge-backed routes for preview, truth-view, basis, subscription,
  writeback, and cross-runtime continuity semantics
- Query must mediate the bridge crossing, but the bridge remains authoritative
  for continuation/coordination semantics
- the ordinary public bridge-routing lane must be envelope-backed only; raw
  declarations, canonical declarations, legality evidence, foundational
  evidence, route plans, and receipts alone may not satisfy the ordinary
  bridge-routing boundary
- bridge routing must consume the canonical declaration's retained family
  taxonomy plus retained envelope truth rather than rediscovering continuation
  class from semantic family labels or declaration payload structure
- bridge continuation artifacts must consume admitted configured-handle proof as
  prior operating-world authority, not reconstruct policy/access/invariant
  posture from ad hoc declaration metadata
- bridge continuation routing must stay admitted-handle-bound; wrong-handle or
  wrong-world envelopes must deny before lower bridge contact
- this phase must not hide bridge-sensitive differences behind one fake
  universal continuation family
- mixed-authority declaration families must remain eligible for the ordinary
  bridge lane when their bridge slice is admitted, but Phase 13 lowers only the
  bridge slice and may not reopen Phase 12 relational routing decisions
- bridge continuation artifacts must preserve whether continuation
  participation was required, optional, or deferred for the routed declaration
  family whenever Phase 9 route plans carry that distinction
- Query must provide bridge-routed continuation artifacts that replace
  caller-owned runtime builder assembly and caller-owned current/historical
  basis-binding choreography for supported runtime-capable declaration families
- current-snapshot, historical, preview, and subscription-basis continuation
  must all be expressible through Query-owned bridge continuation artifacts
  rather than downstream glue code
- continuation mode and truth-context differences must enter Query as typed
  inputs, not as ambient builder flags, optional tokens, or host-local control
  flow around a generic continuation method
- bridge-routing digests must be sensitive to retained truth context,
  continuation mode, admitted world identity, and denial topology so preview,
  historical, and current continuation do not collapse into one canonical
  posture
- bridge lowering must reuse retained basis/boundary proof once per routed
  artifact rather than repeatedly rediscovering current/historical/preview
  basis posture during request shaping

**DX target**

- bridge-backed declarations should feel like powerful continuation features
  available through Query, not like users are being pushed into bridge
  specialist mode
- the surface should make preview, truth-view, subscription, and writeback
  differences understandable without overwhelming the user
- runtime-capable bridge paths should read like "continue into
  runtime/workspace/basis" rather than "assemble backend parts and bind a basis
  context yourself"

**Documentation obligation**

- yes; these families change discoverable public behavior and need explicit
  docs with examples

**Compile-time enforcement**

- non-continuation families must not expose bridge-routing verbs
- bridge-backed routes must consume retained envelope truth rather than ambient
  caller-owned bridge setup
- runtime continuation verbs may appear only for declaration families whose
  route plans and support matrices admit Query-owned runtime preparation and
  basis-binding continuation
- truth-context and continuation-request families must gate which continuation
  verbs and bridge families are visible at compile time
- route plans and receipts alone must not satisfy the ordinary bridge-routing
  lane without envelope truth

**Acceptance evidence**

- parity suites showing equivalent bridge-backed declarations converge to the
  same bridge-routing digest and continuation posture
- typed denial assertions for unsupported continuation families
- typed denial assertions proving wrong-handle or wrong-world envelopes are
  rejected before lower bridge contact
- parity and divergence suites proving truth-context-sensitive bridge routes
  stay distinct across current, historical, and preview continuation posture
- compile-fail coverage proving direct bridge contact, route plans, and
  receipts are not the ordinary lane
- replacement tests proving representative runtime-capable domain flows no
  longer require caller-owned `WORTHQueryRuntime::builder()` assembly or
  caller-owned basis/preflight/historical binding choreography once they pass
  through Query-owned bridge continuation artifacts

**Open questions before implementation**

- if later mixed-authority or geometry-heavy families need richer route-
  multiplicity or continuation-participation contracts, should Phase 13 consume
  those as route-plan postures or promote them into first-class bridge route
  contract artifacts?
- what is the minimal Query-owned runtime continuation artifact that fully
  replaces local runtime builder and basis-binding choreography without hiding
  bridge-specific differences?

### Phase 14: Signal Compatibility Boundary

This phase freezes the declaration-to-derived-execution continuation contract
without yet claiming that `9.3.8` executes those declarations through Signal.

**Required Query artifacts**

- one signal-compatibility contract family
- one envelope-backed signal-compatibility input family
- one Query-owned signal-compatibility artifact family
- one checked compatibility outcome family with `Compatible`, `Deferred`,
  `Denied`, and `Failed` posture
- one signal-compatibility support/readiness inspection surface

**Locked external surfaces**

- `worth_signal::facade::runtime`
- `worth_signal::facade::diagnostics`
- `worth_signal::facade::history`

**Requirements**

- this phase must state which declaration families are later allowed to
  continue into signal-backed derived execution and which are not
- signal compatibility must remain a typed public contract, not a note in
  prose or a host-local convention
- the ordinary public lane must be envelope-backed only; signal compatibility
  may not start from raw declarations, canonical declarations, legality
  evidence, foundational evidence, route plans, or receipts alone
- no public API in this phase may imply that `9.3.8` already owns signal
  execution semantics
- signal compatibility must be a Query-owned artifact with its own digest and
  explanation surface rather than a boolean or a small posture enum stuffed
  into route plans or envelopes
- the retained envelope must preserve route posture, receipt posture, and
  evidence origin as input evidence to compatibility classification without
  re-running Phase 12 or Phase 13 lowering internally
- signal compatibility must remain a modifier over the declaration family's
  primary lower-authority posture rather than being reinterpreted as a peer
  authority family in later phases
- signal compatibility must reuse `basis_lifecycle::BasisFamily` vocabulary
  directly rather than inventing a second public signal truth-context grammar
- later derived-execution continuation must consume typed basis-sensitive
  compatibility truth rather than ambient execution-mode switches bolted on
  after declaration entry
- common-lane helpers must exist only for structurally signal-compatible
  declaration families; deferred and incompatible families remain support-
  visible and checked-visible instead of gaining fake ordinary success lanes
- handle-owned compatibility classification must reject wrong-handle or
  wrong-world envelopes before compatibility success can occur

**DX target**

- users should be able to tell whether a declaration can later continue into
  derived execution without mistaking that for immediate signal execution
- compatibility should feel like a real contract, not a vague future promise

**Documentation obligation**

- yes; compatibility contracts are public and should be explained where they
  affect user expectations

**Compile-time enforcement**

- families that are not signal-compatible must not claim signal continuation
  flags or methods
- the ordinary signal-compatibility lane must be unavailable from raw
  declarations, canonical declarations, legality evidence, route plans, and
  receipts
- signal-compatibility posture and witness availability must be derivable from
  typed family/capability state, not from free-form tags
- direct construction of signal-compatibility artifacts must remain private

**Acceptance evidence**

- compile-fail tests for illegal signal-compatibility claims
- parity tests showing equivalent declarations expose identical
  signal-compatibility posture
- denial tests for families intentionally excluded from signal continuation
- hostile proofs that different admitted worlds can diverge in compatibility
  digest for the same declaration meaning
- hostile proofs that wrong-handle envelopes deny before compatibility
  classification
- parity proofs that basis-family differences remain distinct in digest and
  explanation

**Locked implementation outcomes**

- signal compatibility is a richer Query-owned artifact boundary, not a simple
  route-plan or envelope posture bit
- the public lane is envelope-backed only
- basis-sensitive compatibility uses `BasisFamily` directly rather than a
  second signal-specific truth-context grammar
- signal-deferred posture remains distinct from signal-incompatible posture in
  checked and support/readiness surfaces

### Phase 15: Seam Classification Boundary

This phase forces every declaration-entry crossing to be classified explicitly
so the milestone cannot hide ambiguous or convenience-driven seams.

**Required Query artifacts**

- one seam classification enum/family
- one classification row per covered crossing
- one public inventory surface exposing those rows

**Locked classification vocabulary**

- `canonical reuse`
- `Query boundary adapter`
- `compatibility debt`
- `deferred neighbor`
- `forbidden duplicate`

**Requirements**

- every crossing covered by Phases 9 through 14 must have exactly one seam
  classification row
- classification rows must say which lower crate owns the semantics, what Query
  owns, and what exit criteria apply if the seam is debt or deferred
- classification rows for Phase 14 must distinguish retained signal
  compatibility from later actual signal execution so the seam table does not
  blur compatibility ownership with execution ownership
- no direct lower-crate path may remain uncategorized
- classification must become part of the certification and documentation story
  later in the milestone

**DX target**

- this phase is mostly internal, but its outcome should make future public docs
  and inspection read more honestly
- users should indirectly benefit from cleaner seam naming even if they never
  see the raw classification table

**Documentation obligation**

- no public feature doc required at this phase by default; classification is
  primarily internal until surfaced through later inspection/docs phases

**Compile-time enforcement**

- adding a covered crossing without classifying it must fail the inventory or
  compile-boundary checks
- forbidden-duplicate seams must not have ordinary-lane entry points

**Acceptance evidence**

- executable inventory checks proving every covered crossing has one
  classification
- compile-fail or harness-fail coverage for uncategorized seam additions
- certification parity showing classification posture aligns with actual route
  plan and receipt behavior

**Open questions before implementation**

- should seam classification live as code-level typed rows, generated metadata,
  or certification-harness fixtures that are checked against code?
- how much of the classification table should be inspectable publicly versus
  reserved as internal certification metadata?
- what is the exact exit-criteria shape for `compatibility debt` and `deferred
  neighbor` rows so they remain actionable rather than decorative?

### Phase 16: Concrete Crossing Inventory Boundary

This phase replaces abstract family talk with a named, executable inventory of
the actual public and internal crossings that `9.3.8` covers.

**Required Query artifacts**

- one crossing inventory row family
- one locked covered-crossing table
- one digest over the covered inventory for certification
- one support matrix projection over the same inventory

**Locked inventory content**

- every ordinary-lane entry surface added in Phases 1 through 15
- every checked/proof lane surface that can reach a lower-authority crossing
- every relational-backed and bridge-backed route-plan/receipt/envelope family
- every signal-compatibility contract row
- every signal-compatibility support/readiness row keyed by execution family
  and basis family

**Requirements**

- each row must name the public entrypoint, declaration family, handle posture,
  route-plan class, receipt family, envelope family, seam classification, and
  owner crate
- rows that include signal compatibility must also name execution family,
  required basis families, and whether the retained compatibility posture is
  `Compatible`, `Deferred`, `Denied`, or `Failed`
- the inventory must be executable and certification-readable rather than a
  prose appendix only
- support/readiness posture must be derivable against the same row set
- later docs and hostile certification must use this inventory as the source
  of truth for coverage

**DX target**

- this phase is mostly internal, but it should guarantee that the public story
  stays synchronized with the actual surface
- users should benefit later by not encountering undocumented or
  inconsistently-supported surfaces

**Documentation obligation**

- no standalone public doc required at this phase, but the inventory becomes a
  mandatory backing source for later docs and goldens

**Compile-time enforcement**

- inventory rows for public surfaces must stay synchronized with live APIs
- adding a new covered crossing without updating the inventory must fail the
  certification or compile-boundary harness

**Acceptance evidence**

- digest parity over the covered crossing inventory
- compile-fail or harness-fail checks for unsynchronized public surfaces
- certification bundles proving the live ordinary lane breadth matches the
  inventory exactly

**Open questions before implementation**

- should the crossing inventory be hand-authored, generated from typed route
  definitions, or assembled through a hybrid model?
- what row granularity is sufficient to keep coverage honest without exploding
  the inventory into low-value duplication?
- how should inventory rows map back to support matrix rows, docs, and golden
  transcripts so synchronization stays cheap enough to maintain?

### Phase 17: Cross-Authority Inspection Boundary

This phase makes the full platform-entry seam understandable through one
Query-owned inspection surface instead of requiring callers to inspect each
lower crate independently.

**Required Query artifacts**

- one unified declaration-entry inspection surface
- one inspection result family spanning declaration, route plan, receipt,
  envelope, relational posture, bridge posture, and signal-compatibility
  posture
- one inspection digest/evidence surface for certification

**Locked local and external surfaces**

- Query inspection should follow the public-inspection posture already taught by
  the runtime and `milestone-9.3.1.md`
- bridge-backed explanation surfaces must compose through
  `worth_runtime_bridge::facade` diagnostics exports
- signal-backed compatibility explanations must remain compatible with
  `worth_signal::facade::diagnostics`
- foundational evidence and receipt surfaces from Phase 8 and Phase 10 remain
  the descriptive substrate

**Requirements**

- one inspection call must be able to explain what was declared, what family it
  belongs to, what legality and progression posture it achieved, how it routed,
  what lower authority it touched, and what remains deferred or denied
- inspection must remain a public product surface, not a debug-only or
  certification-only artifact
- Query may compose lower-crate evidence, but the final inspection result must
  be Query-shaped and stable
- inspection must not silently erase authority distinctions in the name of one
  pretty summary
- inspection must expose route-plan explanation and typed route-denial causes
  as first-class read surfaces rather than leaving callers to reverse-engineer
  why a route set was planned or denied
- inspection must expose receipt explanation, receipt class/kind, and typed
  receipt-boundary denial causes as first-class read surfaces rather than
  flattening Phase 10 crossing posture into one generic "receipt status"
- inspection must expose signal-compatibility class, signal execution family,
  required basis families, support/readiness posture, and typed compatibility
  denial causes as first-class read surfaces rather than flattening Phase 14
  into one generic "signal ready" label
- when a declaration carries `9.3.7` domain-capability contribution artifacts
  such as explanation, support, advisory, or violation evidence, unified
  inspection must compose those as typed contribution evidence alongside the
  entry-phase crossing story rather than forcing callers to query a second
  contribution-specific inspection system

**DX target**

- inspection should feel like a superpower: one place to ask what happened and
  why
- users should be able to trust Query inspection instead of spelunking lower
  runtimes

**Documentation obligation**

- yes; inspection is a major public contract and needs strong docs and
  examples

**Compile-time enforcement**

- inspection surfaces must consume typed route/receipt/envelope artifacts, not
  free-form logs or host-assembled summaries
- ordinary-lane inspection must remain available without importing lower-crate
  inspection helpers directly

**Acceptance evidence**

- parity suites showing equivalent paths yield identical unified inspection
  results
- hostile inspection cases proving intentionally different route, receipt, or
  denial posture remains visible
- composition tests proving `9.3.7` declaration-scoped explanation and
  inspection contributions attached to a declaration are visible through the
  unified inspection surface without a separate contribution-specific
  inspection call
- compile-fail coverage proving direct lower-crate inspection types are not the
  required public lane

**Open questions before implementation**

- should unified inspection return one deeply-structured result family or a
  smaller top-level result with typed expandable views?
- how much lower-authority evidence should be embedded directly in inspection
  results versus referenced through route/receipt/envelope-backed links?
- what is the minimum inspection digest/evidence surface needed for
  certification without turning inspection into a second envelope system?

### Phase 18: Support And Readiness Boundary

This phase makes support, readiness, deferral, and denial first-class public
surfaces across the entire declaration-entry seam.

**Required Query artifacts**

- one platform-entry support matrix
- one readiness/disposition report for retained family taxonomy rows and
  crossings
- one denial/deferred residual-debt projection over the same matrix
- one typed support/readiness request family for querying capability and
  admission posture explicitly

**Locked local surfaces**

- `worth_query::facade::WORTHQuerySupportMatrix`
- `worth_query::facade::WORTHQuerySupportReport`
- `worth_query::facade::WORTHQueryCapabilityDescriptor`
- `worth_query::facade::WORTHQueryCapabilitySupportStatus`

**Requirements**

- support posture must be visible for domain entry, family-taxonomy selection,
  route planning, lower-authority routing, signal compatibility, and
  neighborhood readiness where applicable
- support/readiness must distinguish structural witness availability from
  operating-world family admission wherever those are different truths; the
  public product surface may not collapse "this witness does not exist" and
  "this family is structurally valid but currently denied here" into one vague
  unavailable posture
- support/readiness must also distinguish legality denial from family-admission
  denial wherever those are different truths; the public product surface may
  not collapse "this family can admit here" and "this admitted declaration is
  still structurally illegal here" into one generic denial posture
- support/readiness must also distinguish progression denial, progression
  deferral, stale progression, and rebind-required progression from both
  earlier family-admission denial and legality denial
- support/readiness must also expose typed route-plan denial causes and
  explicit route multiplicity instead of collapsing those into generic
  unsupported or mixed labels
- support/readiness must also distinguish route-plan denial from
  receipt-boundary denial where a route is admissible in principle but the
  current crossing kind is not yet admitted as a successful receipt kind
- support/readiness must also distinguish signal-compatible admitted posture,
  signal-deferred posture, signal-family unsupported posture, invalid-basis
  posture, and later execution denial posture rather than collapsing them into
  one flat "signal supported" bit
- support/readiness posture must also be visible for admitted operating-world
  questions: which configured regimes are supported, which continuation modes
  are supported for a given admitted world, and which truth contexts are valid
  inside that world
- the platform-entry support matrix must compose with `9.3.7`
  declaration-scoped support contributions rather than presenting a separate
  readiness vocabulary. Entry-phase support/readiness and contribution-phase
  declaration-scoped support must be accessible through one support surface
  with typed distinctions, not two unrelated support systems
- the support matrix must be the same source of truth used by ordinary DX,
  checked outcomes, docs, and certification
- support/readiness rows must be keyed by retained domain-scoped family
  identity plus Query-owned taxonomy posture where family meaning matters; the
  matrix may not fall back to raw family-label folklore
- signal-compatibility support/readiness rows must additionally be keyed by
  signal execution family and basis family where those distinctions matter
- unsupported, deferred, and forbidden surfaces must remain explicitly named
  and may not degrade into "not implemented" folklore
- support/readiness posture must be inspectable before expensive work happens
- callers must be able to ask explicit typed support/readiness questions about
  operating context, route families, continuation modes, truth contexts,
  grouped declaration semantics, and handle-vs-declaration posture rather than
  inferring readiness from trial and error

**DX target**

- support posture should help users choose viable paths early rather than
  discover dead ends late
- readiness language should feel crisp and decision-useful, not bureaucratic

**Documentation obligation**

- yes; support/readiness is a public-facing product surface and must be taught

**Compile-time enforcement**

- public surfaces advertised as ordinary-lane features must have a matching
  support/readiness row
- checked-lane denial posture must not contradict support matrix posture

**Acceptance evidence**

- parity tests between live public surfaces and the support matrix
- typed denial tests for deferred and unsupported family-taxonomy rows
- certification bundles proving support/readiness digests align with the
  crossing inventory from Phase 16

**Open questions before implementation**

- should support/readiness requests be family-specific typed inputs, one shared
  request family with typed selectors, or a hybrid?
- how much readiness detail belongs in ordinary-lane support answers versus
  checked/inspection-only surfaces?
- what is the cheapest synchronization mechanism between the live support
  matrix, compile-time gating, and documented feature coverage?

### Phase 18 Addendum: 9.3.7 Composition Lock

This addendum closes the structural composition gap between `9.3.8`
declaration-entry seam truth and the already-shipped `9.3.7`
domain-capability contribution seam.

**Required Query artifacts**

- one declaration-entry composition contract for `9.3.7` contribution evidence
- one unified inspection composition path for declaration-scoped contribution
  evidence
- one unified support/readiness composition path for declaration-scoped
  contribution posture

**Locked composition rule**

- `9.3.8` platform-entry lifecycle and `9.3.7`
  domain-capability-contribution lifecycle are sequential phases of one Query
  pipeline
- admitted declarations, route plans, receipts, envelopes, and retained seam
  artifacts produced by `9.3.8` must remain valid binding targets for
  declaration-scoped `9.3.7` contribution authoring and materialization
- entry-phase inspection/support and contribution-phase
  explanation/support/advisory/violation/workflow/continuity/aftermath posture
  must compose into one public Query story rather than two unrelated public
  systems

**Requirements**

- unified inspection from Phase 17 must compose declaration-scoped `9.3.7`
  contribution evidence when present, including explanation/inspection
  artifacts and the broader declaration-scoped contribution categories that
  materially affect what happened and why
- support/readiness from Phase 18 must compose declaration-scoped `9.3.7`
  contribution posture when present, including declaration support,
  advisory/violation posture, and broader declaration-scoped contribution
  categories that materially affect readiness truth
- this composition must be on-demand through Query-owned composition inputs or
  request surfaces; the seam ledger must not pretend it discovered domain
  contribution evidence automatically from ambient state or hidden registries
- declaration-bound contribution evidence may compose directly against retained
  declaration-entry seam subjects, but admitted-plan-bound and
  lower-runtime-bound contribution categories must require matching retained
  downstream proof explicitly rather than being inferred from declaration-entry
  posture alone
- support/readiness composition must admit a retained-subject-aware path so
  Query can fail closed on wrong-handle, wrong-world, or mismatched
  declaration-digest posture before contribution composition succeeds
- this composition must preserve the distinction between entry-phase truth and
  contribution-phase meaning; it may compose them, but it may not flatten them
  into one unlabeled summary bucket
- callers must not have to query a second contribution-specific inspection or
  support system to understand one declarationâ€™s full public story

**Acceptance evidence**

- composition tests proving declaration-scoped `9.3.7` contribution evidence is
  visible through unified inspection without a second contribution-specific
  inspection call
- composition tests proving declaration-scoped `9.3.7` support/readiness
  posture is visible through the platform-entry support matrix without a second
  support system
- hostile tests proving admitted-plan-bound and lower-runtime-bound
  contribution categories are accepted only when matching retained downstream
  proof is attached, and rejected with typed composition mismatch posture when
  that proof is absent or wrong
- hostile tests proving entry-phase denial/readiness truth and contribution-
  phase advisory/violation/support meaning remain distinct when both are
  present

### Phase 19: Admitted-Orchestration Law Boundary

This phase replaces the old "happy path" framing with the real rule: Query's
ordinary public surface is a domain-first view over the same canonical
declaration-entry pipeline, not convenience sugar around a different hidden
implementation.

**Required Query artifacts**

- one admitted-orchestration law for the ordinary public surface
- one explicit statement of the canonical pipeline shared by ordinary,
  checked, and proof-visible surfaces
- one typed refusal posture saying when automation must stop instead of guess
- one explicit lowering-boundary rule saying how far ordinary orchestration is
  admitted to progress before later phases define richer orchestration
  artifacts

**Locked local and adjacent references**

- orchestration must compile onto Phases 1 through 18 artifacts rather than
  bypassing them
- ordinary, checked, and proof-visible surfaces are multiple exposures over
  one canonical pipeline, not rival implementations
- `9.3.8` declaration-entry and `9.3.7` contribution lifecycle remain one
  sequential Query pipeline under this phase and later orchestration phases
- ordinary orchestration may shorten caller-owned choreography, but it may not
  erase route-plan, receipt, envelope, seam-ledger, or contribution meaning
  that earlier phases already froze as retained Query artifacts

**Requirements**

- the ordinary public surface must let a serious domain author establish one
  admitted operating world, state domain intent once, and have Query own every
  transition it can prove while refusing every transition it cannot prove
- orchestration may automate sequencing only where admitted operating-world
  proof, canonical declaration proof, legality evidence, progression proof,
  and later retained crossing artifacts already justify that automation
- the ordinary surface must expose typed refusal posture for at least these
  orchestration-stop classes whenever they differ semantically:
  unsupported automation, explicit-intent-required, stronger-proof-required,
  authority-transition-required, expensive-work-not-admitted-by-default, and
  prepared-but-not-executed continuation posture
- until later orchestration phases define richer execution products, ordinary
  orchestration may lower only as far as the strongest already-admitted
  retained Query artifact boundary. It may stop at declaration progression,
  foundational description, route plan, receipt, envelope, or other later
  retained seam artifacts as admitted, but it may not silently imply that
  farther continuation or execution already happened
- the ordinary surface must preserve typed denial, advisory, unsupported,
  deferred, stale, rebind-required, authority-transition, receipt, envelope,
  and transcript posture as first-class outcomes rather than escape hatches
- for the first public admitted-orchestration ceiling, ordinary success must
  return the retained declaration envelope rather than inventing a broader
  execution or continuation product early
- checked and proof-visible surfaces must remain explicit visibility levels
  over the same pipeline rather than turning the ordinary surface into the only
  real implementation path
- the first proof-visible surface may stay intentionally thin, but its stage
  records must still identify the farthest retained declaration-entry boundary
  actually crossed and the real stop stage for the semantic outcome Query
  returned
- orchestration must stay handle-bound and proof-preserving; it may not
  re-bind operating worlds, basis facts, or lower-authority facts through
  parallel helper parameters later in the flow
- orchestration may not silently cross meaningful cost, authority, or
  continuation boundaries. Expensive continuation, basis rebinding,
  lower-authority execution, and contribution-side effects must remain explicit
  unless a later orchestration phase defines an admitted typed contract for
  that exact automation

**DX target**

- the platform should feel framework-owned rather than helper-owned
- the user should feel that they stated intent once and Query assumed
  responsibility for every admitted transition without erasing the underlying
  choreography

**Documentation obligation**

- yes; this law must be taught explicitly because it governs every later
  orchestration phase

**Compile-time enforcement**

- any ordinary orchestrated surface must compile onto typed prior-phase
  artifacts rather than ad hoc host chains
- ordinary, checked, and proof-visible public surfaces must not be able to
  diverge semantically for the same admitted intent

**Acceptance evidence**

- parity suites proving ordinary, checked, and proof-visible orchestration
  converge for the same admitted declaration-entry intent
- hostile tests proving the ordinary surface refuses unsupported automation
  rather than inventing it by convenience
- proof-visible stage-record suites proving the reported stop boundary matches
  the farthest retained declaration-entry boundary actually crossed for the
  checked/ordinary semantic outcome, including route-level non-success paths
  that still lower through receipt truth before stopping

**Open questions before implementation**

- which orchestration outcomes deserve distinct public families versus shared
  outcome envelopes with typed sub-posture?

### Phase 20: Orchestration Artifact Model Boundary

This phase turns orchestration into a certifiable subsystem rather than a pile
of helper methods by giving it its own typed artifact family.

**Required Query artifacts**

- one orchestration input family
- one orchestration plan family
- one orchestration outcome family
- one orchestration transcript family
- one orchestration denial / refusal family
- one orchestration exposure-level family
- one orchestration artifact-policy family
- one orchestration step-record inventory

**Requirements**

- orchestration artifacts must be Query-owned and must reference retained
  declaration-entry truth from Phases 1 through 18 rather than reconstructing
  it from host-local helper context
- orchestration plans must record which phase transitions were admitted,
  automated, refused, deferred, denied, or left explicit for the caller
- orchestration outcomes must preserve route-plan, receipt, envelope,
  relational-routing, bridge-routing, and signal-compatibility posture where
  those artifacts were crossed
- transcripts must be first-class public/certification artifacts rather than
  debug logs, and must be capable of explaining what was automated and what was
  intentionally left explicit
- artifact policy and exposure level must be typed so ordinary surfaces,
  checked surfaces, and proof-visible surfaces can differ in visibility without
  becoming separate semantic implementations
- Phase 20 closes the envelope-ceiling artifact model only; proof-visible
  surfaces expose the transcript family in this phase while ordinary and
  checked surfaces remain transcript-free projections over the same canonical
  lowering

**Compile-time enforcement**

- orchestration plans and outcomes must be sealed so callers cannot mint fake
  orchestration truth
- transcript-bearing and non-transcript-bearing surfaces must still share the
  same canonical orchestration identity where semantics are identical

**Documentation obligation**

- yes; this phase must teach the public orchestration artifact families and how
  they differ from later richer transcript, policy, and continuation products

**Acceptance evidence**

- canonical digest parity for equivalent orchestration inputs across exposure
  levels
- hostile tests proving transcript, denial, and artifact-policy posture cannot
  drift apart for the same underlying orchestration

### Phase 21: Public Orchestration Verb Grammar Boundary

This phase freezes the front-door verb family so the public surface feels
singular instead of fragmenting into too many equal helper paths.

Phase 20 already stabilizes the generic admitted-handle front door:

- `orchestrate_declaration_entry(...)`
- `orchestrate_declaration_entry_checked(...)`
- `orchestrate_declaration_entry_proof(...)`

Phase 21 builds on that base. It should extend, specialize, or clarify the
front-door grammar from the already-shipped trio rather than casually
re-litigating whether one generic front door exists at all.

Phase 21 also freezes one public grammar inventory for that trio:

- `WORTHQueryDeclarationEntryOrchestrationVerbInventory`
- `WORTHQueryDeclarationEntryOrchestrationVerb`
- `WORTHQueryDeclarationEntryOrchestrationVerbFamily`
- `WORTHQueryDeclarationEntryOrchestrationVerbCeiling`

**Required Query artifacts**

- one ordinary public verb grammar
- one checked public verb grammar
- one proof-visible / transcript-visible verb grammar
- one public grammar inventory over the live orchestration trio

**Requirements**

- the ordinary public verbs must stay domain-first in naming and shape; they
  may not devolve into `WORTHQuery*` ceremony, builder choreography, or
  substrate-driven parameter bags
- the verb grammar must answer, structurally and explicitly, which surfaces
  return progression-only posture, route plans, receipts, envelopes,
  compatibility posture, prepared continuation, or contribution-composed
  artifacts
- the checked equivalent and proof-visible equivalent of an ordinary verb must
  be predictable and discoverable rather than invented ad hoc per family
- generic verbs and family-specific verbs must have one obvious relationship;
  family-specific surfaces may specialize domain naming, but they may not
  invent parallel semantics
- Phase 21 itself keeps the generic trio as the only public orchestration
  verb family; family-specific aliases remain a later-phase concern
- any new family-specific verb must still lower through the Phase 20 canonical
  orchestration artifact model rather than standing up a competing helper path
- alternate proof-visible suffix families such as `_transcript`, `_trace`,
  `_debug`, or `_verbose` are forbidden in this phase
- verbs that imply expensive work, runtime continuation, workspace entry, or
  basis binding must advertise that boundary in the API shape rather than
  masquerading as cheap getters

**Documentation obligation**

- yes; this phase must be taught explicitly because the public verb grammar is
  the main caller-facing map of the orchestration surface

**Acceptance evidence**

- inventory tests proving every public orchestration verb maps to exactly one
  canonical orchestration surface
- hostile naming/shape review proving there is one obvious front door per
  supported orchestration family instead of many equal entry paths

**Historical closure notes**

- how should the grammar inventory rows be synchronized mechanically with docs,
  goldens, and compile-fail boundaries before Phase 30 expands that story?

### Phase 22: Canonical Sequencing Automation Boundary

This phase specifies exactly how Query automates the declaration-entry pipeline
and exactly where it must refuse automation.

**Required Query artifacts**

- one canonical sequencing plan over the declaration-entry pipeline
- one typed automation-refusal family
- one sequencing parity surface tying explicit phase-by-phase calls to the
  orchestrated surface

This phase extends the sequencing rules that Phase 20 already froze in the
public orchestration artifact model:

- orchestration input
- orchestration plan
- orchestration outcome
- orchestration transcript

It must not introduce a second planning abstraction that competes with the
existing Phase 20 plan artifact. The work here is to enrich sequencing law and
automation/refusal posture on top of that model.

It must also preserve the Phase 21 grammar lock:

- the generic orchestration trio remains the only public sequencing front door
- Phase 22 may deepen sequencing law, but it may not widen the grammar by
  convenience

**Requirements**

- the ordinary surface may automate only the canonical sequence:
  admitted operating world -> canonical declaration -> legality ->
  progression -> foundational materialization -> route planning ->
  receipt issuance -> envelope construction -> later admitted orchestration
- the automation path must consume each retained proof exactly once and pass it
  forward rather than re-deciding phase eligibility later
- sequencing automation may not skip progression, jump around route/receipt/
  envelope boundaries, or surface a later-phase artifact without the earlier
  retained proof it depends on
- any richer sequencing metadata must remain an extension of the Phase 20
  orchestration plan/outcome model rather than a parallel execution grammar
- automation refusal must be typed and explain whether the stop came from
  unsupported surface breadth, missing proof, explicit expensive-work
  requirement, authority-transition requirement, or family-specific non-admission

**Documentation obligation**

- yes; the canonical sequencing story and its refusal boundaries must be taught
  so ordinary orchestration does not look like hidden host-local convenience

**Acceptance evidence**

- parity suites showing explicit phase-by-phase declaration-entry progression
  converges with orchestrated sequencing
- hostile tests proving unsupported or proof-insufficient shortcuts fail closed

### Phase 23: Artifact Materialization And Cost Policy Boundary

This phase makes richness and expensive work explicit so the ordinary surface
stays ergonomically strong without becoming cost-dishonest.

**Required Query artifacts**

- one orchestration artifact-richness policy family
- one explicit cost-posture family
- one prepared-vs-executed family where expensive work can be staged but not
  run implicitly

Phase 20 already freezes a different policy axis:

- `OrdinaryEnvelopeOnly`
- `CheckedOutcomeOnly`
- `ProofVisibleTranscript`

Those are visibility policies over one canonical orchestration truth. Phase 23
must add richness and cost policy as a separate axis. It must not retroactively
reinterpret the Phase 20 visibility policy as if it already expressed richness,
prepared-work posture, or expensive-execution admission.

Phase 22 now also freezes the sequencing axis:

- one envelope-ceiling automation boundary
- one canonical automation-step order
- one typed automation-refusal family

Phase 23 must therefore build on that shipped sequencing law rather than
smuggling cost policy into hidden sequencing changes.

**Requirements**

- the ordinary orchestration default should use lean foundational publication
  plus support-ready receipt and envelope publication unless a later explicit
  materialization request admits richer publication
- orchestration may prepare expensive work, but it may not silently execute
  expensive continuation, workspace entry, signal-backed execution, or other
  costly lower-runtime work without an API shape that advertises that cost
- cost policy must preserve the already-shipped Phase 22 distinction between
  ordinary typed non-success posture and automation refusal
- cost posture must distinguish at least: cheap retained-artifact assembly,
  prepared-but-not-executed continuation, explicit execution acknowledgment
  required, and unsupported-by-default expensive work
- artifact-richness policy must stay separate from semantic meaning: richer
  transcripts and richer descriptive artifacts may increase visibility but may
  not change canonical orchestration truth
- artifact-richness policy and cost posture must also stay separate from the
  Phase 20 exposure-level and visibility-policy surface
- the shipped Phase 23 default now treats ordinary, checked, and proof-visible
  lanes as the same declaration-entry truth with different visibility over one
  inspectable materialization policy; later phases may widen explicit rich
  requests, but they must not silently move the ordinary default back to
  full-descriptive publication

**Documentation obligation**

- yes; this phase must explain ordinary defaults, explicit cost gates, and the
  difference between richer visibility and changed orchestration meaning

**Acceptance evidence**

- parity tests proving richer and leaner materialization policies do not change
  orchestration truth unless semantics intentionally differ
- hostile tests proving cheap-looking ordinary verbs cannot hide expensive work

### Phase 24: Route / Receipt / Envelope Orchestration Boundary

This phase closes the last pure declaration-entry automation layer by making
route planning, receipt issuance, and envelope construction ordinary Query
orchestration products without turning them into hidden internals.

**Required Query artifacts**

- one orchestrated route-planning surface
- one orchestrated receipt surface
- one orchestrated envelope surface
- one transcript / inventory mapping from orchestrated verbs back to retained
  route, receipt, and envelope artifacts

**Requirements**

- the ordinary route-planning surface should prefer
  `plan_routes_from_progressed` and
  `declare_review_progress_describe_and_plan` semantics rather than making
  callers materialize foundational evidence or route-plan inputs unless they
  intentionally drop to a more explicit exposure level
- the ordinary receipt surface should prefer
  `receipt_routes_from_progressed`,
  `receipt_routes_from_progressed_with_intent`, and
  `declare_review_progress_describe_plan_and_receipt` semantics rather than
  making callers manually assemble receipt inputs unless they intentionally
  drop lower
- the ordinary envelope surface should preserve retained route-plan, receipt,
  evidence-origin, denial, and explanation posture explicitly; envelopes may
  be orchestrated, but they may not disappear into opaque success results
- route/receipt/envelope orchestration must remain handle-bound and must deny
  wrong-handle or wrong-world retained artifacts before later continuation or
  contribution composition can proceed
- any orchestrated route/receipt/envelope surface must project from the
  existing Phase 20 canonical orchestration artifacts rather than creating a
  fresh helper stack with separate transcript or denial logic
- any widening here must preserve the shipped Phase 22 stop-boundary honesty,
  including caller handoff at route or receipt when the public automation
  contract intentionally stops there
- any widening here must preserve the shipped Phase 23 materialization law:
  richer publication may widen descriptive breadth, but it may not change
  route, receipt, or envelope semantic truth

**Documentation obligation**

- yes; route, receipt, and envelope orchestration must be documented as public
  products rather than treated as internal sequencing details

**Closed status**

- shipped in Phase 24:
  - `orchestrate_routes_from_progressed(...)`
  - `orchestrate_receipt_from_progressed(...)`
  - `orchestrate_envelope_from_progressed(...)`
  - checked/proof-visible and explicit-intent variants on the same retained
    pipeline

**Acceptance evidence**

- parity suites proving orchestrated route/receipt/envelope products converge
  with explicit retained-artifact assembly
- hostile tests proving orchestration cannot erase route denial, receipt
  denial, or evidence-origin distinctions

### Phase 24 Addendum: Aspect Contract And Granularity Extraction

This addendum retrofits aggressive aspect-aware contract law across the
already-closed declaration-entry and product-target boundaries. Its job is not
to sprinkle aspect metadata over the same old retained-artifact story. Its job
is to make the declaration-entry pipeline speak the same semantic granularity,
masking, performance, and narrowing truth that runtime, relational, bridge,
and signal already use.

The guiding principle for every section below is:

- what do aspects need to provide here for later phases to live up to the
  highest DX promised in `worth_query_vision.md`?

That means later declaration-entry, continuation, grouped-authoring, and
binding surfaces must be able to ask Query for the current admissible artifact
that satisfies a semantic aspect contract instead of guessing from broad
artifact class, source-order folklore, or raw geometry target strings.

**Shared contract**

This addendum lands one shared declaration-entry aspect vocabulary before Phase
25 widens extractors and resolvers:

- `required`: slices that must be present for meaningful binding or progress
- `preserved`: slices carried forward unchanged by a retained artifact
- `published`: slices intentionally exposed on descriptive/public artifacts
- `masked`: slices intentionally elided or withheld by policy or publication
- `incompatible`: slices that make a candidate or next step semantically
  non-bindable

It also lands one shared fit taxonomy:

- `Exact`
- `CompatibleSuperset`
- `Partial`
- `MissingRequired`
- `Conflict`

These aspect contracts are load-bearing. They do not replace admitted-world
identity, family posture, route/receipt/envelope class, or authority lanes.
They compose with those axes so later phases can narrow semantically without
reopening the whole proof chain.

**Grounding references**

- [Declaration Family Capability Matrix](../../crates/worth-query/docs/domain-capabilities/declaration-family-capability-matrix.md)
- [Declaration Legality](../../crates/worth-query/docs/domain-capabilities/declaration-legality.md)
- [Declaration Progression](../../crates/worth-query/docs/domain-capabilities/declaration-progression.md)
- [Declaration Foundational Evidence](../../crates/worth-query/docs/domain-capabilities/declaration-foundational-evidence.md)
- [Declaration Route Plans](../../crates/worth-query/docs/domain-capabilities/declaration-route-plan.md)
- [Declaration Boundary Receipts](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-receipts.md)
- [Declaration Boundary Envelopes](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-envelopes.md)
- [Declaration Relational Truth Routing](../../crates/worth-query/docs/domain-capabilities/declaration-relational-truth-routing.md)
- [Declaration Bridge Continuation Routing](../../crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md)
- [Declaration Signal Compatibility](../../crates/worth-query/docs/domain-capabilities/declaration-signal-compatibility.md)
- [Declaration Entry Orchestration](../../crates/worth-query/docs/domain-capabilities/declaration-entry-orchestration.md)
- [Aspects And Authority Lanes](../../crates/worth-query/docs/modeling/aspects-and-authority-lanes.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [relational_architecture.md](../worth-relational/relational_architecture.md)

#### Phase 5: Declaration Family Capability Matrix

**Why aspects matter here**

Phase 5 already freezes family-scoped support and witness posture, but it still
answers at family breadth more often than semantic breadth. That leaves later
phases with a coarse "family admitted" fact when the real truth is usually
"family admitted, but only for some semantic slices."

**Required aspect contract**

- support rows must distinguish broad family admission from narrower
  aspect-qualified availability
- support must be able to expose slices that are structurally available,
  permission-limited, invariant-sensitive, masked, or unsupported
- later phases must be able to trust family support as the first support gate
  without pretending it already proved every slice they need

**Implementation changes**

- support rows and witness-readiness projections gain aspect-qualified support
  posture
- family-scoped readiness remains the public outer shape, but rows may report a
  narrower semantic slice as unsupported, masked, or denied
- the declaration-entry aspect vocabulary becomes visible this early so later
  phases do not mint local support dialects

**Documentation updates**

- update the capability-matrix doc to treat dynamic geometry context as the
  primary public mental model
- keep canonical family strings only as low-level internal declaration facts,
  not as the main targeting story

**Acceptance criteria**

- support reports can distinguish "family admitted" from "requested slice not
  supported"
- later legality/progression docs consume support as a family-first but not
  slice-complete gate
- adjacent review: [Declaration Family Taxonomy](../../crates/worth-query/docs/domain-capabilities/declaration-family-taxonomy.md),
  [Declaration Legality](../../crates/worth-query/docs/domain-capabilities/declaration-legality.md)

#### Phase 6: Declaration Legality

**Why aspects matter here**

Legality currently proves that a broad declaration artifact passed structural
review. Phase 24b requires legality to say which semantic slices were actually
reviewed so later phases do not over-trust a coarse legality success.

**Required aspect contract**

- legality evidence records reviewed slices explicitly
- legality may deny on aspect-sensitive incompleteness or conflict where that
  is the real failure
- later phases may trust legality scope without reopening broad declaration
  meaning

**Implementation changes**

- legality evidence gains aspect-sensitive structural scope
- legality denial surfaces may carry aspect-sensitive sub-causes without
  flattening into generic unsupported or denied posture

**Documentation updates**

- update the legality doc so its examples teach dynamic geometry context rather
  than raw target lookup
- make explicit that legality certifies a reviewed semantic slice, not just a
  whole declaration bag

**Acceptance criteria**

- legality success and denial are inspectable in semantic-slice terms
- progression and neighboring docs consume legality scope as retained proof
- adjacent review: [Canonical Domain Declarations](../../crates/worth-query/docs/domain-capabilities/canonical-domain-declarations.md),
  [Declaration Progression](../../crates/worth-query/docs/domain-capabilities/declaration-progression.md)

#### Phase 7: Declaration Progression

**Why aspects matter here**

Progression is the first retained proof surface later product binding consumes.
If progression stays digest-first and aspect-blind, later route/receipt/
envelope/orchestration work inherits ambiguity that should already have been
carried structurally here.

**Required aspect contract**

- progression carries aspect-qualified admissible truth
- `binding_target()` exposes aspect contract and coverage
- later product binding may narrow by aspect fit rather than only progression
  identity or family label

**Implementation changes**

- progressed artifacts gain required/preserved/published/masked/incompatible
  slice posture
- the shared binding seam must treat progressed truth as aspect-aware, not only
  digest-aware

**Documentation updates**

- update the progression doc to make aspect-qualified admissible truth explicit
- update closeout/handoff language so later phases are described as consumers
  of progressed aspect truth rather than rediscoverers of it

**Acceptance criteria**

- progressed artifacts expose contract and coverage needed by later binding
- later product-target surfaces can narrow or deny by aspect fit
- adjacent review: [Declaration Legality](../../crates/worth-query/docs/domain-capabilities/declaration-legality.md),
  [Declaration Foundational Evidence](../../crates/worth-query/docs/domain-capabilities/declaration-foundational-evidence.md),
  [milestone-9.3.7-closeout.md](./milestone-9.3.7-closeout.md)

#### Phase 8: Declaration Foundational Evidence

**Why aspects matter here**

Foundational evidence is where descriptive publication begins. If it only names
profiles and richness classes, later phases cannot tell which semantic slices
were widened, elided, or masked.

**Required aspect contract**

- foundational evidence describes present, widened, elided, and masked slices
- publication breadth becomes semantic-slice honest rather than profile-name
  honest

**Implementation changes**

- foundational evidence and related materialization helpers expose one aspect
  publication contract
- retained explanations may say what was intentionally not published

**Documentation updates**

- update the foundational-evidence doc so materialization is described in
  aspect terms rather than only richness terms

**Acceptance criteria**

- foundational evidence exposes aspect publication breadth
- later route and materialization phases consume the same vocabulary
- adjacent review: [Declaration Progression](../../crates/worth-query/docs/domain-capabilities/declaration-progression.md),
  [Aftermath Review Support Eligibility And Materialization](../../crates/worth-query/docs/domain-capabilities/aftermath/aftermath-review-support-eligibility-and-materialization.md),
  [Declaration Route Plans](../../crates/worth-query/docs/domain-capabilities/declaration-route-plan.md)

#### Phase 9: Declaration Route Plans

**Why aspects matter here**

Route plans are the first place later phases start making "which path is
meaningful?" decisions. That choice is often really about semantic slice
fitness, not merely route family or retained identity.

**Required aspect contract**

- route plans expose required, preserved, and incompatible slices
- route explanation preserves the route-relevant semantic slice
- route denial may be aspect-sensitive when that is the real reason

**Implementation changes**

- route-plan artifacts gain route-relevant aspect contract and fit posture

**Documentation updates**

- update the route-plan doc with explicit aspect-aware admission and explanation

**Acceptance criteria**

- route artifacts expose route-relevant aspect truth
- later receipts and envelopes consume route slices rather than broad route
  shape
- adjacent review: [Declaration Foundational Evidence](../../crates/worth-query/docs/domain-capabilities/declaration-foundational-evidence.md),
  [Declaration Boundary Receipts](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-receipts.md)

#### Phase 10: Declaration Boundary Receipts

**Why aspects matter here**

Receipts are crossing claims. If they are coarse, later phases bind from a
broad "crossed" story even when only some semantic slices truly crossed.

**Required aspect contract**

- receipts expose the slices the crossing posture actually covers
- receipt truth may not overclaim beyond route-backed semantic coverage

**Implementation changes**

- receipt artifacts carry aspect-scoped crossing coverage

**Documentation updates**

- update the receipt doc so receipts are described as scoped crossing claims

**Acceptance criteria**

- later binding can distinguish covered slices from adjacent or masked slices
- adjacent review: [Declaration Route Plans](../../crates/worth-query/docs/domain-capabilities/declaration-route-plan.md),
  [Declaration Boundary Envelopes](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-envelopes.md)

#### Phase 11: Declaration Boundary Envelopes

**Why aspects matter here**

Envelopes are the public crossing artifact. Later continuation and grouped
authoring should be able to bind from the published meaning honestly instead of
reopening lower artifacts to rediscover which semantic slices crossed.

**Required aspect contract**

- envelopes expose published and masked slices
- public crossing meaning stays self-describing at aspect granularity

**Implementation changes**

- envelope artifacts gain one public publication contract over semantic slices

**Documentation updates**

- update the envelope doc to describe aspect-scoped public meaning explicitly

**Acceptance criteria**

- later continuation can bind from envelope meaning without reopening route or
  receipt truth
- adjacent review: [Declaration Boundary Receipts](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-receipts.md),
  [Declaration Entry Orchestration](../../crates/worth-query/docs/domain-capabilities/declaration-entry-orchestration.md)

#### Phase 12: Declaration Relational Truth Routing

**Why aspects matter here**

Relational already owns real aspect-filtered truth access. Query should route
into that surface using the same semantic slice vocabulary instead of faking a
broader local notion of relational truth.

**Required aspect contract**

- routing aligns with relational `required_aspects()`
- routing exposes required, covered, and missing relational slices
- scope, invariant, history, and merge-sensitive truth claims become
  aspect-scoped

**Implementation changes**

- relational-routing artifacts expose relational aspect requirements and
  coverage

**Documentation updates**

- update the relational-routing doc to connect directly to relational
  projection and aspect-filtered truth access

**Acceptance criteria**

- routing success/denial aligns with relational aspect contracts
- adjacent review: [Declaration Boundary Envelopes](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-envelopes.md),
  [Declaration Signal Compatibility](../../crates/worth-query/docs/domain-capabilities/declaration-signal-compatibility.md),
  [relational_architecture.md](../worth-relational/relational_architecture.md)

#### Phase 13: Declaration Bridge Continuation Routing

**Why aspects matter here**

Bridge mapping ambiguity is already real in the bridge layer. Query should not
make later continuation phases rediscover mapped vs missing vs partial aspect
coverage themselves.

**Required aspect contract**

- bridge routing exposes mapped, missing, partial, and ambiguous aspect sets
- bridge mapping becomes retained semantic truth rather than hidden backend
  detail

**Implementation changes**

- bridge-routing artifacts carry aspect-map coverage posture explicitly

**Documentation updates**

- update the bridge-routing doc so later continuation work is described as a
  consumer of shipped mapping truth

**Acceptance criteria**

- mapping ambiguity or partial coverage is typed and observable
- adjacent review: [Declaration Relational Truth Routing](../../crates/worth-query/docs/domain-capabilities/declaration-relational-truth-routing.md),
  [Declaration Signal Compatibility](../../crates/worth-query/docs/domain-capabilities/declaration-signal-compatibility.md)

#### Phase 14: Declaration Signal Compatibility

**Why aspects matter here**

Signal is already deeply aspect-aware. Compatibility that stays family-first
and aspect-vague underuses the real semantics the signal layer already exposes.

**Required aspect contract**

- compatibility surfaces dependency aspects, produced aspects, and
  basis-sensitive aspect requirements
- aspect-level incompatibility becomes explicit and typed

**Implementation changes**

- signal-compatibility artifacts carry semantic-slice compatibility posture

**Documentation updates**

- update the signal-compatibility doc to align with runtime/signal aspect
  vocabulary rather than a local approximation

**Acceptance criteria**

- compatibility denial can occur at semantic-slice level
- adjacent review: [Declaration Bridge Continuation Routing](../../crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md),
  [Aspects And Authority Lanes](../../crates/worth-query/docs/modeling/aspects-and-authority-lanes.md)

#### Phase 23: Materialization / Aftermath

**Why aspects matter here**

Lean/support-ready/full-descriptive tiers already exist, but they currently say
too much in terms of profile shape and too little in terms of semantic slices.

**Required aspect contract**

- tiers define semantic-slice widening, masking, and elision explicitly
- cost posture remains honest about what truth is actually being published

**Implementation changes**

- materialization and aftermath publication surfaces expose semantic publication
  breadth

**Documentation updates**

- update aftermath/materialization docs to align with foundational-evidence and
  orchestration terminology

**Acceptance criteria**

- tiers are comparable in aspect-contract terms without changing semantic truth
- adjacent review: [Declaration Foundational Evidence](../../crates/worth-query/docs/domain-capabilities/declaration-foundational-evidence.md),
  [Declaration Entry Orchestration](../../crates/worth-query/docs/domain-capabilities/declaration-entry-orchestration.md)

#### Phase 24: Declaration Entry Orchestration

**Why aspects matter here**

This is where candidate choice becomes user-visible. If orchestration prefers
source order or broad artifact class before semantic slice fit, it recreates
the same ambiguity bugs this addendum is meant to prevent.

**Required aspect contract**

- orchestration candidate selection prefers aspect fit and coverage before
  fallback precedence
- plans and transcripts expose aspect-contract digests and narrowing reasons
- masked slices may not count as successful binding coverage

**Implementation changes**

- orchestration plans and transcripts carry aspect-contract and fit explanation
- ambiguity denies when best-fit candidates tie

**Documentation updates**

- update orchestration docs so dynamic context binding is the primary geometry
  mental model
- keep low-level canonical strings as internals, not as the ideal DX story

**Acceptance criteria**

- orchestration resolves or denies by best aspect fit
- transcripts explain why a candidate won or why ambiguity denied
- adjacent review: [Declaration Route Plans](../../crates/worth-query/docs/domain-capabilities/declaration-route-plan.md),
  [Declaration Boundary Receipts](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-receipts.md),
  [Declaration Boundary Envelopes](../../crates/worth-query/docs/domain-capabilities/declaration-boundary-envelopes.md),
  [milestone-9.3.8-closeout.md](./milestone-9.3.8-closeout.md),
  [worth_query_vision.md](./worth_query_vision.md)

**Shared acceptance evidence**

- parity suites proving aspect-qualified and non-qualified paths converge when
  semantics are identical and diverge observably when contracts differ
- hostile tests proving ambiguity resolves or denies by aspect fit rather than
  folklore ordering
- hostile tests proving masked or unsupported slices never leak into later
  retained binding success
- materialization and cost tests proving publication breadth may change without
  changing declaration-entry semantic truth

### Phase 25: Typed Binding / Extractor / Resolver Boundary

This phase gives `9.3.8` its Rust-native equivalent of route-model binding:
not ambient magic, not a DI container, but a typed binding pipeline that turns
declared context plus retained proof plus explicit aspect contracts into the
next admissible Query artifact.

The first slice of this phase now ships under
`crates/worth-query/src/binding_pipeline/` as the Query-owned extraction,
resolution, witness, and proof boundary over the shared retained
target-binding substrate from Phase 24a and the aspect-law retrofit from
Phase 24b.

The first shipped slice now includes both context extractors and retained
artifact resolvers:

- declaration binding from explicit declaration candidates
- route / receipt / envelope request binding from current progression context
- continuation request binding from current envelope context
- route / receipt / envelope / continuation resolver requests from retained
  progression, route-plan, receipt, and envelope targets

This phase should be read as a generalization of the typed target-binding seam
that already shipped in `9.3.7`, not as a second binding invention beside it.
`9.3.7` proved that contribution authoring needed one typed target-binding
family; Phase 25 lifts that same idea into the shared binding substrate for
all later declaration-entry, orchestration, continuation, and ergonomic
surfaces in this milestone family. Phase 24b now lands first so the extractor
and resolver model inherits an already-shipped aspect granularity law instead
of trying to invent both seams at once.

The goal is to make the new platform-entry and product-target orchestration
surfaces feel compact and declarative without reintroducing the exact
architectural lies the earlier phases just removed. Binding must therefore
remain:

- proof-bearing rather than heuristic
- capability-scoped rather than ambient
- inspectable rather than magical
- authority-preserving rather than convenience-first

**Required Query artifacts**

- one typed request-extractor family for binding external session/tool/UI
  context into admitted declaration-entry requests
- one retained-artifact resolver family for binding canonical Query artifacts
  into the next admissible route/receipt/envelope/continuation inputs
- one family-scoped binding contract family that lets declaration families
  expose binding semantics without minting local pseudo-Query layers
- one aspect-fit / aspect-coverage binding layer that can prefer exact semantic
  slice match over folklore source ordering
- one context-bound capability witness family so route narrowing, world
  identity, and authority posture can be carried structurally rather than
  rediscovered from ambient glue
- one binding transcript / explanation surface mapping declarative binding
  requests back to the same canonical orchestration and admission artifacts

**Requirements**

- the shipped `9.3.7` typed contribution target-binding family must either be
  reused directly or subsumed by a stronger shared binding substrate; it may
  not remain as a parallel contribution-only binding world once this phase
  closes
- binding may not be implemented as an ambient service container, dynamic
  decorator stack, or hidden runtime registry that changes behavior outside
  the type signatures
- extraction from session/tool/UI context must lower into the same admitted
  declaration-entry and progressed-declaration truths the explicit path would
  have produced, or reject before construction if the required context is
  unavailable
- retained-artifact resolvers must consume proof-bearing Query artifacts such
  as canonical declarations, progressed declarations, route plans, receipts,
  envelopes, and future continuation artifacts rather than re-deciding meaning
  from raw host data
- extractors and resolvers must compile onto the Phase 24b declaration-entry
  aspect contract vocabulary rather than inventing a second local notion of
  semantic granularity
- family-scoped binding contracts must compile onto the existing Phase 5
  family capability and Phase 21-24 orchestration laws instead of creating a
  second family-registration or family-helper world
- capability witnesses must preserve wrong-world, wrong-handle, basis, and
  authority posture distinctions explicitly; binding may not turn those into
  generic "dependency missing" folklore
- declarative binding convenience must not imply later route, receipt,
  envelope, continuation, runtime, workspace, or signal execution happened if
  the pipeline only prepared or admitted the next step
- binding surfaces must preserve materialization, sequencing, and denial truth;
  they are an ergonomic lowering layer, not a semantics-changing layer
- Query should be able to say "this declaration request binds from the active
  geometry selection" or "this continuation request binds from the current
  receipt and admitted basis" without forcing callers to manually reassemble
  those same facts

**DX target**

- ordinary domain code should read like "bind request from my current context"
  or "bind continuation from this retained artifact" rather than "load UI
  state, resolve ids, construct helpers, and manually prove the same
  preconditions"
- advanced users must still be able to drop to explicit retained-artifact
  constructors whenever they want exact control

**Documentation obligation**

- yes; this phase must explicitly teach the difference between typed binding
  and ambient framework magic, because the DX story will otherwise be
  misunderstood as hidden execution

**Acceptance evidence**

- parity suites proving bound declaration-entry, route, receipt, and envelope
  requests converge with explicit retained-artifact construction when they
  describe the same meaning
- hostile tests for wrong-world, wrong-handle, missing-context, stale-basis,
  and authority-mismatch binding posture
- compile-fail coverage proving capability witnesses, family-scoped binding
  contracts, and binding-only constructors cannot be WORTHd outside the
  proving authority

**Shipped first-slice answers**

- the initial build ships both context extractors and retained-artifact
  resolvers rather than forcing one side to invent local glue first
- the proof surface is a small distinct binding artifact family
  (`WORTHQueryBindingChecked` and `WORTHQueryBindingTranscript`) that still
  maps back to canonical retained artifacts
- family-scoped contracts now live in the Query-owned binding pipeline seam as
  public contract artifacts instead of helper-local registries

### Phase 26: Denial-Preserving Ordinary Outcome Boundary

This phase ensures the ordinary public surface returns the same typed
non-success posture the explicit surfaces already know how to expose.

Phase 20 already preserves typed non-success posture on the generic ordinary
entry surface through `WORTHQueryDeclarationEntryOrchestrationTerminalError`.
Phase 26 should therefore be read as follow-on work for any broader ordinary
outcome expansion, explanation shaping, or parity closure beyond the current
envelope-ceiling ordinary result shape. It should not assume that ordinary
entry currently collapses non-success posture into an untyped or stringly
error.

Phase 22 also already freezes a second distinction that Phase 26 must preserve:

- automation refusal is not the same thing as denial, deferral, stale,
  rebind-required, or failure posture

Phase 23 now freezes a third distinction that Phase 26 must also preserve:

- expensive work or richer publication gates are not themselves denial,
  deferral, stale, rebind-required, or failure posture

Phase 25 now ships the ordinary/checked/proof-visible binding outcome family,
so Phase 26 must extend that same `binding_pipeline` seam rather than
inventing a second ordinary-outcome vocabulary for context and retained-target
binding.

This phase now ships. Historical readers should therefore treat the ordinary
outcome family as closed public surface, not as a future placeholder:

- `WORTHQueryOrdinaryOutcome<T>`
- `WORTHQueryOrdinaryPosture`
- `WORTHQueryOrdinaryPostureKind`
- `WORTHQueryOrdinaryNextStep`
- `WORTHQueryOrdinaryCheckedTopology`

**Required Query artifacts**

- one ordinary-surface denial / advisory / deferred / unsupported outcome
  family
- one ordinary-to-checked/proof denial parity mapping
- one typed explanation surface tying ordinary outcomes back to retained
  declaration-entry truth

**Requirements**

- ordinary orchestration may not flatten unsupported family, denied legality,
  denied route, denied receipt, deferred neighbor, stale basis, rebind
  required, invalid basis, incompatible signal family, or forbidden duplicate
  posture into opaque success/failure results
- denial posture must preserve explicit legality-contract-driven causes, Phase 7
  progression outcomes, Phase 9 route-plan denial causes, Phase 10 receipt
  denial causes, and Phase 14 signal-compatibility denial posture where those
  are the real reasons
- ordinary outcomes must remain concise, but the concise surface must still map
  exactly onto the same typed denial topology visible through checked and
  proof-visible surfaces

**Documentation obligation**

- yes; this phase must teach how ordinary denial posture stays concise without
  flattening the typed topology behind it

**Acceptance evidence**

- parity tests proving equivalent non-success causes converge across ordinary,
  checked, and proof-visible orchestration
- hostile tests proving convenience surfaces cannot erase denial topology

### Phase 27: Runtime / Workspace / Basis Continuation Boundary

This phase is the hardest continuation boundary in the orchestration stack:
Query must remove caller-owned runtime/workspace/basis glue without lying about
truth context, basis identity, authority, or execution cost.

Phase 25 now ships one aspect-aware binding pipeline for declared context and
retained artifacts. Continuation, workspace, and basis widening in this phase
must compile onto that shipped `binding_pipeline` surface rather than
introducing continuation-local extractors, workspace-local target recovery, or
basis-local helper registries.

Phase 26 now also ships one denial-preserving ordinary outcome family. Any
concise continuation surface in this phase must project onto that shipped
ordinary layer rather than inventing continuation-local terminal enums or
stringly fallback lanes.

This phase now ships. Historical readers should therefore treat the prepared
continuation surface as closed public product, not as a future placeholder:

- `WORTHQueryPreparedContinuation`
- `WORTHQueryPreparedContinuationChecked`
- `WORTHQueryPreparedContinuationTranscript`
- `WORTHQueryPreparedContinuationOutcome`
- `WORTHQueryContinuationExecution`
- `WORTHQueryContinuationExecutionChecked`
- `WORTHQueryContinuationExecutionTranscript`
- `WORTHQueryContinuationExecutionOutcome`
- `prepare_continuation_from_target(...)`
- `prepare_continuation_from_context(...)`
- `execute_prepared_continuation(...)`

**Required Query artifacts**

- one prepared continuation artifact family
- one runtime / workspace / basis continuation contract family
- one typed truth-context / basis-binding / workspace-entry posture family
- one prepared-vs-executed continuation transcript family
- one continuation-binding projection over the shared Phase 25 binding
  substrate, not a continuation-local binding model

**Locked local and adjacent references**

- Phase 13 bridge continuation routing remains the authoritative continuation
  route posture
- Phase 14 signal compatibility remains the authoritative declaration-to-
  derived-execution compatibility posture
- basis lifecycle remains the source of basis authority and rebinding truth

**Requirements**

- supported runtime-capable families must be able to continue from admitted
  declaration-entry truth without caller-owned runtime builder, backend-part,
  workspace-entry, or basis-binding choreography
- this phase should compile onto the shipped Phase 25 typed binding model
  rather than inventing a second continuation-local extractor or resolver
  vocabulary
- when continuation also consumes `9.3.7` contribution-authored posture, both
  contribution binding and continuation binding must converge on the same
  shared substrate rather than meeting through adapters between two binding
  systems
- continuation must distinguish current, historical, and preview truth/basis
  posture explicitly; it may not collapse them into ambient "non-current"
  folklore
- continuation must distinguish prepared, explicitly executable, denied,
  deferred, wrong-world, wrong-handle, stale-basis, invalid-basis, and
  authority-transition-required posture explicitly
- workspace entry, runtime entry, basis rebinding, and truth-context binding
  must stay typed and inspectable; the ordinary surface may automate them only
  when prior retained proof admits that automation
- mixed-authority declarations, bridge continuation posture, and signal
  compatibility posture must remain observably distinct inside prepared
  continuation rather than merging into one fake universal continuation story
- prepared continuation may stage expensive lower-runtime work, but execution
  must remain explicit where cost, basis, or authority acknowledgment matters

**DX target**

- runtime-capable paths should read like "prepare continuation" and "execute
  continuation" rather than "assemble adapters, bind basis, enter workspace,
  then call the bridge"

**Documentation obligation**

- yes; runtime/workspace/basis continuation must be taught carefully because it
  is the first orchestration layer where DX can easily become structurally
  dishonest

**Acceptance evidence**

- end-to-end examples proving representative declarations can continue into
  runtime-backed workspace and basis-bound paths without caller-owned glue
- hostile tests for wrong-world, wrong-handle, stale-basis, invalid-basis,
  preview/current/historical divergence, and prepared-vs-executed honesty

### Phase 28: Signal Compatibility Orchestration Boundary

This phase composes the Phase 14 compatibility boundary into ordinary
orchestration without erasing execution family, required basis families, or
typed compatibility denials.

This phase now ships. Later phases must therefore treat signal-facing
compatibility orchestration as one real public seam rather than unresolved
glue between retained compatibility and continuation preparation.

Phase 26 now ships the ordinary outcome surface that this phase must reuse.
Signal-facing concise outcomes therefore belong inside the shared ordinary
family, with checked/proof-visible topology still remaining authoritative.

Phase 27 now also ships the prepared/executed continuation pipeline. This
phase must therefore compose signal compatibility with that shipped prepared
continuation seam instead of inventing a signal-local continuation readiness
story or a second execution-admission ladder.

**Required Query artifacts**

- one orchestrated signal-compatibility surface
- one prepared-from-compatible continuation admission surface
- one compatibility-preserving ordinary outcome family

**Requirements**

- when the declaration family is structurally signal-compatible, ordinary
  orchestration should be able to lower through envelope construction and
  signal-compatibility classification without forcing the caller to manually
  inspect basis-sensitive support posture first
- ordinary orchestration that surfaces Phase 14 results may not erase signal
  execution family, required basis families, or typed compatibility denial
  posture
- compatibility must remain a declaration-to-derived-execution boundary rather
  than silently turning into actual signal execution

**Documentation obligation**

- yes; this phase must teach the distinction between compatibility,
  continuation preparation, and actual derived execution

**Acceptance evidence**

- parity suites proving orchestrated compatibility results converge with the
  explicit Phase 14 checked surfaces
- hostile tests proving compatible, deferred, unsupported, and invalid-basis
  posture remain distinct through the ordinary surface

This shipped boundary also settles three questions that later phases must not
reopen:

- `Compatible` and `Prepared` are distinct public states
- retained Phase 14 signal truth is reused rather than rediscovered inside the
  signal-facing composition lane
- signal-facing concise outcomes belong on the shared ordinary outcome family,
  not a signal-local convenience vocabulary

### Phase 29: Contribution-Composed Orchestration Boundary

This phase composes `9.3.8` declaration-entry orchestration with `9.3.7`
domain-capability contribution authoring so the user does not have to cross a
second public grammar seam.

Phase 25 now closes the shared binding pipeline for both context extraction and
retained-target resolution. This phase must therefore be read as contribution
composition over the shipped binding pipeline, not as a place where
contribution authoring or orchestration can each grow local binding glue.

That same rule applies to aspects. Contribution composition must preserve the
declaration-side aspect contract and the contribution-side aspect coverage that
already live in the shared binding substrate. This phase is not allowed to
launder those semantic slices into one flat "declaration succeeded" story or
one flat "contribution attached" story.

Phase 26 now closes the ordinary outcome vocabulary those composed surfaces
must reuse. Contribution composition may add richer typed posture, but it must
not fork a second concise terminal story.

Phase 27 now also closes the prepared/executed continuation seam. If
contribution-composed orchestration later reaches continuation preparation or
execution, that composition must lower through the shipped continuation
pipeline rather than contribution-local runtime/workspace glue.

Phase 28 now also closes the signal-facing composition seam. If
contribution-composed orchestration later needs to classify signal readiness
or continue from retained compatibility, it must lower through the shipped
signal-compatibility orchestration surface rather than a contribution-local
signal readiness helper family.

This phase now ships and settles three questions that later phases must not
reopen:

- declaration-side and contribution-side non-success posture stay distinct even
  when one public call composes both proof chains
- declaration-scoped contribution composition reuses the shared binding
  substrate rather than inventing composition-local target glue
- concise contribution-composed results project onto the shared ordinary
  outcome family rather than introducing a local convenience vocabulary

**Required Query artifacts**

- one contribution-composed orchestration surface
- one typed contribution-composed transcript / inspection surface
- one typed contribution-composed denial / support posture mapping
- one aspect-aware contribution-composed binding and inspection story over the
  shipped shared binding substrate

**Requirements**

- for supported declaration families, the ordinary orchestration surface must
  be composable with `9.3.7` domain-capability contribution authoring without
  surfacing the entry/contribution lifecycle boundary as a visible caller seam
- the contribution composition surface must reuse the shared Phase 25 binding
  substrate so declaration extraction, contribution target binding, and
  orchestration target binding remain one continuous public story
- contribution composition must preserve declaration aspect contract and
  contribution aspect coverage explicitly enough that later phases can tell
  whether two composed intents depended on the same semantic slice or on
  distinct slices
- callers should be able to state domain intent, add declaration-scoped
  capability posture, and obtain canonical materialized artifacts in one
  coherent flow while Query preserves that entry and contribution are still two
  proof-bearing internal progressions
- contribution evidence, denial posture, support posture, and grouped-target
  semantics must remain typed and inspectable inside orchestration products
- mixed contribution results must remain inspectable with enough retained
  target-binding and aspect-binding truth that later collaborative or
  merge-aware phases do not have to rediscover aspect overlap from host-local
  folklore or free-form diagnostics

**Documentation obligation**

- yes; this phase must explicitly teach how contribution composition enters the
  orchestration story without creating a second public grammar seam, and it
  must teach how declaration-side aspect contract and contribution-side aspect
  coverage remain part of the retained composed truth

**Acceptance evidence**

- composition tests proving `9.3.8` entry orchestration and `9.3.7`
  contribution authoring converge to one canonical orchestrated story
- hostile tests proving contribution denial or advisory posture cannot be
  laundered away by ordinary orchestration
- parity and hostile tests proving aspect-aware declaration binding and
  contribution binding survive composition without flattening into target-only
  or digest-only explanations

### Phase 30: Semantic Orchestration Inventory And Transcript Boundary

This phase synchronizes the live public orchestration surface with transcripts,
support/readiness, semantic attachments, docs, and certification so the
ergonomic layer stays honest under growth.

**Required Query artifacts**

- one expanded semantic orchestration inventory over every admitted
  orchestration family
- one orchestration transcript inventory
- one coverage map from verbs to support/readiness rows, seam-ledger rows,
  docs, goldens, and certification suites
- one typed semantic attachment layer over aspect posture, basis/preview
  posture, policy/tenant posture, lower-authority family attachment, strategy
  family attachment, contribution compatibility, and future collaborative
  extension posture

Phase 21 already ships the first public grammar inventory for the generic trio.
Phase 30 extends that seed into the full anti-drift synchronization boundary;
it must not invent a second competing inventory model.

Phase 26 now adds a second already-shipped public layer that the inventory must
track explicitly:

- the ordinary outcome surface and its checked-topology links

Phase 27 now adds a third already-shipped public layer the same inventory must
track explicitly:

- prepared continuation artifacts, execution artifacts, and their proof-visible
  transcripts

Phase 28 now adds a fourth already-shipped public layer the same inventory must
track explicitly:

- signal-compatibility orchestration artifacts, their ordinary outcome
  projections, and their proof-visible transcripts

Phase 29 now adds a fifth already-shipped public layer the same inventory must
track explicitly:

- contribution-composed orchestration artifacts, their ordinary outcome
  projections, and their proof-visible transcripts

**Requirements**

- ordinary public verbs, checked verbs, proof-visible verbs, support/readiness
  rows, crossing inventory rows, transcript records, semantic attachment rows,
  docs/goldens, and certification coverage must all agree about what
  orchestration surfaces actually exist
- the same inventory must explicitly track the shared binding substrate and its
  projections so later helper or continuation work cannot silently grow a
  second binding vocabulary
- the same inventory must explicitly track aspect posture, basis posture,
  lower-authority attachment, and strategy-family relevance so later preview,
  merge, collaborative, or helper surfaces cannot smuggle semantic drift
  through docs-only explanations
- contribution-composed and grouped surfaces must advertise their contribution
  compatibility posture explicitly enough that later collaborative or recovery
  phases do not have to rediscover category compatibility from local helpers;
  for some rows the honest posture may be "none", and that absence must remain
  explicit rather than inferred
- adding a new admitted orchestration surface must fail closure if there is no
  matching transcript/inventory/support/doc/certification coverage or if its
  semantic attachment row is incomplete
- transcript inventory must preserve prepared-vs-executed continuation posture,
  contribution composition posture, and denial topology where those matter

**Documentation obligation**

- yes; this phase must define and teach the inventory/transcript/doc
  synchronization contract because it is the main anti-drift mechanism for the
  orchestration layer

**Acceptance evidence**

- parity tests proving live verbs, transcript inventory, and support/readiness
  inventory stay synchronized
- hostile certification showing no documented or exported orchestration surface
  lacks transcript, support, proof, or semantic attachment coverage

### Phase 31: Denial And Recovery UX Boundary

This phase makes failure as usable as success by turning denials, stale states,
rebind requirements, and recovery posture into public product artifacts.

Phase 26 now ships the concise ordinary outcome vocabulary that this phase must
widen rather than replace. Recovery UX must therefore refine the shipped
ordinary posture story, not branch into a second convenience-only denial API.

Phase 27 now also ships typed continuation preparation and execution outcomes.
Recovery UX in this phase must therefore include continuation-specific
wrong-world, wrong-handle, stale-basis, basis-mismatch, authority-mismatch,
prepared-only, and execution-failed posture as refinements of the same shared
ordinary story rather than a second continuation-only recovery vocabulary.

Phase 28 now also ships signal-facing compatibility orchestration outcomes.
Recovery UX in this phase must therefore preserve `Compatible`, `Prepared`,
`BasisMismatch`, `WrongWorld`, `WrongHandle`, `Deferred`, `Denied`, and
`Unsupported` as distinct recovery surfaces rather than collapsing them into a
generic "signal not ready" explanation.

Phase 29 now also ships contribution-composed orchestration outcomes. Recovery
UX in this phase must therefore preserve declaration-side defer/deny/stale/
rebind posture separately from contribution-side deny/stale/rebind posture,
rather than collapsing all composed non-success into one generic declaration
failure or one generic "contribution could not be applied" message.

Phase 30 now also ships the orchestration inventory and audit boundary.
Recovery UX in this phase must therefore register its public ordinary,
checked, and proof-visible recovery entry points through that shipped registry
instead of relying on helper-local discoverability or docs-only coverage.

**Required Query artifacts**

- one denial family for ordinary and checked lanes
- one recovery/rebind/stale posture family
- one explanation surface that ties denial posture back to route, family, and
  authority context
- one typed recovery-request family for rebind, retry, repair, or reroute paths
  that are actually supported

**Locked local and external surfaces**

- checked/proof progression outcomes from Phase 7
- foundational receipt/support/provenance surfaces from Phase 8
- bridge and relational diagnostics only as composed evidence, not as direct
  public substitutes

**Requirements**

- denials must distinguish unsupported family, illegal structure, denied
  legality, denied route, stale basis, rebind required, deferred neighbor, and
  forbidden duplicate posture where those are semantically different
- denial posture must preserve explicit legality-contract-driven causes such as
  wrong admitted world, illegal role claim, and illegal surface disposition
  instead of flattening legality into a generic "not allowed" outcome
- denial posture must preserve explicit Phase 7 progression outcomes such as
  `Deferred`, `Stale`, `RebindRequired`, and `Failed` rather than flattening
  them into generic route failure or retry folklore
- denial posture must preserve the typed Phase 9 route-plan denial causes and
  route explanation surface rather than reducing them to free-form summary
  strings
- denial posture must preserve the typed Phase 10 receipt-boundary denial
  causes and receipt explanation surface rather than flattening unsupported or
  failed crossing issuance into generic route denial or generic retry folklore
- denial posture must preserve the typed Phase 14 signal-compatibility
  classes, signal execution family, required basis families, and compatibility
  denial causes rather than flattening basis mismatch or incompatible-family
  posture into generic "signal not ready"
- denial posture must be able to surface Phase 8 foundational evidence for
  legality denials and non-success progression lanes directly, rather than
  forcing recovery UX to reconstruct descriptive context from raw denial enums
- denial and recovery posture must distinguish at least three repair surfaces
  when they differ semantically: admitted operating-world repair,
  declaration-meaning repair, and truth/continuation-context repair
- the recovery story must explain what the user can do next without forcing
  them to reverse-engineer lower-crate semantics
- ordinary-surface denial UX must be concise but truthful; checked and
  proof-visible surfaces must expose the exact typed structure underneath
- denial explanations must remain route-sensitive and family-sensitive
- supported recovery paths must take typed recovery inputs rather than relying
  on developers to remember which bits of context need to be repaired or
  replayed after a denial, stale, or rebind outcome

**DX target**

- failure should feel actionable, not like the framework just said no
- users should leave denial paths knowing whether to rebind, narrow, pick a
  different family, or stop

**Documentation obligation**

- yes; denial and recovery posture are public behavior and need explicit docs

**Compile-time enforcement**

- denial surfaces must be typed and non-binary; they may not collapse into one
  generic error string
- stale/rebind posture must remain a distinct outcome family rather than an
  attached note on generic failure

**Acceptance evidence**

- hostile denial suites covering unsupported, illegal, stale, rebind, deferred,
  and forbidden cases
- parity tests proving equivalent failure causes converge to the same denial
  posture across ordinary and checked lanes
- compile-fail coverage for code paths that try to treat typed denial families
  as opaque success/failure booleans

**Open questions before implementation**

- should recovery requests be one typed family with submodes or several
  dedicated recovery input families for rebind, retry, repair, and reroute?
- which recovery paths are ergonomic enough for the ordinary lane versus better
  left to checked/proof guidance only?
- how much automatic repair guidance can Query offer without pretending to own
  lower-authority semantics it does not actually control?

### Phase 32: Family-Specific Ergonomics Boundary

This phase gives the major declaration families native-feeling public helper
surfaces instead of forcing everything through one generic orchestration shape.

Phase 25 now ships the public request, outcome, witness, and transcript seam
for aspect-aware binding. Family helpers in this phase must project onto those
same shipped binding artifacts rather than exposing family-local binding bags,
ambient source probing, or undocumented proof shortcuts.

Phase 26 now ships the shared ordinary outcome family those helpers must use on
their concise public lanes.

Phase 29 now also ships contribution-composed orchestration. Family helpers in
this phase must therefore compose onto the same declaration-plus-contribution
surface when they promise declaration-scoped contribution posture, rather than
inventing helper-local composition lanes or bypassing the composed proof story.

Phase 30 now also ships the orchestration inventory and audit boundary. Family
helpers in this phase must therefore register their public verbs, checked/proof
lanes, ordinary projections, docs references, and certification references
through that shipped inventory instead of relying on helper-local folklore or
docs-only discoverability.

Phase 31 now also ships the recovery boundary. Family helpers in this phase
must therefore project helper-local denial and repair posture onto that same
typed recovery brief/request surface when they expose helper-specific failure
guidance, rather than inventing helper-local retry or repair bags.

**Required Query artifacts**

- one family-specific helper surface per admitted major family
- one mapping from helper surface back to canonical orchestration and family
  identity
- one support/readiness and denial posture projection per helper family

**Requirements**

- helper surfaces must read like the user's domain intent, not like generic
  framework contribution plumbing
- helpers must compile onto the same canonical declaration, route, receipt,
  envelope, orchestration, and contribution-composed artifacts as the generic
  surface
- helper surfaces should prefer the shipped typed binding / extractor /
  resolver seam where that keeps domain callsites declarative without hiding
  retained proof or authority posture
- helper surfaces are additive aliases under the locked generic trio, not a
  replacement grammar and not a second equally-primary front door
- helper surfaces must compile onto the same Phase 5 family capability
  boundary, including the same support/admission checks and the same structural
  witness availability rules
- helper surfaces may not invent new authority classes, new progression rules,
  or new receipt semantics
- helper surfaces may not invent helper-local binders or resolvers; they must
  project onto the shared binding substrate and canonical orchestration path
- any helper that meaningfully changes semantics must declare a distinct family
  rather than posing as a synonym

**DX target**

- major declaration families should feel native and pleasant to author, not
  like users are always falling back to the generic substrate path
- helpers should increase confidence and speed without increasing ambiguity

**Documentation obligation**

- yes; helper families are public conveniences and need family-specific docs

**Compile-time enforcement**

- helper surfaces must remain family-gated and unavailable on the wrong handle
  or family posture
- helper-generated artifacts must remain equivalent to the generic canonical
  path when semantically identical

**Acceptance evidence**

- golden DX transcripts for the major helper families
- canonical digest parity between helper surfaces and generic surfaces for
  equivalent meaning
- compile-fail suites proving helper families are not callable where the
  capability matrix forbids them

**Historical closure notes**

- the first shipped helper family is geometry, not a speculative all-families
  sweep
- helper access now lands through one admitted-handle helper facade with
  family-gated geometry surfaces rather than a second public root
- continuation-oriented geometry helpers currently project onto the shipped
  signal-compatibility orchestration seam instead of minting helper-local
  continuation artifacts
- declaration-scoped material-attachment helpers currently project onto the
  shipped contribution-composed orchestration seam instead of inventing a
  helper-local contribution grammar
- later phases must extend this helper seam rather than reopening whether
  helpers are allowed to bypass shared binding, shared recovery, or the
  orchestration inventory
- how do we keep helper naming pleasant without creating overlapping synonyms
  that weaken the â€œone obvious pathâ€ rule?

### Phase 33: Neighborhood Authoring DX Boundary

This phase makes meaningful grouped declarations first-class so geometry and
topology domains can work in local neighborhoods rather than only isolated
single declarations.

Phase 25 now ships the retained-target and context-binding seam that grouped
authoring must consume. Group-level and member-level resolution in this phase
must therefore extend the shipped binding pipeline instead of inventing
neighborhood-specific binders or grouped-only target-recovery helpers.

Phase 26 now ships the shared ordinary outcome family. Grouped authoring must
therefore extend that same concise result story instead of creating grouped-only
terminal shortcuts.

Phase 29 now also ships contribution-composed orchestration. Grouped authoring
in this phase must therefore preserve a path to group-level and member-level
contribution composition on top of the shipped composed seam rather than
inventing a second grouped-only contribution grammar.

Phase 30 now also ships the orchestration inventory and audit boundary. Grouped
authoring in this phase must therefore extend the canonical inventory with its
group-level orchestration families, transcript families, support rows, docs
references, and certification references rather than introducing grouped-only
surface maps.

Phase 31 now also ships the recovery boundary. Grouped authoring in this phase
must therefore extend that same recovery surface for group-level and
member-level denial, stale, rebind, and support posture instead of inventing a
grouped-only repair vocabulary.

This phase now ships as the public grouped-authoring seam for Query's first
real neighborhood-shaped declaration flow. Later phases must therefore build on
that real grouped declaration artifact, grouped orchestration lane, grouped
helper projection, grouped inventory registration, and grouped recovery path
instead of treating grouped authoring as unresolved future glue.

**Required Query artifacts**

- one grouped declaration artifact family
- one grouped route/receipt/envelope/orchestration family
- one grouped support/readiness and denial posture family
- one grouped canonical digest/equivalence surface
- one typed grouped-declaration input family that captures grouping semantics
  explicitly rather than accepting a bare collection

**Requirements**

- neighborhood and batch declarations must remain explicit groups, not ad hoc
  arrays of unrelated single declarations
- grouped authoring must reuse the same shared binding substrate for group-
  level and member-level target resolution instead of introducing neighborhood-
  specific binders
- grouped declaration formation must consume retained grouped-posture proof from
  earlier phases rather than rediscovering grouping semantics from family names
  or generic collection shape
- grouped declaration semantics must preserve shared posture, shared rationale,
  and shared route/denial context where that grouping is semantically real
- grouped declaration artifacts must preserve the ability to carry `9.3.7`
  domain-capability contributions at group level and at member level.
  Grouped contribution binding must compose with `9.3.7`
  contribution-target binding families rather than inventing a second grouped
  contribution vocabulary
- batching may improve ergonomics and cost posture, but it may not silently
  merge declarations whose semantics should stay distinct
- grouped artifacts must still map back to single-declaration family and route
  inventory rows in a certification-readable way
- grouped declaration inputs must carry typed semantics for shared posture,
  continuity assumptions, ordering/atomicity, and exploratory vs authoritative
  grouping where those distinctions are admitted

**DX target**

- grouped authoring should feel natural for geometry neighborhoods and other
  real multi-declaration work
- users should be able to think in meaningful local structures rather than
  manually coordinating many isolated declarations

**Documentation obligation**

- yes; grouped/neighborhood authoring changes public usage substantially and
  needs dedicated docs

**Compile-time enforcement**

- grouped declaration APIs must distinguish intentional neighborhood groups from
  arbitrary collections
- unsupported family mixes inside one grouped declaration must deny or fail
  explicitly rather than widening by convenience

**Acceptance evidence**

- parity suites showing semantically equivalent grouped authoring paths
  canonicalize identically
- hostile tests for illegal grouped mixes and illegal shared-posture claims
- cost/readiness evidence showing batching does not hide widened work

**Open questions before implementation**

- should grouped declarations start with one general typed input family or a
  small set of specialized neighborhood/batch families with shared traits?
- which grouped semantics must be first-class from day one: shared rationale,
  shared support posture, ordering, atomicity, exploratory grouping, or all of
  them?
- how should grouped route/receipt/envelope artifacts expose member-level versus
  group-level facts without becoming unreadable?

**Historical closure notes**

- the first shipped grouped semantics slice is `LocalNeighborhood`, not an
  all-grouped abstraction sweep
- the first shipped grouped front door is geometry-only and extends the helper
  seam rather than introducing a second grouped root
- grouped declaration admission and grouped orchestration now ship as real
  retained public artifacts instead of helper-local bags
- grouped wrong-world and wrong-handle posture now stop at the grouped lane
  before member lowering and remain visible through grouped recovery
- later phases must extend this grouped seam rather than reopening whether
  grouped authoring is allowed to degrade into loops over single-member helpers

### Phase 34: Public Documentation And Golden Teaching Boundary

This phase makes the platform-entry seam teachable and ensures the docs do not
lose critical behavior to oral tradition.

Phase 25 now ships a dedicated typed binding pipeline feature surface, so this
docs/goldens phase must treat that surface as part of the ordinary public
teaching inventory rather than as background architecture implied by
orchestration examples.

Phase 27 now ships a dedicated continuation pipeline feature surface too, so
this docs/goldens phase must treat prepared/executed continuation as part of
the ordinary public teaching inventory rather than as hidden bridge follow-up
knowledge.

Phase 28 now also ships a dedicated signal-compatibility orchestration feature
surface, so this docs/goldens phase must treat signal-facing
`Compatible`/`Prepared` composition as part of the ordinary public teaching
inventory rather than as a footnote inside retained compatibility or
continuation docs.

Phase 29 now also ships a dedicated contribution-composed orchestration feature
surface, so this docs/goldens phase must treat declaration-plus-contribution
composition as part of the ordinary public teaching inventory rather than as a
local advanced recipe hidden between declaration-entry and contribution docs.

Phase 30 now also ships the orchestration inventory and audit boundary, so this
docs/goldens phase must consume that registry as the source of truth for
covered public orchestration surfaces rather than hand-maintaining a parallel
surface list in docs-only metadata.

Phase 31 now also ships a dedicated recovery feature surface, so this
docs/goldens phase must treat typed recovery briefs, requests, and
route-sensitive denial explanation as part of the ordinary public teaching
inventory rather than leaving failure handling to scattered examples or helper
folklore.

Phase 33 now also ships a dedicated grouped-authoring feature surface, so this
docs/goldens phase must treat grouped declaration inputs, grouped artifacts,
grouped orchestration, grouped helper verbs, and grouped recovery as part of
the ordinary public teaching inventory rather than burying them inside helper
docs or milestone history.

This phase now ships as the public-doc-coverage seam for Query's teaching
surface. Later phases must therefore consume the shipped coverage inventory,
golden catalog, and docs-coverage audit instead of rebuilding docs/goldens
breadth from scattered markdown, file names, or certification folklore.

**Required Query artifacts**

- one documentation inventory over the admitted platform-entry and admitted-
  orchestration surfaces
- one golden transcript catalog that matches the ordinary public surface
- one coverage map from docs and transcripts back to the crossing inventory and
  orchestration inventory

**Requirements**

- every admitted ordinary public family, continuation surface,
  contribution-composed surface, grouped-authoring surface, and major helper
  surface must have one honest documented path
- the docs/goldens coverage inventory must derive from the shipped
  orchestration inventory and its audit posture rather than a second manual
  list of public orchestration surfaces
- the docs and goldens must teach one shared binding story that connects
  `9.3.7` contribution target binding, `9.3.8` declaration/product binding,
  and later continuation/grouped binding, rather than documenting those as
  unrelated convenience families
- docs must teach support/readiness posture, denial posture, route/receipt/
  envelope meaning, continuation truth, contribution composition, and lower-
  authority ownership honestly
- golden transcripts must be treated as product artifacts, not blog-style
  examples
- docs must not reintroduce obsolete split-seam or "happy path" framing

**DX target**

- the docs should make the right path feel obvious enough that most users never
  need to guess
- examples should teach confidence and mental model clarity, not just syntax

**Documentation obligation**

- yes by definition; this phase is the docs closure phase

**Compile-time enforcement**

- documented ordinary paths must stay synchronized with live public surfaces and
  golden transcript coverage
- undocumented public surfaces in the covered inventory must fail doc coverage
  checks or closeout certification

**Acceptance evidence**

- doc coverage inventory aligned with the seam ledger and orchestration
  inventory
- golden transcript parity checks against live public APIs
- QA pass proving no critical platform-entry behavior is lost to history

**Historical closure notes**

- the shipped docs/goldens authority is now `public_doc_coverage`, not the
  README alone and not orchestration inventory references by themselves
- the shipped golden catalog now distinguishes real surface-coverage goldens
  from the docs-coverage boundary readout itself
- helper and grouped-authoring public surfaces are now first-class coverage
  rows instead of implicit coverage through nearby generic pages
- later certification work must consume this shipped coverage seam instead of
  reconstructing docs breadth by hand

**Open questions before implementation**

- what is the exact doc inventory format that keeps feature docs, goldens,
  crossing inventory, and orchestration inventory synchronized without too much
  manual bookkeeping?
- which examples should be the canonical teaching cases for the first public
  rollout, especially for runtime-capable, contribution-composed, and grouped
  declaration paths?
- how should docs surface lower-authority ownership honestly without forcing
  users to learn relational/bridge/signal internals too early?

### Phase 35: Certification And Closeout Boundary

This phase closes the milestone with hostile proof rather than plausibility.

**Required Query artifacts**

- one certification bundle for the full platform-entry and orchestration seam
- one compile-fail suite spanning Phases 1 through 33
- one parity suite spanning ordinary, checked, proof-visible, helper, grouped,
  and contribution-composed surfaces
- one hostile certification harness over route plans, receipts, envelopes,
  support posture, denials, continuation, signal-facing compatibility
  orchestration, transcripts, grouped declarations, and docs coverage

**Locked certification expectations**

- the bundle must extend the certification mentality already used in
  `worth-query`
- route-plan, receipt, envelope, support/readiness, denial, and grouped
  declaration digests must all be machine-checkable

**Requirements**

- equivalent public/proof/generic/helper/grouped/contribution-composed paths
  must converge canonically when semantically identical
- equivalent contribution-binding, declaration-binding, product-binding, and
  continuation-binding paths must converge onto the same shared binding
  substrate artifacts when semantically identical
- equivalent prepared-continuation and executed-continuation paths must
  converge canonically with explicit retained bridge/signal/basis routing when
  semantically identical
- equivalent retained signal-compatibility, signal-facing orchestration, and
  prepared-continuation paths must converge canonically when they describe the
  same retained declaration story and intentionally diverge when orchestration
  stops at compatibility instead of preparation
- equivalent recovery briefs and recovery requests must converge across
  ordinary, checked, and proof-visible stop surfaces when they describe the
  same semantic stop, and intentionally diverge when the fix belongs to a
  different authority surface
- equivalent grouped-helper and generic grouped-authoring paths must converge
  canonically when they describe the same grouped declaration meaning, and
  intentionally diverge when group-level versus member-level posture differs
- intentionally different family, route, authority, denial, cost, continuation,
  support, and contribution posture must diverge observably and predictably
- compile-fail coverage must match the live ordinary lane breadth
- certification closure must treat the shipped orchestration inventory/audit
  boundary as authoritative for public orchestration breadth, and must fail if
  any covered public orchestration surface lacks an inventory row, transcript
  family, docs reference, or certification reference
- certification closure must also treat the shipped public-doc-coverage
  inventory/audit boundary as authoritative for feature-doc, README-discovery,
  journey, and golden-readout breadth, and must fail if any covered public
  surface lacks a coverage row, valid golden kind, or aligned journey
- inventory, docs, support matrix, transcript inventory, and certification
  bundle breadth must all agree exactly

**DX target**

- users should feel that the platform is trustworthy because every admitted path
  is proven, not just plausible
- the final public experience should feel coherent across ordinary, checked,
  proof-visible, helper, grouped, and contribution-composed surfaces

**Documentation obligation**

- no new feature docs by default, but closeout must verify that all required
  phase-local docs obligations were actually satisfied

**Compile-time enforcement**

- milestone closure is blocked if any covered public surface lacks compile-fail,
  parity, or inventory/doc coverage
- pseudo-Query bypasses that skip canonicalization, legality, progression,
  route planning, or envelope production must fail closed

**Acceptance evidence**

- end-to-end hostile certification bundles
- compile-fail suites for the full public breadth
- parity suites covering ordinary vs checked vs proof-visible vs helper vs
  grouped vs contribution-composed paths
- support matrix, seam inventory, orchestration inventory, docs, and
  certification digest equality checks

This phase now ships as the platform-entry closeout seam for the late
platform-entry public product. Runtime API stabilization and later roadmap
work must therefore consume the shipped closeout surface, closeout bundle,
compile-fail manifest, parity manifest, hostile manifest, and docs/inventory
alignment audits instead of rebuilding closure from scattered seam-local
tests, markdown pages, or remembered command lists.

**Historical closure notes**

- the shipped closeout authority is now `platform_entry_closeout`, not a bag
  of green test commands or milestone prose
- compile-fail closure now consumes the shipped public-doc golden registry for
  surface-readout breadth instead of a frozen remembered subset
- parity and hostile manifests now require both a proof path and a named proof
  anchor so stale rows cannot pass by pointing at an existing file only
- grouped recovery parity is intentionally certified as checked-vs-proof
  convergence for the same grouped stop rather than over-claiming
  ordinary-vs-checked equality where the shipped product still projects the
  ordinary lane through generic binding recovery
- later roadmap work must treat the shipped closeout seam as the final
  certification ledger over orchestration breadth, docs/goldens breadth,
  compile-fail breadth, parity breadth, and hostile breadth

### Phase 36: Collaborative Strategy Admission Boundary

This phase closes the missing Query-owned front door for admitted collaborative
strategy posture before Query tries to retain or classify the larger
collaborative world.

The hard problem is not "can lower runtimes merge or write back?" They already
can. The hard problem is that Query still lacks one product-grade public seam
for admitting and naming which lower strategy families are in play together for
one collaborative domain world.

This phase therefore does not implement merge engines, signal delivery engines,
or writeback engines in Query. It adds one Query-facing typed admission surface
over those lower-authority families so later collaborative witnesses and
classifications stop depending on host-local strategy folklore.

**Required Query artifacts**

- one Query-owned collaborative strategy admission input family
- one checked / proof-visible collaborative strategy admission result family
- one typed descriptor family over admitted relational strategy family,
  admitted signal strategy family, admitted bridge writeback/promotion family,
  and admitted cross-family compatibility posture
- one support/readiness report over collaborative strategy admission that
  converges with ordinary / checked / proof / helper-facing public lanes

**Locked references**

- `worth-relational` remains authoritative for merge, commit-strategy, and
  authoritative truth execution semantics
- `worth-signal` remains authoritative for signal merge, invalidation, and
  delivery semantics
- `worth-runtime-bridge` remains authoritative for preview promotion,
  writeback-family execution, and merge-aware bridge continuity semantics
- `worth-foundational` remains authoritative for shared merge/transition
  strategy vocabulary, basis vocabulary, proof-bearing readmission, and
  support-grade recovery posture
- local foundational docs this phase must reuse instead of replacing:
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/README.md`
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/merge-planning-and-verdicts.md`
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/transition-strategy-and-basis-semantics.md`
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/transition-canonical-basis-locators-and-current-basis.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/README.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/target-aware-profile-attachment.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/profile-identity-difference-and-canonical-basis.md`
  - `crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md`
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/domain-capabilities/declaration-relational-truth-routing.md`
  - `crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md`
  - `crates/worth-query/docs/domain-capabilities/signal-compatibility-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/continuation-pipeline.md`
  - `crates/worth-query/docs/domain-capabilities/platform-entry.md`
  - `crates/worth-query/docs/domain-capabilities/configured-domain-handles.md`
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-admission.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-support-and-readiness.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/application/declaration_relational_routing/`
  - `crates/worth-query/src/application/declaration_bridge_routing/`
  - `crates/worth-query/src/application/domain_handle/`
  - `crates/worth-query/src/orchestration_inventory/`
  - `crates/worth-query/src/public_doc_coverage/`
  - `crates/worth-query/src/platform_entry_closeout/`
  - `crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs`
  - `crates/worth-foundational/src/facade.rs`

**Requirements**

- Query must expose one typed public admission seam for collaborative strategy
  posture instead of forcing domains to reopen lower-runtime strategy
  registries directly
- the strategy admission seam must preserve the authority split above instead
  of re-owning merge, signal, or writeback semantics inside Query
- Query must distinguish at minimum:
  relational strategy admission, signal strategy admission, bridge writeback /
  promotion admission, and cross-family compatibility admission
- equivalent collaborative strategy posture reached through ordinary, checked,
  proof, helper, continuation, and contribution-composed lanes must converge
  on the same admitted strategy descriptor family
- strategy identity, strategy descriptor digests, strategy contract basis, and
  strategy basis must remain durable enough for later replay, certification,
  and collaborative classification work
- Query must fail closed when a caller attempts to combine unsupported or
  authority-incoherent strategy families rather than silently widening into
  "best effort" compatibility
- this phase must make future collaborative conflict classification depend on
  one Query-facing strategy-admission artifact, not on helper-local strategy
  selection heuristics

**DX target**

- a serious downstream domain should be able to ask Query "which collaborative
  strategy families are admitted here?" and receive one retained, typed answer
  instead of raw lower-runtime handles and host glue

**Documentation obligation**

- yes; this phase introduces a discoverable public strategy-admission surface
  and must teach what Query owns versus what lower crates still own

**Compile-time enforcement**

- public construction of admitted collaborative strategy descriptors must
  remain sealed
- unsupported or incoherent collaborative strategy combinations must fail
  through typed denial / deferred / unsupported lanes rather than ambient
  fallback

**Acceptance evidence**

- parity proving equivalent public lanes converge onto the same admitted
  collaborative strategy descriptor
- hostile tests proving Query cannot synthesize a strategy family, descriptor
  digest, or basis story that was not admitted by lower-authority seams
- docs, inventory rows, and support/readiness rows aligned with the new public
  strategy-admission surface

### Phase 37: Collaborative Strategy Witness And Compatibility Boundary

This phase turns the admitted strategy surface into one retained,
proof-bearing collaborative strategy witness that later collaborative entry can
consume canonically.

The hard problem is not naming one lower strategy. It is retaining the exact
cross-family strategy story, basis story, and authority story that shaped a
collaborative world so later readmission and conflict classification do not
rebuild them from scratch.

**Required Query artifacts**

- one Query-owned collaborative strategy witness family
- one typed compatibility report over admitted relational, signal, and bridge
  strategy families plus their basis / authority / policy posture
- one canonical witness identity / digest surface that converges when the same
  strategy world is retained and diverges when strategy, basis, or authority
  meaning changes
- one sealed mapping surface from the strategy witness back to existing
  routing, continuation, signal-compatibility, contribution, and recovery
  substrates

**Locked references**

- `worth-foundational` transition strategy and basis vocabulary remains
  authoritative for strategy identity, descriptor digest, contract basis,
  strategy basis, and branch-basis drift meaning
- `worth-foundational` profile identity and target-aware attachment surfaces
  remain authoritative for profile-bearing compatibility and support attachment
  semantics
- `worth-foundational` support-truth surfaces remain authoritative for
  degraded or support-grade recovery posture where this witness cannot claim
  stronger current-basis authority
- local foundational docs this phase must reuse instead of replacing:
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/transition-strategy-and-basis-semantics.md`
  - `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/transition-canonical-basis-locators-and-current-basis.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/requested-admitted-and-materialized-profile-progression.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/proof-bearing-profile-certification-and-readmission.md`
  - `crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md`
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-admission.md`
  - `crates/worth-query/docs/domain-capabilities/declaration-relational-truth-routing.md`
  - `crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md`
  - `crates/worth-query/docs/domain-capabilities/signal-compatibility-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/continuation-pipeline.md`
  - `crates/worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md`
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-witness.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-compatibility.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/application/domain_handle/operating_context.rs`
  - `crates/worth-query/src/continuation_pipeline/`
  - `crates/worth-query/src/recovery_boundary/`
  - `crates/worth-query/src/orchestration_inventory/`
  - `crates/worth-foundational/src/boundary_evidence/`
  - `crates/worth-foundational/src/canonicalization/`

**Requirements**

- Query must retain one canonical collaborative strategy witness instead of
  making later phases reconstruct admitted strategy posture from multiple
  lower-runtime artifacts
- the witness must keep strategy identity, descriptor digest, contract basis,
  strategy basis, and relevant basis-drift posture explicit
- the witness must preserve enough authority and policy posture that later
  collaborative classification can distinguish replay issues from strategy
  issues, basis issues, and authority issues without reopening host glue
- equivalent collaborative strategy worlds reached through ordinary, checked,
  proof, helper, continuation, grouped, and contribution-composed lanes must
  converge on the same witness family
- the witness must be proof-bearing and sealed; no caller-authored witness
  literals or bag-shaped compatibility contexts are allowed
- this phase must not invent a second strategy identity or basis identity
  surface when foundational vocabulary already defines one shared meaning

**DX target**

- a downstream domain should be able to ask Query "what strategy-shaped
  collaborative world have we actually retained?" and get one inspectable
  witness rather than one relational answer, one signal answer, and one bridge
  answer that must be correlated manually

**Documentation obligation**

- yes; the witness and compatibility model change the public collaborative
  story and must be teachable as one Query-facing retained artifact

**Compile-time enforcement**

- public construction of collaborative strategy witnesses must remain sealed
- helper-facing compatibility projections must compile onto the same witness
  family instead of inventing helper-local compatibility enums

**Acceptance evidence**

- parity proving equivalent collaborative strategy posture yields the same
  witness identity across the public lanes
- hostile tests proving Query does not widen or reinterpret lower-authority
  strategy identity, basis, or drift meaning
- docs and inventory rows proving the witness family is public product surface,
  not test-only scaffolding

### Phase 38: Collaborative Strategy Inspection And Preparation Boundary

This phase makes the retained collaborative strategy witness operational before
the later authority witness / conflict / recovery phases build on it.

The goal is one Query-facing seam that can inspect the retained collaborative
strategy world and prepare the next lower merge/writeback/inspection handoff
without pretending Query executed or re-owned those lower authorities.

**Required Query artifacts**

- one collaborative strategy inspection artifact family
- one typed collaborative strategy preparation request family
- one checked / proof-visible preparation result family that can stop at
  support, inspection, merge-preparation, writeback-preparation, or explicit
  incompatibility
- one explanation artifact that joins admitted strategy posture, retained
  witness posture, and next-step preparation posture

**Locked references**

- `worth-relational` remains authoritative for prepared merge execution
  semantics
- `worth-runtime-bridge` remains authoritative for writeback preparation,
  preview promotion, and merge-aware bridge publication semantics
- `worth-signal` remains authoritative for execution-side signal merge,
  invalidation, and delivery semantics
- `worth-foundational` support-truth, target-aware profile attachment, and
  proof-bearing readmission surfaces remain authoritative for how weaker
  inspection/preparation posture is represented honestly
- local foundational docs this phase must reuse instead of replacing:
  - `crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/target-aware-profile-attachment.md`
  - `crates/worth-foundational/docs/profiles-and-policy-vocabulary/proof-bearing-profile-certification-and-readmission.md`
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-witness.md`
  - `crates/worth-query/docs/domain-capabilities/continuation-pipeline.md`
  - `crates/worth-query/docs/domain-capabilities/signal-compatibility-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md`
  - `crates/worth-query/docs/domain-capabilities/explanation/lower-runtime-explanation-contributions.md`
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-inspection-and-preparation.md`
  - `crates/worth-query/docs/domain-capabilities/choosing/collaborative-strategy-vs-continuation-vs-recovery.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/continuation_pipeline/`
  - `crates/worth-query/src/signal_compatibility_orchestration/`
  - `crates/worth-query/src/recovery_boundary/`
  - `crates/worth-query/src/application/domain_handle/admitted_handle/`
  - `crates/worth-query/src/platform_entry_closeout/`

**Requirements**

- Query must expose one public strategy-facing inspection surface that tells a
  caller which collaborative strategy world was retained and which lower
  next-step families are honestly preparable now
- preparation must remain explicit and typed; prepared does not mean executed
  and inspected does not mean replayable
- preparation must preserve relational merge, bridge writeback/promotion, and
  signal-facing ownership instead of laundering them into a Query-local
  execution abstraction
- equivalent retained strategy posture must produce the same inspection and
  preparation identity across ordinary / checked / proof / helper lanes
- this phase must preserve enough retained explanation posture that later
  collaborative authority witnesses and conflict classifications do not reopen
  raw strategy-routing details from scratch

**DX target**

- a serious downstream domain should be able to ask Query "what collaborative
  strategy world do we have, and what exact next lower step is honestly
  preparable from here?" and get one typed answer

**Documentation obligation**

- yes; this phase creates a new discoverable preparation and inspection seam
  that must be taught as distinct from continuation execution and recovery

**Compile-time enforcement**

- collaborative strategy preparation requests must remain sealed to retained
  witness posture and admitted strategy families
- invalid preparation transitions must fail at the type level rather than as
  late runtime fallback

**Acceptance evidence**

- parity proving equivalent retained collaborative strategy witness posture
  yields the same inspection and preparation outputs
- hostile tests proving Query cannot claim a merge/writeback/inspection step is
  preparable when the lower-authority family was not admitted
- docs, inventory, and closeout hooks proving this is a real public strategy
  seam rather than substrate-only plumbing

### Phase 39: Collaborative Authority Witness Boundary

This phase extends the now-shipped platform-entry seam so collaborative,
branch-aware, preview-aware, and policy-aware downstream domains can enter
Query once and carry one stable collaboration witness instead of rebuilding
local authority bags above Query.

This phase does not make `worth-query` the owner of branch workflows, preview
session lifecycle, merge semantics, lineage semantics, tenant policy, or
mutation evidence. It makes Query the stable entry surface that retains and
projects those already-planned lower-authority truths coherently.

**Required Query artifacts**

- one Query-owned collaborative authority witness family
- one typed descriptor over branch/workspace basis, preview posture, lineage /
  identity continuity posture, collaborator / tenant authority posture, and
  mutation-evidence posture
- one sealed mapping surface from the collaborative witness back to the
  existing platform-entry binding, route, continuation, contribution, and
  recovery substrates

**Locked references**

- `Milestone 5.2` remains authoritative for preview-session query contexts
- `Milestone 5.5` remains authoritative for query-authored mutation / merge /
  writeback lowering
- `Milestone 6` remains authoritative for branch-scoped, historical, and diff
  truth posture
- `Milestone 7` remains authoritative for lineage / correspondence / identity
  evolution semantics
- `Milestone 9` remains authoritative for tenant schema and policy variation
- `worth-runtime-bridge` `Milestone 4` remains authoritative for historical
  and branch-aware bridge evaluation posture
- `worth-runtime-bridge` `Milestone 10` remains authoritative for speculative
  truth-branch to signal-branch coordination and preview flows
- `worth-runtime-bridge` `Milestone 11` remains authoritative for
  cross-runtime policy propagation
- the `Runtime Authoritative Mutation Evidence Gate` remains authoritative for
  mutation-evidence ownership and proof shape
- `worth-relational` remains authoritative for relational merge-strategy
  semantics and execution
- `worth-signal` remains authoritative for signal merge / invalidation /
  delivery strategy semantics and execution
- the shipped Phase 36 through 38 collaborative strategy substrate remains
  authoritative for admitted strategy families, retained strategy identity, and
  Query-facing preparation posture
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/foundations/branches-and-previews.md`
  - `crates/worth-query/docs/domain-capabilities/platform-entry.md`
  - `crates/worth-query/docs/domain-capabilities/configured-domain-handles.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-admission.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-witness.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-strategy-inspection-and-preparation.md`
  - `crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md`
  - `crates/worth-query/docs/domain-capabilities/continuation-pipeline.md`
  - `crates/worth-query/docs/domain-capabilities/contribution-composed-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/continuity/continuity-contributions-and-authoritative-successors.md`
  - `crates/worth-query/docs/domain-capabilities/workflow/preview-inspection-and-mutation-planning.md`
  - `crates/worth-query/docs/domain-capabilities/workflow/runtime-preflight-workflow-contributions.md`
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-authority-witness.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/application/domain_handle/`
  - `crates/worth-query/src/continuation_pipeline/`
  - `crates/worth-query/src/recovery_boundary/`
  - `crates/worth-query/src/orchestration_inventory/`

**Requirements**

- Query must expose one stable collaboration-facing witness without forcing
  callers to stitch together branch, preview, lineage, policy, and mutation
  facts from multiple lower crates by hand
- the collaborative witness must be proof-bearing and typed; no bag-shaped
  `HashMap`-, json-, or string-label-based authority context is allowed
- the collaborative witness must preserve the ownership split above instead of
  duplicating lower-authority semantics inside Query
- equivalent collaborative posture reached through ordinary, checked, proof,
  helper, grouped, continuation, and contribution-composed lanes must converge
  onto the same witness family
- the collaborative witness must preserve enough bridge-owned branch/history/
  preview/policy carry-forward evidence that later collaborative
  classification does not need to rediscover those facts from host glue
- the collaborative witness must retain enough strategy-facing posture that
  later collaborative readmission can lower into extensible relational and
  signal merge-strategy families without forcing domains to reopen host-local
  merge glue
- Query must provide ergonomic domain hooks for admitted relational and signal
  merge-strategy families, but those hooks must lower into lower-authority
  strategy surfaces instead of re-owning merge semantics in Query
- the witness must be rich enough to support later readmission, conflict
  classification, and recovery without reopening declaration meaning or target
  binding from scratch

**DX target**

- a downstream domain should be able to ask Query "what collaborative world am
  I in?" and receive one retained, inspectable answer instead of a pile of
  lower-runtime handles and host-local glue

**Documentation obligation**

- yes; this phase changes the public platform-entry story and must document the
  witness, what it retains, and what it intentionally does not own

**Compile-time enforcement**

- public construction of collaborative authority witnesses must remain sealed
- wrong or incomplete collaborative authority posture must fail through typed
  denial / unsupported / mismatch lanes rather than ambient fallback

**Acceptance evidence**

- parity proving equivalent collaboration-facing entry lanes converge onto the
  same witness
- hostile tests proving Query does not synthesize witness posture that was not
  admitted by lower-authority seams
- docs and inventory rows aligned with the new collaborative witness surface

### Phase 40: Collaborative Readmission And Conflict Classification Boundary

This phase turns the retained collaborative witness into one typed Query-facing
readmission and conflict-classification seam.

The hard problem is not detecting "something changed." It is preserving one
typed, authority-honest classification over concurrent collaborative posture
without making Query reimplement branch merge, preview promotion, lineage
evolution, policy, or mutation semantics.

**Required Query artifacts**

- one collaborative readmission input family over the retained collaborative
  witness
- one checked / proof-visible conflict-classification result family
- one typed classification taxonomy that distinguishes at minimum:
  `Replayable`, `Stale`, `RebindRequired`, `AuthorityMismatch`,
  `MergeInspectionRequired`, and `SemanticConflict`
- one transcript / inspection artifact explaining which retained witness facts
  drove the classification

**Locked references**

- `Milestone 5.5` remains authoritative for mutation / merge lowering and
  mutation-side inspection posture
- `Milestone 6` remains authoritative for branch/diff basis truth
- `Milestone 7` remains authoritative for lineage / correspondence semantics
- `worth-runtime-bridge` `Milestone 3` remains authoritative for
  lineage-aware subscription continuity and merge-like continuity rejection
- `worth-runtime-bridge` `Milestone 9` remains authoritative for merge-aware
  bridge semantics and multi-parent history consumption
- `worth-runtime-bridge` `Milestone 10` remains authoritative for preview /
  speculative branch coordination posture where collaborative readmission
  depends on preview-local versus authoritative branch meaning
- `worth-runtime-bridge` `Milestone 11` remains authoritative for
  cross-runtime policy propagation
- `worth-runtime-bridge` `Milestone 12` and `12b` remain authoritative for
  bridge-mediated commit strategies and extensible writeback-family protocol
  meaning
- `Runtime API Public Stabilization Gate` remains authoritative for the
  ordinary public handle / workspace / state contract that this phase must
  compose onto rather than bypass
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/domain-capabilities/signal-compatibility-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/continuation-pipeline.md`
  - `crates/worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md`
  - `crates/worth-query/docs/domain-capabilities/contribution-composed-orchestration.md`
  - `crates/worth-query/docs/domain-capabilities/grouped-authoring.md`
  - `crates/worth-query/docs/domain-capabilities/continuity/continuity-contributions-and-authoritative-successors.md`
  - `crates/worth-query/docs/domain-capabilities/explanation/lower-runtime-explanation-contributions.md`
  - `crates/worth-query/docs/foundations/branches-and-previews.md`
- `worth-relational` remains authoritative for extensible relational
  merge-strategy semantics
- `worth-signal` remains authoritative for extensible signal merge /
  invalidation / delivery strategy semantics
- the shipped Phase 36 through 38 collaborative strategy substrate remains
  authoritative for admitted strategy identity, basis story, compatibility
  posture, and preparation posture
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-readmission-and-conflict-classification.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/recovery_boundary/`
  - `crates/worth-query/src/continuation_pipeline/`
  - `crates/worth-query/src/application/domain_handle/admitted_handle/`
  - `crates/worth-query/src/orchestration_inventory/`

**Requirements**

- Query must classify collaborative readmission against retained witness truth
  rather than rebuilding declaration meaning, contribution meaning, or target
  identity from ad hoc host glue
- the classification family must remain distinct from generic denial or
  unsupported posture; collaborative conflict cannot collapse into ordinary
  "failed" bags
- the classification transcript must name enough retained facts that an
  operator or later recovery surface can tell whether the problem is replay,
  rebind, authority, merge inspection, or semantic conflict
- equivalent collaborative scenarios must classify canonically regardless of
  whether they were reached through declaration-entry, continuation,
  contribution-composed, helper, or grouped public lanes
- collaborative classification must be strategy-aware enough to distinguish
  conflicts that arise from admitted relational merge strategies versus
  admitted signal merge / delivery strategies when those lower-authority
  families would classify the scenario differently
- collaborative classification must also preserve bridge-owned distinctions for
  merge-like continuity, preview-branch mismatch, policy-propagation mismatch,
  and writeback-family mismatch where those lower-authority bridge seams admit
  distinct outcomes
- Query must expose ergonomic domain hooks that let serious downstream domains
  select or admit the relevant relational and signal merge-strategy families
  through Query-facing types instead of dropping to raw lower-runtime APIs
- Query may lower to lower-authority merge / preview / lineage / policy logic,
  but it must not silently reinterpret or override that logic

**DX target**

- a caller should be able to ask Query "can this collaborative intent be
  replayed here, and if not, what exact class of problem blocked it?" and get
  one typed answer with one explanation artifact

**Documentation obligation**

- yes; the public docs must teach the classification family as a Query entry
  surface over lower-authority truth, not as a second merge engine

**Compile-time enforcement**

- public collaborative conflict classifications must be sealed to Query-owned
  proving functions
- convenience helpers must project the same classification family rather than
  inventing helper-local conflict enums

**Acceptance evidence**

- hostile tests proving replayable vs stale vs rebind-required vs
  authority-mismatch vs merge-inspection-required vs semantic-conflict remain
  distinct
- parity proving the same retained collaborative witness yields the same
  classification across equivalent public lanes
- transcript / inventory / docs coverage proving the classification seam is
  product-grade rather than test-local

### Phase 41: Collaborative Recovery And Inspection Boundary

This phase makes collaborative conflict posture operable instead of merely
detectable.

The goal is one Query-facing recovery and inspection seam that tells callers
what to do next without forcing them to spelunk preview, merge, lineage,
policy, or mutation-evidence crates directly. It must remain a projection over
lower-authority truths, not a second authority owner.

**Required Query artifacts**

- one collaborative recovery brief family over collaborative classification
- one typed collaborative recovery request family that can express at minimum:
  replay, rebind, authority repair, merge inspection, and semantic-repair
  escalation
- one inspection surface that joins the collaborative witness, collaborative
  classification, and collaborative recovery posture into one explanation
  artifact

**Locked references**

- `Milestone 5.2` remains authoritative for preview promotion / discard /
  lifecycle semantics
- `Milestone 5.5` remains authoritative for mutation, merge, and writeback
  semantics
- `worth-runtime-bridge` `Milestone 10` remains authoritative for preview-flow
  discard / commit coordination and zero-residue preview behavior
- `worth-runtime-bridge` `Milestone 12` and `12b` remain authoritative for
  bridge-mediated writeback and extensible writeback-family recovery posture
- `Milestone 11` remains authoritative for durable continuation / saved
  artifact / restart-stable recovery stories
- `Milestone 13` remains authoritative for later certification breadth over
  collaborative and merge-aware scenarios
- local Query substrate docs that this phase must extend honestly:
  - `crates/worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md`
  - `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/runtime-installed-domains.md`
  - `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md`
  - `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/platform-entry-closeout.md`
  - `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/public-doc-coverage.md`
  - `crates/worth-query/docs/domain-capabilities/orchestration-inventory.md`
  - `crates/worth-query/docs/domain-capabilities/explanation/lower-runtime-explanation-contributions.md`
  - `crates/worth-query/docs/foundations/branches-and-previews.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-authority-witness.md`
  - `crates/worth-query/docs/domain-capabilities/collaborative-readmission-and-conflict-classification.md`
- new public docs this phase must add:
  - `crates/worth-query/docs/domain-capabilities/collaborative-recovery-and-inspection.md`
- code contexts this phase must extend instead of bypassing:
  - `crates/worth-query/src/recovery_boundary/`
  - `crates/worth-query/src/platform_entry_closeout/`
  - `crates/worth-query/src/public_doc_coverage/`
  - `crates/worth-query/src/orchestration_inventory/`
  - `crates/worth-query/src/application/domain_handle/admitted_handle/`

**Requirements**

- collaborative recovery must preserve the distinction between declaration /
  continuation / contribution / grouped posture and collaborative conflict
  posture rather than flattening them into one "fix it" bag
- recovery briefs must remain typed and machine-inspectable so later operator,
  UI, and AI surfaces can consume them directly
- collaborative inspection must reuse the shipped inventory, docs-coverage,
  and closeout mentality rather than living as a helper-only escape hatch
- collaborative recovery and inspection must preserve bridge-owned preview,
  continuity, causality, and writeback-family evidence strongly enough that
  downstream domains do not reopen raw bridge diagnostics just to explain the
  next-step class
- this phase may describe the next-step class, but it must not take ownership
  of durable replay, persisted workflow state, or full merge execution away
  from later roadmap milestones

**DX target**

- a serious downstream domain should be able to surface a trustworthy "what do
  we do now?" answer for collaborative conflicts without inventing its own
  recovery grammar above Query

**Documentation obligation**

- yes; collaborative recovery and inspection must ship with explicit examples
  and anti-patterns because this is where operator trust is won or lost

**Compile-time enforcement**

- collaborative recovery request construction must remain sealed to admitted
  collaborative classification posture
- invalid recovery transitions must be unrepresentable at the type level

**Acceptance evidence**

- hostile tests proving collaborative recovery remains typed across replay,
  rebind, authority repair, merge inspection, and semantic-repair escalation
- parity between ordinary / checked / proof / helper-facing collaborative
  recovery entrypoints when they describe the same retained collaborative stop
- inventory, docs, and certification hooks proving the collaborative recovery
  seam is part of the public platform rather than local host glue

## Remaining Phase Detail

Phases 1 through 35 now all have boundary-level requirements, and those
boundaries ship as one closed original `9.3.8` platform-entry product.

Phases 36 through 41 are the new planned collaboration-entry extension over
that shipped substrate.

Phases 36 through 38 intentionally close the missing collaborative strategy
foundation first: Query-facing strategy admission, retained strategy witness /
compatibility, and strategy-facing inspection / preparation.

Phases 39 through 41 then build the larger collaboration witness,
readmission/conflict-classification, and recovery/inspection stack on top of
that strategy substrate instead of rediscovering merge, writeback, and signal
strategy posture ad hoc inside the later collaborative phases.

This late extension therefore intentionally stops at stable Query entry,
strategy-facing preparation, collaborative readmission, classification,
recovery, and inspection veneers over already-planned lower-authority roadmap
work. It does not claim to close preview workflow ownership, merge execution
ownership, durable workflow continuation, or full certification breadth for
later collaborative milestones.

Future hostile QA may still tighten wording if later roadmap work uncovers a
teaching or sequencing gap, but `9.3.8` now has one explicit late frontier:
Query must become a stable platform entry for collaboration-aware branch /
preview / lineage / policy / mutation-evidence posture and for the admitted
strategy substrate that shapes collaborative merge / writeback / delivery
meaning before the runtime API stabilization gate and later roadmap work
consume that entry seam.
