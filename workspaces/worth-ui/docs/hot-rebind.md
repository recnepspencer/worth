# Hot Rebind

## What This Feature Is

Hot rebind turns a settled source edit or another admitted observation into one
bounded successor of the running Worth UI application. It classifies semantic
change, resolves only declared consumers, decides mounted identity lifecycle,
compiles an immutable plan, performs governed host effects, and publishes
atomically.

Rebind is not a file-watcher callback that swaps application state. The active
session remains the only publication owner, and the predecessor stays current
until the successor is fully admitted and presented.

## Why You Use It

- Apply watched `.wui` edits without launching a second application session.
- Preserve the last published generation after malformed or inadmissible edits.
- Recompute only consumers named by produced-fact and consumed-fact contracts.
- Distinguish no change, evidence-only succession, bounded change, rejection,
  in-flight effects, and uncertain effects.
- Retain exact retry or recovery authority instead of guessing what the host
  did.

## Stable Entry Points

The ordinary source-edit route is:

- `WorthUiFilesystemSourceWatcher`
- `WorthUiSettledSourceSnapshot`
- `UiSourceRebindRequest`
- `WorthUiNativeApplicationShell::begin_source_rebind(...)`
- `WorthUiNativeSourceRebindDenial`
- `WorthUiNativeManagedSourceRebindOutcome`
- `WorthUiNativeManagedRebindProgress`

Framework integrations that already own admitted observation evidence may use
the advanced active-session progression:

- `WorthUiActiveApplicationSession::begin_observation_turn()`
- `UiObservationTurn::admit_source(...)` and the owner-specific admission
  methods
- `WorthUiActiveApplicationSession::classify_observations(...)`
- `WorthUiActiveApplicationSession::resolve_affected_scope(...)`
- `UiResolvedAffectedScope::resolve_identity_lifecycle()`
- `WorthUiActiveApplicationSession::compile_rebind_plan(...)`
- `WorthUiActiveApplicationSession::compile_preservation_rebind(...)`
- `WorthUiActiveApplicationSession::prepare_rebind(...)`
- `UiPreparedRebind::execute(...)`

Application code should prefer `begin_source_rebind`. The advanced progression
exists for framework-owned observation families, not to rebuild the ordinary
source bridge by hand.

## Core Mental Model

The progression is compiler-visible:

```text
settled source snapshot or owner-specific observation
-> admitted observation set
-> semantic classification
-> affected consumers from declared fact contracts and indexes
-> identity lifecycle
-> immutable rebind plan
-> final admission and reserved resources
-> host effects
-> one atomic successor publication
```

Each stage consumes an owner-issued value and returns the only value accepted
by the next stage. Identities, digests, inspection receipts, raw host events,
and captured pixels cannot construct governed stages.

Observation families retain their own ordering and loss laws. Source revisions,
host viewport/device-scale reports, solicited measurements, Query
consequences, committed scroll extents, and committed portal anchors do not
enter a universal event bag. Classification produces `Changed`,
`EvidenceOnly`, or `ObservedNoChange` from sealed semantic evidence, never from
pixel equality.

## How It Executes

For a watched file edit:

1. The production watcher settles one exact filesystem revision.
2. `begin_source_rebind` compiles that held snapshot once against the active
   capability basis.
3. The active session admits the resulting source observation, classifies it,
   resolves affected scope and identity lifecycle, and compiles the plan.
4. Final admission checks predecessor/source affinity, deadline,
   cancellation, concurrency, resources, and policy.
5. The prepared rebind presents through the canonical host contract.
6. Only complete effects publish the successor generation and mounted frame.

A malformed snapshot returns `WorthUiNativeSourceRebindDenial::Source` with its
compile report and exact revision affinity. It does not change the current
generation. A duplicate or historical observation is a typed terminal outcome
before effects.

## Compiled Public Example

This complete example is compiled by the existing two-session Worth UI
compile-contract runner. The first function is the ordinary source bridge. The
second shows the advanced owner-issued planning progression.

<!-- compile-pass-source:tests/ui/rebind/pass/rebind_phase_progression_uses_owner_issued_values.rs -->
```rust
use worth_ui::facade::app::{
    WorthUiActiveApplicationSession, WorthUiNativeApplicationShell,
    WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial,
};
use worth_ui::facade::inspection::{UiRebindDecisionLookup, UiRebindDecisionRecord};
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindPlanningDenial, UiRebindReceipt,
    UiSourceRebindRequest,
};
use worth_ui::facade::source::WorthUiSettledSourceSnapshot;

fn begin_settled_source_rebind(
    shell: &mut WorthUiNativeApplicationShell,
    snapshot: WorthUiSettledSourceSnapshot,
    now_tick: u64,
) -> Result<WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial> {
    let request = UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(now_tick.saturating_add(1)))
        .observed_at_tick(now_tick);
    shell.begin_source_rebind(request)
}

fn inspect_rebind_decision(receipt: &UiRebindReceipt) -> Option<UiRebindDecisionRecord> {
    let record = receipt.decision_record();
    match receipt.decision_index().lookup(record.key()) {
        UiRebindDecisionLookup::Found(exact) => Some(*exact),
        UiRebindDecisionLookup::Expired | UiRebindDecisionLookup::Unavailable => None,
    }
}

fn compile_owner_issued_change(
    session: &WorthUiActiveApplicationSession,
    outcome: UiChangeClassificationOutcome,
) -> Result<(), UiRebindPlanningDenial> {
    match outcome {
        UiChangeClassificationOutcome::Changed(change) => {
            let scope = session
                .resolve_affected_scope(change)
                .expect("owner-issued change resolves");
            let lifecycle = scope
                .resolve_identity_lifecycle()
                .expect("resolved scope advances one phase");
            session
                .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
                .map(|_| ())
        }
        UiChangeClassificationOutcome::EvidenceOnly(evidence) => session
            .compile_preservation_rebind(evidence, UiRebindExecutionPolicy::ordinary())
            .map(|_| ()),
        UiChangeClassificationOutcome::ObservedNoChange(_) => Ok(()),
    }
}

fn main() {
    let _ = (
        begin_settled_source_rebind,
        inspect_rebind_decision,
        compile_owner_issued_change,
    );
}
```

The shell owns incomplete work. A caller receives `Pending` and must advance it
through `progress_managed_rebind(...)` or `retry_managed_rebind(...)`; it cannot
detach a completion or retry authority and publish outside shell reconciliation.

## Outcomes And Recovery

`WorthUiNativeManagedSourceRebindOutcome` distinguishes immediate `Published`,
owned `Pending`, and terminal `Stopped`. Managed progression then distinguishes:

- `Published` for one complete atomic successor;
- `Stopped` with the exact no-effect or denial reason while the predecessor
  remains current truth;
- `AwaitingProgress` while the shell owns completion, retry, reconstruction, or
  indeterminate recovery authority; and
- typed internal-defect posture when realized effects contradict the plan.

Retry and recovery run through the shell's retained exact authority. Shutdown
disposes that authority through the same owner. The runtime never labels
uncertain native state as rolled back.

## Facts, Scope, And Identity

Produced facts retain an owner and family. Consumers declare which fact
selectors and aspects they consume. Derived indexes map those contracts to
affected graph consumers; deleting an index must permit exact reconstruction
from sealed application truth without rereading source.

Identity lifecycle is separate from semantic classification. A changed fact
may preserve, create, retire, rebind, move, or remount individual identities.
Unrelated instances remain outside the plan. Evidence-only succession advances
authored truth without inventing physical work.

When adding a new observation family:

1. give its source owner a typed evidence value and ordering law;
2. add the produced-fact family and explicit consumed-fact contract;
3. place its derived index under the owner that can rebuild it;
4. extend the independent change/order models and mixed-source tests; and
5. feed the existing planner and canonical executor.

Do not add a parallel executor, a generic event payload, or a session-owned
catch-all map.

## Cost And Capacity

Post-classification work is accounted as:

```text
O + F + A + C + R + G + M + B
```

where observations, produced facts, affected consumers, conflicts, resets,
graph work, mounted work, and retained bookkeeping are counted separately.
Source acquisition and DSL compilation are reconstructive source cost.
Presentation and reconciliation are physical host cost. Rich diagnostics are
materialized only when requested.

Profiles bound observations, plans, fan-out, effects, retained records,
completion handles, and recovery handles. Saturation returns typed denial or
backpressure; it must not silently widen to a whole-graph scan or remount.
Unchanged frames perform no new rebind work.

## Inspection And Debugging

Start with the exact source revision and active generation. Then inspect:

- classification and produced-fact references;
- affected-scope basis, selected consumers, and scope cost;
- identity decisions and plan basis;
- stopped phase, denial cause, and valid next action;
- planned versus realized effects;
- mounted predecessor/successor frame identities; and
- shutdown counts for plans, completions, recovery, and retained evidence.

Inspection remains read-only. It can explain these references but cannot turn
them back into planning or execution authority. See
[Application inspection](./inspection.md).

## Appearance And Content Cutover

The replacement path prepares semantic content, per-occurrence geometry,
owner-state succession, theme resolution, appearance, accepted Motion samples,
relational Portal/Backdrop order, and retention before presentation. The
presenter accepts only that completed phase. Assembly alone cannot reach the
host.

Publication settles per mounted surface. If one surface accepts and another is
rejected, the exact content revision remains pending for the rejected surface.
A later retry rebases the retained prepared evidence and cannot count a newly
mounted occurrence as stale coverage. Returning to an omitted surface uses
reconstruction and its current semantic provenance. Acceptance commits the
same prepared owner lifecycle that produced the pixels; rejection and
cancellation retain predecessor paint and state.

## Anti-Patterns

- Publishing directly from a watcher or compiler callback.
- Rereading the latest file after a snapshot has settled.
- Treating pixel equality as semantic preservation.
- Widening one fact to every graph node or remounting every surface.
- Reconstructing a plan from inspection output or identities.
- Retrying an outcome without its returned typed authority.
- Dropping uncertain effects and reporting rollback.
- Charging source compilation or rich diagnostics to steady-frame execution.

## Current Limits

Hot rebind compiles already-authored, currently supported Worth UI meaning. It
coordinates the current projected content, service, appearance, theme, Motion,
and Portal owners through their typed preparation and settlement boundaries; it
does not absorb their authority. Expressions and modules remain outside the
current authoring surface. None of these features relocates source settlement,
publication, or host authority.

## Related Docs

- [Application lifecycle](./application-lifecycle.md)
- [Authored composition](./authored-composition.md)
- [Application inspection](./inspection.md)
- [Visual inspection](./visual-inspection.md)
- [Worth UI architecture](./architecture.md)
- [Runtime subsystems](./runtime-subsystems.md)
