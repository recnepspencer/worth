# Branches and Previews

## What This Feature Is

Preview and branch sessions are isolated runtime contexts built from the same
workspace model. They let you reuse retained live, computed, and effect
surfaces while moving writes and intent-like work into preview-local or
branch-local authority lanes instead of mutating current truth directly.

## Why You Use It

- you want to try changes without touching authoritative truth
- you need isolated preview-local writes and closeout behavior
- you need branch-local intent staging that never silently executes against the
  authoritative lane

## Stable Entry Points

Stable session entry points:

- `workspace.preview(...)`
- `workspace.preview_with_options(...)`
- `workspace.branch(...)`
- `workspace.branch_with_options(...)`

Stable option constructors:

- `WorthQueryPreviewOptions::derive_only()`
- `WorthQueryPreviewOptions::muted()`
- `WorthQueryPreviewOptions::redirected_delivery()`
- `WorthQueryPreviewOptions::sandboxed_write_intent()`
- `WorthQueryBranchOptions::derive_only()`
- `WorthQueryBranchOptions::muted()`
- `WorthQueryBranchOptions::redirected_delivery()`
- `WorthQueryBranchOptions::sandboxed_write_intent()`

Ordinary preview and branch entry uses typed session labels:

- pass `WorthQuerySessionLabel`, not raw strings
- equivalent label identity re-entry stops through
  `WorthQueryStopClass::SessionLabelCollision`
- display rendering is presentation; basis admission and closeout identity use
  canonical label identity instead

Stable preview-local operations:

- bind live/computed/effect handles into a preview
- stage preview-local writes
- discard or promote a preview outcome

Stable ordinary branch-merge operations under
`worth_query::facade::workflow`:

- `declare_branch_merge(...)`
- `branch_merge(...)`
- `WorthQueryBranchMergeOutcome`
- `WorthQueryBranchMergeSettlementDeferred`
- `WorthQueryBranchMergeNextAction::RepairDeferredBranchMergeSettlement`
- `WorthQueryRuntime::repair_deferred_branch_merge_settlement(...)`
- `WorthQueryWorkspace::repair_deferred_branch_merge_settlement(...)`

Stable preview-live planning entry points live under
`worth_query::facade::policy`:

- `admit_scoped_preview_session_plan_binding(...)`
- `admit_scoped_preview_live_session_plan(...)`
- `execute_scoped_preview_live_session_plan(...)`
- `assess_preview_live_drift(...)`
- `ScopedPreviewSessionPlanBinding`
- `ScopedPreviewLiveSessionPlanBinding`

Support-gated neighbors:

- preview-local or branch-local intent execution still depends on admitted
  intent support and the correct sandboxed policy

## Core Mental Model

Preview and branch sessions are not separate products. They are lane-shifted
contexts over the same retained runtime surfaces.

Preview:

- binds existing handles into a preview lane
- keeps preview-owned active subscriptions separate from authoritative active
  subscriptions when their basis identity or checkpoint identity differs
- keeps preview-owned temporal wakes, async result-state, mixed-cause residue,
  and crossed preview or authoritative drift residue preview-local until
  discard or authoritative re-admission resolves them
- can stage preview-local writes
- can discard or promote staged work
- treats promotion as a rebinding boundary, not as structural reuse of the
  preview-owned basis
- defaults to `derive_only`

Branch:

- opens a branch-local lane
- is primarily about branch-local intent staging in the current public surface
- defaults to `derive_only`

The key idea is isolation by authority lane, not by reimplementing the whole
runtime.

Under that Query lane, Relational owns the actual branch reference. Each branch
has one mutable reference cell that selects an immutable root. An exact
reference observation records its selected target and branch-local truth
version together. Equal version numbers on different branches do not describe
the same state.

An owner-admitted branch basis pins the selected immutable root. Reads through
that basis are repeatable even when the live reference advances. A transaction
opened from the basis is detached from the runtime: it carries the exact basis,
overlay, and footprint without holding a general runtime borrow. Preparation
produces an opaque, single-use candidate. Publication compares that candidate
against only its branch cell, so unrelated branches have independent
linearization points.

Serialized basis descriptors are descriptive, not operational. Resolving one
checks its shape; only the owning Relational runtime can readmit it against the
live reference, retained root, lifecycle posture, and runtime identity. Query
consumes those admitted owner products through its facade rather than
reconstructing branch authority from IDs or digests.

For preview-live planning, isolation is also basis-scoped. A preview plan
binding and a scoped observation basis must agree structurally before Query
mints `ScopedPreviewSessionPlanBinding`. Live admission then consumes that
binding and returns `ScopedPreviewLiveSessionPlanBinding`, which is the only
artifact accepted by scoped execution and drift assessment. The sealed binding
keeps preview identity, live-plan identity, and basis authority together.

Graph touch obligations follow that same isolation rule. Branch and preview
graph touches must carry an operating world descriptor, and Query must select
registered obligations from that descriptor rather than pretending branch-local,
preview-local, and authoritative graph work share one validator table.

## How It Executes

Preview path:

1. Open a preview session from the workspace.
2. Bind live, computed, and optional effect handles.
3. Stage preview-local writes or preview-local intent work, depending on
   policy and support.
4. Discard or promote the preview.

Preview-live planning path:

1. Produce a preview session plan binding from an admitted preflight and active
   preview context.
2. Scope the matching observation world through `basis_lifecycle()`.
3. Admit both into `ScopedPreviewSessionPlanBinding`.
4. Admit the live plan into `ScopedPreviewLiveSessionPlanBinding`.
5. Execute or assess drift with that sealed scoped artifact.

Branch path:

1. Open a branch session from the workspace.
2. Use the declared branch effect policy.
3. Stage branch-local intent work when support and policy admit it.

An admitted Relational-backed mutation then follows a stricter owner path:

1. Observe or readmit one exact branch basis.
2. Open a detached transaction from that basis and stage work.
3. Validate and prepare one opaque branch-bound candidate without moving the
   reference.
4. Compare-and-publish under that branch's bounded critical section.
5. Settle durability and refresh any required Query indexes.

The performed and settled boundaries are distinct. If branch movement succeeds
but settlement fails, the operation returns typed settlement recovery. It must
not rerun the mutation or merge.

`derive_only` is the default for both preview and branch sessions. That means
derived behavior is allowed, but delivery and write-intent work are denied or
muted unless you explicitly choose a broader policy.

## Small Example

```rust
use worth_query::facade::runtime::{WorthQueryPreviewOptions, WorthQuerySessionLabel};

let mut workspace = runtime.workspace("preview").unwrap();
let label = WorthQuerySessionLabel::scoped_strs("workflow", ["draft-create"]).unwrap();

let mut preview = workspace
    .preview_with_options(
        label,
        WorthQueryPreviewOptions::sandboxed_write_intent(),
    )
    .unwrap();

preview
    .insert("Task", |task| {
        task.aspect("identity.id", "preview-1")
            .aspect("title.value", "Preview-only task")
    })
    .unwrap();

let outcome = preview.discard();
```

This is the smallest honest example because it shows preview-local staging with
an explicit closeout result instead of pretending preview writes are ordinary
truth writes.

## Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryBranchOptions, WorthQueryInspection, WorthQueryLiveView,
    WorthQueryPreviewOptions, WorthQuerySessionLabel,
};
use serde_json::json;
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("workflow").unwrap();
let preview_label =
    WorthQuerySessionLabel::scoped_strs("workflow", ["preview-execution"]).unwrap();
let branch_label =
    WorthQuerySessionLabel::scoped_strs("workflow", ["branch-local-intent"]).unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.preview-bind", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-preview")
    })
    .unwrap();

let preview_outcome = {
    let mut preview = workspace
        .preview_with_options(
            preview_label,
            WorthQueryPreviewOptions::redirected_delivery(),
        )
        .unwrap();
    preview.use_view(&live);
    preview
        .insert("Task", |task| {
            task.aspect("identity.id", "preview-task")
                .aspect("title.value", "Preview execution task")
        })
        .unwrap();
    preview.discard()
};

let branch_result = {
    let mut branch = workspace
        .branch_with_options(
            branch_label,
            WorthQueryBranchOptions::sandboxed_write_intent(),
        )
        .unwrap();
    branch.execute_intent(worth_query::facade::runtime::WorthQueryIntentDeclaration::strategy_commit(
        "branch-reconcile",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        json!({ "entity": "task-1", "title": "branch title" }),
    ))
};
```

An ordinary branch merge handles performed-but-unsettled publication through
the workspace that owns the Relational runtime:

```rust
use worth_query::facade::workflow::{
    branch_merge, declare_branch_merge, WorthQueryBranchMergeNextAction,
    WorthQueryBranchMergeOutcome,
};

let declaration = declare_branch_merge("main", "candidate")?;
let context = branch_merge(&workspace, &declaration)?;

match declaration.using(context).run(&mut workspace) {
    WorthQueryBranchMergeOutcome::Completed(completion) => {
        inspect_merge(completion);
    }
    WorthQueryBranchMergeOutcome::SettlementDeferred(deferred) => {
        assert_eq!(
            deferred.next_action(),
            WorthQueryBranchMergeNextAction::RepairDeferredBranchMergeSettlement,
        );
        let receipt = workspace
            .repair_deferred_branch_merge_settlement(&deferred)?;
        record_repaired_merge(receipt);
    }
    other => handle_merge_stop(other),
}
```

The deferred carrier hides the raw Relational settlement. Repair with a foreign
runtime is rejected, repair with the owning workspace is idempotent, and a
subsequent public operation can proceed after repair.

When a preview also carries a live plan, keep basis admission explicit:

```rust
use worth_query::facade::{
    foundation::basis_lifecycle,
    policy::{
        admit_scoped_preview_live_session_plan,
        admit_scoped_preview_session_plan_binding,
        execute_scoped_preview_live_session_plan,
    },
};

let scoped_basis = basis_lifecycle()
    .current_head()
    .observe()?;
let scoped_binding = admit_scoped_preview_session_plan_binding(
    scoped_basis,
    preview_binding,
)?;
let preview_live =
    admit_scoped_preview_live_session_plan(scoped_binding, live_plan)?;
let execution = execute_scoped_preview_live_session_plan(&preview_live)?;
```

Here `preview_binding` and `live_plan` are admitted planning artifacts produced
earlier in the preview pipeline. Consumer code moves the scoped binding
forward; it does not pass the basis digest and preview identity separately.

If you are authoring workflow capability evidence about a preview session, that
is a separate typed identity lane. Opening the preview uses
`WorthQuerySessionLabel`; binding workflow preview inspection or mutation
planning uses `worth_query::facade::runtime::BridgePreviewSessionIdentity`.
Do not collapse those two roles into one raw string.

Preview binding and preview outcome inspection stay on the
`WorthQuerySessionLabel` lane too. Use rendered labels for DX only; if replay
identity or collision posture matters, keep the typed session-label artifact.

What is authoritative:

- the workspace's current truth lane

What is isolated:

- preview writes land in `PreviewTruth`
- branch-local admitted intents land in `BranchLocalTruth`

What gets retained:

- binding evidence
- execution evidence
- closeout evidence
- preview or branch-local intent receipts when admitted
- exact branch observations and immutable-root retention for admitted
  Relational bases
- opaque settlement recovery when branch movement performed but durability did
  not settle

What closeout means:

- `discard()` proves staged work did not leak into authoritative truth
- `discard()` also retains preview-owned temporal, async, and mixed-cause
  residue counts so the runtime can prove what stayed preview-local
- `promote()` attempts an authoritative handoff, records a rebinding digest,
  and can fail typed and early when crossed preview residue requires
  authoritative re-admission first

## How It Relates To Other Features

- Use [Live Views](../runtime-surfaces/live-views.md) and [Computed](../runtime-surfaces/computed.md) as the
  retained handles you bind into previews.
- Use [Writes and Intent Boundaries](../execution/writes-and-intents.md) when deciding
  whether work belongs in direct writes, preview-local staging, or branch-local
  intent paths.
- Use [State and Readiness Surfaces](state.md) when you need typed posture
  snapshots around stable versus deferred families.

Preview and branch sessions are lane-control features, not alternate truth
engines.

## Inspection And Debugging

The runtime can explain:

- preview handle binding evidence
- preview execution evidence
- preview outcome and closeout residue
- preview-local and branch-local intent receipts

Look for:

- effect policy
- source lane versus target lane
- residue class counts
- temporal wake, async result-state, mixed-cause, and crossed-authoritative
  residue counts on preview closeout
- active checkpoint identity and future-bearing lane posture when a preview is
  bound to temporal or async live meaning
- rebinding digests on successful promotion
- basis snapshot tokens, rebinding digests, recovery posture, and denial
  digests on failed promotion
- branch-merge settlement message, counters, and `next_action()` without raw
  settlement access

## Anti-Patterns

- Treating preview writes as if they are already authoritative.
- Assuming `derive_only` allows delivery or write-intent work.
- Expecting branch-local or preview-local intents to bypass support admission.
- Executing or drift-checking preview-live work from an unscoped plan binding.
- Reconstructing preview-live authority from a basis digest, preview label, and
  live-plan digest.
- Treating preview or branch sessions as separate domain runtimes rather than
  authority-lane shifts over retained surfaces.
- Reconstructing a live branch basis from branch ID, commit ID, version, or a
  serialized descriptor instead of owner readmission.
- Assuming a reference-selected read follows later branch movement; an admitted
  basis is repeatable and remains pinned to its immutable root.
- Retrying a merge after `WorthQueryBranchMergeOutcome::SettlementDeferred` or
  reaching for the raw Relational repair capability.

## Current Limits

- Preview and branch sessions are stable as runtime-backed isolation contexts.
- Preview-local writes, binding, discard, and typed promotion denials are part
  of the current surface.
- Temporal and async preview subscriptions are runtime-backed active objects,
  not wrapper-only observers, and they do not share authoritative active state
  when basis identity differs.
- Preview closeout now preserves preview-owned temporal, async, and mixed-cause
  residue explicitly, and promotion denies with typed rebinding recovery
  posture when crossed preview residue cannot be promoted honestly.
- Preview-local and branch-local intent work still depends on admitted intent
  support and the correct sandboxed policy.
- Relational branch-local MVCC, exact-basis readmission, detached transactions,
  and typed branch-merge settlement repair are implemented owner capabilities.
  They do not make every Query branch-session effect family supported.
- Signal owner services provide exact component bases and per-branch progress.
  [Runtime World](../../../../../../crates/worth-runtime-world/README.md) implements
  memory-resident composite history and coordinated Relational-plus-Signal
  publication in milestone 9.17.2. Query carriage, outbox gating, and public
  product-branch integration remain milestone 9.17.3 work; existing Query branch
  sessions do not acquire those guarantees merely because the owner exists.
- Durable preview replay and temporal/async branch-session behavior remain
  future work.

## Related Docs

- [Graph Touch Obligation Authority](../authoring/graph-touch-obligation-authority.md)
- [Workspace Overview](workspace-overview.md)
- [Writes and Intent Boundaries](../execution/writes-and-intents.md)
- [State and Readiness Surfaces](state.md)


