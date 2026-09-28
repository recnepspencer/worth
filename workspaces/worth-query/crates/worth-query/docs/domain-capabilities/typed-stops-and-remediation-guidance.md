# Typed Stops And Remediation Guidance

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Query preserves typed stops so application code can understand why work did not
advance and choose an appropriate next ordinary command. The remediation
surface projects a stop into a concise explanation, a recommended action, and
the retained context that supports that recommendation.

The public Rust types use `WorthQueryRecovery*` names. Those types describe a
stop and recommend application action. They do not retry work, repair state,
authorize a command, reverse an effect, or prove that an external action
completed.

## Why You Use It

- keep stale, mismatched, unsupported, denied, and conflicted outcomes distinct;
- preserve whether a stop came from declaration, continuation, Signal,
  contribution, or grouped orchestration;
- retain basis, aspect, route, receipt, or grouped-member context;
- choose between refresh, rebind, narrowing, support inspection, declaration
  repair, contribution review, explicit handoff, or escalation;
- avoid parsing error strings or treating every stop as retryable.

## Stable Entry Points

For an ordinary outcome:

- `recover_from_outcome(...)`
- `worth_query_recovery_brief_from_ordinary_outcome(...)`

For retained checked or proof-visible products, use the matching method:

- `recover_from_declaration_entry_checked(...)`
- `recover_from_declaration_entry_proof(...)`
- `recover_from_declaration_route_plan_checked(...)`
- `recover_from_declaration_receipt_checked(...)`
- `recover_from_prepared_continuation_checked(...)`
- `recover_from_prepared_continuation_proof(...)`
- `recover_from_continuation_execution_checked(...)`
- `recover_from_continuation_execution_proof(...)`
- `recover_from_signal_compatibility_checked(...)`
- `recover_from_signal_compatibility_proof(...)`
- `recover_from_contribution_composed_checked(...)`
- `recover_from_contribution_composed_proof(...)`
- `recover_from_grouped_orchestration_checked(...)`
- `recover_from_grouped_orchestration_proof(...)`

The main result is `WorthQueryRecoveryBrief`. Its supporting public types are:

- `WorthQueryRecoveryExplanation`;
- `WorthQueryRecoveryRequest` and `WorthQueryRecoveryRequestKind`;
- `WorthQueryRecoveryStopFamily` and `WorthQueryRecoveryStopKind`;
- `WorthQueryRecoveryAuthoritySurface`;
- `WorthQueryRecoveryAction`;
- `WorthQueryRecoverySourceFamily`;
- basis, aspect, conflict, and evidence-strength posture types.

## Core Mental Model

The source stop remains authoritative. Remediation is a projection over that
stop, not a second outcome system and not a stronger capability.

Every brief answers four immediate questions:

1. `stop_family()`: which public feature stopped?
2. `stop_kind()`: what kind of stop occurred?
3. `authority_surface()`: which application boundary owns the correction?
4. `recommended_action()`: what should application code consider next?

Its explanation retains supporting context:

- `source_family()` identifies the originating feature;
- `basis_posture()` distinguishes stale, mismatched, reduced, or unknown basis;
- `aspect_posture()` explains whether aspect meaning was required or retained;
- `evidence_strength()` distinguishes ordinary, checked, and proof-visible
  evidence;
- `conflict_posture()` retains source-conflict meaning;
- route, receipt, contribution, and grouped-member accessors preserve
  source-specific facts.

The recommendation does not carry the authority needed to perform it. For
example, `RefreshBasis`, `RebindContext`, or `RetryLater` tells the application
what kind of command to prepare. The command must still pass its normal public
declaration, admission, currentness, and execution boundaries.

## How It Executes

```text
typed non-success outcome
    -> stop-family classification
    -> stop-kind classification
    -> retained explanation
    -> owning application boundary
    -> recommended next ordinary action
```

The projection performs no lower-runtime work and mints no operational
authority. Checked and proof-visible entry points keep stronger retained
context when the source product contains it.

Examples:

- stale continuation basis recommends `RefreshBasis`;
- asynchronous request drift recommends `RebindContext`;
- policy remask drift recommends `CheckSupport`;
- contribution denial recommends `ReviewContributionIntent`;
- preview-crossed residue recommends `UseExplicitHandoff`;
- wrong handle or world remains distinct from authority mismatch.

## Small Example

```rust
let outcome = handle.orchestrate_declaration_with_contributions_outcome(input);

if let Some(brief) = handle.recover_from_outcome(&outcome) {
    match brief.recommended_action() {
        WorthQueryRecoveryAction::ReviewContributionIntent => {
            show_contribution_review(brief.explanation());
        }
        other => show_remediation(other, brief.reason()),
    }
}
```

The example inspects a recommendation. It does not execute the recommendation
or treat the brief as admission authority.

## Real Example

```rust
let checked = handle.execute_prepared_continuation_checked(prepared);

if let Some(brief) = handle.recover_from_continuation_execution_checked(checked) {
    match brief.recommended_action() {
        WorthQueryRecoveryAction::RefreshBasis => {
            let request = application.prepare_fresh_basis_request(
                brief.explanation().basis_posture(),
            );
            queue_ordinary_request(request);
        }
        WorthQueryRecoveryAction::RebindContext => {
            request_context_rebinding(brief.explanation());
        }
        WorthQueryRecoveryAction::CheckSupport => {
            inspect_runtime_support(brief.explanation());
        }
        other => route_for_operator_review(other, brief.explanation()),
    }
}
```

The application converts the recommendation into its own next request. Query
does not let the brief bypass authentication, basis selection, capability
admission, or continuation readmission.

## How It Relates To Other Features

- [Ordinary Outcomes](./ordinary-outcomes.md) provide the compact source stop.
- [Continuation Pipeline](./continuation-pipeline.md) owns continuation state
  and readmission.
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
  owns Signal-facing compatibility evidence.
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
  preserves declaration and contribution ownership.
- [Grouped Authoring](./grouped-authoring.md) preserves member-local grouped
  context.
- [Inspection](../capabilities/inspection.md) exposes broader runtime evidence
  without changing authority.

## Inspection And Debugging

Use the brief for routing:

- `stop_family()`;
- `stop_kind()`;
- `authority_surface()`;
- `recommended_action()`;
- `reason()`.

Use the explanation for evidence:

- `checked_topology()`;
- `stop_stage()`;
- `retained_digest()`;
- `route_governing_reason()` and `route_denial_cause()`;
- `receipt_governing_reason()` and `receipt_denial_cause()`;
- `contribution_digest()` and `contribution_intent_descriptor()`;
- `grouped_member_context()`;
- foundational support and diagnostic posture.

Prefer typed posture and retained context over parsing `reason()`.

## Anti-Patterns

- calling every non-success outcome retryable;
- treating `WorthQueryRecoveryBrief` as a recovery handle or capability;
- executing a recommended action without ordinary admission;
- treating `RetryLater` as proof that duplicate execution is safe;
- treating `RefreshBasis` as permission to reuse stale authority;
- flattening stale basis and basis mismatch into one error;
- parsing prose when a typed stop kind exists;
- dropping grouped-member or contribution context before operator review.

## Current Limits

- The surface explains stops and recommends actions; it does not perform them.
- Recommendations do not prove idempotency, external completion, or effect
  safety.
- Receipt-bound recovery after a committed application operation belongs to
  the application-aftermath surface; a recovery brief remains descriptive.
- Route-plan and receipt products retain less aspect context than continuation,
  Signal, contribution, and grouped proof products.
- Materialization is intentionally lean unless the source carries a stronger
  retained profile.

## Related Docs

- [Ordinary Outcomes](./ordinary-outcomes.md)
- [Continuation Pipeline](./continuation-pipeline.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
- [Grouped Authoring](./grouped-authoring.md)
- [Inspection](../capabilities/inspection.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
