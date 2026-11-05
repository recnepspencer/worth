use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

use crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenialKind;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::denial;








/// Owner stages and the complete reconstruction return through this boundary.
/// A worker's map allowance ends before its returned payload acquires custody.
pub fn retain_reconstruction_result<T>(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    result: Result<T, super::super::WorthQueryManagedDerivedViewDenial>,
    on_admission_denial: impl FnOnce(
        worth_execution::LeaseDenial,
    ) -> super::super::WorthQueryManagedDerivedViewDenial,
) -> Result<T, super::super::WorthQueryManagedDerivedViewDenial> {
    use super::super::WorthQueryManagedDerivedViewDenial as Denial;
    use worth_execution::ChargedBytes;
    if let Err(Denial::ReadDenied { denial, .. }) = &result
        && denial.payload.custody.get().is_none()
    {
        let hold = worth_execution::ExecutionMemoryReservation::reserve_in_scope(
            lease,
            denial.additional_charged_bytes(),
        )
        .map_err(on_admission_denial)?;
        denial.retain_custody(hold);
    }
    result
}



#[cfg(test)]
mod tests {
    use {crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::denial, crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind};

    #[test]
    fn execution_denial_preserves_query_and_internal_subject_separately() {
        let denial = denial(
            WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded,
            "FrameMeasurementCoverageQuery",
            "root/relation[0]/field[0]",
        );

        assert_eq!(denial.query(), "FrameMeasurementCoverageQuery");
        assert_eq!(denial.subject(), "root/relation[0]/field[0]");
    }
}


mod court_doors_for_test;
