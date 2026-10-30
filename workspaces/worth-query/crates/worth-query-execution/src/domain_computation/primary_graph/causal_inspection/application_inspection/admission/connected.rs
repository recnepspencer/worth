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

pub(super) struct SourceRead<'scope> {
    pub(super) scope: &'scope dyn super::source_scope::InspectionSourceScope,
    pub(super) policy: BoundSourcePolicy,
}

/// Owned policy custody may cross release only after the read's final proof.
pub(super) struct BoundSourcePolicy {
    pub(super) target: Arc<WorthQueryApplicationInspectionTarget>,
    pub(super) authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    pub(super) governance: WorthQueryApplicationQueryGovernance,
    pub(super) diagnostic: WorthQueryApplicationQueryGovernance,
}
impl SourceRead<'_> {
    pub(super) fn fact_count(&self) -> usize {
        self.policy.authorization.exact_fact_count()
            + self
                .policy
                .governance
                .authorization()
                .map_or(0, |v| v.exact_fact_count())
            + self
                .policy
                .diagnostic
                .authorization()
                .map_or(0, |v| v.exact_fact_count())
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_source<'scope>(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    session: &WorthQueryManagedGraphWorkSession,
    root: &Arc<WorthQueryApplicationInspectionTarget>,
    principal: EntityId,
    scope: &'scope dyn super::source_scope::InspectionSourceScope,
    policy: PreparedInspectionPolicy,
) -> Result<SourceRead<'scope>, InspectionAdmissionDenial> {
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
            scope.entity_id(),
        )?;
        policy
            .bind_inspection(binding)
            .map_err(InspectionAdmissionDenial::Governance)
    };
    Ok(SourceRead {
        scope,
        policy: BoundSourcePolicy {
            target: Arc::clone(target),
            authorization: policy.authorization,
            governance: bind(policy.metadata, memory)?,
            diagnostic: bind(
                policy
                    .diagnostic
                    .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?,
                memory,
            )?,
        },
    })
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PendingInspectionPolicies
{
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) metadata:
        Option<WorthQueryPendingApplicationQueryGovernance>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) diagnostic:
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
