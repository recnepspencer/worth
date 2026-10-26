use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// A required cue reserves the Query read's maximum, plus its retained
    /// custody copies, on its existing carried admission before execution,
    /// then returns unused Work after observing the actual root/tree/custody
    /// phases. Only root and tree work count against the Query's declared
    /// limit. A denial settles its partial work before the caller stops the
    /// wave.
    pub(in crate::domain_computation::primary_graph) fn execute_application_query_one_shot_admitted<
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
    ) -> Result<WorthQueryApplicationOneShotResult<Query, QueryResult>, WorthQueryAdmittedOneShotStop>
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        self.execute_application_query_one_shot_admitted_with_native_access(
            execution,
            plan,
            &mut super::super::ApplicationQueryNativeAccess::acquire(),
        )
        .map(|completed| completed.read)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn execute_application_query_one_shot_admitted_with_native_access<
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
        let mut plan = plan;
        super::plan_admission::validate_one_shot_plan_with_native_access(
            execution, self, &plan, native,
        )
        .map_err(WorthQueryAdmittedOneShotStop::Execution)?;

        let (raw, authorization_work, read_proof) =
            match super::super::authorized_read::execute_selected_authorized_read_with_native_access(
                execution, self, &mut plan, &spent, native,
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
            self,
            plan,
            raw,
            authorization_work,
            read_proof,
        );
        settle_admitted_read(&spent, result)
    }
}

fn settle_admitted_read<Query, QueryResult>(
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

fn map_selected_read_stop(
    stop: SelectedAuthorizedReadStop,
    query: &str,
) -> WorthQueryAdmittedOneShotStop {
    let kind = match stop {
        SelectedAuthorizedReadStop::Currentness(SelectedAuthorizationCurrentnessStop::Admission(stop)) => return WorthQueryAdmittedOneShotStop::Admission(stop),

        SelectedAuthorizedReadStop::Currentness(SelectedAuthorizationCurrentnessStop::Execution(cause)) => {
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
            match crate::facade::primary_graph::relational_execution_kind(cause, None) {
                Ok(kind) => WorthQueryApplicationOneShotDenialKind::ExecutionRequest(kind),
                Err(Control::Cancelled) => WorthQueryApplicationOneShotDenialKind::Cancelled,
                Err(Control::TimedOut) => WorthQueryApplicationOneShotDenialKind::DeadlineExceeded,
            }
        }



        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop::World,
        ) | SelectedAuthorizedReadStop::StaleSecurity => {
            WorthQueryApplicationOneShotDenialKind::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleAuthorization),
            )
        }
        SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::StaleIssuedAccess
            | SelectedAuthorizationCurrentnessStop::StaleDecision
            | SelectedAuthorizationCurrentnessStop::UnsupportedGovernance,
        ) => WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleAuthorization),
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
        SelectedAuthorizedReadStop::Read { .. } => {
            unreachable!("the read-denial branch settles its reservation before mapping")
        }
    };
    WorthQueryAdmittedOneShotStop::Execution(denial(kind, query, query))
}
