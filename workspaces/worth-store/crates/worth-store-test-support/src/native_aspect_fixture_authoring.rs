use worth_foundational::{
    aspects, AspectContract, AspectKey, AspectLocator, AspectValue, AspectValueLocator,
    AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectStateArtifact,
    ContractValidatedAspectArtifact, ContractValidationInput, InternedString, LocatorAuthority,
    ScalarAspectType, StructAspectValue,
};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectBoundaryLocator,
    StoreAspectFieldBoundaryLocator, StoreAspectIdentity, StoreAspectPatchAuthorityInput,
    StoreAspectPatchBoundaryFact, StoreAspectValueBoundaryLocator, StorePhysicalBoundaryWitness,
};
use worth_store_contracts::{
    PhysicalAuthorityBoundaryInstance, StorePhysicalAuthorityWitness,
    ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE, ROADMAP_2_PRIMARY_PHYSICAL_BOUNDARY,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthoredNativeStoreAspectFixture {
    pub(crate) identity: StoreAspectIdentity,
    pub(crate) contract: AspectContract,
    pub(crate) scalar_value: Option<AspectValue>,
    pub(crate) struct_value: Option<StructAspectValue>,
    pub(crate) validated_value: ContractValidatedAspectArtifact,
    pub(crate) authoritative_state: AuthoritativeRecordAspectStateArtifact,
    pub(crate) boundary_fact: StoreAspectBoundaryFact,
    pub(crate) patch_fact: StoreAspectPatchBoundaryFact,
    pub(crate) aspect_locator: StoreAspectBoundaryLocator,
    pub(crate) value_locator: StoreAspectValueBoundaryLocator,
    pub(crate) field_locator: Option<StoreAspectFieldBoundaryLocator>,
    pub(crate) physical_witness: StorePhysicalBoundaryWitness,
}

pub(crate) fn authored_segment_header_fixture(
    segment: &str,
    generation: u64,
) -> AuthoredNativeStoreAspectFixture {
    let aspect_key = aspect_key("store.physical.segment.header");
    let contract = segment_header_contract(aspect_key.clone());
    let struct_value = segment_header_value(segment, generation);
    let physical_witness = physical_witness();
    let field_locator = Some(store_field_locator(
        StoreAspectIdentity::from_aspect_key(aspect_key.clone()),
        aspect_key.clone(),
        "segment",
    ));

    authored_fixture_from_parts(
        StoreAspectIdentity::from_aspect_key(aspect_key),
        contract,
        None,
        Some(struct_value.clone()),
        ContractValidationInput::from(struct_value),
        physical_witness,
        field_locator,
    )
}

pub(crate) fn authored_scalar_string_fixture(value: &str) -> AuthoredNativeStoreAspectFixture {
    authored_scalar_string_fixture_on_boundary(value, ROADMAP_2_PRIMARY_PHYSICAL_BOUNDARY)
}

fn authored_scalar_string_fixture_on_boundary(
    value: &str,
    boundary_instance: PhysicalAuthorityBoundaryInstance,
) -> AuthoredNativeStoreAspectFixture {
    let aspect_key = aspect_key("store.physical.segment.identity");
    let contract = scalar_string_contract(aspect_key.clone());
    let scalar_value = AspectValue::String(InternedString::from(value));
    let physical_witness = physical_witness_for_boundary(boundary_instance);

    authored_fixture_from_parts(
        StoreAspectIdentity::from_aspect_key(aspect_key),
        contract,
        Some(scalar_value.clone()),
        None,
        ContractValidationInput::from(scalar_value),
        physical_witness,
        None,
    )
}

fn authored_fixture_from_parts(
    identity: StoreAspectIdentity,
    contract: AspectContract,
    scalar_value: Option<AspectValue>,
    struct_value: Option<StructAspectValue>,
    validation_input: ContractValidationInput,
    physical_witness: StorePhysicalBoundaryWitness,
    field_locator: Option<StoreAspectFieldBoundaryLocator>,
) -> AuthoredNativeStoreAspectFixture {
    let validated_value = validate_native_value(&contract, validation_input);
    let authoritative_state = admit_authoritative_state(validated_value.clone());
    let boundary_fact = store_boundary_fact(
        identity.clone(),
        authoritative_state.clone(),
        physical_witness,
    );
    let patch_fact = store_patch_fact(
        identity.clone(),
        whole_aspect_patch(validated_value.clone()),
        physical_witness,
    );
    let store_aspect_locator =
        StoreAspectBoundaryLocator::new(identity.clone(), aspect_locator(contract.key().clone()))
            .unwrap();
    let value_locator = StoreAspectValueBoundaryLocator::new(
        identity.clone(),
        AspectValueLocator::whole_aspect(aspect_locator(contract.key().clone())),
    )
    .unwrap();

    AuthoredNativeStoreAspectFixture {
        identity,
        contract,
        scalar_value,
        struct_value,
        validated_value,
        authoritative_state,
        boundary_fact,
        patch_fact,
        aspect_locator: store_aspect_locator,
        value_locator,
        field_locator,
        physical_witness,
    }
}

fn aspect_key(raw: &str) -> AspectKey {
    aspects().vocabulary().key(raw).unwrap()
}

fn scalar_string_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(41))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}

fn segment_header_contract(aspect_key: AspectKey) -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .required("segment", ScalarAspectType::String)
        .required("generation", ScalarAspectType::UInt64)
        .finish()
        .unwrap();

    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(aspects().vocabulary().identity(42))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape)
}

fn segment_header_value(segment: &str, generation: u64) -> StructAspectValue {
    aspects()
        .vocabulary()
        .struct_value()
        .with_field(
            "segment",
            AspectValue::String(InternedString::from(segment)),
        )
        .with_field("generation", AspectValue::UInt64(generation))
        .finish()
        .unwrap()
}

fn validate_native_value(
    contract: &AspectContract,
    value: ContractValidationInput,
) -> ContractValidatedAspectArtifact {
    match aspects().validate().against(contract).value(value) {
        TransitionOutcome::Success(validated) => validated,
        outcome => panic!("native fixture value should validate: {outcome:?}"),
    }
}

fn admit_authoritative_state(
    validated_value: ContractValidatedAspectArtifact,
) -> AuthoritativeRecordAspectStateArtifact {
    match aspects().authoritative_state().admit([validated_value]) {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("native fixture state should admit: {outcome:?}"),
    }
}

fn whole_aspect_patch(
    validated_value: ContractValidatedAspectArtifact,
) -> AuthoritativeRecordAspectPatch {
    match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value)
        .finish()
    {
        TransitionOutcome::Success(patch) => patch,
        outcome => panic!("native fixture patch should construct: {outcome:?}"),
    }
}

fn store_boundary_fact(
    identity: StoreAspectIdentity,
    state: AuthoritativeRecordAspectStateArtifact,
    physical_witness: StorePhysicalBoundaryWitness,
) -> StoreAspectBoundaryFact {
    StoreAspectBoundaryFact::from_admitted_state(
        identity,
        StoreAspectAuthorityInput::new(state, physical_witness),
    )
    .unwrap()
}

fn store_patch_fact(
    identity: StoreAspectIdentity,
    patch: AuthoritativeRecordAspectPatch,
    physical_witness: StorePhysicalBoundaryWitness,
) -> StoreAspectPatchBoundaryFact {
    StoreAspectPatchBoundaryFact::from_authoritative_patch(
        identity,
        StoreAspectPatchAuthorityInput::new(patch, physical_witness),
    )
    .unwrap()
}

fn aspect_locator(aspect_key: AspectKey) -> AspectLocator {
    AspectLocator::new(LocatorAuthority::Projected, aspect_key)
}

fn store_field_locator(
    identity: StoreAspectIdentity,
    aspect_key: AspectKey,
    field: &str,
) -> StoreAspectFieldBoundaryLocator {
    let field_path = aspects().vocabulary().field_path([field]).unwrap();
    StoreAspectFieldBoundaryLocator::new(
        identity,
        worth_foundational::AspectFieldLocator::new(
            LocatorAuthority::Projected,
            aspect_key,
            field_path,
        ),
    )
    .unwrap()
}

fn physical_witness() -> StorePhysicalBoundaryWitness {
    physical_witness_for_boundary(ROADMAP_2_PRIMARY_PHYSICAL_BOUNDARY)
}

fn physical_witness_for_boundary(
    boundary_instance: PhysicalAuthorityBoundaryInstance,
) -> StorePhysicalBoundaryWitness {
    StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary_instance(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
            boundary_instance,
        )
        .unwrap(),
    )
    .unwrap()
}
