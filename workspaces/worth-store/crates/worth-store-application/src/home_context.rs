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
