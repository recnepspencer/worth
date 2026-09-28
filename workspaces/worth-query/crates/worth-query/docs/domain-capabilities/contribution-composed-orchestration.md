# Contribution-Composed Orchestration

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Contribution-composed orchestration is the public Query lane that lets you run
one declaration-entry operation and attach declaration-scoped domain-capability
contributions in the same call.

Use it when your app already knows the declaration it wants to make and also
knows that the run should carry support, explanation, admission, workflow, or
continuity
contribution posture at the same time.

This feature does not replace declaration entry, and it does not replace the
underlying contribution-authoring system. It composes them into one Query-owned
front door while keeping the two proof chains separate and inspectable.

In plain English: this is the "run the declaration and keep the contribution
story attached" surface. It is especially useful when one contribution admits,
another denies, and your app still needs the admitted truth instead of losing
the whole run to first-failure collapse.

## Why You Use It

- keep declaration entry and declaration-scoped contribution authoring on one
  admitted-handle surface
- preserve declaration success plus mixed contribution truth as a real retained
  result set
- keep partial success inspectable when one contribution admits and a later
  contribution denies
- keep the declaration's retained aspect contract and each contribution's
  aspect-scoped binding story attached to the same composed run
- materialize contribution summaries without rebuilding the explicit
  declaration-bound authoring pipeline by hand
- inspect retained declaration and contribution evidence together
- keep the compact ordinary outcome lane without flattening contribution
  posture into generic declaration success

## Stable Entry Points

Admitted-handle entry points:

- `orchestrate_declaration_with_contributions(...)`
- `orchestrate_declaration_with_contributions_outcome(...)`
- `orchestrate_declaration_with_contributions_checked(...)`
- `orchestrate_declaration_with_contributions_proof(...)`

Input and intent types:

- `WorthQueryContributionComposedOrchestrationInput<D, I>`
- `WorthQueryContributionIntent`
- `WorthQueryContributionComposedMaterializationPolicy`

Result and proof types:

- `WorthQueryContributionComposedOrchestration<D, I>`
- `WorthQueryContributionComposedComposition`
- `WorthQueryContributionComposedClassification`
- `WorthQueryContributionComposedIntentResult`
- `WorthQueryContributionComposedIntentStageResult`
- `WorthQueryContributionComposedContribution`
- `WorthQueryContributionComposedSummary`
- `WorthQueryContributionComposedOrchestrationOutcome<D, I>`
- `WorthQueryContributionComposedOrchestrationChecked<D, I>`
- `WorthQueryContributionComposedOrchestrationPosture<D, I>`
- `WorthQueryContributionComposedOrchestrationTranscript<D, I>`

## Core Mental Model

Think of this feature as one composed run with two retained stories inside it:

1. Query lowers the declaration through the normal declaration-entry path until
   it has an envelope.
2. Query binds each contribution intent to the declaration-scoped contribution
   target for that retained declaration.
3. Query evaluates, admits, and optionally materializes each contribution
   intent independently.
4. Query retains the whole per-intent result set and then classifies the
   composed run.

That means the public lane now has two honest shapes after declaration reaches
envelope truth:

- if no contribution admits, the lane still returns a typed contribution-owned
  stop
- if at least one contribution admits, the lane keeps one bound composed
  artifact even when later intents deny or materialization narrows/fails

The important rule is:

- declaration success does not automatically mean contribution success
- one failing contribution does not erase earlier admitted contribution truth

This surface keeps that distinction typed.

Another important rule is:

- this feature requires at least one contribution intent

And one more rule matters for geometry- and workflow-heavy domains:

- this feature is aspect-aware because it reuses the shared binding pipeline

That means the composed lane is not only "declaration plus contributions." It
is "declaration plus contributions over one retained declaration-bound target
and one retained aspect-aware binding story." If two contribution intents later
need to be compared for overlap or conflict, this lane should not have erased
which semantic slice each one was bound against.

If you only want the declaration-side envelope run, use declaration-entry
orchestration directly. An empty contribution-composed request is rejected as
`Unsupported` on purpose so the feature boundary stays honest.

## How It Executes

`WorthQueryContributionComposedOrchestrationInput<D, I>` owns three things:

- one declaration input
- one or more typed contribution intents
- one materialization policy

The current contribution intents are:

- `WorthQueryContributionIntent::admission(...)`
- `WorthQueryContributionIntent::support(...)`
- `WorthQueryContributionIntent::explanation(...)`
- `WorthQueryContributionIntent::workflow(...)`
- `WorthQueryContributionIntent::continuity(...)`

The execution shape is:

1. `declare_review_and_progress(...)`
2. `envelope_routes_from_progressed(...)`
3. bind each contribution intent to the declaration-bound target and its
   retained aspect-aware declaration meaning
4. evaluate and admit each contribution intent
5. optionally materialize per-intent summaries
6. retain every reachable per-intent result
7. return either:
   - one bound composed artifact with full or partial contribution truth
   - one typed non-success posture when no contribution truth admitted

This feature does not silently continue into continuation preparation, signal
execution, or runtime execution. It stops at declaration envelope plus
contribution posture.

The aspect story is important here:

- declaration entry still owns declaration aspect contract
- contribution authoring still owns contribution-local aspect meaning
- contribution-composed orchestration keeps those truths aligned on one bound
  target instead of forcing your app to restitch them later

The classification on a bound artifact is the summary of the whole retained
intent-result set:

- `FullyAdmitted`
  - every contribution intent admitted
- `PartiallyAdmitted`
  - at least one contribution admitted and at least one later intent rejected
    or failed to materialize fully
- `NoContributionAdmitted`
  - declaration reached envelope truth, but contribution posture stayed on the
    non-success lane
- `MaterializationFailedAfterAdmission`
  - admission truth exists, but later summary materialization did not fully
    succeed

## Small Example

```rust
let composed = handle.orchestrate_declaration_with_contributions(
    WorthQueryContributionComposedOrchestrationInput::new(
        geometry_session.attach_material_for_active_face_selection()?,
    )
    .with_contribution(WorthQueryContributionIntent::support(
        WorthQuerySupportContributionAuthoring::declaration_traceability(
            "domain.traceability.face",
            "face selection remains traceable through declaration entry",
        ),
    )),
)?;

let _ = composed.envelope().envelope_digest();
let _ = composed.contributions()[0].evidence().evidence_digest();
```

## Real Example

```rust
let outcome = handle.orchestrate_declaration_with_contributions_outcome(
    WorthQueryContributionComposedOrchestrationInput::new(
        geometry_session.publish_boundary_change_for_active_face_selection()?,
    )
    .with_contribution(WorthQueryContributionIntent::workflow(
        WorthQueryWorkflowContributionAuthoring::preview_only(
            "domain.workflow.face",
            "preview workflow remains read-only at declaration entry",
        ),
    ))
    .with_contribution(WorthQueryContributionIntent::support(
        WorthQuerySupportContributionAuthoring::declaration_traceability(
            "domain.traceability.face",
            "face selection remains traceable through declaration entry",
        ),
    ))
    .materialize_summaries_with_profile(profile),
);

match outcome {
    WorthQueryOrdinaryOutcome::Bound(composed) => {
        let _ = composed.envelope().envelope_digest();
        let _ = composed.classification();
        let _ = composed.intent_results();
        let _ = composed.admitted_contributions();
        let _ = composed.rejected_intents();
        let _ = composed.composition().rejected_category_families();
        let _ = composed.materialized_artifacts();
    }
    WorthQueryOrdinaryOutcome::Denied(posture) => {
        let _ = posture.reason();
        let _ = posture.checked_topology().contribution_composed_kind();
        let _ = posture
            .checked_topology()
            .contribution_composed_linked_artifacts();
    }
    WorthQueryOrdinaryOutcome::Deferred(posture) => {
        let _ = posture.reason();
        let _ = posture.next_step();
    }
    other => {
        let _ = other;
    }
}
```

Good to know:

- if the declaration succeeds, one contribution admits, and a later intent
  denies, the composed lane still returns `Bound` with
  `classification() == PartiallyAdmitted`
- if the declaration succeeds and no contribution admits, the composed lane
  returns `ContributionDenied`, `Stale`, `RebindRequired`, `Unsupported`, or
  `Failed` based on the retained contribution stop
- if the declaration support posture is deferred before declaration entry can
  start, the composed lane returns `Deferred`, not `DeclarationDenied`
- if you pass no contribution intents at all, the checked lane returns
  `Unsupported`; use Declaration Entry Orchestration when you only need the
  declaration-side run
- the retained composed artifact is already a better foundation for later
  aspect-overlap or collaborative-conflict work than a first-failure summary,
  because it keeps per-intent target binding and contribution-local meaning
  visible

## How It Relates To Other Features

- Use [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
  when you only need the declaration-side envelope-ceiling run.
- Use [Ordinary Outcomes](./ordinary-outcomes.md) when you want the compact
  public result lane over this composed surface.
- Use [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) when the composed lane
  stopped and your app needs one typed answer that preserves declaration-side
  vs contribution-side repair ownership.
- Use [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
  when you need to explain how strong that recovery answer is.
- Use Runtime-Installed Domain Handles for the
  admitted-handle entry points that own this run.
- Use [Family Helpers](./family-helpers.md) when you already know the
  declaration family and want a family-native wrapper over this composed input
  and result surface instead of constructing the generic input yourself. The
  geometry material-attachment helper currently exposes support, explanation,
  and workflow contribution sugar while still lowering to these same canonical
  contribution intents.
- Use the `support/`, `workflow/`, `admission/`, and `explanation/` docs in
  this tree when you need the standalone contribution-authoring model behind
  the composed lane.
- Use [Continuation Pipeline](./continuation-pipeline.md) or
  [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
  only after you actually need continuation or signal-facing follow-up. This
  feature does not execute either one.

## Inspection And Debugging

Use the bound artifact when you need the success surface:

- `envelope()`
- `declaration_artifact()`
- `intent_results()`
- `admitted_contributions()`
- `rejected_intents()`
- `composition()`
- `classification()`
- `composition_digest()`
- `contribution_composition()`
- `contributions()`
- `materialized_artifacts()`
- `composed_digest()`

Use that surface when you need to answer questions like:

- which contribution actually admitted
- which contribution was rejected
- whether two intents were part of the same declaration-bound run
- which target and target binding each retained result came from

Each intent result exposes:

- `request()`
- `order_index()`
- `category_family()`
- `request_digest()`
- `semantic_code()`
- `detail()`
- `target_digest()`
- `target_binding_digest()`
- `evaluation()`
- `admission()`
- `materialization()`
- `classification()`
- `contribution()`

That means a denied or stale intent is still explainable from the retained
result itself. You do not have to reconstruct "which contribution was this?"
from top-level posture or from free-form reason strings.

That also means later tooling can compare intent-local target and binding
identity without guessing from top-level composition status alone.

Each admitted contribution exposes:

- `evidence()`
- `contribution_category()`
- `semantic_posture()`
- `support_outcome_kind()`
- `request_digest()`
- `summary()`

Use the checked lane when you need typed stop posture:

- `Deferred`
- `DeclarationDenied`
- `ContributionDenied`
- `Stale`
- `RebindRequired`
- `Unsupported`
- `Failed`

The posture surface exposes:

- `kind()`
- `stop_stage()`
- `reason()`
- `linked_artifacts()`
- `contribution_digest()`

Use the proof lane when you need retained proof over the whole composed run:

- `request_descriptor()`
- `request_digest()`
- `materialization_policy()`
- `declaration()`
- `intent_results()`
- `composition_classification()`
- `outcome()`
- `linked_artifacts()`
- `contribution_digest()`
- `into_checked()`

Use the recovery lane when the next job is repair guidance instead of more
inspection:

- `recover_from_outcome(...)`
- `recover_from_contribution_composed_checked(...)`
- `recover_from_contribution_composed_proof(...)`

## Anti-Patterns

- treating declaration success as if it implies contribution success
- using this feature as a declaration-only shortcut
- using this feature when you really need contribution authoring on a
  non-declaration target
- teaching this as if it replaced the underlying contribution-authoring model
- assuming materialized summaries imply continuation preparation or lower
  runtime execution
- flattening `DeclarationDenied` and `ContributionDenied` into the same app
  behavior
- treating `Bound` as if it always means every intent admitted
- ignoring `intent_results()` and reconstructing mixed contribution truth from
  `reason()` strings
- parsing `reason()` when the checked or ordinary posture kind already carries
  the decision you need

## Current Limits

- this surface currently binds contributions only to declaration-scoped targets
  derived from the retained canonical declaration
- continuity contributions on this surface are declaration-bound continuity
  posture, not admitted-plan continuity evidence materialization; if you need
  admitted-plan correspondence evidence or runtime continuity artifacts, stay
  on the admitted-plan continuity lane
- calling this surface with no contribution intents is rejected as
  `Unsupported`; use Declaration Entry Orchestration directly when you only
  need the declaration-side envelope run
- the current public inspection surface preserves the aspect-aware binding
  substrate implicitly through declaration-bound target and target-binding
  truth, but it is not yet a standalone aspect-conflict classifier
- the strongest mixed-result behavior on this public lane today is:
  - partial admission that remains `Bound`
  - contribution-owned denial when no contribution admits
  - declaration-owned `Deferred` when declaration support posture blocks entry
- the feature preserves contribution-local `Stale`, `RebindRequired`, and
  `Failed` posture in the retained model, but those are less natural on the
  immediate declaration-bound lane than partial admission and contribution
  denial
- the current materialization policy is summary-only or none
- contribution-owned `Stale` and `RebindRequired` are not common on the
  declaration-bound generic lane today; the retained per-intent model exists so
  later stronger retained seams can surface them honestly without redesigning
  this boundary again
- this feature stops at declaration envelope plus contribution posture
- this feature does not prepare continuation, execute continuation, or execute
  Signal work
- grouped neighborhoods now reuse this engine through the grouped contribution
  surface; use [Grouped Contributions](./grouped-contributions.md) when the
  contributions need to stay attached to one retained neighborhood instead of
  one single declaration

## Related Docs

- Runtime-Installed Domain Handles
- [Ordinary Outcomes](./ordinary-outcomes.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Continuation Pipeline](./continuation-pipeline.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Family Helpers](./family-helpers.md)
- [Grouped Contributions](./grouped-contributions.md)
- Declaration-Scoped Support And Traceability
- Advisory And Violation Contributions
- Preview Inspection And Mutation Planning
- Lower-Runtime Explanation Contributions
