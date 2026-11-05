use std::path::{Path, PathBuf};

use worth_proof::AdmittedApplicationHomeBinding;
use worth_query_installation::facade::WorthQueryHomeBindingAdmission;
use worth_relational::facade::runtime::RelationalRuntimeDefinition;

/// Declared owner construction inputs. A descriptive binding or a raw digest
/// cannot stand in for Query's installed-schema/closed-roster admission.
pub struct ApplicationHomeRequest {
    pub(crate) location: PathBuf,
    pub(crate) relational: RelationalRuntimeDefinition,
    pub(crate) binding: AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
    pub(crate) build: worth_foundational::facade::ApplicationBuildIdentity,
    pub(crate) predecessor: Option<HomeTransitionPredecessorInput>,
    #[cfg(feature = "certification-test-authority")]
    pub(crate) controls: crate::home_controls_for_test::HomeControlsForTest,
}

impl ApplicationHomeRequest {
    pub fn new(
        location: PathBuf,
        relational: RelationalRuntimeDefinition,
        binding: AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
        build: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Self {
        Self {
            location,
            relational,
            binding,
            build,
            predecessor: None,
            #[cfg(feature = "certification-test-authority")]
            controls: Default::default(),
        }
    }

    pub fn location(&self) -> &Path {
        &self.location
    }

    /// Supply the compiled predecessor admitted by Query for transition planning.
    /// This does not authorize a migration effect or change the recorded binding.
    pub fn with_program_transition_predecessor(
        mut self,
        relational: RelationalRuntimeDefinition,
        binding: AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
    ) -> Self {
        self.predecessor = Some(HomeTransitionPredecessorInput {
            relational,
            binding,
        });
        self
    }
}

pub(crate) struct HomeTransitionPredecessorInput {
    pub(crate) relational: RelationalRuntimeDefinition,
    pub(crate) binding: AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
}

impl std::fmt::Debug for ApplicationHomeRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationHomeRequest")
            .field("location", &self.location)
            .field("relational", &self.relational)
            .field("binding", &self.binding)
            .field("build", &self.build)
            .finish_non_exhaustive()
    }
}
