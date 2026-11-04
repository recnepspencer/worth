use std::sync::Arc;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResourceProfile;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionInstallationCommitDenial;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryInstalledDomainExecutionAuthority;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
pub fn empty_runtime(
) -> worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0
}
