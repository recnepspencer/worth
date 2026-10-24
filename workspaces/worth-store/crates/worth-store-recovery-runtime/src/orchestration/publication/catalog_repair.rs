//! Republication of the selected root when recovery would otherwise publish
//! nothing but the root protocol is unsettled.
//!
//! Recovery selects through the current selector; an ordinary open trusts the
//! bootstrap catalog. Two states need recovery's own root-protocol publication
//! even when no new root is due:
//!
//! - the dead runtime replaced its selectors but not its catalog, so the
//!   catalog names the previous selector's generation; and
//! - the dead runtime left physical-work obligations, so its renames and
//!   synchronizations are unproven until recovery performs its own.
//!
//! The republication is the selected root manifest and both selectors exactly
//! as recovery admitted them, and a catalog naming the selected generation,
//! through recovery's ordinary publication command.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCandidate, RecoveryRootProtocolPublicationPlan,
};
use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration,
    DurablePhysicalRootManifest, DurableRootSelector, RecordArtifactFile, RootSelectorRole,
};

use crate::progression::RecoveryPublicationExpectation;

/// A root-protocol artifact is one fixed frame; anything larger is damage.
const MAXIMUM_PROTOCOL_ARTIFACT_BYTES: u64 = 4096;

pub(super) struct RootRepublication {
    pub(super) candidates: Box<[PhysicalRecoveryPublicationCandidate]>,
    pub(super) protocol: RecoveryRootProtocolPublicationPlan,
    /// The selected generation, republished exactly as it is on media.
    pub(super) generation: u64,
}

/// `Ok(None)` when the catalog names the selected generation and nothing
/// forces a republication. `Err` refuses an inconsistent root protocol.
pub(super) fn plan(
    media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    expectation: &RecoveryPublicationExpectation,
    unproven_effects: bool,
) -> Result<Option<RootRepublication>, ()> {
    let read = |artifact| {
        media
            .read_recovery_root_protocol_artifact(artifact, MAXIMUM_PROTOCOL_ARTIFACT_BYTES)
            .map_err(|_| ())?
            .ok_or(())
    };
    let store = media.store_identity();
    let catalog =
        BootstrapCatalog::decode(&read(RecordArtifactFile::BootstrapCatalog)?).map_err(|_| ())?;
    let selected_root = expectation.recovered_root();
    let selected = selected_root.generation();
    let cataloged = catalog.current_root().generation().get();
    if catalog.store_identity() != store || cataloged > selected {
        return Err(());
    }
    if cataloged == selected && !unproven_effects {
        return Ok(None);
    }
    let root = read(RecordArtifactFile::RootManifest {
        generation: selected,
    })?;
    let current = read(RecordArtifactFile::CurrentRootSelector)?;
    let previous = read(RecordArtifactFile::PreviousRootSelector)?;
    let admitted_root = DurablePhysicalRootManifest::decode(&root, u16::MAX)
        .map(|(manifest, _)| manifest)
        .map_err(|_| ())?;
    let current_selector = DurableRootSelector::decode(&current).map_err(|_| ())?;
    let previous_selector = DurableRootSelector::decode(&previous).map_err(|_| ())?;
    // Republish only what recovery admitted; an interrupted replacement can
    // only have left the catalog at the previous selector's generation.
    if admitted_root != *selected_root
        || current_selector != expectation.current_selector()
        || previous_selector.store_identity() != store
        || (cataloged != selected && previous_selector.root_generation() != cataloged)
    {
        return Err(());
    }
    let publication = publication(expectation.plan_identity(), selected);
    let repaired = BootstrapCatalog::new(
        store,
        catalog.format(),
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(selected).ok_or(())?),
    )
    .encode();
    let mut candidates = [
        (
            RecordArtifactFile::RootManifest {
                generation: selected,
            },
            root,
        ),
        (
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Previous,
                publication,
            },
            previous,
        ),
        (
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication,
            },
            current,
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
    Ok(Some(RootRepublication {
        candidates: candidates.into_boxed_slice(),
        protocol,
        generation: selected,
    }))
}

/// A deterministic, nonzero publication identity for this plan's republication.
fn publication(plan: [u8; 32], generation: u64) -> u64 {
    let digest = Sha256::new()
        .chain_update(b"worth.store.recovery.root-republication.v1")
        .chain_update(plan)
        .chain_update(generation.to_le_bytes())
        .finalize();
    u64::from_le_bytes(digest[..8].try_into().expect("digest prefix")).max(1)
}
