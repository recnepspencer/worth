# Inspection

## What This Feature Is

Inspection is Worth Query's explanation surface. It lets you ask the runtime
why a live view, computed view, effect, write receipt, batch write receipt,
intent artifact, preview artifact, branch artifact, or typed basis lifecycle
artifact looks the way it does without reaching through private runtime
plumbing.

This includes both:

- ordinary `workspace.inspections()?.inspect(...)` convenience inspection
- covered inspection intent families that pass through the shared intent
  admission lattice before materializing runtime-backed inspection results

## What This Is Not

- **Cross-runtime causal inspection** — construct `CausalInspection` from the
  originating receipt and a `ScopedInspectionBasis`, then use the causal
  admission/materialization lane rather than `workspace.inspections()?.inspect`.
  See [cross-runtime causal inspection](cross-runtime-causal-inspection.md) and
  [inspection vs cross-runtime explanation](../domain-capabilities/choosing/inspection-vs-cross-runtime-explanation.md).
- **Explanation contributions** — domain declaration posture in
  lower-runtime explanation contributions; does not replace inspect or
  causal inspection APIs.
- **Authoritative mutation write proof** — use
  [authoritative mutation evidence](authoritative-mutation-evidence.md) for
  bridge-backed write receipts, not inspect alone.

## Why You Use It

- you need to confirm that a live view really installed the subscription shape
  you thought it did
- you need to debug computed dependencies, pending patches, or refresh-fallback
  posture
- you need to explain effect routing, suppression, pending write-intents, or
  feedback phases
- you need a trustworthy record for preview promotion, branch-local staging, or
  intent denial behavior
- you need to confirm that a basis-sensitive path is bound to the admitted
  branch, historical snapshot, preview, or lower-runtime witness you expected

## Stable Entry Points

- `workspace.inspections()?.inspect(...)`
- `workspace.inspections()?.inspect_intent(...)`
- `workspace.inspect_derived_intent(...)`
- `WorthQueryInspection`
- `WorthQueryInspectionTarget`
- `WorthQueryBasisLifecycleInspection`

The inspection family is part of the stabilized public runtime facade. It is
safe to build against now for synchronous runtime-backed surfaces.

## Core Mental Model

Inspection does not execute the feature again. It reads retained runtime
evidence that already exists because the feature was declared, installed,
executed, or closed out.

Product inspection distinguishes **selected** truth from **current** truth. A
selected product observation retains its exact composite commit and component
bases even after that branch advances. Current inspection asks Runtime World
for the branch's present occurrence. Neither descriptive commit IDs nor
rendered basis tokens can authorize a read, publication, cleanup, or retry.

History and recovery inspection are bounded but use distinct continuation
contracts. A product-history page protects its exact commit chain and can only
continue from its owner-issued `next_parent_commit()`. A
`RuntimeWorldRecoveryCursor` resumes at the recovery catalog's next unexamined
slot, including after an empty page; the cursor is descriptive and protects no
resources. Retained observations, pending cleanup, aftermath, and recovery
carriers protect their referenced resources until released or consumed.

What you are holding:

- a typed explanation artifact chosen by the target you inspect
- evidence that is lane-aware, digest-bound, and specific to one runtime
  artifact family

Good to know:

- basis artifacts now inspect through the same `workspace.inspections()?.inspect(...)` surface
- the basis result family is `WorthQueryInspection::BasisLifecycle(...)`
- that result tells you whether the basis stayed ready, advisory, denied, or
  unsupported without making you decode raw branch, snapshot, preview, or
  lower-runtime binding identifiers yourself
- live-view inspection now retains the last delivered live cause too, including
  whether the delivery carried a relational patch or was time-only
- time-only causes stay explicit as retained delivery evidence instead of
  disappearing into patch absence or support diagnostics
- mixed-cause delivery now stays one canonical retained delivery artifact with
  ordered member kinds, coalescing posture, and preserved suppressed or denied
  cause identities instead of exposing host callback order as product truth
- async/resource-backed live inspection also retains the current async
  result-state artifact, including the typed result-state kind plus causality,
  basis, and generation digests
- live-view inspection now also retains typed remask posture when policy,
  tenant, relationship-proof, or schema context narrowed temporal/async
  runtime meaning before public projection
- ordinary live inspection now also exposes one compact runtime posture
  projection, so callers can read first-order temporal/async state through the
  same scalar surface before deciding whether they need the richer retained
  delivery or async artifacts
- temporal/async "why" questions still stay on the dedicated
  `CausalInspection` lane rather than turning `workspace.inspections()?.inspect(...)` into a
  partial causal-explanation clone
- continuation repair questions still stay on the continuation/recovery lane;
  inspection can support those artifacts, but typed async-request drift,
  replay drift, remask drift, stale completion, and preview-crossed residue
  are owned by continuation checked outcomes and recovery briefs

What the runtime keeps track of automatically:

- authority lane and basis posture
- typed basis lifecycle posture when the inspected artifact came from admitted
  Query basis capability rather than raw branch, snapshot, preview, or replay
  identifiers
- declaration digests
- support/admission evidence
- authored mutation metadata on write receipts when the write declared it
- structured declared and resolved target evidence on write receipts and batch
  components when mutation-family routing needs more than bare touched fallout
- existing-truth binding evidence on write receipts and batch components when a
  mutation intentionally targeted admitted authoritative preexisting truth
- existing-truth assertion evidence on write receipts and batch components when
  a mutation retained or backend-verified authoritative truth without mutating
  stored values
- canonical existing-truth binding digests on write receipts and aggregated
  binding digests on batch/session inspection when you need one stable session
  explanation instead of component-by-component reconstruction
- canonical naming mutation digests on batch/session inspection when one
  ordered authority session mixes attach, rebind, and remove outcomes and you
  need one stable aggregate explanation instead of re-summarizing components
- continuity-aware authority evidence on write receipts and batch components
  when an admitted update-existing mutation carries predecessor and successor
  identity through the bridge-backed authority lane
- same-batch symbolic target reference evidence on write receipts and batch
  components when an ordered batch intentionally targets truth created earlier
  in that same batch
- authoritative causality evidence on write receipts and batch components when
  the mutation crossed the bridge-backed authority lane
- authoritative provenance evidence on write receipts and batch components when
  the mutation crossed the bridge-backed authority lane
- aggregate batch mutation evidence on batch write receipts so session-wide
  target and authority breadth does not collapse into a final-component shadow
- retained counters and residue counts
- retained async result-state posture for live async/resource-backed
  subscriptions, projected from completion causality rather than app-local
  loading folklore
- unified inspection digests for later auditing

## How It Executes

1. You pass a handle or receipt into `workspace.inspections()?.inspect(...)`.
2. The runtime converts it into a `WorthQueryInspectionTarget`.
3. The runtime chooses the correct explanation family.
4. It derives a sealed inspection artifact from retained runtime evidence.
5. You pattern-match on `WorthQueryInspection` and read the relevant fields.

Inspection is unified at the entry point, but specialized in the result.

Good to know:

- `workspace.inspections()?.inspect(...)` is the convenience path
- `workspace.inspections()?.inspect_intent(...)` and `workspace.inspect_derived_intent(...)`
  are the explicit covered intent paths
- they converge on the same inspection result families rather than publishing
  a second explanation system
- projection consumption does not currently arrive through `workspace.inspections()?.inspect(...)`
- it uses admitted `WorthQueryConsumedProjectionAuthority`, whose `facts()`,
  `receipt()`, and evidence getters are inspection-only projections
- use [Projection Consumption](projection-consumption.md) when the feature
  you need to inspect is “typed facts consumed from this materialization”

- when that materialization was temporal, async, mixed-cause, or remask-bound,
  the receipt-first projection-consumption lane now retains that typed posture
  directly instead of making callers rediscover it from lower runtime evidence

## Small Example

```rust
use worth_query::facade::runtime::{WorthQueryInspection, WorthQueryLiveView};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("tasks").unwrap();

let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

let inspection = workspace.inspections()?.inspect(&view).unwrap();

match inspection {
    WorthQueryInspection::LiveView(live) => {
        assert_eq!(live.view_name(), "tasks.table");
        assert_eq!(
            live.authority_lane().as_str(),
            "authoritative-truth"
        );
    }
    other => panic!("expected live inspection, got {other:?}"),
}
```

This is the smallest honest example because it proves inspection is unified at
the call site but typed at the result.

When you need the admitted proof chain explicitly, use the intent path:

```rust
let review = workspace.inspections()?.inspect_intent(&view).review()?;
let admitted = review.admit()?;
let result = admitted.execute()?;

let trace = result.receipt().decision_trace_envelope();
let provenance = result.receipt().execution_provenance();
```

## Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryInspection, WorthQueryLiveView, WorthQueryPreviewOptions,
};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("workflow").unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value", "status.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

let titles = workspace
    .computed(
        "tasks.title-rollup",
        |c| {
            c.depends_on_live(&live)
                .reads(["title.value", "status.value"])
                .produces(["runtime.title_rollup"])
        },
        TitleListMaintainer,
    )
    .unwrap();

let effect = workspace
    .effect("tasks.publish-rollup", |e| {
        e.when_computed(&titles, ["runtime.title_rollup"])
            .condition_expression(
                "expr.publish-rollup",
                ["runtime.title_rollup"],
                ["ui.rollup"],
            )
            .deliver("ui.rollup")
            .meaningful_change_suppression()
    })
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Inspection target")
            .aspect("status.value", "open")
    })
    .unwrap();

let effect_inspection = workspace.inspections()?.inspect(&effect).unwrap();
let preview_label =
    WorthQuerySessionLabel::scoped_strs("inspection", ["rollup-preview"]).unwrap();

let mut preview = workspace
    .preview_with_options(
        preview_label.clone(),
        WorthQueryPreviewOptions::redirected_delivery(),
    )
    .unwrap();
let preview_binding = preview.use_effect(&effect).unwrap();
let preview_outcome = preview.discard().unwrap();

let binding_inspection = workspace.inspections()?.inspect(&preview_binding).unwrap();
let outcome_inspection = workspace.inspections()?.inspect(&preview_outcome).unwrap();

match effect_inspection {
    WorthQueryInspection::Effect(effect) => {
        assert_eq!(effect.trigger_source(), "tasks.title-rollup");
        assert_eq!(effect.target(), "ui.rollup");
        assert!(effect.pending_delivery_count() >= 1);
    }
    other => panic!("expected effect inspection, got {other:?}"),
}

match binding_inspection {
    WorthQueryInspection::PreviewBinding(binding) => {
        assert_eq!(binding.label(), preview_label.display());
        assert_eq!(binding.session_label(), &preview_label);
        assert_eq!(binding.effect_policy().as_str(), "redirected");
    }
    other => panic!("expected preview binding inspection, got {other:?}"),
}

match outcome_inspection {
    WorthQueryInspection::PreviewOutcome(outcome) => {
        assert!(outcome.discarded());
        assert_eq!(outcome.session_label(), &preview_label);
        assert_eq!(outcome.promoted_write_count(), 0);
    }
    other => panic!("expected preview outcome inspection, got {other:?}"),
}
```

What is authoritative:

- live truth for `Task`

What is derived:

- computed rollup state
- effect routing evidence
- preview-local binding and closeout evidence

For preview inspection specifically, keep the distinction straight:

- `label()` is the rendered DX string
- `session_label()` is the typed `WorthQuerySessionLabel` artifact

Use the typed artifact when identity, replay linkage, or collision posture
matters.

What gets retained:

- live installation digests

Write receipt and batch-component inspection now also preserve typed authority
evidence for:

- existing-truth target bindings
- same-batch symbolic target references
- naming-aware attachment, rebind, and removal outcomes
- causality and provenance bundles carried from the bridge
- computed dependency and produced-aspect evidence
- effect trigger, condition, phase, and pending-delivery evidence
- preview basis, policy, and residue evidence

What gets inspected:

- family-specific explanation artifacts through one entry point
- write receipt declared aspect operations, retained mutation metadata, and
  resolved target evidence when you need to certify authored meaning instead of
  only touched fallout
- write receipt existing-truth binding evidence when you need to prove which
  authoritative identity selected a preexisting target and which canonical
  binding artifact the runtime admitted
- write receipt and batch-component symbolic target evidence when you need to
  prove which same-batch declaration a later mutation resolved through
- write receipt and batch-component assertion evidence when you need to prove
  whether an existing-truth assertion was merely retained or actually
  backend-verified at execution time
- write receipt and batch-component existing-truth binding family and resolved
  target identity when you need to distinguish entity-targeted and
  relation-targeted authoritative writes that touched similar aspect sets
- write receipt and batch-component continuity evidence when you need to prove
  which authoritative predecessor continued as which successor set, which
  existing-truth binding basis anchored the mutation, and which
  lineage/continuity digests the bridge preserved
- typed continuity denials on preview lanes when authored intent asks for
  continuity evidence that only the authoritative bridge-backed lane can mint
- batch write receipt aggregate existing-truth and symbolic digests when you
  need one inspectable session summary for mixed authoritative-import lanes
- batch write receipt aggregate continuity digest when one session contains
  multiple continuity-aware updates and you need one inspectable continuity
  summary instead of reconstructing it from component artifacts
- batch write receipt aggregated touched aspects, affected live/derived
  surfaces, and ordered component operations when one semantic import or
  workflow had to execute as multiple data-dependent writes

## How It Relates To Other Features

- Pair this with [Live Views](../runtime-surfaces/live-views.md) to explain subscription
  installation and active-lane posture.
- Pair it with [Computed](../runtime-surfaces/computed.md) to inspect dependencies, materialized
  rows, and pending derived patches.
- Pair it with [Effects](../execution/effects.md) to inspect routing, suppression, pending
  write-intents, and feedback phases.
- Pair it with [Branches And Previews](../foundations/branches-and-previews.md) to inspect
  policy, residue, and promotion closeout.
- Pair it with [Intent Admission](../execution/intent-admission.md) when you
  need the shared common-path versus advanced-path story for the covered
  inspection families.
- Pair it with [Projection Consumption](projection-consumption.md) when a
  read result, write receipt, or query-context execution needs receipt-first
  typed fact inspection rather than `workspace.inspections()?.inspect(...)`.
- Pair it with [Async Resources And Result State](async-resources-and-result-state.md)
  when the main question is how async declaration meaning, live async
  result-state, materialized fact posture, and continuation drift fit together.

Inspection is the trust surface that keeps the rest of the runtime usable.

## Inspection And Debugging

`workspace.inspections()?.inspect(...)` can currently return:

- `LiveView`
- `DerivedView`
- `Effect`
- `WriteReceipt`
- `BatchWriteReceipt`
- `BasisLifecycle`
- `IntentReceipt`
- `IntentDenial`
- `EffectIntentReceipt`
- `UnifiedInspectionResult` through covered inspection intent execution
- `PreviewBinding`
- `PreviewOutcome`
- `PreviewIntentReceipt`
- `BranchIntentReceipt`

Some especially important things to look for:

- live view: subscription family, basis digest, active lane digest, consumer
  attachment digest, budget policies, counter digests
- live view scalar posture: `ordinary_runtime_posture().kind()`,
  `ordinary_runtime_posture().cause_posture()`, and
  `ordinary_runtime_posture().async_posture()`
- live view scalar qualifiers:
  `ordinary_runtime_posture().basis_posture()` and
  `ordinary_runtime_posture().support_evidence_digest()`
- live view remask qualifiers:
  `ordinary_runtime_posture().remask_posture()` plus
  `LiveViewInspection::remask_posture()` when you need the retained narrowing
  authority and basis/policy/proof/schema digests directly
- cross-runtime temporal/async "why" questions:
  use receipt-and-`ScopedInspectionBasis`-bound `CausalInspection` artifacts
  and their
  `temporal_async_explanation()` surface instead of expecting
  `workspace.inspections()?.inspect(...)` to materialize cross-runtime explanation envelopes
- basis-sensitive artifacts: admitted basis digest, scoped digest, lower-runtime
  binding digest, retained world-basis support digest, and whether the artifact
  remained ready, advisory, stale, denied, or unsupported through the Query
  basis lifecycle
- computed: upstream live/computed dependencies, dependency aspects, produced
  aspects, incremental posture, pending patch counts
- effect: trigger source, condition descriptor, target lane, effect policy,
  pending delivery counts, latest phase evidence, feedback graph
- preview: effect policy, basis evidence, admitted side-effect posture, closeout
  kind, residue counts, promotion/discard posture
- preview temporal/async closeout: temporal wake residue count, async
  result-state residue count, mixed-cause residue count, crossed-authoritative
  residue count, and promotion rebinding digest when promotion succeeded
- preview promotion denials: stale-basis, atomic-batch, write-failed, and
  rebinding-required posture, including typed recovery posture on the denial
  evidence itself
- saved-query reuse: frozen temporal/async surface posture, rebinding matrix
  rows, and whether a mismatch is legal fresh-freeze-required reuse or hard
  semantic drift denial
- intent artifacts: source and target lanes, strategy identity/version, outcome
  digests, invariant evidence, denial stage

## Anti-Patterns

- Treating inspection as a cheap replacement for reads or materialization.
- Assuming the unified entry point means all artifact families expose the same
  fields.
- Reading private meaning into digest strings instead of using the typed accessors.
- Using inspection as permission to bypass support admission, effect policy, or
  the covered inspection intent path when you need the admitted proof chain.

## Current Limits

- Inspection is stable for the runtime-backed synchronous artifact families
  listed above.
- Future temporal and async families must extend this explanation surface rather
  than creating a second debugging API.
- Mixed-cause delivery explanation must also extend this same retained
  inspection world rather than introducing a separate delivery-debug facade.
- Preview discard and promotion now retain temporal/async closeout posture on
  the preview inspection lane rather than expecting callers to reconstruct it
  from live delivery artifacts after the fact.
- Compact runtime posture is a projection of retained live evidence, not a
  second live-state engine or a replacement for the richer `LiveView`
  inspection fields.
- Canonical mixed-cause ordering comes from Bridge law; inspection only reads
  the retained ordered/coalesced result.
- Canonical remask posture is resolved before public runtime projection.
  Inspection reads that retained remask truth; it does not materialize first
  and mask later.
- Reuse surfaces do not get to silently flatten future-bearing declaration
  meaning into ordinary-only saved-query or view-shape posture. Inspection can
  read the retained freeze/reuse posture, but it does not retroactively repair
  a surface that should have denied or deferred earlier.
- Inspection explains runtime artifacts. It does not turn unsupported families
  into admitted ones.

## Related Docs

- [Cross-runtime causal inspection](cross-runtime-causal-inspection.md)
- [Async Resources And Result State](async-resources-and-result-state.md)
- [Inspection vs cross-runtime explanation (chooser)](../domain-capabilities/choosing/inspection-vs-cross-runtime-explanation.md)
- [Workspace Overview](../foundations/workspace-overview.md)
- [Live Views](../runtime-surfaces/live-views.md)
- [Computed](../runtime-surfaces/computed.md)
- [Effects](../execution/effects.md)
- [Branches And Previews](../foundations/branches-and-previews.md)
- [Projection Consumption](projection-consumption.md)


