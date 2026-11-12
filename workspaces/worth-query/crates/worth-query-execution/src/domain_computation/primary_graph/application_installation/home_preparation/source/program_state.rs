//! The admitted programme after every contribution has moved to its installed owner.
use super::super::*;
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) struct HomeServingProgramState<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) runtime:
        WorthQueryExecutionRuntime,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) installed:
        WorthQueryInstalledApplicationSchema<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) program:
        WorthQueryInstalledApplicationProgram<Schema, Program>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) supported:
        Vec<WorthQuerySupportedProgramRecord>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) support:
        program_admission::WorthQueryAdmittedProgramSupport<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) activation:
        WorthQueryProgramActivationCell,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) transition_installation:
        Option<
            worth_proof::AdmittedHomeProgramTransitionSuccessor<
                worth_foundational::facade::HomeProgramTransitionInstallationBinding,
            >,
        >,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) limits:
        WorthQueryApplicationLimits,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) build_identity:
        worth_foundational::facade::ApplicationBuildIdentity,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) authorization_clock:
        crate::domain_computation::authorization::WorthQueryRuntimeClock,
}
