# Async Resources And Result State

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Async capabilities let you declare that a Query artifact depends on resource or
completion-driven work, then read the retained async state through the same
runtime surfaces you already use for live views, materialized facts,
continuations, and downstream delivery.

Use this when a declaration depends on an external, host, or bridge-backed
resource and you need that dependency to be part of Query's identity and
runtime model instead of a local `loading` flag or app-owned retry loop.

This feature does **not** give you a separate `workspace.async(...)` facade.
Async meaning is carried through existing Query surfaces.

## Why You Use It

- you want async/resource meaning to change declaration identity when the
  request really changed
- you want loading, failure, retry, stale, cancellation, or revalidation
  posture to stay Query-owned on live runtime surfaces
- you want projection consumption, continuation, and downstream delivery to
  preserve async posture instead of flattening it into ordinary rows or generic
  stale/failure notes
- you want families to fail closed when async behavior is unsupported or still
  deferred for that declaration family

## Stable Entry Points

Declaration-side authoring:

- `WorthQueryAsyncDeclarationClause::resource_request(...)`
- `WorthQueryAsyncDeclarationClause::completion_request(...)`
- `WorthQueryAsyncRequestIdentityPart::text(...)`
- `WorthQueryAsyncSourceFamily::{BridgeResource, ExternalResource, HostResource}`
- `WorthQueryAsyncLoadingPosture::{Blocking, BackgroundRefresh}`
- `WorthQueryAsyncFailurePosture::{FailClosed, RetainStaleValue}`
- `WorthQueryDeclarationFamilyMarker::async_declaration_support()`
- `WorthQueryInstalledDomainDeclarationContext::declare(...)`
- `WorthQueryInstalledDomainDeclarationContext::declare_checked(...)`

Runtime-backed observation and explanation:

- `workspace.state(...)`
- `workspace.inspections()?.inspect(...)`
- `workspace.downstream_delivery(...)`
- live inspection `async_result_state()`
- compact runtime posture `ordinary_runtime_posture().async_posture()`

Materialized fact and typed-stop remediation surfaces:

- `WorthQueryReadCompletion::consume_projection(...)`
- `read::project_facts()`
- `ProjectionConsumptionReceipt::materialized_fact_posture()`
- `execute_prepared_continuation(...)`
- `execute_prepared_continuation_outcome(...)`
- `recover_from_continuation_execution_checked(...)` projects descriptive
  remediation guidance from a typed stop

Good to know:

- families can report async support as `Unsupported`, `CanonicalIdentityOnly`,
  or `DeferredDebt`
- `CanonicalIdentityOnly` means the declaration can retain async meaning even
  if a broader runtime family is not a standalone public facade
- the public support matrix still treats blanket `AsyncResource` facade support
  as a gated family; the supported parts today live on existing declaration,
  live, inspection, projection-consumption, continuation, and downstream
  surfaces

## Core Mental Model

Think of async capabilities as three linked layers.

1. **Declaration identity**
   You describe the async dependency as part of the declaration itself:
   source family, request identity, loading posture, and failure posture.

2. **Retained runtime state**
   When that declaration participates in a runtime-backed live surface, Query
   retains one async result-state artifact such as `pending`, `current`,
   `failed`, `stale`, `cancelled`, `retried`, `revalidating`, `superseded`, or
   `denied`.

3. **Downstream consumers**
   Projection consumption, continuation remediation, and downstream delivery
   carry that async posture forward so callers do not have to reopen
   Bridge/runtime artifacts to understand what happened.

In this doc, "basis" means the concrete runtime identity the current async
state is bound to. If that identity drifts, Query should preserve typed drift
or denial posture instead of pretending the old async result is still current.

The important rule is:

- async meaning belongs to Query when it affects declaration identity or
  retained runtime posture
- app code should not rebuild that meaning as ad hoc fetch metadata

## How It Executes

1. A declaration family opts into async clauses through
   `async_declaration_support()`.
2. Your declaration input returns one or more
   `WorthQueryAsyncDeclarationClause` values.
3. Query normalizes and canonicalizes those clauses into declaration identity.
4. If the family does not admit async meaning, declaration admission fails
   closed as `AsyncUnsupported` or `AsyncDeferred`.
5. When a runtime-backed live surface retains async state, `workspace.state(...)`
   and `workspace.inspections()?.inspect(...)` project that retained result-state.
6. If you materialize or consume facts later, Query preserves async-backed
   posture on the contract, fact set, and receipt.
7. If continuation resumes against drifted async meaning, Query reports typed
   drift such as `AsyncRequestDrift`, `ReplayDrift`, `RemaskDrift`, or
   `StaleCompletion` instead of flattening the stop into generic stale/failure
   language.
8. If another runtime needs the latest live delivery, `workspace.downstream_delivery(...)`
   projects the retained async-aware delivery contract instead of forcing the
   consumer to reinterpret local callback order or stale delivery batches.

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryAsyncDeclarationClause, WorthQueryAsyncFailurePosture,
    WorthQueryAsyncLoadingPosture, WorthQueryAsyncRequestIdentityPart,
    WorthQueryAsyncSourceFamily,
};

fn async_resource_declaration_clauses(&self) -> Vec<WorthQueryAsyncDeclarationClause> {
    vec![WorthQueryAsyncDeclarationClause::resource_request(
        WorthQueryAsyncSourceFamily::BridgeResource,
        WorthQueryAsyncLoadingPosture::BackgroundRefresh,
        WorthQueryAsyncFailurePosture::RetainStaleValue,
        vec![
            WorthQueryAsyncRequestIdentityPart::text(
                "edge_ref",
                self.edge_ref.as_str(),
            ),
            WorthQueryAsyncRequestIdentityPart::text(
                "inspection_profile",
                "thermal-loss-audit",
            ),
        ],
    )]
}
```

This is the smallest honest example because it shows the real authoring
boundary: async meaning is declared as part of the Query declaration, not added
later by a transport adapter or UI loading model.

## Real Example

Imagine a building-inspection workflow where a live geometry declaration reads a
bridge-backed thermal profile for a specific warehouse door edge. The resource
should refresh in the background, keep the last good value during short bridge
outages, and also retain completion lifecycle so later continuation and
projection surfaces can tell whether a reconciliation run was fulfilled,
retried, or denied.

```rust
use worth_query::facade::foundation::{
    WorthQueryAsyncDeclarationClause, WorthQueryAsyncFailurePosture,
    WorthQueryAsyncLoadingPosture, WorthQueryAsyncRequestIdentityPart,
    WorthQueryAsyncSourceFamily, WorthQueryDeclarationCanonicalEntry,
    WorthQueryDeclarationInput, WorthQueryTemporalDeclarationClause,
    WorthQueryTemporalDuration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct InspectColdStorageDoorEdge {
    edge_ref: String,
    warehouse_ref: String,
}

impl WorthQueryDeclarationInput<GeometryDomain> for InspectColdStorageDoorEdge {
    type Family = InspectThermalProfile;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![
            WorthQueryDeclarationCanonicalEntry::text("edge_ref", self.edge_ref.clone()),
            WorthQueryDeclarationCanonicalEntry::text(
                "warehouse_ref",
                self.warehouse_ref.clone(),
            ),
            WorthQueryDeclarationCanonicalEntry::text(
                "inspection_mode",
                "cold-storage-audit",
            ),
        ]
    }

    fn temporal_declaration_clauses(&self) -> Vec<WorthQueryTemporalDeclarationClause> {
        vec![WorthQueryTemporalDeclarationClause::stale_after(
            WorthQueryTemporalDuration::seconds(45),
        )]
    }

    fn async_resource_declaration_clauses(&self) -> Vec<WorthQueryAsyncDeclarationClause> {
        vec![
            WorthQueryAsyncDeclarationClause::resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                vec![
                    WorthQueryAsyncRequestIdentityPart::text(
                        "edge_ref",
                        self.edge_ref.as_str(),
                    ),
                    WorthQueryAsyncRequestIdentityPart::text(
                        "warehouse_ref",
                        self.warehouse_ref.as_str(),
                    ),
                    WorthQueryAsyncRequestIdentityPart::text(
                        "sensor_profile",
                        "cold-storage-thermal-v3",
                    ),
                ],
            ),
            WorthQueryAsyncDeclarationClause::completion_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                vec![
                    WorthQueryAsyncRequestIdentityPart::text(
                        "edge_ref",
                        self.edge_ref.as_str(),
                    ),
                    WorthQueryAsyncRequestIdentityPart::text(
                        "reconciliation_kind",
                        "thermal-profile-refresh",
                    ),
                ],
            ),
        ]
    }
}
```

Then the family is declared through the normal admitted-handle path:

```rust
let declaration = handle.declare(InspectColdStorageDoorEdge {
    edge_ref: "warehouse-a:door-17:hinge-side".to_string(),
    warehouse_ref: "warehouse-a".to_string(),
})?;

assert_eq!(declaration.async_resource_clauses().len(), 2);
```

Later, when the runtime-backed live surface is active, you do **not** switch to
an async-specific facade. You read retained async posture from the existing live
surfaces:

```rust
let state = workspace.state(&door_edge_live)?;
let inspection = workspace.inspections()?.inspect(&door_edge_live)?;

let scalar_async = state
    .ordinary_runtime_posture()
    .and_then(|posture| posture.async_posture());

match inspection {
    WorthQueryInspection::LiveView(live) => {
        let async_state = live
            .async_result_state()
            .expect("thermal profile should retain async result-state");

        assert_eq!(scalar_async.expect("scalar async posture").as_str(), "current");
        assert_eq!(async_state.kind().as_str(), "current");
    }
    other => panic!("expected live inspection, got {other:?}"),
}
```

If an ordinary read completes over the same async-backed declaration, consume
facts from the completion. Query owns the result-shape and authorization
binding and keeps the async-backed posture on the consumption receipt:

```rust
let outcome = completion.consume_projection(
    read::project_facts()
        .display_field(profile_display_name_field_path()),
);
let (authority, _) = outcome
    .into_admitted()
    .expect("thermal read should stay admitted");

assert_eq!(
    authority
        .receipt()
        .materialized_fact_posture()
        .expect("async-backed posture should survive")
        .kind()
        .as_str(),
    "async_backed"
);
```

What is authoritative here:

- the admitted domain handle and declared Query meaning

What is derived:

- the retained async result-state for the live runtime surface
- the materialized async-backed fact posture on the consumption receipt

What gets retained automatically:

- request identity
- loading and failure posture
- basis and generation binding for runtime async result-state
- typed drift posture if continuation or replay no longer matches

## How It Relates To Other Features

- Use Installed Domain Declarations
  when you need to author async meaning directly on declaration input.
- Use [Subscription Selection And Diagnostics](subscription-selection-and-diagnostics.md)
  when the question is which live family Query should select for future-bearing
  live meaning.
- Use [Inspection](inspection.md) when you need the retained live async
  result-state or compact runtime posture.
- Use [Projection Consumption](projection-consumption.md) when async-backed
  materialized meaning must stay attached to typed consumed facts.
- Use [Continuation Pipeline](../domain-capabilities/continuation-pipeline.md)
  when async request drift or stale completion affects replay or resume.
- Use [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
  when another runtime needs the async-aware delivery contract rather than a
  local callback interpretation.

## Inspection And Debugging

These are the most useful surfaces when async behavior is not doing what you
expected:

- declaration authoring:
  `declaration.async_resource_clauses()`
- family admission:
  `handle.declare_checked(...)`
- scalar live posture:
  `workspace.state(&view)?.ordinary_runtime_posture()?.async_posture()`
- rich live explanation:
  `workspace.inspections()?.inspect(&view)?` then `async_result_state()`
- materialized fact posture:
  `WorthQueryConsumedProjectionAuthority::receipt().materialized_fact_posture()`
- downstream delivery:
  `workspace.downstream_delivery(&view)?`
- continuation drift:
  `execute_prepared_continuation_outcome(...)` and
  [Typed Stops And Remediation Guidance](../domain-capabilities/typed-stops-and-remediation-guidance.md)

When you inspect retained async runtime state, expect typed result-state values
such as:

- `pending`
- `current`
- `failed`
- `stale`
- `cancelled`
- `retried`
- `revalidating`
- `superseded`
- `denied`

When the problem is "why did this async event happen across runtime lanes?",
switch to [Cross-runtime causal inspection](cross-runtime-causal-inspection.md)
instead of expecting `workspace.inspections()?.inspect(...)` to become a second causal API.

## Anti-Patterns

- Treating async behavior as a UI-only `loading` enum instead of declaration
  meaning plus retained runtime posture.
- Encoding request identity in one opaque string when separate identity parts
  would let Query distinguish meaningful changes.
- Adding async clauses to a family that never opted into async support and then
  assuming the runtime will figure it out later.
- Building a second app-owned retry or stale-state taxonomy on top of Query's
  retained async result-state without a real product reason.
- Reopening raw runtime artifacts or transport callbacks in downstream code
  when `workspace.inspections()?.inspect(...)`, projection consumption receipts, or
  downstream delivery already expose the async posture you need.
- Treating async request drift, replay drift, remask drift, and stale
  completion as interchangeable continuation failures.

## Current Limits

- Query does not currently expose a separate async root facade. Async
  capabilities are carried through existing declaration, live, inspection,
  continuation, projection-consumption, and downstream-delivery surfaces.
- Families still opt into async meaning individually, and unsupported or
  deferred families must fail closed at declaration admission.
- Blanket `AsyncResource` facade-family support remains a support-matrix gate,
  not a promise that every runtime profile can expose every async pattern as
  ordinary product DX.
- Durable restart/reload parity and store-backed replay are still deferred.
- This feature preserves async meaning and runtime posture; it does not replace
  domain-specific retry policy or transport-specific serialization choices.

## Related Docs

- Installed Domain Declarations
- [Subscription Selection And Diagnostics](subscription-selection-and-diagnostics.md)
- [Inspection](inspection.md)
- [Projection Consumption](projection-consumption.md)
- [Continuation Pipeline](../domain-capabilities/continuation-pipeline.md)
- [Cross-runtime causal inspection](cross-runtime-causal-inspection.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
