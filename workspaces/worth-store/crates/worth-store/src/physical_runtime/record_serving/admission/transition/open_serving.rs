use worth_proof::TransitionOutcome;

use super::super::super::{
    RecordAllocationFrontier, RecordStoreInitializationOutcome, RecordStoreOpenOutcome,
    ServingPhysicalRuntime,
};
use crate::physical_runtime::{
    instance::PhysicalStoreInstanceFoundation, MediaOwnedPhysicalRuntime,
};

pub(super) fn initialize_serving(
    runtime: MediaOwnedPhysicalRuntime,
    state: super::super::super::RecordServingState,
    residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    work_profile: crate::physical_runtime::PhysicalWorkProfileDeclaration,
    durability: crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
    read_protection: crate::physical_runtime::stability::PhysicalReadProtectionOwner,
) -> RecordStoreInitializationOutcome {
    let frontier = RecordAllocationFrontier::new(&state.free_space);
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
        recovered_checkpoint_custody: None,
        checkpoint_custody_origin:
            crate::physical_runtime::durability::CheckpointCustodyOrigin::FreshGenesis,
        read_protection,
        termination,
        media,
        core,
        bootstrap: state,
        allocation_frontier: frontier,
        residency,
        work_profile,
        durability,
    }) {
        Ok(serving) => TransitionOutcome::success(serving).into(),
        Err(failure) => TransitionOutcome::failed(failure).into(),
    }
}

pub(super) fn open_serving(
    runtime: MediaOwnedPhysicalRuntime,
    state: super::super::super::RecordServingState,
    residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    work_profile: crate::physical_runtime::PhysicalWorkProfileDeclaration,
    durability: crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
    read_protection: crate::physical_runtime::stability::PhysicalReadProtectionOwner,
    recovered_checkpoint_custody: Option<
        crate::physical_runtime::durability::PreparedRecoveredCheckpointCustody,
    >,
) -> RecordStoreOpenOutcome {
    let frontier = RecordAllocationFrontier::new(&state.free_space);
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
        recovered_checkpoint_custody,
        checkpoint_custody_origin:
            crate::physical_runtime::durability::CheckpointCustodyOrigin::ReopenRequiresC8,
        read_protection,
        termination,
        media,
        core,
        bootstrap: state,
        allocation_frontier: frontier,
        residency,
        work_profile,
        durability,
    }) {
        Ok(serving) => TransitionOutcome::success(serving).into(),
        Err(failure) => TransitionOutcome::failed(failure).into(),
    }
}
