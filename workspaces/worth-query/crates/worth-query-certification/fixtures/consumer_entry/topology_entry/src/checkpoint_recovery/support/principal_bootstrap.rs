//! The installed principal shared by the checkpoint fixture's bootstrap callers.
use super::{external_identity, ConsumerPrincipalBinding, TopologySchemaBinding};
use worth_query_host::facade::{
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

pub(in crate::checkpoint_recovery) fn bind_model_owner<Schema: TopologySchemaBinding>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let principal = installed
        .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
        .expect("the principal mapping is installed");
    graph.bind_principal(
        &principal,
        WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
        1_u64,
        external_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )
}
