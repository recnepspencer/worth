//! Completion of a root-protocol replacement the dead runtime interrupted
//! after its selectors but before its catalog.
//!
//! Recovery selects through the current selector; an ordinary open trusts the
//! bootstrap catalog. When the selected root needs no new publication but the
//! catalog still names an older generation, recovery republishes the selected
//! root: the on-disk selectors verbatim and a catalog naming the selected
//! generation, through its ordinary publication command.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCandidate, RecoveryRootProtocolPublicationPlan,
};
use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration, RecordArtifactFile,
    RootSelectorRole,
};

use crate::progression::RecoveryPublicationExpectation;

/// A root-protocol artifact is one fixed frame; anything larger is damage.
const MAXIMUM_PROTOCOL_ARTIFACT_BYTES: u64 = 4096;

pub(super) struct CatalogRepair {
    pub(super) candidates: Box<[PhysicalRecoveryPublicationCandidate]>,
    pub(super) protocol: RecoveryRootProtocolPublicationPlan,
    /// The selected generation, republished exactly as it is on media.
    pub(super) generation: u64,
}

/// `Ok(None)` when the catalog already names the selected generation.
pub(super) fn plan(
    media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    expectation: &RecoveryPublicationExpectation,
) -> Result<Option<CatalogRepair>, ()> {
    let read = |artifact| {
        media
            .read_recovery_root_protocol_artifact(artifact, MAXIMUM_PROTOCOL_ARTIFACT_BYTES)
            .map_err(|_| ())?
            .ok_or(())
    };
    let catalog =
        BootstrapCatalog::decode(&read(RecordArtifactFile::BootstrapCatalog)?).map_err(|_| ())?;
    let selected = expectation.recovered_root().generation();
    let cataloged = catalog.current_root().generation().get();
    if cataloged == selected {
        return Ok(None);
    }
    if cataloged > selected {
        return Err(());
    }
    let publication = publication(expectation.plan_identity(), selected);
    let repaired = BootstrapCatalog::new(
        catalog.store_identity(),
        catalog.format(),
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(selected).ok_or(())?),
    )
    .encode();
    let mut candidates = [
        (
            RecordArtifactFile::RootManifest {
                generation: selected,
            },
            read(RecordArtifactFile::RootManifest {
                generation: selected,
            })?,
        ),
        (
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Previous,
                publication,
            },
            read(RecordArtifactFile::PreviousRootSelector)?,
        ),
        (
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication,
            },
            read(RecordArtifactFile::CurrentRootSelector)?,
        ),
        (
            RecordArtifactFile::CatalogCandidate { publication },
            Box::from(repaired.as_slice()),
        ),
    ]
    .into_iter()
    .map(|(artifact, bytes)| {
        let digest = Sha256::digest(&bytes).into();
        PhysicalRecoveryPublicationCandidate::new(artifact, bytes, digest).ok_or(())
    })
    .collect::<Result<Vec<_>, ()>>()?;
    candidates.sort_by_key(PhysicalRecoveryPublicationCandidate::artifact);
    let protocol = RecoveryRootProtocolPublicationPlan::from_catalog_candidate(
        RecordArtifactFile::CatalogCandidate { publication },
    )
    .map_err(|_| ())?;
    Ok(Some(CatalogRepair {
        candidates: candidates.into_boxed_slice(),
        protocol,
        generation: selected,
    }))
}

/// A deterministic, nonzero publication identity for this plan's repair.
fn publication(plan: [u8; 32], generation: u64) -> u64 {
    let digest = Sha256::new()
        .chain_update(b"worth.store.recovery.catalog-repair.v1")
        .chain_update(plan)
        .chain_update(generation.to_le_bytes())
        .finalize();
    u64::from_le_bytes(digest[..8].try_into().expect("digest prefix")).max(1)
}
