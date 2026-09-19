# Query-Backed UI Views

## What This Feature Is

Query-backed views let Worth UI mount native values produced by an installed
Worth Query runtime. Query is the application-facing composition authority and
owns its execution contract, result posture, compatibility, recovery, and
resource lifecycle; the domain and lower runtimes retain the truth they own.
Worth UI owns the declared projection requirement, UI consequence, mounted
identity, and presentation.

## Why You Use It

- Present scalar or keyed collection data without copying Query truth into UI.
- Preserve pending, current, stale, revalidating, and exact stop posture.
- Carry native values into mounted semantic text without JSON or string
  reconstruction.
- Rebind only declared consumers when Query meaning changes.
- Correlate Query evidence, a projection fact, mounted identity, and pixels.

## Stable Entry Points

- `worth_ui::facade::query_binding::WorthUiStatusSourceOwner`
- `worth_ui::facade::query_binding::WorthUiPresentationAsyncHostPlan`
- `UiScalarProjectionRegistration`
- `UiCollectionProjectionRegistration`
- `UiScalarProjectionObservation`
- `UiCollectionProjectionObservation`
- `UiProjectionAvailability`
- `UiPresentProjection`
- `WorthUiApplicationBuilder::register_scalar_projection(...)`
- `WorthUiApplicationBuilder::register_collection_projection(...)`
- `WorthUiNativeApplicationShell::begin_projection_rebind(...)`

There is no product `WorthUiQueryWorkspaceExt` import. Query-free applications
do not install a dummy Query runtime or register a dummy projection.

The governing Query audience crates are:

- `worth-query-decl` for application declarations and requirements;
- `worth-query-host` for hosted installation and progression; and
- `worth-query-replay` only for certification-owned replay or reconstruction.

Ordinary UI code does not use `worth-query-replay`, and product applications do
not import the raw Query engine as a convenience surface.

## Core Mental Model

A projection has orthogonal contracts:

- shape: scalar or collection;
- schema: selected fields, native family, and row identity where applicable;
- lifecycle: snapshot or live;
- availability: unavailable, present, or stopped;
- currency/activity: current or retained stale, optionally revalidating;
- compatibility: exact admitted replacement or a typed stop;
- budget: bounded accesses, rows, bytes, and retained resources.

These are types, not fields an application may reassemble. A reporting identity
cannot become a binding, an inspection projection cannot become an operational
observation, and a collection observation cannot enter a scalar consumer.

## Small Example

### Register A Projection

An installation owner first obtains an `UiInstalledProjectionView` from Query
authority. Product application code then declares the required shape and
registers it before `freeze()`.

The following fragment is compiled inside the public facade contract.

<!-- compile-pass-fragment:register_scalar_projection -->
```rust
fn register_scalar(view: UiInstalledProjectionView) {
    let registration = UiScalarProjectionRegistration::text(
        view,
        UiProjectionFieldRequirement::declared("status").expect("valid selected field"),
    );
    let _app = worth_ui::facade::app::WorthUi::app()
        .with_change_profile(
            worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
        )
        .register_scalar_projection(registration)
        .expect("installed scalar projection registration");
}
```

For a collection, use `UiCollectionProjectionRegistration::text(...)` or
`native(...)` with an explicit row-identity field, selected fields,
completeness requirement, and continuation posture. Registration never
executes Query or creates UI-local result state.

## Install The Production Query Runtime

Platform Pulse installs its authored status source and a separate
presentation-only async package:

```text
WorthUiStatusSourceOwner::install()
-> obtain the initial application scalar projection
-> register it on WorthUi::app()
WorthUiPresentationAsyncHostPlan::prepare()
-> split the presentation request and completion
-> worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::install(...)
-> completion.complete(installation)
```

The authored source owns application Query progression. The presentation
installation contains no second status source. The host audience remains the
Query progression boundary; Worth UI does not emulate it or reach into the raw
engine.

## Observe And Rebind

The live owner issues one affine observation. Submit it through
`begin_projection_rebind(...)`, exhaust the typed rebind outcome, and return the
released shape-specific observation to the Query lifecycle owner. Publication
completion is what allows that owner to advance again.

```text
Query-issued observation
-> ordinary 3.12 observation admission and classification
-> indexed affected-scope plan
-> mounted semantic text
-> host presentation and atomic publication
-> released scalar or collection fact
-> Query publication admission
```

`UiProjectionAvailability` must be matched exhaustively. `Present(Current)`
carries current native value. `Present(RetainedStale { ... })` carries the
predecessor plus activity. `Unavailable` and `Stopped` carry exact typed
posture and mint no fabricated value.

## Replacement And Invalidation

Compatible replacement preserves logical binding identity only through
Query-issued compatibility proof. Schema, native-family, payload-shape, row
identity, world, generation, basis, and budget mismatches stop before a
successor. The predecessor remains available only when the typed outcome says
it was retained.

Changed scalar and collection facts enter the same source/viewport observation
ordering used by the rest of the application. Runtime follows declared
consumer indexes; it does not poll Query, scan the mounted graph, or introduce a
second executor.

The closed Milestone 3.14.1 host supplies the native production ingress used by
this path. It does not move Query execution authority into the UI runtime or
product facade.

## Cost And Lifecycle

- Query-free and unchanged turns perform zero projection/content work.
- Scalar access is bounded by declared selected fields.
- Collection work scales with selected or changed rows, not total collection
  size; completeness and continuation remain explicit.
- Diagnostic detail and closure stress belong to separate cost lanes.
- Cancellation, denial, replacement, retry, continuation, reset, close, and
  shutdown dispose their governed resources exactly once.

## Inspection And Debugging

Correlate the Query transition or attempt identity, projection observation identity,
application generation, mounted frame/node identity, and presentation evidence.
Request compact evidence first and expand detail under an explicit disclosure,
retention, and byte budget. None of those reporting values can execute Query,
construct a fact, or publish.

## Anti-Patterns

- Importing a nonexistent workspace-extension convenience API.
- Copying Query result state into a UI cache or local loading/error enum.
- Selecting operational values by a literal field lookup after admission.
- Converting native values through JSON, debug text, or widened numbers.
- Reassembling authority from identities, digests, reports, or inspection.
- Querying from a renderer or host adapter.
- Importing raw `worth-query` from product code or ordinary UI services.
- Importing `worth-query-replay` from ordinary inspection or live binding.
- Treating collection continuation as scalar posture.
- Retrying or replacing without consuming the returned affine owner.

## Current Limits

The direct grammar supports declared scalar and keyed collection projections.
Rich tables, joins, authored expressions and formatting, and general
composition remain additive successor work. Intent payloads may consume
admitted projection observations; they do not replace this binding,
observation, or publication path.

The current `worth-ui-query-binding` manifest still contains a temporarily
admitted raw `worth-query` predecessor edge. That is implementation debt, not a
supported application API or destination topology. New work must not widen the
edge; the binding owner must remove the remaining raw-engine imports so
`worth-query-decl` and `worth-query-host` are its only ordinary Query audiences
before broader Query-facing capabilities are added.

## Related Docs

- [Application lifecycle](./application-lifecycle.md)
- [Worth UI architecture](./architecture.md)
- [Runtime subsystems](./runtime-subsystems.md)
- [Application inspection](./inspection.md)
- [Worth Query AI README](../../worth-query/crates/worth-query/docs/AI_README.md)
