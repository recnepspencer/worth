# Worth UI Vision

## Thesis

Worth needs a first-class desktop application platform, not a collection of
widgets around a renderer.

`worth-ui` is the Rust-native desktop platform layer that turns an event-driven
Worth-owned native host and canonical runtime meaning into shippable, polished,
inspectable, reactive, cross-platform desktop applications. It is not an
Electron clone, not a web runtime in a native wrapper, and not a thin skin over
a renderer. It is the app platform that supplies the shell, command system,
layout, professional widgets, native operating-system integration,
accessibility, state binding, preview workflows, delivery tooling, and runtime
semantics that serious desktop software needs.

The ambition is direct:

- people should prefer building serious desktop applications with Worth UI over
  Electron
- desktop developers should get the immediacy and portability web developers
  enjoy without inheriting the web runtime's weight and fragmentation
- web developers should look at Worth UI and feel jealous of how coherent,
  typed, native, reactive, and inspectable desktop app development can be

The Worth native host owns the pixel, input-observation, and event loop. Worth
UI is the application platform. Worth Query is the application-facing
composition authority over the domain and lower runtimes that own truth.
Together they should make desktop apps feel lighter than Electron, more
coherent than web frontends, more ergonomic than traditional native UI, and
more truthful than ad hoc application state.

## What This Platform Is For

`worth-ui` exists for product surfaces where users live inside the application:
tools, editors, consoles, workbenches, modeling environments, inspectors,
simulators, data applications, AI-native editing systems, and operational
software where desktop quality matters.

It is meant to support:

- developer tools, IDE-like workbenches, graph editors, debugging consoles, and
  runtime inspectors that need dockable panels, live tables, command palettes,
  rich diagnostics, and high-density professional layouts
- geometry, CAD, topology, chip-design, and simulation software that needs
  branch-aware previews, exact selection models, canvas tooling, inspectors,
  historical diffs, and confidence before mutation
- AI-native editing environments that need speculative branches, preview
  panels, recovery actions, causal explanations, and world-state inspection as
  ordinary UI affordances
- operational and data applications that need sortable tables, live metrics,
  policy-aware projection, resilient background tasks, command workflows, and
  durable user settings without adopting a web stack
- internal business applications where teams want the velocity of web
  development but the feel, performance, and deployment model of native
  desktop software
- plugin-driven applications where commands, panels, file types, inspectors,
  settings, and background services can be contributed without dissolving the
  app into local glue

The technical thesis is the same across all of them:

- UI surfaces should bind to declared runtime meaning, not local cache folklore
- commands should be canonical application actions, not scattered callbacks
- desktop shell behavior should be solved once and reused everywhere
- professional widgets should carry data, focus, accessibility, theme, and
  testing semantics together
- high-frequency canvas, HUD, game, and simulation surfaces must be able to run
  through frame-efficient execution lanes without giving up semantic binding or
  hot reload
- preview, branch, history, inspection, and recovery should be native product
  patterns
- delivery, updating, crash recovery, and platform integration are part of the
  framework, not afterthoughts

## Why This Platform Is Different

These are the strategic differences that make `worth-ui` more than another UI
crate:

- event-driven native presentation with platform-grade retained application
  infrastructure above host mechanics
- first-class application shell: windows, menus, tabs, panels, docking,
  shortcuts, status, lifecycle, and persisted workspace layouts
- command registry as the shared action spine for menus, toolbars, shortcuts,
  command palettes, context menus, automation, future owner-admitted undo/redo
  projection, telemetry, and runtime intent
- Query-bound surfaces where tables, trees, inspectors, timelines, canvases,
  and dashboards are backed by canonical declared read meaning
- hot-lowered UI composition where source files in the codebase lower into
  canonical runtime artifacts and host-neutral execution plans without
  recompiling Rust
- specialized execution lanes for desktop widgets, virtualized data surfaces,
  spatial canvases, real-time HUDs, and shader/material-backed overlays
- live view binding through Query-published projections and Query-owned
  lifecycle instead of widget-owned event subscriptions or direct Bridge,
  Relational, or Signal access
- intent-aware buttons and workflows whose enabled, denied, advisory, stopped,
  recoverable, and completed states come from structured runtime posture
- preview and branch UX as ordinary desktop affordances, not special-case
  feature code
- built-in professional widgets for the workbench class of applications:
  virtualized tables, trees, inspectors, split panes, dock areas, timelines,
  logs, consoles, property editors, graph/canvas tools, markdown/rich text,
  and forms
- native OS integration as a platform contract: menus, tray, clipboard, drag
  and drop, file associations, deep links, notifications, keychain, installers,
  auto-update, crash reporting, and single-instance behavior
- accessibility and keyboard navigation designed into shell, command, layout,
  and component systems from the start
- developer tooling for component galleries, theme editing, layout debugging,
  command inspection, accessibility inspection, live query inspection, task
  monitoring, screenshot testing, packaging, and release

The important breakthrough is not "a renderer with more widgets." The
breakthrough is a desktop platform where every visible surface can be traced
back to an application command, a declared query, a runtime artifact, a native
platform contract, or an explicit piece of app state.

## Mission

`worth-ui` exists to make serious Rust desktop applications pleasant to build,
beautiful to use, and trustworthy to operate.

It must answer these questions as native platform responsibilities:

- How does a developer define a full desktop app shell without rebuilding
  windows, menus, docking, commands, shortcuts, settings, and lifecycle from
  scratch?
- How does a table, tree, inspector, timeline, or canvas declare what truth it
  needs and stay live without local cache and subscription code?
- How does a button know whether an action is ready, denied, advisory,
  recoverable, or unsafe before the user clicks it?
- How do preview, branch, history, diff, and merge workflows become normal UI
  patterns rather than bespoke screens?
- How do users inspect why a value is present, why a command is disabled, what
  changed, what recomputed, and how to recover?
- How do desktop apps ship with native menus, installers, auto-update, crash
  recovery, file associations, and platform integrations without each team
  rebuilding distribution infrastructure?
- How do platform components carry design tokens, density, focus,
  accessibility, keyboard behavior, and test hooks consistently?
- How do plugins contribute commands, panels, settings, file handlers,
  inspectors, and background services through stable app extension points?
- How do developers get the fast iteration of immediate-mode UI without losing
  the durable structure expected from platform-grade applications?

If Worth UI is weak, teams will keep choosing Electron because Electron gives
them a whole app ecosystem even when the runtime is heavy. Worth UI must win by
giving them the whole app ecosystem with Rust-native performance, runtime
truthfulness, and better desktop ergonomics.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-ui-host-contract` | stable presentation and observation boundary | host-neutral physical work, presentation, input, capture, readiness, and cleanup contracts |
| `worth-ui-host-native` and `worth-ui-host-headless` | host mechanics | native or deterministic mechanical execution; never UI meaning |
| `worth-ui` | desktop application platform | UI declarations, shell, commands, layout, widgets, design system, accessibility, UI services, and presentation meaning |
| `worth-query-decl` and `worth-query-host` | Query audience boundaries | application declaration and host installation, admission, execution, and publication entry |
| `worth-query` | application-facing composition authority | installed meaning, authority composition, typed progression, execution products, live maintenance, runtime-local recovery, and publication |
| `worth-runtime-bridge` | installed correspondence and lowering | lawful correspondence between Query and lower runtimes; not graph truth or application policy |
| `worth-relational` | authoritative graph truth runtime | entities, relations, aspects, versions, snapshots, transactions, and commit mechanics |
| `worth-signal` | scoped derived-computation runtime | local evaluation slots, readiness and scheduling, performed execution receipts, and condition outcomes |

### Ownership boundary

`worth-ui` owns:

- application shell and lifecycle
- window, tab, dock, panel, and workspace layout orchestration
- command registry and desktop action binding
- host-neutral component system and professional widget suite
- design tokens, themes, density, typography, focus, and visual states
- keyboard navigation and accessibility semantics at the UI layer
- UI binding to Query-published or Query-bound projections, outcomes, live
  lifecycle, and runtime posture through the Query audience boundary
- native desktop integration and cross-platform app affordances
- background task presentation and user-facing progress/cancellation surfaces
- settings, preferences, persisted layouts, recent files, and app-local user
  state
- screenshot, accessibility, layout, and component testing harnesses
- packaging, updater, release-channel, and crash-reporting integration points
- plugin contribution points for UI-facing app extensions

`worth-ui` does not own:

- authoritative truth semantics
- query planning or query legality
- relational mutation authority
- Signal-local scheduling, evaluation, or performed-computation truth
- Bridge correspondence and lowering internals
- domain-specific kernel operations
- network protocol delivery
- final OS implementation details where lower platform adapters own the native
  calls

### Structural rule

Worth UI renders, binds, orchestrates, and delivers user experience. It does not
invent truth, query, mutation, causality, or recovery semantics above the
runtime stack.

Product surfaces start from the UI, but their meaning should flow through
canonical commands, UI routing and operability, Query declarations and
installed operations where domain work is required, owner-issued runtime
artifacts, governed publication, and structured outcomes. UI admission never
substitutes for Query application-operation admission.

## Principles

1. The first screen of a Worth UI app should feel like a real product, not a
   demo harness.
2. Event-driven native mechanics and retained application infrastructure
   should remain complementary rather than competing authority paths.
3. Every user action should be expressible as a command.
4. Every serious data surface should be able to bind to declared Query meaning.
5. Live updates should be query-shaped, not raw event-shaped.
6. Runtime posture must not be flattened into booleans or generic loading
   states.
7. Preview, branch, history, diff, and recovery are desktop UX primitives.
8. Professional desktop widgets are core platform infrastructure, not optional
   examples.
9. Native OS integration is part of the product contract.
10. Accessibility, focus, and keyboard navigation are built into components and
    shell behavior from the beginning.
11. Layout is a durable workspace artifact, not transient widget placement.
12. Design tokens and density modes must make apps feel coherent without
    preventing deep customization.
13. Developer tooling is part of the framework's quality, not secondary polish.
14. Packaging, updating, crash recovery, and release channels belong in the
    platform story.
15. Escape hatches are necessary, but the ordinary path should be coherent,
    typed, and pleasant.
16. The platform should make the easiest path also the most runtime-honest path.

## Foundational Decisions

These are locked architectural decisions:

- the Worth-owned native host is the sole native-display platform
- Worth UI meaning remains above host contracts and native mechanics
- Rust defines platform capabilities; hot-reloadable UI source composes those
  capabilities
- UI source lowers into a canonical runtime artifact before mounted and native
  presentation
- the native event path consumes owner-issued presentation work rather than
  parsing, validating, or resolving UI source during presentation
- failed UI reloads keep the last valid plan active and surface diagnostics
  without disturbing the running app
- commands are first-class registry entries with stable identifiers
- UI views consume Query-published or Query-bound projection products through
  the installed audience-facade path rather than host-local data adapters or a
  raw Query-engine dependency
- runtime outcomes, denials, advisories, stops, recovery briefs, receipts, and
  inspection artifacts remain structured through the UI boundary
- professional app shell primitives ship before broad ornamental component
  breadth
- docking, split panes, panel state, window state, and tab state are persisted
  workspace concepts
- tables, trees, inspectors, and lists are virtualized and live-ready by
  default
- high-frequency canvas, HUD, game, and simulation surfaces use specialized
  frame execution plans instead of ordinary widget interpretation
- UI frame-cost counters are product-visible enough to certify hot paths rather
  than relying on vague performance claims
- design tokens are semantic rather than raw color piles
- accessibility metadata is carried by platform components
- background tasks expose progress, cancellation, and diagnostics through
  platform surfaces
- packaging and release flows are treated as product capabilities
- plugins contribute through typed extension points, not arbitrary global UI
  mutation
- native platform adapters remain explicit and testable per operating system
- UI testing must include screenshot, accessibility, focus, and interaction
  behavior, not only unit-level widget checks

## How This Vision Drives Engineering

This document is intentionally written so a roadmap can be derived from it.

The derivation rule is:

- each capability pillar below implies concrete platform surfaces that must
  exist
- each technical role implies constraints that implementation must preserve
- each "what this enables" section implies real product use cases the platform
  must serve, not marketing examples
- if a capability is named here but not yet fully present in code, it belongs
  on the roadmap as remaining engineering work
- if a capability is present in code but not yet proven under desktop, runtime,
  accessibility, packaging, and failure scenarios, it belongs on the roadmap as
  certification work

In other words:

- the vision says what the desktop platform must be able to do
- the roadmap says what still must be engineered
- the test requirements should say what must be proven before the capability is
  trusted

## Capability Pillars

### Hot-Lowered UI Iteration Architecture

#### Codebase-authored UI source

Technical role:
Worth UI must let developers author interface composition in repo-local source
files that live beside the rest of the application. The source should be
human-readable, diffable, searchable, reviewable, and editable in ordinary code
editors. It should describe app shell composition, panels, menus, toolbars,
tables, inspectors, forms, command placement, theme tokens, layout defaults,
and bindings to registered platform artifacts.

What this enables:

- developers keep UI design inside the codebase instead of moving product
  structure into an external visual tool
- layout, labels, spacing, density, table columns, inspector sections, command
  placement, and theme values can change without a Rust rebuild
- product and interaction iteration can happen at visual speed while compiled
  Rust remains responsible for durable behavior
- UI source gets normal repository benefits: version control, code review,
  formatting, search, and branch comparison

#### Rust-native composition escape hatch

Technical role:
Worth UI should also expose a pure Rust composition path for teams that want
native Rust authoring, macro-backed composition, generated UI structures, or
advanced cases where an external source format would add friction. This path
must produce the same canonical UI source or artifact meaning as file-authored
UI, pass through the same capability registries, and lower into the same
runtime plans.

This is an escape hatch for UI structure, not arbitrary Rust-code hot patching.
New behavior, custom widgets, native integrations, Query declarations, and
performance-sensitive rendering still belong in compiled Rust capability
registration. The Rust-authored composition path is hot-reload-compatible only
where it produces replaceable artifact input for the running Worth runtime.

What this enables:

- Rust-first teams can stay entirely inside Rust without giving up canonical
  artifacts, inspection, diagnostics, stable identity, or plan swaps
- generated UI and advanced app shells can use ordinary Rust tooling while
  remaining visible to Worth UI as platform-owned artifacts
- the easiest path for serious apps is the Worth runtime path, where UI
  composition, Query bindings, shell state, diagnostics, and plan swaps share
  one coherent host
- external or adapter-hosted usage can remain possible without becoming the
  primary development story or weakening runtime-owned hot reload

#### Capability registries

Technical role:
Compiled Rust should register the capabilities that hot-reloadable UI source is
allowed to compose: components, commands, Query views, settings, icons, theme
tokens, panels, file handlers, plugin slots, and data shapes.

What this enables:

- the UI source references known platform artifacts instead of arbitrary
  strings or executable scripts
- invalid component names, command IDs, query bindings, missing props, illegal
  table columns, and missing theme tokens can be rejected mechanically
- Rust remains the language for new behavior, custom widgets, domain logic,
  Query declarations, native integrations, and performance-sensitive rendering
- the platform can expose rich diagnostics when source no longer matches the
  compiled app capability set

#### Canonical UI lowering

Technical role:
Worth UI should parse, validate, and lower UI source into one canonical runtime
artifact before rendering. That artifact carries stable IDs, component
references, command bindings, Query/view bindings, accessibility metadata,
theme-token references, layout intent, plugin contribution references, and
diagnostics.

Both file-authored UI and Rust-authored composition must converge here. The
runtime should not need to know whether artifact input came from a `.worth-ui`
source file, a Rust builder, generated Rust, or a macro expansion once lowering
has produced canonical meaning.

What this enables:

- every UI surface can be inspected as a real platform artifact rather than
  reverse-engineered from Rust control flow
- hot reload can validate source off the frame path and swap only valid lowered
  plans into the running app
- failed reloads preserve the last valid UI while showing actionable errors
- UI testing and tooling can inspect the same canonical artifact that runtime
  rendering consumes

#### Host-neutral execution plans

Technical role:
The lowered artifact should compile into frame-efficient, host-neutral execution
plans. The active application session owns the executable plan generation; the
native host consumes owner-issued presentation work without choosing UI meaning
or plan strategy. The event loop consumes compact handles, pre-resolved child
ranges, interned IDs, command handles, component handles, style-token handles,
and specialized plans for tables, trees, inspectors, dock areas, and canvas
surfaces.

Plan equivalence must cover complete executable meaning and remain distinct
from activation freshness. Equal hashes or visually identical output cannot
hide a changed Query binding, host-support contract, lane policy, resource, or
other execution-bearing fact. Equivalent replacements should produce a typed
no-op; non-equivalent plans publish atomically with application authority.

What this enables:

- hot reload does not become per-frame source interpretation
- ordinary UI reloads can feel instant because parsing, validation, and
  lowering happen before the next plan swap
- steady-state rendering remains within the qualified native-host cost
- large surfaces can use virtualization, cached text layout, and specialized
  execution plans instead of generic tree walking where needed

#### Stable identity and state reconciliation

Technical role:
Every lowered UI node that owns durable interaction state should have a stable
identity. Hot reload must reconcile old and new artifacts by identity so focus,
scroll position, selection, text input state, panel visibility, splitter
positions, tab state, table column widths, Query projection attachments, and
component-local state can survive source changes when their owning lifecycle
admits preservation.

What this enables:

- editing a layout file does not reset the running app
- developers can iterate on UI while preserving the exact state they were
  inspecting
- plugin and app surfaces can evolve without losing workspace continuity
- identity changes become explicit replacement events rather than accidental
  state loss

#### Hot reload as plan swap

Technical role:
Hot reload should be implemented as a file-watch, debounce, parse, validate,
lower, plan, and atomic-swap pipeline. Reload work happens off the ordinary
frame path where possible. A valid new execution plan replaces the old one only
at a safe frame boundary; invalid reloads keep the previous plan active.

The primary host for this pipeline is the running Worth runtime. The runtime
owns the watched artifact inputs, active artifact, stable identity map,
reconciliation state, diagnostics surface, and execution-plan swap boundary.
This is what makes hot reload feel like product iteration instead of a build
tool wrapped around a renderer.

What this enables:

- common UI changes appear in the running app within tens of milliseconds
- developers get web-like iteration without turning Worth UI into a web runtime
- reload diagnostics can appear inside the app without breaking interaction
- UI authoring can remain fast even when the Rust workspace is large

#### Hot and cold development lanes

Technical role:
Worth UI must separate high-churn composition from low-churn compiled behavior.
The hot lane is runtime-reloadable UI composition. The cold lane is compiled
Rust behavior. The hot lane may be authored in a dedicated UI source format or
through Rust APIs that emit the same canonical artifact input.

Hot lane:

- layout hierarchy
- panels and dock composition
- menus, toolbars, and command placement
- labels and copy
- spacing, density, and theme tokens
- table columns and inspector sections
- forms and validation presentation
- component props and variants
- simple visibility and binding expressions
- Rust-authored composition that produces replaceable canonical artifact input

Cold lane:

- new commands
- new Query declarations
- new custom widgets
- new native integrations
- new domain behavior
- new data types
- performance-sensitive rendering kernels
- platform adapter internals

What this enables:

- Worth UI avoids using Rust compilation for the parts of UI work where compile
  time is pure iteration friction
- compiled Rust remains the authority for behavior and capability definition
- the Rust escape hatch strengthens the platform path instead of creating a
  separate UI runtime
- teams can tune visual and product structure quickly without weakening the
  platform's type and runtime discipline
- granular crate topology still matters, but high-frequency UI shaping no
  longer depends on rebuild speed

### Frame-Efficient Execution And Real-Time Surfaces

#### Performance-stratified execution lanes

Technical role:
Worth UI should share one semantic platform while lowering different surface
classes into specialized execution lanes. Desktop widgets, virtualized data
surfaces, spatial canvases, real-time HUDs, shader/material overlays, and
custom render surfaces should not all pay the same frame mechanics.

What this enables:

- ordinary app UI can stay rich, accessible, command-bound, and Query-aware
- tables and lists can use virtualization, collection patches, and cached text
  plans
- canvases can use spatial indexes, overlay plans, selection layers, and
  direct render-surface integration
- HUDs and game overlays can use prebuilt geometry, dynamic uniforms,
  material/shader bindings, text atlases, and explicit render-pass ordering
- a single app can mix a dense desktop shell, live data panels, editor canvases,
  and real-time overlays without forcing one execution model onto all of them

#### 240 FPS adversarial frame budget

Technical role:
The platform should be designed against the hostile case where a high-resolution
game, simulation, or editor surface must run at 240 FPS while UI overlays,
inspectors, diagnostics, and hot-reloadable HUD elements remain active.

What this enables:

- Worth UI avoids architectures that are pleasant for settings panels but
  collapse under high-frequency interactive surfaces
- hot reload remains compatible with game-grade rendering because reload work
  happens at lowering and plan-swap boundaries, not inside the steady-state
  frame loop
- frame execution can be certified against explicit costs instead of relying on
  "it feels fast" claims
- users can build products that cross the line between desktop tool, game
  engine editor, simulation console, and real-time visualization

#### No per-frame source interpretation

Technical role:
The steady-state frame loop must not parse, validate, resolve registry strings,
or interpret hot UI source. It consumes execution plans produced by the lowering
pipeline. Source reload may invalidate and rebuild plans, but it must not add
ordinary per-frame interpretation cost.

What this enables:

- hot reload and 240 FPS execution can coexist
- reload failures do not disturb the active frame plan
- the runtime can keep expensive validation, diagnostics, and artifact
  reconciliation off the hot path
- app authors can get instant-feeling visual iteration without poisoning
  steady-state performance

#### Bounded frame-cost instrumentation

Technical role:
Worth UI should expose named frame-cost counters for hot paths: nodes visited,
layout regions recomputed, hit-test regions checked, text layouts shaped, glyph
uploads, allocations, draw batches, render passes, material changes, clipped
regions, and virtualized rows touched.

What this enables:

- performance claims can become testable contracts
- regressions such as accidental full-tree walks, per-frame allocations, or
  broad text reshaping can fail mechanically
- developers can see why a surface is expensive without guessing from elapsed
  time alone
- roadmap certification can distinguish verified frame paths from intentional
  performance debt

#### Real-time HUD and shader-backed overlays

Technical role:
HUDs and overlays should lower into render-ready plans with stable IDs,
anchors, safe areas, text atlas entries, material bindings, shader parameters,
animation curves, dynamic state bindings, and render-pass dependencies.

What this enables:

- a glass-themed HUD can sample the scene buffer, run blur/refraction materials,
  and composite UI without forcing the scene into the widget system
- rapidly changing camera, lighting, and background pixels do not require
  structural UI relayout
- game state values update through compact dynamic buffers or bindings while
  geometry, layout, and materials stay cached
- HUDs can remain hot-reloadable as UI assets while the renderer owns the
  volatile world scene

#### Canvas and editor render surfaces

Technical role:
Canvas-like surfaces should have a first-class lane for direct render passes,
object layers, overlays, hit testing, selection, snapping, world/screen
projection, and command integration.

What this enables:

- geometry, topology, node-editor, simulation, game-tool, and visual-debugging
  surfaces can stay performant without abandoning the Worth UI platform
- selection and tool state can integrate with commands, Query-bound object
  layers, inspection, and preview branches
- custom renderers can participate in shell layout and platform diagnostics
  without being squeezed through ordinary widgets

#### Semantic integration without renderer ownership

Technical role:
Worth UI should not attempt to own every pixel of a real-time scene. The game,
simulation, or custom renderer owns volatile world rendering. Worth UI owns the
semantic UI layers, overlays, commands, bindings, accessibility/inspection
metadata where applicable, and composition with the shell.

What this enables:

- a scene may change across most pixels every frame while UI structure remains
  stable and cheap
- Worth UI can integrate with renderers instead of replacing them
- real-time products keep native UI affordances, hot reload, and runtime
  explainability without giving up their rendering architecture

### Application Shell

#### Multi-window application model

Technical role:
The platform must own the ordinary desktop application shell: app lifecycle,
windows, native menus, command routing, focus, active document, modal surfaces,
and shutdown behavior.

What this enables:

- apps can open multiple documents, tools, and inspectors without bespoke
  window plumbing
- commands can route to the active window, active tab, active selection, or
  global app context predictably
- lifecycle-sensitive features such as autosave, crash recovery, and background
  shutdown can be platform behavior

#### Workspace layout

Technical role:
Docking, tabs, split panes, sidebars, bottom panels, status bars, and saved
layouts must be first-class app structures.

What this enables:

- IDE-like and tool-like applications can ship immediately with professional
  workbench ergonomics
- users can customize and restore their workspace across sessions
- product teams can add panels and inspectors without hand-maintaining nested
  layout state

#### Menus, toolbars, command palette, and context menus

Technical role:
All action surfaces should be projections of the command registry.

What this enables:

- the same command appears consistently in the menu bar, toolbar, command
  palette, context menu, keyboard shortcut, and automation surface
- enabled/disabled/advisory state stays coherent across all affordances
- apps can expose power-user workflows without duplicating action code

### Command Architecture

#### Canonical command registry

Technical role:
Every user-visible action has a stable identifier, label, shortcut, icon,
availability predicate, execution handler, and optional runtime intent binding.

What this enables:

- menus, shortcuts, toolbars, command palette, context menus, and plugins share
  one action spine
- app behavior becomes searchable, inspectable, testable, and scriptable
- command availability can reflect runtime posture instead of local UI guesses

#### Intent-aware command execution

Technical role:
Commands that mutate runtime truth or trigger domain workflows first use Worth
UI routing, targeting, operability, and confirmation. They then enter the
installed Query application-operation path through `worth-query-host`, where
authentication, principal binding, capability, purpose, disclosure, graph
obligations, execution, commit, aftermath, and publication retain their own
typed progression. A UI dispatch receipt cannot authorize the Query operation.

What this enables:

- buttons and menu items can explain why they are unavailable
- risky actions can show advisory or violation posture before execution
- mutations can produce exact commit, aftermath, publication, recovery-support,
  and inspection posture without a UI callback pretending to own the effect
- command execution becomes part of the runtime story rather than a callback
  side effect

#### Undo, redo, and transaction presentation

Technical role:
The platform must eventually present owner-issued undoable and redoable command
history while respecting runtime transaction authority, committed aftermath,
and mutation evidence. Undo and redo are an aspirational product capability,
not a currently admitted ordinary Query contract. Until the owning runtime
admits them, Worth UI may present history and exact recoverability posture but
must not expose an actionable UI-local undo or redo authority.
Query's `provisional_aftermath` remains a compiled experiment and is not an
accepted undo/redo product contract.

What this enables:

- users see meaningful action names, affected scope, and recoverability
- once admitted by the owning runtime, undo/redo can participate in branch,
  preview, and transaction-scoped workflows
- applications avoid shallow UI-only undo stacks that drift from truth

### Query-Bound Surfaces

#### Tables and data grids

Technical role:
The data grid should bind to collection queries with typed columns, ordering,
opaque cursors, virtualized rows, live collection patches, selection, editing,
aggregation, and policy-aware projection.

What this enables:

- operational apps can ship serious tables without building a data engine per
  feature
- large collections remain responsive
- sorting, filtering, pagination, live updates, and selection stay tied to
  canonical query meaning
- policy-masked aspects never become hidden columns accidentally read by UI

#### Trees and graph navigation

Technical role:
Tree views and graph browsers should bind to bounded traversal, relationship
proofs, grouped neighborhoods, lineage, and structural correspondence where
available.

What this enables:

- project explorers, topology browsers, dependency trees, and object graphs can
  be live and query-shaped
- expansion state and selection can survive identity evolution where the
  runtime can prove continuity
- tree updates can be precise instead of rebuilding entire hierarchies

#### Inspectors and property editors

Technical role:
Inspectors should bind to detail and inspector view shapes with aspect
projection, editable fields, validation, runtime posture, and mutation receipts.

What this enables:

- selected objects can show exactly the aspects the current user and context may
  inspect
- edits can be admitted, previewed, rejected, or committed with structured
  evidence
- property panels become runtime-aware rather than local form state islands

#### Timelines, diffs, and history views

Technical role:
History-oriented UI should bind to historical basis, diff, lineage,
correspondence, branch, and causal inspection surfaces.

What this enables:

- users can see what changed, when, why, and what prior identity became
- branch comparison and review screens can be standard platform patterns
- AI and operator workflows can inspect speculative or historical state without
  app-local history plumbing

#### Canvas and spatial work surfaces

Technical role:
Canvas-like views should integrate selection, hit-testing, overlays, tools,
pan/zoom, snapping, command routing, query-backed object layers, and preview
rendering.

What this enables:

- geometry, topology, workflow, node-editor, and simulation apps get a shared
  interactive work surface substrate
- visual tools can bind object layers to declared runtime surfaces
- branch-local or speculative edits can render as preview layers before commit

### Runtime-Aware UX

#### Live view binding

Technical role:
Worth UI should promote one-shot Query-backed surfaces into live views without
forcing widgets to consume raw change events.

What this enables:

- "read once" and "stay updated" use the same declared view meaning
- live delivery is query-shaped: row patches, property patches, group movement,
  timeline changes, and inspector updates
- widgets can remain focused on presentation while the runtime owns legality,
  invalidation, and delivery posture

#### Structured outcome rendering

Technical role:
The UI platform should render runtime outcomes as first-class UX states without
replacing the owner-issued outcome type. Presentation may group ready, pending,
denied, advisory, violation, stopped, recoverable, committed, completed,
unsupported, deferred, stale, partial, indeterminate, and failed posture, but
must retain distinctions that change legal next actions, effects, disclosure,
inspection, or resource release.

What this enables:

- users receive specific reasons and recovery actions instead of vague error
  messages
- developers avoid inventing one-off status enums around every runtime call
- product surfaces can expose support posture honestly when a feature is
  visible but not admitted

#### Inspection and explanation

Technical role:
Worth UI should provide reusable inspection surfaces for retained handles,
query artifacts, command readiness, mutation evidence, projection consumption,
and cross-runtime causal explanations.

What this enables:

- users and developers can ask "why is this here?", "why did this change?",
  "why is this disabled?", and "what happens if I accept this?"
- debugging panels become product features rather than hidden logs
- complex applications become trustworthy because explanations are runtime
  artifacts, not narrative guesses

#### Preview and branch workflows

Technical role:
The platform should make speculative branches, preview state, mutation
planning, before/after comparison, merge inspection, and accept/discard flows
ordinary UI primitives.

What this enables:

- users can explore consequences before committing truth
- AI-assisted edits can be reviewed, explained, diffed, and accepted with
  confidence
- domain tools can show ghosted, staged, or branch-local state without mixing it
  into authoritative UI state

### Component And Design System

#### Semantic design tokens

Technical role:
Themes should be built from semantic tokens for surfaces, text, borders,
accent, danger, warning, success, selection, focus, overlays, charts, and
runtime states.

What this enables:

- apps look coherent across widgets, panels, and plugins
- light, dark, high-contrast, and custom themes can preserve meaning
- runtime posture can have consistent visual language across the platform

#### Density and professional layout

Technical role:
Worth UI should support comfortable, compact, and dense modes without breaking
alignment, focus, hit targets, or text fit.

What this enables:

- data-heavy tools can show real information density
- apps can serve both casual and professional users
- web developers notice desktop UI can be compact without becoming chaotic

#### Professional widget suite

Technical role:
The platform must ship high-quality widgets for desktop work:

- virtualized table/data grid
- virtualized list
- tree view
- property inspector
- dock area
- split pane
- tab bar
- command palette
- toolbar and icon button system
- breadcrumb
- searchable select
- form fields and validation
- timeline
- log viewer
- console/terminal surface
- markdown/rich text viewer
- code editor integration seam
- graph/canvas primitives
- notifications/toasts
- progress and task views
- file/project browser

What this enables:

- teams can build complete desktop applications without spending years on
  basic infrastructure
- components carry consistent focus, keyboard, accessibility, theming, and test
  semantics
- the platform competes with Electron ecosystems at the application level, not
  only the rendering level

### Native Platform Integration

#### Operating-system affordances

Technical role:
Worth UI must provide explicit adapters for native platform behavior across
Windows, macOS, and Linux.

What this enables:

- apps get native menus, window controls, file dialogs, tray icons,
  notifications, clipboard formats, drag and drop, and platform theme detection
- users experience the app as real desktop software, not a web page in a box
- product teams do not rebuild native edge behavior per application

#### App identity and file integration

Technical role:
The platform should own app identity surfaces such as file associations, URL
handlers, recent documents, project opening, single-instance behavior, and app
metadata.

What this enables:

- users can open Worth UI apps from files, links, recents, and OS launchers
- project/document workflows feel native
- enterprise and internal deployments can manage app identity predictably

#### Security and credentials

Technical role:
Credential storage, permissions, secure clipboard behavior, trusted plugin
boundaries, and native keychain integration must be platform capabilities.

What this enables:

- apps can handle tokens and secrets without local storage folklore
- plugin and automation features can be permissioned instead of all-powerful
- serious internal and commercial software can adopt the platform safely

### Background Work And Operations

#### Task runtime presentation

Technical role:
Worth UI should expose background work through task lists, progress bars,
cancellation, logs, retry, completion, and diagnostics.

What this enables:

- indexing, imports, exports, analysis, AI jobs, recomputation, sync, and
  packaging operations become visible and controllable
- long-running desktop work feels professional rather than frozen or mysterious
- task state can connect to status bars, notifications, panels, and commands

#### Error and diagnostic surfaces

Technical role:
Errors should carry structured categories, runtime evidence, user-facing
messages, developer diagnostics, recovery affordances, and optional reports.

What this enables:

- users understand what happened and what they can do next
- developers can inspect failures without scraping logs
- crash recovery and runtime recovery share a coherent presentation layer

#### Logging and observability UI

Technical role:
The platform should provide app-local log viewers, event inspectors, command
traces, query traces, task traces, and performance panels.

What this enables:

- developer tools can expose their own internals without bespoke admin UI
- support teams can debug customer issues with structured artifacts
- runtime explainability becomes visible in the desktop product

### Persistence And Settings

#### User, workspace, and project settings

Technical role:
Settings should be typed, scoped, persisted, migrated, and automatically
bindable to UI controls.

What this enables:

- app preferences, project preferences, plugin preferences, and user settings
  can coexist without local config sprawl
- settings panels can be generated or composed from typed settings definitions
- migrations and defaults remain explicit across releases

#### Layout and session persistence

Technical role:
Window positions, dock layouts, open tabs, recent files, selection context,
panel visibility, and task state should be persisted where appropriate.

What this enables:

- users return to the same workspace they left
- crash recovery can restore meaningful UI state
- professional apps feel durable rather than ephemeral

#### Document and project workflows

Technical role:
The platform should support project manifests, document tabs, dirty tracking,
autosave, recovery snapshots, conflict handling, and recent project workflows.

What this enables:

- editor-like and project-like applications get standard user expectations out
  of the box
- save, revert, recover, compare, and publish flows can be command-backed and
  runtime-aware
- teams avoid reimplementing document state for every app

### Accessibility And Interaction Quality

#### Accessible component semantics

Technical role:
Components must carry roles, names, descriptions, focus behavior, keyboard
navigation, state, and high-contrast semantics.

What this enables:

- screen-reader support and keyboard-only use can be certified component by
  component
- accessibility does not depend on every app author remembering hidden rules
- app testing can inspect accessibility structure as a first-class artifact

#### Keyboard-first desktop ergonomics

Technical role:
Worth UI should make keyboard navigation, shortcuts, focus traversal, command
palette, quick open, searchable selects, and typeahead natural.

What this enables:

- professional users can move quickly without pointer dependence
- dense desktop workflows feel better than equivalent web apps
- commands become discoverable and teachable through the platform

#### Motion, scaling, and comfort

Technical role:
The design system should honor text scaling, reduced motion, focus visibility,
contrast, and pointer target constraints.

What this enables:

- apps remain usable across display densities, accessibility settings, and user
  preferences
- polish is not traded against usability
- enterprise and public-facing apps can adopt the platform with confidence

### Developer Experience

#### App authoring ergonomics

Technical role:
The ordinary app authoring path should be small, typed, and composable:
register commands, define shell layout, bind panels to state or Query surfaces,
and run.

What this enables:

- small apps start quickly without ceremony
- large apps grow into shell, command, Query, plugin, and delivery systems
  without rewriting foundations
- Rust developers get a productive desktop path that does not feel like
  fighting the framework

#### Examples and templates

Technical role:
Worth UI should ship templates for common product classes:

- document editor
- data application
- workbench/IDE
- graph editor
- project explorer
- runtime inspector
- dashboard
- plugin host

What this enables:

- teams can start from a working product shape
- roadmap gaps become visible through realistic examples
- platform ergonomics can be judged by complete apps, not isolated widgets

#### Tooling

Technical role:
Developer tools should include:

- component gallery
- theme editor
- layout debugger
- command registry inspector
- shortcut conflict inspector
- accessibility inspector
- live Query/view inspector
- task monitor
- performance profiler
- screenshot test harness
- packaging CLI
- release checklist tooling

What this enables:

- developers can understand and debug their applications visually
- framework quality improves through self-hosted introspection
- Worth UI feels like an ecosystem rather than a library folder

### Delivery And Operations

#### Packaging and installers

Technical role:
Worth UI should provide a standard path to build installers and app bundles for
Windows, macOS, and Linux.

What this enables:

- teams can ship desktop apps without assembling release infrastructure from
  scratch
- examples and templates can become distributable products
- adoption does not stall at "the app runs locally"

#### Auto-update and release channels

Technical role:
The platform should support stable, beta, nightly, and internal release
channels, update checks, update application, rollback posture, and update
diagnostics.

What this enables:

- commercial and internal apps can maintain users safely
- teams can dogfood new features without disrupting stable users
- desktop delivery can compete with the web's deployment convenience

#### Crash reporting and recovery

Technical role:
Crash capture, local recovery, optional reporting, and session restore should
be platform capabilities.

What this enables:

- users lose less work
- developers receive actionable failure artifacts
- runtime recovery and UI recovery can share structured explanation surfaces

### Extension And Plugin Architecture

#### Typed contribution points

Technical role:
Plugins should contribute through explicit typed extension points:

- commands
- menus and command palette entries
- panels
- inspectors
- file handlers
- settings
- background services
- query views
- themes
- toolbars
- project templates

What this enables:

- applications can grow into ecosystems without surrendering structure
- plugins participate in command, permission, theme, accessibility, and
  testing systems
- host apps can inspect what a plugin contributed and why

#### Capability and permission model

Technical role:
Plugins and automation should declare capabilities for filesystem, network,
credentials, commands, panels, runtime mutation, and project access.

What this enables:

- extension systems can be powerful without becoming unsafe by default
- users and administrators can understand plugin authority
- enterprise deployments can allow or deny plugin families predictably

#### Runtime-aware extension hooks

Technical role:
Plugins that bind to runtime data should use Query declarations through
`worth-query-decl`, installed host progression through `worth-query-host`,
Worth UI command routing, and structured outcomes rather than raw Query or
lower-runtime access.

What this enables:

- extension-authored UI remains honest to the same runtime semantics as first
  party UI
- plugin data surfaces can be live, inspectable, policy-aware, and recoverable
- host apps avoid local pseudo-runtime layers for extension code

## Desired Outcomes

Worth UI succeeds when the following outcomes are true:

- building a serious desktop app in Rust feels faster and more coherent than
  starting an Electron application
- high-frequency UI composition changes feel instant because they hot reload
  through lowered runtime artifacts instead of Rust recompilation
- app authors can keep UI structure in the codebase while still getting
  web-like iteration speed
- high-frequency canvas, HUD, game, simulation, and visualization surfaces can
  hit explicit frame budgets through specialized execution lanes
- frame performance is observable through named counters rather than vague
  impressions
- the first-party app shell is good enough that teams stop writing their own
  menu, command, docking, settings, and panel infrastructure
- professional widgets are trusted for production data-heavy tools
- live UI surfaces are backed by declared Query meaning instead of local event
  plumbing
- command availability, UI routing, Query application-operation admission,
  runtime denials, advisory posture, recovery, and mutation evidence are
  visible as ordinary UX without collapsing their authority boundaries
- preview and branch workflows feel natural enough that users expect to inspect
  consequences before committing important changes
- app developers can explain why values changed, why recomputation happened,
  why commands are disabled, and what can be recovered
- packaging, updating, crash recovery, and native integration no longer decide
  teams against Rust desktop apps
- accessibility and keyboard-first interaction are platform defaults
- plugin ecosystems can grow without devolving into arbitrary global mutation
- web developers recognize that desktop can offer better density, lower
  latency, stronger typing, richer native integration, and deeper runtime
  explainability than typical web stacks
- end users experience Worth UI apps as polished native tools, not framework
  demos

## Domain Fit

### Workbenches And Developer Tools

`worth-ui` should support:

- dockable panels and persisted workspaces
- command palette and keyboard-first operation
- project explorers, logs, consoles, inspectors, and runtime diagnostics
- live Query-backed tables and trees
- plugin-contributed commands and panels

Revolutionary use:
a team can build an IDE-class workbench without first spending years creating
the workbench shell.

### Geometry, CAD, And Topology Tools

`worth-ui` should support:

- spatial canvases
- branch-local previews
- topology inspectors
- selection models
- before/after diffs
- mutation planning and recovery
- historical and lineage-aware inspection

Revolutionary use:
users can explore and inspect structural edits before committing them, with the
runtime able to explain exactly what changed and why.

### AI-Native Editing Systems

`worth-ui` should support:

- speculative edit branches
- AI-generated change previews
- causal explanations
- confidence and advisory surfaces
- accept/discard/merge workflows
- recovery briefs and structured failure handling

Revolutionary use:
AI stops feeling like a hidden side effect generator and becomes a visible,
inspectable collaborator inside the desktop app.

### Data And Operations Applications

`worth-ui` should support:

- high-density live tables
- dashboards and metrics
- background jobs
- policy-aware views
- command workflows
- audit, history, and recovery panels

Revolutionary use:
internal software can feel as quick to build as web apps while being faster,
denser, more native, and more inspectable.

### Plugin-Driven Product Platforms

`worth-ui` should support:

- typed plugin contribution points
- permissioned commands and services
- contributed panels and inspectors
- contributed settings and themes
- runtime-aware data bindings

Revolutionary use:
a Worth UI app can grow into a platform without becoming an ungoverned pile of
extension callbacks.

## Roadmap Direction

This file is a vision document, not the execution roadmap. But the future work
should be derivable from it.

The highest-signal Worth UI programs are:

- Worth-native app shell and lifecycle
- hot-lowered UI source, canonical UI artifacts, host-neutral execution plans, and
  stable-ID state reconciliation
- component, command, Query view, settings, icon, token, and plugin capability
  registries for validating hot-reloadable UI source
- performance-stratified execution lanes for widgets, data grids, canvases,
  real-time HUDs, shader/material overlays, and custom render surfaces
- 240 FPS frame-budget instrumentation and certification for hot paths
- command registry and command projection surfaces
- dockable workspace layout and persistence
- design token, theme, density, and component-state system
- professional table, tree, inspector, list, tab, split, dock, timeline, log,
  console, form, and canvas primitives
- Query-bound view surfaces and live view binding
- structured runtime outcome rendering
- intent-aware command execution whose UI dispatch remains distinct from Query
  application-operation admission and mutation evidence
- preview, branch, diff, merge, and recovery UX primitives
- accessibility, focus, and keyboard navigation architecture
- native OS integration adapters
- settings, preferences, project, document, autosave, and recovery workflows
- background task presentation and diagnostics
- plugin contribution and permission architecture
- developer tooling: component gallery, layout debugger, command inspector,
  accessibility inspector, Query inspector, profiler, screenshot tests
- packaging, installers, auto-update, release channels, crash reporting, and
  session restore

If a capability is named here and not yet built, it is roadmap work.

If a capability is built but not yet proven under multi-window, live-update,
preview, accessibility, packaging, and failure scenarios, it is certification
work.

## Non-Goals

- reintroducing a third-party immediate UI substrate or host-owned UI meaning
- turning Worth UI into a web runtime
- copying Electron's architecture instead of competing with its product
  completeness
- owning authoritative truth, query planning, mutation authority, or signal
  scheduling
- treating desktop-native integration as optional polish
- treating accessibility as a later compliance pass
- letting every app invent its own command, settings, docking, live data, and
  recovery systems
- building ornamental widgets before the professional app shell and workbench
  primitives are trustworthy
- exposing lower runtime internals directly to UI code when Query should own
  the public semantic lane

## Companion Documents

- [WORTH Query orientation for AI agents](../../workspaces/worth-query/crates/worth-query/docs/AI_README.md)
- [plans/worth-query/worth_query_vision.md](../worth-query/worth_query_vision.md)
- [plans/worth-runtime-bridge/worth_runtime_bridge_vision.md](../worth-runtime-bridge/worth_runtime_bridge_vision.md)
- [plans/worth-relational/worth_relational_vision.md](../worth-relational/worth_relational_vision.md)
- [plans/worth_signal/worth_signal_vision.md](../worth_signal/worth_signal_vision.md)

Worth UI is where the stack becomes a product people can touch. If the UI
platform is shallow, the runtime's truth, query, branch, preview, inspection,
and recovery capabilities remain invisible. If the UI platform is strong, the
desktop application becomes the place where the whole Worth stack feels obvious:
fast, native, typed, live, explainable, shippable, and more pleasant than the
web stack people settled for.
