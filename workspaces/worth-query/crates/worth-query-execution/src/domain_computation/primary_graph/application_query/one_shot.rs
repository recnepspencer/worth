use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod admitted;
mod correction_disclosure;
mod dependent_pair;
pub use dependent_pair::WorthQueryDependentPairReadContext;
mod batch;
pub use batch::WorthQueryApplicationBatchReadDenial;
pub use batch::WorthQueryApplicationBatchResult;

mod denial;
pub(super) mod outcome;
pub(super) mod plan_admission;
use plan_admission::admit_request;
use plan_admission::validate_authentication_lifetime;
use plan_admission::validate_basis_lifetime;
use plan_admission::validate_one_shot_plan;
pub(super) mod pair_plans;
mod result;
pub use pair_plans::WorthQueryDerivedPairReadPlans;

use denial::authorization_denial;
use denial::denial;
pub(in crate::domain_computation::primary_graph) use denial::retain_reconstruction_result;
pub use denial::WorthQueryApplicationOneShotDenial;
pub use denial::WorthQueryApplicationOneShotDenialKind;

use super::authorized_read::execute_authorized_read;
use super::authorized_read::refresh_governed_authorization;
use super::authorized_read::SelectedAuthorizationCurrentnessStop;
use super::authorized_read::SelectedAuthorizedReadStop;
use super::authorized_read::WorthQueryAuthorizedApplicationReadDenial;
use super::read_execution::read_bounded_root_rows;
use super::read_execution::OneShotReadWorkObservation;
use super::read_execution::WorthQueryApplicationReadExecutionDenialKind;
use super::WorthQueryAdmittedApplicationQueryControls;
use super::WorthQueryAdmittedApplicationQueryPlan;
use super::WorthQueryApplicationProjection;
use super::WorthQueryApplicationQueryAccessReceipt;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use outcome::finalize_one_shot;

/// The complete result of a one-shot query read: projected rows, the observed
/// sources and result set, and the access receipt.
///
/// The observed sources can later be bound as a mutation's source expectation.
pub struct WorthQueryApplicationOneShotResult<Query, QueryResult> {
    rows: Vec<QueryResult>,
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    result_set_observation: super::WorthQueryObservedResultSet<Query>,
    request_affinity: super::admitted_result::WorthQueryApplicationQueryRequestAffinity,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

/// The admitted read keeps its original semantic denial and accounts for
/// newly performed Query execution work on either outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryAdmittedOneShotStop {
    Admission(CompanionPreflightStop),

    WorkCounterOverflow,
    WorkAccountingMismatch,
    Execution(WorthQueryApplicationOneShotDenial),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn execute_application_query_one_shot<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        let request = match phase.execution_request_for(&self.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(authorization_denial(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody"), plan.query.name())),
        };
        let execution = &request;
        self.execute_application_query_one_shot_with_request(execution, plan)
    }

    pub(in crate::domain_computation) fn execute_application_query_one_shot_with_request<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        self.execute_application_query_one_shot_core(execution, plan, None)
    }

    fn execute_application_query_one_shot_core<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        spent: Option<&OneShotReadWorkObservation<'_>>,
    ) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        validate_one_shot_plan(execution, self, &plan)?;
        refresh_governed_authorization(execution, self, &mut plan)
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;

        let (raw, authorization_work, read_proof) =
            execute_authorized_read(execution, self, &plan, |runtime, graph, plan| {
                read_bounded_root_rows(runtime, execution, graph, plan, spent)
            })
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        finalize_one_shot(execution, self, plan, raw, authorization_work, read_proof)
    }
}

pub(super) fn map_authorized_read_denial(
    denial_value: WorthQueryAuthorizedApplicationReadDenial,
    query: &str,
) -> WorthQueryApplicationOneShotDenial {
    let (kind, subject) = match denial_value {
        WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => (
            WorthQueryApplicationOneShotDenialKind::StalePrincipal,
            query.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::StaleScope
        | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => (
            WorthQueryApplicationOneShotDenialKind::StaleScope,
            query.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::Authorization(denial) => {
            return authorization_denial(denial, query);
        }
        WorthQueryAuthorizedApplicationReadDenial::Read(read) => {
            let kind = match read.kind() {
                WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(kind) => {
                    WorthQueryApplicationOneShotDenialKind::ExecutionRequest(kind)
                }
                WorthQueryApplicationReadExecutionDenialKind::Cancelled => {
                    WorthQueryApplicationOneShotDenialKind::Cancelled
                }
                WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded => {
                    WorthQueryApplicationOneShotDenialKind::DeadlineExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable => {
                    WorthQueryApplicationOneShotDenialKind::PredicateIndexUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::ResultLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch => {
                    WorthQueryApplicationOneShotDenialKind::CardinalityMismatch
                }
                WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable => {
                    WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityLookupOverflow
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound => {
                    WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationGenerationChanged
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationPageWidthInvalid => {
                    WorthQueryApplicationOneShotDenialKind::TraversalUnavailable
                }
            };
            return denial(kind, query, read.into_subject());
        }
        WorthQueryAuthorizedApplicationReadDenial::Session => (
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            query.to_string(),
        ),
    };
    denial(kind, query, subject)
}
