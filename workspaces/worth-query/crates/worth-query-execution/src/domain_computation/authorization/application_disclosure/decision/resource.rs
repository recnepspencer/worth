//! Resource facets use the same actual capability decision on a Resource session.
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryInstalledResourceAccessContract, WorthQueryResourceAccessPurpose,
};

pub(in crate::domain_computation) fn admit_application_resource_governance(
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    binding: WorthQueryApplicationGovernanceBinding,
) -> Result<WorthQueryApplicationQueryGovernance, WorthQueryApplicationQueryGovernanceDenialKind> {
    if !binding.is_resource() {
        return Err(WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch);
    }
    admit_application_query_governance(contract, pending, binding)
}

impl WorthQueryApplicationQueryGovernance {
    pub(in crate::domain_computation) fn resource_matches(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        contract: &Arc<WorthQueryInstalledResourceAccessContract>,
        purpose: WorthQueryResourceAccessPurpose,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        match self {
            Self::Public => graph_work
                .resource_read_binding()
                .is_some_and(|(actual, _)| {
                    Arc::ptr_eq(actual, contract) && actual.purpose() == purpose
                }),
            Self::Governed { computation, .. } => computation
                .binding
                .matches_resource(graph_work, contract, purpose, parameters, principal, scope),
        }
    }
}
