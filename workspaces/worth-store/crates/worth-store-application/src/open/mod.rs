pub(crate) mod physical_admission;

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
        Ok(HomeAdmitted::from_owner(AdmittedApplicationHome {
            context: HomeContext {
                checkpoint: Arc::new(Default::default()),
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
) -> Result<RelationalReadmitted<ReadmittedApplicationHome>, ApplicationHomeOpenRefusal> {
    admitted.consume(|admitted| {
        execution
            .consult()
            .map_err(ApplicationHomeOpenRefusal::Execution)?;
        let manifest = crate::checkpoint::read(&admitted.context.serving)
            .map_err(ApplicationHomeOpenRefusal::Checkpoint)?;
        #[cfg(not(feature = "certification-test-authority"))]
        let relational = worth_store_relational::RelationalStoreSection::attach(
            admitted.definition,
            admitted.context.serving.clone(),
            physical_admission::placement(),
            admitted.context.checkpoint.selection_source(),
        );
        #[cfg(feature = "certification-test-authority")]
        let relational = match admitted.read_observer {
            Some(observer) => {
                worth_store_relational::RelationalStoreSection::attach_with_read_observer_for_test(
                    admitted.definition,
                    admitted.context.serving.clone(),
                    physical_admission::placement(),
                    admitted.context.checkpoint.selection_source(),
                    observer,
                )
            }
            None => worth_store_relational::RelationalStoreSection::attach(
                admitted.definition,
                admitted.context.serving.clone(),
                physical_admission::placement(),
                admitted.context.checkpoint.selection_source(),
            ),
        };
        let mut relational = relational.map_err(ApplicationHomeOpenRefusal::RuntimeConstruction)?;
        relational
            .readmit(
                manifest.as_ref().map(|manifest| manifest.relational()),
                execution,
            )
            .map_err(ApplicationHomeOpenRefusal::Relational)?;
        Ok(RelationalReadmitted::from_owner(
            ReadmittedApplicationHome {
                relational,
                context: admitted.context,
            },
        ))
    })
}
