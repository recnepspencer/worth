//! Independent source policy shares one Inspection session, never query authority.
mod materialization;
mod preparation;
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) use preparation::{prepare,prepare_query};
use super::policy::PreparedInspectionPolicy;
use crate::domain_computation::authorization::application_disclosure::{WorthQueryPreparedApplicationGovernance, WorthQueryPendingApplicationQueryGovernance};
use super::*;
use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernance;
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::EntityId;

pub(super) struct SourceRead {
    pub(super) target: Arc<WorthQueryApplicationInspectionTarget>,
    pub(super) authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    pub(super) governance: WorthQueryApplicationQueryGovernance,
    pub(super) diagnostic: WorthQueryApplicationQueryGovernance,
}
impl SourceRead {
    pub(super) fn fact_count(&self) -> usize {
        self.authorization.exact_fact_count()
            + self
                .governance
                .authorization()
                .map_or(0, |v| v.exact_fact_count())
            + self
                .diagnostic
                .authorization()
                .map_or(0, |v| v.exact_fact_count())
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_source(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    session: &WorthQueryManagedGraphWorkSession,
    root: &Arc<WorthQueryApplicationInspectionTarget>,
    principal: EntityId,
    scope: EntityId,
    policy: PreparedInspectionPolicy,
) -> Result<SourceRead, InspectionAdmissionDenial> {
    let target = root
        .connection_source()
        .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
    let bind = |policy: WorthQueryPreparedApplicationGovernance,
                memory: &mut Option<ExecutionMemoryReservation>| {
        let binding = governance_binding::capture(
            execution,
            memory,
            session,
            target,
            target.parameters(),
            principal,
            scope,
        )?;
        policy
            .bind_inspection(binding)
            .map_err(InspectionAdmissionDenial::Governance)
    };
    Ok(SourceRead {
        target: Arc::clone(target),
        authorization: policy.authorization,
        governance: bind(policy.metadata, memory)?,
        diagnostic: bind(
            policy
                .diagnostic
                .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?,
            memory,
        )?,
    })
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct ConnectionPolicies
{
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) query_metadata:
        Option<WorthQueryPendingApplicationQueryGovernance>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) query_diagnostic:
        Option<WorthQueryPendingApplicationQueryGovernance>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) source_metadata:
        Option<WorthQueryPendingApplicationQueryGovernance>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) source_diagnostic:
        WorthQueryPendingApplicationQueryGovernance,
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PreparedConnectionQuery
{
    pub(super) contract:
        worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract,
    pub(super) policy: PreparedInspectionPolicy,
    pub(super) reserved:
        crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity,
}
