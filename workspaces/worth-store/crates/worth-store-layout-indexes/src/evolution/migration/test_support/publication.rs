use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalReferenceAuthority,
    PhysicalRootReference,
};
use worth_store_physical_isolation::{
    physical_read_stability_authority_for_certification_test, CurrentPhysicalRoot,
    PhysicalOrderingContract, PublicationRootCandidate,
};

pub(super) fn current_publication_source(
    authority: &StoreCurrentAuthorityWitness,
) -> PublicationRootCandidate {
    let store = worth_store_physical_format::PhysicalStoreIdentity::from_aspect_identity(
        authority.identity().clone(),
    );
    let physical =
        physical_read_stability_authority_for_certification_test(20, store.authority_identity());
    let root = CurrentPhysicalRoot::from_physical_isolation_entry(
        physical.root_epoch_basis().current_root_basis(),
        PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .unwrap();
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let cell = generations
        .root_publication_cell(PhysicalRootReference::from_raw(root.scope()).unwrap())
        .with_root_publication_generation(PhysicalGeneration::from_raw(1).unwrap());
    let validation = references
        .validate_root_publication(references.admit_root_publication(cell), cell)
        .unwrap();
    PublicationRootCandidate::admit(root, validation).unwrap()
}
