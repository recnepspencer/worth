//! Concrete, distinct recovery custody payloads. None constructs application service.
use std::sync::Arc;
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
    pub(crate) context: HomeContext,
    pub(crate) definition: RelationalRuntimeDefinition,
    #[cfg(feature = "certification-test-authority")]
    pub(crate) read_observer: Option<
        Box<
            dyn Fn(
                    worth_relational::facade::publication::PatchStreamPosition,
                    worth_store_relational::RelationalStoreColdReadCounters,
                ) + Send
                + Sync,
        >,
    >,
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
    pub(crate) initialization: crate::relational_preparation::HomeInitializationOrigin,
    pub(crate) world_publications: Arc<crate::world_section::WorldPublications>,
    pub(crate) world_close_report: Option<worth_runtime_world::facade::RuntimeWorldCloseReport>,
    pub(crate) world_slot: Option<crate::world_section::UnclaimedWorldSlot>,
    pub(crate) checkpoint: Arc<crate::checkpoint::owner::HomeCheckpointOwner>,
    pub(crate) location: std::path::PathBuf,
    pub(crate) descriptor: crate::descriptor::HomeDescriptor,
    pub(crate) serving: Arc<ServingPhysicalRuntime>,
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
