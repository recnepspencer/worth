//! The ordinary temporal contribution installed through the real Home opener.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub fn request(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        (publication_limit::configuration(),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryApplicationLimits::new(
            product_world_resources(1_024).with_execution_policy(
                runtime::ExecutionRequestPolicy::new(
                    runtime::ExecutionPosture::Serial,
                    runtime::DeterminismContract::CanonicalBitwise,
                    runtime::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
                ),
            ),
            runtime::WorthQueryApplicationQueryResourceProfile::default(),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "ready", 0, 1, true);
            Ok(())
        },
    )
}
