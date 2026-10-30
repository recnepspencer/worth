pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectLocator, worth_foundational::AspectValue,
    worth_foundational::BoundarySourceLocator, worth_foundational::InternedString,
    worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store_aspect_native::project_store_boundary_fact_to_terminal_json,
    worth_store_aspect_native::readmit_external_terminal_projection_document_as_store_aspect_state,
    worth_store_aspect_native::StoreAspectAuthorityInput,
    worth_store_aspect_native::StoreAspectBoundaryFact,
    worth_store_aspect_native::StoreAspectIdentity,
    worth_store_aspect_native::StorePhysicalBoundaryWitness,
    worth_store_aspect_native::StoreTerminalProjectionDenial,
    worth_store_aspect_native::StoreTerminalProjectionDisplayLabel,
};
pub(crate) use {
    worth_store_contracts::StorePhysicalAuthorityWitness,
    worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};

pub(crate) fn struct_boundary_fact(
    contract: &AspectContract,
    segment: &str,
    generation: u64,
) -> StoreAspectBoundaryFact {
    let struct_value = aspects()
        .vocabulary()
        .struct_value()
        .with_field(
            "segment",
            worth_foundational::AspectValue::String(worth_foundational::InternedString::from(
                segment,
            )),
        )
        .with_field(
            "generation",
            worth_foundational::AspectValue::UInt64(generation),
        )
        .finish()
        .unwrap();
    let validated = match aspects().validate().against(contract).value(struct_value) {
        worth_proof::TransitionOutcome::Success(value) => value,
        outcome => panic!("struct validation should succeed: {outcome:?}"),
    };
    let state = match aspects().authoritative_state().admit([validated]) {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };
    worth_store_aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(contract.key().clone()),
        worth_store_aspect_native::StoreAspectAuthorityInput::new(state, physical_witness()),
    )
    .unwrap()
}

pub(crate) fn segment_header_contract(aspect_key: AspectKey) -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .required("segment", worth_foundational::ScalarAspectType::String)
        .required("generation", worth_foundational::ScalarAspectType::UInt64)
        .finish()
        .unwrap();
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(31))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape)
}

pub(crate) fn aspect_key(raw: &str) -> AspectKey {
    aspects().vocabulary().key(raw).unwrap()
}

pub(crate) fn source_locator(aspect_key: AspectKey) -> BoundarySourceLocator {
    worth_foundational::BoundarySourceLocator::Aspect(worth_foundational::AspectLocator::new(
        worth_foundational::LocatorAuthority::Projected,
        aspect_key,
    ))
}

pub(crate) fn physical_witness() -> StorePhysicalBoundaryWitness {
    worth_store_aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store_contracts::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap()
}
