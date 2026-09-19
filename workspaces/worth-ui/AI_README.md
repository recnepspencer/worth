# Worth UI AI Discovery

Worth UI is the product-facing UI platform boundary. Start with the named
`worth_ui::facade` audiences. Read lower crates only when maintaining the
implementation that owns that boundary.

Canonical reading order:

1. [Architecture](./docs/architecture.md)
2. [Authored composition](./docs/authored-composition.md)
3. [Interaction and intents](./docs/interaction-and-intents.md)
4. [Runtime services](./docs/runtime-services.md)
5. [Application lifecycle](./docs/application-lifecycle.md)
6. [Native host platform](./docs/native-host-platform.md)
7. [Application inspection](./docs/inspection.md)
8. [Runtime subsystem map](./docs/runtime-subsystems.md)
9. [Query-backed UI views](./docs/query-binding.md)
10. [Appearance and themes](./docs/appearance-and-themes.md)
11. [Milestone 3.10.1 migration](./docs/migration-3.10.1.md)

The longer contributor orientation remains in
[worth-ui-readme.md](./docs/worth-ui-readme.md).

## Ordinary Product Path

```text
WorthUiNativePlatform::prepare(profile)
-> UiPreparedNativePlatform
-> run(UiNativeApplicationDefinition)
-> UiNativeApplicationPreparation
-> complete()
-> WorthUiApp
-> UiPreparedNativeApplication
-> qualified native event loop
-> Closed | Stopped | ApplicationPreparationDenied
```

`WorthUiApp` is one prepared generation. Launch consumes it. The active session
keeps application, graph, planning, Query, host, mounted publication, and
inspection identities coherent.

The platform supplies the only native host binding. An unbound
`WorthUi::app()` cannot freeze, and a bound builder cannot bind a second host.
See [Native host platform](./docs/native-host-platform.md) for the compiled
preparation and execution progression.

Treat the returned mounted-frame outcome exhaustively. Do not import an
intermediate runtime phase to skip a denial or recover a raw executor.

For Query-backed content, discover from
`worth_ui::facade::query_binding`. Follow the concrete type progression:

```text
host installation plan
-> installed scalar or collection registration
-> Query-issued projection observation
-> ordinary projection rebind
-> shape-specific affine fact receipt
-> mounted semantic text and host publication
```

Availability, currency/activity, stop posture, compatibility, native value,
and collection continuation are separate typed axes. Reporting identities and
inspection projections explain this progression but cannot enter it.

For human actions, keep this progression equally explicit:

```text
loss-aware native observations
-> presentation-bound semantic interaction
-> typed route, payload, and operability proof
-> move-only UI admission
-> exact typed provider execution
-> separate product or Query admission
-> declared consequence through ordinary rebind and mounted publication
```

Start with [Interaction and intents](./docs/interaction-and-intents.md). Native
input is not intent authority, UI admission is not domain admission, and a
diagnostic or visible posture cannot be promoted into either.

For runtime services, follow the family-specific owner after the exact request
origin. Explicit `OpenPortal`, `ClosePortal`, and `InvokeCommand` operations
cross intent admission. Window focus, scroll delta, Tick, rebind, dismissal,
restoration, and continuation do not pretend to be intents. Cross-family work
passes through the non-publishing proposal compiler, then the existing
publication and host-settlement owners. See
[Runtime services](./docs/runtime-services.md).

For appearance, application authors declare component-scoped roles whose
finite cells cover supported aspects and state partitions. A typed slot catalog
defines value kinds; complete theme definitions supply values; an active theme
binding is surface-scoped. The runtime resolves those inputs with current
UI-owned state and concrete mounted occurrence geometry, then composes accepted
Motion samples and Portal-issued overlay relations. Presentation consumes the
completed prepared phase, and acceptance commits the same owner succession used
to derive its pixels. Start with
[Appearance and themes](./docs/appearance-and-themes.md).

## Authority Boundaries

- `worth-ui-dsl` owns authored syntax, source structure, language diagnostics,
  normalization, and the sealed semantic package.
- `facade::source` owns file transport, watcher settlement, and runtime ingress.
- `worth-ui-runtime` owns active application, graph, planning, execution,
  interaction and intent admission, mounting, publication, host exchange, and
  operational inspection state.
- `facade::query_binding` and `worth-ui-query-binding` are the only
  Query-to-UI product route.
- `worth-ui-host-contract`, the native host, and the headless recorder own
  mechanics, not UI meaning.
- `worth-ui-native-platform` owns native application admission and the affine
  platform-binding grant; product code cannot extract or replace that grant.
- `facade::inspection` exposes read-only queries and receipts. It cannot mutate
  or reconstruct operational state.

IDs, digests, reports, and inspection receipts explain authority. They do not
grant it.

When investigating Query-backed output, start with
[Query-backed UI views](./docs/query-binding.md), then inspect the public
facade, the binding owner, the runtime projection-rebind entry, and only then
the Query substrate. Do not infer a workspace extension, literal field lookup,
or renderer-side query lane from older names.

## Authored Inputs

Use `WorthUiApplicationBuilder::with_rust_authored_input(...)` for typed Rust
composition. Use the production filesystem provider/watcher plus DSL compiler
to obtain one complete `WorthUiWatchedCandidateSubmission`, then pass it to
`with_candidate_submission(...)`.

Do not serialize typed authoring through JSON, split a candidate into loose
declarations, or parse source on a frame.

## Runtime Placement

Place runtime work using [Runtime subsystems](./docs/runtime-subsystems.md).
The platform owners are application, graph, planning, mounting, observation,
inspection, the thin session composition root, and the six sibling runtime-
service owners: portal, focus, motion, command routing, scroll, and selection.
Service proposal compilation coordinates those owners but cannot publish,
issue host effects, or become another service owner. See the subsystem map
before placing state or authority; a generic `Service` bag is not a lawful
destination.

## Verification

Before claiming a WORTH UI change is complete, run the focused evidence and the
repository-owned topology, compiler-contract, strict Clippy, formatting,
WORTH UI line-cap, boundary, and agent-context checks. A behavioral pass does
not override a red ownership or reachability gate.
