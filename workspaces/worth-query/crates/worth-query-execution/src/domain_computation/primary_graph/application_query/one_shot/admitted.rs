use super::*;

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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryApplicationOneShotResult<Query, QueryResult>, WorthQueryAdmittedOneShotStop>
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        let spent = OneShotReadWorkObservation::new();
        let mut plan = plan;
        admit_one_shot_entry(&plan, admission)?;
        validate_one_shot_plan(self, &plan).map_err(WorthQueryAdmittedOneShotStop::Execution)?;
        let result_buffer = reserve_one_shot_result_buffer(self, &plan)
            .map_err(WorthQueryAdmittedOneShotStop::Execution)?;
        let custody = custody_work::RetainedCustodyWork::of(
            plan.scope.identity_locator(),
            plan.scope.identity_value(),
            plan.query.name(),
            plan.basis.identity().branch_id(),
        )
        .and_then(custody_work::RetainedCustodyWork::total)
        .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
        let (raw, authorization_work, read_proof, reservation) =
            match execute_selected_authorized_read(
                self,
                &mut plan,
                admission,
                result_buffer,
                &spent,
                custody,
            ) {
                Ok(read) => read,
                Err(SelectedAuthorizedReadStop::Read {
                    denial,
                    reservation,
                }) => {
                    let result = Err(map_authorized_read_denial(
                        WorthQueryAuthorizedApplicationReadDenial::Read(denial),
                        plan.query.name(),
                    ));
                    return settle_admitted_read(reservation, &spent, result);
                }
                Err(stop) => return Err(map_selected_read_stop(stop, plan.query.name())),
            };
        let result = finalize_one_shot(
            self,
            plan,
            raw,
            authorization_work,
            read_proof,
            Some(&spent),
        );
        settle_admitted_read(reservation, &spent, result)
    }
}

fn admit_one_shot_entry<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryAdmittedOneShotStop> {
    // Pay the selected metadata reads before measuring the installed name or
    // its authority-seal validation requirement.
    admission
        .charge_external_work(2)
        .map_err(WorthQueryAdmittedOneShotStop::Admission)?;
    let name = u64::try_from(plan.query.name().len())
        .map_err(|_| WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    let validation = plan
        .query
        .validation_work_bound()
        .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    // An installed-query rejection holds its own subject while the outer
    // OneShot denial constructs both query and subject. Other exits use at
    // most the latter two copies. Read-denial subjects move into the outer
    // denial instead of being copied a second time.
    let names = name
        .checked_mul(3)
        .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    // Runtime/current-schema checks, the two installed identity digests,
    // lane/request/lifetime probes, and result-buffer Arc/atomic reservation.
    let fixed = 1_u64 + 4 + 4 + 64 + 1 + 1 + 1 + 1 + 1 + 1 + 4 + 8;
    let work = validation
        .checked_add(fixed)
        .and_then(|work| work.checked_add(names))
        .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    admission
        .admit_read_scratch(names)
        .and_then(|()| admission.charge_external_work(work))
        .map_err(WorthQueryAdmittedOneShotStop::Admission)
}

fn settle_admitted_read<Query, QueryResult>(
    reservation: ReservedExternalWork<'_>,
    spent: &OneShotReadWorkObservation,
    result: Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >,
) -> Result<WorthQueryApplicationOneShotResult<Query, QueryResult>, WorthQueryAdmittedOneShotStop> {
    let observed = spent
        .total()
        .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    let observed_u64 =
        u64::try_from(observed).map_err(|_| WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
    if let Ok(read) = &result {
        let read_work = spent
            .read()
            .ok_or(WorthQueryAdmittedOneShotStop::WorkCounterOverflow)?;
        if read.receipt().work().total_work_units() != read_work {
            return Err(WorthQueryAdmittedOneShotStop::WorkAccountingMismatch);
        }
    }
    reservation
        .settle(observed_u64)
        .map_err(WorthQueryAdmittedOneShotStop::Admission)?;
    match result {
        Ok(read) => Ok(read),
        Err(stop) => Err(WorthQueryAdmittedOneShotStop::Execution(stop)),
    }
}

fn map_selected_read_stop(
    stop: SelectedAuthorizedReadStop<'_>,
    query: &str,
) -> WorthQueryAdmittedOneShotStop {
    let kind = match stop {
        SelectedAuthorizedReadStop::Admission(stop)
        | SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::Admission(stop),
        ) => return WorthQueryAdmittedOneShotStop::Admission(stop),
        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop::Handle(handle),
        ) => WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Handle(handle)),
        SelectedAuthorizedReadStop::WorkUnavailable => {
            return WorthQueryAdmittedOneShotStop::WorkUnavailable;
        }
        SelectedAuthorizedReadStop::WorkCounterOverflow => {
            return WorthQueryAdmittedOneShotStop::WorkCounterOverflow;
        }
        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop::Admission(stop),
        ) => return WorthQueryAdmittedOneShotStop::Admission(stop),
        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop::AccountingOverflow,
        ) => return WorthQueryAdmittedOneShotStop::WorkCounterOverflow,
        SelectedAuthorizedReadStop::Security(
            crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop::World,
        ) | SelectedAuthorizedReadStop::StaleSecurity => {
            WorthQueryApplicationOneShotDenialKind::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
            )
        }
        SelectedAuthorizedReadStop::Currentness(
            SelectedAuthorizationCurrentnessStop::StaleIssuedAccess
            | SelectedAuthorizationCurrentnessStop::StaleDecision
            | SelectedAuthorizationCurrentnessStop::UnsupportedGovernance,
        ) => WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
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
