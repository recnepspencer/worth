use worth_foundational::facade::{AbsenceLaw, AspectValue, StructAspectValue};
use worth_query::facade::domain;

use super::fixture::{bind, matrix_workspace_with_values};
use super::samples::{
    defaulted_field, matrix_aspect_key, matrix_value, optional_field, sample_field, scalar_samples,
};

#[test]
fn installed_native_access_preserves_every_foundational_family_and_absence_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let expected_struct = matrix_value(0);
    let mut workspace = matrix_workspace_with_values(
        request_execution,
        "installed-native-family-matrix",
        std::slice::from_ref(&expected_struct),
        false,
    );
    let bound = bind(&workspace);
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let mut expected = vec![(
        builder.select_display_native_aspect().unwrap(),
        ExpectedNativeValue::Struct(expected_struct.clone()),
    )];
    expected.extend(scalar_samples().into_iter().enumerate().map(|(index, _)| {
        let field = sample_field(index);
        let value = expected_struct.get(&field).unwrap().clone();
        (
            builder.select_derived_native_field(field).unwrap(),
            ExpectedNativeValue::Scalar(value),
        )
    }));
    expected.push((
        builder
            .select_derived_native_field(optional_field())
            .unwrap(),
        ExpectedNativeValue::Absent(AbsenceLaw::Optional),
    ));
    expected.push((
        builder
            .select_derived_native_field(defaulted_field())
            .unwrap(),
        ExpectedNativeValue::Absent(AbsenceLaw::Defaulted),
    ));
    let request = builder.build().unwrap();
    let resolved = expected
        .into_iter()
        .map(|(selection, expected)| {
            let resolution = request.resolve_native_key(&selection).unwrap();
            assert_constant_resolution(resolution.counters());
            (resolution.into_key(), expected)
        })
        .collect::<Vec<_>>();
    let settled = bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();

    for (key, expected) in &resolved {
        assert_eq!(key.contract_key(), &matrix_aspect_key());
        assert_eq!(
            key.field_path().native_aspect_key(),
            Some(&matrix_aspect_key())
        );
        let access = settled.native_value(key, 0).unwrap();
        match expected {
            ExpectedNativeValue::Struct(value) => {
                assert_eq!(access.value().struct_value(), Some(value));
            }
            ExpectedNativeValue::Scalar(value) => {
                assert_eq!(access.value().scalar(), Some(value));
            }
            ExpectedNativeValue::Absent(posture) => {
                assert_eq!(access.value().absence(), Some(*posture));
            }
        }
        let wrong_refinement = match expected {
            ExpectedNativeValue::Struct(_) => Some(access.fact().as_uint64().unwrap_err()),
            ExpectedNativeValue::Scalar(AspectValue::String(_)) => {
                Some(access.fact().as_uint64().unwrap_err())
            }
            ExpectedNativeValue::Absent(_) => Some(access.fact().as_null().unwrap_err()),
            ExpectedNativeValue::Scalar(_) => None,
        };
        if let Some(denial) = wrong_refinement {
            assert_refinement_context(&denial, key, access.fact());
        }
        assert_constant_access(access.counters());
    }

    let binding = settled.native_access_binding_counters().unwrap();
    assert_eq!(binding.declared_key_routes, resolved.len());
    assert_eq!(binding.declared_key_layout_checks, resolved.len());
    assert_eq!(binding.lane_shape_checks, 2);
    assert_eq!(binding.fact_scans, 0);
    assert_eq!(binding.row_scans, 0);
    assert_eq!(binding.path_parses, 0);
    assert_eq!(binding.view_registry_inspections, 0);
    assert_eq!(binding.domain_registry_inspections, 0);
}

fn assert_refinement_context(
    denial: &worth_query::facade::foundation::ConsumedNativeRefinementDenial,
    key: &domain::WorthQueryNativeAccessKey,
    fact: &worth_query::facade::foundation::ConsumedFieldValueFact,
) {
    assert_eq!(denial.field_path(), key.field_path());
    assert_eq!(denial.contract_key(), Some(key.contract_key()));
    assert_eq!(denial.contract_identity(), Some(key.contract_identity()));
    assert_eq!(denial.contract_revision(), Some(key.contract_revision()));
    assert_eq!(denial.source_family(), fact.source_family());
    assert_eq!(denial.source_identity(), fact.source_identity());
    assert_eq!(denial.source_row_identity(), fact.source_row_identity());
    assert_eq!(denial.projection_authority(), fact.projection_authority());
}

enum ExpectedNativeValue {
    Struct(StructAspectValue),
    Scalar(AspectValue),
    Absent(AbsenceLaw),
}

fn assert_constant_resolution(counters: domain::WorthQueryNativeKeyResolutionCounters) {
    assert_eq!(counters.declaration_checks, 1);
    assert_eq!(counters.indexed_slot_lookups, 2);
    assert_eq!(counters.path_matches, 0);
    assert_eq!(counters.key_scans, 0);
    assert_eq!(counters.path_parses, 0);
}

fn assert_constant_access(counters: domain::WorthQueryNativeAccessCounters) {
    assert_eq!(counters.authority_checks, 5);
    assert_eq!(counters.indexed_accesses, 1);
    assert_eq!(counters.refinement_checks, 1);
    assert_eq!(counters.fact_scans, 0);
    assert_eq!(counters.row_scans, 0);
    assert_eq!(counters.path_parses, 0);
    assert_eq!(counters.view_registry_inspections, 0);
    assert_eq!(counters.domain_registry_inspections, 0);
}
