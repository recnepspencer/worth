# WORTH Query Todo Showcase Spec

> **Status:** Superseded by
> [Milestone 9.16](./milestone-9.16.md).
>
> This draft is retained as historical design context only. It is not an
> acceptance target, roadmap authority, or substitute for the authenticated,
> asynchronous bank courtroom. New Query front-door work must follow Milestone
> 9.16.

> **Scope class:** Fast-moving application showcase, not a new roadmap milestone
>
> **Vision parent:** [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_vision.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md)
>
> **Test requirements reference:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
>
> **Most relevant capability specs:**
> - [milestone-5.2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.2.md)
> - [milestone-5.5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.5.md)
> - [milestone-8.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-8.md)
>
> **Primary architectural driver:** prove that `worth-query` can act as the
> app-facing surface for a small but visually ambitious todo application whose
> code stays compact while exposing capabilities that ordinary CRUD apps usually
> need far more glue to achieve

## Goal

Ship a small in-memory sprint-planning todo application that demonstrates, in a
single coherent UI, that `worth-query` can provide live views, grouped board
semantics, focused inspector semantics, preview-branch planning, compare-to-
main inspection, and promote/discard workflow posture with substantially less
application glue than a conventional CRUD architecture would require.

## Why This Showcase Exists

The WORTH Query vision makes a very strong product claim:

- `worth-relational` defines truth
- `worth-query` is how consumers ask for it

That claim is easiest to believe when the consumer is not a framework author or
an internal harness, but an ordinary application.

This showcase exists to make that product claim visible in one compact app:

- one small truth model
- one small set of queries
- several meaningfully different surfaces
- branch-native planning behavior
- very little bespoke application state management

The point is not to prove every roadmap capability or close a new subsystem
milestone. The point is to make a viewer feel, within seconds, that:

`worth-query` is not a read helper. It is the app surface.

The app should therefore optimize for capability density per line of app code,
not for exhaustive feature count and not for certification-grade subsystem
proof.

## Governing Summaries

- `MENTALITY.md`: the load-bearing problem is not "make a todo app quickly." It
  is "make a small app reveal architecture-level powers honestly without
  leaning on hidden glue." Because the lower foundations already exist, this
  spec should move fast at the feature layer while staying strict about
  authority and honesty.
- `arch_laws.md`: the key protection is facade and authority discipline. The
  showcase must make `worth-query` the daily-driver facade while leaving truth,
  preview lifecycle, and workflow authority in the lower runtimes that already
  own them. Query result surfaces must stay typed and explicit rather than host-
  stitched bags.
- `perf_laws.md`: the app must demonstrate semantic-delta behavior rather than
  hiding broad recomputation behind pretty UI. Board movement, inspector
  updates, compare surfaces, and summary changes should visibly map to narrow,
  query-shaped updates instead of whole-app refresh folklore.
- `domain_laws.md`: the spec must separate responsibilities cleanly. Task
  modeling, query surfaces, UI panels, explainability surfaces, and demo data
  should not collapse into one generic app blob.
- `worth_query_vision.md`: the single most important thing it protects is that
  consumers should ask for truth through typed queries with live, branch, diff,
  view-shape, and inspector semantics as native capability. This showcase must
  look and feel like that thesis made concrete.
- `worth_query_roadmap.md`: the strongest shaping constraint is that `WORTH-
  query` is meant to be the platform-level framework surface for ordinary
  developers while lower crates remain authoritative. This showcase must reuse
  admitted capability families rather than inventing shadow app semantics.
- `test-requirements.md`: the most important protection is honesty about proof
  level. This showcase should reference certification expectations, but it must
  not pretend to close a subsystem capability boundary merely because the demo
  works. The app needs demo-grade verification and honest capability scope, not
  fake milestone closure.
- `milestone-5.2.md`: preview sessions are session-shaped, basis-explicit, and
  not ambient branch aliases. Any preview/planning story in the app must make
  that explicit.
- `milestone-5.5.md`: branch workflow surfaces must remain authority-
  preserving. If the showcase exposes promote/discard or compare-oriented
  workflow actions, they must be framed as query-lowered workflow posture, not
  as a second mutation engine living in the app.
- `milestone-8.md`: view shapes are semantic surfaces, not cosmetic tabs. The
  board, list, and focused inspector in this app should feel like different
  query interpretations over one truth model, not three separate hand-built UI
  states.

## Adversarial Constraint

This showcase must survive the following hostile condition:

> A viewer watches a tiny todo application switch between board, list,
> inspector, preview, compare, and promote/discard flows, and every one of
> those experiences must read as one coherent query-native system rather than a
> pile of UI-local state machines, host-side diffs, manual subscriptions, and
> branch aliases.

Concretely, the design fails if any key demo moment depends on:

- app-local mirrored state that silently becomes authoritative
- host-side transformation that reimplements a query capability already owned by
  `worth-query`
- preview mode represented as a boolean or branch-name alias instead of an
  explicit preview context
- compare views assembled from arbitrary result bag diffs rather than query-
  native basis-aware comparison surfaces
- full-surface rerenders or broad hidden recomputation where the product story
  claims selective or query-shaped updates
- one-off hand wiring that makes the demo impressive but teaches the wrong
  platform story

The app therefore succeeds only if the visible wow moments are mostly the
direct consequence of existing WORTH Query surfaces, with the application layer
providing mostly schema, seed data, layout, and a small amount of presentation
logic.

## Product Decision Lock

- this is a `worth-query` showcase first and a todo app second
- the todo domain is intentionally ordinary so the capability delta is legible
- the app remains in-memory and local-first for the first version
- the app code should consume one canonical `Task` truth model and one small
  query family rather than many bespoke state channels
- the hero capability is branch-native sprint planning, not checkbox CRUD
- board, list, inspector, summary, and compare are all different surfaces over
  the same underlying task truth
- preview/planning behavior must stay basis-explicit and must not collapse into
  local draft state
- "why did this change?" is part of the product story, not an afterthought
- the UI should feel premium and intentional rather than tutorial-grade
- speed matters more than exhaustive proof, but honesty matters more than speed
- if a capability is not admitted cleanly through `worth-query`, the showcase
  must either defer it or present it as explicit debt rather than fake it

## User Experience Thesis

The viewer experience should progress in this order:

1. this looks like a polished planning tool
2. this is one small task app with one coherent model
3. this same truth is driving multiple surfaces
4. this app has branch-native planning and compare
5. this app explains its own updates
6. this is far more capability than a normal todo app should have for this
   amount of app code

The intended closing line is:

`This todo app has live query views, grouped board semantics, focused inspector updates, preview-branch planning, compare-to-main, and promote/discard workflow posture in less app code than most CRUD apps spend on state plumbing.`

The hero user story is `Plan Sprint`:

- start on `main`
- open a preview planning session
- move, reprioritize, and reassign a few tasks
- see board, summary, and inspector update coherently
- compare the preview against `main`
- promote or discard the plan

That story is understandable to any audience and exposes several WORTH Query
capabilities in one compact loop.

## UI Architecture

The UI should feel like an operational planning instrument, not a toy CRUD app
and not a Jira clone.

### Layout Thesis

The application is organized into three persistent zones:

- `Reality Bar`: top-level branch, preview, compare, and workflow controls
- `Planning Stage`: the main board/list/compare surface
- `Focus Rail`: inspector, signals, and query explainability

This structure makes branch reality and query behavior visually central rather
than hiding them in secondary menus.

### Reality Bar

The `Reality Bar` is the app's signature UI element.

It must show:

- app title, likely `Sprint Planning`
- current reality badge:
  - `main`
  - `preview: sprint-next`
- current mode chip:
  - `Board`
  - `List`
  - `Compare`
- divergence summary such as `4 changes across 3 tasks`
- primary actions:
  - `Plan Sprint`
  - `Compare To Main`
  - `Promote`
  - `Discard`

Design requirements:

- preview mode visibly changes the bar's tone
- compare mode feels like crossing into a second reality, not opening a modal
- the branch state must remain obvious at all times

### Planning Stage

The main stage owns three semantic views over the same task truth:

- `Board`
- `List`
- `Compare`

#### Board

The board is the hero surface.

It should use grouped kanban semantics over task status:

- `Todo`
- `Doing`
- `Blocked`
- `Done`

Each lane should expose:

- task count
- optional pressure/load count
- compact premium cards with title, assignee, and priority

The board must not resemble generic Trello styling. It should look deliberate,
structured, and data-aware.

#### List

The list is the analytical view of the same task query.

Suggested columns:

- title
- status
- assignee
- priority
- preview/change marker

The list should feel like a result-shape transformation of the same truth, not
like a separate screen with separate state.

#### Compare

The compare view is the mind-blowing surface.

It should show the task truth difference between preview and main in a way that
feels like comparing realities, not reading a debug dump.

Minimum compare requirements:

- explicit `main` versus `preview` framing
- grouped changed tasks
- before/after field differences
- visible status-lane movement where relevant

### Focus Rail

The `Focus Rail` holds three stacked panels:

- `Inspector`
- `Signals`
- `Why This Changed`

#### Inspector

The inspector shows the selected task with focused detail semantics.

It must make it obvious that small field changes do not require a whole-screen
refresh. The selected task becomes the easiest place to feel focused query
 patches.

Minimum task fields:

- title
- status
- assignee
- priority

#### Signals

The signals panel should remain compact and operational.

Suggested metrics:

- open tasks
- doing now
- blocked count
- high-priority open
- assignee load

These should feel like derived operational readouts, not generic dashboard
tiles.

#### Why This Changed

This panel is critical to the product story.

It should explain recent update reasons in short query-native language, such as:

- task status changed from `todo` to `doing`
- grouped lane membership updated
- focused inspector patch applied
- summary recomputed for `doing_count`
- compare basis changed from `main` to `preview`

The panel should build trust without overwhelming the main UX.

## Scope

### In Scope

- one in-memory `Task` truth model with a small seed dataset
- one canonical query family over tasks
- board/list/compare stage modes
- a focused task inspector
- compact derived summary signals/readouts
- preview-branch sprint planning flow
- compare-to-main flow
- promote/discard workflow posture where admitted
- a small query explainability panel
- a visually ambitious but implementation-realistic UI

### Intentionally Out Of Scope

- persistence
- auth
- collaboration
- comments or attachments
- nested subtasks
- large filter builders
- search
- notifications
- arbitrary reporting
- drag-and-drop if it materially increases code volume or obscures the query
  story
- any new lower-layer capability that would be better solved inside a WORTH
  Query roadmap milestone

### Initial Task Truth Model

The first version should keep the task schema small:

- `id`
- `title`
- `status`
- `assignee`
- `priority`

Optional sixth field only if it materially helps the story:

- `sprint`
- or `due_bucket`

The schema should stay compact enough that the audience can grasp it instantly.

## Phases

### Phase 1: Freeze The Showcase Truth Model And Query Surface

Phase 1 exists to prevent the demo from turning into ad hoc UI state with a
task skin on top.

This phase should define:

- the canonical `Task` truth model
- the canonical task query family
- the minimal seeded dataset
- the admitted view-shape set the app will actually show
- the basis/context set the app will actually show:
  - main
  - preview
  - compare

This phase leaves the system in a coherent state where the rest of the app can
be expressed as surfaces over one small query vocabulary instead of inventing
feature-local data flows.

### Phase 2: Ship The Core Planning Stage

Phase 2 exists to make the app already impressive before branch workflow lands.

This phase should ship:

- the `Reality Bar` shell
- the `Board` stage
- the `List` stage
- the `Inspector`
- the `Signals` panel

This phase should prove that:

- one task truth can drive multiple views
- grouped board semantics are visible
- focused inspector semantics are visible
- the app already feels premium before compare/preview arrive

This phase leaves the system in a coherent state where the app is already a
strong live task demo even without branching.

### Phase 3: Ship Branch Preview And Compare

Phase 3 is the hero phase.

This phase should add:

- `Plan Sprint`
- explicit preview basis state in the UI
- task edits within preview
- compare-to-main stage
- promote/discard actions where admitted

This phase must keep preview honest:

- no draft-only local shadow state
- no hidden branch aliasing
- no compare-by-arbitrary-JSON-diff shortcuts

This phase leaves the system in a coherent state where the showcase can already
deliver its main on-stage story.

### Phase 4: Ship Query Explainability And Capability Density

Phase 4 exists to turn "cool demo" into "clear product message."

This phase should add:

- `Why This Changed`
- stronger change markers in board/list/compare
- concise query-native update explanations
- small copy refinements that teach the right mental model

This phase leaves the system in a coherent state where a viewer can understand
why the app feels more capable than its code size suggests.

### Phase 5: Polish The Demo Surface For Live Presentation

Phase 5 exists to make the showcase memorable rather than merely correct.

This phase should focus on:

- typography and visual hierarchy
- mode transitions
- preview/compare atmospheric cues
- empty states
- seed data quality
- demo-script-oriented defaults

This phase should not add major new capabilities. It should sharpen the story
the existing capabilities tell.

## Must Ship

- one compact in-memory task schema and seed dataset
- one canonical task query family that the app reuses across visible surfaces
- one grouped board view
- one analytical list view
- one focused inspector surface
- one compact summary/signals surface
- one explicit preview planning flow
- one compare-to-main surface
- promote/discard actions or explicit placeholder debt markers if one of those
  actions is not admitted cleanly yet
- one explainability surface that helps users understand why the UI changed
- one polished visual direction strong enough that the app does not read as a
  tutorial
- one concise demo script embedded in the app's default flow through initial
  data, labels, and affordances

## Must Preserve

- `worth-query` remains the app-facing facade
- lower runtimes remain authoritative for truth, preview lifecycle, and
  workflow semantics
- the app does not create shadow authority in UI-local state
- view changes do not become separate duplicated data pipelines
- preview remains basis-explicit and not branch-alias folklore
- compare remains basis-aware and not arbitrary bag diffing
- explainability remains additive and does not become the hot-path authority
- implementation speed does not justify misleading capability claims
- if a workflow capability is deferred, the UI must say less rather than imply
  more

## Acceptance Evidence

This showcase is accepted when all of the following are true:

- a viewer can understand the app's core story in under one minute
- the app shows board, list, inspector, preview, compare, and summary surfaces
  over one small task model
- the preview planning loop is legible and basis-explicit
- compare-to-main visibly communicates "difference between realities"
- the app's most impressive moments come from `worth-query` capability reuse,
  not from hidden bespoke plumbing
- the code remains compact enough that a reasonable engineer could believe the
  "more capability than a typical CRUD app for fewer lines of app glue" claim
- the app is visually polished enough to feel intentional on first impression

Minimum implementation verification should include:

- one smoke test or harness proving seeded task truth can render through the
  primary stage without runtime errors
- one verification path proving preview and compare flows can execute end to
  end for the admitted happy path
- one verification path proving focused selection/inspector updates still work
  through preview changes
- one short recorded or rehearsable demo sequence that exercises the hero loop

This showcase does not claim:

- subsystem milestone closure
- certification matrix completion
- full support for every WORTH Query feature family
- durable or production-hard guarantees beyond the admitted demo path

## Architectural Notes

### This Is A Showcase Spec, Not A New Milestone

This document deliberately borrows milestone discipline without pretending to
be a roadmap milestone.

Why:

- the app is downstream of already-specified WORTH Query capability families
- the user goal is fast delivery and strong storytelling
- the right question is "what proves the product thesis quickly and honestly,"
  not "what new subsystem capability are we certifying"

### Capability Density Beats Feature Count

The correct optimization target is:

- fewer lines of app-specific code
- more visible capability
- less bespoke state plumbing

The wrong optimization target is:

- absolute minimum line count
- giant feature surface
- "impressive" UI that secretly bypasses WORTH Query

### The Todo Domain Must Stay Ordinary

The more ordinary the domain, the more impressive the capability density feels.

That means:

- no exotic schema
- no business-specific jargon
- no domain complexity that competes with the platform story

The magic should come from what the app can do, not from what the task domain
is.

### The UI Is Part Of The Proof

A bland CRUD UI would undersell the platform even if the architecture were
excellent.

The visual design therefore has a real architectural job:

- make branch reality visible
- make query mode visible
- make compare feel meaningful
- make selective updates feel intentional

The UI is not decoration around the platform story. It is the delivery vehicle
for that story.

### Explainability Must Stay Productive

The explainability surface exists to convert invisible infrastructure into
visible product value.

It should:

- explain only the most recent or relevant changes
- use query-native language where possible
- avoid flooding the user with internal jargon or trace spam

If the panel starts reading like a debug console instead of a trust-building
surface, the showcase has missed the point.

## Sequencing Notes

This spec belongs under `plans/worth-query/` because it is a product-facing
demonstration of WORTH Query's application surface, not a generic UI exercise.

It should be built after, and explicitly on top of, already-admitted capability
families from:

- preview session query contexts
- view-shape semantics
- workflow posture where admitted

It should not block or reorder the roadmap because it does not define new query
semantics. It consumes them.

Recommended implementation order:

1. task truth model and seeded data
2. canonical query family
3. board/list/inspector shell
4. preview planning flow
5. compare surface
6. explainability panel
7. visual polish and demo defaults

## Explicitly Deferred

- persistence and durable resume
- multi-user or collaboration behavior
- tenant or policy storytelling
- general-purpose filtering/search
- arbitrary historical reads beyond the main showcase loop
- additional workflow families beyond preview, compare, and promote/discard
- feature work that would require a new WORTH Query roadmap milestone to be
  honest
