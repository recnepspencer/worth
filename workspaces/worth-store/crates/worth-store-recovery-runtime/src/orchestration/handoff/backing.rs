//! Both later handoff phases and Store's returned core body are admitted before
//! any recovery effects.
use super::completion::PendingCompletion;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};
use worth_store::physical_runtime::{
    PreparedRecoveredRuntimeCoreBacking, RecoveredPhysicalRuntimeCore,
};
pub(crate) struct RecoveryHandoffBacking {
    pub(super) pending: Vec<PendingCompletion>,
    pub(super) core: Vec<RecoveredPhysicalRuntimeCore>,
    pub(super) core_body: Option<PreparedRecoveredRuntimeCoreBacking>,
}
impl RecoveryHandoffBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        let pending = allowance.reserve(1)?;
        let core = allowance.reserve(1)?;
        let body_requested = PreparedRecoveredRuntimeCoreBacking::required_bytes();
        allowance.transient(body_requested)?;
        let core_body = PreparedRecoveredRuntimeCoreBacking::try_prepare().map_err(|cause| {
            PlanningMemoryDenial::Allocation {
                requested_bytes: body_requested,
                cause,
            }
        })?;
        let actual_body = core_body
            .owned_heap_bytes()
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
        allowance.retain(actual_body)?;
        Ok(Self {
            pending,
            core,
            core_body: Some(core_body),
        })
    }
    pub(crate) fn required_bytes() -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::slot_bytes::<PendingCompletion>(1)?
            .checked_add(PlanningResidentAllowance::slot_bytes::<
                RecoveredPhysicalRuntimeCore,
            >(1)?)
            .and_then(|bytes| bytes.checked_add(PreparedRecoveredRuntimeCoreBacking::required_bytes()))
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    }
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        let overflow = || PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX };
        let body_bytes = match &self.core_body {
            Some(body) => body.owned_heap_bytes().ok_or_else(overflow)?,
            None => 0,
        };
        PlanningResidentAllowance::vector_bytes(&self.pending)?
            .checked_add(PlanningResidentAllowance::vector_bytes(&self.core)?)
            .and_then(|bytes| bytes.checked_add(body_bytes))
            .ok_or_else(overflow)
    }
}
#[cfg(test)]
#[path = "backing_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_backing_prepays_core_body_and_refuses_one_byte_short() {
        let required = RecoveryHandoffBacking::required_bytes().unwrap();
        assert!(
            PreparedRecoveredRuntimeCoreBacking::required_bytes()
                > std::mem::size_of::<RecoveredPhysicalRuntimeCore>() as u64,
            "handoff backing must fund the full Store core body, not only its header"
        );
        let mut denied = PlanningResidentAllowance::new(13, 13 + required - 1).unwrap();
        assert_eq!(
            RecoveryHandoffBacking::prepare(&mut denied).err(),
            Some(PlanningMemoryDenial::RecoveryMemoryBytes {
                observed: 13 + required
            })
        );
        let mut admitted = PlanningResidentAllowance::new(13, 13 + required).unwrap();
        let backing = RecoveryHandoffBacking::prepare(&mut admitted).unwrap();
        let actual = backing.backing_bytes().unwrap();
        assert!(actual >= required);
        assert_eq!(admitted.used(), 13 + actual);
        assert_eq!(admitted.peak(), 13 + actual);
        assert!(backing.core_body.is_some());
    }

    #[test]
    fn near_limit_recovery_refuses_one_byte_short_and_succeeds_at_exact_peak() {
        use std::path::Path;
        use worth_proof::TransitionOutcome;
        use worth_store::physical_runtime::{
            PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
            PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
        };
        use worth_store_test_support::harness::physical_residency::{
            canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
        };
        use crate::entry::{
            PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension,
            PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
            PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
        };

        let recover = |root: &Path, recovery_memory_bytes: u64| -> PhysicalRecoveryOutcome {
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
                recovery_memory_bytes,
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
            let request = PhysicalRecoveryOpenRequest::declare(
                root.to_path_buf(),
                configuration,
                profile,
                limits,
                authority,
            );
            crate::WorthStoreRecovery::recover(request)
        };

        let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
            "near_limit_handoff_backing",
            4,
        )
        .unwrap();
        let retained = world.retained_root();
        canonical_physical_mutation_acknowledgment(&world, [0x91; 32], &vec![9; 3_000]);
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new([0x92; 32]),
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

        let PhysicalRecoveryOutcome::Recovered(handoff) = recover(retained.path(), 64 << 20) else {
            panic!("generous recovery must succeed");
        };
        let peak = handoff.planning_counters().peak_recovery_bytes();
        let retained_bytes = handoff
            .store_rejoin_retained_bytes()
            .expect("retained handoff bytes");
        assert!(
            retained_bytes >= PreparedRecoveredRuntimeCoreBacking::required_bytes(),
            "handoff retained sample must include the prepaid Store core body"
        );
        assert!(handoff.into_core().into_serving_handoff().is_some());

        let PhysicalRecoveryOutcome::Blocked(blocked) = recover(retained.path(), peak - 1) else {
            panic!("one byte below peak ({peak}) must block before recovery effects");
        };
        assert_eq!(blocked.recovery_effects(), 0);
        assert_eq!(
            blocked.cause().limit().map(|limit| limit.dimension()),
            Some(PhysicalRecoveryLimitDimension::RecoveryMemoryBytes)
        );

        let exact_ceiling = peak.max(retained_bytes);
        let PhysicalRecoveryOutcome::Recovered(handoff) = recover(retained.path(), exact_ceiling)
        else {
            panic!("exact peak ceiling ({exact_ceiling}) must complete recovery");
        };
        assert!(handoff.into_core().into_serving_handoff().is_some());
    }
}
