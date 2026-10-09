//! Canonical selector/catalog bytes for the recovery candidate cutover.

use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, BootstrapCatalog, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, DurableRootSelector, PhysicalRecordFormatDeclaration,
    RecordArtifactFile, RootSelectorIdentity, RootSelectorRole,
};

use super::{CandidateBuild, CandidateBuildDenial};

pub(super) fn push_candidates(
    build: &mut CandidateBuild,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    selected: DurableRootSelector,
    generation: u64,
    publication: u64,
) -> Result<DurableRootSelector, CandidateBuildDenial> {
    let previous_identity = selected.identity();
    let current_identity =
        RootSelectorIdentity::new(generation).ok_or(CandidateBuildDenial::Invalid)?;
    let previous = DurableRootSelector::new(
        store,
        format,
        previous_identity,
        RootSelectorRole::Previous,
        selected.root_generation(),
        Some(current_identity),
        Some(generation),
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let current = DurableRootSelector::new(
        store,
        format,
        current_identity,
        RootSelectorRole::Current,
        generation,
        Some(previous_identity),
        Some(selected.root_generation()),
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let catalog = BootstrapCatalog::new(
        store,
        format,
        CurrentRootCatalogEntry::new(
            CurrentRootCatalogGeneration::new(generation).ok_or(CandidateBuildDenial::Invalid)?,
        ),
    );
    push_encoded(
        build,
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            publication,
        },
        previous.encode(),
    )?;
    push_encoded(
        build,
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            publication,
        },
        current.encode(),
    )?;
    push_encoded(
        build,
        RecordArtifactFile::CatalogCandidate { publication },
        catalog.encode(),
    )?;
    Ok(current)
}

fn push_encoded<const N: usize>(
    build: &mut CandidateBuild<'_>,
    artifact: RecordArtifactFile,
    encoded: [u8; N],
) -> Result<(), CandidateBuildDenial> {
    let mut bytes = build.allowance.reserve::<u8>(N)?;
    bytes.extend_from_slice(&encoded);
    build.push(artifact, bytes)
}
