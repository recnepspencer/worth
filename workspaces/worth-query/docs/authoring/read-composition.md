# Read Composition

## What This Feature Is

Read composition turns a typed application-query declaration into one installed
read graph that every supported execution lane shares. Use it when an
application needs a stable query shape across current, continuation,
historical, preview, and live execution.

Application authors declare roots, predicates, ordering, result fields,
relations, cardinality, basis support, and lane eligibility. Query installs the
canonical graph and performs access planning internally. Callers do not attach
an invariant callback or construct an admitted plan.

## Why You Use It

- Define one typed result shape instead of assembling rows after execution.
- Keep current, historical, preview, continuation, and live lanes on the same
  canonical read graph.
- Make traversal, projection, cardinality, ordering, and resource ceilings part
  of installed meaning.
- Let Query derive graph-read requirements and typed denials before execution.

## Stable Entry Points

Start with the **unified derive facade** (zero `ApplicationQueryDefinitionBuilder`
and zero string field names):

- `#[derive(WorthQueryRow)]` / `#[derive(WorthQueryProjection)]` (directly
  usable as `result = MyRow` on `#[derive(WorthQuery)]`)
- `#[derive(WorthQuery)]` supporting:
  - `where_equal` / `where_not_equal` / `where_lt` / `where_lte` / `where_gt` /
    `where_gte` (literal constants and required parameters)
  - `where_optional` / `where_equal_optional` / `where_not_equal_optional` /
    `where_lt_optional` / `where_lte_optional` / `where_gt_optional` /
    `where_gte_optional` for `Option<T>` parameters
  - `where_no_related(Relation, ...)` (`NOT EXISTS`) and
    `where_related_exists(Relation, ...)` (`EXISTS`)
  - `root_path(...)`, `order_by(...)`, `managed_index = Idx`, and
    `live = whole_result`
- `home.session(&me)` (`WorthQueryHostSession`) exposing
  `session.query(intent) -> Result<Vec<Row>, ...>`,
  `session.subscribe(intent) -> Result<WorthQueryTypedLiveSubscription, ...>`,
  `session.watch(intent)`, and `session.ensure_index_ready(intent)` / `session.ensure_default_index_ready::<Query>()`

Lower-level builder entry points for dynamic or specialized composition:

- `application_schema::fragment::{Fragment, Entity, Aspect, Record, Scalar}`
- `worth_query_application_query!`
- `ApplicationQueryDefinitionBuilder`
- `ApplicationQueryResultShapeBuilder`
- `ApplicationQueryResultShapeBuilder::relation_where_equal(...)` when one
  related child must be selected by a typed child field and query parameter
- `ApplicationQueryWhereFilterOperands` and `.where_filter(...)` to narrow
  selected root rows by a parameterized Foundation expression
- `ApplicationQueryRelatedPredicate`, `.where_no_related(...)`, and
  `.where_related_exists(...)` for bounded, metered related-record absence
  (`NOT EXISTS`) and existence (`EXISTS`) narrowing over declared relations
- `ApplicationQueryDisclosureContract::use_relation_by(...)` to govern internal
  relation reads in governed queries
- `ApplicationQueryResultFieldRef` for required fields
- `ApplicationQueryOptionalResultFieldRef` for optional fields
- `row.field(...)` and `row.optional_field(...)` during domain projection
- `row.disclosed_field(...)` and `row.disclosed_optional_field(...)` for
  governed results
- typed parameter, predicate, ordering, field, and relation references
- installed-query inspection through `worth_query_host::facade::domain`
- ordinary typed application-query execution through the host runtime

There is no `compose_read_with_invariant_pack(...)` or
`define_read_family_with_invariant_pack(...)`. Read-only application work never
enters proposal or invariant phases.

## Core Mental Model

The declaration owns semantic shape. Installation owns canonical identity.
Admission owns access requirements, inventory matching, cost, budget, and
capacity. Execution owns the branch- and basis-bound read session. Publication
owns a non-authoritative view of the terminal result.

```text
typed query declaration
  -> installed canonical read graph
  -> installed GraphRead obligation
  -> specialized access review and reserved plan
  -> managed session and session-owned read port
  -> shaped result and read terminal
  -> publication receipt and inspection
```

The same installed graph is reused across lanes, but each attempt has its own
plan, session, and basis identity. Equal graph meaning does not make attempts
interchangeable.

Required-versus-optional presence is part of the installed result shape. A
required selector means the field must exist on every returned record. An
optional selector means the schema permits that field to be absent. Query
preserves that distinction through installation and execution; it does not use
an empty string, zero, or another sentinel for absence.

Disclosure is a separate decision. For a public optional field, `Option<T>` is
`Some(value)` or lawful `None`. For a governed optional field,
`WorthQueryApplicationDisclosed<Option<T>>` can instead be `Omitted` when the
caller is not allowed to learn either the value or its absence. `Omitted` never
means that the schema value was missing.

## How It Executes

1. The schema installs the typed query definition.
2. Installation derives one canonical read graph and one `GraphRead`
   obligation row.
3. Admission derives requirements from that exact graph and reviews runtime
   inventory and budget.
4. Query reserves capacity and opens a managed session on the typed branch and
   basis.
5. The session-owned read port reaches the graph owner.
6. The semantic query layer shapes the returned rows.
7. Execution seals the terminal only after exact cleanup and release evidence.

Historical, preview, continuation, and live lanes change their basis or
lifecycle posture. They do not construct another graph-read plan.

## Recommended Unified Derive Example

Declare the row and the complete filtered, related-absence, whole-result live
query directly on the Rust structs:

```rust
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DaymarkSchema, root = Task, namespace = ROW_NS)]
pub struct TaskRow {
    #[worth_query(field = TaskKey)] pub task_key: u64,
    #[worth_query(field = Title)]   pub title: String,
    #[worth_query(field = State)]   pub state: String,
    #[worth_query(field = Priority)] pub priority: u64,
    #[worth_query(field = DueDate)] pub due_date: Option<CanonicalDate>,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema, context = DaymarkContext, namespace = READY_NS,
    result = TaskRow, root = Task, scope = ProjectKey, fields = TaskFacts,
    root_path(Contains),
    where_equal(state = "OPEN"),
    where_optional(priority = priority),
    where_lte_optional(due_date = due_before),
    where_no_related(BlockedBy, state = "OPEN"),
    order_by(TaskRow::task_key(), Ascending),
    live = whole_result,
    limits = 64,
)]
pub struct ReadyTasks {
    #[worth_query(scope)] pub project: u64,
    pub priority: Option<u64>,
    pub due_before: Option<CanonicalDate>,
}

let session = home.session(&me);
let rows: Vec<TaskRow> = session.query(ReadyTasks {
    project: 1, priority: Some(1), due_before: Some(cutoff),
})?;
let mut live = session.subscribe(ReadyTasks {
    project: 1, priority: None, due_before: Some(cutoff),
})?;
let current: Option<&[TaskRow]> = live.current();
if let Some(updated) = live.refresh(&session)? {
    let _ = updated;
}
live.close()?;
```

## Small Builder Example

```rust
worth_query_application_query!(
    AccountActivity in BankSchema,
    parameters AccountActivityParameters,
    result AccountActivityResult,
    scope Account,
    name "account_activity"
);
```

The marker is only the typed query identity. The schema also installs its full
definition and result shape.

Declare required and optional result positions explicitly:

```rust
let shape = ApplicationQueryResultShapeBuilder::<
    BankSchema,
    AccountDetail,
    Account,
    AccountDetailResult,
>::new(Account::reference())
.field(account_id())
.optional_field(customer_note())
.build();

let result = AccountDetailResult {
    account: row.field(account_id())?,
    note: row.optional_field(customer_note())?,
};
```

The field declaration behind `customer_note()` must itself be optional. The
required and optional selector types cannot be substituted for one another.

## Real Example

```rust
let shape = ApplicationQueryResultShapeBuilder::<
    BankSchema,
    AccountActivity,
    Account,
    AccountActivityResult,
>::new(Account::reference())
.field(account_id_result)
.optional_field(account_note_result)
.relation(activity_relation, activity_shape)
.build();

let definition = ApplicationQueryDefinitionBuilder::declare(AccountActivity::reference())
.root(Account::reference())
.scope(Account::reference())
.result_shape(shape)
.cardinality(ApplicationQueryCardinality::Many)
.disclosure(ApplicationQueryDisclosureContract::installed_policy("account-holder"))
.basis_support(ApplicationQueryBasisSupport::current_and_pinned())
.lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
.public()
.parameter(account_parameter)
.where_equal(AccountId::reference(), account_parameter)
.order_by(activity_sequence, ApplicationQueryOrderingDirection::Descending);
```

Filter a relation at the result-shape boundary when the application needs one
specific child from a larger adjacency:

```rust
let occurrence_shape = ApplicationQueryResultShapeBuilder::new(
    GeometricOccurrence::reference(),
)
.field(occurrence_transform());

let shape = ApplicationQueryResultShapeBuilder::new(BodySet::reference())
    .relation_where_equal(
        selected_occurrence(),
        occurrence_shape,
        GeometricOccurrenceKey::reference(),
        selected_occurrence_parameter,
    )
    .build();
```

Query applies the typed equality predicate to actual relation targets before
relation cardinality and child projection. The predicate field must be declared
equality-queryable and must belong to the nested child entity. Source evidence
retains every examined sibling and the predicate aspect revision, including
siblings excluded from the public result. A changed excluded sibling therefore
invalidates source authority. Filtered relations cannot also be continuation
targets because that pagination contract is not defined.

### Residual where filters

`where_equal` and root paths select candidates structurally. A where filter
then narrows those candidate rows with one parameterized Boolean expression,
admitted statically against the root entity's fields and the query's typed
parameters:

```rust
let filter = ApplicationQueryWhereFilterOperands::new()
    .field("title", Title::at("Title"))?
    .field("notes", Notes::at("Notes"))?
    .parameter("q", search_text)?
    .parse("contains_ignore_ascii_case(title, q) || contains_ignore_ascii_case(notes, q)")?;

ApplicationQueryDefinitionBuilder::declare(TaskSearch::reference())
    // .root(..).scope(..).result_shape(..) and controls as above
    .root_path(ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()))
    .parameter(search_text)
    .where_filter(filter)
    .order_by(key(), ApplicationQueryOrderingDirection::Ascending);
```

Foundation owns the language, typing and metered evaluation; there is no
second predicate language. An optional field is an `Option` operand
(`is_some(due) && unwrap(due) <= max_due`).

Optional query parameters use `ApplicationQueryParameterRef::optional(name)`,
declare `.parameter(reference)` on the query definition, bind a typed
`Option<T>` via `ApplicationQueryParameterSet::bind_optional(reference, value)`,
and enter a `where_filter` as `Option` operands (`!is_some(limit) || ...`).
This lets one query combine multiple independently optional filters without
multiplying query definitions. Structured `where_equal` predicates and
managed-index probes still require non-optional parameters.

Fixed scalar constants do not need dummy runtime parameters:
- Use `.where_equal_value(field, ApplicationEncodedScalarValue::try_new(value)?)`
  on `ApplicationQueryDefinitionBuilder` for index-seeking or root-path equality
  narrowing against an equality-queryable field.
- Use `.literal(name, ApplicationEncodedScalarValue::try_new(value)?)` on
  `ApplicationQueryWhereFilterOperands` to bind a typed authored constant
  (including `CanonicalDate` or domain-encoded scalars) directly inside a
  residual `where_filter` expression.

Unknown names, non-Bool results,
type mismatches, unused operands and unit-bearing values refuse when the
filter is admitted; undeclared parameters, parameter family or optionality
mismatches, invalid literal values and fields outside the root entity refuse at
schema declaration with `ApplicationQueryDefinitionDenial`.

At execution the filter runs after index-seeking selection and before
ordering, cardinality and result limits. Each candidate is a safe point;
evaluation runs in bounded slices whose work is charged to the request, and
cancellation or deadline between slices refuses with a typed denial. Every
examined candidate's filter fields join the result's set evidence, including
rows the filter excluded and fields the result never projects, so editing a
filter-only field stales sources observed earlier. Governed queries must grant
each filter field the membership influences their lanes expose, exactly as for
`where_equal`.

Current bounds: filters apply to exact-root and root-path selections; orbit,
component, managed-index (including managed Sources) and continuation shapes
refuse. Operands are the root entity's own fields, not traversed entities.
Decimal operands wait for a schema Decimal field binding, and unit-bearing
Quantity operands wait for a schema-unit to expression-dimension contract;
until then unit-bearing operands refuse with `UnitBearingOperand`. Unfiltered definitions keep their exact canonical identity and
archive bytes. The public journeys are in the
[where-filter court](../../crates/worth-query-court-decl-host-where-filter/tests/worth_query_court_decl_host_where_filter/where_filter/mod.rs).

### Related-record absence and existence predicates

Use `.where_no_related(...)` (`NOT EXISTS`) or `.where_related_exists(...)`
(`EXISTS`) on `ApplicationQueryDefinitionBuilder` to narrow selected root
rows by the absence or presence of matching related records across a declared
forward or reverse relation (for example, Daymark's `Ready` view selecting open
tasks with zero open `BlockedBy` blockers):

```rust
let open = ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".into())?;

ApplicationQueryDefinitionBuilder::declare(ReadyTasksQuery::reference())
    // .root(..).scope(..).result_shape(..) and controls as above
    .root_path(ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()))
    .where_equal_value(Status::at("Status"), open.clone())
    .where_no_related(
        ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
            .where_equal_value(Status::at("Status"), open),
    )
    .order_by(key(), ApplicationQueryOrderingDirection::Ascending);
```

A related predicate can be unfiltered (any live neighbor across the relation)
or combine typed `.where_equal(field, param)`, `.where_equal_value(field, val)`,
and `.where_filter(filter)` predicates evaluated against the target entity's
fields. Admission derives the required forward or reverse adjacency index from
the installed graph automatically. Execution checks governance for both the
relation (`use_relation_by`) and every referenced target field (`use_field_by`)
before touching any rows or edges, meters every frontier/adjacency scan, sort,
dedup, memo lookup, and target evaluation against the caller's work and memory
budgets, and records each examined candidate's adjacency structural revision
and each examined target's guarded fields in the observed source footprint so
mutations to a root, relation edge, or blocker's guarded field invalidate prior
sources with zero stale reads. Definitions without related predicates keep
their exact canonical identity and archive bytes. Current bounds: exact-root
and root-path selections only (orbit, component, managed-index, and
continuation shapes refuse). The formal model is in
`specs/tla/related_absence/RelatedAbsence.tla`, and the public journeys and
64-seed differential graph DST live in the
[related-absence court](../../crates/worth-query-court-decl-host-where-filter/tests/worth_query_court_decl_host_where_filter/related_absence/mod.rs).

### Whole-result live subscriptions

When a live query's membership depends on residual filters (`where_filter`,
negative evidence) or related-record predicates (`where_no_related` /
`where_related_exists`, related-record evidence) rather than single-target
effect routing (`live_root_by` / `live_by`), declare `.live_whole_result()` on
a live-enabled `ApplicationQueryDefinitionBuilder`:

```rust
ApplicationQueryDefinitionBuilder::declare(ReadyLiveQuery::reference())
    // .root(..).scope(..).result_shape(..) and controls as above
    .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
    .live_whole_result()
    .root_path(ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()))
    .where_equal_value(Status::at("Status"), open.clone())
    .where_no_related(
        ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
            .where_equal_value(Status::at("Status"), open),
    )
    .order_by(key(), ApplicationQueryOrderingDirection::Ascending);
```

Open a whole-result live subscription through the ordinary Host request surface
with `subscribe_whole_result(limits)` (or `subscribe_whole_result_governed(...)`
/ `subscribe_whole_result_approved(...)` for capability-governed queries, plus
`*_in_advancement` variants for funded execution phases). Inspect the initial
snapshot via `sub.initial()` / `sub.take_initial()`, poll for committed updates
with `sub.next(&fresh_request)` (returning `Delivered(update)`, `Pending`,
`AuthorizationDenied`, `StalePrincipal`, `ExecutionRequest`, `Overflow`, or
`Closed`), and close cleanly with `sub.close()`.

Whole-result live subscriptions reuse the same installed canonical read graph
and preserve the complete deduplicated invalidation footprint across both
`result_set_observation` (scope paths, candidate filter fields including
excluded rows, related-record adjacencies, and examined blocker target fields)
and returned-row `observed_sources` (projected row fields). Every `next` call
re-admits principal and capability authority against current branch truth
before polling or delivering any queued commit batch, so revoked grants or
disabled principals fail closed with zero leaked rows. Unrelated-project edits
and unexamined aspects leave the footprint current and return `Pending` without
waking. Queries without `.live_whole_result()` keep their exact prior
canonical identity and archive bytes. Current bounds: exact-root and root-path
selections only (combining with `live_cause`, `orbit`, `component_selection`,
`managed_index`, or `continuation` refuses). The formal model is in
`specs/tla/live_subscriptions/LiveSubscriptions.tla`, and the end-to-end courts
and 64-seed differential DST live in the
[whole-result live court](../../crates/worth-query-court-decl-host-where-filter/tests/worth_query_court_decl_host_where_filter/live_whole_result/mod.rs).

When the definition is governed, give the optional slot its own disclosure
rule and keep policy omission typed in the result:

```rust
let disclosure = ApplicationQueryDisclosureContract::governed_by(
    "account-detail",
    ViewAccountDetailCapability::reference(),
)
.disclose_field_by(
    account_id(),
    AccountField::Identity,
    ApplicationQueryInfluenceContract::forbid_all(),
)
.disclose_optional_field_by(
    customer_note(),
    AccountField::CustomerNote,
    ApplicationQueryInfluenceContract::forbid_all(),
);

let note = row.disclosed_optional_field(customer_note())?;
// WorthQueryApplicationDisclosed<Option<String>>
```

Query evaluates disclosure before domain projection. Application projection
therefore never receives a protected optional value and cannot infer whether
that value was absent.

The complete compiling declarations are exercised in
`worth-query-installation/src/application_query/tests.rs` and the Bank domain.
The external public progression is exercised in the
[host graph-progression court](../../crates/worth-query-court-decl-host-integration/tests/worth_query_court_decl_host_integration/host/canonical_graph_progression/mod.rs).

## How It Relates To Other Features

- [Graph Read Access Planning](graph-read-access-planning.md) explains the
  specialized access review inside admission.
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
  explains how the installed read enters one session and terminal chain.
- Live views reuse the same installed graph while adding retained delivery and
  close semantics.
- Historical and preview execution preserve the typed branch and basis instead
  of defaulting to current truth.
- Generic workspace reads were retired with the `worth-query` engine.

## Inspection And Debugging

Inspect:

- installed query and canonical read-graph identities;
- result fields, relation paths, cardinality and ordering;
- graph-read obligation and selection identity;
- requirement, cost, budget, and inventory review;
- plan, session, branch, and basis identity;
- traversal and projection counters; and
- terminal release and publication evidence.

A read denial should identify whether declaration, parameter binding, access
support, budget, basis, authorization, execution, or disclosure stopped the
request. Do not replace a denial with caller-owned traversal.

## Anti-Patterns

- Constructing a raw application-query plan in the caller.
- Calling Relational directly from one lane.
- Giving live, preview, or historical execution a separate planner.
- Passing a manual invariant callback to a read.
- Treating a read result as proposal or commit authority.
- Declaring an optional schema field with `.field(...)`, or a required field
  with `.optional_field(...)`.
- Converting lawful `None` into a disclosure omission, or converting `Omitted`
  into `None`.
- Using sentinel values such as an empty string or zero to represent optional
  absence.
- Rebuilding relation trees or N+1 neighbor lookups after a typed denial.

## Current Limits

- Unsupported access structures deny or require a stronger admitted posture.
- Read-only execution cannot enter proposed-state, invariant, or commit phases.
- Optional result fields cover scalar field presence. Relation cardinality is
  declared separately with exact, optional-one, or many relation selectors.
- Multiple branch heads and concurrent branch writers remain outside the
  current application-query contract.
- Generic workspace read-family APIs were retired with the `worth-query`
  engine.

## Related Docs

- [Graph Read Access Planning](graph-read-access-planning.md)
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md)
