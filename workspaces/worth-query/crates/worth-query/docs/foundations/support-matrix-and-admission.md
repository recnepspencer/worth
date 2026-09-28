# Support Matrix And Admission

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

The public support matrix is Worth Query's explicit contract for what the
runtime facade supports now, what remains intentionally deferred, and what must
fail closed until a real implementation exists.

Admission is the executable form of that contract. It lets callers ask the
runtime whether a public family is actually available before they build on it.

## Why You Use It

- you need to know whether a public surface is truly supported or only visible
  so neighboring capability work extends the same facade
- you want typed denials instead of silent fallback when a family is deferred
  or unsupported
- you want one machine-checkable way to teach support posture in docs, product
  code, and certification
- you need to distinguish shipped runtime-backed temporal/async behavior on
  ordinary live handles from support-gated sibling facade-family roots

## Stable Entry Points

- `workspace.public_api_contract()`
- `workspace.public_handle_contract()`
- `workspace.public_support_matrix()`
- `workspace.public_downstream_delivery_contract()`
- `workspace.public_mutation_surface_report()`
- `workspace.admit_public_api_family(...)`
- `WorthQueryRuntimeFacadeFamily`
- `WorthQueryRuntimeFamilySupportStatus`

## Core Mental Model

There are three different questions here:

- what names exist in the public vocabulary
- which families are supported for real runtime-backed use
- which families are visible now so future work extends the same facade

The support matrix answers those questions row by row.

For mutation-specific preferred-versus-lower-level posture, pair it with
`workspace.public_mutation_surface_report()`.

When downstream consumers need to distinguish Query-owned support gaps from
local architecture residue, derive that evidence from the live support matrix,
admission boundary, and consumer-residue reports. Do not maintain a parallel
hand-edited support manifest.

Each row tells you:

- the surface or facade family
- whether it is `Supported`, `DeferredDebt`, or `Unsupported`
- whether it is ordinary runtime DX, support-gated, visible but deferred, or
  visible vocabulary only
- the owning roadmap closure or runtime gate
- whether it must fail closed
- whether future work is forbidden from creating a sibling API instead

Good to know:

- method presence is not a support claim
- `Intent` in the public facade means the shared intent vocabulary exists and
  covered intent families can be admitted when the runtime profile supports
  them
- it does not mean every intent-shaped operation is admitted as an ordinary
  production-facing runtime path in every runtime profile
- visible temporal and async rows also imply a declaration-side contract:
  once those live meanings are admitted into subscription declarations, their
  temporal basis and async request identity must be bound into one canonical
  declaration digest before lifecycle work starts
- temporal and async runtime meaning now also has an explicit remask gate:
  policy, tenant, relationship-proof, and schema-context drift must narrow or
  deny retained runtime meaning before public delivery, state, or inspection
  posture is projected
- runtime-backed reuse surfaces also have explicit temporal/async preservation
  posture: composition, view-shape admission, and saved-query freeze/reuse must
  preserve that meaning, defer it, require a fresh freeze, or deny it instead
  of silently degrading to ordinary-only reuse
- composition closure is also published through the application support report:
  when `QueryComposition` is admitted, the query-composition support profile now
  exposes `named_scope_expansion:verified` and `template_instantiation:verified`
  while observed-inspector and focused-inspector template neighbors stay
  visible-but-deferred and grouped collection templates are admitted directly
- that same published profile also closes the core runtime-backed view-family
  rows honestly: `table`, `detail`, `inspector_detail_observed`, and
  `inspector_detail_focused` are verified support rows, and `kanban_grouped`
  is now verified too. Remaining grouped follow-on work belongs to later
  durable/store-backed neighbors rather than runtime-backed grouped reusable
  composition/template closure or the grouped view-family row
- the application support report also publishes
  `support_report().identity_boundary_closure()`, whose posture is explicit:
  `Closed` means the identity-boundary residue scans are clean on the ordinary
  runtime-backed path, `Partial` means typed closure work exists but same-class
  residue or support posture still blocks full closure, and `Open` means the
  closure is genuinely not finished yet

## How It Executes

1. the runtime derives a public API contract from its support profile
2. the public support matrix freezes that into supported, support-gated,
   deferred, and unsupported rows
3. your code calls `workspace.admit_public_api_family(...)` before depending on
   a family that may be support-gated, deferred, or unsupported
4. admitted families return a sealed family contract
5. deferred or unsupported families deny typed and early

## Installed Operation Consumer Support

Runtime-installed operations carry a narrower support contract bound to one
operation and runtime generation. After binding, call:

```rust
let consumer = bound.consumer_projection_contract()?;
```

The contract derives requirements from the installed operation and evaluates
them against the Query-owned runtime support profile. It covers basis, live,
continuation, async/result state, recovery, inspection, projection consumption,
dependency impact, sharing, invalidation, collection delivery, and conditional
evaluation/comparator/trigger posture.

The word `recovery` in a consumer-support dimension does not mint a live
application-aftermath recovery handle. Receipt-bound recovery has its own
currentness, disclosure, effect-authority, and lifecycle checks.

Use `WorthQuerySupportRequirement::NotRequired` when the operation does not
need a dimension. Provider presence and downstream defaults cannot promote a
dimension to supported.

`WorthQueryConsumerBoundary` carries downstream presentation and allocation
requirements beside the Query contract. It cannot mutate Query requirements.

Compatibility admission returns a pair-bound witness or a
dimension-specific typed denial. A support report or matching digest is useful
for inspection but cannot authorize projection consumption.

## Consumer Snapshots And Pins

For downstream consumers, live support inspection is not always enough. A crate
may need to freeze the rows it depends on and fail its own build when those
rows regress. Use the [Consumer Kit](consumer-kit.md) for that job.

The consumer path is:

1. project the live matrix with `project_workspace_support_snapshot(...)`
2. export or load the schema-versioned support snapshot document when needed
3. declare required rows with `support_pinning_contract(...)`
4. evaluate the pin contract against the snapshot
5. fail through typed `WorthQuerySupportPinFinding` rows when required posture
   regresses

Support snapshots are projections of this matrix. They are not a second support
authority. Support pins bind to typed row identity and live row digests; a
local list of required family names is not real pinning.

## Graph Obligation Rows

Installed application queries and operations carry sealed graph-obligation
rows. The fixed kind vocabulary is:

- `GraphRead`
- `AuthorizationObservation`
- `MutationTouch`
- `EffectApplication`
- `InvariantExecution`

Each row also carries exact owner requirements, selection basis, resource and
effect posture, and a required terminal. Support is admitted through the
installed selection, graph-read review, execution-resource profile, and
capacity reservation. A support row or selected kind is never execution
evidence.

Application code does not register these rows or pin a consumer-authored
support table. Downstream code may inspect the installed set through
`worth_query_host::facade::inspect_installed_graph_obligations`, which grants no
selection, planning, or execution transition.

See [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
for the owner and terminal map.

## Graph Read Access Planning Rows

Graph read access planning is the access and cost contract for declared graph
reads. It is separate from graph touch obligation authority:
obligation rows describe graph meaning that must be checked, while graph read
access rows describe the access structures required to read graph-shaped data
without hidden N+1 traversal or unbounded local materialization.

The declaration is the authoring surface. The access plan is the accountability
surface. Support rows must make it clear whether a graph read executes inline,
uses bounded ephemeral access structures, streams, requires a persistent index,
requires async materialization, requires store-backed capability, requires
domain capability registration, or denies.

The graph read access admission posture vocabulary is:

- `inline_indexed`
- `bounded_ephemeral_index`
- `admitted_paged_streaming`
- `paged_streaming_required`
- `persistent_index_required`
- `async_materialization_required`
- `store_backed_capability_required`
- `access_capability_registration_required`
- `denied`

The graph read access denial vocabulary is:

- `budget_exceeded`
- `required_async_materialization`
- `required_access_capability_registration`
- `required_persistent_index`
- `unsupported_graph_index_support`

The graph read access requirement vocabulary is:

- `directional_adjacency`
- `reverse_adjacency`
- `predicate_support`
- `ordering_support`
- `traversal_workset`
- `visited_set`
- `dedup_set`
- `proof_support`
- `result_buffer`
- `materialization_lifecycle`
- `live_maintenance_support`
- `domain_operation_capability_registration`

The required capability owner vocabulary is:

- `query_runtime`
- `lower_runtime`
- `persistent_store`
- `domain_registration`
- `async_materializer`

Broad boolean graph reads and high-fanout traversals must not be described as
ordinary inline reads unless their admitted access plan proves that posture. If
the budget is exceeded, docs and support rows must preserve the typed denial
and suggested posture instead of recommending local relation loops or broad RAM
expansion.

Receipt proof is part of support honesty. A read that claims graph-read access
planning support must expose the consumed plan digest, admission digest,
requirement-set digest, plan-consumption digest, and execution counters through
the read receipt.

## Small Example

```rust
use worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily;

let workspace = runtime.workspace("support")?;

let matrix = workspace.public_support_matrix();
let live = matrix.row_for_family(WorthQueryRuntimeFacadeFamily::Live)?;

assert_eq!(live.status().as_str(), "supported");
assert!(!live.support_contract_digest().is_empty());

workspace
    .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Live)
    ?;
```

This is the smallest honest example because it shows both sides: inspect the
support matrix, then ask for executable admission.

## Real Example

```rust
use worth_query::facade::runtime::{WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily};

let workspace = runtime.workspace("future-gates")?;
let matrix = workspace.public_support_matrix();

let temporal = matrix.row("temporal")?;
let async_resource = matrix.row("async-resource")?;
let downstream_delivery = matrix.row("downstream-delivery-contract")?;
let intent = matrix
    .row_for_family(WorthQueryRuntimeFacadeFamily::Intent)
    ?;

assert_eq!(temporal.status().as_str(), "supported");
assert_eq!(temporal.teaching_posture().as_str(), "support-gate-only");
assert!(temporal.admission_fail_closed());
assert!(!temporal.ordinary_downstream_dx());

assert_eq!(async_resource.status().as_str(), "supported");
assert_eq!(async_resource.teaching_posture().as_str(), "support-gate-only");
assert!(async_resource.parallel_api_forbidden());

assert_eq!(downstream_delivery.status().as_str(), "supported");
assert_eq!(downstream_delivery.teaching_posture().as_str(), "support-gate-only");
assert!(downstream_delivery.support_contract_digest().is_some());

assert_eq!(intent.status().as_str(), "unsupported");
assert_eq!(intent.teaching_posture().as_str(), "visible-vocabulary-only");
assert!(intent.admission_fail_closed());
assert!(!intent.ordinary_downstream_dx());

for family in [
    WorthQueryRuntimeFacadeFamily::Intent,
    WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
    WorthQueryRuntimeFacadeFamily::DurableArtifacts,
] {
    let error = workspace
        .admit_public_api_family(family)
        .expect_err("deferred or unsupported families must fail closed");

    match error {
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
            assert_eq!(denial.family(), family);
            assert!(!denial.reason().is_empty());
        }
        other => panic!("expected typed support denial, got {other:?}"),
    }
}
```

Read that `Intent` row carefully. It means "do not teach blanket facade-family
intent support here." It does not erase the concrete covered intent
families described in [Intent Admission](../execution/intent-admission.md).

Also read the support-gated temporal and async family rows carefully. They no
longer mean "temporal or async semantics are absent." They mean Query has
shipped runtime-backed temporal/async meaning through ordinary live handles,
retained state/inspection, remask posture, and downstream delivery, while still
refusing to publish sibling `workspace.temporal(...)`-style runtime roots.

What is supported now:

- `Read`
- `Live`
- `Computed`
- `Effect`
- `BranchPreview`
- `Write`
- `Inspect`
- `Temporal`
- `AsyncResource`
- `MixedCauseDelivery`
- `temporal-async-certification`
- `temporal-async-remask`
- `downstream-delivery-contract`

What is support-gated:

- `Temporal` -> support-gated extension marker whose runtime-backed meaning extends ordinary live handles
- `AsyncResource` -> support-gated extension marker whose runtime-backed meaning extends ordinary live handles
- `MixedCauseDelivery` -> support-gated extension marker whose meaning projects through retained live and downstream delivery surfaces
- `temporal-async-certification` -> temporal/async certification closure

What is visible but deferred:

- `StoreBackedExecution` -> store-backed execution parity
- `DurableArtifacts` -> durable artifact reload and continuation

What is public vocabulary but not blanket facade-family support:

- `Intent`

Teaching posture is the quickest honest summary:

- `ordinary-runtime-dx` means downstream code can teach the family as part of
  the normal runtime surface
- `support-gate-only` means the row is shipped and machine-checkable, but it is
  a support or extension marker rather than a parallel runtime family you call
  directly
- `visible-but-deferred` means the family name is published now, but admission
  must fail closed because the underlying store-backed or durable neighbor is
  still roadmap debt
- `visible-vocabulary-only` means the public vocabulary exists, but normal
  runtime DX must not imply blanket family support

## How It Relates To Other Features

- Use [Workspace Overview](workspace-overview.md) when you are deciding what
  runtime DX you can rely on today.
- Use [State](state.md) when you need posture before execution exists.
- Use [Intent Admission](../execution/intent-admission.md) when you need the
  concrete covered intent families and their common-path or advanced-path
  usage, not just the facade-family support posture.
- Use [Inspection](../capabilities/inspection.md) when you are building
  tooling, certification, or explainability on top of runtime evidence.

## Inspection And Debugging

The public support matrix gives you row-by-row posture. The public API
contract and handle contract give you the richer details behind it.

Look for:

- `status()`
- `teaching_posture()`
- `ordinary_downstream_dx()`
- `owner_milestone()`
- `extension_rule()`
- `parallel_api_forbidden()`
- `admission_fail_closed()`
- `support_contract_digest()`

`owner_milestone()` is roadmap provenance. Product code should usually branch
on teaching posture, support status, fail-closed posture, extension rule, and
the support contract digest rather than hard-coding milestone names.

For deeper checks:

- `workspace.public_api_contract()` for family-level lane and evidence posture
- `workspace.public_handle_contract()` for handle families and required
  inspection sections
- `workspace.public_downstream_delivery_contract()` for the stable downstream
  delivery/resume contract that transport consumers should inherit
- `workspace.public_mutation_surface_report()` for preferred and lower-level
  mutation posture

## Anti-Patterns

- assuming a public method is supported because it exists
- teaching blanket intent support from the `Intent` facade-family row
- building runtime features against deferred async or temporal families without
  an admission gate
- adding a new public sibling API for future async work instead of extending
  the stabilized facade

## Current Limits

- the support matrix is the source of truth for facade-family posture today
- deferred families are intentionally visible before implementation so
  downstream work can plan honestly
- temporal/async and mixed-cause runtime-backed semantics are shipped now on
  ordinary live handles, state/inspection, and downstream delivery; the
  deferred rows refer to separate facade-family roots, not to absence of the
  runtime-backed feature itself
- temporal/async remask posture is a supported gate row now; it narrows runtime
  meaning before public projection instead of masking already-materialized
  delivery afterward
- downstream delivery is also a supported gate row now; it projects runtime
  delivery class, basis negotiation, and durable-resume debt into one
  Query-owned contract before transport consumers see the update
- admission tells you support posture; it does not perform the feature on your
  behalf

## Related Docs

- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Graph Touch Obligation Authority](../authoring/graph-touch-obligation-authority.md)
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Workspace Overview](workspace-overview.md)
- [State](state.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Intent Admission](../execution/intent-admission.md)
- [Inspection](../capabilities/inspection.md)
- [Writes And Intents](../execution/writes-and-intents.md)
