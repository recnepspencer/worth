//! Which of the dead runtime's physical-work obligations this recovery has
//! adjudicated, and their discharge.
//!
//! An obligation says an effect may or may not have reached media, or that a
//! synchronization may not have made it durable. Recovery discharges one only
//! when its own publication settled it:
//!
//! - WAL intervals: recovery replayed the admitted tail into artifacts it
//!   staged and synchronized itself; frames it did not admit are unreachable.
//!   This holds whether or not recovery had anything to publish.
//! - Record-namespace synchronization and catalog replacement: recovery's own
//!   root-protocol replacement and namespace synchronization supersede them.
//! - Record ranges and artifact synchronization: only for an artifact recovery
//!   rewrote and synchronized, or one the recovered root does not reach.
//!
//! Record removal, checkpoint and WAL-reclamation obligations, records that do
//! not admit, and records of more than one runtime incarnation are retained,
//! so serving still requires inspection for them. Every obligation other than
//! a WAL interval needs recovery's own publication; recovery republishes the
//! selected root when one is present (`requires_republication`).

use std::collections::BTreeSet;

use worth_store::physical_runtime::{PhysicalWorkRecoveryTarget, RecoveryPriorObligation};
use worth_store_physical_format::RecordArtifactFile;

use crate::entry::PhysicalRecoveryPublicationSettlement;
use crate::progression::ReopenedPhysicalRecovery;

/// The bound on prior-runtime obligation records one recovery reads.
pub(crate) const MAXIMUM_PRIOR_OBLIGATIONS: usize = 1024;

pub(super) enum DischargeFailure {
    /// The obligation journal held damaged evidence; nothing was removed.
    Damaged,
    /// A removal started but did not settle.
    Indeterminate,
}

pub(super) fn discharge(reopened: &ReopenedPhysicalRecovery) -> Result<(), DischargeFailure> {
    let owner = reopened.state.coordination.owner();
    let media = &reopened.state.authority.media;
    let obligations = owner
        .prior_physical_work_obligations(media, MAXIMUM_PRIOR_OBLIGATIONS)
        .map_err(|_| DischargeFailure::Damaged)?;
    let published = matches!(
        reopened.publication_settlement.settlement(),
        PhysicalRecoveryPublicationSettlement::Completed(_)
    );
    if obligations.is_empty() || !single_incarnation(&obligations) {
        return Ok(());
    }
    let created: BTreeSet<RecordArtifactFile> = reopened
        .expectation
        .created_artifacts()
        .iter()
        .copied()
        .collect();
    let reached: BTreeSet<RecordArtifactFile> = reopened
        .expectation
        .referenced_artifacts()
        .iter()
        .chain(reopened.state.base.source_artifacts())
        .copied()
        .collect();
    for obligation in &obligations {
        let Some(locator) = obligation.locator() else {
            continue;
        };
        let target = locator.target();
        let wal = matches!(
            target,
            PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
        );
        if (wal || published) && adjudicated(target, &created, &reached) {
            owner
                .discharge_prior_physical_work_obligation(media, obligation)
                .map_err(|_| DischargeFailure::Indeterminate)?;
        }
    }
    Ok(())
}

/// Whether any obligation is one that only recovery's own root-protocol
/// publication and namespace synchronization can settle.
pub(crate) fn requires_republication(obligations: &[RecoveryPriorObligation]) -> bool {
    obligations.iter().any(|obligation| {
        obligation.locator().is_some_and(|locator| {
            matches!(
                locator.target(),
                PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization
                    | PhysicalWorkRecoveryTarget::CatalogReplacement(_)
                    | PhysicalWorkRecoveryTarget::Range(_)
                    | PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(_)
                    | PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(_)
            )
        })
    })
}

/// Every record admitted and belongs to one runtime incarnation.
fn single_incarnation(obligations: &[RecoveryPriorObligation]) -> bool {
    let mut runtime = None;
    obligations.iter().all(|obligation| {
        obligation
            .locator()
            .is_some_and(|locator| *runtime.get_or_insert(locator.runtime()) == locator.runtime())
    })
}

fn adjudicated(
    target: PhysicalWorkRecoveryTarget,
    created: &BTreeSet<RecordArtifactFile>,
    reached: &BTreeSet<RecordArtifactFile>,
) -> bool {
    let settled = |artifact: RecordArtifactFile| {
        created.contains(&artifact) || is_root_protocol(artifact) || !reached.contains(&artifact)
    };
    match target {
        PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
        | PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization
        | PhysicalWorkRecoveryTarget::CatalogReplacement(_) => true,
        PhysicalWorkRecoveryTarget::Range(coordinate) => settled(coordinate.artifact()),
        PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(artifact)
        | PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(artifact) => settled(artifact),
        PhysicalWorkRecoveryTarget::ArtifactRemoval(_)
        | PhysicalWorkRecoveryTarget::Checkpoint { .. }
        | PhysicalWorkRecoveryTarget::WalSegmentReclamation { .. } => false,
    }
}

/// Recovery's own publication rewrote and synchronized every root-protocol
/// artifact.
const fn is_root_protocol(artifact: RecordArtifactFile) -> bool {
    matches!(
        artifact,
        RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector
            | RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootSelectorCandidate { .. }
    )
}
