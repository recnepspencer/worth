# WORTH Site Spec

## Purpose

Build a static developer-facing website for WORTH that can be deployed easily to
GitHub Pages or any other static host.

The site has three jobs:

1. present WORTH with a polished landing page that feels current, animated, and
   high-end
2. surface the existing docs in a much nicer reading experience without
   rewriting their substance
3. provide a progression of demos that show what each WORTH primitive buys the
   developer, including before/after code comparisons and explicit built-in
   outputs

This is not a generic marketing site and not a generic docs site. It is a
product presentation layer for WORTH.

## Required Product Positioning

The site must position WORTH as a framework where a single authored primitive
gives the developer multiple built-in capabilities that would otherwise require
assembling several libraries plus custom glue.

The site must remain developer-focused:

- the primary unit of explanation is authored code
- the primary unit of proof is what WORTH gives the developer from that code
- the site must explain capability surface, not just show polished end-user UI

## Required Tooling Honesty

All WORTH-side demos, examples, and comparisons must use the real WORTH
surfaces.

This is mandatory.

Examples:

- router demos must use `signals.router.*(...)`
- form demos must use `signals.form(...)`
- resource demos must use WORTH resource lines and related WORTH resource
  surfaces
- history/replay demos must use WORTH history, replay, restore, or branching
  surfaces where those behaviors are being claimed

The site must not:

- fake WORTH behavior with custom app glue while presenting it as a built-in
  WORTH capability
- replace WORTH router examples with another router while still calling the demo
  a WORTH router demo
- simulate built-in outputs that are not grounded in actual WORTH APIs

The alternative side of a comparison may use strong conventional tools such as
React Query, React Router, or a strong form stack. The WORTH side must use
WORTH.

## Static Hosting Requirement

The site must be fully static.

Requirements:

- deployable as a generated static output directory
- suitable for GitHub Pages or equivalent static hosting
- no backend required for docs, navigation, or demos
- no server-side session state
- no dependence on runtime API infrastructure for the site experience itself

Client-side interactivity is allowed. The deployed site artifact must remain a
static site.

## Audience

Primary audience:

- developers evaluating WORTH
- developers trying to understand why WORTH primitives are different from
  ordinary alternatives
- developers deciding whether WORTH reduces glue code in forms, routing,
  resources, and history-heavy workflows

Secondary audience:

- people skimming the landing page who need a fast sense of what WORTH does
- existing WORTH users who want a better entry point into docs and examples

## Site Areas

The site consists of three main areas:

- Landing
- Docs
- Demos

Required top-level routes:

- `/`
- `/docs`
- `/docs/...`
- `/demos`
- `/demos/:demoId`

The site should feel like one product, not three separate applications.

## Landing Page Requirements

The landing page is the primary product pitch.

It must feel modern, animated, and high-polish. It should behave like a premium
framework/product site where content reveals itself as the user scrolls.

The landing page must include:

- a hero section that frames WORTH at a high level
- a sequence of major feature sections as the user scrolls
- strong animated transitions or staged reveals between sections
- per-feature explanation blocks
- before/after code comparisons for major primitives
- explicit "what you get" output lists for those code blocks
- entry points into the docs
- entry points into the demos

The landing page must not collapse into:

- a plain grid of feature cards
- a shallow aesthetic shell with no clear technical explanation
- a list of APIs without clear developer payoff

## Hero Requirements

The hero must:

- communicate what WORTH is in one strong sentence
- establish that WORTH offers more built-in structure than a typical assembled
  stack
- visually signal that WORTH spans multiple layers of app behavior
- include a primary CTA to demos
- include a secondary CTA to docs

The hero does not need to explain every concept. It must establish intrigue,
scope, and momentum.

## Landing Feature Section Requirements

The landing page must include distinct feature sections for:

- Signals / local reactive state
- Forms
- Router
- Resources
- History / replay / restore
- Composed workflows

Each feature section must contain:

- a short feature headline
- a short explanation of what the feature is for
- a before/after code comparison
- a "what you get" list showing built-in outputs of the WORTH code
- a CTA to the relevant demo
- a CTA to the relevant docs

Each feature section may also contain:

- a small live or simulated visual preview
- an animated reveal tied to scroll
- a staged progression from authored code to surfaced capabilities

## Before / After Comparison Requirements

This is a core part of the site.

For each major feature section, the site must show:

- a steelman alternative example
- a WORTH example
- a concise explanation of what extra code, glue, or infrastructure the
  alternative requires
- a concise explanation of what WORTH collapses into a smaller or more unified
  surface

The comparison must be fair. It must not compare WORTH to a deliberately weak
strawman.

The point is not to mock alternatives. The point is to show how much structure
WORTH provides per authored block.

Each comparison must also include a "what you get" list that makes the output
of the WORTH block explicit.

## Docs Requirements

The docs are the existing WORTH docs, presented inside a polished static docs
experience.

The docs should not be rewritten as a new authored documentation corpus in this
phase.

Requirements:

- ingest, mirror, or otherwise render the existing docs content
- preserve the substance of the current docs
- render those docs in a much nicer shell and reading experience
- provide section navigation
- provide article rendering
- provide links from docs to demos when relevant
- provide links from demos back to docs
- visually match the rest of the site

The docs area should feel like "the real docs, but presented beautifully."

## Demo Philosophy

The demos are not meant to be flashy app toys first.

They are meant to prove what WORTH primitives buy the developer.

Every demo should answer:

- what did the developer write?
- what built-in behavior did WORTH supply?
- what would take significantly more assembly in a conventional stack?

The demos must be progressive. They should become more powerful as the user
moves through them.

For the first pass, demos must stay simple and legible.

This is mandatory.

The first-pass demos should prefer:

- one small scenario each
- one primary idea each
- minimal domain complexity
- obvious visible outcomes
- small enough code blocks that a developer can understand the authored surface
  at a glance

The first-pass demos should avoid:

- large “workspace” or “studio” style demos
- multi-actor collaboration
- elaborate fake product domains
- sprawling multi-pane interfaces that hide the authored code
- stacking every WORTH surface at once before the user has seen the simpler
  building blocks

## Demo Index Requirements

The demos index page must:

- list all demos in progression order
- briefly explain what each demo proves
- indicate relative complexity or progression stage
- make it easy to start from the beginning or jump directly to a later demo

The demos index should communicate a ladder, not a pile.

## Required Demo Set

### Demo 1: Signals

Purpose:
Show the smallest useful WORTH primitive and what it gives you for local
reactive state.

Scenario:
A simple counter with derived values.

The UI should contain:

- a counter value
- increment/decrement controls
- at least two derived reads such as doubled value and status label

The authored WORTH code should stay very small and center on:

- `signals.input(...)`
- `signals.computed(...)`
- optionally one published output surface if needed for presentation

This demo must demonstrate:

- authored local signal/input
- derived/computed outputs
- reactive updates
- basic diagnostics/history awareness if appropriate

This demo must show:

- a small authored WORTH code sample
- a live result
- a before/after comparison versus an ordinary local-state approach
- a "what you get" output list

Primary message:
WORTH gives you more structured local reactivity than plain ad hoc component
state.

### Demo 2: Form

Purpose:
Show what `signals.form(...)` buys you in a simple but real form.

Scenario:
A publishable article form with `title` and `status`.

The UI should contain:

- title field
- status field
- validation messages
- readiness state
- visible source, draft, and effective snapshots

The form behavior should stay simple:

- empty title is invalid
- changing fields marks the form dirty
- readiness changes visibly
- submitting or publish action posture is visible

The authored WORTH code should center on:

- `signals.form(...)`
- a small field set
- simple validation and readiness behavior
- minimal action configuration if needed

This demo must demonstrate:

- source vs draft vs effective values
- dynamic validation
- dirty truth
- readiness
- action or submission posture
- visible form state surfaces

This demo must show:

- an authored form block
- live form behavior
- a before/after comparison versus a conventional form stack
- an output list of built-in capabilities

Primary message:
WORTH forms are not just fields and validation wiring; they expose a full form
model.

### Demo 3: Router

Purpose:
Show that WORTH routing is not just string matching or view switching.

This demo must use the WORTH router surface, not a substitute.

Scenario:
A tiny app with `Home`, `Items`, and `Item Detail`.

The UI should contain:

- navigation links
- current route display
- breadcrumb display
- route params readout for the detail page
- route projection/admission explanation in a compact inspector

The authored WORTH code should center on:

- `signals.router.route(...)`
- `signals.router.define(...)`
- `routeRef.to(...)`
- route projection or admission reads

The demo should stay small and should not become a large application shell.

This demo must demonstrate:

- route declaration
- typed route generation
- projection
- admission
- breadcrumbs and navigation structure
- pending or visible route policy if appropriate

This demo must show:

- an authored `signals.router` block
- live route transitions or route inspection
- a before/after comparison versus a conventional routing setup
- an output list of built-in route surfaces

Primary message:
WORTH routing owns richer navigation truth than standard route libraries.

### Demo 4: Resource Line

Purpose:
Show what a WORTH resource declaration buys you beyond fetch state.

This demo must use WORTH resource lines and related WORTH resource surfaces.

Scenario:
A task detail resource with refresh and update.

The UI should contain:

- one loaded task record
- visible pending/settled state
- an editable status or title change action
- mutation response or reconciliation readout

The authored WORTH code should center on:

- one API declaration
- one detail resource
- one line materialization
- one write or patch path

The demo should show a very small resource surface, not a full dashboard.

This demo must demonstrate:

- resource line declaration
- loading and settled state
- mutation response handling
- reconciliation or fallback posture
- richer resource lifecycle than ordinary fetch abstractions

This demo must show:

- an authored WORTH resource block
- live resource behavior
- a before/after comparison versus React Query or equivalent
- an output list of built-in capabilities

Primary message:
WORTH resource lines carry more application semantics than conventional
server-state hooks.

### Demo 5: Route-Coupled Resource-Backed Form

Purpose:
Show the first truly stacked example.

This demo must use real WORTH surfaces for routing, form modeling, and resource
truth.

Scenario:
An edit-task flow with two routes: `Detail` and `Edit`.

The UI should contain:

- a detail page showing loaded task truth
- an edit route with a resource-backed form
- navigation between detail and edit
- visible continuity or dirty-state behavior when leaving edit
- visible source/draft/effective values in the edit step

The authored WORTH code should center on:

- one route definition set
- one resource line as source truth
- one form backed by that resource
- one simple continuity or route-coupled behavior

This is the first stacked demo, but it must still stay narrow. It should not
expand into approvals, teams, collaboration, or complex domain workflows.

This demo must demonstrate:

- form + route + resource interaction
- route-aware form flow
- server-backed source truth
- continuity through navigation
- multi-layer built-in behavior from a composed setup

This demo must show:

- a moderate-sized authored WORTH example
- live behavior across multiple steps
- a capability list showing how multiple WORTH surfaces compose
- related docs links for forms, router, and resources

Primary message:
WORTH primitives compose into workflows without requiring a separate
orchestration layer.

### Demo 6: History / Replay / Branching

Purpose:
Show the most advanced differentiator in the sequence.

This demo must use real WORTH history, replay, restore, or branching surfaces
for the claims it makes.

Scenario:
Return to the edit-task flow from Demo 5 and add a simple history/replay view.

The UI should contain:

- a small session timeline or history list
- ability to jump back to an earlier state
- visible restoration of prior route/form/resource truth
- visible preservation of the current path when the user resumes editing from an
  earlier point, if the chosen WORTH surface honestly supports that claim

The authored WORTH code should center on:

- the same narrow task-edit domain from Demo 5
- WORTH history, replay, restore, or branching surfaces
- a compact visible history control

This demo must remain a simple extension of Demo 5. It should not introduce a
new domain.

This demo must demonstrate:

- retained history
- replay or restore
- revisiting earlier states
- non-destructive alternate paths or branching if supported by the chosen
  presentation
- richer state/time behavior than ordinary undo-style implementations

This demo must show:

- authored code focused on history-bearing behavior
- a live replay or restore interface
- a before/after explanation describing how much custom machinery would
  otherwise be required
- an explicit output list of built-in history surfaces

Primary message:
WORTH treats time, history, and revisitation as first-class structured
behavior.

## Demo Page Requirements

Every demo detail page must contain:

- demo title and short purpose
- authored WORTH code block
- before/after comparison block
- live example area
- "what you get" capability list
- links to related docs
- links to previous/next demos

Optional but recommended:

- inspector panel
- tabs for code/result/capabilities
- staged reveal of extra surfaces as the user interacts

Every demo page must use the same demo shell and the same structural regions.

## Demo Scope Guardrails

The required demo progression for the first version is:

1. Counter
2. Publishable article form
3. Home / items / item-detail router
4. Task detail resource
5. Task detail / task edit route-coupled resource-backed form
6. History/replay on the same task edit flow

This ordering is required.

The first implementation should not replace these with broader or more cinematic
domains unless the spec is intentionally revised.

The goal is clarity first:

- the user should understand each demo immediately
- each step should obviously build on the previous one
- the code should remain readable enough to support the “what do I get from
  this block?” framing

## Shared Consistency Requirements

To keep implementation clean, the following are required:

- one shared site shell
- one shared token system
- one shared docs shell
- one shared demo shell
- one shared comparison/codeblock system
- one shared "what you get" capability-list system
- one shared section/panel/card base system

The implementation must avoid:

- duplicated style logic for landing, docs, and demos
- multiple unrelated layouts for similar content
- hand-built per-demo scaffolding
- copy-pasted before/after comparison UI across demos
- page-specific ad hoc styling systems

## Structured Content Requirements

The implementation should reuse structured content wherever possible.

At minimum, the following should be modeled as structured content:

- landing feature section metadata
- demo metadata
- docs navigation metadata
- relationships between features, docs, and demos

This is important so that:

- landing links do not get hardcoded repeatedly
- docs-to-demo connections remain consistent
- demo order and relationships stay manageable

## Implementation Priorities

The implementation should be approached in this order:

1. establish static site architecture
2. establish shared shell and shared token primitives
3. establish docs ingestion/rendering approach for existing docs
4. establish shared comparison/codeblock/capability-list primitives
5. implement landing page structure and feature sections
6. implement demos index and demo shell
7. implement demos in progression order
8. polish motion and transitions once structure is stable

## Acceptance Criteria

This work is successful when:

- the site can be deployed as a static build
- the landing page feels premium and modern
- the landing page clearly explains each major WORTH feature
- before/after comparisons clearly show WORTH's leverage
- each feature section explicitly lists what the WORTH code block gives you
- the docs are the existing docs presented in a polished static docs experience
- the demos are clear, progressive, and developer-focused
- the router demo uses WORTH router surfaces
- the form demo uses WORTH form surfaces
- the resource demo uses WORTH resource surfaces
- the history demo uses WORTH history/replay/restore surfaces for the claims it
  makes
- later demos show deeper composition without becoming a disconnected mini-app
- implementation stays structurally consistent and avoids ugly duplication
