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
intent). This guide owns lowering, execution, and recovery when a Relational
effect performed its branch movement but durability settlement did not finish.

## Why You Use It

- Execute admitted effects with the correct owner and receipt type.
- Check support before claiming a store-backed or durable path exists.
- Distinguish denial from work that already performed and must be settled.
- Repair an exact performed Relational publication without rerunning the
  effect.

## Stable Entry Points

Import the lifecycle from `worth_query::facade::foundation`:

- `effect_lifecycle_support_matrix()` and
  `discover_effect_lifecycle_support(...)`
- `evaluate_effect_eligibility(...)` and `admit_effect_intent(...)`
- `EffectExecutionAuthority`
- `EffectExecutionStop` and `EffectBatchExecutionStop`
- `EffectExecutionSettlementDeferred` and `EffectBatchSettlementDeferred`
- `EffectSettlementRepairError`

The authoring surface remains [Effects](effects.md). A raw Relational
`DeferredPublicationSettlement` is not a Query application surface.

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
       `-> performed, but settlement deferred
              -> repair with the same owning Relational runtime
```

`EffectExecutionStop::SettlementDeferred` and
`EffectBatchExecutionStop::SettlementDeferred` do not mean execution was
denied. The canonical branch movement already happened. The opaque carrier
retains the exact repair capability and exposes `.repair_with(...)`; it does
not expose the raw lower-runtime token.

Support postures remain explicit: `Admitted`, `Advisory`, `Denied`,
`RebindRequired`, `Deferred`, and `Unsupported`.

## How It Executes

1. Discover support for the exact basis and effect family.
2. Evaluate eligibility and admit the effect intent.
3. Lower the admitted effect or batch.
4. Execute with `EffectExecutionAuthority` for the declared owner.
5. On success, consume the execution receipt.
6. On `SettlementDeferred`, retain the typed carrier and call `repair_with`
   using fresh authority for the runtime that performed the effect.

Repair is idempotent. It completes durability for the exact performed route;
it does not execute the effect program again.

## Small Example

```rust
use worth_query::facade::foundation::{
    EffectExecutionAuthority, EffectExecutionStop,
};

match lowered.execute_receipt_with(EffectExecutionAuthority::relational(&mut relational)) {
    Ok(receipt) => consume(receipt),
    Err(EffectExecutionStop::SettlementDeferred(deferred)) => {
        let receipt = deferred.repair_with(
            EffectExecutionAuthority::relational(&mut relational),
        )?;
        consume_repaired(receipt);
    }
    Err(stop) => handle_non_performed_stop(stop),
}
```

This is the smallest honest recovery example because it branches on the typed
terminal and repairs the existing publication instead of retrying `lowered`.

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
    Err(EffectBatchExecutionStop::SettlementDeferred(deferred)) => {
        audit(deferred.batch_identity(), deferred.counters());
        deferred.repair_with(
            EffectExecutionAuthority::relational(&mut relational),
        )?;
    }
    Err(stop) => handle_batch_stop(stop),
}
```

All components in a native mutation batch share the aggregate commit. Repair
settles that one performed commit and must not loop over components.

## How It Relates To Other Features

- [Effects](effects.md) covers declaration, inspection, and staging DX.
- [Intent Admission](intent-admission.md) consumes staged write intent.
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
  supplies the exact basis before admission.
- [Application Aftermath And Recovery](application-aftermath-and-recovery.md)
  explains the separate application-level settlement and post-commit recovery
  boundaries.

## Inspection And Debugging

Inspect the typed stop, lifecycle counters, plan or batch identity, and repair
error. A performed-but-unsettled effect reports executed work and
`publication_settlement_deferred_count() == 1`; it must not increment denial
telemetry.

`EffectSettlementRepairError::MissingRelationalAuthority` means the repair was
attempted without the owner required by the carrier. A wrapped settlement error
can report a foreign runtime, missing or mismatched performed route, or another
durability failure.

## Anti-Patterns

- Retrying the effect after `SettlementDeferred`.
- Treating `SettlementDeferred` as a denial or recording denial telemetry.
- Extracting, serializing, or accepting a raw lower-runtime settlement token.
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
