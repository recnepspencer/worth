use worth_store_physical_backend::{ArtifactTreeFailureKind, QualifiedFilesystemMedia};
use worth_store_physical_format::{
    durable_artifact_checksum, BootstrapCatalog, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, RecordArtifactFile, RootSelectorIdentity, RootSelectorRole,
};

use super::super::residency::initialization_artifacts::InitializationRecordArtifacts;
use super::super::{
    admission::bootstrap::{
        backend_after_effect, backend_before_effect, BootstrapTransitionFailure,
        PhysicalRecordBootstrapOwner,
    },
    residency::artifact_tree::{RecordFamilyCreationFailure, RecordFamilyInventory},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    RecordBootstrapDenial, RecordBootstrapFailure,
};

pub(in crate::physical_runtime::record_serving) fn initialize(
    media: &QualifiedFilesystemMedia,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
) -> Result<PhysicalRecordBootstrapOwner, BootstrapTransitionFailure> {
    if !placement.admits(format) || !access.admits(format) {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::ConfigurationMismatch,
        ));
    }
    let artifacts = InitializationRecordArtifacts::new(media);
    match artifacts.inventory().map_err(backend_before_effect)? {
        RecordFamilyInventory::ProvenAbsent => {}
        RecordFamilyInventory::Published => {
            return Err(BootstrapTransitionFailure::Denied(
                RecordBootstrapDenial::RecordFamilyAlreadyExists,
            ));
        }
        RecordFamilyInventory::Residue => {
            return Err(BootstrapTransitionFailure::Denied(
                RecordBootstrapDenial::AmbiguousRecordFamilyResidue,
            ));
        }
    }
    let declaration = format.declaration();
    let arena_alignment = super::super::arena::qualified_arena_alignment(media).ok_or(
        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::ConfigurationMismatch),
    )?;
    let tree_identity = new_tree_identity()?;
    let free_space = DurableFreeSpaceManifestHeader::new(
        1,
        tree_identity,
        placement.manifest_capacity().get(),
        placement.segment_pages().get(),
        0,
        1,
        1,
        1,
        1,
        placement.arena_capacity().get(),
        arena_alignment,
        1,
        None,
    )
    .ok_or(BootstrapTransitionFailure::Failed(
        RecordBootstrapFailure::FormatEncoding,
    ))?;
    let free_space_bytes = free_space.encode(declaration);
    let current_root = DurablePhysicalRootManifest::builder(
        1,
        tree_identity,
        placement.manifest_capacity().get(),
        durable_artifact_checksum(&free_space_bytes),
    )
    .free_space_root(free_space.root())
    .admit()
    .ok_or(BootstrapTransitionFailure::Failed(
        RecordBootstrapFailure::FormatEncoding,
    ))?;
    let root_generation = CurrentRootCatalogGeneration::new(1).ok_or(
        BootstrapTransitionFailure::Failed(RecordBootstrapFailure::FormatEncoding),
    )?;
    let catalog = BootstrapCatalog::new(
        media.store_identity(),
        declaration,
        CurrentRootCatalogEntry::new(root_generation),
    )
    .encode();
    let current_selector = DurableRootSelector::new(
        media.store_identity(),
        declaration,
        RootSelectorIdentity::new(1).expect("initial selector identity is nonzero"),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .expect("initial root selector has a valid unlinked current role")
    .encode();

    artifacts
        .create_record_family()
        .map_err(|failure| match failure {
            RecordFamilyCreationFailure::BeforeEffect(failure)
                if failure.kind() == ArtifactTreeFailureKind::DeniedBeforeEffect =>
            {
                backend_before_effect(failure)
            }
            RecordFamilyCreationFailure::BeforeEffect(failure)
                if failure.kind() == ArtifactTreeFailureKind::AlreadyExists =>
            {
                BootstrapTransitionFailure::Denied(
                    RecordBootstrapDenial::AmbiguousRecordFamilyResidue,
                )
            }
            RecordFamilyCreationFailure::BeforeEffect(failure)
            | RecordFamilyCreationFailure::AfterEffect(failure) => backend_after_effect(failure),
        })?;
    let root_artifact = RecordArtifactFile::RootManifest { generation: 1 };
    artifacts
        .write_new(root_artifact, &current_root.encode(declaration))
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact(root_artifact)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact_parent(root_artifact)
        .map_err(backend_after_effect)?;
    let free_space_artifact = RecordArtifactFile::FreeSpaceManifest { generation: 1 };
    artifacts
        .write_new(free_space_artifact, &free_space_bytes)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact(free_space_artifact)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact_parent(free_space_artifact)
        .map_err(backend_after_effect)?;

    let current_selector_artifact = RecordArtifactFile::CurrentRootSelector;
    artifacts
        .write_new(current_selector_artifact, &current_selector)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact(current_selector_artifact)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact_parent(current_selector_artifact)
        .map_err(backend_after_effect)?;

    let candidate = RecordArtifactFile::CatalogCandidate { publication: 1 };
    artifacts
        .write_new(candidate, &catalog)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_artifact(candidate)
        .map_err(backend_after_effect)?;
    artifacts
        .replace_catalog(candidate)
        .map_err(backend_after_effect)?;
    artifacts
        .synchronize_record_family()
        .map_err(backend_after_effect)?;
    Ok(PhysicalRecordBootstrapOwner {
        format,
        access,
        current_root: CurrentRootCatalogEntry::new(root_generation),
        observed_staging_residue: false,
    })
}

fn new_tree_identity() -> Result<u64, BootstrapTransitionFailure> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).map_err(|_| {
        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::IdentityEntropyUnavailable)
    })?;
    let identity = u64::from_le_bytes(bytes);
    (identity != 0)
        .then_some(identity)
        .ok_or(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::IdentityEntropyUnavailable,
        ))
}
