//! Admit the installed schema and its conditional publication inventory.
use super::{
    ApplicationSchema, WorthQueryExecutionRuntime, WorthQueryInstalledApplicationSchema,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(in crate::domain_computation::primary_graph::application_runtime) fn require_no_conditional_bindings<
    Schema,
>(
    runtime: &WorthQueryExecutionRuntime,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let count = runtime
        .installed_packages()
        .installed_conditional_node_count_for_schema(
            installed_schema.owner(),
            installed_schema.schema_name(),
        );
    if count == 0 {
        Ok(())
    } else {
        Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::ConditionalBindingsRequired,
            format!(
                "{} declared conditional nodes require the conditional publication progression",
                count
            ),
        ))
    }
}
