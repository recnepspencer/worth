mod native_source;
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
    UncleanRecoveryUnsupported(worth_foundational::facade::UncleanRecoveryUnsupported),
    Physical(physical_admission::ApplicationHomePhysicalRefusal),
    Descriptor(crate::descriptor::ApplicationHomeDescriptorDenial),
    Checkpoint(crate::checkpoint::ApplicationHomeCheckpointDenial),
    Transition(readmission::transition::ApplicationHomeTransitionIntentDenial),
    RuntimeConstruction(worth_relational::facade::runtime::RelationalRuntimeConstructionDenial),
    Relational(worth_store_relational::RelationalSectionReadmissionDenial),
    World(worth_runtime_world::facade::WorldReadmissionDenial),
    SignalSourceRequired,
    SignalMedium(worth_signal::facade::durability::SignalDurableMediumDenial),
    SignalImage(Box<worth_signal::facade::durability::SignalOwnerReadmissionDenial>),
    SignalInitial(Box<worth_signal::facade::durability::SignalInitialSourceDenial>),
    SignalInitialProgress(Box<crate::signal_medium::HomeSignalInitialSourceDenial>),
    SignalReadmission(Box<worth_signal::facade::durability::SignalOwnerReadmissionDenial>),
    SignalAttachment(Box<worth_runtime_world::facade::RuntimeWorldAuthoredSignalAttachmentDenial>),
    BridgeInitial(Box<worth_runtime_bridge::facade::BridgeInitialSourceDenial>),
    AuthoredBridge(Box<worth_runtime_bridge::facade::BridgeAuthoredReadmissionDenial>),

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
    let serving = Arc::new(opened.serving);
    request.opening_observation.bind_physical(&serving);
    let recovered = RecoveredPhysicalHome {
        serving,
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
            opening_observation,
            predecessor,
            #[cfg(feature = "certification-test-authority")]
            mut controls,
        } = recovered.request;
        let binding = binding.consume(|admitted| admitted.into_description());
        let selected_source =
            crate::checkpoint::SelectedHomeSource::capture(&recovered.serving, execution)
                .map_err(ApplicationHomeOpenRefusal::Checkpoint)?;
        use crate::{ApplicationHomeOpeningEdge as Edge, ApplicationHomeOpeningStage as Stage};
        opening_observation.arrive_for_test(Stage::DescriptorIncarnation, Edge::Before);
        let prepared_descriptor = crate::descriptor::prepare_admission(
            &recovered.serving,
            recovered.origin,
            relational,
            binding,
            predecessor,
            build,
            &selected_source,
            execution,
        )?;
        let selected = selected_source
            .selected(&recovered.serving, execution)
            .map_err(ApplicationHomeOpenRefusal::Checkpoint)?;
        let native_preparation = native_source::prepare(
            &recovered.serving,
            prepared_descriptor.home_identity(),
            recovered.origin,
            &selected_source,
            selected.as_ref(),
            execution,
            #[cfg(feature = "certification-test-authority")]
            &mut controls.relational,
        )?;
        let world_data = if recovered.origin
            == worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts
        {
            let descriptor = prepared_descriptor.descriptor();
            let mut binding =
                worth_runtime_world::facade::WorldDurableHomeBinding::from_admitted_adapter(
                    crate::world_section::UnclaimedWorldSlot::owner_identity_for_home(
                        descriptor.identity,
                    ),
                    descriptor.incarnation,
                    *descriptor.binding.selected().revision().bytes(),
                );
            if let Some(lineage) = prepared_descriptor
                .world_program_lineage(execution)
                .map_err(ApplicationHomeOpenRefusal::Transition)?
            {
                binding = binding
                    .with_recorded_program_lineage(lineage)
                    .map_err(ApplicationHomeOpenRefusal::World)?;
            }
            let source = crate::world_section::checkpoint::prepare_source(
                &selected_source,
                &recovered.serving,
                std::sync::Weak::new(),
                execution,
            )
            .map_err(|cause| {
                ApplicationHomeOpenRefusal::World(
                    worth_runtime_world::facade::WorldReadmissionDenial::Medium(cause),
                )
            })?;
            Some(
                worth_runtime_world::facade::PreparedWorldHomeData::from_home_source(
                    binding, source, execution,
                )
                .map_err(ApplicationHomeOpenRefusal::World)?,
            )
        } else {
            None
        };
        let native_source = native_preparation.finish(
            world_data.as_ref().and_then(|data| data.native_selection()),
            execution,
        )?;
        let crate::descriptor::PublishedHomeDescriptor {
            descriptor,
            definition: relational,
            transition: transition_admission,
        } = crate::descriptor::publish(&recovered.serving, prepared_descriptor, execution)
            .map_err(ApplicationHomeOpenRefusal::Descriptor)?;
        opening_observation.arrive_for_test(Stage::DescriptorIncarnation, Edge::After);
        let checkpoint = Arc::new(Default::default());
        let transition = readmission::transition::HomeTransitionOwner::new(
            descriptor.identity,
            descriptor.incarnation.fence(),
            transition_admission,
            execution,
        )
        .map_err(ApplicationHomeOpenRefusal::Execution)?;
        let world_slot = crate::world_section::UnclaimedWorldSlot::from_admitted_home(
            Arc::clone(&recovered.serving),
            &descriptor,
            world_data.as_ref().map(|data| data.home_binding().clone()),
            transition.authority(),
            transition.reserved_attempt_frontier(),
            &checkpoint,
            execution,
        )
        .map_err(ApplicationHomeOpenRefusal::Execution)?;
        let query_obligations =
            Arc::new(crate::query_obligations::ApplicationQueryObligations::new(
                &recovered.serving,
                descriptor.identity,
                descriptor.incarnation,
            ));
        let query_corrections =
            Arc::new(crate::query_corrections::ApplicationQueryCorrections::new(
                &recovered.serving,
                descriptor.identity,
                descriptor.incarnation,
            ));
        let query_dispatch = Arc::new(crate::query_dispatch::ApplicationHomeDispatchJournal::new(
            &recovered.serving,
            descriptor.identity,
            descriptor.incarnation,
        ));
        let mut context = HomeContext {
            query_corrections,
            query_obligations,
            query_dispatch,
            signal_close_probe: std::sync::Mutex::new(None),
            world_preparation: None,
            opening_observation,
            transition,
            selected_home_source: Some(selected_source),
            initialization: crate::relational_preparation::HomeInitializationOrigin::Available(
                recovered.origin,
            ),
            world_publications: world_slot.publications(),
            world_close_report: None,
            world_origin_retirement: None,
            world_owner_retirement: None,
            signal_owner_retirement: None,
            world_close_checkpoint: std::sync::Mutex::new(world_close_checkpoint),
            world_slot: Some(world_slot),
            checkpoint,
            location,
            descriptor,
            serving: recovered.serving,
        };
        if let Some(data) = world_data {
            context.world_preparation = Some(
                crate::world_section::native_selection::HomeWorldPreparation::bind(
                    &context, data, execution,
                )
                .map_err(ApplicationHomeOpenRefusal::World)?,
            );
        }
        Ok(HomeAdmitted::from_owner(AdmittedApplicationHome {
            native_source,
            context,
            definition: relational,
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

impl From<crate::descriptor::ApplicationHomeDescriptorDenial> for ApplicationHomeOpenRefusal {
    fn from(cause: crate::descriptor::ApplicationHomeDescriptorDenial) -> Self {
        Self::Descriptor(cause)
    }
}
