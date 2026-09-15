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

- `worth_query_application_schema!`
- `worth_query_application_query!`
- `ApplicationQueryDefinitionBuilder`
- `ApplicationQueryResultShapeBuilder`
- `ApplicationQueryResultShapeBuilder::relation_where_equal(...)` when one
  related child must be selected by a typed child field and query parameter
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

## Small Example

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
.dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 3))
.disclosure(ApplicationQueryDisclosureContract::installed_policy("account-holder"))
.basis_support(ApplicationQueryBasisSupport::current_and_pinned())
.lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
.public()
.parameter(account_parameter)
.where_equal(AccountId::reference(), account_parameter)
.order_by(activity_sequence, ApplicationQueryOrderingDirection::Descending)
.build()?;
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
The external public progression is exercised in
`worth-query-host/tests/canonical_graph_progression.rs`.

## How It Relates To Other Features

- [Graph Read Access Planning](graph-read-access-planning.md) explains the
  specialized access review inside admission.
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
  explains how the installed read enters one session and terminal chain.
- Live views reuse the same installed graph while adding retained delivery and
  close semantics.
- Historical and preview execution preserve the typed branch and basis instead
  of defaulting to current truth.
- Generic workspace reads remain separate non-application functionality.

## Inspection And Debugging

Inspect:

- installed query and canonical read-graph identities;
- result fields, relation paths, cardinality, ordering, and dependency ceiling;
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
- Generic workspace read-family APIs are preserved but are not the application
  front door described here.

## Related Docs

- [Graph Read Access Planning](graph-read-access-planning.md)
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md)
- [Declarative Query Experience](../capabilities/declarative-query-experience.md)
