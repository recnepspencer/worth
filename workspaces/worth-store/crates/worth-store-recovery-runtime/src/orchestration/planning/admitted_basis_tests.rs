use super::*;
use std::path::Path;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, StoreRecoveryBindingSampleAllocationDenial,
    StoreRecoveryBindingSampleDenial,
};
use worth_store_recovery_physics::PhysicalRedoPlanningDenial;
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};
use crate::progression::SelectedPhysicalRecovery;

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

fn with_fixture(name: &str, checkpoint_mutations: u8, run: impl FnOnce(&Path)) {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(name, 4)
        .unwrap();
    let retained = world.retained_root();
    for index in 0..checkpoint_mutations {
        canonical_physical_mutation_acknowledgment(
            &world,
            [0x40 + index; 32],
            &vec![index + 1; 256],
        );
    }
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
    canonical_durable_wal_attempt_without_execution(&world, [0x71; 32], b"wal-tail-real-target");
    drop(world);
    run(retained.path());
}

#[test]
fn sampling_total_ceiling_bounds_primary_plus_wal_decode_grant_and_preserves_online_order() {
    with_fixture("sampling_total_ceiling_boundary", 1, |root| {
    let selected = select_real(root);
    let (ctx, _) = super::super::context::PlanningContextBody::from_selected(selected).unwrap();
    let basis = StoreRecoverySamplingBasis::Checkpoint(
        ctx.selection.checkpoint().expect("checkpoint").checkpoint(),
    );
    let wal_view = ctx
        .integrity
        .admitted_wal()
        .recoverable_frame_view(ctx.selection.wal_tail());
    let generous_cleanup = 64 << 20;

    // 1. At 0 total ceiling, primary census refuses before reserving.
    let err0 = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        wal_view,
        ctx.limits.operation_bindings,
        ctx.limits.redo_bytes,
        generous_cleanup,
        0,
    )
    .expect_err("zero total ceiling must refuse primary census");
    let primary_requested = match err0.allocation_denial() {
        Some(StoreRecoveryBindingSampleAllocationDenial::LocalLimit { required, admitted }) => {
            assert_eq!(*admitted, 0);
            assert!(*required > 0);
            *required
        }
        other => panic!("expected LocalLimit at 0, got {other:?}"),
    };

    // 2. At primary_requested alone, the co-live WAL member decode grant pushes
    //    total peak past primary_requested and must refuse before reserving decode.
    let err_primary = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        wal_view,
        ctx.limits.operation_bindings,
        ctx.limits.redo_bytes,
        generous_cleanup,
        primary_requested,
    )
    .expect_err("primary-only ceiling must refuse co-live WAL decode grant");
    let full_peak = match err_primary.allocation_denial() {
        Some(StoreRecoveryBindingSampleAllocationDenial::LocalLimit { required, admitted }) => {
            assert_eq!(*admitted, primary_requested);
            assert!(
                *required > primary_requested,
                "decode grant must be strictly positive for real retained member"
            );
            *required
        }
        other => panic!("expected LocalLimit for decode grant, got {other:?}"),
    };

    // 3. One byte short of full peak refuses at exact boundary.
    let err_short = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        wal_view,
        ctx.limits.operation_bindings,
        ctx.limits.redo_bytes,
        generous_cleanup,
        full_peak - 1,
    )
    .expect_err("one byte short of full peak must refuse");
    assert_eq!(
        err_short.allocation_denial(),
        Some(&StoreRecoveryBindingSampleAllocationDenial::LocalLimit {
            required: full_peak,
            admitted: full_peak - 1,
        })
    );

    // 4. Online semantic-denial order is preserved: RedoByteLimit fires before decode budget.
    let err_redo = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        wal_view,
        ctx.limits.operation_bindings,
        0,
        generous_cleanup,
        full_peak - 1,
    )
    .expect_err("zero redo byte limit must fire online before member decode");
    assert_eq!(err_redo.denial(), StoreRecoveryBindingSampleDenial::RedoByteLimit);
    assert!(err_redo.allocation_denial().is_none());

    // 5. Exact full peak admits cleanly with the real retained WAL member.
    let sample = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        wal_view,
        ctx.limits.operation_bindings,
        ctx.limits.redo_bytes,
        generous_cleanup,
        full_peak,
    )
    .expect("exact full sampling peak must admit");
    assert_eq!(sample.wal_members().len(), 1);
    drop(sample);
    let body = *ctx;
    drop(body.authority.media);
    body.authority.session.refuse();
    });
}

#[test]
fn nonempty_reconciliation_and_real_physical_redo_target_admission_enforce_exact_boundaries() {
    with_fixture("reconcile_redo_target_boundaries", 3, |root| {
    let selected = select_real(root);
    let (ctx, _) = super::super::context::PlanningContextBody::from_selected(selected).unwrap();
    let basis = StoreRecoverySamplingBasis::Checkpoint(
        ctx.selection.checkpoint().expect("checkpoint").checkpoint(),
    );
    let sample = PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        ctx.coordination.owner(),
        &ctx.authority.media,
        basis,
        ctx.integrity
            .admitted_wal()
            .recoverable_frame_view(ctx.selection.wal_tail()),
        ctx.limits.operation_bindings,
        ctx.limits.redo_bytes,
        ctx.limits.recovery_memory_bytes,
        ctx.limits.recovery_memory_bytes,
    )
    .unwrap();
    assert!(sample.operations().len() >= 4);
    assert_eq!(sample.wal_members().len(), 1);

    // 1. Nonempty reconciliation isolated exact / one-short boundary.
    let base = 1_024_u64;
    let reconcile_need =
        PlanningResidentAllowance::slot_bytes::<RecoveryOperationEvidenceInput>(
            sample.operations().len(),
        )
        .unwrap()
            + PlanningResidentAllowance::slot_bytes::<ReconciledOperationFate>(
                sample.operations().len(),
            )
            .unwrap();
    assert!(reconcile_need > 0);
    let mut short_allowance =
        PlanningResidentAllowance::new(base, base + reconcile_need - 1).unwrap();
    let Err(ReconcileAdmissionError::Memory(PlanningMemoryDenial::RecoveryMemoryBytes {
        observed,
    })) = admit_reconciled_fates(&sample, ctx.limits.operation_bindings, &mut short_allowance)
    else {
        panic!("one byte short of nonempty reconciliation must refuse before allocation");
    };
    assert_eq!(observed, base + reconcile_need);
    assert_eq!(short_allowance.used(), base);

    let mut exact_allowance = PlanningResidentAllowance::new(base, base + reconcile_need).unwrap();
    let fates = admit_reconciled_fates(&sample, ctx.limits.operation_bindings, &mut exact_allowance)
        .expect("exact reconciliation boundary must admit");
    assert_eq!(fates.operations().len(), sample.operations().len());
    assert_eq!(
        exact_allowance.used(),
        base + fates.owned_heap_bytes().unwrap()
    );

    // 2. Real physical redo pre-decode boundary and retained redo / target-roster boundaries.
    let redo_inputs = operation_join::redo_inputs(&sample, &fates).unwrap();
    assert_eq!(redo_inputs.len(), 1);
    let format = ctx.selection.root().selected().selector().format();
    let redo_bytes: u64 = redo_inputs
        .iter()
        .map(|m| m.canonical_redo().len() as u64)
        .sum();
    let remaining_manifest = ctx
        .limits
        .manifest_entries
        .saturating_sub(ctx.counters.manifest_entries);
    let mk_redo_limits = |recovery_memory_bytes| PhysicalRedoAdmissionLimits {
        recovery_memory_bytes,
        targets: ctx.limits.redo_targets,
        distinct_targets: ctx.limits.distinct_pages_and_extents,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: ctx.limits.redo_targets,
            record_identities: ctx.limits.redo_targets,
            placements: remaining_manifest,
            segment_updates: remaining_manifest,
            manifests: redo_bytes,
            total_entries: remaining_manifest
                .saturating_mul(2)
                .saturating_add(redo_bytes),
            inline_allocations: remaining_manifest,
        },
    };

    // 2a. Zero remaining memory refuses before decode allocates records/projection.
    let Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
        observed: first_alloc,
        admitted,
    }) = admit_physical_redo_members_with_memory_budget(
        redo_inputs.clone(),
        ctx.authority.media.store_identity(),
        format,
        mk_redo_limits(0),
        0,
    )
    else {
        panic!("zero redo allowance must refuse before decoding");
    };
    assert_eq!(admitted, 0);
    assert!(first_alloc > 0);
    assert_eq!(
        admit_physical_redo_members_with_memory_budget(
            redo_inputs.clone(),
            ctx.authority.media.store_identity(),
            format,
            mk_redo_limits(first_alloc - 1),
            first_alloc - 1,
        ),
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: first_alloc,
            admitted: first_alloc - 1,
        })
    );

    // 2b. Admit real physical redo and test target-roster isolated exact / one-short boundary
    //     with live admitted redo debited (and consumed redo_inputs released).
    let redo = admit_physical_redo_members_with_memory_budget(
        redo_inputs,
        ctx.authority.media.store_identity(),
        format,
        mk_redo_limits(ctx.limits.recovery_memory_bytes),
        ctx.limits.recovery_memory_bytes,
    )
    .expect("healthy redo admission succeeds");
    assert!(redo.target_identity_count() >= 1);
    assert!(redo.distinct_target_count() >= 1);
    let redo_live = redo.owned_heap_bytes().expect("admitted redo heap bytes");
    assert!(redo_live > 0);
    let targets_bytes = PlanningResidentAllowance::slot_bytes::<PhysicalRedoTargetIdentity>(
        redo.target_identity_count(),
    )
    .unwrap();
    assert!(targets_bytes > 0);

    let live_with_redo = base + redo_live;
    let mut short_targets =
        PlanningResidentAllowance::new(live_with_redo, live_with_redo + targets_bytes - 1).unwrap();
    assert_eq!(
        admit_target_identities(&redo, &mut short_targets),
        Err(PlanningMemoryDenial::RecoveryMemoryBytes {
            observed: live_with_redo + targets_bytes,
        })
    );
    assert_eq!(short_targets.used(), live_with_redo);

    let mut exact_targets =
        PlanningResidentAllowance::new(live_with_redo, live_with_redo + targets_bytes).unwrap();
    let targets =
        admit_target_identities(&redo, &mut exact_targets).expect("exact target roster admits");
    assert_eq!(targets.len(), redo.target_identity_count());
    assert_eq!(exact_targets.used(), live_with_redo + targets_bytes);
    assert_eq!(
        targets
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u64,
        redo.distinct_target_count()
    );

    drop(sample);
    let body = *ctx;
    drop(body.authority.media);
    body.authority.session.refuse();
    });
}
