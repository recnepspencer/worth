#[allow(unused_imports)]
use crate::aspect_native_terminal_projection_hostile_readmission::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectLocator, worth_foundational::AspectValue,
    worth_foundational::BoundarySourceLocator, worth_foundational::InternedString,
    worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType,
};
use {
    worth_store_aspect_native::project_store_boundary_fact_to_terminal_json,
    worth_store_aspect_native::readmit_external_terminal_projection_document_as_store_aspect_state,
    worth_store_aspect_native::StoreAspectAuthorityInput,
    worth_store_aspect_native::StoreAspectBoundaryFact,
    worth_store_aspect_native::StoreAspectIdentity,
    worth_store_aspect_native::StorePhysicalBoundaryWitness,
    worth_store_aspect_native::StoreTerminalProjectionDenial,
    worth_store_aspect_native::StoreTerminalProjectionDisplayLabel,
};
use {
    worth_store_contracts::StorePhysicalAuthorityWitness,
    worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};

#[test]
fn labelled_terminal_json_document_cannot_readmit_as_store_authority() {
    let aspect_key = aspect_key("store.physical.segment.header");
    let contract = segment_header_contract(aspect_key.clone());
    let fact = struct_boundary_fact(&contract, "segment-0014", 14);
    let projection = project_store_boundary_fact_to_terminal_json(&fact).unwrap();
    let label =
        worth_store_aspect_native::StoreTerminalProjectionDisplayLabel::new("Operator segment")
            .unwrap();

    let labelled_document = serde_json::from_slice(
        projection
            .to_labelled_terminal_json_document_bytes(&label)
            .unwrap()
            .terminal_projection_bytes(),
    )
    .unwrap();

    let readmission = readmit_external_terminal_projection_document_as_store_aspect_state(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(aspect_key.clone()),
        labelled_document,
        contract,
        source_locator(aspect_key),
        physical_witness(),
    );

    assert!(matches!(
        readmission,
        worth_proof::TransitionOutcome::Denied(
            worth_store_aspect_native::StoreTerminalProjectionDenial::JsonCompatibilityDenied(_)
        )
    ));
}
