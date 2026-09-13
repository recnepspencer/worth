use worth_proof::TransitionOutcome;

use crate::physical_runtime::{
    instance::PhysicalStoreInstanceFoundation, MediaOwnedPhysicalRuntime,
};

use super::super::{
    admission::bootstrap::BootstrapTransitionFailure, PhysicalRecordInitialization,
    PhysicalRecordOpen, RecordAllocationFrontier, RecordBootstrapFailure,
    RecordServingAdmissionInspectionRequired, RecordServingAdmissionRebindRequired,
    RecordServingAdmissionStale, RecordServingRebindReason, RecordStoreInitializationDenial,
    RecordStoreInitializationOutcome, RecordStoreOpenDenial, RecordStoreOpenOutcome,
    ServingPhysicalRuntime,
};
use super::{initialization, open as record_open};

pub(in crate::physical_runtime) fn initialize(
    runtime: MediaOwnedPhysicalRuntime,
    request: PhysicalRecordInitialization,
) -> RecordStoreInitializationOutcome {
    let PhysicalRecordInitialization {
        format,
        placement,
        access,
        residency: residency_policy,
        work_profile,
        durability,
    } = request;
    let durability = match crate::physical_runtime::durability::bind_policy_to_runtime(
        durability,
        runtime.record_serving_media(),
        runtime.runtime_identity(),
    ) {
        Ok(owner) => owner,
        Err(reason) => {
            return TransitionOutcome::rebind_required(RecordServingAdmissionRebindRequired::new(
                runtime,
                durability_rebind_reason(reason),
            ))
            .into()
        }
    };
    let residency = match crate::physical_runtime::instance::PhysicalResidencyOwner::admit(
        runtime.store_identity(),
        residency_policy,
    ) {
        Ok(owner) => owner,
        Err(reason) => {
            return TransitionOutcome::denied(RecordStoreInitializationDenial::new(
                runtime,
                super::super::RecordBootstrapDenial::from_residency(reason),
            ))
            .into();
        }
    };
    let frame_ports = residency.ports().clone();
    let bootstrap_allocation = match bootstrap_allocation(&frame_ports, format) {
        Ok(allocation) => allocation,
        Err(reason) => {
            return TransitionOutcome::denied(RecordStoreInitializationDenial::new(
                runtime, reason,
            ))
            .into();
        }
    };
    let bootstrap =
        match initialization::initialize(runtime.record_serving_media(), format, placement, access)
        {
            Ok(bootstrap) => bootstrap,
            Err(BootstrapTransitionFailure::Denied(reason)) => {
                return TransitionOutcome::denied(RecordStoreInitializationDenial::new(
                    runtime, reason,
                ))
                .into();
            }
            Err(BootstrapTransitionFailure::Failed(cause)) => {
                return initialization_failed(runtime, cause)
            }
            Err(BootstrapTransitionFailure::Stale(reason)) => {
                return initialization_failed(
                    runtime,
                    RecordBootstrapFailure::PublishedRootStale(reason),
                );
            }
            Err(BootstrapTransitionFailure::RebindRequired(reason)) => {
                return initialization_failed(
                    runtime,
                    RecordBootstrapFailure::PublishedRootRebindRequired(reason),
                );
            }
        };
    match record_open::load_current_root(
        runtime.record_serving_media(),
        frame_ports.loader(),
        &bootstrap_allocation,
        bootstrap,
        runtime.lifecycle_state(),
        crate::physical_runtime::PhysicalRootProtocolRoute::Initialization,
        runtime.root_protocol_counter_cells(),
        frame_ports.resident_integrity_counter_cells(),
    ) {
        Ok(state) => initialize_serving(runtime, state, residency, work_profile, durability),
        Err(BootstrapTransitionFailure::Denied(reason)) => initialization_failed(
            runtime,
            RecordBootstrapFailure::PublishedRootReadmission(reason),
        ),
        Err(BootstrapTransitionFailure::Stale(reason)) => {
            initialization_failed(runtime, RecordBootstrapFailure::PublishedRootStale(reason))
        }
        Err(BootstrapTransitionFailure::RebindRequired(reason)) => initialization_failed(
            runtime,
            RecordBootstrapFailure::PublishedRootRebindRequired(reason),
        ),
        Err(BootstrapTransitionFailure::Failed(cause)) => initialization_failed(runtime, cause),
    }
}

pub(in crate::physical_runtime) fn open(
    runtime: MediaOwnedPhysicalRuntime,
    request: PhysicalRecordOpen,
) -> RecordStoreOpenOutcome {
    let PhysicalRecordOpen {
        format,
        access,
        residency: residency_policy,
        work_profile,
        durability,
    } = request;
    let durability = match crate::physical_runtime::durability::bind_policy_to_runtime(
        durability,
        runtime.record_serving_media(),
        runtime.runtime_identity(),
    ) {
        Ok(owner) => owner,
        Err(reason) => {
            return TransitionOutcome::rebind_required(RecordServingAdmissionRebindRequired::new(
                runtime,
                durability_rebind_reason(reason),
            ))
            .into()
        }
    };
    let residency = match crate::physical_runtime::instance::PhysicalResidencyOwner::admit(
        runtime.store_identity(),
        residency_policy,
    ) {
        Ok(owner) => owner,
        Err(reason) => {
            return TransitionOutcome::denied(RecordStoreOpenDenial::new(
                runtime,
                super::super::RecordBootstrapDenial::from_residency(reason),
            ))
            .into();
        }
    };
    let frame_ports = residency.ports().clone();
    let bootstrap_allocation = match bootstrap_allocation(&frame_ports, format) {
        Ok(allocation) => allocation,
        Err(reason) => {
            return TransitionOutcome::denied(RecordStoreOpenDenial::new(runtime, reason)).into();
        }
    };
    let bootstrap = match record_open::open(
        runtime.record_serving_media(),
        frame_ports.loader(),
        &bootstrap_allocation,
        format,
        access,
        runtime.lifecycle_state(),
        frame_ports.resident_integrity_counter_cells(),
    ) {
        Ok(bootstrap) => bootstrap,
        Err(failure) => return open_failure(runtime, failure),
    };
    match record_open::load_current_root(
        runtime.record_serving_media(),
        frame_ports.loader(),
        &bootstrap_allocation,
        bootstrap,
        runtime.lifecycle_state(),
        crate::physical_runtime::PhysicalRootProtocolRoute::OrdinaryOpen,
        runtime.root_protocol_counter_cells(),
        frame_ports.resident_integrity_counter_cells(),
    ) {
        Ok(state) => open_serving(runtime, state, residency, work_profile, durability),
        Err(failure) => open_failure(runtime, failure),
    }
}

fn bootstrap_allocation(
    frame_ports: &super::super::residency::frame_ports::RecordFramePorts,
    format: super::super::AdmittedPhysicalRecordFormat,
) -> Result<worth_store_buffer_pool::OperationAllocationGrant, super::super::RecordBootstrapDenial>
{
    frame_ports
        .begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery,
            std::num::NonZeroU64::new(u64::from(format.declaration().page_size().bytes()))
                .expect("an admitted physical page size is nonzero"),
        )
        .map_err(super::super::RecordBootstrapDenial::from_residency)
}

fn initialize_serving(
    runtime: MediaOwnedPhysicalRuntime,
    state: super::super::RecordServingState,
    residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    work_profile: crate::physical_runtime::PhysicalWorkProfileDeclaration,
    durability: crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
) -> RecordStoreInitializationOutcome {
    let frontier = RecordAllocationFrontier::new(&state.free_space);
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
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

fn open_serving(
    runtime: MediaOwnedPhysicalRuntime,
    state: super::super::RecordServingState,
    residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    work_profile: crate::physical_runtime::PhysicalWorkProfileDeclaration,
    durability: crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
) -> RecordStoreOpenOutcome {
    let frontier = RecordAllocationFrontier::new(&state.free_space);
    let (termination, media, core) = runtime.into_record_serving_parts();
    core.progress_to_record_serving();
    residency
        .ports()
        .invalidate_integrity_validation_for_runtime_transition();
    match ServingPhysicalRuntime::from_admission(PhysicalStoreInstanceFoundation {
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

fn durability_rebind_reason(
    reason: crate::physical_runtime::durability::PhysicalDurabilityRuntimeRebind,
) -> RecordServingRebindReason {
    match reason {
        crate::physical_runtime::durability::PhysicalDurabilityRuntimeRebind::StoreIdentityMismatch => {
            RecordServingRebindReason::PhysicalDurabilityStoreMismatch
        }
        crate::physical_runtime::durability::PhysicalDurabilityRuntimeRebind::AdmissionBasisMismatch => {
            RecordServingRebindReason::PhysicalDurabilityAdmissionBasisMismatch
        }
    }
}

fn open_failure(
    runtime: MediaOwnedPhysicalRuntime,
    failure: BootstrapTransitionFailure,
) -> RecordStoreOpenOutcome {
    match failure {
        BootstrapTransitionFailure::Denied(reason) => {
            TransitionOutcome::denied(RecordStoreOpenDenial::new(runtime, reason)).into()
        }
        BootstrapTransitionFailure::Stale(reason) => {
            TransitionOutcome::stale(RecordServingAdmissionStale::new(runtime, reason)).into()
        }
        BootstrapTransitionFailure::RebindRequired(reason) => TransitionOutcome::rebind_required(
            RecordServingAdmissionRebindRequired::new(runtime, reason),
        )
        .into(),
        BootstrapTransitionFailure::Failed(cause) => {
            let identity = runtime.runtime_identity();
            TransitionOutcome::failed(RecordServingAdmissionInspectionRequired::new(
                identity,
                runtime.abort(),
                cause,
            ))
            .into()
        }
    }
}

fn initialization_failed(
    runtime: MediaOwnedPhysicalRuntime,
    cause: RecordBootstrapFailure,
) -> RecordStoreInitializationOutcome {
    let identity = runtime.runtime_identity();
    TransitionOutcome::failed(RecordServingAdmissionInspectionRequired::new(
        identity,
        runtime.abort(),
        cause,
    ))
    .into()
}
