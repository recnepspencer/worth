#[allow(unused_imports)]
use crate::aspect_native_terminal_projection::*;
use {
    worth_store_aspect_native::project_store_boundary_fact_to_terminal_json,
    worth_store_aspect_native::readmit_external_terminal_projection_document_as_store_aspect_state,
    worth_store_aspect_native::readmit_terminal_json_projection_as_store_aspect_state,
};

#[test]
fn terminal_json_projection_is_one_way_until_readmission() {
    let aspect_key = aspect_key("store.physical.segment.identity");
    let contract = scalar_string_contract(aspect_key.clone());
    let fact = boundary_fact(&contract, "segment-0007");

    let projection = project_store_boundary_fact_to_terminal_json(&fact).unwrap();
    assert_eq!(projection.terminal_projection_identity(), fact.identity());

    let readmission = match readmit_terminal_json_projection_as_store_aspect_state(
        projection,
        contract,
        source_locator(aspect_key.clone()),
        physical_witness(),
    ) {
        worth_proof::TransitionOutcome::Success(readmission) => readmission,
        outcome => panic!("terminal projection should readmit: {outcome:?}"),
    };

    let readmitted_fact = worth_store_aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(aspect_key),
        worth_store_aspect_native::StoreAspectAuthorityInput::new(
            readmission.admitted_state().clone(),
            readmission.physical_witness(),
        ),
    )
    .unwrap();
    assert_eq!(readmitted_fact.identity(), readmission.identity());
}

#[test]
fn external_terminal_json_ingress_must_use_store_readmission() {
    let aspect_key = aspect_key("store.physical.segment.identity");
    let identity =
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(aspect_key.clone());
    let contract = scalar_string_contract(aspect_key.clone());

    let readmission = match readmit_external_terminal_projection_document_as_store_aspect_state(
        identity.clone(),
        "segment-0010".into(),
        contract,
        source_locator(aspect_key),
        physical_witness(),
    ) {
        worth_proof::TransitionOutcome::Success(readmission) => readmission,
        outcome => panic!("external terminal projection should readmit: {outcome:?}"),
    };

    assert_eq!(readmission.identity(), &identity);
}

#[test]
fn external_terminal_json_ingress_rejects_type_incompatible_projection_document() {
    let aspect_key = aspect_key("store.physical.segment.identity");
    let identity =
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(aspect_key.clone());
    let contract = scalar_string_contract(aspect_key.clone());

    let outcome = readmit_external_terminal_projection_document_as_store_aspect_state(
        identity,
        7.into(),
        contract,
        source_locator(aspect_key),
        physical_witness(),
    );

    assert!(matches!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_store_aspect_native::StoreTerminalProjectionDenial::JsonCompatibilityDenied(_)
        )
    ));
}

#[test]
fn terminal_json_readmission_rejects_mismatched_contract_identity() {
    let identity_key = aspect_key("store.physical.segment.identity");
    let fact = boundary_fact(
        &scalar_string_contract(identity_key.clone()),
        "segment-0008",
    );
    let projection = project_store_boundary_fact_to_terminal_json(&fact).unwrap();
    let wrong_contract = scalar_string_contract(aspect_key("store.physical.segment.header"));

    let outcome = readmit_terminal_json_projection_as_store_aspect_state(
        projection,
        wrong_contract,
        source_locator(identity_key),
        physical_witness(),
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_store_aspect_native::StoreTerminalProjectionDenial::ContractIdentityMismatch
        )
    );
}

#[test]
fn terminal_json_projection_round_trips_validated_struct_state() {
    let aspect_key = aspect_key("store.physical.segment.header");
    let contract = segment_header_contract(aspect_key.clone());
    let fact = struct_boundary_fact(&contract, "segment-0009", 9);

    let projection = project_store_boundary_fact_to_terminal_json(&fact).unwrap();
    let outcome = readmit_terminal_json_projection_as_store_aspect_state(
        projection,
        contract,
        source_locator(aspect_key),
        physical_witness(),
    );

    assert!(matches!(
        outcome,
        worth_proof::TransitionOutcome::Success(_)
    ));
}

#[test]
fn terminal_json_rendering_changes_checksum_not_store_digest_authority() {
    let aspect_key = aspect_key("store.physical.segment.header");
    let contract = segment_header_contract(aspect_key.clone());
    let fact = struct_boundary_fact(&contract, "segment-0011", 11);
    let expected_authority = basis_and_digest_for_boundary_fact(&fact);
    let projection = project_store_boundary_fact_to_terminal_json(&fact).unwrap();

    let compact_checksum = terminal_checksum(
        projection
            .to_compact_terminal_json_document_bytes()
            .unwrap(),
    );
    let pretty_checksum =
        terminal_checksum(projection.to_pretty_terminal_json_document_bytes().unwrap());
    let first_label =
        worth_store_aspect_native::StoreTerminalProjectionDisplayLabel::new("Operator segment")
            .unwrap();
    let second_label =
        worth_store_aspect_native::StoreTerminalProjectionDisplayLabel::new("Review segment")
            .unwrap();
    let first_labelled_checksum = terminal_checksum(
        projection
            .to_labelled_terminal_json_document_bytes(&first_label)
            .unwrap(),
    );
    let second_labelled_checksum = terminal_checksum(
        projection
            .to_labelled_terminal_json_document_bytes(&second_label)
            .unwrap(),
    );

    assert_ne!(
        compact_checksum.terminal_checksum_bytes(),
        pretty_checksum.terminal_checksum_bytes()
    );
    assert_ne!(
        first_labelled_checksum.terminal_checksum_bytes(),
        second_labelled_checksum.terminal_checksum_bytes()
    );

    let readmission = match readmit_terminal_json_projection_as_store_aspect_state(
        projection,
        contract,
        source_locator(aspect_key),
        physical_witness(),
    ) {
        worth_proof::TransitionOutcome::Success(readmission) => readmission,
        outcome => panic!("terminal projection should readmit: {outcome:?}"),
    };
    let readmitted_authority =
        basis_and_digest_for_boundary_fact(&readmission.rebuild_store_boundary_fact().unwrap());

    assert_same_native_basis_and_store_digest(&expected_authority, &readmitted_authority);
}

#[test]
fn terminal_json_field_order_changes_do_not_change_readmitted_native_digest() {
    let aspect_key = aspect_key("store.physical.segment.header");
    let identity =
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(aspect_key.clone());
    let first_contract = segment_header_contract(aspect_key.clone());
    let second_contract = segment_header_contract(aspect_key.clone());

    let first_readmission = readmitted_external_struct_projection(
        identity.clone(),
        first_contract,
        [
            (
                "segment",
                serde_json::Value::String("segment-0012".to_string()),
            ),
            ("generation", serde_json::Number::from(12).into()),
        ],
    );
    let second_readmission = readmitted_external_struct_projection(
        identity,
        second_contract,
        [
            ("generation", serde_json::Number::from(12).into()),
            (
                "segment",
                serde_json::Value::String("segment-0012".to_string()),
            ),
        ],
    );

    let first_authority = basis_and_digest_for_boundary_fact(
        &first_readmission.rebuild_store_boundary_fact().unwrap(),
    );
    let second_authority = basis_and_digest_for_boundary_fact(
        &second_readmission.rebuild_store_boundary_fact().unwrap(),
    );

    assert_same_native_basis_and_store_digest(&first_authority, &second_authority);
}
