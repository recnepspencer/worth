//! Exact selected read, with the carried Work reservation opened after auth.

use std::num::NonZeroUsize;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{execute_graph_read_with_snapshot, validate_selected_authorization_currentness};
use crate::domain_computation::primary_graph::{
    application_query::{
        read_execution::{
            read_bounded_root_rows, OneShotReadWorkObservation, RawNonLiveKernelOutcome,
            WorthQueryApplicationReadExecutionDenial,
        },
        resource_lifecycle::WorthQueryApplicationResultBufferReservation,
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    },
    output_lineage::invalidation::{InvalidationEditAdmission, ReservedExternalWork},
    product_operation::SelectedPermissionSecurityStop,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(in crate::domain_computation::primary_graph::application_query) enum SelectedAuthorizedReadStop<
    'a,
> {
    Admission(CompanionPreflightStop),
    Currentness(super::SelectedAuthorizationCurrentnessStop),
    Security(SelectedPermissionSecurityStop),
    StaleSecurity,
    Session,
    WorkUnavailable,
    WorkCounterOverflow,
    Read {
        denial: WorthQueryApplicationReadExecutionDenial,
        reservation: ReservedExternalWork<'a>,
    },
}

pub(in crate::domain_computation::primary_graph::application_query) fn execute_selected_authorized_read<
    'a,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: &mut WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    admission: &'a mut InvalidationEditAdmission,
    result_buffer: WorthQueryApplicationResultBufferReservation,
    spent: &OneShotReadWorkObservation,
    retained_custody_work: usize,
) -> Result<
    (
        RawNonLiveKernelOutcome,
        WorthQueryApplicationAuthorizationWorkEvidence,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
        ReservedExternalWork<'a>,
    ),
    SelectedAuthorizedReadStop<'a>,
>
where
    Schema: ApplicationSchema,
{
    let super::WorthQueryQueryIndexPosture::SelectedInstalled(prepared) = &plan.index_posture
    else {
        return Err(SelectedAuthorizedReadStop::StaleSecurity);
    };
    let security = application
        .admit_matching_head_query_prepared_read_security_basis(
            &plan.security_product,
            &plan.basis,
            prepared,
            admission,
        )
        .map_err(SelectedAuthorizedReadStop::Security)?
        .ok_or(SelectedAuthorizedReadStop::StaleSecurity)?;
    let ((raw, authorization_work, reservation), proof) = execute_graph_read_with_snapshot(
        &plan.graph_work,
        &plan.basis,
        security.snapshot_handle(),
        || SelectedAuthorizedReadStop::Session,
        (),
        |runtime, layout, security, ()| {
            validate_selected_authorization_currentness(
                application,
                runtime,
                security,
                plan,
                admission,
            )
            .map_err(SelectedAuthorizedReadStop::Currentness)?;
            // The read's own work stays under the Query's declared limit;
            // its retained custody copies are reserved beside it.
            let (maximum, reserved) = selected_read_reservation(
                plan.controls().maximum_work().get(),
                admission.remaining_work_amount(),
                retained_custody_work,
            )?;
            let reservation = admission
                .reserve_external_work(reserved)
                .map_err(SelectedAuthorizedReadStop::Admission)?;
            let raw = match read_bounded_root_rows(
                runtime,
                layout,
                plan,
                result_buffer,
                Some(spent),
                maximum.get(),
            ) {
                Ok(raw) => raw,
                Err(denial) => {
                    return Err(SelectedAuthorizedReadStop::Read {
                        denial,
                        reservation,
                    });
                }
            };
            Ok((
                raw,
                plan.authorization_work
                    .with_execution_security_product_resolution(),
                reservation,
            ))
        },
    )?;
    Ok((raw, authorization_work, proof, reservation))
}

/// Keep custody beside the read in the request's work amount, before taking
/// the read's machine-word bound.
fn selected_read_reservation(
    declared: usize,
    remaining: u64,
    custody: usize,
) -> Result<(NonZeroUsize, u64), SelectedAuthorizedReadStop<'static>> {
    let overflow = || SelectedAuthorizedReadStop::WorkCounterOverflow;
    let custody = u64::try_from(custody).map_err(|_| overflow())?;
    let declared = u64::try_from(declared).map_err(|_| overflow())?;
    let maximum_amount = declared.min(remaining.saturating_sub(custody));
    let maximum = usize::try_from(maximum_amount).map_err(|_| overflow())?;
    let maximum = NonZeroUsize::new(maximum).ok_or(SelectedAuthorizedReadStop::WorkUnavailable)?;
    let reserved = maximum_amount.checked_add(custody).ok_or_else(overflow)?;
    Ok((maximum, reserved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custody_is_subtracted_from_the_amount_before_the_read_bound() {
        // Both counts fit on 32 bits; their sum is the exact 2^32 allowance.
        let actual =
            selected_read_reservation(1, 1_u64 << 32, usize::try_from(u32::MAX).unwrap()).ok();
        assert_eq!(
            actual,
            Some((NonZeroUsize::new(1).unwrap(), 1_u64 << 32)),
            "custody subtracts from the u64 amount before narrowing the read bound"
        );
        assert!(
            matches!(
                selected_read_reservation(
                    1,
                    u64::from(u32::MAX),
                    usize::try_from(u32::MAX).unwrap()
                ),
                Err(SelectedAuthorizedReadStop::WorkUnavailable)
            ),
            "custody cannot borrow the read's work"
        );
    }
}
