use worth_foundational::aspects;
use worth_proof::TransitionOutcome;
use worth_store::aspect_native::{StoreAspectPatchAuthorityInput, StoreAspectPatchBoundaryFact};
use worth_store::physical_runtime::{
    PhysicalMutationWorkRequest, PhysicalWorkProfileDeclaration, PhysicalWorkScope,
    PhysicalWorkSemanticBasis,
};
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::authority::{admitted_contract, security_scope, validated_value};

pub(in crate::physical_work) fn disjoint_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 16, 8).unwrap(),
    )
}

pub(in crate::physical_work) fn foreground_saturation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    [PhysicalMutationWorkRequest; 4],
) {
    let (profile, basis, security) = mutation_basis();
    let request = |offset| {
        PhysicalMutationWorkRequest::exact_write(
            PhysicalWorkScope::one(
                RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, offset, 8)
                    .unwrap(),
            ),
            basis.clone(),
            security,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap()
    };
    (profile, [request(8), request(16), request(24), request(32)])
}

pub(in crate::physical_work) fn overlapping_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 12, 8).unwrap(),
    )
}

pub(in crate::physical_work) fn whole_catalog_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    let (profile, basis, security) = mutation_basis();
    let range = PhysicalMutationWorkRequest::exact_write(
        PhysicalWorkScope::one(
            RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap(),
        ),
        basis.clone(),
        security,
        ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    let whole = PhysicalMutationWorkRequest::exact_write(
        PhysicalWorkScope::artifact(RecordArtifactFile::BootstrapCatalog),
        basis,
        security,
        ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    (profile, range, whole)
}

pub(in crate::physical_work) fn disjoint_artifact_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 1 }, 0, 8)
            .unwrap(),
    )
}

fn mutation_basis() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalWorkSemanticBasis,
    worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
) {
    let (contract, identity, contract_admission, physical_witness) = admitted_contract(1);
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(&contract, "disjoint-mutations"))
        .finish()
    {
        TransitionOutcome::Success(patch) => patch,
        outcome => panic!("patch should construct: {outcome:?}"),
    };
    let patch_fact = StoreAspectPatchBoundaryFact::from_authoritative_patch(
        identity,
        StoreAspectPatchAuthorityInput::new(patch, physical_witness),
    )
    .unwrap();
    let basis =
        PhysicalWorkSemanticBasis::mutation(patch_fact, contract_admission.clone()).unwrap();
    (
        PhysicalWorkProfileDeclaration::new(security_scope(physical_witness), [contract_admission])
            .unwrap(),
        basis,
        security_scope(physical_witness),
    )
}

fn mutation_pair_fixture(
    second_coordinate: RecordFrameCoordinate,
) -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    let (profile, basis, security) = mutation_basis();
    let request = |offset, basis| {
        PhysicalMutationWorkRequest::exact_write(
            PhysicalWorkScope::one(
                RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, offset, 8)
                    .unwrap(),
            ),
            basis,
            security,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap()
    };
    (
        profile,
        request(8, basis.clone()),
        PhysicalMutationWorkRequest::exact_write(
            PhysicalWorkScope::one(second_coordinate),
            basis,
            security,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap(),
    )
}
