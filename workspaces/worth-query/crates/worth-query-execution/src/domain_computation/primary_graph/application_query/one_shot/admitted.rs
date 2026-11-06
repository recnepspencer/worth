use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::WorthQueryAdmittedOneShotStop;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub(in crate::domain_computation::primary_graph::application_query) trait AdmittedScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn execute_application_query_one_shot_admitted_with_native_access<
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

        native: &mut super::super::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        super::outcome::CompletedOneShotRead<Query, QueryResult>,
        WorthQueryAdmittedOneShotStop,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        let spent = OneShotReadWorkObservation::new();
        super::plan_admission::validate_one_shot_plan_with_native_access(
            execution,
            self.application_query_host(),
            &plan,
            native,
        )
        .map_err(WorthQueryAdmittedOneShotStop::Execution)?;

        let (raw, authorization_work, read_proof) =
            match super::super::authorized_read::execute_selected_authorized_read_with_native_access(
                execution,
                self.application_query_host(),
                &plan,
                &spent,
                native,
            ) {
                Ok(read) => read,
                Err(SelectedAuthorizedReadStop::Read(denial)) => {
                    let result = Err(map_authorized_read_denial(
                        WorthQueryAuthorizedApplicationReadDenial::Read(denial),
                        plan.query.name(),
                    ));
                    return settle_admitted_read(&spent, result);
                }
                Err(stop) => return Err(map_selected_read_stop(stop, plan.query.name())),
            };
        let result = super::outcome::complete_one_shot(
            execution,
            self.application_query_host(),
            plan,
            raw,
            authorization_work,
            read_proof,
        );
        settle_admitted_read(&spent, result)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AdmittedScopedOps<Schema> for HostRuntime
{
}

pub(in crate::domain_computation::primary_graph::application_query) fn settle_admitted_read<
    Query,
    QueryResult,
>(
    spent: &OneShotReadWorkObservation,
    result: Result<
        super::outcome::CompletedOneShotRead<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >,
) -> Result<super::outcome::CompletedOneShotRead<Query, QueryResult>, WorthQueryAdmittedOneShotStop>
{
    if let Ok(read) = &result {
        let read_work = spent
            .read()
            .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
        if read.read.receipt().work().total_work_units() != read_work {
            return Err(WorthQueryAdmittedOneShotStop::WorkAccountingMismatch);
        }
    }
    match result {
        Ok(read) => Ok(read),
        Err(stop) => Err(WorthQueryAdmittedOneShotStop::Execution(stop)),
    }
}

pub(in crate::domain_computation::primary_graph::application_query) fn map_selected_read_stop(
    stop: SelectedAuthorizedReadStop,
    query: &str,
) -> WorthQueryAdmittedOneShotStop {
    let kind = match stop {
        SelectedAuthorizedReadStop::Currentness(SelectedAuthorizationCurrentnessStop::Admission(stop)) => return WorthQueryAdmittedOneShotStop::Admission(stop),

        SelectedAuthorizedReadStop::Currentness(SelectedAuthorizationCurrentnessStop::Execution(cause)) => {
use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
            match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(cause, None) {
                Ok(kind) => WorthQueryApplicationOneShotDenialKind::ExecutionRequest(kind),
                Err(Control::Cancelled) => WorthQueryApplicationOneShotDenialKind::Cancelled,
                Err(Control::TimedOut) => WorthQueryApplicationOneShotDenialKind::DeadlineExceeded,
            }
        }
        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop,
        ) | SelectedAuthorizedReadStop::StaleSecurity => {
            WorthQueryApplicationOneShotDenialKind::Authorization(
                crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleAuthorization),
            )
        }
        SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::StaleIssuedAccess
            | SelectedAuthorizationCurrentnessStop::StaleDecision
            | SelectedAuthorizationCurrentnessStop::UnsupportedGovernance,
        ) => WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleAuthorization),
        ),
        SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::Interrupted(
                WorthQueryRequestInterruption::Cancelled,
            ),
        ) => WorthQueryApplicationOneShotDenialKind::Cancelled,
        SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::Interrupted(
                WorthQueryRequestInterruption::DeadlineExceeded,
            ),
        ) => WorthQueryApplicationOneShotDenialKind::DeadlineExceeded,
        SelectedAuthorizedReadStop::Session => WorthQueryApplicationOneShotDenialKind::ForeignPlan,
        SelectedAuthorizedReadStop::Read(denial) => {
            return WorthQueryAdmittedOneShotStop::Execution(map_authorized_read_denial(
                WorthQueryAuthorizedApplicationReadDenial::Read(denial), query,
            ));
        }
    };
    WorthQueryAdmittedOneShotStop::Execution(denial(kind, query, query))
}
