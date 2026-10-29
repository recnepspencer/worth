pub use worth_query::facade::{domain, foundation, read};

pub use super::fixture::{bind, matrix_workspace};
pub use super::samples::sample_field;
pub use super::world_scale::{
    bind_foreign_family, bind_foreign_revision, bind_foreign_same_contract,
};

#[macro_export]
macro_rules! worlds_query_installed_operating_world_operation_native_access_matrix_rejection_rs_request_and_key {
    ($bound:expr, $field:expr) => {{
        let mut builder = $bound
            .consumer_projection_contract()
            .unwrap()
            .projection_request();
        let selection = builder.select_display_native_field($field).unwrap();
        let request = builder.build().unwrap();
        let key = request.resolve_native_key(&selection).unwrap().into_key();
        (request, key)
    }};
}
pub use crate::worlds_query_installed_operating_world_operation_native_access_matrix_rejection_rs_request_and_key as request_and_key;

pub fn execute_owner(
    bound: domain::WorthQueryBoundDomainOperation<
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::ReadFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    request: domain::WorthQueryBoundProjectionRequest<
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::ReadFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
) -> domain::WorthQuerySettledDomainProjection<
    worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::GeometryDomain,
    super::fixture::NativeMatrixRead,
    worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    bound
        .admit_execution_resources(
            (),
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap()
}

pub fn assert_denial_before_access(
    denial: &domain::WorthQueryNativeAccessDenial,
    kind: domain::WorthQueryNativeAccessDenialKind,
) {
    assert_eq!(denial.kind(), kind);
    assert_eq!(denial.counters().indexed_accesses, 0);
    assert_eq!(denial.counters().refinement_checks, 0);
    assert_eq!(denial.counters().fact_scans, 0);
    assert_eq!(denial.counters().row_scans, 0);
    assert_eq!(denial.counters().path_parses, 0);
}
