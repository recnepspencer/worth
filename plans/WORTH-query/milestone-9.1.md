# Milestone 9.1 Engineering Spec: Query-Owned Subscription Declaration Families, Lowering, And Admission

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.md](./milestone-9.md)
>
> **Prior closeout:** [milestone-9-closeout.md](./milestone-9-closeout.md)
>
> **Next milestones:** Milestone 9.2 will own active subscription lifecycle,
> sharing, continuation, preview isolation, and query-shaped active delivery.
> Milestone 9.3 will own automatic subscription diagnostics, bridge parity, and
> runtime-backed subscription certification closure.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make subscription declaration, family
> selection, basis binding, bridge lowering, and admission first-class
> query-owned artifacts so a live query can become a bridge-facing subscription
> without host observer inference, raw CDC fallback, or one generic subscription
> kind.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-8-closeout.md](./milestone-8-closeout.md)
> - [milestone-9-closeout.md](./milestone-9-closeout.md)

## Goal

Make subscriptions first-class query artifacts by lowering admitted live query
meaning into explicit query subscription declaration families and bridge-facing
subscription plans before activation. The same canonical live query must carry
one subscription identity, one family decision, one basis binding, one admitted
bridge declaration, and one denial explanation when the combination is not
supported.

## Why This Milestone Exists

Milestone 8 made composition, saved-query freeze, and view-shape intent
query-owned. Milestone 9 made policy masking, tenant truth/schema basis,
relationship-proof admission, policy-aware execution seams, policy-aware live
admission, and delivery shape query-owned.

Those surfaces now define what a live query is allowed to observe, but they do
not yet define what a subscription is.

Without Milestone 9.1, `worth-query` can still promote a read into "live" while
letting the long-lived observation contract be assembled somewhere else:

- a host could infer subscription slices from observer state
- a server could choose a bridge subscription kind without query-visible proof
- policy and tenant basis could bind one-shot execution but drift during
  subscription admission
- grouped or inspector view shape could affect live delivery while
  subscription identity remains a generic collection lane
- saved/scope/template construction could produce the same live query but
  accidentally allocate distinct subscription identities
- unsupported combinations could quietly degrade into raw CDC or broad fallback

Milestone 9.1 therefore freezes the declaration boundary only. It answers:

- which query subscription family corresponds to this admitted live query
- which bridge declaration family it lowers into
- which slices, delivery intent, basis, policy, tenant, view, and proof
  artifacts define subscription identity
- which differences change subscription meaning versus only lifecycle instance
  identity
- why a requested subscription was admitted, denied, or deferred

It intentionally stops before active lifecycle and delivery sharing. Those
belong to Milestone 9.2.

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "add a subscribe API." It is making
  long-lived observation preserve the same canonical query meaning under
  composition, policy, tenant, basis, and view-shape pressure without relying
  on host glue.
- `arch_laws.md`: Laws 4, 7, 17, 21, 27, 30, 32, 34, 35, 40, and 41 dominate
  this milestone. Subscription declaration must be planner-owned,
  proof-bearing, bridge-honest, and incapable of skipping from live query input
  to active observer state.
- `perf_laws.md`: subscription support is only honest if family selection,
  slice width, basis binding, bridge lowering, fallback denial, and delivery
  intent width are counter-visible. Cheap subscription helpers may not conceal
  broad CDC, broad slice expansion, or hidden bridge fallback.
- `domain_laws.md`: subscription family selection, declaration identity, basis
  binding, bridge lowering, admission denial, support reporting, diagnostics,
  and certification rows are separate responsibilities. They must not collapse
  into one broad `subscription.rs` bag.
- `worth_query_vision.md`: live query promotion is a native query capability,
  and query-to-signal bridging must preserve query-shaped intent. Milestone
  9.1 turns the declaration side of that promise into concrete artifacts.
- `worth_query_roadmap.md`: Milestone 9.1 must prove query-owned subscription
  declaration families and lowering preserve canonical query meaning across
  policy, tenant, basis, and view-shape variations without inventing a second
  live-query semantics path or one fake universal subscription kind.
- `test-requirements.md`: the `Query Subscription Declaration And Lowering
  Parity Test` is the closeout proof. It requires equivalent live query inputs
  to lower to the same subscription-family declaration and bridge-facing plan,
  while meaning-changing policy, tenant, basis, and view-shape variations
  change subscription meaning explicitly.
- `milestone-8-closeout.md`: composition and view-shape semantics are already
  closed for the admitted runtime-backed surface. Milestone 9.1 must consume
  those artifacts rather than deriving subscription meaning from host helper
  shape or UI labels.
- `milestone-9-closeout.md`: policy/tenant/relationship-proof narrowing and
  live admission are already closed for the admitted runtime-backed surface.
  Milestone 9.1 must consume `NarrowedPolicyQueryArtifact`-class meaning and
  policy-aware live evidence rather than running a separate subscription policy
  path.

## Adversarial Constraint

Milestone 9.1 must survive the following hostile condition:

> The same canonical query is authored directly, through scopes, through a
> template, through an ephemeral saved-query artifact, and through an admitted
> runtime facade helper; it is policy narrowed, tenant/schema bound,
> relationship-proof admitted, view-shape lowered, and live-promoted under
> current, branch-local, runtime-historical, and admitted preview-like bases.
> Every semantically equivalent lane must lower into the same query
> subscription declaration family and the same bridge-facing subscription plan,
> while every policy, tenant, basis, view-shape, relationship-proof, or
> delivery-intent variation that changes live meaning must change subscription
> meaning explicitly or fail before activation.

If any supported path:

- infers subscription meaning from host observer state
- treats raw CDC as an acceptable substitute for query-shaped subscription
  declaration
- collapses detail, collection, grouped, inspector, bounded materialization, or
  policy-aware live families into one generic subscription kind
- chooses a bridge declaration family without a query-owned family decision
- binds subscription basis separately from the query basis and policy/tenant
  basis that admitted live execution
- lets saved/scope/template/facade paths allocate distinct subscription meaning
  for the same canonical live query
- hides unsupported slice kinds, unsupported view families, policy drift, or
  tenant schema drift behind a fallback observer
- implies durable subscription artifact reload or restart-stable subscription
  metadata before later store-backed milestones close

then Milestone 9.1 has failed.

## Product Decision Lock

- `worth-query` owns query subscription declaration-family artifacts,
  subscription identity, query-side equivalence, basis requests, bridge
  lowering requests, admission diagnostics, support reporting, and
  certification for admitted Milestone 9.1 families.
- `worth-runtime-bridge` remains authoritative for bridge subscription protocol
  semantics, including `BridgeSubscriptionDeclaration`,
  `BridgeSubscriptionDeclarationFamilyKind`, validated bridge basis binding,
  bridge signal strategy selection, bridge admission, and bridge-specific
  counters.
- `worth-signal` remains authoritative for observation execution, dependency
  tracking, invalidation, and scheduling strategies beneath the bridge-facing
  subscription plan.
- `worth-query` may select among admitted bridge declaration families, but it
  may not mint bridge protocol semantics or reinterpret bridge declarations as
  query truth.
- Query subscription identity is not raw query identity:
  - raw query identity says what the query asks for
  - live promotion identity says which live relevance contract exists
  - subscription declaration identity says which long-lived observation family,
    slices, basis, delivery intent, and bridge lowering are requested
  - active subscription identity belongs to Milestone 9.2 lifecycle work
- Subscription declaration must consume already admitted artifacts:
  - canonical query digest
  - validated or policy-narrowed query artifact
  - policy digest and authorized projection digest where policy applies
  - tenant truth and tenant schema basis digests where tenant scope applies
  - relationship-proof admission digest where proof clauses apply
  - view-shape plan digest and view delivery posture where view shape applies
  - live promotion descriptor and live family
  - basis descriptor for current, branch-local, runtime-historical, or admitted
    preview context
- No API may accept raw CDC, raw host callbacks, raw observer closures, or
  untyped subscription JSON as subscription declaration authority.
- Query-owned subscription family selection must be explicit:
  - `QuerySubscriptionFamily::DetailExact` lowers to bridge
    `DetailExact` where detail or inspector detail semantics admit exact
    projected slices
  - `QuerySubscriptionFamily::CollectionMembership` lowers to bridge
    `CollectionMembership` where ordered collection or grouped collection
    semantics admit membership and projected delta slices
  - `QuerySubscriptionFamily::BoundedMaterialization` may lower through
    collection membership plus explicit scope slices only when the bridge
    family registry admits the required slice kinds; otherwise it denies
  - additional query subscription families remain explicit debt until bridge
    and signal support can be named and certified
- View shape influences subscription declaration only through admitted
  view-shape artifacts:
  - table may map onto collection membership
  - detail may map onto detail exact
  - observed inspector and focused inspector may map onto detail exact only
    when aspect-focus and identity classification remain explicit
  - grouped/kanban may map onto collection membership only when grouped
    desired-state and grouped delta metadata are part of the query-side
    declaration artifact
  - cosmetic view labels may not affect subscription family choice
- Policy and tenant changes are subscription-meaning changes when they alter
  authorized projection, live relevance, tenant truth basis, tenant schema
  basis, relationship-proof admission, or caller-visible delivery shape.
- Subscription declaration may reject historical or preview bases where the
  live family cannot honestly bind them. Rejection is preferable to fake
  "snapshot live" semantics.
- Subscription declaration must declare one `QuerySubscriptionCostPosture`:
  - `BoundedExact`
  - `BoundedMembership`
  - `BoundedWithViewGrouping`
  - `DeniedWouldWiden`
  - `DeferredStoreBacked`
  - `DebtExplicit`
- Subscription declaration must declare one `QuerySubscriptionBasisPosture`:
  - `CurrentHead`
  - `BranchHead`
  - `RuntimeHistoricalSnapshot`
  - `PreviewScoped`
  - `DeniedUnsupportedBasis`
- Subscription declaration must declare one `QuerySubscriptionBridgePosture`:
  - `BridgeDeclarationAdmitted`
  - `BridgeFamilyUnsupported`
  - `BridgeSliceUnsupported`
  - `BridgeBasisBindingDenied`
  - `BridgeLoweringDeferred`
- Durable subscription artifact persistence, restart-stable subscription
  metadata, durable continuation checkpoints, and store-backed restart parity
  remain later-milestone debt.

Normative consequence:

- any implementation path that activates a bridge subscription without a
  query-owned declaration artifact is out of spec
- any implementation path that creates a query subscription from raw CDC
  filters, host callbacks, or observer state is out of spec
- any implementation path that treats all live queries as one subscription kind
  is out of spec
- any implementation path that computes bridge slices from unmasked query
  fields after policy narrowing is out of spec
- any implementation path that lets subscription basis drift from the query
  basis that admitted live execution is out of spec
- any implementation path that silently falls back from unsupported exact slices
  to broad collection or CDC subscription is out of spec
- any implementation path that claims durable subscription reload through
  Milestone 9.1-only artifacts is out of spec

## Typed Phase Progression Lock

Milestone 9.1 must define one proof-bearing phase chain. Subscription
declaration cannot be an ad hoc helper attached after live promotion.

Required phase progression:

- `LiveQueryAdmissionArtifact`
  - the existing policy-aware or ordinary live promotion artifact whose query,
    plan, result shape, view shape, basis, and policy meaning are already
    admitted
- `QuerySubscriptionFamilySelection`
  - the query-owned family decision with explicit equivalence basis, view
    interpretation, and cost posture
- `QuerySubscriptionDeclarationArtifact`
  - the sealed query subscription declaration containing canonical query
    identity, live family, subscription family, slice intent, delivery intent,
    policy/tenant/proof/view/basis digests, and declaration digest
- `QuerySubscriptionBasisBindingRequest`
  - the query-owned basis request to the bridge, derived from admitted query
    basis rather than host observer state
- `BridgeSubscriptionLoweringPlan`
  - the mapping from query declaration artifact to bridge
    `BridgeSubscriptionDeclaration`, bridge basis request, and admitted signal
    strategy request
- `QuerySubscriptionAdmissionArtifact`
  - the admitted or denied runtime-backed subscription declaration result,
    including bridge declaration digest, bridge basis digest, signal strategy
    digest where admitted, denial class where rejected, and exact counters
- `SubscriptionActivationInput`
  - the only artifact Milestone 9.2 may consume to create active subscription
    lifecycle objects

Rules:

- no API may create `SubscriptionActivationInput` from raw live descriptors,
  raw query plans, raw bridge declarations, or host observer state
- no API may skip family selection and lower directly into bridge declaration
  construction
- no API may mutate subscription family, delivery intent, basis, policy, tenant,
  proof, or view digests after `QuerySubscriptionDeclarationArtifact`
- no API may bind bridge subscription basis from a branch, snapshot, or preview
  value that was not already part of admitted query/live basis meaning
- no API may allow active lifecycle code to reinterpret declaration denial as a
  fallback subscription

## Compile-Time Enforcement Policy

Milestone 9.1 must classify which subscription guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible query subscription declarations with no canonical
  query digest, live family, subscription family, basis digest, and bridge
  lowering digest
- live subscription admission dimensions whose required widths can be expressed
  as zero or hidden defaults; required public dimension constructors must use
  nonzero width types and family-shaped constructors rather than loose `usize`
  bags
- publicly constructible subscription family selection that does not carry the
  admitted live family and view-shape posture it interpreted
- publicly constructible bridge lowering plans that do not carry both query
  declaration identity and bridge declaration identity
- publicly constructible admission artifacts that omit bridge basis binding,
  bridge family posture, or denial classification
- result or diagnostics bundles that collapse declaration-family changes into
  lifecycle-instance changes

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `QuerySubscriptionDeclarationArtifact`,
  `QuerySubscriptionFamilySelection`, `QuerySubscriptionBasisBindingRequest`,
  `BridgeSubscriptionLoweringPlan`, `QuerySubscriptionAdmissionArtifact`, or
  materially equivalent proof-bearing types without crate-owned lowering
- public APIs that accept raw CDC filters, raw observer callbacks, raw strings,
  raw JSON, or host-local subscription structs as subscription authority
- public APIs that accept raw integer projection, ordering, grouping,
  relation-scope, or view-shape metadata widths for required subscription
  dimensions instead of nonzero typed width evidence
- public APIs that allow `SubscriptionActivationInput` to be constructed
  without a query-owned admission artifact
- public APIs that expose bool shortcuts such as `subscribe_all`,
  `use_cdc_fallback`, `is_collection_subscription`, or `bridge_default`
- public APIs that allow policy, tenant, relationship-proof, or view-shape
  digests to be patched after subscription declaration freezes

`Construction-time rejection`:

- unsupported bridge declaration family
- unsupported bridge slice kind
- unsupported basis for a live family
- policy or tenant drift since live admission
- relationship-proof admission drift
- view-family/subscription-family mismatch
- delivery intent unsupported by bridge family
- store-backed restart or durable subscription reload request
- ambiguous equivalence basis for saved/scope/template/facade construction

## Phases

### Phase 1: Subscription Family Vocabulary And Equivalence Basis

Define the query-owned subscription vocabulary that sits between live
promotion and bridge declaration.

Must ship:

- `QuerySubscriptionFamily` with admitted initial families:
  - `DetailExact`
  - `CollectionMembership`
  - `BoundedMaterialization`
  - `GroupedCollectionMembership`
  - `InspectorDetailExact`
- `QuerySubscriptionFamilySelection`
- `QuerySubscriptionEquivalenceBasis`
- `QuerySubscriptionMeaningDigest`
- `QuerySubscriptionCostPosture`
- `QuerySubscriptionBasisPosture`
- `QuerySubscriptionBridgePosture`
- `QuerySubscriptionDeclarationCounters`

Semantic rules:

- equivalent direct/scope/template/saved/facade live inputs produce the same
  family selection and equivalence basis
- family selection consumes live family plus view-shape posture; it may not
  inspect host observer state
- grouped and inspector families are distinct query-side meanings even when
  they lower to admitted bridge families beneath them
- subscription identity changes when policy, tenant, proof, basis, view, live
  relevance, delivery intent, or slice meaning changes

Proof obligations:

- direct versus scope versus template versus saved exact-reuse family selection
  parity
- detail versus inspector and collection versus grouped distinctions are
  mechanically visible
- unsupported view-family/subscription-family pairs deny before bridge lowering
- exact counters for family selection count, family denial count, equivalence
  digest inputs, and fallback denial count

### Phase 2: Query Subscription Declaration Artifact

Freeze subscription declaration as a query-owned artifact before any bridge
activation or lifecycle concern can observe it.

Must ship:

- `QuerySubscriptionDeclarationArtifact`
- `QuerySubscriptionSliceIntent`
- `QuerySubscriptionDeliveryIntent`
- `QuerySubscriptionDeclarationDigest`
- `QuerySubscriptionDeclarationDenial`
- `QuerySubscriptionDeclarationDenialKind`
- declaration construction from:
  - ordinary live promotion descriptors
  - policy-aware live admission artifacts
  - view-shape live artifacts
  - scope/template/saved-query exact reuse artifacts

Semantic rules:

- slice intent is derived from authorized projection, ordering, grouping,
  traversal, view-shape, and live relevance artifacts only
- masked fields never appear in subscription slice intent unless Milestone 9
  admitted a purpose-specific non-disclosing witness for the exact influence
  purpose
- delivery intent is a query declaration concern, not a transport-local option
- declaration digest binds family selection, basis posture, policy digest,
  tenant digest, proof digest, view digest, delivery intent, and slice intent

Proof obligations:

- semantically equivalent construction paths produce identical declaration
  digests
- policy and tenant variations that alter authorized projection or live
  relevance alter declaration digest
- unsupported masked influence, unsupported grouping slices, and unsupported
  bounded materialization slices deny before bridge lowering
- exact counters for declared slice count, deduplicated slice count, masked
  slice denial count, delivery-intent denial count, and declaration digest part
  count

### Phase 3: Bridge Declaration And Basis Lowering

Lower query subscription declarations into bridge-native subscription
declarations and basis requests without stealing bridge authority.

Must ship:

- `BridgeSubscriptionLoweringPlan`
- `QueryToBridgeSubscriptionFamilyMap`
- `QueryToBridgeSliceMap`
- `QuerySubscriptionBasisBindingRequest`
- `QuerySubscriptionSignalStrategyRequest`
- bridge lowering diagnostics that include:
  - query declaration digest
  - bridge declaration digest
  - bridge family kind
  - bridge slice kinds
  - basis request kind
  - signal strategy request
  - lowering denial class

Semantic rules:

- `DetailExact` and `InspectorDetailExact` lower to bridge
  `BridgeSubscriptionDeclarationFamilyKind::DetailExact` only when exact
  projected slices are admitted
- `CollectionMembership` and `GroupedCollectionMembership` lower to bridge
  `BridgeSubscriptionDeclarationFamilyKind::CollectionMembership` only when
  membership, ordering, grouping, and projected delta slices are explicit
- `BoundedMaterialization` lowers only when the bridge registry admits required
  region, partition, relation, or facet slice kinds; otherwise it denies
- query basis requests lower to bridge snapshot or branch-head basis requests
  only from already admitted query basis meaning
- runtime-historical or preview-scoped bases deny when the bridge cannot bind
  them honestly for the selected family

Proof obligations:

- every admitted query family maps to one explicit bridge family and slice set
- unsupported bridge family or slice kind produces typed denial before
  activation
- bridge declaration digest is stable for equivalent query declaration inputs
- basis binding request digest changes when branch, snapshot, policy epoch, or
  tenant basis changes
- exact counters for bridge family lowering count, bridge slice count, bridge
  slice denial count, basis binding count, basis denial count, and signal
  strategy request count

### Phase 4: Admission, Diagnostics, Support, And Certification

Seal the admitted runtime-backed declaration result and prove it can be
consumed by Milestone 9.2 without reinterpreting query meaning.

Must ship:

- `QuerySubscriptionAdmissionArtifact`
- `SubscriptionActivationInput`
- `QuerySubscriptionAdmissionDiagnostics`
- `QuerySubscriptionSupportProfile`
- `QuerySubscriptionCertificationBundle`
- `harness/milestone_nine_one_certification`
- compile-fail tests for declaration fabrication and activation bypass

Semantic rules:

- activation input exists only after query declaration, bridge lowering, basis
  binding, and signal strategy request are admitted
- diagnostics can localize failures to family selection, declaration, bridge
  family lowering, slice lowering, basis binding, policy/tenant drift, view
  mismatch, delivery intent, or durable overclaim
- support profile truth derives from executable admission/certification rows,
  not feature labels
- store-backed restart and durable reload requests remain explicit denial or
  debt surfaces

Proof obligations:

- the `Query Subscription Declaration And Lowering Parity Test` passes with
  canonical bundles
- equivalent live query inputs lower to equal query declaration and bridge plan
  digests
- policy, tenant, basis, and view-shape variations that change live meaning
  change subscription meaning explicitly
- unsupported or ambiguous subscription bindings fail before activation
- no admitted path interprets raw CDC or one baked-in subscription kind as
  query-shaped subscription intent

## Must Ship

- query-owned subscription family vocabulary and equivalence basis
- sealed query subscription declaration artifacts
- query-owned slice intent and delivery intent derived from admitted live,
  policy, tenant, relationship-proof, view, and basis artifacts
- bridge-lowering plan into admitted bridge declaration families, slice kinds,
  basis requests, and signal strategy requests
- subscription admission artifact and activation input for Milestone 9.2
- denial taxonomy for unsupported family, unsupported slice, unsupported basis,
  policy/tenant drift, relationship-proof drift, view mismatch, delivery intent
  mismatch, bridge lowering failure, raw CDC fallback, host observer inference,
  and durable overclaim
- diagnostics, support profile, exact counters, certification bundle, and
  compile-fail proof boundaries

## Must Preserve

- query semantics remain owned by `worth-query`
- bridge subscription protocol semantics remain owned by
  `worth-runtime-bridge`
- signal execution strategy remains owned by `worth-signal` and bridge-facing
  strategy selection
- subscription declaration does not become active lifecycle; that is Milestone
  9.2
- active lifecycle code must consume `SubscriptionActivationInput`, not raw
  live promotion descriptors
- policy-aware and tenant-aware narrowing remain single-source artifacts from
  Milestone 9
- view-shape semantics remain single-source artifacts from Milestone 8
- durable subscription persistence and restart parity remain later debt

## Complexity / Proof Obligations

Performance is part of the architecture, not an after-the-fact benchmark. Every
phase must carry a proof of what work it is allowed to do, and later phases
must consume that proof instead of rediscovering cost.

Required performance proof artifacts:

- `QuerySubscriptionWorkBudget`
  - carried by `QuerySubscriptionFamilySelection`
  - declares maximum admitted slice count, authorized projection width,
    view-shape metadata width, policy/tenant digest width, bridge family map
    lookup count, and whether allocation is allowed
- `QuerySubscriptionSliceBudget`
  - carried by `QuerySubscriptionDeclarationArtifact`
  - declares projected slice width, ordering slice width, grouping slice width,
    relation/scope slice width, deduplication input width, deduplicated output
    width, and masked-slice denial count
- `QuerySubscriptionBridgeLoweringBudget`
  - carried by `BridgeSubscriptionLoweringPlan`
  - declares bridge family registry lookup count, bridge slice-kind lookup
    count, bridge declaration input width, basis request width, signal strategy
    request width, and fallback posture
- `QuerySubscriptionAdmissionBudget`
  - carried by `QuerySubscriptionAdmissionArtifact`
  - declares the final admitted work envelope that Milestone 9.2 may consume;
    active lifecycle may not widen it without a new admission surface
- `QuerySubscriptionScaleSlopeReport`
  - emitted by certification
  - compares small, medium, and larger fixture sizes for the same semantic
    query families and proves the declared slopes remain stable

Budget rules:

- every public phase output must expose its budget and counter snapshot through
  read-only accessors
- a later phase may narrow a budget but may not widen it silently
- a phase that cannot prove its budget must deny with `UnknownSubscriptionCost`
  or `SubscriptionWorkBudgetExceeded`
- budget proof must be constructed before bridge declaration construction; the
  bridge cannot be used as the place where query cost is discovered
- registry and mapping lookup counts must be explicit; hidden scans over all
  bridge families, all view families, all saved-query artifacts, or all policy
  masks are out of spec
- allocation policy must be explicit:
  - `NoAllocation` for pure digest comparison and equivalence checks
  - `ScratchBufferOnly` for canonicalization/deduplication inside one phase
  - `DeniedAllocationRequired` when the requested declaration would require
    active buffers, fanout state, checkpoints, continuation indexes, or
    per-slice heap allocation
- `Vec` usage is acceptable only inside a phase-owned scratch lifecycle that is
  consumed into a proof-bearing artifact; public proof artifacts expose slices
  or fixed ownership, not mutable accumulation surfaces
- digest construction must use canonical ordered inputs; sorting cost must be
  bounded by declared input width and counted separately from bridge lowering
- deduplication must happen once at declaration construction; bridge lowering
  must consume the deduplicated proof rather than deduplicating again

Named contracts:

- `QuerySubscriptionFamilySelectionContract`
  - bounded by live family count, view-shape descriptor width, policy/tenant
    digest width, and admitted family registry width
- `QuerySubscriptionDeclarationContract`
  - bounded by declared slice count, delivery intent width, digest part count,
    and authorized projection width
- `QuerySubscriptionBridgeLoweringContract`
  - bounded by bridge family mapping count, bridge slice count, basis request
    width, and signal strategy request width
- `QuerySubscriptionAdmissionContract`
  - bounded by one declaration artifact, one bridge lowering plan, one basis
    binding result, and one signal strategy request per admission
- `QuerySubscriptionScaleSlopeContract`
  - bounded by fixture row count, projected field width, grouped key count,
    relation-proof width, bridge slice count, and declaration digest part count
  - certification must prove constant factors stay tied to declared structural
    widths rather than ambient entity count where the query family should not
    scan all entities

Required counters:

- `subscription_family_selection_count`
- `subscription_family_denial_count`
- `subscription_family_registry_lookup_count`
- `subscription_view_family_registry_lookup_count`
- `subscription_equivalence_digest_part_count`
- `subscription_declaration_count`
- `subscription_declaration_denial_count`
- `subscription_declared_slice_count`
- `subscription_deduplicated_slice_count`
- `subscription_slice_deduplication_input_count`
- `subscription_slice_sort_comparison_count`
- `subscription_masked_slice_denial_count`
- `subscription_delivery_intent_denial_count`
- `subscription_work_budget_denial_count`
- `subscription_unknown_cost_denial_count`
- `subscription_bridge_lowering_count`
- `subscription_bridge_family_denial_count`
- `subscription_bridge_family_registry_lookup_count`
- `subscription_bridge_slice_count`
- `subscription_bridge_slice_denial_count`
- `subscription_bridge_slice_registry_lookup_count`
- `subscription_basis_binding_request_count`
- `subscription_basis_binding_denial_count`
- `subscription_signal_strategy_request_count`
- `subscription_raw_cdc_fallback_denial_count`
- `subscription_host_observer_inference_denial_count`
- `subscription_durable_overclaim_denial_count`
- `subscription_activation_input_count`
- `subscription_active_state_allocation_denial_count`
- `subscription_declaration_time_checkpoint_denial_count`
- `subscription_scratch_allocation_count`
- `subscription_forbidden_heap_allocation_denial_count`
- `subscription_scale_fixture_row_count`
- `subscription_scale_slope_digest_part_count`

Counter rules:

- exact counts are required in certification rows; threshold-based elapsed time
  assertions do not satisfy this milestone
- bridge fallback count must remain zero for admitted rows
- raw CDC fallback denial and host observer inference denial must be distinct
  counters
- executor semantic rediscovery count must remain zero in subscription
  declaration and lowering rows
- declaration digest part count must change when policy, tenant, basis, view,
  proof, or delivery meaning changes
- active lifecycle allocation, fanout allocation, checkpoint allocation, and
  continuation allocation counts must remain zero for declaration/admission
  rows; attempts to create them in 9.1 must increment denial counters instead
- registry lookup counts must be exact and bounded by admitted family maps, not
  by total registered runtime entities or total active subscriptions
- bridge lowering must not rescan query projection, policy masks, saved-query
  registries, or view-shape registries; if it needs those facts, the prior proof
  type is incomplete
- scale-slope rows must prove detail exact, collection membership,
  grouped membership, and inspector detail declaration cost grows only with
  declared projection/slice/grouping/proof widths, not with unrelated fixture
  row count
- forbidden heap allocation, active-state allocation, and checkpoint allocation
  counters must be zero in admitted lanes and exactly one in their hostile
  denial lanes

## Acceptance Evidence

Milestone 9.1 is complete only when `worth-query` can prove:

- the `Query Subscription Declaration And Lowering Parity Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- equivalent direct, scope-composed, template-instantiated, saved-exact, and
  facade-authored live query inputs lower to the same
  `QuerySubscriptionDeclarationArtifact`
- admitted query subscription declarations lower to explicit bridge
  `BridgeSubscriptionDeclaration` families and basis requests
- policy, tenant, relationship-proof, basis, and view-shape differences that
  change live meaning also change subscription declaration meaning explicitly
- unsupported or ambiguous subscription bindings fail before activation
- no admitted path interprets raw CDC, host observer callbacks, or one fixed
  baked-in subscription kind as a substitute for query-shaped subscription
  intent
- durable subscription artifact persistence, restart-stable reload, durable
  continuation, and store-backed restart parity stay explicit debt

Required verification output must include:

- `query_digest`
- `live_family_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `policy_digest`
- `tenant_basis_digest`
- `relationship_proof_digest`
- `view_shape_digest`
- `basis_digest`
- `bridge_declaration_digest`
- `bridge_basis_digest`
- `signal_strategy_digest`
- `admission_digest`
- `failure_digest`
- `fixture_digest`
- `compile_fail_boundary_digest`
- `counter_snapshot`
- `support_matrix_digest`

## Representative Scenario Matrix

Minimum admitted rows:

- `direct-scope-template-saved-subscription-parity`
  - equivalent direct, scope-composed, template-instantiated, and saved-exact
    live inputs produce the same subscription family and declaration digests
- `facade-helper-subscription-parity`
  - runtime facade helper construction lowers to the same declaration as direct
    canonical live input
- `detail-exact-bridge-lowering`
  - admitted detail live query lowers to query `DetailExact` and bridge
    `DetailExact`
- `inspector-detail-exact-bridge-lowering`
  - admitted observed/focused inspector lowers through exact detail semantics
    while preserving inspector view and identity classification digests
- `collection-membership-bridge-lowering`
  - admitted ordered collection lowers to query `CollectionMembership` and
    bridge `CollectionMembership`
- `grouped-membership-bridge-lowering`
  - admitted grouped/kanban live query lowers to collection membership with
    grouped desired-state and grouped delta metadata bound into declaration
    meaning
- `bounded-materialization-scope-slice-admission`
  - admitted bounded materialization lowers only when required bridge slice
    kinds are supported
- `policy-masked-subscription-declaration`
  - masked policy basis removes unauthorized fields from slice intent and
    changes subscription declaration meaning compared to unmasked policy
- `tenant-basis-subscription-declaration`
  - tenant truth/schema basis digests bind into subscription declaration and
    basis request
- `relationship-proof-subscription-declaration`
  - admitted proof descriptor binds into declaration; broken proof denies
    before bridge lowering
- `branch-head-basis-lowering`
  - admitted branch-local live query produces bridge branch-head basis request
- `snapshot-basis-lowering`
  - admitted runtime-historical or snapshot-bound query produces bridge
    snapshot basis request where the live family supports it
- `signal-strategy-request-lowering`
  - admitted query declaration emits signal strategy request matching selected
    bridge family and slice intent

Minimum rejection rows:

- `raw-cdc-fallback-forbidden`
- `host-observer-inference-forbidden`
- `generic-subscription-kind-forbidden`
- `unsupported-view-family-subscription-forbidden`
- `unsupported-bridge-family-forbidden`
- `unsupported-bridge-slice-kind-forbidden`
- `unsupported-basis-for-live-family-forbidden`
- `masked-slice-after-policy-forbidden`
- `policy-tenant-drift-after-live-admission-forbidden`
- `relationship-proof-drift-after-live-admission-forbidden`
- `saved-query-subscription-equivalence-ambiguity-forbidden`
- `bridge-basis-mismatch-forbidden`
- `durable-subscription-reload-deferred`
- `store-backed-restart-parity-deferred`
- `activation-without-query-admission-forbidden`
- `declaration-time-active-state-allocation-forbidden`
- `declaration-time-checkpoint-allocation-forbidden`

## Concrete First-Ship Fixture

Milestone 9.1 must not close against abstract declaration labels alone. It
needs one concrete fixture that forces subscription identity, slice selection,
policy masking, tenant scope, view shape, and bridge lowering to become
mechanically visible.

Use the Milestone 9 `EmployeeRecord` policy fixture as the first-ship scenario
source. If implementation cannot reuse that fixture directly, the 9.1 fixture
must preserve the same semantic shape and document the compatibility mapping in
the certification bundle.

Required fixture shape:

- entity type: `EmployeeRecord`
- visible aspects:
  - `identity.employee_id`
  - `profile.display_name`
  - `profile.department`
  - `management.manager_id`
- masked aspect/field:
  - `compensation.salary_band`
- relation/proof path:
  - `employee -> department -> manager`
- tenant variants:
  - `TenantAlpha` has `compensation.salary_band`, masked for ordinary users
  - `TenantBeta` has an incompatible or differently shaped compensation basis
- branch/basis variants:
  - current head
  - named branch head
  - runtime snapshot
  - unsupported durable restart basis

Required admitted subscription declarations:

- detail subscription over `identity.employee_id`, `profile.display_name`, and
  `profile.department` lowers to query `DetailExact` and bridge `DetailExact`
- table subscription ordered by `profile.display_name` lowers to query
  `CollectionMembership` and bridge `CollectionMembership`
- grouped subscription by `profile.department` lowers to query
  `GroupedCollectionMembership` and bridge `CollectionMembership`, with grouped
  desired-state metadata included in the query declaration digest
- focused inspector subscription over `management.manager_id` lowers to query
  `InspectorDetailExact` and bridge `DetailExact`, with inspector identity
  classification digest included

Required performance declarations for those lanes:

- detail exact:
  - family registry lookups: exactly one query family lookup and one bridge
    family lookup
  - slice width: equal to authorized projected field count
  - allocation: `NoAllocation` or `ScratchBufferOnly`, never per-field heap
    allocation
- table collection membership:
  - slice width: projected field count plus ordering field count plus one
    membership slice
  - declaration cost must not scale with collection row count
- grouped collection membership:
  - slice width: projected field count plus grouping key count plus one
    membership slice plus grouped metadata width
  - grouped metadata width must be counted explicitly; hidden grouped desired
    state materialization is forbidden in 9.1
- focused inspector detail:
  - slice width: focused aspect field count plus identity classification width
  - identity classification is metadata width, not an extra bridge scan

Required hostile fixture lanes:

- attempted detail subscription that includes `compensation.salary_band` after
  masking denies before query slice intent exists
- attempted table subscription ordered by `compensation.salary_band` denies
  before bridge lowering because row position would leak masked truth
- attempted grouped subscription by `compensation.salary_band` denies before
  bridge lowering because group membership/counts would leak masked truth
- attempted relationship-proof subscription with a broken
  `employee -> department -> manager` chain denies before bridge lowering
- attempted `TenantAlpha` saved subscription exact reuse under `TenantBeta`
  denies or requires fresh declaration; it may not reuse the old declaration
  digest
- attempted durable restart subscription request emits explicit
  `DeferredStoreBacked` or durable-overclaim denial, not an admitted activation
  input

Fixture proof rules:

- every admitted fixture lane must emit both query-side and bridge-side digests
- every hostile lane must emit a typed failure digest and exact denial counter
- no fixture lane may compare a digest only to itself
- direct, scope, template, saved-exact, and facade-authored variants must all
  be represented for at least the detail and collection families
- grouped and inspector fixture lanes must prove they are distinct query-side
  subscription meanings even when they lower onto bridge families shared with
  ordinary collection or detail subscriptions
- fixture scale must run at small, medium, and larger row counts while keeping
  projection width, grouping key width, and relation-proof width explicit
- detail and inspector declaration counters must not grow with unrelated row
  count
- collection and grouped declaration counters may grow only with declared slice
  and grouping metadata width, not with all rows in the fixture
- bridge family and slice registry lookup counters must stay constant for the
  same admitted family regardless of fixture row count
- any slope drift changes `QuerySubscriptionScaleSlopeReport` and fails the
  certification row

## Public Facade And Typestate API Shape

Milestone 9.1 must leave an implementation path that is obvious enough for the
compiler to enforce. The public surface should expose only facade-level
functions over proof-bearing input and output types; internal constructors stay
private.

Required facade shape, subject to local naming adjustment:

```rust
pub fn select_query_subscription_family(
    live: LiveQueryAdmissionArtifact,
    budget: QuerySubscriptionWorkBudget,
) -> Result<QuerySubscriptionFamilySelection, QuerySubscriptionFamilySelectionError>;

pub fn declare_query_subscription(
    selection: QuerySubscriptionFamilySelection,
    slice_budget: QuerySubscriptionSliceBudget,
) -> Result<QuerySubscriptionDeclarationArtifact, QuerySubscriptionDeclarationError>;

pub fn lower_query_subscription_to_bridge(
    declaration: QuerySubscriptionDeclarationArtifact,
    lowering_budget: QuerySubscriptionBridgeLoweringBudget,
) -> Result<BridgeSubscriptionLoweringPlan, QuerySubscriptionBridgeLoweringError>;

pub fn admit_query_subscription(
    lowering: BridgeSubscriptionLoweringPlan,
    admission_budget: QuerySubscriptionAdmissionBudget,
) -> Result<QuerySubscriptionAdmissionArtifact, QuerySubscriptionAdmissionError>;

pub fn prepare_subscription_activation(
    admission: QuerySubscriptionAdmissionArtifact,
) -> SubscriptionActivationInput;
```

Rules:

- each function consumes the prior proof type and returns the next proof type
- each fallible phase also consumes the budget proof for that phase; the phase
  may deny but may not invent an implicit budget
- no function accepts raw query plans, raw live descriptors, raw bridge
  declarations, raw bridge basis requests, raw CDC filters, or host observer
  callbacks
- live admission helper constructors must require an explicit
  `QuerySubscriptionAdmissionDimensions` value whose required family widths are
  statically nonzero; missing or mismatched dimension shapes must deny before
  slice counting, digest construction, or bridge lowering
- no function takes booleans to choose family, fallback, delivery, grouping, or
  bridge behavior
- `prepare_subscription_activation` may not return `Result`; if admission
  succeeded, activation input construction must be infallible and purely
  structural
- failure must happen before the next proof type is constructed, not after a
  partially usable artifact exists
- budget failure must be typed separately from semantic denial so performance
  regressions cannot masquerade as unsupported query meaning
- diagnostics-rich variants may wrap these functions, but may not expose weaker
  inputs

Compile-time consequence:

- there is no callable path from `LiveQueryAdmissionArtifact` directly to
  `SubscriptionActivationInput`
- there is no callable path from `BridgeSubscriptionDeclaration` directly to
  `SubscriptionActivationInput`
- there is no callable path from `QuerySubscriptionDeclarationArtifact` directly
  to active lifecycle without bridge lowering and admission
- adding a new required declaration field must break construction at every
  internal phase boundary until the field is propagated

## Bridge Mapping Table

The initial query-to-bridge mapping must be explicit enough that implementation
cannot hide a broad fallback behind bridge lowering.

| Query subscription family | Required bridge family | Required bridge slice posture | Denial if missing |
| --- | --- | --- | --- |
| `DetailExact` | `BridgeSubscriptionDeclarationFamilyKind::DetailExact` | one or more `SignalField` or admitted exact detail slice intents derived from authorized projection | `UnsupportedBridgeSliceKind` |
| `InspectorDetailExact` | `BridgeSubscriptionDeclarationFamilyKind::DetailExact` | exact detail slices plus inspector identity/view digest in query declaration, not bridge protocol meaning | `ViewFamilySubscriptionMismatch` or `UnsupportedBridgeSliceKind` |
| `CollectionMembership` | `BridgeSubscriptionDeclarationFamilyKind::CollectionMembership` | membership slice plus projected delta/order slice intents, with no hidden full-collection refresh | `UnsupportedBridgeSliceKind` |
| `GroupedCollectionMembership` | `BridgeSubscriptionDeclarationFamilyKind::CollectionMembership` | collection membership plus grouped desired-state and grouped delta metadata in query declaration digest | `GroupedBridgeMetadataMissing` |
| `BoundedMaterialization` | `CollectionMembership` only where admitted | region, partition, relation, or facet slice kinds must be admitted by bridge registry | `UnsupportedBridgeSliceKind` or `DeniedWouldWiden` |

Mapping rules:

- bridge family selection is data, not a match statement hidden in facade glue
- bridge slice kinds must come from bridge-admitted vocabulary such as
  `SignalField`, `SignalLens`, `SignalRegion`, `SignalPartition`,
  `SignalFacet`, or `RegisteredCoarseFallback`; query may only use the subset
  admitted for the selected bridge family
- `RegisteredCoarseFallback` is denial by default for Milestone 9.1 unless the
  selected family has an explicit certified fallback row and a nonzero fallback
  counter expectation
- grouped metadata is query declaration meaning; bridge must not be asked to
  understand grouped query semantics unless a later bridge family explicitly
  owns that protocol
- inspector metadata is query declaration meaning; bridge exact-detail protocol
  must not be polluted with inspector-specific semantic claims
- bridge-lowering diagnostics must state whether a query-side semantic is
  encoded in bridge protocol, retained in query declaration metadata, or denied
  because neither layer owns it

## Compile-Fail Boundary Matrix

Milestone 9.1 must add compile-fail coverage for the traps most likely to
become accidental public API.

Required trybuild targets:

- `subscription_declaration_constructor_private.rs`
- `subscription_family_selection_constructor_private.rs`
- `subscription_bridge_lowering_plan_constructor_private.rs`
- `subscription_admission_artifact_constructor_private.rs`
- `subscription_activation_input_constructor_private.rs`
- `subscription_raw_live_descriptor_activation_forbidden.rs`
- `subscription_raw_bridge_declaration_activation_forbidden.rs`
- `subscription_raw_cdc_filter_declaration_forbidden.rs`
- `subscription_host_observer_callback_forbidden.rs`
- `subscription_bool_family_shortcut_forbidden.rs`
- `subscription_masked_slice_intent_constructor_forbidden.rs`
- `subscription_saved_exact_reuse_without_equivalence_forbidden.rs`
- `subscription_bridge_basis_request_without_query_basis_forbidden.rs`
- `subscription_policy_digest_patch_forbidden.rs`
- `subscription_tenant_digest_patch_forbidden.rs`
- `subscription_relationship_proof_digest_patch_forbidden.rs`
- `subscription_durable_reload_admission_forbidden.rs`
- `subscription_generic_kind_fallback_forbidden.rs`

Each compile-fail target must prove construction is impossible through the
public facade, not merely that one helper function rejects a value at runtime.

## Proposed Module Topology

Prefer focused modules that mirror responsibility boundaries:

```text
crates/worth-query/src/subscription/
  mod.rs
  family.rs
  equivalence.rs
  declaration.rs
  slice_intent.rs
  delivery_intent.rs
  basis.rs
  bridge_lowering.rs
  admission.rs
  diagnostics.rs
  support.rs
  counters.rs
  tests.rs
  facade.rs

crates/worth-query/src/harness/milestone_nine_one_certification/
  mod.rs
  matrix.rs
  tests.rs
```

The `subscription` module must not become an active lifecycle manager. It
owns declaration and admission only. Active handles, sharing, fanout,
continuation, preview discard, and delivery windows belong to Milestone 9.2.

## Store Dependency

- Runtime-backed subscription declaration, bridge lowering, basis request, and
  admission are not blocked on `worth-store`.
- Store-backed subscription execution parity remains Milestone 10 scope.
- Durable subscription artifact persistence, durable subscription reload,
  durable continuation checkpoints, and restart-stable subscription metadata
  remain Milestone 11 scope.
- Milestone 9.1 may emit store/debt posture artifacts, but it may not claim
  durable replay or restart survival.

## Explicit Assumptions And Deferred Decisions

These assumptions are load-bearing and must be encoded as types, support
metadata, or denial rows rather than left as implementer intuition.

- bridge currently exposes `DetailExact` and `CollectionMembership` declaration
  families as the admitted protocol families for this milestone; query-side
  grouped, inspector, and bounded-materialization meaning must therefore remain
  query metadata unless bridge later adds a dedicated family
- bridge currently binds subscription basis through snapshot or branch-head
  requests; query-side current, branch-local, and runtime snapshot bases must
  lower to those shapes or deny
- preview-scoped subscription declaration is admitted only if an existing
  preview basis can be represented as one of the bridge-admitted basis request
  shapes without losing preview lifecycle identity; otherwise it is explicit
  Milestone 9.2/9.3 debt
- runtime-historical snapshot subscription declaration is a snapshot-basis
  declaration, not durable historical replay; durable restart and
  snapshot-plus-tail survival remain Milestone 10/11 scope
- policy, tenant, and relationship-proof drift after live admission are not
  repaired inside subscription declaration; Milestone 9.1 consumes immutable
  admitted digests, rejects relationship-proof posture drift before family
  lookup, and makes public digest patching uncompilable. A changed policy,
  tenant, or proof context must mint a fresh admitted live artifact and produce
  distinct subscription meaning or an upstream typed denial.
- grouped desired-state metadata and inspector identity classification are
  preserved in query declaration digests; bridge declaration digests are not
  required to encode those query-only semantics
- sharing and deduplication are not in scope except for producing the
  equivalence digest that Milestone 9.2 will consume
- no subscription declaration may allocate or retain active buffers, windows,
  fanout state, acknowledgement frontiers, checkpoints, or continuation
  indexes; those belong to active lifecycle and retained delivery milestones

If any assumption changes during implementation, the spec must be updated
before the code claims support. Silent expansion from assumption to behavior is
architectural drift.

## Explicit Failure Taxonomy

- unsupported query subscription family
- unsupported view/subscription family pairing
- unsupported bridge declaration family
- unsupported bridge slice kind
- grouped bridge metadata missing
- unsupported subscription basis
- bridge basis mismatch
- policy basis drift after live admission
- tenant basis drift after live admission
- relationship-proof admission drift
- masked slice intent after policy narrowing
- delivery intent unsupported by bridge family
- raw CDC fallback attempt
- host observer inference attempt
- generic subscription kind attempt
- ambiguous subscription equivalence
- activation without query admission
- active lifecycle state allocation attempt
- declaration-time fanout or checkpoint allocation attempt
- durable subscription reload overclaim
- store-backed restart parity overclaim

## Anti-Patterns Explicitly Rejected

- `subscribe(query)` secretly choosing bridge observer behavior from host state
- raw CDC filters as the subscription declaration authority
- one generic subscription kind for all live families
- bridge subscription declarations built directly from unmasked query fields
- view labels changing subscription behavior without view-shape artifacts
- saved-query subscription reuse without equivalence classification
- lifecycle handles minted from raw live promotion descriptors
- declaration code allocating active delivery buffers, fanout state,
  acknowledgement frontiers, checkpoints, or continuation indexes
- bridge protocol digests used as a substitute for query declaration digests
- durable subscription claims through runtime-backed declaration artifacts
- one mega-module mixing declaration, active lifecycle, sharing, preview,
  continuation, delivery windows, diagnostics, and certification

## Sequencing Notes

Milestone 9.1 belongs immediately after Milestone 9 because subscription
declaration must consume policy-safe, tenant-safe, relationship-proof-aware,
view-shaped live meaning. Building subscription declaration before policy and
tenant closure would have baked unauthorized or ambient basis semantics into
the long-lived observation surface.

It belongs before Milestone 9.2 because active subscription lifecycle and
sharing need one canonical declaration and one activation input. Otherwise
deduplication, fanout, preview isolation, and continuation would have to infer
meaning from runtime handles.

It belongs before Milestone 9.3 because diagnostics and bridge parity can only
certify a subscription surface after declaration and admission artifacts exist.

It belongs before Milestone 10 because store-backed parity should extend one
already bridge-honest runtime-backed subscription declaration model rather than
discovering subscription families per backend.

## Parallelization Notes

Once Phase 1 freezes family vocabulary and equivalence basis:

- declaration artifact construction can proceed in parallel with bridge
  lowering maps
- policy/tenant/view-shape subscription rows can proceed in parallel with
  bridge-family denial rows
- compile-fail enforcement can proceed in parallel with certification matrix
  construction
- final closure should wait until equivalent construction paths, bridge
  lowering, basis binding, support reporting, and denial counters all agree on
  the same declaration semantics

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes the missing boundary between admitted live
query meaning and bridge-facing long-lived subscription declaration.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where subscriptions appear to work only because host observer state, raw
CDC filters, or one generic bridge kind reconstructs meaning after query
admission.

The milestone preserves authority boundaries because `worth-query` owns query
subscription declaration and equivalence, while `worth-runtime-bridge` owns
bridge subscription protocol semantics and `worth-signal` owns observation
execution strategy.

The milestone defines proof obligations rather than implementation chores
because parity, bridge lowering, basis binding, denial taxonomy, exact counters,
compile-fail boundaries, and certification bundles are all required before
closeout.

A competent engineer should be able to map this spec into honest
`subscription::family`, `subscription::equivalence`,
`subscription::declaration`, `subscription::bridge_lowering`,
`subscription::admission`, `subscription::diagnostics`, support, facade,
compile-fail, and certification subdomains without inventing architecture
during implementation.

This milestone belongs at 9.1 because it is the declaration/admission layer
between policy-safe live query meaning and active subscription lifecycle.

## Closeout Standard

Milestone 9.1 is complete only when all of the following are true:

- query subscription family selection is explicit and proof-bearing
- query subscription declarations bind canonical query, live family, policy,
  tenant, relationship-proof, view-shape, basis, delivery intent, and slice
  intent meaning
- admitted declarations lower into explicit bridge declaration families and
  bridge basis requests
- equivalent direct/scope/template/saved/facade live inputs produce identical
  subscription declarations
- meaning-changing policy, tenant, basis, proof, or view-shape variations
  produce distinct declaration meaning or typed denial
- unsupported bridge families, unsupported slices, unsupported bases, raw CDC
  fallback, host observer inference, and generic subscription kinds fail typed
  before activation
- `SubscriptionActivationInput` is the only handoff to Milestone 9.2 active
  lifecycle
- durable subscription persistence and restart-stable reload remain explicit
  debt

If code lands but active lifecycle can still start from raw live descriptors,
host observer state can still infer subscription meaning, bridge declarations
can still be built from unmasked query fields, or unsupported subscription
families can still fall back to raw CDC, Milestone 9.1 is not complete.
