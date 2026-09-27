# Milestone 3.10.2: Platform Pulse Seed and Visible Lifecycle Closure

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status

Status: Complete. Phases 1 through 4 closed on 2026-07-26. The required
human-visible product capability is closed on this same permanent Platform
Pulse application.

Post-completion evidence correction: the automated pulse courtrooms exercise
real filesystem, watcher, public application-shell, mounted, and egui
integration in-process. They do not enter the product binary's `main`,
`eframe::run_native`, `PlatformPulseNativeFrame`, or watcher worker. The
separately measured human/native launch remains valid and found the
main-thread stack defect, but it is not automated executable-world
certification. Milestone 3.10.3 owns that required corrective gate and now
blocks Milestone 3.11. This correction narrows the historical evidence claim;
it does not retract the completed visible product behavior.

## Placement

Milestones 3.10 and 3.10.1 are complete. Their mounted host contract,
real-filesystem lifecycle, DSL ownership, runtime topology, and facade closure
remain closed truth.

The permanent human-visible Platform Pulse requirement was adopted after those
milestones completed. This milestone supplies that missing 3.10 seed without
rewriting either historical completion claim. Pixel-to-mounted identity needs a
real visible pixel produced by the product path, so 3.10.2 originally gated
Milestone 3.11. The later evidence audit added Milestone 3.10.3 as the final
executable-world gate before 3.11 may begin.

## Goal

Make one deliberately small file-authored page visibly render through the real
Worth UI lifecycle:

```text
checked-in .wui bytes
  -> production filesystem snapshot
  -> DSL-owned sealed semantic handoff
  -> public application preparation and launch
  -> committed allocation + mounted frame publication
  -> complete runtime-owned static-paint mechanic
  -> canonical mounted host contract
  -> egui native shape
  -> human-visible pixel
```

The same page, source location, scenario identity, executable, documentation
section, in-process proof family, and Milestone 3.10.3 executable-world family
become the cumulative Platform Pulse for Milestones 3.11 through 3.23.

## Existing Truth and Exact Gap

The catch-up starts from evidence, not from a generalized renderer project.

- Phase 10 already proves actual `.wui` filesystem acquisition, operating-
  system watcher delivery, coherent replacement, mounted publication,
  production headless recording, and a real `egui::Context::run` frame.
- Milestone 3.10.1 already proves that file-authored source crosses the
  DSL-owned sealed handoff and that ordinary downstream execution reaches one
  condensed public facade.
- `UiMountedPaintBatchRow` currently carries primitive count, layer, optional
  resource reference, and primitive family. It does not carry the complete
  geometry and color required to draw a filled rectangle.
- `WorthUiHostEgui` therefore rejects any admitted `NativePaint` effect before
  effects. Its honest successful case is a no-effect frame, and
  `mounted_egui_adapter.rs` explicitly asserts that the frame produces no egui
  shapes.
- The Worth UI workspace has no checked-in `.wui` application and no permanent
  native application composition root a human can launch.

The missing claim is narrow but real: completed architecture has not yet
produced a visible page through its own canonical product path.

## Adversarial Constraint

Assume an implementer tries to make a window look correct without closing the
product boundary. The implementation must remain red if it:

- draws directly through `egui::Painter`, `egui::Ui`, or an eframe callback
  outside `WorthUiHostEgui`'s mounted-mechanic translation;
- lets the adapter choose a color, rectangle, visibility, layer, or fallback
  when mounted paint is incomplete;
- loads pulse source through an in-memory registration, certification helper,
  handcrafted package, or fake watcher event;
- takes a screenshot of unrelated egui content and associates it with a mounted
  frame after the fact;
- imports `worth-ui-runtime`, `worth-ui-dsl`, certification, or test-support
  internals from the pulse application;
- treats count-only paint evidence as a drawable primitive;
- changes visible pixels before the complete successor application is
  published; or
- claims bounded semantic hot rebind when it performed only the whole-
  application replacement already owned by 3.10.

## Governing Decisions

### One Permanent Downstream Application

Add exactly one workspace package named `worth-ui-platform-pulse`. It is a
downstream product composition root, not a new authority crate and not a
milestone-local demo. It may depend on:

- the curated `worth-ui` product facade;
- `worth-ui-host-egui`;
- the native shell dependency required to run an egui event loop; and
- narrow operating-system or logging dependencies only when the inventory
  proves the public lifecycle does not already own that mechanism.

It must not depend on `worth-ui-runtime`, `worth-ui-dsl`,
`worth-ui-certification`, or `worth-ui-test-support`. It cannot mint mounted
truth, choose runtime semantics, or expose a second product facade.

All later pulse work extends this package. No later 3.x milestone receives a
second pulse executable.

### One Canonical Checked-In Source

The pulse source lives at:

```text
workspaces/worth-ui/apps/platform-pulse/app/main.wui
```

That file is the human-editable product source and stable scenario root. Tests
may copy its exact bytes into an isolated temporary workspace so they can
exercise real watcher delivery without mutating the checkout. They may not
maintain a semantically equivalent second fixture by hand.

The source identity must change when admitted authored color meaning changes
and remain relatable across replacements through the existing source snapshot,
application generation, graph, mounted, and publication identities. Stable
scenario identity does not mean forged equality between replaced artifacts.

### Minimal Static Paint, Not Early Appearance

The only new visible primitive required here is a filled rectangle. Its
complete mounted mechanic joins:

- one exact mounted node receipt;
- one committed allocation box;
- one admitted file-authored color value or token reference resolved by Worth
  UI;
- one explicit layer and clip posture;
- one surface binding generation; and
- one mounted frame identity.

No field may be optional at the effect boundary. Missing allocation, unresolved
or invalid color, omitted layer/clip posture, stale node receipt, foreign
surface binding, unsupported protocol, or missing native-paint capability must
deny before effects and preserve predecessor truth.

This milestone does not introduce appearance roles, component defaults, hover/
pressed/focus/selected state styling, theme switching, theme invalidation,
typography, border/radius/shadow systems, or renderer-chosen fallbacks. Those
remain Milestone 3.16. The pulse proves only that already-admitted static color
and allocation meaning can become an honest native pixel.

### Runtime Meaning, Host Mechanics

Runtime mounting owns the complete static-paint row and its relation to node,
allocation, layer, surface, and frame identity. The host contract carries only
the sealed mechanic an adapter needs.

`WorthUiHostEgui` validates the whole mechanic before effects and mechanically
translates its canonical rectangle and color into egui coordinates and
`Color32`. It reports the actual effect family and translated primitive cost.
It does not query declarations, resolve tokens, invent layout, or select
appearance.

The production headless adapter records the same complete mechanic
deterministically. It may report the native effect as intentionally unperformed,
but its transcript must contain enough canonical values and identity to compare
against an independent authored expectation.

### Replacement, Not Hot Rebind

The pulse must exercise one real valid edit and one malformed edit through the
existing production filesystem watcher.

- A valid static-color edit prepares and presents one complete successor
  application, then publishes it atomically.
- Before publication, the predecessor remains both runtime-current and
  human-visible.
- A malformed edit yields the DSL/source owner's typed denial and keeps the
  predecessor application, mounted frame, and visible pixels current.

This is whole-application replacement. No test, API, documentation sentence, or
counter may label it preservation-aware semantic rebind. Milestone 3.12 remains
the owner of bounded rebind planning and unaffected-region preservation.

## Destination Topology

The implementation plan must confirm exact file names during its inventory, but
responsibilities land in these existing or committed homes:

```text
workspaces/worth-ui/
  Cargo.toml                                      [modify: one app member]
  apps/platform-pulse/
    Cargo.toml                                    [create: downstream package]
    app/main.wui                                  [create: canonical pulse source]
    src/main.rs                                   [create: native shell entry only]
    src/application.rs                            [create: public Worth UI lifecycle]
    src/native_frame.rs                           [create: egui frame/event-loop bridge]
  crates/worth-ui-host-contract/src/
    mounted_projection/                           [modify: complete filled-rect mechanic]
  crates/worth-ui-runtime/src/
    mounting/projection/                          [modify: runtime-owned paint completion]
  crates/worth-ui-host-egui/src/adapter/
    egui_host.rs                                  [modify: admission and dispatch]
    native_paint.rs                               [create: mechanical translation]
  crates/worth-ui-certification/tests/
    application_contracts.rs                      [modify: existing suite owner]
    application_contracts/platform_pulse.rs       [create: consolidated courtroom]
  docs/application-lifecycle.md                   [modify: human run authority]
```

If responsibility separation would make any code or test file exceed the
400-line cap, the implementation plan must split by named responsibility before
coding. It must not create `helpers.rs`, `common.rs`, `util.rs`, or `shared.rs`.

### Future Insertion

- Milestone 3.10.3 adds one permanent executable-world target beside, not
  inside, the existing in-process `application_contracts` owner. It enters
  through the exact Cargo-built pulse binary and does not reopen paint
  authority.
- Milestone 3.11 adds snapshot and point-to-mounted evidence beside the existing
  mounted projection and extends both cumulative pulse evidence lanes.
- Milestone 3.12 replaces the whole-application edit observation with bounded
  semantic admission/rebind evidence without changing the pulse executable.
- Milestone 3.16 broadens the minimal static color mechanic into admitted
  appearance roles and states rather than adding renderer-local styling.
- Milestones 3.19 through 3.22 attach diagnostics, visual evaluation, agent
  tools, and Worth Inspector to this same live application identity.

## Public Contract and DX Budget

The pulse executable must use the ordinary public path. If the current facade
cannot express a native file-authored application without importing runtime
types, this milestone may add the smallest audience-facing application-shell
contract to `worth_ui::facade::app`. It may not publicly expose prepared
mounting, projection assembly, publication, source AST, or adapter internals.

The human workflow has one canonical command:

```powershell
cargo run --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui-platform-pulse
```

`docs/application-lifecycle.md` must state:

- the exact source file the process watches;
- what visible rectangle and admitted color appear on first launch;
- the exact bounded color edit a human can make;
- the visible successor result and publication evidence handle;
- the malformed edit and expected predecessor-preservation result;
- how to restore the checked-in source; and
- how the process shuts down watcher and host resources.

The command must work from a clean checkout without generating tracked files,
requiring test-only features, or asking the human to assemble authority values.

## Performance and Build Budgets

This milestone deliberately adds one native executable because a human-visible
claim cannot be closed by a library test alone. The cost increase must be
visible and bounded:

- record clean, warm, link, and launch evidence for the new package before and
  after implementation;
- keep the executable outside ordinary test execution;
- reuse the existing `application_contracts` integration target;
- add no compiler-contract session and no nested Cargo invocation;
- keep steady unchanged frames free of source parsing, broad artifact scans,
  token lookup by string, paint reconstruction, or pulse-specific allocation;
- charge initial paint completion to admitted node/primitive width;
- charge a valid replacement to the existing replacement transaction plus
  affected paint completion; and
- record actual adapter translated-row, translated-byte, native-shape, and
  resource-cache evidence.

Any new dependency or material timing increase requires an explicit closing
amendment rather than disappearing into workspace totals.

Historical scope note: "outside ordinary test execution" meant that 3.10.2
measured the real binary separately while its automated courtrooms remained
in-process. Milestone 3.10.3 preserves that fast lane and adds one separately
budgeted required executable-world lane. It does not reinterpret the
3.10.2 integration target as having launched a native window.

## Phase 1: Inventory, Topology, and Courtroom Freeze

### Required Work

- Inventory the exact source-to-native path, including existing authority,
  identity, failure, effect, and cost owners.
- Record why the permanent app package is an application composition root and
  why neither the facade crate nor the egui adapter may own it.
- Freeze the canonical pulse source, stable scenario identity, exact first
  visible mechanic, valid edit, malformed edit, and independent oracles.
- Freeze the complete static-paint contract and every reject-before-effects
  condition before changing implementation.
- Record opening target/session/build budgets and the one allowed workspace
  member increase.

### Test Requirements

- A source-to-pixel edge ledger names producer, consumer, cardinality,
  lifetime, failure owner, cost class, and forbidden shortcut for every edge.
- Mutation controls prove that direct egui drawing, injected source, count-only
  paint, and detached screenshot evidence would make the courtroom red.
- The inventory records the existing no-shape egui case as honest predecessor
  evidence, not as a failure to delete.

### Exit Gate

No row is ambiguous, no authority is assigned to the app or adapter by
convenience, and the destination topology passes boundary and agent-context
checks before implementation begins.

### Phase 1 Closure Evidence

The exact closure ledger is
`plans/worth-ui/milestone-3.10.2-phase-1-proof-ledger.csv`. All eighteen rows
are proved against the frozen source-to-pixel contract, the opening cost
baseline, the complete boundary-check and Worth UI certification suites, and
the production constitutional entrypoints.

Live Cargo metadata remains at 11 Worth UI workspace members, 20 Cargo targets,
and 9 integration targets. The pulse package and native executable remain
unborn, so this phase has not smuggled Phase 2 or Phase 3 implementation into
the inventory gate.

The authoritative `worth-ui` line-cap scope passes, and every dirty Rust file,
including boundary-check tooling outside that script's workspace paths, is at
or below 400 lines. The broader repository-wide invocation remains red on
pre-existing, untouched over-cap files in other product workspaces. That
inherited debt is not represented as green evidence for this phase and was not
expanded into this milestone's scope.

Phase 2 real-boundary execution reopened the provisional rectangle origin.
No admitted source, allocation, runtime, or host authority owned the originally
inventoried `32,32` offset. The frozen courtroom contract and Phase 1 ledger are
therefore honestly amended to the runtime-derived full-viewport box
`0,0,160,96`; the adapter remains forbidden from inventing placement.

## Phase 2: Complete Static-Paint Authority

### Required Work

- Add the smallest host-contract representation for a complete filled
  rectangle with canonical geometry, admitted color, layer/clip posture, node
  receipt, and frame/surface basis.
- Complete that representation inside runtime mounting from carried admitted
  source/token and committed allocation authority.
- Keep count-only lane paint honest and non-drawable unless it can be joined to
  the exact complete primitive.
- Add capacity limits, typed denials, digest/version participation, retention
  accounting, and delta/reuse behavior for the new table or payload.
- Extend headless translation to record the complete canonical mechanic.

### Test Requirements

- Independent fixtures cover valid completion and every missing, stale,
  foreign, overflow, and unsupported input.
- Equal primitive counts with different geometry, color, node, layer, binding,
  or frame basis do not alias.
- Unchanged frames reuse only with an exact witness; a color or allocation
  change invalidates the affected paint mechanic.
- Headless expectations are authored independently rather than generated from
  the same production row being asserted.

### Exit Gate

Runtime can produce a complete static-paint mechanic without egui, and no
adapter or application code can construct or enrich it.

### Phase 2 Closure Evidence

The exact closure ledger is
`plans/worth-ui/milestone-3.10.2-phase-2-proof-ledger.csv`; all sixteen claims
are `PROVED`. The final source passed topology `111/111`, runtime `835/835`,
host-contract `8/8`, the honest egui predecessor `2/2`, and application
contracts `155/155`, plus formatting, workspace compile, both Rust line-cap
audits, boundary-check, and agent-context.

Holistic QA reopened three guarantees before closure. Mounting's token lookup
was moved behind execution-plan meaning so the historical mounting-to-
capability edge did not grow. The historical 3.10.1 inventories were amended
only for intentional product and successor evidence rather than absorbing
3.10.2 into old measurements. Finally, static-paint lowering now distinguishes
a component with no static-paint meaning from a static-paint component whose
required token is missing; the former remains non-drawable while the latter
still denies before effects.

## Phase 3: Native Translation and Permanent Pulse Application

### Required Work

- Teach `WorthUiHostEgui` to advertise and perform only the complete native
  filled-rectangle effect.
- Validate the full surface projection before the first shape is added.
- Translate canonical coordinates and color mechanically, emit actual egui
  shapes, and report completed native-paint effects and exact cost.
- Add the permanent pulse package and checked-in `.wui` source.
- Compose filesystem acquisition, watcher startup, application launch, surface
  registration, mounted frame execution, egui presentation, replacement, and
  shutdown through public facades only.
- Keep the native shell responsible only for event-loop scheduling and
  displaying adapter effects.

### Test Requirements

- A real `egui::Context::run` frame produces non-empty native shapes from the
  mounted projection and no application- or adapter-owned debug shapes.
- Unsupported or incomplete paint still rejects before any egui shape.
- Shape count and canonical bounds/color agree with independent expectations
  and the published frame identity.
- A downstream dependency audit rejects runtime, DSL, certification, and
  test-support imports from the pulse package.
- The documented launch command starts the real checked-in page.

### Exit Gate

A human can launch the checked-in page and see a native rectangle whose entire
meaning was complete before it crossed the host boundary.

### Phase 3 Closure Evidence

The exact closure ledger is
`plans/worth-ui/milestone-3.10.2-phase-3-proof-ledger.csv`; all twelve claims
are `PROVED`. The permanent `worth-ui-platform-pulse` package imports only the
curated product facade, egui adapter, and frozen eframe event-loop dependency.
Its checked-in source reaches one complete runtime-owned viewport rectangle,
and the real egui adapter produces one independently expected blue native
shape. Exact adapter evidence is one presented surface, six translated rows,
560 translated bytes, one native shape, and zero resource-cache or
asynchronous-handoff events.

A real executable launch uncovered an honesty defect that the library
courtrooms had missed: the Windows main thread overflowed before initial
publication because a 160-byte move-only source fact was multiplied across 64
inline scheduler slots and several by-value successor states. Moving the source
truth behind one owned ingress pointer reduced the ingress from 216 to 64
bytes and the prepared replacement commit from 52,264 to 23,080 bytes without
changing fixed-capacity mailbox operations. A 24 KiB footprint cap and a
512 KiB replacement prepare/receipt/commit regression retain that proof.

## Phase 4: Real Edit Lifecycle, Documentation, and Closure

### Required Work

- Extend the consolidated application courtroom with a temporary copy of the
  canonical pulse source.
- Exercise initial publication, one valid watched color replacement, one
  malformed edit, predecessor preservation, recovery, and shutdown.
- Relate filesystem snapshot, source diagnostic, application generation,
  mounted frame, paint mechanic, adapter effect, and native observation without
  collapsing them into one oracle.
- Add the permanent Platform Pulse section to
  `docs/application-lifecycle.md`.
- Record closing build, execution, allocation, projection, adapter, watcher,
  and retained-evidence costs.
- Update the roadmap status and handoff to Milestone 3.11 only after exact-source
  verification is green.

### Test Requirements

- The valid edit reaches the production operating-system watcher and changes
  visible color only after the successor mounted frame publishes.
- The malformed edit yields a typed source denial and leaves the predecessor
  application, mounted frame, adapter presentation, and pixels current.
- Killing or delaying watcher delivery cannot be mistaken for a successful
  replacement.
- A pulse-specific in-memory constructor, fake event, alternate fixture, direct
  egui paint, or screenshot-only assertion makes a named mutation control fail.
- The executable shuts down registered surfaces, watcher resources, and the
  active application cleanly.

### Exit Gate

The documented human workflow proves the actual product composition root. The
independent automated courtrooms prove real source, watcher, application,
mounted, and egui integration in-process. Their evidence agrees at the
application/host boundary without being mislabeled as one end-to-end automated
world. Milestone 3.10.3 owns the later-required executable join.

### Phase 4 Closure Evidence

The exact closure ledger is
`plans/worth-ui/milestone-3.10.2-phase-4-proof-ledger.csv`; all twelve claims
remain `PROVED` at their corrected evidence boundary. The consolidated
in-process courtroom copies the canonical source bytes to an isolated real
workspace, observes initial blue egui publication, performs a real atomic
blue-to-green watcher edit, rejects malformed source with the DSL owner's typed
diagnostic while preserving green egui pixels and generation, and restores a
fresh blue successor. Component-level shutdown evidence releases the watcher,
active host session, and exactly one registered surface. It does not prove
native-window close or child-process teardown.

`plans/worth-ui/milestone-3.10.2-phase-4-closing-cost-evidence.json` records
the final-source cost posture. Clean link is 142.257 seconds against a
180-second budget, warm relink is 3.636 seconds against a 15-second budget, and
native launch reaches `WORTH_UI_PLATFORM_PULSE_PUBLISHED` in an upper-bound
258 milliseconds against a 5,000-millisecond budget. The full isolated
dependency target is reported honestly as 5,277,464,138 bytes; only
263,012,921 bytes are identifiable to the added pulse package and charged
against the 2,000,000,000-byte retained-artifact budget. The isolated target
was removed after measurement.

The durable human workflow now lives in
`workspaces/worth-ui/docs/application-lifecycle.md`, including the exact
command, watched file, valid edit, malformed edit, publication markers,
canonical restoration, and normal shutdown behavior. This remains
whole-application replacement; Milestone 3.12 still owns bounded semantic
rebind.

## Must Ship

- one canonical checked-in Platform Pulse `.wui` page
- one permanent downstream pulse executable
- one minimal complete mounted filled-rectangle mechanic
- runtime-owned completion and egui-owned mechanical translation
- production headless and real egui observations of the same sealed meaning
- real valid and malformed filesystem replacement evidence
- durable human run documentation
- explicit source-to-pixel identity, denial, mutation, cost, and cleanup proof

## Must Preserve

- every completed 3.10 and 3.10.1 guarantee
- Query-free operation without dummy Query ceremony
- one public application lifecycle and one runtime-to-host presentation path
- predecessor truth on every pre-effect or preparation denial
- host exclusion from declaration, graph, Query, plan, allocation, publication,
  and appearance authority
- later milestone ownership described by the roadmap
- existing consolidated test and compile-contract topology

## Completion Standard

Milestone 3.10.2 is complete only when:

- a clean-checkout human run visibly renders the canonical file-authored pulse
  through the public lifecycle and real egui adapter;
- the visible primitive is complete runtime-owned meaning before host effects;
- production headless, mounted-publication, and egui-native evidence bind the
  same exact source and frame without sharing an oracle;
- a real watched valid edit publishes one coherent visible successor;
- a malformed edit preserves the last admitted visible predecessor with a
  typed denial;
- every anti-bypass mutation control fails for the intended reason;
- the permanent executable adds no product authority, private imports,
  milestone-local successor target, or ordinary-test window launch;
- boundary, agent-context, line-cap, format, clippy, workspace, certification,
  documentation, and cost gates pass on the exact final source; and
- Milestone 3.11 can attach pixel-to-mounted identity to this same live page
  without changing its composition root or reopening paint authority.

This historical completion standard required a reproducible human product run
and separately automated integration evidence; it did not require the
automation itself to launch the product binary. Milestone 3.10.3 supersedes
that evidence posture before 3.11 by adding the sole cumulative executable-
world target.
