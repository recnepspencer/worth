//! Construct the relational backend under the admitted open limits.
use super::super::WorthQueryApplicationLimits;
use super::contribution_configuration::ConfiguredInstallation;
use worth_query_installation::facade::ApplicationSchema;

pub(super) struct BackendInstallation<Schema: ApplicationSchema> {
    pub(super) configured: ConfiguredInstallation<Schema>,
    pub(super) relational_runtime: worth_relational::facade::runtime::RelationalRuntime,
}

pub(super) fn construct<Schema: ApplicationSchema>(
    configured: ConfiguredInstallation<Schema>,
    limits: &WorthQueryApplicationLimits,
) -> BackendInstallation<Schema> {
    let mut relational_builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .profile(limits.profile.relational_profile());
    if let Some(publication) = limits
        .profile
        .publication_override(limits.maximum_publication_records)
    {
        relational_builder = relational_builder.publication(publication);
    }
    if let Some(scope_budget) = limits.profile.relation_integrity_scope_budget() {
        relational_builder = relational_builder.relation_integrity_scope_budget(scope_budget);
    }
    BackendInstallation {
        configured,
        relational_runtime: relational_builder.build(),
    }
}
