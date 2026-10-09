use worth_proof::TransitionOutcome;

use super::super::super::{
    RecordAllocationFrontier, RecordStoreInitializationOutcome, RecordStoreOpenOutcome,
    ServingPhysicalRuntime,
};
use crate::physical_runtime::{
    durability::{reopen_binding_compaction, CheckpointCustodyCandidate},
    instance::{OpenedCheckpointCustody, PhysicalStoreInstanceFoundation},
    MediaOwnedPhysicalRuntime,
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
    let checkpoint_custody = OpenedCheckpointCustody {
        checkpoint: reopen_binding_compaction(runtime.record_serving_media()),
        candidate: CheckpointCustodyCandidate::FreshGenesis,
        recovered: None,
    };
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
        checkpoint_custody,
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
    checkpoint_custody: OpenedCheckpointCustody,
) -> RecordStoreOpenOutcome {
    let frontier = RecordAllocationFrontier::new(&state.free_space);
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
        checkpoint_custody,
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
