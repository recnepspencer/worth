//! Installation of the primary provider's graph participation authority.

use std::sync::Arc;

use worth_foundational::facade::TruthPartitionRole;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use crate::domain_computation::execution_runtime::WorthQueryExecutionInstallationAuthority;
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn install(
    authority: &WorthQueryExecutionInstallationAuthority,
    truth_partition_role: Option<TruthPartitionRole>,
    provider_anchor: Arc<WorthQueryGraphProviderAnchor>,
) -> Result<WorthQueryInstalledGraphParticipationAuthority, WorthQueryPrimaryGraphInstallationDenial>
{
    WorthQueryInstalledGraphParticipationAuthority::install_with_truth_partition(
        authority.installation_runtime(),
        "primary",
        provider_anchor.provider_identity(),
        true,
        Some("primary"),
        truth_partition_role,
        provider_anchor,
    )
    .map_err(|detail| {
        WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
            detail,
        )
    })
}
