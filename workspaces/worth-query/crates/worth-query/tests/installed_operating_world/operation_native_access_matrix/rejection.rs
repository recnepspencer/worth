use worth_query::facade::{domain, foundation, read};

use super::fixture::{bind, matrix_workspace};
use super::samples::sample_field;
use super::world_scale::{bind_foreign_family, bind_foreign_revision, bind_foreign_same_contract};

macro_rules! request_and_key {
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

#[test]
fn one_bound_capability_cannot_mint_a_distinct_second_request() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = matrix_workspace(
        request_execution,
        "installed-native-one-shot-request",
        1,
        false,
    );
    let bound = bind(&workspace);
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let _selection = builder
        .select_display_native_field(sample_field(0))
        .unwrap();
    let first = builder.build().unwrap();
    assert_eq!(first.declared_native_selection_count(), 1);
    assert!(matches!(
        bound.consumer_projection_contract(),
        Err(domain::WorthQueryConsumerProjectionContractDenial::AlreadyMinted { .. })
    ));
}

#[test]
fn selector_from_another_declaration_denies_before_any_key_lookup() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = matrix_workspace(
        request_execution,
        "installed-native-foreign-selector",
        1,
        false,
    );
    let owner = bind(&workspace);
    let foreign = bind(&workspace);
    let mut owner_builder = owner
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    owner_builder
        .select_display_native_field(sample_field(2))
        .unwrap();
    let owner_request = owner_builder.build().unwrap();
    let mut foreign_builder = foreign
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let foreign_selector = foreign_builder
        .select_display_native_field(sample_field(2))
        .unwrap();
    let denial = owner_request
        .resolve_native_key(&foreign_selector)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeSelectionDenialKind::DeclarationMismatch
    );
    assert_eq!(denial.counters().declaration_checks, 1);
    assert_eq!(denial.counters().indexed_slot_lookups, 0);
    assert_eq!(denial.counters().key_scans, 0);
    assert_eq!(denial.counters().path_matches, 0);
    assert_eq!(denial.counters().path_parses, 0);
}

#[test]
fn same_path_capability_domain_revision_family_and_runtime_drift_deny_before_access() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut owner_workspace =
        matrix_workspace(request_execution, "installed-native-owner", 1, true);
    let owner = bind(&owner_workspace);
    let foreign_capability = bind(&owner_workspace);
    let (_foreign_request, foreign_capability_key) =
        request_and_key!(&foreign_capability, sample_field(2));

    let foreign_same = bind_foreign_same_contract(&owner_workspace);
    let (_foreign_same_request, foreign_same_key) =
        request_and_key!(&foreign_same, sample_field(2));
    let foreign_revision = bind_foreign_revision(&owner_workspace);
    let (_foreign_revision_request, foreign_revision_key) =
        request_and_key!(&foreign_revision, sample_field(2));
    let foreign_family = bind_foreign_family(&owner_workspace);
    let (_foreign_family_request, foreign_family_key) =
        request_and_key!(&foreign_family, sample_field(2));

    assert_eq!(
        foreign_revision_key.contract_revision(),
        worth_foundational::facade::AspectContractRevision(2)
    );
    assert_eq!(
        foreign_family_key.expected_shape(),
        foundation::AspectValuePosture::Scalar(read::ScalarAspectType::String)
    );

    let foreign_runtime_workspace = matrix_workspace(
        request_execution,
        "installed-native-foreign-runtime",
        1,
        false,
    );
    let foreign_runtime = bind(&foreign_runtime_workspace);
    let (_foreign_runtime_request, foreign_runtime_key) =
        request_and_key!(&foreign_runtime, sample_field(2));

    let (owner_request, owner_key) = request_and_key!(&owner, sample_field(2));
    assert_eq!(
        owner_key.contract_identity(),
        foreign_same_key.contract_identity()
    );
    for key in [
        &foreign_capability_key,
        &foreign_same_key,
        &foreign_revision_key,
        &foreign_family_key,
        &foreign_runtime_key,
    ] {
        assert_eq!(key.field_path(), owner_key.field_path());
    }
    assert_eq!(
        owner_key.contract_revision(),
        foreign_same_key.contract_revision()
    );
    assert_eq!(
        owner_key.expected_shape(),
        foreign_same_key.expected_shape()
    );
    assert_ne!(
        owner_key.contract_revision(),
        foreign_revision_key.contract_revision()
    );
    assert_ne!(
        owner_key.expected_shape(),
        foreign_family_key.expected_shape()
    );
    let settled = execute_owner(owner, &mut owner_workspace, owner_request);
    assert!(settled.native_value(&owner_key, 0).is_ok());

    let capability_denial = settled
        .native_value(&foreign_capability_key, 0)
        .unwrap_err();
    assert_denial_before_access(
        &capability_denial,
        domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch,
    );

    for key in [
        &foreign_same_key,
        &foreign_revision_key,
        &foreign_family_key,
    ] {
        let denial = settled.native_value(key, 0).unwrap_err();
        assert_denial_before_access(
            &denial,
            domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch,
        );
    }

    let runtime_denial = settled.native_value(&foreign_runtime_key, 0).unwrap_err();
    assert_denial_before_access(
        &runtime_denial,
        domain::WorthQueryNativeAccessDenialKind::RuntimeMismatch,
    );
}

#[test]
fn undeclared_field_is_rejected_before_a_native_key_exists() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = matrix_workspace(
        request_execution,
        "installed-native-unknown-field",
        1,
        false,
    );
    let mut builder = bind(&workspace)
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let denial = match builder
        .select_display_native_field(worth_foundational::facade::FieldKey::new("unknown").unwrap())
    {
        Ok(_) => panic!("an undeclared field must not mint an access-key builder"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeProjectionRequestDenialKind::UnknownField
    );
    assert_eq!(denial.requested_field().unwrap().as_str(), "unknown");
}

fn execute_owner(
    bound: domain::WorthQueryBoundDomainOperation<
        crate::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        crate::suite::installed_operation_fixture::ReadFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    request: domain::WorthQueryBoundProjectionRequest<
        crate::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        crate::suite::installed_operation_fixture::ReadFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
) -> domain::WorthQuerySettledDomainProjection<
    crate::suite::installed_operation_fixture::GeometryDomain,
    super::fixture::NativeMatrixRead,
    crate::suite::installed_operation_fixture::ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
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

fn assert_denial_before_access(
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
