pub(crate) use tempfile::tempdir;
pub(crate) use worth_signal::facade::PartitionSubscription;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AspectMask, worth_foundational::MutationMask,
    worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store::aspect_native::StoreAspectContractAdmission,
    worth_store::aspect_native::StoreAspectIdentity,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration,
    worth_store::physical_runtime::PhysicalSignalAspectRole,
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamilySet,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::alternative_physical_witness,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope_from_authority,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT,
};

pub(crate) fn contract_admission(
    key: &str,
    identity: u64,
    scalar: ScalarAspectType,
    witness: worth_store::aspect_native::StorePhysicalBoundaryWitness,
) -> StoreAspectContractAdmission {
    let key = aspects().vocabulary().key(key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(identity))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(scalar);
    worth_store::aspect_native::StoreAspectContractAdmission::new(
        worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key),
        contract,
        witness,
    )
    .unwrap()
    .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
    .unwrap()
}
