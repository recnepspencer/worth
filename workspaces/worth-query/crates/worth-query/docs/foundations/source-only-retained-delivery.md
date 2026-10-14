# Retained delivery for no-output actions

## What This Feature Is

A no-output action declared with `ApplicationRetainedCommit` owns its original mutation and delivers a read-only observation of its actual World commit. Fresh execution returns the handler result. Replay and unpublished recovery return native recovered evidence, without inventing that result.

## Why You Use It

Use this delivery when a response must describe the original committed state even if current state advances before the response is delivered. It also preserves unpublished recovery and cleanup custody across interrupted requests.

## Stable Entry Points

Declare `operation_action::<Binding, Delivery>()` with `Delivery::Mode = ApplicationRetainedCommit`. Retain the original description with `idempotency_owned(key).retain_operation::<Program, Action>(&runtime)`.

Retained mode requires `advance(&fresh_request, maximum_history_work)`, where the bound is `NonZeroUsize`. Source-completion mode uses `advance(&fresh_request)` and carries no historical-read argument. There is one advance entrance for each declared mode.

## Core Mental Model

The descriptor owns the original input, conditions, source, key and branch. It carries no principal or permission. Each attempt borrows a fresh request and uses the existing binding-selected authorization owner. World history supplies the committed observation; the receipt identifies which occurrence to select.

## How It Executes

Fresh retained execution already holds the exact native observation. Replay and recovery first retain actual recovered evidence, then advance to select that evidence's original commit through native `at_commit`. A refused delivery returns the same operation. A completed owner retains any cleanup obligations until `close` succeeds.

## Small Example

With your program and retained action declared:

```rust
let owner = request.mutate(intent)
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(key)
    .retain_operation::<Program, RetainedAction>(&runtime)?;
let next = owner.advance(&fresh_request, maximum_history_work);
```

The history bound is supplied for this attempt. It is neither part of operation meaning nor inferred from query limits.

## Real Example

When an advance returns `Completed(completed)`, inspect its evidence and use the actual retained observation for a read with fresh authority:

```rust
let observation = match completed.evidence() {
    WorthQuerySourceCompletion::Fresh { retained: Some(read), .. }
    | WorthQuerySourceCompletion::Recovered { retained: Some(read), .. } => read,
    _ => unreachable!("this action declares retained delivery"),
};
let original_state = fresh_request.at(observation).query(read_intent).execute()?;
let closed = completed.close(&fresh_request);
```

A later current query may return different domain values. This read remains explicitly historical at the original commit. If close refuses, its error contains the completed owner; retain it and retry close with a lawful fresh request. Do not discard it because response data is already available.

## How It Relates To Other Features

Capability-bound actions use capability admission for initial execution, recovery and keyed resolution. Required-output actions use their fixed owning lane; declaring source-only delivery cannot erase required-output ownership. Pinned mutation sources remain original descriptor meaning, distinct from the historical observation delivered after a commit.

### Recover a capability-bound partial

After a real append failure returns `ProductUnpublished`, retain the operation returned by `Progress` and authenticate again for the next request. Call the same mode's `advance` entrance; the owner carries its original input, key, conditions, source and branch into fresh capability admission before native recovery. Authentication alone does not authorize recovery. A principal permitted to revoke grants can still lack the capability for this action.

If that admission refuses, `Refused` returns the original operation with its partial custody intact. Retain it and retry with a freshly authenticated principal whose current capability authorizes the original action. Recovery adopts the original native owner effects and resolves their keyed outcome without invoking the source handler again. Recovered completion supplies native evidence rather than a recreated handler result.

For example, a retention action can land its change to 18 days before publication completes. A fresh request from a revoke-only principal refuses recovery; the original retention principal can then recover the same operation and independently query 18 days. Retain completed custody when `close` refuses, and retry cleanup with a fresh lawful request. This applies to both source-completion and retained-commit delivery; retained mode still requires its explicit per-attempt history bound.

## Inspection And Debugging

`RetainedDeliveryPending` means the owner holds recovered evidence awaiting native history selection. `History(CommitUnavailable)` means the commit was not found in branch history within the supplied bound. It does not prove historical absence. Retain the owner and fund a fresh attempt; cancellation and foreign-runtime refusals also preserve custody.

## Anti-Patterns

Do not reconstruct a World observation from a receipt, read current truth as the original result, rerun a landed handler, substitute a new key/source/conditions, or treat a bounded history miss as terminal absence. Do not use an application-level registry or polling engine to duplicate the native owner.

## Current Limits

Native history selection reports a bounded unavailable result without proving whether more work would find the occurrence. The owning operation therefore preserves evidence for retry. Recovery supplies receipt/publication evidence rather than a recreated handler result. Retained delivery does not certify crash-durable continuation storage or complete the application's transport lifecycle.

## Related Docs

- [Application front door](ordinary-application-front-door.md)
- [Build an application](../../../../../../docs/build-an-application.md)
- [Executable retained courts](../../../worth-query-certification/tests/application_graph/source_only/retained.rs)
- [Executable capability recovery court](../../../worth-query-certification/tests/application_graph/source_only/capability/recovery.rs)
