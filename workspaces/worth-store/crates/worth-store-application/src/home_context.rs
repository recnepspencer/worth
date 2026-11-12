//! Concrete, distinct recovery custody payloads. None constructs application service.
use std::sync::{Arc, Mutex};
use worth_proof::AuthorityMarker;
use worth_relational::facade::runtime::RelationalRuntimeDefinition;
use worth_store::physical_runtime::{RecoveryStorePresence, ServingPhysicalRuntime};
use worth_store_relational::RelationalStoreSection;

/// Concrete physical custody cannot be extracted and rewrapped as descriptor
/// admission: the next port requires its different private payload.
///
/// ```compile_fail,E0308
/// use worth_proof::{HomeAdmitted, StoreRecovered};
/// use worth_store_application::{RecoveredPhysicalHome, readmit_relational};
/// use worth_execution::ExecutionRequest;
/// fn skip(proof: StoreRecovered<RecoveredPhysicalHome>, request: &ExecutionRequest<'_, '_>) {
///     let forged = proof.consume(HomeAdmitted::from_owner);
///     let _ = readmit_relational(forged, request);
/// }
/// ```
pub struct RecoveredPhysicalHome {
    pub(crate) serving: Arc<ServingPhysicalRuntime>,
    pub(crate) origin: RecoveryStorePresence,
    pub(crate) request: crate::ApplicationHomeRequest,
}
impl AuthorityMarker for RecoveredPhysicalHome {}

pub struct AdmittedApplicationHome {
    pub(crate) native_source: RelationalHomeOpeningInput,
    pub(crate) context: HomeContext,
    pub(crate) definition: RelationalRuntimeDefinition,
}
pub(crate) enum RelationalHomeOpeningInput {
    Fresh(worth_store_relational::PreparedFreshRelationalHomeSource),
    Existing(worth_store_relational::PreparedRelationalHomeSource),
}
impl AuthorityMarker for AdmittedApplicationHome {}

impl AdmittedApplicationHome {
    /// Choose terminal close before any Native runtime has been constructed.
    /// The original definition is discharged only by this consuming choice;
    /// physical refusal retains the same Home lease in recovery custody.
    pub fn close(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> crate::ApplicationHomeCloseOutcome {
        let Self {
            context,
            definition,
            ..
        } = self;
        drop(definition);
        crate::ApplicationHomeRecoveryCustody::from_never_attached_home(context)
            .try_release(request)
    }
}

/// Native recovery alone cannot construct application service.
///
/// ```compile_fail,E0599
/// use worth_store_application::ReadmittedApplicationHome;
/// fn premature(home: ReadmittedApplicationHome) { home.application_service(); }
/// ```
pub struct ReadmittedApplicationHome {
    pub(crate) relational: RelationalStoreSection,
    pub(crate) context: HomeContext,
}
impl AuthorityMarker for ReadmittedApplicationHome {}

pub(crate) struct HomeContext {
    pub(crate) query_corrections: Arc<crate::query_corrections::ApplicationQueryCorrections>,
    pub(crate) query_obligations: Arc<crate::query_obligations::ApplicationQueryObligations>,
    pub(crate) query_dispatch: Arc<crate::query_dispatch::ApplicationHomeDispatchJournal>,
    pub(crate) signal_close_probe: Mutex<Option<std::sync::Weak<crate::StoreSignalMedium>>>,
    pub(crate) world_preparation:
        Option<crate::world_section::native_selection::HomeWorldPreparation>,
    pub(crate) opening_observation: crate::ApplicationHomeOpeningObservation,
    pub(crate) transition: crate::open::readmission::transition::HomeTransitionOwner,
    pub(crate) selected_home_source: Option<crate::checkpoint::SelectedHomeSource>,
    pub(crate) initialization: crate::relational_preparation::HomeInitializationOrigin,
    pub(crate) world_publications: Arc<crate::world_section::WorldPublications>,
    pub(crate) world_origin_retirement:
        Option<worth_runtime_world::facade::WorldCheckpointOriginRetirement>,
    pub(crate) signal_owner_retirement:
        Option<worth_signal::facade::durability::SignalRetainedAuthoredHomeAllocation>,
    pub(crate) world_owner_retirement:
        Option<worth_runtime_world::facade::RuntimeWorldHomeRetirement>,
    pub(crate) world_close_report: Option<worth_runtime_world::facade::RuntimeWorldCloseReport>,
    // The cached failure may own a Send-only original physical write source.
    // Exclusive close accesses use get_mut; no gate is held across callbacks.
    pub(crate) world_close_checkpoint:
        Mutex<crate::close::checkpoint_result::HomeCloseCheckpointResult>,
    pub(crate) world_slot: Option<crate::world_section::UnclaimedWorldSlot>,
    pub(crate) checkpoint: Arc<crate::checkpoint::owner::HomeCheckpointOwner>,
    pub(crate) location: std::path::PathBuf,
    pub(crate) descriptor: crate::descriptor::HomeDescriptor,
    pub(crate) serving: Arc<ServingPhysicalRuntime>,
}

impl HomeContext {
    pub(crate) fn selected_home_manifest(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<crate::checkpoint::SelectedHomeManifest>,
        crate::ApplicationHomeCheckpointDenial,
    > {
        if self.selected_home_source.is_none() {
            self.selected_home_source = Some(crate::checkpoint::SelectedHomeSource::capture(
                &self.serving,
                request,
            )?);
        }
        self.selected_home_source
            .as_ref()
            .expect("original selected source retained before admission")
            .selected(&self.serving, request)
    }
}

impl ReadmittedApplicationHome {
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.context.descriptor.identity
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.context.descriptor.incarnation
    }
    pub fn binding(&self) -> &worth_foundational::facade::ApplicationHomeBinding {
        &self.context.descriptor.binding
    }

    pub fn build_identity(&self) -> worth_foundational::facade::ApplicationBuildIdentity {
        self.context.descriptor.build
    }

    /// Native component access for the next owner readmission. This is not an
    /// application service or a World/Query recovery proof.
    pub fn relational(&self) -> &RelationalStoreSection {
        &self.relational
    }
}
