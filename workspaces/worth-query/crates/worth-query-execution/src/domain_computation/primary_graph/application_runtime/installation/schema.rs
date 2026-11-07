//! Validate installation schema affinity without erasing its original cause.
use super::{
    ApplicationSchema, WorthQueryExecutionRuntime, WorthQueryInstalledApplicationSchema,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
pub(in crate::domain_computation::primary_graph) fn validate_application_schema<Schema>(
    runtime: &WorthQueryExecutionRuntime,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    runtime
        .installed_packages()
        .validate_application_schema(installed_schema)
        .map_err(|denial| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                denial.subject(),
            )
            .with_schema_cause(denial)
        })
}
