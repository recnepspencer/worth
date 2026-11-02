pub(crate) mod physical_admission;
pub(crate) mod readmission;

use crate::home_context::{
    AdmittedApplicationHome, HomeContext, ReadmittedApplicationHome, RecoveredPhysicalHome,
};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_proof::{HomeAdmitted, RelationalReadmitted, StoreRecovered};

#[derive(Debug)]
pub enum ApplicationHomeOpenRefusal {
    Execution(WorkCeilingDenial),
    Physical(physical_admission::ApplicationHomePhysicalRefusal),
    Descriptor(crate::descriptor::ApplicationHomeDescriptorDenial),
    Checkpoint(crate::checkpoint::ApplicationHomeCheckpointDenial),
    RuntimeConstruction(worth_relational::facade::runtime::RelationalRuntimeConstructionDenial),
    Relational(worth_store_relational::RelationalSectionReadmissionDenial),
    World(worth_runtime_world::facade::WorldReadmissionDenial),
    BridgeSource(worth_runtime_bridge::facade::RelationalBridgeSourceConfigurationError),
    Bridge(worth_runtime_bridge::facade::BridgeBuildError),
    ConditionalBridge(worth_runtime_bridge::facade::BridgeConditionalDenial),
}

pub fn recover_home(
    request: crate::ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<StoreRecovered<RecoveredPhysicalHome>, ApplicationHomeOpenRefusal> {
    recover_home_observed(request, execution, |_| {})
}

pub(crate) fn recover_home_observed(
    request: crate::ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
    observed: impl FnOnce(&worth_store::physical_runtime::ServingPhysicalRuntime),
) -> Result<StoreRecovered<RecoveredPhysicalHome>, ApplicationHomeOpenRefusal> {
    #[cfg(feature = "certification-test-authority")]
    let mut request = request;
    execution
        .consult()
        .map_err(ApplicationHomeOpenRefusal::Execution)?;
    let opening = physical_admission::PhysicalOpening::production();
    #[cfg(feature = "certification-test-authority")]
    let opening = request.controls.opening.take().unwrap_or(opening);
    let opened = physical_admission::open(&request.location, opening)
        .map_err(ApplicationHomeOpenRefusal::Physical)?;
    observed(&opened.serving);
    let recovered = RecoveredPhysicalHome {
        serving: Arc::new(opened.serving),
        origin: opened.origin,
        request,
    };
    execution
        .consult()
        .map_err(ApplicationHomeOpenRefusal::Execution)?;
    Ok(StoreRecovered::from_owner(recovered))
}

pub fn admit_home(
    recovered: StoreRecovered<RecoveredPhysicalHome>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<HomeAdmitted<AdmittedApplicationHome>, ApplicationHomeOpenRefusal> {
    recovered.consume(|recovered| {
        // Release must not need a fresh allocation after checkpoint exhaustion.
        // Admit the one actual cold result slot before descriptor effects.
        let world_close_checkpoint =
            crate::close::checkpoint_result::HomeCloseCheckpointResult::prepare(execution)
                .map_err(ApplicationHomeOpenRefusal::Execution)?;
        let crate::ApplicationHomeRequest {
            location,
            relational,
            binding,
            build,
            #[cfg(feature = "certification-test-authority")]
            controls,
        } = recovered.request;
        let binding = binding.consume(|admitted| admitted.into_description());
        let descriptor = crate::descriptor::admit(
            &recovered.serving,
            recovered.origin,
            &relational,
            binding,
            build,
            execution,
        )
        .map_err(ApplicationHomeOpenRefusal::Descriptor)?;
        let checkpoint = Arc::new(Default::default());
        let world_slot = crate::world_section::UnclaimedWorldSlot::from_admitted_home(
            Arc::clone(&recovered.serving),
            &descriptor,
            &checkpoint,
        );
        Ok(HomeAdmitted::from_owner(AdmittedApplicationHome {
            context: HomeContext {
                selected_home_source: None,
                initialization: crate::relational_preparation::HomeInitializationOrigin::Available(
                    recovered.origin,
                ),
                world_publications: world_slot.publications(),
                world_close_report: None,
                world_close_checkpoint: std::sync::Mutex::new(world_close_checkpoint),
                world_slot: Some(world_slot),
                checkpoint,
                location,
                descriptor,
                serving: recovered.serving,
            },
            definition: relational,
            #[cfg(feature = "certification-test-authority")]
            read_observer: controls.relational,
        }))
    })
}

pub fn readmit_relational(
    admitted: HomeAdmitted<AdmittedApplicationHome>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<RelationalReadmitted<ReadmittedApplicationHome>, crate::RelationalHomeOpenRefused> {
    let attached = crate::relational_preparation::attach_relational(admitted, execution)
        .map_err(crate::RelationalHomeOpenRefused::Attachment)?;
    attached
        .readmit_source_for_home(execution)
        .map(|proof| proof.consume(|home| home.share()))
        .map_err(crate::RelationalHomeOpenRefused::Readmission)
}
