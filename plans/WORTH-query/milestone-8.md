# Milestone 8 Engineering Spec: Scopes, Templates, Saved Query Artifacts, And View-Shape Semantics

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-7.md](./milestone-7.md)
>
> **Adjacent milestones:** [milestone-6.md](./milestone-6.md) and
> [milestone-7.md](./milestone-7.md) are already closed and remain
> authority-distinct inputs for basis-explicit branch/history/diff semantics
> and lineage/correspondence identity semantics. Milestone 5 and Milestone 5.1
> are also already closed and remain authority-distinct inputs for live
> promotion, ordered collection maintenance, locality-aware invalidation, and
> stream-contract lowering.
>
> **Prior closeouts:** [milestone-6-closeout.md](./milestone-6-closeout.md)
> and any later Milestone 7 closeout artifact when it exists
>
> **Closeout:** [milestone-8-closeout.md](./milestone-8-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make reusable query composition and
> presentation intent first-class query-owned artifacts so direct queries,
> scope-composed queries, template-instantiated queries, ephemeral saved-query
> artifacts, and admitted view shapes all preserve one canonical query meaning
> while materially affecting planning, live invalidation, delivery shape, and
> patch semantics
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.md](./milestone-5.md)
> - [milestone-5.1.md](./milestone-5.1.md)
> - [milestone-6.md](./milestone-6.md)
> - [milestone-7.md](./milestone-7.md)

## Goal

Make scopes, templates, ephemeral saved-query artifacts, and admitted
view-shape semantics first-class query-owned artifacts so reusable query
composition remains canonically equivalent to direct construction and declared
view intent becomes planner-visible, live-visible, delivery-visible, and
certification-visible rather than cosmetic typing layered on top of ordinary
query execution.

## Why This Milestone Exists

Milestone 6 made truth basis explicit. Milestone 7 made identity evolution
explicit. Those milestones ensured that the same canonical query meaning can
survive branch, historical, diff, and lineage pressure without host repair.

They did not yet solve a different product-critical problem:

- how reusable query composition can exist without creating alternate,
  host-owned query ASTs
- how named scopes can express domain vocabulary without mutating canonical
  query meaning after the fact
- how templates can parameterize query shape without degrading into stringly
  host interpolation
- how saved queries can exist before durable artifact support lands without
  pretending that ad hoc host serialization is the canonical authority
- how "table", "detail", "inspector", or "kanban" can become structural query
  semantics rather than presentation-only labels
- how view shape can change planning, invalidation, delivery, and patch
  semantics without changing truth semantics

That gap is now load-bearing.

The current crate already exposes strong lower seams:

- authoring is canonicalized through typed detail/collection builders rather
  than string parsing
- result-shape families remain narrowly typed as detail or collection
- collection planning already carries ordering, traversal, aggregation,
  rollup, and derived-field posture
- live promotion already distinguishes detail, ordered collection, and bounded
  materialization families
- region-scoped live narrowing already derives semantic basis from projected
  fields, ordering fields, and traversal relations
- application capability routing already uses sealed witnesses and support
  profiles for milestone-owned features

But the composition and presentation layer is still structurally thin:

- there is no first-class scope subsystem
- there is no first-class template subsystem
- there is no first-class saved-query artifact vocabulary
- there is no first-class view-shape subsystem distinct from raw result-shape
  family
- table/detail semantics are implicit in existing collection/detail paths
  rather than expressed as named view-shape artifacts
- grouped or temporal live semantics do not yet exist as planner-owned query
  contracts

If Milestone 8 does not freeze those contracts now:

- hosts will start inventing their own composition DSLs or helper chains
- scopes will become post-canonicalization mutation hooks instead of canonical
  expansion artifacts
- templates will become interpolation bags that bypass schema-aware authoring
  and validation
- saved queries will drift into controller-owned JSON conventions rather than
  query-owned semantic artifacts
- view labels will be attached after execution, so planning and live delivery
  will never know the consumer's real intent
- future policy, tenant, and durable-artifact milestones will inherit soft
  composition semantics and cosmetic view semantics instead of explicit query
  artifacts

Milestone 8 therefore exists to freeze:

- that scopes and templates are canonical query composition artifacts, not host
  sugar
- that ephemeral saved-query artifacts are query-owned semantic freeze points,
  not durable authorities
- that view shape is distinct from result shape
- that admitted view shape must alter planning and live semantics where the
  product claims it does
- that unsupported view families, non-canonical composition paths, and
  durability claims beyond the current store boundary fail typed and early

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "add reusable builders" or "add UI
  view enums." The hard problem is preventing composition sugar and
  presentation intent from becoming ambient, host-owned meaning that the
  planner and live runtime never actually see.
- `arch_laws.md`: Laws 2, 4, 7, 11, 14, 16, 21, 22, 27, 30, 32, 40, and 41
  dominate this milestone. Scope expansion, template binding, saved-query
  freezing, view-shape lowering, and live patch semantics must be explicit,
  proof-bearing, and planner-owned rather than convention-bound.
- `perf_laws.md`: composition and view surfaces are only honest if scope
  expansion breadth, template binding width, grouped-view invalidation,
  temporal-window breadth, delivery width, and patch-width posture are made
  mechanically visible. Cheap-looking view helpers must not conceal broader
  recomputation or richer delivery lanes.
- `domain_laws.md`: scope descriptors, template descriptors, saved-query
  artifacts, view-shape descriptors, view planning, live patch semantics,
  support profiles, and certification rows are separate responsibilities and
  must not collapse into one broad `views.rs` or `composition.rs` bag.
- `worth_query_vision.md`: named scopes, query templates, saved and named query
  definitions, table/detail/kanban/timeline/inspector view shapes, and
  inspector-style detail are explicit product pillars. Milestone 8 is where
  those become structural query artifacts instead of roadmap prose.
- `worth_query_roadmap.md`: Milestone 8 must prove view-shape-specific live
  semantics for at least table/detail plus one grouped or temporal view. It
  must also establish scopes, templates, and saved-query semantics now while
  keeping durable reload explicitly deferred to Milestone 11.
- `test-requirements.md`: the `Scope / Template / View-Shape Semantic Parity
  Test` is the closeout proof. It requires scope composition and template
  instantiation to preserve canonical meaning and requires shipped view shapes
  to affect planning, invalidation, delivery, and patch semantics.
- `milestone-5.md` and `milestone-5.1.md`: live promotion, ordered collection
  semantics, bounded materialization semantics, and locality-aware delivery are
  already planner-visible. Milestone 8 must build on those contracts rather
  than replacing them with view-layer convenience logic.
- `milestone-6.md`: branch/history/diff basis metadata is already explicit.
  Milestone 8 must let view shapes consume that explicit basis without
  inventing separate branch-comparison or historical presentation APIs.
- `milestone-7.md`: identity-evolution outputs are already typed and replay-
  honest. Milestone 8 must allow inspector/detail and grouped views to consume
  those outputs without flattening lineage-specific meaning.

## Adversarial Constraint

Milestone 8 must survive the following hostile condition:

> The same declared query intent is authored directly, through one or more
> named scopes, through a parameterized template instantiation, and through an
> ephemeral saved-query artifact, then executed as a one-shot read and as a
> live-maintained query across admitted table/detail/inspector/grouped-or-
> temporal views; every admitted lane must preserve one canonical query
> meaning while making composition lineage, view-shape lowering, invalidation,
> delivery, and patch semantics explicit, with no host repair, post-execution
> presentation reinterpretation, or durability overclaim.

Concretely, the design must remain correct when all of the following are true:

- direct construction, scope composition, and template binding all produce the
  same logical query
- view-shape choice changes how results are planned and maintained, not what
  truth means
- the same canonical query may be executed against current, branch,
  historical, or diff basis from Milestone 6
- the same canonical query may participate in identity-evolution inspection
  from Milestone 7
- live execution must emit patch families that match admitted view semantics
- saved-query artifacts may exist in memory or ephemeral host storage before
  durable artifact support is ready
- a naive implementation would be tempted to:
  - treat scopes as helper functions that mutate the canonical query after
    validation
  - treat templates as string interpolation or untyped parameter bags
  - serialize saved queries through ad hoc host JSON without one canonical
    artifact digest
  - treat view shape as UI metadata added after planning and live promotion
  - reuse ordered-collection patch semantics for grouped or inspector views
    without explicit admission and counters
  - let table/detail/kanban/timeline labels exist only in result typing rather
    than execution semantics

If any supported path:

- produces different canonical meaning for direct versus scope/template
  construction
- lets hosts mutate scope or template semantics after canonicalization
- lets saved-query reload or reuse depend on opaque host glue rather than a
  canonical saved-query artifact
- treats view shape as cosmetic metadata with no planning or live effect
- silently widens grouped or temporal invalidation into generic collection
  refresh without explicit admission
- lets inspector/detail view semantics read more than their declared aspect
  focus because the runtime cannot see the view intent
- implies durable saved-query reload, portable saved-query exchange, or
  restart-stable cursor semantics before the store-backed milestones close

then Milestone 8 has failed.

## Product Decision Lock

- `worth-query` owns scope descriptors, template descriptors, ephemeral
  saved-query artifacts, view-shape descriptors, view-shape lowering,
  view-specific delivery/patch semantics, diagnostics, support profiles, and
  certification for admitted Milestone 8 families
- `worth-relational` remains authoritative for truth semantics, branch/history
  basis semantics, and identity semantics consumed by composed or view-shaped
  queries
- `worth-signal` and the runtime bridge remain authoritative for live
  maintenance, locality, and delivery execution beneath the query-owned view
  contracts
- a saved query is not a second query AST; it is a semantic freeze of a
  canonical query artifact plus explicit composition and view metadata
- view shape is not result shape:
  - result shape answers "which fields and structure are delivered"
  - view shape answers "which semantic presentation contract the planner and
    live runtime must honor"
- direct construction, scope expansion, and template instantiation must all
  converge to one canonical query artifact for the same meaning
- table, detail, observed inspector detail, focused inspector detail, and one
  grouped or temporal view are the initial admitted view families for this
  milestone
- inspector semantics must not be collapsed into one fuzzy contract
- `InspectorDetailObserved` is distinct from ordinary detail only if it changes
  invalidation, delivery, or patch posture while preserving ordinary detail
  projection legality
- `InspectorDetailFocused` is distinct only if it additionally constrains
  projection legality, delivery width, and aspect-focus admission
- identity-aware inspector semantics are admitted in this milestone only as an
  explicit view-shape contract carried by inspector families; identity-aware
  inspector is not ambient metadata and not a generic bool flag over all view
  families
- identity-aware focused inspector must preserve Milestone 7 identity
  classification explicitly, including advisory candidates, ambiguity, explicit
  identity break, and denial, rather than collapsing them into one winner or
  one empty patch interpretation
- grouped or temporal view admission must stay explicit; no generic
  "collection but grouped somehow" bag is allowed
- ephemeral saved-query artifacts may ship now, but durable reload,
  import/export portability, and restart-stable continuation remain deferred to
  later store-backed milestones
- Milestone 8 does not close policy masking, tenant schema variation,
  relationship-proof denial, durable saved-query reload, or store-backed
  delivery portability

Normative consequence:

- any implementation path that adds scopes or templates as host-only helper
  sugar with no canonical expansion artifact is out of spec
- any implementation path that stores saved queries as raw builder closures,
  controller-specific structs, or ad hoc strings is out of spec
- any implementation path that exposes one generic `view: String` bag is out
  of spec
- any implementation path that reuses one generic collection patch family for
  grouped or inspector semantics without explicit admission is out of spec
- any implementation path that claims durable saved-query semantics before
  store-backed milestones close is out of spec

## Typed Phase Progression Lock

Milestone 8 must not rely on conceptual sequencing alone. It must define one
proof-bearing phase chain so composition and view semantics cannot be skipped,
reordered, or rewritten after admission.

Required phase progression:

- `RawComposedIntent`
  - direct authoring, named scope declarations, and template declarations may
    exist here
- `ExpandedComposedIntent`
  - all scope expansion and template slot binding are complete here
  - no canonical query artifact exists yet
- `CanonicalComposedArtifact`
  - the existing canonicalization pipeline has frozen canonical query and
    result-shape meaning
  - composition lineage is attached, not re-interpreted
- `SavedQueryFreezeArtifact`
  - optional Milestone 8 freeze point over canonical meaning plus composition
    and view metadata
  - not yet durable by default
- `AdmittedViewShape`
  - view family compatibility, aspect-focus legality, grouping-key legality,
    and saved-artifact legality are already proven
- `ViewShapePlanArtifact`
  - planner-visible delivery, invalidation, and patch posture are already
    fixed
- `LiveViewShapeArtifact`
  - optional live-maintained form whose patch family and refresh posture were
    already fixed by the view-shape plan
  - must also carry the bound live-family mapping from the lowering map below
- `ViewShapeExecutionEnvelope`
  - delivered or replayed result/patch envelope with metadata and counters

Rules:

- no API may admit execution directly from `RawComposedIntent`
- no API may mutate view family after `AdmittedViewShape`
- no API may mutate template slot bindings after `ExpandedComposedIntent`
- no API may mutate canonical query meaning after `CanonicalComposedArtifact`
- saved-query reuse must start from `SavedQueryFreezeArtifact`, not from raw
  builder state
- live lowering must consume `ViewShapePlanArtifact`, never raw collection or
  detail plans directly for Milestone 8-owned view families

Normative consequence:

- if an implementation can construct a view-shaped live query by bolting view
  metadata onto an already-planned ordinary query, the phase chain is broken
- if an implementation can reconstruct scope or template meaning after
  canonicalization, the phase chain is broken
- if an implementation can build saved-query artifacts from authored helper
  state without a canonical artifact, the phase chain is broken

## Compile-Time Enforcement Policy

Milestone 8 must classify which composition and view guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible scope expansions that do not carry canonical query
  digest and scope lineage metadata
- publicly constructible template instantiations that do not carry explicit
  template identity, parameter-binding digest, and resulting canonical query
  digest
- publicly constructible saved-query artifacts that do not carry canonical
  query digest, view-shape digest, and saved-artifact identity
- publicly constructible view-shape artifacts that do not carry one closed
  `ViewShapeFamily`, explicit planning semantics, and explicit live-patch
  family where admitted
- publicly constructible result bundles that erase whether the query was
  admitted as table, detail, observed inspector detail, focused inspector
  detail, or grouped/temporal view
- publicly constructible composition results that collapse direct, scope, and
  template origin into one unclassified bag with no lineage metadata
- publicly constructible phase transitions that let a consumer skip from raw
  composition directly to view planning or live execution

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `QueryScopeDescriptor`, `ExpandedScopeArtifact`,
  `QueryTemplateDescriptor`, `TemplateInstantiationArtifact`,
  `SavedQueryArtifact`, `ViewShapeDescriptor`, `AdmittedViewShape`,
  `ViewShapePlanArtifact`, `ViewShapePatchEnvelope`, or materially equivalent
  proof-bearing types without crate-owned lowering
- public APIs that accept raw closures, raw JSON bags, raw strings, or
  framework-local presentation metadata as scope/template/view-shape authority
- public APIs that let consumers mutate admitted view family after planning or
  live promotion
- public APIs that expose bool-driven shortcuts such as `grouped: bool`,
  `inspector: bool`, `focused_inspector: bool`, `template_mode: bool`, or
  `saved: bool`
- public APIs that allow saved-query artifacts to bypass canonical validation
  and canonicalization
- public APIs that expose one generic `patch_for_view()` entrypoint returning
  untyped dynamic payloads for all view families
- public APIs that allow construction of milestone-owned execution envelopes
  without consuming an admitted view-shape artifact or a view-shape plan

`Construction-time rejection`:

- unsupported scope families
- unsupported template parameter binding forms
- scope expansion that would require hidden widening or illegal predicate
  mutation
- template instantiation whose parameters change result-shape family or query
  family illegally
- view-family requests unsupported by the admitted query family
- focused inspector requests that would require broader projection than the
  declared inspector aspect focus admits
- observed inspector requests that attempt to claim focused-inspector delivery
  budgets or projection legality without the focused-inspector contract
- grouped or temporal view requests whose grouping/temporal keys are not part
  of the validated query vocabulary
- saved-query requests that claim durable persistence, restart stability, or
  store-backed reload before later milestones
- saved-query reuse requests whose schema basis, query basis, or parameter-slot
  equivalence contract cannot be proven explicitly

Rules:

- the strongest available boundary must be used
- composition, saved-artifact, and view-shape proof types must use sealed
  constructors and private fields
- adding a new scope family, template family, saved-artifact persistence
  family, or view-shape family must force exhaustive compile failures across
  authoring, canonicalization, planning, live lowering, support reporting, and
  certification until handled explicitly
- wildcard or catch-all matching over view family or composition family is out
  of spec in milestone-owned code paths
- compile-fail coverage is required for:
  - no external construction of admitted scope expansions
  - no external construction of admitted template instantiations
  - no ad hoc host-local saved-query constructor
  - no direct conversion from raw composition to view plan
  - no direct conversion from canonical query artifact to live view artifact
    without admitted view-shape lowering
  - no bool-driven view-family selection
  - no post-admission view-family mutation
  - no generic dynamic patch accessor across all view families
  - no durable saved-query claim through Milestone 8 artifacts
- runtime rejection is allowed only for facts genuinely unavailable until
  schema validation, basis admission, or live admission resolves the declared
  composition or view contract

## Scope

### In Scope

- named scope descriptors and canonical scope expansion artifacts
- parameterized template descriptors and template instantiation artifacts
- ephemeral saved-query artifacts as semantic freeze points
- admitted view-shape families for:
  - table
  - detail
  - observed inspector detail
  - focused inspector detail
  - one grouped view in this milestone's initial admission matrix
- planner-visible view-shape lowering
- live-visible view-shape invalidation, delivery, and patch semantics for
  admitted view families
- support-profile and capability-matrix integration for composition and view
  semantics
- milestone-native certification for composition parity and view-shape
  semantics

### Explicitly Out Of Scope

- durable saved-query reload, import/export portability, and restart-stable
  artifact continuation
- portable cursor resume or durable delivery continuation
- policy masking, tenant schema variation, and relationship-proof semantics
- store-backed saved-query or store-backed view-delivery portability
- broad library of domain-specific scopes beyond the initial admitted families
- arbitrary user-authored scripting inside templates or scopes
- chart/timeline/kanban parity across every future view family
- UI rendering concerns, component libraries, or server transport framing

## Initial Admission Matrix

Milestone 8 must not leave composition or view semantics ambient.

Initial admitted composition families:

- `NamedScopeExpansion`
- `TemplateInstantiation`
- `EphemeralSavedQueryArtifact`

Initial admitted scope families:

- `PredicateScope`
- `OrderingScope`
- `ProjectionScope`
- `TraversalBoundScope`
- `BasisAwareScope` only where it consumes already-admitted Milestone 6 basis
  artifacts rather than inventing basis semantics

Initial admitted template families:

- `DetailTemplate`
- `CollectionTemplate`
- `ObservedInspectorDetailTemplate`
- `FocusedInspectorDetailTemplate`
- `GroupedCollectionTemplate`

Initial admitted saved-query persistence family:

- `EphemeralProcessOwned`

Initial admitted view-shape families:

- `Table`
- `Detail`
- `InspectorDetailObserved`
- `InspectorDetailFocused`
- `KanbanGrouped`

Deferred view families:

- `TimelineTemporal`
- `ChartAggregate`
- `PortableSavedQuery`
- `DurableSavedQuery`

Required vocabulary artifacts:

- `QueryCompositionFamily`
- `ScopeFamily`
- `TemplateFamily`
- `SavedQueryPersistenceFamily`
- `ViewShapeFamily`
- `ViewShapePatchFamily`
- `ViewShapeAdmissionFailureClass`
- `QueryCompositionAdmissionFailureClass`
- `ViewShapeCostClass`
- `ViewShapeBudgetClass`
- `QueryCompositionCostClass`
- `QueryCompositionBudgetClass`
- `ViewShapePredictionDriftOutcome`
- `QueryCompositionPredictionDriftOutcome`
- `QueryCompositionComplexityContract`
- `ViewShapeComplexityContract`
- `ViewShapePerformanceStatus`
- `ViewShapeRefreshAdmissionClass`
- `ViewShapeFallbackDisposition`

Required composition artifacts:

- `QueryScopeDescriptor`
- `ExpandedScopeArtifact`
- `QueryTemplateDescriptor`
- `TemplateParameterSlot`
- `TemplateInstantiationArtifact`
- `SavedQueryArtifact`
- `SavedQueryReuseDescriptor`
- `SavedQueryMetadata`
- `QueryCompositionSupportProfile`
- `SchemaBoundSavedQueryEquivalenceContract`
- `TemplateBindingEquivalenceContract`

Required view artifacts:

- `ViewShapeDescriptor`
- `AdmittedViewShape`
- `ViewShapePlanArtifact`
- `ViewShapeDeliveryMetadata`
- `ViewShapePatchEnvelope`
- `ViewShapePatchRow`
- `ViewShapePredictionReport`
- `ViewShapeComplexityReport`
- `ViewShapeSupportProfile`
- `GroupedViewResultArtifact`
- `GroupedViewDeltaContract`
- `ViewShapeFallbackReport`

Required metadata content:

- canonical `query_digest`
- canonical `view_shape_digest`
- explicit `composition_digest`
- explicit `scope_lineage_digest` where relevant
- explicit `template_digest` where relevant
- explicit `saved_query_digest` where relevant
- explicit `schema_basis_digest` where relevant
- explicit `basis_digest` where basis-aware composition or view execution
  occurs
- explicit `delivery_digest`
- explicit `patch_digest` where live patch semantics are admitted

Required patch families:

- `TableRowPatch`
- `DetailFieldPatch`
- `InspectorAspectPatch`
- `KanbanGroupMembershipPatch`

Required saved-query equivalence metadata:

- canonical `query_digest`
- canonical `result_shape_digest`
- canonical `view_shape_digest`
- canonical `schema_basis_digest`
- canonical `template_binding_digest` where relevant
- canonical `saved_query_persistence_family`
- explicit `rebind_legality_class`
- explicit `semantic_drift_outcome`

## Structural Assessment Of Current Code

The current crate shape informs Milestone 8 directly.

Already present and reusable:

- `authoring` already defines typed detail and collection query builders
- `canonicalization` already owns the only honest path from authored query to
  canonical artifact
- `collection` already exposes ordering, traversal, aggregation, rollup, and
  derived-field planning artifacts
- `live` already exposes `Detail`, `OrderedCollection`, and
  `BoundedMaterialization` live families with explicit relevance contracts
- `live::region_scoped` already derives locality semantics from live query
  family plus projection/ordering/traversal posture
- `query_context` already proves basis and diff metadata must be explicit,
  sealed, and replay-friendly
- `identity_evolution` already proves milestone-owned support profiles,
  complexity contracts, and result-family classification
- the application facade already routes milestone-owned capability witnesses
  through the support matrix

Not yet structurally present:

- no `composition` subsystem
- no `view_shape` subsystem
- no `saved_query` subsystem
- no support profile or capability family for composition/view semantics
- no first-class distinction between result-shape family and view-shape family
- no grouped or inspector-specific live patch family
- no split between observed inspector semantics and focused inspector semantics

Milestone 8 must therefore extend the architecture without violating the
existing ownership seams:

- composition must sit above authoring/canonicalization, not beside them
- view shape must sit above result shape and collection/live planning, not
  replace them
- ephemeral saved-query artifacts must freeze canonical artifacts, not own
  alternate authored query forms
- capability routing should mirror the existing `query_context` and
  `identity_evolution` pattern rather than inventing a bag-shaped facade

Concrete consequence for implementation:

- Milestone 8 should add dedicated subsystems such as `composition/`,
  `saved_query/`, and `view_shape/`
- Milestone 8 should not add broad helpers into `authoring/`, `collection/`,
  or `live/` and call the milestone complete
- the facade should expose Milestone 8 through explicit capability witnesses
  and support profiles just like `QueryContext` and `IdentityEvolution`

## Capability And Support Lock

Milestone 8 must not leave capability ownership or support reporting vague.

Required application-surface additions:

- `WORTHQueryCapabilityFamily::QueryComposition`
- `WORTHQueryCapabilityFamily::ViewShape`
- `WORTHQueryCapabilityFamily::SavedQueryArtifacts`

Required ownership:

- `QueryComposition` is owned by the query subsystem and config-gated by the
  query section
- `ViewShape` is owned by the query subsystem for planning semantics and may
  depend on admitted live support where live lowering is requested
- `SavedQueryArtifacts` is owned by the query subsystem for Milestone 8
  ephemeral freeze semantics, with durable sub-capabilities remaining deferred
  to the store owner in later milestones

Required support-profile additions:

- `QueryCompositionSupportProfile`
  - admitted scope families
  - admitted template families
  - deferred composition markers
  - complexity status per admitted composition family
- `ViewShapeSupportProfile`
  - admitted view families
  - admitted live patch families
  - deferred view families
  - complexity status per admitted view family
  - fallback disposition per admitted view family
- `SavedQueryArtifactSupportProfile`
  - admitted persistence families
  - deferred durability families
  - admitted equivalence contracts

Required facade witnesses:

- either one dedicated `QueryCompositionCapability`,
  `ViewShapeCapability`, and `SavedQueryArtifactCapability`
- or one `CompositionAndViewCapability` plus one
  `SavedQueryArtifactCapability`

The spec prefers the first option unless implementation review proves the
combined witness is still cost-honest and failure-topology-honest.

## Performance Encoding Lock

Milestone 8 must encode performance as first-class semantics, not as
after-the-fact instrumentation.

Required named performance surfaces:

- one `QueryCompositionComplexityContract` family
- one `ViewShapeComplexityContract` family
- one `ComplexityStatus::{Verified,Debt}` surface for Milestone 8-owned
  composition and view families
- one `ViewShapePerformanceStatus`
- one `ViewShapeRefreshAdmissionClass`
- one `ViewShapeFallbackDisposition`

Required admitted complexity contracts:

- `named_scope_expansion`
  - declared Big-O: `O(scope_entries + emitted_query_entries)`
  - forbidden broadening clause: no post-expansion reinterpretation during
    canonicalization or planning
- `template_instantiation`
  - declared Big-O: `O(template_slots + bound_parameters + emitted_query_entries)`
  - forbidden broadening clause: no host-local interpolation or secondary slot
    resolution during planning
- `saved_query_freeze`
  - declared Big-O: `O(canonical_artifact_width + metadata_width)`
  - forbidden broadening clause: no hidden schema/basis discovery during reuse
- `table_view_lowering`
  - declared Big-O: `O(ordering_keys + projection_width)`
  - forbidden broadening clause: no grouped or temporal maintenance hidden
    behind table admission
- `observed_inspector_lowering`
  - declared Big-O: `O(detail_projection_width + observation_keys)`
  - forbidden broadening clause: no focused-inspector projection narrowing
    silently inferred later
- `focused_inspector_lowering`
  - declared Big-O: `O(aspect_focus_width + focused_projection_width)`
  - forbidden broadening clause: no out-of-focus projection rescue or broad
    detail fallback after admission
- `kanban_grouped_lowering`
  - declared Big-O: `O(grouping_keys + grouped_desired_state_width + delta_width)`
  - forbidden broadening clause: no hidden full regroup or generic collection
    refresh without explicit fallback admission

Required view-shape cost classes:

- `TableNarrowOrdered`
- `DetailNarrowObserved`
- `ObservedInspectorNarrow`
- `FocusedInspectorAspectBound`
- `KanbanGroupedDeltaBound`
- `KanbanGroupedRefreshDeferredDebt`

Required fallback dispositions:

- `RefreshForbidden`
- `RefreshAdmittedWithinBudget`
- `RefreshDeniedBudgetExceeded`
- `GroupedFullRegroupDeferredDebt`

Rules:

- every admitted Milestone 8 lane must declare exactly one complexity contract
- every admitted Milestone 8 lane must declare exactly one cost class
- every admitted Milestone 8 lane must declare exactly one complexity status
  of `Verified` or `Debt`
- every admitted Milestone 8 lane must declare exactly one fallback
  disposition, even if that disposition is `RefreshForbidden`
- no shipped view family may rely on an implicit "just refresh" escape hatch
- any view family whose honest behavior still requires generic refresh must be
  marked explicit debt rather than implied completeness
- grouped-view refresh and regrouping policy must be encoded before live
  execution begins, not discovered reactively inside the patch engine
- focused-inspector widening must deny before rich artifact construction, not
  degrade to ordinary detail silently
- performance status must distinguish `Verified` from `Debt` at the view-family
  level, not only at the milestone level

Required support-matrix consequence:

- the support matrix must not mark a Milestone 8 family simply as "admitted"
  without also exposing whether its complexity posture is `Verified` or `Debt`
- `KanbanGrouped` specifically may be admitted with `ComplexityStatus::Debt`
  only if its fallback disposition is explicit and its deferred full-regroup
  behavior is surfaced as debt rather than hidden support

Normative consequence:

- if an implementation cannot answer whether `KanbanGrouped` is delta-bound or
  refresh-backed from the admitted plan artifact alone, the performance
  contract is incomplete
- if an implementation can silently fall back from focused inspector to
  ordinary detail under pressure, the performance and correctness contracts are
  both broken

### Grouped Delta Admission Boundary

Grouped-view support is only honest if the plan can state when incremental
maintenance remains semantically bounded and when it must deny, defer, or
surface debt.

Required surfaces:

- one `GroupedDeltaAdmissionBoundary`
- one `GroupedDeltaPredictionReport`
- one `GroupedDeltaRealizationReport`
- one `GroupedRefreshDecision`

Required decision inputs:

- predicted group membership transition count
- predicted grouped desired-state row width
- predicted grouped delta row width
- predicted lane-count churn
- predicted projection width per lane
- realized versions of the same measures after maintenance

Normative boundary rules:

- `KanbanGroupedDeltaBound`
  - may be admitted only when the grouped plan artifact names explicit maximums
    for membership transitions, lane churn, and grouped delta width
  - realized grouped maintenance must remain within the admitted bounds
- `KanbanGroupedRefreshDeferredDebt`
  - may be admitted only when the support profile marks the lane as
    `ComplexityStatus::Debt` and the fallback disposition is explicit
  - refresh or regroup behavior must be surfaced before execution, not
    discovered mid-patch
- if realized membership transitions exceed the admitted bound, the lane must
  emit an explicit denial or debt-classified refresh decision rather than
  silently widening into generic ordered-collection refresh
- if realized lane-count churn exceeds the admitted bound, the lane must emit
  `GroupedRefreshDecision::RefreshDeniedBudgetExceeded` or the explicit debt
  fallback already declared in the admitted plan
- if predicted and realized grouped delta width diverge beyond the admitted
  budget, the lane must produce a denial/debt artifact and increment the
  corresponding fallback counters
- no grouped lane may claim `DeltaBound` when full regroup remains the normal
  maintenance strategy

Implementation instructions:

- `ViewShapePlanArtifact` for `KanbanGrouped` must embed one
  `GroupedDeltaPredictionReport` and one `GroupedDeltaAdmissionBoundary`
- `ViewShapePatchEnvelope` for grouped lanes must embed one
  `GroupedDeltaRealizationReport`
- certification must compare prediction versus realization and assert that
  admitted bounds were either respected or explicitly denied/deferred
- grouped patch generation must derive from prior/next grouped desired-state
  artifacts first; refresh policy is a separate explicit decision, not a hidden
  branch inside patch computation

## Phase Plan

### Phase 1: Composition Vocabulary And Canonical Expansion

Phase 1 exists to make query composition first-class before view semantics
build on it.

Milestone 8 must first ship:

- a dedicated `composition` subdomain
- scope descriptors with one closed scope-family vocabulary
- template descriptors with explicit parameter slots
- expansion and instantiation artifacts that lower into the existing authored
  query and canonicalization pipeline
- composition digests and scope/template lineage metadata
- one explicit typed bridge from `ExpandedComposedIntent` into the existing
  authored-query/canonicalization pipeline
- typed denials for unsupported scope/template families and illegal widening

This phase leaves the system in a coherent state where:

- direct construction, scope expansion, and template instantiation all target
  one canonical query artifact
- no host needs to invent its own composition AST
- later view semantics can consume one already-expanded, already-canonical
  query meaning

Phase exit criterion:

- scope/template expansion can prove canonical parity with direct construction
  for admitted families

### Phase 2: Saved Query Artifact Freeze

Phase 2 exists to create semantic freeze points without overclaiming durable
 persistence.

Milestone 8 must next ship:

- one `SavedQueryArtifact` family for ephemeral process-owned saved queries
- explicit saved-query metadata carrying canonical query digest, composition
  digest, view-shape digest, and persistence-family identity
- saved-query reuse descriptors that keep later parameter rebinding explicit
- explicit saved-query equivalence contracts over schema basis, template
  bindings, and basis admissibility
- typed denials for durable reload claims, import/export claims, or
  restart-stable continuation claims

This phase leaves the system in a coherent state where:

- saved-query semantics exist as query-owned artifacts
- later Milestone 11 durability work can extend the same artifact model
  instead of replacing host-local conventions
- ephemeral artifact mode remains explicit debt, not implied product
  completeness

Phase exit criterion:

- saved-query freeze artifacts preserve canonical meaning and explicitly deny
  durability overclaims

### Saved Query Rebinding Legality Matrix

Saved-query reuse must not depend on vague "looks close enough" host logic.
Milestone 8 must name exactly which rebind operations preserve the saved
artifact's semantic identity and which ones constitute semantic drift.

Required surfaces:

- one `SavedQueryRebindingDimension` vocabulary
- one `SavedQueryRebindingLegality` vocabulary with:
  - `LegalNoSemanticChange`
  - `LegalRequiresFreshFreeze`
  - `IllegalSemanticDrift`
- one `SavedQueryBindingMatrixArtifact`
- one `SavedQueryReuseDecision`

Required rebinding dimensions:

- schema basis digest change
- basis family change
- template slot value change
- template slot set change
- view-family change
- result-shape family change
- composition lineage change
- support-profile / capability-family change

Normative legality rules:

- schema basis digest change
  - `LegalNoSemanticChange` only when the saved artifact carries an explicit
    schema-basis equivalence contract proving the same admitted basis family
    and the same projection legality surface
  - otherwise `IllegalSemanticDrift`
- basis family change
  - always `IllegalSemanticDrift` in Milestone 8
  - branch/current/historical/diff rebinding belongs to fresh construction or
    fresh freeze, not silent saved-query reuse
- template slot value change
  - `LegalRequiresFreshFreeze` only when the slot remains within the same
    admitted slot family and the canonical query meaning is expected to change
    explicitly
  - the prior saved artifact may not be re-labeled as unchanged
- template slot set change
  - always `IllegalSemanticDrift`
- view-family change
  - always `LegalRequiresFreshFreeze`
  - one saved artifact may not silently rebind from `Table` to
    `InspectorDetailFocused` or from `Detail` to `KanbanGrouped`
- result-shape family change
  - always `IllegalSemanticDrift`
- composition lineage change
  - `LegalNoSemanticChange` only if canonical query digest, composition digest,
    and scope/template lineage digest all prove equivalence
  - otherwise `IllegalSemanticDrift`
- support-profile / capability-family change
  - always `IllegalSemanticDrift` for reuse claims
  - support posture may not be upgraded or narrowed by host glue

Implementation instructions:

- `SavedQueryArtifact` must carry one `SavedQueryBindingMatrixArtifact`
  capturing every admitted rebinding dimension above
- `SavedQueryReuseDecision` must be produced before planning or live lowering
- `LegalRequiresFreshFreeze` must terminate the old artifact's reuse path and
  require a new saved-query freeze artifact with a new digest
- no planner, executor, or host cache may reinterpret an
  `IllegalSemanticDrift` row as "best effort reuse"
- compile-fail and typed-denial coverage must include at least one row for each
  of:
  - basis-family change
  - template slot set change
  - view-family change without fresh freeze
  - support-profile change disguised as the same artifact

### Phase 3: View-Shape Vocabulary And Planning Semantics

Phase 3 exists to separate view intent from raw result shape.

Milestone 8 must next ship:

- one dedicated `view_shape` subdomain
- one `ViewShapeFamily` vocabulary with admitted `Table`, `Detail`,
  `InspectorDetailObserved`, `InspectorDetailFocused`, and `KanbanGrouped`
- view-shape descriptors that bind to already-validated canonical query
  artifacts
- one explicit compatibility matrix from canonical query family/result-shape
  family into admitted view families
- planner-visible view-shape lowering that produces:
  - view-shape digest
  - delivery metadata
  - invalidation-key posture
  - patch-family posture
  - complexity reports
- typed denials when query family and view family are incompatible

This phase leaves the system in a coherent state where:

- result shape and view shape are structurally distinct
- the planner can tell the difference between an ordered collection shown as a
  table versus grouped into kanban lanes
- inspector detail has a structural contract instead of being "detail but UI
  code only looks at some fields"

Phase exit criterion:

- admitted view shapes materially alter planning artifacts and digest surfaces

### Phase 4: Live Maintenance And Patch Semantics

Phase 4 exists because view shape is dishonest if live semantics do not change.

Milestone 8 must next ship:

- live lowering from admitted view-shape plan artifacts into view-specific
  delivery contracts
- explicit live patch families:
  - table row patch
  - detail field patch
  - observed inspector patch
  - focused inspector aspect patch
  - kanban group-membership patch
- grouped desired-state artifacts and grouped delta contracts, so grouped
  patches are derived from one canonical grouped result rather than ad hoc
  incremental regrouping
- explicit grouped invalidation posture keyed by declared grouping aspect
- explicit observed-inspector invalidation posture
- explicit focused-inspector invalidation posture keyed by declared aspect
  focus
- explicit fallback dispositions and refresh-admission policy encoded in the
  admitted view-shape plan before live execution begins
- exact counters for group membership moves, group count changes, patch width,
  and fallback/refresh denials

This phase leaves the system in a coherent state where:

- shipped view shapes affect live invalidation, delivery, and patch semantics
- grouped view support is more than "table plus front-end regrouping"
- inspector detail support is more than "detail with a nicer type alias"

Phase exit criterion:

- live-maintained admitted view families emit distinct planner-owned and
  patch-owned artifacts rather than generic collection patches

### Phase 5: Facade, Support Profiles, And Certification

Phase 5 exists to make Milestone 8 part of the daily-driver surface.

Milestone 8 must finally ship:

- one application-facing composition/view capability witness or one pair of
  witnesses, provided they remain authority-preserving and not bag-shaped
- support-matrix rows for `QueryComposition`, `ViewShape`, and
  `SavedQueryArtifacts`
- support-matrix and support-report integration for admitted composition and
  view families
- explicit deferred-scope markers for durable saved-query work and unsupported
  view families
- the `Scope / Template / View-Shape Semantic Parity Test`
- representative canonical and rejection row catalogs
- compile-fail coverage for milestone-owned proof boundaries

This phase leaves the system in a coherent state where:

- composition and view semantics are part of the supported product surface
- the support report can honestly describe what is admitted, deferred, and
  unsupported
- later policy, tenant, and durability milestones can compose with explicit
  Milestone 8 artifacts

Phase exit criterion:

- certification proves composition parity and semantic view shaping through
  canonical machine-checkable artifacts

## Must Ship

- proof-bearing `QueryScopeDescriptor`, `ExpandedScopeArtifact`,
  `QueryTemplateDescriptor`, `TemplateInstantiationArtifact`,
  `SavedQueryArtifact`, `SavedQueryMetadata`, `ViewShapeDescriptor`,
  `AdmittedViewShape`, `ViewShapePlanArtifact`, `ViewShapeDeliveryMetadata`,
  and `ViewShapePatchEnvelope` families or materially equivalent types
- explicit admitted scope families, template families, and view-shape
  families
- one typed phase chain from raw composition through admitted view-shape
  lowering and, where applicable, live view-shape execution
- canonical expansion from direct/scope/template construction into one
  canonical query artifact
- ephemeral saved-query artifacts that freeze canonical meaning without
  implying durable support
- planner-visible view-shape lowering for table, detail, observed inspector
  detail, focused inspector detail, and kanban grouped view
- live-visible view-shape patch semantics for shipped view families
- identity-aware inspector delivery metadata and saved-query binding for the
  admitted inspector families
- dedicated performance/counter subdomains for composition and view-shape
  semantics rather than generic telemetry-only logging
- typed diagnostics, replay bundles, and exact counters for composition
  expansion, saved-query freeze, view planning, and live patch shaping
- support-profile and capability-report integration for admitted Milestone 8
  families
- milestone-native certification proving composition parity and non-cosmetic
  view semantics
- explicit saved-query equivalence contracts proving which rebinding actions are
  legal versus semantic drift
- explicit grouped desired-state artifacts proving grouped patch semantics are
  derived, not heuristic
- explicit per-view-family complexity contracts, cost classes, performance
  status, and fallback dispositions

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- schema-aware validation from Milestone 2 remains authoritative
- plan ownership from Milestone 3 remains authoritative
- collection ordering, traversal, aggregation, rollup, and derived-field
  semantics from Milestone 4 remain authoritative
- live promotion and region-scoped live semantics from Milestones 5 and 5.1
  remain authoritative beneath view-shape lowering
- basis and diff ownership from Milestone 6 remain authoritative where view
  shapes execute against branch or historical contexts
- identity-evolution ownership from Milestone 7 remains authoritative where
  inspector or grouped views consume lineage/correspondence results
- identity-aware inspector contract changes are semantic changes for
  saved-query freeze and reuse, not cosmetic view metadata drift
- `worth-relational`, `worth-signal`, and the runtime bridge remain the
  authorities beneath query-owned composition and view contracts
- saved-query artifacts remain semantic freezes of canonical query meaning, not
  alternate query authorities
- view shape remains distinct from result shape
- unsupported view families and durable artifact claims fail typed and early

## Complexity / Proof Obligations

Milestone 8 must name costs and proofs in terms of:

- scope expansion count
- scope expansion width
- template slot count
- template binding width
- template defaulting count
- saved-query freeze width
- saved-query reuse count
- view-shape planning count
- view grouping key count
- view temporal window width where admitted later
- observed inspector delivery width
- focused inspector aspect-focus width
- focused inspector projection width
- table ordering key count
- grouped desired-state row count
- grouped delta row count
- grouped membership transition count
- grouped lane count
- view patch width
- view delivery width
- refresh fallback count
- view-shape executor rediscovery count
- scope rediscovery count
- template rediscovery count
- view-family fallback denial count
- view-family refresh admission count
- view-family refresh forbidden count
- grouped full-regroup denial count
- focused inspector widening denial count

Minimum required counters:

- `scope_expansion_count`
- `scope_expansion_width`
- `template_slot_count`
- `template_binding_width`
- `template_defaulting_count`
- `saved_query_freeze_width`
- `saved_query_reuse_count`
- `view_shape_planning_count`
- `view_grouping_key_count`
- `observed_inspector_delivery_width`
- `focused_inspector_aspect_focus_width`
- `focused_inspector_projection_width`
- `table_ordering_key_count`
- `grouped_desired_state_row_count`
- `grouped_delta_row_count`
- `grouped_membership_transition_count`
- `grouped_lane_count`
- `view_patch_width`
- `view_delivery_width`
- `view_refresh_fallback_count`
- `view_shape_executor_rediscovery_count`
- `scope_rediscovery_count`
- `template_rediscovery_count`
- `view_family_fallback_denial_count`
- `view_family_refresh_admission_count`
- `view_family_refresh_forbidden_count`
- `grouped_full_regroup_denial_count`
- `focused_inspector_widening_denial_count`
- `durable_saved_query_denial_count`
- `cosmetic_view_semantics_denial_count`
- `complexity_status_debt_count`

Rules:

- counters belong to admitted composition artifacts, admitted view artifacts,
  denial bundles, and certification bundles
- representative certification scenarios must assert exact counts
- `view_shape_executor_rediscovery_count` must be exactly zero on every
  admitted lane
- `scope_rediscovery_count` must be exactly zero on every admitted scope lane
- `template_rediscovery_count` must be exactly zero on every admitted template
  lane
- every admitted view lane must emit exactly one view-shape complexity report
- every denied durable saved-query request must increment
  `durable_saved_query_denial_count`
- every denied attempt to treat view shape as cosmetic-only must increment
  `cosmetic_view_semantics_denial_count`
- every denied unsupported view fallback must increment
  `view_family_fallback_denial_count`
- every lane that explicitly admits refresh must increment
  `view_family_refresh_admission_count`
- every lane whose contract forbids refresh must increment
  `view_family_refresh_forbidden_count` on denied refresh attempts
- every denied grouped full-regroup attempt must increment
  `grouped_full_regroup_denial_count`
- every denied focused-inspector widening attempt must increment
  `focused_inspector_widening_denial_count`
- grouped-view lanes must make group membership moves and lane counts
  mechanically visible
- grouped-view lanes must also make desired-state row count and delta row count
  visible so grouped patches can be checked against canonical grouped truth
- observed-inspector lanes must make delivery width mechanically visible
- focused-inspector lanes must make both aspect-focus width and projection
  width mechanically visible
- table lanes must make ordering-key posture mechanically visible
- no admitted lane may hide grouped recomputation, regrouping, or inspector
  widening inside generic collection success counters
- elapsed time alone is not acceptable evidence for any Milestone 8
  performance claim; proof must be expressed in structural work counters

Minimum certification rows should include:

- `scope-expansion-canonical-parity`
- `template-instantiation-canonical-parity`
- `saved-query-freeze-explicitness`
- `saved-query-equivalence-contract-explicitness`
- `saved-query-rebinding-legality-matrix`
- `table-view-planning-explicitness`
- `detail-view-patch-explicitness`
- `observed-inspector-delivery-explicitness`
- `focused-inspector-aspect-focus-explicitness`
- `kanban-group-membership-explicitness`
- `kanban-grouped-desired-state-parity`
- `kanban-delta-admission-boundary`
- `view-shape-live-digest-parity`
- `composition-versus-direct-result-parity`
- `support-profile-honesty`
- `cross-feature-precommit-honesty`

Minimum rejection rows should include:

- `unsupported-scope-family`
- `unsupported-template-family`
- `durable-saved-query-deferred-debt`
- `saved-query-schema-basis-drift-forbidden`
- `saved-query-view-family-rebind-without-fresh-freeze-forbidden`
- `view-family-query-family-mismatch`
- `cosmetic-view-semantics-forbidden`
- `post-admission-view-mutation-forbidden`
- `focused-inspector-projection-broadening-forbidden`
- `observed-inspector-cannot-claim-focused-budget`
- `grouped-view-hidden-refresh-forbidden`
- `grouped-delta-boundary-overrun-forbidden`

## Allowed Debt

- durable saved-query reload, import/export portability, and restart-stable
  continuation may remain explicit `Debt`
- additional scope families and additional template families may remain
  explicit `Debt`
- timeline and chart view families may remain explicit `Debt`
- grouped/temporal families beyond the initial admitted family may remain
  explicit `Debt`
- additional inspector families beyond observed/focused may remain explicit
  `Debt`
- direct-vs-scope-vs-template canonical drift may not exist as debt
- cosmetic-only shipped view families may not exist as debt
- hidden durable saved-query overclaims may not exist as debt
- hidden regrouping refreshes or inspector projection widening may not exist as
  debt

## Acceptance Evidence

Milestone 8 is complete only when `worth-query` can prove:

- the `Scope / Template / View-Shape Semantic Parity Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- scopes and templates normalize to the same canonical query meaning as direct
  construction for admitted families
- saved-query artifacts freeze canonical meaning and explicitly deny durable
  claims beyond this milestone
- shipped view shapes affect planning, invalidation, delivery, and patch
  semantics
- grouped or inspector semantics do not exist only as cosmetic typing
- identity-aware inspector semantics are planner-visible, delivery-visible,
  saved-query-visible, and support-report-visible for the admitted inspector
  families
- unsupported composition families, unsupported view families, and durable
  overclaims fail typed and early

Required verification output must include:

- `query_digest`
- `composition_digest`
- `schema_basis_digest`
- `view_shape_digest`
- `plan_digest`
- `result_shape_digest`
- `delivery_digest`
- `patch_digest` where relevant
- `result_digest`
- `failure_digest`
- `counter_snapshot`
- `artifact_binding_matrix`
- `complexity_status`
- `fallback_disposition`

## Representative Scenario Matrix

Milestone 8 must prove the architecture against concrete lanes, not just
capability names.

Minimum representative scenarios:

- `direct-versus-scope-parity`
  - one query built directly and through one named scope expands to the same
    canonical query digest
  - scope lineage remains explicit even though canonical query meaning is the
    same
- `direct-versus-template-parity`
  - one direct query and one template-instantiated query produce the same
    canonical query and result-shape meaning
  - parameter binding digest remains explicit
- `saved-query-freeze-without-durability-overclaim`
  - one canonical query is frozen into one ephemeral saved-query artifact
  - reload inside the same process preserves semantic identity
  - durable persistence claim is denied typed and early
- `saved-query-schema-basis-equivalence`
  - one saved query rebind attempt across a schema-basis change is classified as
    either explicitly legal or semantic drift
  - no silent reuse is allowed
- `saved-query-rebinding-legality-matrix`
  - one matrix lane covers basis-family change, template slot value change,
    template slot set change, and view-family change
  - each dimension emits `LegalNoSemanticChange`,
    `LegalRequiresFreshFreeze`, or `IllegalSemanticDrift`
  - fresh-freeze-required rows prove a new artifact digest is minted
- `table-view-ordering-semantic-lowering`
  - one ordered collection query admitted as table view emits table-specific
    planning and patch metadata
  - ordering posture remains explicit in the view-shape artifact
- `detail-view-field-patch`
  - one detail query admitted as detail view emits field-patch semantics
  - delivery and patch digests are distinct from table semantics
- `observed-inspector-live`
  - one detail query admitted as observed inspector detail proves narrower
    invalidation or delivery semantics while preserving ordinary detail
    projection legality
- `focused-inspector-live`
  - one detail query admitted as focused inspector detail proves narrower
    aspect-focus metadata, narrower projection legality, and aspect-focused
    patch semantics
  - broad focused-inspector projection widening is denied
- `kanban-group-membership-patch`
  - one collection query admitted as grouped kanban view emits group-membership
  patch semantics rather than generic ordered-collection patches
  - grouping aspect is explicit in planning and delivery metadata
- `kanban-desired-state-to-delta-parity`
  - one grouped lane proves that the emitted grouped patch matches the delta
    between prior grouped desired state and next grouped desired state
  - no host regrouping heuristic is needed to explain the patch
- `kanban-delta-admission-boundary`
  - one grouped lane stays within admitted transition and lane-churn budgets
  - one hostile grouped lane exceeds the budget and emits explicit denial or
    debt-classified refresh disposition
  - no lane silently widens into generic ordered-collection refresh
- `branch-aware-view-shape-parity`
  - one admitted table or inspector view executes across Milestone 6 basis
    contexts without changing declared view family
  - basis metadata changes, view family does not
- `identity-aware-focused-inspector-parity`
  - one focused inspector detail lane consumes Milestone 7 identity-evolution
    output without flattening identity classification
- `identity-break-inspector-explicitness`
  - one identity-aware focused inspector lane preserves explicit identity break
    as explicit identity classification instead of degrading it into generic
    denial, empty patch, or missing-data semantics
- `post-admission-view-mutation-forbidden`
  - one hostile lane attempts to mutate admitted view family after planning and
    fails compile-time or typed admission
- `grouped-hidden-refresh-forbidden`
  - one hostile grouped-view lane tries to fall back to hidden full refresh
    without explicit admission and is denied
- `durable-saved-query-deferred`
  - one hostile lane requests durable saved-query reload and receives explicit
    deferred-debt denial

## Milestone 8 Cross-Feature Precommit Matrix

Milestone 8 should pre-name the interaction contracts that later milestones
must preserve so test posture does not become "we will figure it out later."

Required cross-feature rows:

- `scope-template-saved-query-freeze`
  - direct, scope-composed, and template-instantiated construction may all feed
    a saved-query freeze
  - parity requires equal canonical query meaning and explicit non-equal
    composition lineage where applicable
- `scope-template-view-shape-lowering`
  - composition path may not affect admitted view-family compatibility,
    lowering, or fallback posture
- `saved-query-view-family-reload`
  - saved-query reuse may preserve a view family only under
    `LegalNoSemanticChange`
  - changing the view family requires fresh freeze and new digest issuance
- `basis-aware-view-shape-parity`
  - Milestone 6 basis changes may alter basis digests and legal execution
    context without mutating the admitted view-family contract
- `identity-aware-inspector-parity`
  - Milestone 7 identity-evolution outputs may feed observed or focused
    inspector detail without flattening identity classification
- `future-policy-mask-composition`
  - Milestone 9 policy masking must apply after canonical composition and may
    only remove or deny, never reinterpret scope/template meaning
- `future-tenant-schema-saved-query`
  - Milestone 9 tenant/schema variation must route through saved-query
    equivalence and rebinding law rather than host-local "same query" guesses
- `future-durable-artifact-extension`
  - Milestone 11 durability may extend `SavedQueryArtifact` but may not replace
    it with a second portable artifact model

Implementation instructions:

- every row above must appear in the support-report narrative or certification
  appendix even if some later-milestone side is still marked deferred
- every row above must identify which digest is expected to remain equal and
  which digest is expected to change
- later milestone specs should be required to cite these rows instead of
  redefining the Milestone 8 surface informally

Adversarial intent:

- this matrix prevents Milestone 8 from shipping in a way that looks correct in
  isolation but collapses once policy, tenant, durability, or richer identity
  scenarios arrive

## Architectural Notes

### Composition Must Lower Before Canonicalization Freezes

The easiest way to fake scopes and templates is to keep canonicalization
unchanged and let hosts expand helpers around it. That is out of spec.

The required rule is:

- scopes and templates may author query intent
- canonicalization still owns the final canonical query artifact
- composition must lower into the existing canonicalization path before the
  canonical artifact is frozen
- hosts may not rewrite a canonical query after canonicalization to simulate
  scope or template semantics

### Saved Query Is A Freeze Point, Not A Second AST

Milestone 8 must not introduce a parallel query language for saved artifacts.

The required rule is:

- save the canonical artifact and explicit composition/view metadata
- save explicit schema-basis and template-binding equivalence contracts
- do not save raw closures, controller-owned structs, or host-interpreted
  strings as the semantic authority
- do not imply durable portability before later milestones

### View Shape Is Not Result Shape

Current code already has result-shape families and collection planning
artifacts. That is not enough.

The required rule is:

- result shape specifies delivered structure
- view shape specifies presentation-semantic execution posture
- planning and live maintenance must be able to distinguish them
- one result shape may legally support multiple admitted view families if the
  view semantics are explicit and planner-visible

### Table And Detail Must Become Explicit, Not Assumed

Current live code already distinguishes detail and ordered collection. Milestone
8 must formalize that into view-shape artifacts rather than treating those
families as implicit side effects of existing plan structures.

That means:

- table view should explicitly lower through ordered collection semantics
- detail view should explicitly lower through detail semantics
- observed inspector detail should explicitly lower through ordinary detail
  projection legality plus narrower observation semantics
- focused inspector detail should explicitly narrow aspect-focus posture and
  projection legality
- kanban grouped view should explicitly introduce grouped membership semantics

Required lowering map:

- `Table`
  - lowers onto the existing ordered-collection planning and live semantics
  - adds explicit table-view delivery and patch-family metadata
  - binds to existing `LiveQueryFamily::OrderedCollection`; Milestone 8 must
    not create a second table-specific live engine
- `Detail`
  - lowers onto the existing detail planning and live semantics
  - adds explicit detail-view delivery and patch-family metadata
  - binds to existing `LiveQueryFamily::Detail`; Milestone 8 must not create a
    second detail-specific live engine
- `InspectorDetailObserved`
  - lowers onto detail semantics plus explicit observation posture, narrower
    invalidation or delivery semantics, and observed-inspector patch family
  - preserves ordinary detail projection legality
  - binds to existing `LiveQueryFamily::Detail` with Milestone 8-owned
    observation metadata layered through the view-shape plan
- `InspectorDetailFocused`
  - lowers onto detail semantics plus explicit inspector aspect-focus
    constraints, narrower invalidation keys, narrower delivery budgets, and
    focused-inspector patch family
  - does not preserve ordinary detail projection legality automatically
  - binds to existing `LiveQueryFamily::Detail` with additional focused
    projection legality and width budgets; it may not silently degrade to
    ordinary detail on overflow
- `KanbanGrouped`
  - may reuse collection planning inputs where honest
  - must introduce one distinct grouped result artifact, grouping contract, and
    grouped patch family
  - may bind to existing `LiveQueryFamily::OrderedCollection` only if the
    grouped desired-state artifact and grouped delta contract remain the
    authoritative Milestone 8 layer above that family
  - may not mint a second generic collection patch engine that duplicates the
    ordered-collection live path
  - may not be encoded as ordinary ordered-collection semantics plus host-side
    regrouping

### Grouped View Must Not Be Cosmetic Regrouping

If a grouped view is shipped in Milestone 8, it must change real semantics:

- grouping key must be explicit
- grouped desired state must be an explicit artifact
- membership movement between groups must be patch-visible
- group counts or group identities must be delivery-visible where admitted
- grouped patches must be derived as deltas from grouped desired state
- regrouping cannot happen only in host/UI code after generic collection
  delivery

### This Milestone Must Not Steal 9 Or 11

Milestone 8 owns:

- scopes
- templates
- ephemeral saved-query artifacts
- admitted view-shape semantics

It does not own:

- policy masking or tenant schema variation
- durable saved-query reload or import/export portability
- durable cursor resume
- store-backed delivery portability

Milestone 8 must therefore stop at:

- composition artifacts
- ephemeral saved-query freeze
- view-shape lowering
- live patch semantics
- support reporting
- certification

## Sequencing Notes

Milestone 8 belongs immediately after Milestone 7 because the query layer now
has explicit basis and identity semantics and can safely expose reusable
composition and presentation intent on top of those lower contracts.

It must land before Milestone 9 because policy masking and tenant schema
variation should apply to already-canonical scopes/templates/saved artifacts
and already-explicit view families rather than to host-defined helper paths.

It must land before Milestone 11 because durable saved-query support should
extend one frozen saved-artifact model, not introduce a different one after the
fact.

## Parallelization Notes

Once composition vocabulary and view-family vocabulary are frozen:

- support-profile and capability-report work can proceed in parallel
- grouped-view live patch work can proceed in parallel with saved-query freeze
  work
- compile-fail hardening can proceed in parallel with certification row
  construction
- later Milestone 9 policy and tenant work can begin composing with frozen
  scope/template/view artifacts without redefining them

## Store Dependency

- core scope expansion, template instantiation, ephemeral saved-query freeze,
  and runtime-backed view-shape semantics are not blocked on `worth-store`
- durable saved-query reload, import/export portability, restart-stable saved
  artifacts, and durable delivery continuation remain blocked on later
  store-backed milestones and must stay explicit debt until that support exists

## Explicit Failure Taxonomy For Milestone 8

- unsupported scope family
- unsupported template family
- template binding mismatch
- illegal scope widening
- durable saved-query overclaim
- view-family/query-family mismatch
- cosmetic view semantics
- post-admission view mutation
- inspector projection broadening
- grouped hidden refresh
- saved-query semantic drift
- view-shape replay divergence
- view-shape artifact invariant break

## Anti-Patterns Explicitly Rejected

- host-only scope helper chains with no canonical expansion artifact
- string interpolation or untyped map substitution for templates
- saved-query semantics owned by controllers, handlers, or front-end caches
- one generic `view: String` bag
- one generic patch payload for every view family
- grouped views implemented as generic collection delivery plus front-end
  regrouping only
- inspector view implemented as ordinary detail with host-side field pruning
- durable artifact claims through Milestone 8-only surfaces
- one mega-module mixing composition, saving, view lowering, live patching,
  diagnostics, and certification

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes how reusable query composition and declared
presentation intent become planner-owned, live-owned, and replay-owned query
artifacts instead of host-local helper behavior.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where scopes/templates/saved queries appear to work only because hosts
expand them privately, and where view shapes appear to work only because UI
code reinterprets generic query results after execution.

The milestone preserves authority boundaries because `worth-query` owns
composition and view semantics while the lower runtimes remain authorities for
truth, live execution, and durability.

The milestone defines proof obligations rather than implementation chores
because canonical parity, saved-artifact honesty, non-cosmetic view semantics,
live patch families, and exact counters are all required for closeout.

A competent engineer should be able to map this spec into honest `composition`,
`saved_query`, `view_shape`, support-profile, facade-witness, compile-fail, and
certification subdomains without inventing the architecture during
implementation.

This milestone belongs at 8 because it is the composition and presentation
contract layer that should exist before policy, tenant, and durable-artifact
work composes on top of it.

## Closeout Standard

Milestone 8 is complete only when all of the following are true:

- direct construction, scope expansion, and template instantiation preserve one
  canonical query meaning for admitted families
- ephemeral saved-query artifacts freeze semantic meaning without implying
  durable support
- admitted table/detail/observed-inspector/focused-inspector/grouped views are
  explicit, sealed, and planner-visible
- shipped view families affect planning, invalidation, delivery, and patch
  semantics rather than existing only as typing sugar
- unsupported composition families, unsupported view families, and durable
  overclaims fail typed and early
- certification bundles prove composition parity and view-shape semantic
  honesty through canonical machine-checkable artifacts

If code lands but scopes or templates still live only in host helper layers,
saved-query artifacts are still controller-local conventions, or view families
still have no planner-owned or live-owned semantics, Milestone 8 is not
complete.
