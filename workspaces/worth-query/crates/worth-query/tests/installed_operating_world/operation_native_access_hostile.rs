mod fixture;

use worth_foundational::facade::{FieldKey, InternedString};
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};
use fixture::{
    assert_request_denial, collection_workspace, id_field, native_id_request, CollectionRead,
};

#[test]
fn direct_access_indexes_display_and_derived_values_across_committed_rows() {
    let mut workspace = collection_workspace("installed-native-multi-key");
    for value in ["alpha", "beta", "gamma"] {
        workspace
            .insert("Vertex", |mutation| mutation.aspect("identity.id", value))
            .unwrap();
    }
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, CollectionRead)
        .unwrap();
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let display = builder.select_display_native_field(id_field()).unwrap();
    let derived = builder.select_derived_native_field(id_field()).unwrap();
    let request = builder.build().unwrap();
    let keys = [display, derived]
        .iter()
        .map(|selection| request.resolve_native_key(selection).unwrap().into_key())
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), 2);

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

    let binding = settled.native_access_binding_counters().unwrap();
    assert_eq!(binding.declared_key_routes, 2);
    assert_eq!(binding.declared_key_layout_checks, 2);
    assert_eq!(binding.lane_shape_checks, 2);
    assert_eq!(binding.fact_scans, 0);
    assert_eq!(binding.row_scans, 0);
    assert_eq!(binding.path_parses, 0);

    let mut indexed_accesses = 0;
    let mut refinements = 0;
    for (row, expected) in ["alpha", "beta", "gamma"].iter().enumerate() {
        for key in &keys {
            let access = settled.native_value(key, row).unwrap();
            assert_eq!(
                access.fact().as_interned_string(),
                Ok(&InternedString::Raw((*expected).into()))
            );
            let counters = access.counters();
            indexed_accesses += counters.indexed_accesses;
            refinements += counters.refinement_checks;
            assert_eq!(counters.fact_scans, 0);
            assert_eq!(counters.row_scans, 0);
            assert_eq!(counters.path_parses, 0);
        }
    }
    assert_eq!(indexed_accesses, 6);
    assert_eq!(refinements, 6);
}

#[test]
fn request_builder_denials_retain_contract_and_requested_field_context() {
    let workspace = workspace("installed-native-request-denials", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();

    let no_facts = match bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request()
        .build()
    {
        Ok(_) => panic!("an empty request must not mint a bound projection"),
        Err(denial) => denial,
    };
    assert_request_denial(
        &no_facts,
        domain::WorthQueryNativeProjectionRequestDenialKind::NoNativeFacts,
        None,
    );

    let unknown_bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let unknown = FieldKey::new("not-declared").unwrap();
    let mut unknown_builder = unknown_bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let unknown_denial = match unknown_builder.select_display_native_field(unknown.clone()) {
        Ok(_) => panic!("an undeclared native field must not produce a request builder"),
        Err(denial) => denial,
    };
    assert_request_denial(
        &unknown_denial,
        domain::WorthQueryNativeProjectionRequestDenialKind::UnknownField,
        Some(&unknown),
    );

    let duplicate_bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let duplicate_field = id_field();
    let mut duplicate_builder = duplicate_bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let _selection = duplicate_builder
        .select_display_native_field(duplicate_field.clone())
        .unwrap();
    let duplicate_denial =
        match duplicate_builder.select_display_native_field(duplicate_field.clone()) {
            Ok(_) => panic!("a duplicate declaration must not mint two equivalent keys"),
            Err(denial) => denial,
        };
    assert_request_denial(
        &duplicate_denial,
        domain::WorthQueryNativeProjectionRequestDenialKind::ConflictingDeclaration,
        Some(&duplicate_field),
    );
}

#[test]
fn capability_denial_retains_exact_native_source_and_projection_context() {
    let mut workspace = workspace("installed-native-denial-context", false).unwrap();
    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "authoritative-row")
        })
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let owner = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let foreign = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let (owner_request, owner_key) =
        native_id_request(owner.consumer_projection_contract().unwrap());
    let (_foreign_request, foreign_key) =
        native_id_request(foreign.consumer_projection_contract().unwrap());
    let settled = owner
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(owner_request)
        .unwrap()
        .settle()
        .unwrap();
    let admitted = settled.native_value(&owner_key, 0).unwrap();
    let denial = settled.native_value(&foreign_key, 0).unwrap_err();

    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch
    );
    assert_eq!(denial.field_path(), foreign_key.field_path());
    assert_eq!(denial.contract_key(), foreign_key.contract_key());
    assert_eq!(denial.contract_identity(), foreign_key.contract_identity());
    assert_eq!(denial.contract_revision(), foreign_key.contract_revision());
    assert_eq!(denial.expected_shape(), foreign_key.expected_shape());
    assert_eq!(denial.absence_posture(), foreign_key.absence_posture());
    assert_eq!(denial.source_family(), admitted.fact().source_family());
    assert_eq!(denial.source_identity(), admitted.fact().source_identity());
    assert_eq!(
        denial.projection_authority(),
        admitted.fact().projection_authority()
    );
    assert_eq!(denial.counters().indexed_accesses, 0);
    assert_eq!(denial.counters().refinement_checks, 0);
    assert_eq!(denial.counters().fact_scans, 0);
    assert_eq!(denial.counters().row_scans, 0);
    assert_eq!(denial.counters().path_parses, 0);
}
