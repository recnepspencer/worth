Recovery physics exposes pure, proof-carrying decisions. Its sealed records
cannot be created by filling copied fields:

```compile_fail
use worth_store_recovery_physics::PhysicalRedoTarget;

let _forged = PhysicalRedoTarget {
    identity: todo!(),
    extent_coordinate: None,
    artifact: todo!(),
    artifact_offset: 0,
    artifact_length: 0,
    resulting_digest: [0; 32],
};
```

An immutable redo plan must come from the bounded physical decoder and
planner, not from a caller-owned vector of decisions:

```compile_fail
use worth_store_recovery_physics::ImmutablePhysicalRedoPlan;

let _forged = ImmutablePhysicalRedoPlan {
    records: Box::new([]),
    decisions: Box::new([]),
    projections: Box::new([]),
    recovery_root_allocation_bytes: 0,
    counters: todo!(),
};
```

Source selection cannot be forged from an arbitrary root, page-fact, or WAL
tail projection:

```compile_fail
use worth_store_recovery_physics::PhysicalSourceSelection;

let _forged = PhysicalSourceSelection {
    root: todo!(),
    page_facts: todo!(),
    retained_previous_page_facts: None,
    checkpoint: None,
    wal_tail: todo!(),
    compaction: None,
    residue: Vec::new(),
    trace: todo!(),
};
```

Physical root candidates are admitted only by the fixed role, format, store,
and selector boundary:

```compile_fail
use worth_store_recovery_physics::PhysicalRootSourceCandidate;

let _forged = PhysicalRootSourceCandidate {
    selector: todo!(),
    manifest: todo!(),
};
```

The physical checkpoint base is admitted only against a selected root and a
verified checkpoint stream; a raw checkpoint identity is insufficient:

```compile_fail
use worth_store_recovery_physics::PhysicalCheckpointBase;

let _forged = PhysicalCheckpointBase { checkpoint: todo!() };
```

Bounded planning cost is an owned value, not a caller-provided tuple that can
skip the exact limit comparison:

```compile_fail
use worth_store_recovery_physics::RecoveryPlanCost;

let _forged = RecoveryPlanCost {
    redo_targets: 1,
    redo_bytes: 1,
    distinct_targets: 1,
    operation_bindings: 1,
    observation_bytes: 1,
    total_observation_bytes: 1,
    staging_bytes: 1,
    dirty_frames: 1,
};
```

Page-redo eligibility is a pure decision produced by its constructor; copied
fields cannot mint an admitted page transition:

```compile_fail
use worth_store_recovery_physics::PageRedoEligibility;

let _forged = PageRedoEligibility {
    kind: todo!(),
    page_generation: todo!(),
    classified_page_lsn: todo!(),
    redo_frontier: todo!(),
    counters: todo!(),
};
```

The operation-fate join is likewise sealed behind admission and cannot be
replaced by a caller-defined outcome:

```compile_fail
use worth_store_recovery_physics::ReconciledOperationFates;

let _forged = ReconciledOperationFates {
    operations: Vec::new(),
    acknowledged_durable: 0,
    durable_unacknowledged: 0,
    proven_no_effect: 0,
    indeterminate: 0,
};
```

Recovery physics has no runtime effect owner, observer protocol, backend
durability profile, or integrity-handoff constructor. Those surfaces must be
obtained from their owning crates before a runtime boundary can consume them.
