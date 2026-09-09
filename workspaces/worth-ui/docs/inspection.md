# Application Inspection

## What This Feature Is

Application inspection explains prepared or active Worth UI state without
letting the caller mutate, execute, publish, or reconstruct that state. A typed
query returns a generation-bound receipt plus explicit support and relevance
posture. A completed rebind also exposes a compact terminal decision record.

## Why You Use It

- Explain which declarations, graph facts, and runtime decisions were admitted.
- Inspect a running generation without borrowing its mutable owners.
- Ask for compact evidence first and expand selected references when needed.
- Distinguish supported, diagnostic-only, deferred, unsupported, expired, and
  wrong-world requests.
- Correlate a published rebind with its exact source basis and structural work.
- Correlate a Query transition, projection fact, mounted node, frame, and
  presentation without promoting any correlation key into authority.

## Stable Entry Points

- `worth_ui::facade::inspection::UiInspectionQuery`
- `UiInspectionTarget`
- `UiInspectionScope`
- `UiEvidenceRichness`
- `UiEvidenceBudget`
- `WorthUiApp::inspect(...)`
- `WorthUiActiveApplicationSession::inspect(...)`
- `WorthUiApp::inspection_support_report_for(...)`
- `WorthUiApp::expand_evidence_ref(...)`
- `WorthUiActiveApplicationSession::lookup_intent_causal_trace(...)`
- `WorthUiActiveApplicationSession::why_portal_closed()`
- `WorthUiActiveApplicationSession::why_focus_moved()`
- `WorthUiActiveApplicationSession::why_focus_restoration_failed()`
- `WorthUiActiveApplicationSession::why_motion_interrupted()`
- `WorthUiActiveApplicationSession::why_scroll_owner()`
- `WorthUiActiveApplicationSession::why_selection_dropped()`
- `WorthUiActiveApplicationSession::why_command_won()`
- `WorthUiActiveApplicationSession::runtime_service_resource_census()`
- `UiRebindReceipt::decision_record()`
- `UiRebindReceipt::decision_index()`
- `UiRebindDecisionLookup`

## Core Mental Model

Inspection is a read-only projection. The prepared app or active session still
owns the real graph, plan, mounting, observation, rebind, and publication
state. Receipts carry identities and evidence references so results can be
explained, but those values cannot be promoted back into operational authority.

Active inspection carries the exact application-generation identity. Use it to
reject or label stale results after rebind.

A published rebind receipt projects one `UiRebindDecisionRecord`. It carries
the exact decision key, source basis, observation/fact/aspect/consumer counts,
changed-versus-evidence-only disposition, published stop point, and structural
cost. A bounded decision index reports `Found`, `Expired`, or `Unavailable`;
absence is never silently relabeled as an empty decision.

### Intent causal evidence

Each admitted semantic intent may carry one `UiIntentEvidenceReference` from
its presentation-bound interaction into routing, payload projection,
operability, admission, managed attempt, and product outcome. Resolve it with
`lookup_intent_causal_trace`. The result is explicit:

- `Found(UiIntentCausalTraceEvidence)` for the exact live session, slot, and
  generation;
- `Expired` after replacement or for a stale/wrong generation;
- `ForeignSession` when the reference names another active session.

The runtime retains at most
`UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY` (64) semantic causal records in
one fixed replacement ring. Its semantic byte bound is
`UI_INTENT_CAUSAL_TRACE_EVIDENCE_BYTE_CAPACITY`. Pointer motion and preedit
volume do not enter this ring. Admission and attempt updates use fixed
generational slot indexes; lookup does not scan runtime history.

The runtime record ends honestly at the product outcome, including whether
consequence handoff was pending at completion. Platform Pulse correlates that
owner-issued reference with consequence publication, the exact Query
projection, and the mounted publication frame. The executable client observes
the final pixel change independently; the producer does not certify its own
pixels.

These records are immutable reporting data. Copying a reference, digest, trace,
or serialized Pulse projection cannot construct an interaction, operability
proof, admitted intent, attempt, consequence, Query fact, or mounted authority.

### Runtime-service evidence

Each `why_*` method projects the latest bounded evidence owned by one service
family. Command evidence names the winning route and bounded losing candidates;
focus distinguishes movement from failed restoration; scroll names the routed
owner; selection explains a dropped key or range; and portal and motion report
their terminal causes. `runtime_service_resource_census()` reports live family,
proposal, prefix, track, and retention resources for shutdown diagnosis.
Physical focus placement has a separate `UiFocusHostPlacementShutdownReport`;
it is not represented as a service-census row.

These methods answer current developer questions. They do not return mutable
owners, proposal stages, command authority, semantic-focus authority, or
retained history. `None` means no retained summary for that question, not that
the operation succeeded.

## How It Executes

```text
typed target + scope + relevance + richness + budget
-> support and relevance admission
-> indexed read-only projection
-> compact receipt
-> optional bounded evidence expansion
```

A scope can be supported, diagnostic-only, deferred, unsupported, or valid only
in another world. Check support posture rather than treating an empty result as
success.

## Small Example

```rust
use worth_ui::facade::app::WorthUi;
use worth_ui::facade::inspection::{
    UiInspectionQuery, UiInspectionScope, UiInspectionTarget,
};
use worth_ui::facade::rebind::UiChangeProfile;

let app = WorthUi::app()
    .with_change_profile(UiChangeProfile::platform_pulse())
    .freeze()?;
let query = UiInspectionQuery::new(
    UiInspectionTarget::product_root(),
    UiInspectionScope::graph(),
);
let receipt = app.inspect(query);
```

This asks the prepared application for a graph-scoped summary. It does not
launch or mutate the application.

## Rebind Decision Example

This fragment is compiled inside the complete public program in
[Hot rebind](./hot-rebind.md#compiled-public-example).

<!-- compile-pass-fragment:inspect_rebind_decision -->
```rust
fn inspect_rebind_decision(receipt: &UiRebindReceipt) -> Option<UiRebindDecisionRecord> {
    let record = receipt.decision_record();
    match receipt.decision_index().lookup(record.key()) {
        UiRebindDecisionLookup::Found(exact) => Some(*exact),
        UiRebindDecisionLookup::Expired | UiRebindDecisionLookup::Unavailable => None,
    }
}
```

The record is a terminal projection, not retained execution authority. Dropping
the rebind receipt releases its registered terminal capacity; copyable
identities and compact records do not keep retry or recovery handles alive.

## Real Example

```rust
let query = UiInspectionQuery::new(target, scope)
    .with_richness(richness)
    .with_budget(budget)
    .with_relevance(relevance);

let support = app.inspection_support_report_for(&query);
let receipt = app.inspect(query);
if let Some(reference) = selected_evidence_ref(&receipt) {
    let detail = app.expand_evidence_ref(reference, requested_richness);
    present_detail(detail);
}
present_support(support);
```

The support report tells the UI whether the request is available, deferred, or
unsupported. Evidence expansion is bounded and still read-only.

## How It Relates To Other Features

- Inspect `WorthUiApp` before launch for prepared truth.
- Inspect `WorthUiActiveApplicationSession` for generation-bound active truth.
- Query-specific inspection can cite the exact Query attempt or settled
  projection, availability/activity/stop posture, compatibility basis, and
  shape-specific fact without copying them into UI-owned state.
- Rebind decision records cite the exact published source basis and structural
  consequence counts without exposing the plan as mutable authority.
- Visual predecessor/successor comparison uses retained snapshots plus the
  exact rebind receipt; inspection alone cannot infer that relationship.

## Inspection And Debugging

Compare the active inspection receipt's generation identity with the current
session generation before displaying long-lived results. Surface explicit
relevance, `Expired`, `Unavailable`, and support outcomes; do not collapse them
into a generic "no data."

Projection evidence should be requested in layers: compact transition/fact and
mounted correlation first, then lazy detail under an explicit evidence budget.
Disclosure and retention posture apply to Query summaries and mounted
correlations independently. Expired detail is a typed omission, not permission
to reopen Query or reconstruct a fact from a digest. Inspection evidence cannot
construct a binding or fact, even when every reporting identity matches.

For rebind, inspect the decision key and source basis before counts. Then compare
planned structural cost with the mounted and host receipts. A matching pixel or
digest cannot repair wrong generation or frame affinity.

Appearance explanations distinguish a theme-slot source (selected and terminal
slot), an authored literal, and unavailable resolution. Literal cells retain
their role, theme binding, coherent state, and mounted attribution while recording
zero theme-slot lookups. They never report a fabricated slot or alias. This
inspection applies to the staged, unpublished appearance path until cutover.

## Anti-Patterns

- Importing runtime storage, planning, mounting, or publication modules.
- Treating evidence identities or digests as constructors.
- Using inspection output to drive operational mutation.
- Reassembling a projection binding or fact from matching reporting identities.
- Requesting rich evidence globally when a compact reference is enough.
- Retaining hidden execution state so an inspection reference never expires.

## Current Limits

Not every future scope is admitted. Deferred and diagnostic-only rows are
intentional public truth, not incomplete success responses.

Compact rebind decision records are available now. Rich causal-neighborhood
materialization, intent-detail disclosure, and replay/reconstruction remain
deferred. They must extend
the inspection projection and certification authority; ordinary inspection
must not retain hidden runtime state or import replay.

## Related Docs

- [Interaction and intents](./interaction-and-intents.md)
- [Application lifecycle](./application-lifecycle.md)
- [Hot rebind](./hot-rebind.md)
- [Runtime subsystems](./runtime-subsystems.md)
- [Runtime services](./runtime-services.md)
- [Visual inspection](./visual-inspection.md)
- [Query-backed UI views](./query-binding.md)
