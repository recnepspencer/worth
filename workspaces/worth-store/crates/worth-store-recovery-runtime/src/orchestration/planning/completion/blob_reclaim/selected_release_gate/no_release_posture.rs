//! Admit the existing NoRelease selected-release custody family.
use super::{
    pending_replay_matches, pending_wal, selected_has_release, PlanningContext, PlanningCustody,
    ResolvedPlanningState,
};

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningState,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if let PlanningCustody::PendingPrepared { claim: pending, .. } = &basis.custody {
        if pending.marker().is_none() || !pending_replay_matches(basis) {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        return Ok(context);
    }
    if basis.observed_pages.ordered_releases.is_some() {
        return pending_wal::admit_completed_history(context, basis);
    }
    let (next, has_release) = selected_has_release(context, basis)?;
    context = next;
    if has_release {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let Some(shared) = context.coordination.owner().checkpoint() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let mut resident = match super::resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => return Err(context.redo_block(basis.planning_counters(), limit)),
    };
    if super::resident_basis::reserve_box::<
        worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody,
    >(&mut resident)
    .is_err()
    {
        let limit = super::resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters_at_peak(resident.peak()), limit));
    }
    let claim = match worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody::claim_selected_no_release(
        &context.selection,
        shared.stream(),
    ) {
        Ok(claim) => claim,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    if !matches!(&basis.custody, PlanningCustody::Unresolved) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    basis.custody = PlanningCustody::NoRelease(Box::new(claim));
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::{
        PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
        PhysicalCheckpointRequest,
    };
    use worth_store_test_support::harness::physical_residency::{
        canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
    };

    use crate::entry::{
        PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits,
        PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAuthority,
        PhysicalRecoveryStaticConfiguration,
    };
    use crate::orchestration::planning::{admitted_basis, context, resident_memory, resolved_basis};

    fn prepare_resolved(
        root: &Path,
    ) -> (
        PlanningContext,
        crate::orchestration::planning::resolved_basis::ResolvedPlanningBasis,
    ) {
        let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
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
            publication_effects: 4,
            cleanup_candidates: 64,
            cleanup_bytes: 1024 * 1024,
            observation_bytes: 4 * 1024 * 1024,
        })
        .unwrap();
        let configuration = PhysicalRecoveryStaticConfiguration::current();
        let authority = PhysicalRecoveryPlatformAuthority::acquire(
            root.to_path_buf(),
            configuration.clone(),
            limits,
        )
        .unwrap();
        let profile = authority.qualified_backend_profile().clone();
        let admitted = PhysicalRecoveryOpenRequest::declare(
            root.to_path_buf(),
            configuration,
            profile,
            limits,
            authority,
        )
        .admit()
        .unwrap();
        let selected = admitted.discover().unwrap().select().unwrap();
        let (ctx, allowance) = context::PlanningContextBody::from_selected(selected).unwrap();
        let (ctx, basis) = admitted_basis::admit(ctx, allowance).unwrap();
        resolved_basis::resolve(ctx, basis).unwrap()
    }

    #[test]
    fn no_release_custody_box_refuses_one_byte_short_and_admits_exact_boundary() {
        let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
            "no_release_custody_box_boundary",
            4,
        )
        .unwrap();
        let retained = world.retained_root();
        canonical_physical_mutation_acknowledgment(&world, [0x71; 32], &vec![5; 2_000]);
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
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
        drop(world);

        let box_bytes =
            std::mem::size_of::<worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody>()
                as u64;

        let (mut ctx, mut resolved) = prepare_resolved(retained.path());
        let live = resident_memory::live_bytes(&ctx, resolved.state()).unwrap();
        ctx.limits.recovery_memory_bytes = live + box_bytes - 1;
        let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = verify(ctx, resolved.state_mut())
        else {
            panic!("one byte short of NoRelease box must refuse before allocating");
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let limit = blocked.cause().limit().expect("resident limit failure");
        assert_eq!(
            limit.dimension(),
            PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
        );
        assert_eq!(limit.observed(), live + box_bytes);
        assert!(matches!(
            resolved.state().custody,
            PlanningCustody::Unresolved
        ));
        drop(resolved);

        let (mut ctx, mut resolved) = prepare_resolved(retained.path());
        let live = resident_memory::live_bytes(&ctx, resolved.state()).unwrap();
        ctx.limits.recovery_memory_bytes = live + box_bytes;
        let ctx = verify(ctx, resolved.state_mut()).expect("exact boundary must admit NoRelease");
        assert!(matches!(
            resolved.state().custody,
            PlanningCustody::NoRelease(_)
        ));
        assert!(ctx.coordination.shutdown_is_quiescent());
        ctx.authority.session.refuse();
        drop(resolved);
    }
}
