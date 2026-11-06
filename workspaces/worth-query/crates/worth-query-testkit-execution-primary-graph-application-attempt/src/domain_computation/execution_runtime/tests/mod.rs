use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
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
