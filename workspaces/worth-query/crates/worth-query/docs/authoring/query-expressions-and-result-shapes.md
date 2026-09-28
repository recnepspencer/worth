# Typed Query Expressions And Result Shapes

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

This is Worth Query's authoring surface for building query shapes and result
shapes as typed artifacts before they become canonicalized bundles or runtime
surfaces. It includes both raw authoring builders and schema-typed builders.

## Why You Use It

- you want a reusable query artifact outside a one-off `workspace.live_view(...)`
  closure
- you need canonicalization inputs for tooling, saved queries, validation, or
  other authoring workflows
- you want compile-time schema field selection instead of stringly-typed
  selectors
- you need query family and result-shape family compatibility to fail before
  runtime execution

## Stable Entry Points

- raw authoring:
  `DetailQueryBuilder`, `CollectionQueryBuilder`,
  `DetailResultShapeBuilder`, `CollectionResultShapeBuilder`,
  `GuidedAuthoringPath`
- typed authoring:
  `TypedDetailQueryBuilder`, `TypedCollectionQueryBuilder`,
  `TypedDetailResultShapeBuilder`, `TypedCollectionResultShapeBuilder`,
  `TypedGuidedAuthoringPath`
- supporting vocabulary:
  `QueryFamily`, `ResultShapeFamily`, `RootEntityKey`,
  `AuthoredResultShapeField`

This is an authoring feature, not the stabilized runtime workspace facade.
Ordinary runtime DX still prefers `workspace.live_view(...)` closures when you
do not need reusable query artifacts.

## Core Mental Model

There are two parallel contracts:

- the query contract: root entity, projections, traversal, predicates, ordering
- the result-shape contract: which projected fields become delivered output and
  under what names

The guided path pairs them so the families match and the result shape only asks
for projected fields.

Typed builders add one more layer:

- schema root types
- typed field markers for projection, filtering, ordering, and traversal

The runtime does not need these builders to execute a live view, but these
builders are how you author durable query artifacts cleanly and safely.

## How It Executes

1. Build a detail or collection query.
2. Build a matching detail or collection result shape.
3. Pair or canonicalize them through a guided path.
4. If the families mismatch or the result shape references an unprojected
   field, authoring fails early.
5. The resulting canonical bundle can be used by later validation, planning,
   or runtime lowering paths.

## Small Example

```rust
use worth_query::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, CollectionQueryBuilder,
    CollectionResultShapeBuilder, GuidedAuthoringPath, OrderingSelector,
    RootEntityKey,
};

let root = RootEntityKey::new("Task").unwrap();

let query = CollectionQueryBuilder::new(root)
    .project(AspectFieldSelector::new("identity", "id").unwrap())
    .project(AspectFieldSelector::new("title", "value").unwrap())
    .order_by(OrderingSelector::ascending("title", "value").unwrap())
    .build()
    .unwrap();

let result_shape = CollectionResultShapeBuilder::new()
    .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
    .field(AuthoredResultShapeField::new("title", "value", "title").unwrap())
    .build()
    .unwrap();

let bundle = GuidedAuthoringPath::canonicalize_collection(query, result_shape).unwrap();
```

This is the smallest honest example because it shows the whole contract:
projection, result shape, and compatibility pairing.

## Real Example

```rust
use worth_query::facade::runtime::{
    TypedCollectionQueryBuilder, TypedCollectionResultShapeBuilder,
    TypedGuidedAuthoringPath, TypedOrderableField, TypedPresenceField,
    TypedProjectableField, TypedSchemaField, TypedSchemaRoot,
    TypedStringContainsField,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct TaskSchema;

impl TypedSchemaRoot for TaskSchema {
    const ROOT_ENTITY: &'static str = "Task";
}

struct TitleValue;
impl TypedSchemaField for TitleValue {
    type Schema = TaskSchema;
    const ASPECT: &'static str = "title";
    const FIELD: &'static str = "value";
}
impl TypedProjectableField for TitleValue {}
impl TypedStringContainsField for TitleValue {}
impl TypedOrderableField for TitleValue {}

struct StatusValue;
impl TypedSchemaField for StatusValue {
    type Schema = TaskSchema;
    const ASPECT: &'static str = "status";
    const FIELD: &'static str = "value";
}
impl TypedProjectableField for StatusValue {}
impl TypedPresenceField for StatusValue {}

let query = TypedCollectionQueryBuilder::<TaskSchema>::new()
    .project::<TitleValue>()
    .project::<StatusValue>()
    .where_contains::<TitleValue>("approval")
    .where_present::<StatusValue>()
    .order_by_ascending::<TitleValue>()
    .build()
    .unwrap();

let result_shape = TypedCollectionResultShapeBuilder::<TaskSchema>::new()
    .field::<TitleValue>()
    .field_as::<StatusValue>("status")
    .build()
    .unwrap();

let canonical = TypedGuidedAuthoringPath::canonicalize_collection(query, result_shape).unwrap();
```

What is typed here:

- schema root
- projectable fields
- filterable fields
- orderable fields
- delivered result field aliases

What still happens at authoring time rather than runtime execution:

- family matching
- query/result-shape compatibility
- canonical bundle creation

## How It Relates To Other Features

- Use this when you need reusable query authoring artifacts rather than the
  immediate closure form in [Live Views](../runtime-surfaces/live-views.md).
- Use it before validation, canonicalization, planning, or saved-query style
  workflows.
- Use Installed Domain Declarations
  when you need declaration-native temporal meaning such as freshness limits,
  intervals, deadlines, or rolling/sliding windows, or declaration-native
  async/resource meaning such as canonical source family, request identity,
  loading posture, and failure posture.
- Use [Async Resources And Result State](../capabilities/async-resources-and-result-state.md)
  when you want the end-to-end async story instead of only the declaration-side
  authoring boundary.
- Use the runtime workspace closure builders when you just want ordinary live
  view DX inside a runtime-backed app surface.

## Inspection And Debugging

This feature does not use `workspace.inspections()?.inspect(...)` because it is an authoring
surface, not a runtime handle family.

The main debugging signals are:

- `AuthoringError` for malformed authoring input such as empty projection sets,
  empty result-shape field sets, or invalid roots
- `AuthoredBundleError` for query/result-shape compatibility problems
- `QueryCanonicalizationError` when the paired request cannot canonicalize

Important things to watch:

- `QueryFamily` versus `ResultShapeFamily`
- whether every delivered shape field was projected
- whether the typed schema constants map to the real aspect and field names you
  intended

## Anti-Patterns

- Using raw strings everywhere when the same schema is stable enough for typed
  field markers.
- Treating result shapes as optional after authoring a query.
- Assuming authoring-time success means runtime support for every future live,
  policy, history, or async scenario.
- Reaching for typed authoring builders when a simple `workspace.live_view(...)`
  closure is enough.
- Attaching temporal freshness or interval behavior here when the real intent
  belongs to a domain declaration.
- Attaching async source identity or resource loading semantics here when the
  real intent belongs to a domain declaration.

## Current Limits

- This feature stabilizes authoring contracts, not temporal or async runtime
  execution.
- This feature is not the declaration-native temporal clause surface. Typed
  query builders do not replace `WorthQueryTemporalDeclarationClause` or
  `WorthQueryAsyncDeclarationClause`.
- The typed builders help with schema-safe authoring, but they do not replace
  runtime support admission or runtime inspection.
- This page is about query and result-shape authoring only, not the broader
  saved-query, policy, or historical composition feature families.

## Related Docs

- [Live Views](../runtime-surfaces/live-views.md)
- [Reads Observe And Materialize](../runtime-surfaces/reads-observe-materialize.md)
- [Async Resources And Result State](../capabilities/async-resources-and-result-state.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)


