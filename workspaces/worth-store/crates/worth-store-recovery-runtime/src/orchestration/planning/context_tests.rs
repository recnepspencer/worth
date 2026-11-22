use super::*;
use std::path::Path;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, StoreRecoveryBindingSampleAllocationDenial,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};

fn base_limits() -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 4,
        manifest_bytes: 1024 * 1024,
        manifest_entries: 4_096,
        wal_segments: 8,
        wal_frames: 64,
        wal_bytes: 1024 * 1024,
        redo_targets: 64,
        redo_bytes: 1024 * 1024,
        distinct_pages_and_extents: 64,
        operation_bindings: 64,
        staging_bytes: 4 * 1024 * 1024,
        recovery_memory_bytes: 64 * 1024 * 1024,
        dirty_frames: 64,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 64,
        cleanup_bytes: 1024 * 1024,
        observation_bytes: 4 * 1024 * 1024,
    }
}

fn select_real(root: &Path) -> SelectedPhysicalRecovery {
    let limits = PhysicalRecoveryLimits::admit(base_limits()).unwrap();
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_path_buf(), configuration.clone(), limits)
            .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
    .admit()
    .unwrap()
    .discover()
    .unwrap()
    .select()
    .unwrap()
}

fn with_memory_limit(
    selected: SelectedPhysicalRecovery,
    recovery_memory_bytes: u64,
) -> (SelectedPhysicalRecovery, u64) {
    let (
        mut authority,
        coordination,
        selection,
        integrity,
        counters,
        root_protocol_denials,
        integrity_trace,
    ) = selected.into_parts();
    let live_before = super::super::resident_memory::selected_live_bytes(
        &coordination,
        &selection,
        &integrity,
        &root_protocol_denials,
        &integrity_trace,
    )
    .unwrap();
    let mut decl = authority.limits.declaration();
    decl.recovery_memory_bytes = recovery_memory_bytes;
    authority.limits = PhysicalRecoveryLimits::admit(decl).unwrap();
    (
        SelectedPhysicalRecovery::new(
            authority,
            coordination,
            selection,
            integrity,
            counters,
            root_protocol_denials,
            integrity_trace,
        ),
        live_before,
    )
}

#[test]
fn plan_admits_context_backing_before_allocation_and_debits_early_sampling() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
        "plan_context_backing_boundary",
        4,
    )
    .unwrap();
    let retained = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x61; 32], &vec![3; 2_000]);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x62; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint admission");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x63; 32], b"wal-tail-redo");
    drop(world);

    let slot_bytes = std::mem::size_of::<PlanningContextBody>() as u64;

    // Measure live selected bytes first.
    let (selected, live_before) = with_memory_limit(select_real(retained.path()), 64 << 20);
    let _ = selected.cancel_before_reconstruction();

    // 1. One byte short of context backing: `plan()` must refuse before allocating context.
    let (selected, live_short) =
        with_memory_limit(select_real(retained.path()), live_before + slot_bytes - 1);
    assert_eq!(live_short, live_before);
    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = selected.plan() else {
        panic!("one byte short of PlanningContext backing must block before allocation");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::Cost(
            RecoveryPlanCostDenial::RecoveryMemoryBytes,
        ))
    );
    let limit = blocked.cause().limit().expect("resident limit failure");
    assert_eq!(
        limit.dimension(),
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(limit.observed(), live_before + slot_bytes);
    assert_eq!(limit.admitted(), live_before + slot_bytes - 1);

    // 2. Exact context boundary: `from_selected` admits with zero remaining early budget.
    {
        let (selected, live_exact) =
            with_memory_limit(select_real(retained.path()), live_before + slot_bytes);
        assert_eq!(live_exact, live_before);
        let (ctx, allowance) =
            PlanningContextBody::from_selected(selected).expect("exact boundary admits context");
        assert_eq!(
            super::super::resident_memory::context_live_bytes(&ctx),
            Some(live_before + slot_bytes)
        );
        assert_eq!(allowance.remaining(), 0);
        let body = *ctx;
        assert!(body.coordination.shutdown_is_quiescent());
        drop(body.authority.media);
        body.authority.session.refuse();
    }

    // 3. Exact context boundary (0 remaining): `plan()` must deny in binding sampling
    //    before reserving native backing, reporting the exact census requirement.
    let (selected, _) = with_memory_limit(select_real(retained.path()), live_before + slot_bytes);
    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = selected.plan() else {
        panic!("zero remaining sampling allowance must deny before reserving");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let sample_requested = match &blocked.evidence().planning_denial {
        Some(PhysicalRecoveryPlanningDenial::BindingSamplingAllocation(
            StoreRecoveryBindingSampleAllocationDenial::LocalLimit { required, admitted },
        )) => {
            assert_eq!(*admitted, 0);
            assert!(*required > 0);
            *required
        }
        other => panic!("expected sampling LocalLimit denial, got {other:?}"),
    };
    let limit = blocked.cause().limit().expect("sampling limit failure");
    assert_eq!(
        limit.dimension(),
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(limit.observed(), live_before + slot_bytes + sample_requested);
    assert_eq!(limit.admitted(), live_before + slot_bytes);

    // 4. One byte short of nonempty sampling boundary: still refuses in sampling before reserve.
    let (selected, _) = with_memory_limit(
        select_real(retained.path()),
        live_before + slot_bytes + sample_requested - 1,
    );
    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = selected.plan() else {
        panic!("one byte short of nonempty sampling census must deny before reserving");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::BindingSamplingAllocation(
            StoreRecoveryBindingSampleAllocationDenial::LocalLimit {
                required: sample_requested,
                admitted: sample_requested - 1,
            },
        ))
    );
    let limit = blocked.cause().limit().expect("sampling boundary limit");
    assert_eq!(limit.observed(), live_before + slot_bytes + sample_requested);
    assert_eq!(
        limit.admitted(),
        live_before + slot_bytes + sample_requested - 1
    );

    // 5. Exact nonempty sampling boundary: sampling admits, then carried co-live debit
    //    gates downstream reconciliation/redo before allocation.
    let (selected, _) = with_memory_limit(
        select_real(retained.path()),
        live_before + slot_bytes + sample_requested,
    );
    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = selected.plan() else {
        panic!("exact sampling boundary leaves insufficient co-live budget for downstream redo");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert!(
        !matches!(
            &blocked.evidence().planning_denial,
            Some(PhysicalRecoveryPlanningDenial::BindingSamplingAllocation(_))
        ),
        "sampling must admit at its exact boundary; got {:?}",
        blocked.evidence().planning_denial
    );
    assert!(matches!(
        &blocked.evidence().planning_denial,
        Some(
            PhysicalRecoveryPlanningDenial::Cost(RecoveryPlanCostDenial::RecoveryMemoryBytes)
                | PhysicalRecoveryPlanningDenial::Redo(
                    PhysicalRedoPlanningDenial::RecoveryMemoryLimit { .. }
                )
        )
    ));

    // 6. Healthy budget plans cleanly.
    let planned = select_real(retained.path())
        .plan()
        .expect("healthy budget plans cleanly");
    let _ = planned.cancel_before_execution();
}
