# Runtime Services

## What This Feature Is

Runtime services give a Worth UI application production owners for portals,
keyboard focus, motion, command shortcuts, scrolling, and selection. Use them
when behavior must remain correct across native input, hot rebind, presentation,
and shutdown. Application code declares policies and intent destinations; it
does not create mutable service objects or host callbacks.

## Why You Use It

- Open an anchored dropdown and restore focus when it closes.
- Route a typed shortcut by the current portal or focused-control context.
- Keep scroll offsets and selected item keys stable across a source rebind.
- Animate from committed predecessor geometry while hit testing the geometry
  currently on screen.
- Explain a service decision and prove that all live resources reached zero.

## Stable Entry Points

Application code starts from these public surfaces:

- `worth_ui::facade::service` exports the six policy families, typed shortcut
  vocabulary, command route declarations, and normalized policy plan.
- `WorthUi::app()` returns the application builder. Its
  `with_*_policy_defaults(...)` methods configure demanded service families.
- `UiIntentDefinition::<I>::runtime_service(...)` and
  `register_runtime_service_intent_definition(...)` register the explicit
  intent destinations `OpenPortal`, `ClosePortal`, and `InvokeCommand`.
- `CommandDescriptor::with_default_shortcut(...)` and
  `CommandDescriptor::with_intent_destination::<I>()` declare typed command
  routing.
- `WorthUiHostNeutralApp::service_policy_plan()` exposes the normalized,
  read-only installation plan before launch.
- `WorthUiActiveApplicationSession::why_*` and
  `runtime_service_resource_census()` expose bounded inspection after launch.

The six mutable owners and `UiServiceProposalCompiler` are runtime internals.
They are not application construction surfaces. `UiRuntimeServiceFamily` is
classification vocabulary inside the runtime, not a public payload enum.

The WUI language also accepts `portal`, `focus`, `motion`, `command`, `scroll`,
and `selection` declarations. Invalid clauses are rejected with a source span,
the service law that failed, and a lawful repair.

## Core Mental Model

Each family owns a different truth:

| Family | Authoritative meaning | Common mistake |
| --- | --- | --- |
| Portal | logical open/close state, layer, placement, and dismissal | treating a host popup or painted rectangle as the portal |
| Focus | semantic keyboard focus, scopes, modality, and restoration | treating window activation as semantic focus |
| Motion | committed track meaning from exact predecessor and successor receipts | treating the current sample or host interpolation as layout truth |
| Command routing | typed shortcut match and winner over declared context | matching localized shortcut text or registration order |
| Scroll | semantic offset, nested routing, bounds, and anchoring | treating a Query cursor or renderer transform as offset authority |
| Selection | selected stable application item keys and range posture | retaining a row index or Query identity as the item key |

When one operation involves more than one family, owners do not call each
other. Each owner stages its own proposed change. The proposal compiler checks
that every proposal describes the same application generation, surface,
presentation, causal operation, and budget, then orders the sealed family
stages. Existing application and mounted publication either accept the whole
batch or reject it. The compiler owns no service state and cannot publish or
settle a physical effect.

Geometry also carries its phase in the type. Committed layout geometry answers
where the next layout intends to be. Presentation-sampled geometry answers
where pixels and hit targets are now. Host coordinates and physical pixels are
separate again. Do not substitute one because its rectangle has equal numbers.

Motion opacity is quantized by the presentation sampler into `u16` units, with
65,535 representing one and ties rounded to even. Sample receipts, retained
track state, retarget starts, and certification observations preserve those
exact units. Presentation accepts the canonical result without a second float
conversion. This raw Motion factor remains distinct from semantic appearance
opacity; an appearance-composed value must never become a retarget predecessor.

For independently mounted collection items, call
`bind_selection_item(owner_receipt, item_receipt, current_option)` before
SelectionCommit or Selection-driven appearance. Both receipts must be current
and on the same semantic surface. The owner must declare the corresponding
collection content or exact SelectionCommit projection payload. The option
carries the original application item key. A second mounted incarnation of
the same Selection owner cannot replace an existing owner's bindings.

Binding establishes provenance, not selection. Selection remains the sole owner
of selected keys, anchor, and cursor. Appearance and interaction consume the
same binding. Reorder refreshes surviving exact row/key bindings; retirement,
removal, suspension, and graph replacement revoke obsolete bindings. Changing
an item's key requires retiring its mounted identity. Rows drawn inside one
collection node are not independently mounted items.

Initial allocation advances the prepared graph generation. Portal admits that
succession before allocation work and commits it after successful activation,
preserving declared surface bindings, live Portal rows, and stack order. A
failed preparation leaves those bindings on their predecessor generation.

The staged appearance mounting path distinguishes that generation change from
unmounting. Expired semantic projections leave physical-only predecessors for
still-mounted instances, preserving their exact session, incarnation, receipts,
and capacity. Fresh resolution may replace those physical facts but cannot use
the expired projection as semantic authority. Graph replacement carries these
predecessors independently of current projection authority. Candidate mounting
checks attachment membership against its exact graph generation: removing a role
retires its mechanics in that replacement even when the node survives. Explicit
unmount produces removal work and damage from the old visual bounds, including
when the successor frame has no nodes. Completed surface deregistration discards
that surface's physical facts while preserving other surfaces. Aggregate admission
accepts replacements and removals together; a denied candidate preserves both for
retry. Reconstruction after generation expiry requires a real current-owner
observation close. That close queues the existing canonical initial selection;
reconciliation consumes its resolver results once while rebuilding the remaining
current projections. Reconstruction without refreshed semantic provenance denies
and preserves the physical predecessors. This remains unpublished until the
appearance protocol cutover.

Physical outline lowering uses the admitted host appearance profile and each
prepared surface binding's device scale to select the qualified anti-alias
fringe. The same qualification applies during reconstruction. Missing profiles
or unqualified scales deny outline output; they never become a zero-fringe
approximation. Outline damage includes the full expanded visual bounds.

## How It Executes

An explicit user operation follows the ordinary intent path:

```text
native observation
-> presented semantic interaction
-> route, payload, operability, and UI admission
-> OpenPortal | ClosePortal | InvokeCommand destination
-> family-owned request and staged successor
-> proposal compilation when other families participate
-> existing atomic application + mounted publication
-> existing presentation and host settlement
```

Other service work keeps its real origin. A window-focus observation, scroll
delta, clock tick, reduced-motion change, hot rebind, portal dismissal, focus
restoration, or motion continuation does not pretend to be a managed intent.
High-frequency scroll deltas and motion samples use bounded compact lanes; they
do not allocate one intent attempt per delta or frame.

A physical semantic-focus placement has four observable stages:

```text
prepared before effect
-> issued and awaiting acknowledgement
-> settled terminal
or indeterminate and reconciliation required
```

Silence and timeout are never success. Reconciliation consumes current host
truth and does not replay the semantic request. Focus navigation and placement
use mounting's current physical presentation epoch, including accepted Motion
samples. Historical frame-publication receipts remain structural evidence.
Observation admission validates host surface, binding, and epoch together before
updating owners. Tab and container navigation preserve the admitted semantic
surface; an input from another surface cannot navigate the current container.

## Small Example

Policy defaults do not install owners by themselves. This exact fragment is
compiled inside `unused_policy_defaults_install_no_service_family`; the fixture
provides its imports and surrounding test function.

<!-- compile-pass-fragment:runtime-service-unused-defaults -->
```rust
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_portal_policy_defaults(UiPortalPolicy::modal_dialog())
        .with_focus_policy_defaults(UiFocusPolicy::workbench())
        .with_motion_policy_defaults(UiMotionPolicy::system_respecting())
        .with_command_routing_policy_defaults(UiCommandRoutingPolicy::desktop())
        .with_scroll_policy_defaults(UiScrollPolicy::nested_region())
        .with_selection_policy_defaults(UiSelectionPolicy::multiple())
        .freeze()
        .expect("policy defaults alone do not demand runtime owners");

    assert_eq!(app.service_policy_plan().installed_family_count(), 0);
```

The executable counterpart is
[`service_policy_facade.rs`](../crates/worth-ui/tests/service_policy_facade.rs).
A service owner is installed only when declarations or capabilities demand it.
For example, an open-portal intent demands Portal plus its Focus and Motion
requirements, and the Scroll owner that owns the focus reveal those transitions
may emit. A scrolling Mosaic region additionally registers real scroll owners
with it.

## Real Example

This exact fragment is compiled inside
`installed_command_family_exposes_only_its_normalized_policy`. The fixture
defines `CommandIntent` and supplies the imports and surrounding test function:

<!-- compile-pass-fragment:runtime-service-command-installation -->
```rust
    let custom = UiCommandRoutingPolicy::desktop().with_repeat_suppression(false);
    let shortcut = UiCommandShortcutSequence::single(UiCommandShortcutStroke::logical(
        UiCommandKeyCode::S,
        UiCommandModifierSet::none().with_primary(),
    ));
    let app = WorthUi::app()
        .with_change_profile(UiChangeProfile::platform_pulse())
        .with_portal_policy_defaults(UiPortalPolicy::modal_dialog())
        .with_command_routing_policy_defaults(custom)
        .register_command(
            CommandDescriptor::new(
                CommandId::new("service.policy.command").expect("fixture command ID"),
                "Run command",
            )
            .with_default_shortcut(shortcut)
            .with_intent_destination::<CommandIntent>(),
        )
        .register_runtime_service_intent_definition(worth_ui::facade::intent::UiIntentDefinition::<
            CommandIntent,
        >::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        ))
        .expect("command runtime service registers")
        .freeze()
        .expect("installed command policy normalizes");
    let plan = app.service_policy_plan();

    assert_eq!(plan.command_routing(), Some(custom));
    assert_eq!(plan.installed_family_count(), 1);
    assert_eq!(plan.portal(), None);
```

The complete compile-checked source is
[`service_policy_facade.rs`](../crates/worth-ui/tests/service_policy_facade.rs).

The full intent/provider relationship is compiled separately in
[`typed_intent_relationships.rs`](../crates/worth-ui/tests/ui/facade/intent/pass/typed_intent_relationships.rs),
while the command-service composition, launch, inspection, and cleanup use the
production composition root in `service_policy_facade.rs`.
The route receipt still crosses payload projection, operability, UI admission,
and managed execution. If the command requests Query work, Query performs a
separate admission; command success is never Query authority.

The equivalent WUI service declarations are:

```wui
portal completion_menu {
  anchor editor_input
  layer transient
  dismiss escape outside_press accepted_selection anchor_gone
  focus first_enabled restore
  motion system_popover
}

selection results_selection {
  mode multiple
  identity result_key
  preserve stable_key
}

command show_palette {
  shortcut Primary+Shift+P
  scope application
}
```

This exact source is compiled by
[`phase8_service_declaration_tests.rs`](../crates/worth-ui-dsl/src/source/tests/phase8_service_declaration_tests.rs).

## How It Relates To Other Features

- [Interaction and intents](./interaction-and-intents.md) owns explicit user
  admission before an intent-origin service request.
- [Hot rebind](./hot-rebind.md) supplies each owner a current successor basis;
  equal-looking declarations do not bypass incarnation and policy checks.
- [Query-backed UI views](./query-binding.md) may supply content or extent
  evidence. It never supplies service authority, scroll offset, or selection
  identity.
- [Native host platform](./native-host-platform.md) reports mechanics and
  performs the narrow semantic-focus effect. Portals remain ordinary mounted
  overlays, and motion remains runtime-sampled presentation.
- Milestone 3.16 may consume service postures as appearance inputs. It may not
  read service internals or create a second state lane.

## Inspection And Debugging

Call the method that matches the developer question:

- `why_portal_closed()`
- `why_focus_moved()`
- `why_focus_restoration_failed()`
- `why_motion_interrupted()`
- `why_scroll_owner()`
- `why_selection_dropped()`
- `why_command_won()`
- `runtime_service_resource_census()`

Each summary names its family, owner where relevant, revision, bounded result,
and materialization cost. Command evidence includes the winning route and a
bounded list of losing candidates with reasons. These values explain current
owner evidence; they cannot reopen a portal, move focus, reroute a command, or
reconstruct expired state.

At shutdown, the census must reach zero across family records, proposal
occupancy, command prefixes, motion tracks, and portal exit retention. Physical
focus placement is tracked separately by `UiFocusHostPlacementShutdownReport`,
whose abandoned-indeterminate-request field exposes unsettled host work. A
nonzero census row or abandoned host request identifies lifecycle work; do not
hide either by dropping an inspection projection.

Common typed failures preserve the predecessor and name the rejecting owner:

- a stale generation, surface, presentation, causal operation, or budget stops
  proposal compilation before publication;
- an unsupported or ambiguous shortcut, scope, target, or service declaration
  stops at declaration/admission rather than choosing a fallback;
- a physical focus effect rejected before issue leaves semantic focus
  inspectable, while an indeterminate issued effect requires host-truth
  reconciliation;
- a rebind that removes an anchor, participant, command route, scroll owner, or
  selection key follows that family's explicit rebind law;
- a stale two-stroke shortcut prefix is discarded rather than consuming the
  stroke that discovered it: that stroke resolves as a fresh first stroke;
- a portal transition whose target still owns physically pending exit work is
  refused before effect instead of displacing that pending settlement; and
- saturation stays bounded and observable. It does not trigger a global scan
  or unbounded evidence retention. A terminally closed portal leaves the live
  table, so dismissal, placement, and command-routing work stays proportional
  to the currently active portals rather than to session history.

## Anti-Patterns

- Creating an application-local service owner or generic `ServiceManager`.
- Calling Focus from Portal or Scroll from Focus instead of emitting a typed
  requirement for proposal compilation.
- Treating `UiRuntimeServiceFamily` as a payload switch for family behavior.
- Matching a shortcut display string, raw key text, or registration order.
- Using a Query cursor as scroll offset or Query identity as selection key.
- Using committed target geometry for current hit testing during motion.
- Replaying a semantic request or guessing success after an uncertain host
  effect.
- Retaining every scroll delta, motion sample, candidate, or historical state.
- Treating a `why_*` summary, digest, or census row as operational authority.

## Current Limits

- Portals are same-surface mounted overlays. Operating-system popup windows,
  native menus/dialogs, notifications, and detached document windows are
  unsupported.
- Command routing supports typed single-stroke and ordinary two-stroke
  sequences. User-editable keymaps and arbitrary shortcut scripting are not
  shipped. `active_region` scope is source-linked but rejected until its
  runtime authority exists; use application, surface, focused-control, or
  active-portal scope.
- Scroll accepts host momentum deltas but does not implement custom kinetic or
  overscroll physics. Motion does not expose spring solvers or host timelines.
- Selection supports stable-key set and contiguous-range behavior, not spatial
  lasso or two-dimensional canvas selection.
- Accessibility focus is an explicit unsupported integration point for the
  later accessibility milestone; it is not a second live focus tree.
- Service state is not durably persisted. Undo, redo, history capture, and
  `provisional_aftermath` are not runtime-service capabilities.
- Hover, pressed, selected, focused, disabled, and validation-bearing visual
  treatment belongs to the appearance milestone unless a current mounted fact
  already supplies that exact meaning.

## Extension Points And Cutover

Add a new policy or request to the owning family, then expose only its
declarative application surface through `worth_ui::facade::service`. If work
must coordinate with siblings, add a typed requirement and sealed proposal
stage; do not add a cross-family call or a generic service manager. New host
mechanics extend the host contract. New Query work extends the existing Query
declaration/host audience and performs its own admission. New inspection is a
bounded projection from owner evidence, never a second authority lane.

Milestone 3.15 removes the placeholder cutover path instead of preserving
compatibility aliases. The unsupported runtime-service execution binding and
its unsupported-registration builder methods are gone; command shortcut text
is replaced by typed shortcut identity; and window-focus observations and
semantic-focus placement are distinct host meanings. The exact deletion
inventory, baseline evidence, pending-lane boundaries, and existing roadmap
handoff are recorded in
[Milestone 3.15 documentation closeout](../../../_docs/worth-ui/milestone-3.15-documentation-closeout.md).

## Related Docs

- [Interaction and intents](./interaction-and-intents.md)
- [Runtime subsystems](./runtime-subsystems.md)
- [Native host platform](./native-host-platform.md)
- [Application lifecycle and Platform Pulse](./application-lifecycle.md)
- [Application inspection](./inspection.md)
- [Worth UI architecture](./architecture.md)
- [Milestone 3.15 documentation closeout](../../../_docs/worth-ui/milestone-3.15-documentation-closeout.md)
