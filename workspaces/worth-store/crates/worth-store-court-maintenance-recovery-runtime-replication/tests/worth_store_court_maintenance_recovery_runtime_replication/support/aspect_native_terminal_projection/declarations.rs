pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {serde_json::Map, serde_json::Number, serde_json::Value};
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey,
    worth_foundational::AspectLocator, worth_foundational::AspectValue,
    worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisReadyArtifact,
    worth_foundational::CanonicalDigestAlgorithmId,
    worth_foundational::CanonicalizationRuleVersion, worth_foundational::InternedString,
    worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store_aspect_native::project_store_boundary_fact_to_terminal_json,
    worth_store_aspect_native::readmit_external_terminal_projection_document_as_store_aspect_state,
    worth_store_aspect_native::readmit_terminal_json_projection_as_store_aspect_state,
    worth_store_aspect_native::StoreAspectAuthorityInput,
    worth_store_aspect_native::StoreAspectBoundaryFact,
    worth_store_aspect_native::StoreAspectIdentity,
    worth_store_aspect_native::StoreCanonicalBasisConstruction,
    worth_store_aspect_native::StoreCanonicalBasisFamily,
    worth_store_aspect_native::StoreDigestAuthority,
    worth_store_aspect_native::StoreDigestEvidence,
    worth_store_aspect_native::StorePhysicalBoundaryWitness,
    worth_store_aspect_native::StoreTerminalDocumentChecksum,
    worth_store_aspect_native::StoreTerminalProjectionDenial,
    worth_store_aspect_native::StoreTerminalProjectionDisplayLabel,
    worth_store_aspect_native::StoreTerminalProjectionDocumentBytes,
};
pub(crate) use {
    worth_store_contracts::StorePhysicalAuthorityWitness,
    worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};

pub(crate) fn boundary_fact(contract: &AspectContract, raw_value: &str) -> StoreAspectBoundaryFact {
    let validated = match aspects()
        .validate()
        .against(contract)
        .value(aspect_string(raw_value))
    {
        worth_proof::TransitionOutcome::Success(value) => value,
        outcome => panic!("validation should succeed: {outcome:?}"),
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

pub(crate) fn struct_boundary_fact(
    contract: &AspectContract,
    segment: &str,
    generation: u64,
) -> StoreAspectBoundaryFact {
    let struct_value = aspects()
        .vocabulary()
        .struct_value()
        .with_field("segment", aspect_string(segment))
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

pub(crate) fn aspect_key(raw: &str) -> AspectKey {
    aspects().vocabulary().key(raw).unwrap()
}

pub(crate) fn scalar_string_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(29))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::ScalarAspectType::String)
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
        .identified_by(aspects().vocabulary().identity(30))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape)
}

pub(crate) fn aspect_string(value: &str) -> AspectValue {
    worth_foundational::AspectValue::String(worth_foundational::InternedString::from(value))
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

pub(crate) fn terminal_checksum(
    bytes: StoreTerminalProjectionDocumentBytes,
) -> StoreTerminalDocumentChecksum {
    worth_store_aspect_native::StoreTerminalDocumentChecksum::for_terminal_projection_document_bytes(
        &bytes,
    )
}

pub(crate) fn readmitted_external_struct_projection<const N: usize>(
    identity: StoreAspectIdentity,
    contract: AspectContract,
    fields: [(&str, Value); N],
) -> worth_store_aspect_native::StoreTerminalJsonReadmission {
    let mut document = serde_json::Map::new();
    for (field, value) in fields {
        document.insert(field.to_string(), value);
    }

    match readmit_external_terminal_projection_document_as_store_aspect_state(
        identity.clone(),
        serde_json::Value::Object(document),
        contract,
        source_locator(identity.aspect_key().clone()),
        physical_witness(),
    ) {
        worth_proof::TransitionOutcome::Success(readmission) => readmission,
        outcome => panic!("external terminal projection should readmit: {outcome:?}"),
    }
}

pub(crate) struct NativeBasisAndStoreDigestEvidence {
    pub(crate) native_basis: CanonicalBasisReadyArtifact,
    pub(crate) store_digest: StoreDigestEvidence,
}

pub(crate) fn basis_and_digest_for_boundary_fact(
    fact: &StoreAspectBoundaryFact,
) -> NativeBasisAndStoreDigestEvidence {
    let basis = match worth_store_aspect_native::StoreCanonicalBasisConstruction::for_family(
        worth_store_aspect_native::StoreCanonicalBasisFamily::AspectBoundaryFact,
    )
    .with_aspect_boundary_fact(fact)
    .prepare(canonical_rule_version())
    {
        worth_proof::TransitionOutcome::Success(basis) => basis,
        outcome => panic!("canonical basis construction should succeed: {outcome:?}"),
    };

    let digest = match worth_store_aspect_native::StoreDigestAuthority::for_native_basis(
        worth_store_aspect_native::StoreCanonicalBasisFamily::AspectBoundaryFact,
        basis.clone(),
    )
    .derive(worth_foundational::CanonicalDigestAlgorithmId::sha256())
    {
        worth_proof::TransitionOutcome::Success(digest) => digest,
        outcome => panic!("Store digest derivation should succeed: {outcome:?}"),
    };

    NativeBasisAndStoreDigestEvidence {
        native_basis: basis,
        store_digest: digest,
    }
}

pub(crate) fn assert_same_native_basis_and_store_digest(
    left: &NativeBasisAndStoreDigestEvidence,
    right: &NativeBasisAndStoreDigestEvidence,
) {
    assert_eq!(left.native_basis.payload(), right.native_basis.payload());
    assert_eq!(left.store_digest.family(), right.store_digest.family());
    assert_eq!(
        left.store_digest.source_kind(),
        right.store_digest.source_kind()
    );
    assert_eq!(
        left.store_digest.equivalence_basis_identity(),
        right.store_digest.equivalence_basis_identity()
    );
    assert_eq!(
        left.store_digest.canonical_digest().value().bytes(),
        right.store_digest.canonical_digest().value().bytes()
    );
}

pub(crate) fn canonical_rule_version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("worth.store.phase7.terminal-projection")
        .expect("valid canonical rule version")
}
