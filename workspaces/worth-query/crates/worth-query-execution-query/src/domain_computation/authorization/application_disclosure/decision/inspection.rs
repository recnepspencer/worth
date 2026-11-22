//! Metadata facets consume an Inspection subject, never a Query or Resource binding.
use super::*;
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget;
use std::sync::Arc;

pub fn prepare_application_inspection_governance(
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
) -> Result<WorthQueryPreparedApplicationGovernance, WorthQueryApplicationQueryGovernanceDenialKind>
{
    preparation::prepare(contract, pending)
}

impl WorthQueryPreparedApplicationGovernance {
    pub fn bind_inspection(
        self,
        binding: WorthQueryApplicationGovernanceBinding,
    ) -> Result<WorthQueryApplicationQueryGovernance, WorthQueryApplicationQueryGovernanceDenialKind>
    {
        if !binding.is_inspection() {
            return Err(WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch);
        }
        Ok(self.bind(binding))
    }
}

impl WorthQueryApplicationQueryGovernance {
    pub fn inspection_matches(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        target: &Arc<WorthQueryApplicationInspectionTarget>,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        match self {
            Self::Public => graph_work.inspection_binding().is_some_and(|(actual, _)| {
                actual
                    .as_any()
                    .downcast_ref::<WorthQueryApplicationInspectionTarget>()
                    .is_some_and(|actual| {
                        actual.admits_member_subject(target, scope)
                            && graph_work.principal() == principal
                            && graph_work.entity_access_context() == Some(actual.original_subject())
                    })
            }),
            Self::Governed { computation, .. } => computation
                .binding
                .matches_inspection(graph_work, target, parameters, principal, scope),
        }
    }
}

mod court_doors_for_test;
