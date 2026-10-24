# Authority-Scoped Effect Execution

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Authority-scoped effect execution turns an admitted effect into work performed
by the runtime that owns its target. Use it after authoring and admission when
you need to execute a writeback, mutation, or supported delivery neighbor and
retain an honest typed outcome.

A merge runs only through the workspace that owns its runtime
(`branch_merge`), because only that owner knows the graph's unique fields.
`EffectExecutionAuthority` lent a runtime denies a lowered merge with
`EffectExecutionDenialKind::MergeRequiresRuntimeOwner`.

[Effects](effects.md) owns authoring (`workspace.effect`, triggers, and staged
intent). This guide owns lowering, execution, and the typed outcome when a
Relational effect's durable fate is unknown.

## Why You Use It

- Execute admitted effects with the correct owner and receipt type.
- Check support before claiming a store-backed or durable path exists.
- Distinguish denial from work whose durable fate is unresolved.
- Repair a performed commit whose witness was lost, by commit identity,
  without rerunning the effect.

## Stable Entry Points

Import the lifecycle from `worth_query::facade::foundation`:

- `effect_lifecycle_support_matrix()` and
  `discover_effect_lifecycle_support(...)`
- `evaluate_effect_eligibility(...)` and `admit_effect_intent(...)`
- `EffectExecutionAuthority`
- `EffectExecutionStop` and `EffectBatchExecutionStop`
- `EffectExecutionDurabilityUnresolved` and `EffectBatchDurabilityUnresolved`
- `EffectExecutionAuthority::repair_pending_settlement(...)` and
  `EffectSettlementRepairError`

The authoring surface remains [Effects](effects.md).

## Core Mental Model

An admitted effect still has to cross the owner boundary:

```text
support discovery
  -> eligibility
  -> admission
  -> lowering
  -> execute with owner authority
       |-> executed receipt
       |-> denied or deferred before performance
       `-> durability unresolved
              -> reopen the owning Relational runtime
```

Relational publishes every commit to its durable medium before the live
cutover, so settlement after a performed commit cannot fail or defer.
`EffectExecutionStop::DurabilityUnresolved` and
`EffectBatchExecutionStop::DurabilityUnresolved` (accessor
`durability_unresolved()`) mean the durable fate is unknown: nothing was
installed live, but the commit may be durable. They are not denials. The
carrier exposes `commit_id()` and `unresolved()`; it has no repair method. The
Relational runtime refuses later publications until it is reopened, and
reopening resolves the fate.

Support postures remain explicit: `Admitted`, `Advisory`, `Denied`,
`RebindRequired`, `Deferred`, and `Unsupported`.

## How It Executes

1. Discover support for the exact basis and effect family.
2. Evaluate eligibility and admit the effect intent.
3. Lower the admitted effect or batch.
4. Execute with `EffectExecutionAuthority` for the declared owner.
5. On success, consume the execution receipt.
6. On `DurabilityUnresolved`, retain the typed carrier and reopen the owning
   Relational runtime; do not execute the effect program again.

A performed commit whose witness was lost is repaired by commit identity with
`EffectExecutionAuthority::repair_pending_settlement(commit_id)` on the runtime
that performed it. Repair is idempotent and does not execute the effect
program again.

## Small Example

```rust
use worth_query::facade::foundation::{
    EffectExecutionAuthority, EffectExecutionStop,
};

match lowered.execute_receipt_with(EffectExecutionAuthority::relational(&mut relational)) {
    Ok(receipt) => consume(receipt),
    Err(EffectExecutionStop::DurabilityUnresolved(unresolved)) => {
        record_unresolved(unresolved.commit_id(), unresolved.counters());
        reopen_relational_owner();
    }
    Err(stop) => handle_non_performed_stop(stop),
}
```

This is the smallest honest example because it branches on the typed terminal
and leaves the commit's fate to the reopened owner instead of retrying
`lowered`.

## Real Example

A batch follows the same rule while retaining one aggregate publication:

```rust
use worth_query::facade::foundation::{
    EffectBatchExecutionStop, EffectExecutionAuthority,
};

match lowered_batch.execute_receipt_with(
    EffectExecutionAuthority::relational(&mut relational),
) {
    Ok(receipt) => record_batch(receipt),
    Err(EffectBatchExecutionStop::DurabilityUnresolved(unresolved)) => {
        audit(unresolved.batch_identity(), unresolved.counters());
        reopen_relational_owner();
    }
    Err(stop) => handle_batch_stop(stop),
}
```

All components in a native mutation batch share the aggregate commit. One
unresolved outcome covers that one commit; do not loop over components.

## How It Relates To Other Features

- [Effects](effects.md) covers declaration, inspection, and staging DX.
- [Intent Admission](intent-admission.md) consumes staged write intent.
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
  supplies the exact basis before admission.
- [Application Aftermath And Recovery](application-aftermath-and-recovery.md)
  explains the application-level `Indeterminate` outcome and post-commit
  recovery boundaries.

## Inspection And Debugging

Inspect the typed stop, lifecycle counters, plan or batch identity, and repair
error. An effect with an unresolved durable fate reports
`durability_unresolved_count() == 1`; it must not increment denial telemetry.

`EffectSettlementRepairError::MissingRelationalAuthority` means pending
settlement repair was attempted without Relational authority. The wrapped
`PublicationSettlementRepairError` reports why the owning runtime could not
repair that commit identity.

## Anti-Patterns

- Retrying the effect after `DurabilityUnresolved`.
- Treating `DurabilityUnresolved` as a denial or recording denial telemetry.
- Repairing with another runtime merely because its branch or commit IDs look
  equal.
- Executing a family on a basis lane marked unsupported.
- Treating declaration or advisory support as proof of execution.

## Current Limits

| Cause / posture | Meaning |
|---|---|
| `StoreBackedExecutionDeferred` | Store-backed effect execution is deferred. |
| `DurableReplayDeferred` | Durable replay is deferred. |
| `AdvisoryOnlyExecution` | The path is advisory, not full execution authority. |
| `PreviewRebindRequired` | Rebind before execution. |
| `Denied` / `Unsupported` | Do not execute on the public contract. |

See `effect_lifecycle/support_matrix_rows.rs` for the complete inventory.
Settlement repair is currently runtime-affine; a typed carrier from one
Relational runtime cannot be repaired by another.

## Related Docs

- [Effects](effects.md)
- [Intent Admission](intent-admission.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Authoritative Mutation Evidence](../capabilities/authoritative-mutation-evidence.md)
- [Application Aftermath And Recovery](application-aftermath-and-recovery.md)
