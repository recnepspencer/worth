// A codec is native contract meaning even when its physical scalar is unchanged.
pub(crate) use {
    worth_foundational::facade::aspects,
    worth_foundational::facade::canonical_basis_sequence_material,
    worth_foundational::facade::prepare_aspect_contract_for_canonical_basis,
    worth_foundational::facade::AspectContract, worth_foundational::facade::AspectContractRevision,
    worth_foundational::facade::AspectEvolutionKind, worth_foundational::facade::AspectIdentity,
    worth_foundational::facade::AspectKey, worth_foundational::facade::CanonicalDigestId,
    worth_foundational::facade::CanonicalizationRuleVersion,
    worth_foundational::facade::PortableAspectContract,
    worth_foundational::facade::ScalarAspectType, worth_foundational::facade::StructAspectShape,
};

pub(crate) fn contract(codec: Option<CanonicalDigestId>) -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .required(
            "payload",
            worth_foundational::facade::ScalarAspectType::String,
        )
        .finish()
        .unwrap();
    let fields = shape.fields().iter().cloned().map(|field| match codec {
        Some(codec) => field.with_codec(codec),
        None => field,
    });
    aspects()
        .contract()
        .for_key(worth_foundational::facade::AspectKey::new("result").unwrap())
        .identified_by(AspectIdentity(23))
        .at_revision(AspectContractRevision(1))
        .struct_aspect(worth_foundational::facade::StructAspectShape::new(fields).unwrap())
}

pub(crate) fn material(contract: AspectContract) -> String {
    let ready = prepare_aspect_contract_for_canonical_basis(
        worth_foundational::facade::CanonicalizationRuleVersion::new("codec-contract-test.v1")
            .unwrap(),
        contract,
    )
    .into_result()
    .unwrap();
    canonical_basis_sequence_material(ready.payload())
}
