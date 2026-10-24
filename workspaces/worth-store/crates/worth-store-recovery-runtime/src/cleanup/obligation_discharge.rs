//! Which of the dead runtime's physical-work obligations this recovery has
//! adjudicated, and their discharge.
//!
//! An obligation says an effect may or may not have reached media, or that a
//! synchronization may not have made it durable. Recovery discharges one only
//! when its own publication or checkpoint selection settled it:
//!
//! - WAL intervals: recovery replayed the admitted tail into artifacts it
//!   staged and synchronized itself; frames it did not admit are unreachable.
//!   This holds whether or not recovery had anything to publish.
//! - Record-namespace synchronization and catalog replacement: recovery's own
//!   root-protocol replacement and namespace synchronization supersede them.
//! - Record ranges and artifact synchronization: only for an artifact recovery
//!   rewrote and synchronized, or one the recovered root does not reach.
//! - Store checkpoint actions: by recovery's own checkpoint selection, as
//!   `checkpoint_obligation` decides; one for the selected checkpoint only
//!   after recovery has made its publication namespace durable.
//!
//! Record removal and WAL-reclamation obligations, records that do not admit,
//! and records of more than one runtime incarnation are retained, so serving
//! still requires inspection for them. A removal's outcome is not proven by
//! anything recovery observes: an absent artifact may have been removed or
//! never written, and recovery's WAL admission does not yet prove an absent
//! segment lay below the selected checkpoint's retained tail. Obligations
//! other than WAL intervals and checkpoint actions need recovery's own
//! publication; recovery republishes the selected root when one is present
//! (`requires_republication`).

use std::collections::BTreeSet;

use worth_store::physical_runtime::{
    PhysicalRecoveryYieldpointStage, PhysicalWorkRecoveryTarget, RecoveryPriorObligation,
    SelectedCheckpointPublicationDenial,
};
use worth_store_physical_format::RecordArtifactFile;

use crate::entry::PhysicalRecoveryPublicationSettlement;
use crate::progression::ReopenedPhysicalRecovery;

use super::checkpoint_obligation::{self, CheckpointObligationVerdict, SettledCheckpointResidue};

/// The bound on prior-runtime obligation records one recovery reads.
pub(crate) const MAXIMUM_PRIOR_OBLIGATIONS: usize = 1024;

pub(super) enum DischargeFailure {
    /// The obligation journal held damaged evidence; nothing was removed.
    Damaged,
    /// A removal or synchronization started but did not settle.
    Indeterminate,
}

/// How an adjudicated obligation is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Settlement {
    Settled,
    /// Settled once the publication of the checkpoint recovery selected is
    /// namespace durable.
    AfterSelectedCheckpointPublication,
}

/// What recovery's own work proved, against which each obligation is judged.
struct AdjudicationBasis {
    published: bool,
    created: BTreeSet<RecordArtifactFile>,
    reached: BTreeSet<RecordArtifactFile>,
    checkpoint: SettledCheckpointResidue,
}

pub(super) fn discharge(
    reopened: &ReopenedPhysicalRecovery,
    checkpoint: SettledCheckpointResidue,
) -> Result<(), DischargeFailure> {
    let owner = reopened.state.coordination.owner();
    let media = &reopened.state.authority.media;
    let obligations = owner
        .prior_physical_work_obligations(media, MAXIMUM_PRIOR_OBLIGATIONS)
        .map_err(|_| DischargeFailure::Damaged)?;
    if obligations.is_empty() || !single_incarnation(&obligations) {
        return Ok(());
    }
    let basis = AdjudicationBasis::from_recovery(reopened, checkpoint);
    let settled: Vec<_> = obligations
        .iter()
        .filter_map(|obligation| {
            let settlement = basis.settlement(obligation.locator()?.target())?;
            Some((obligation, settlement))
        })
        .collect();
    if settled.is_empty() {
        return Ok(());
    }
    if settled
        .iter()
        .any(|(_, settlement)| *settlement == Settlement::AfterSelectedCheckpointPublication)
    {
        make_selected_checkpoint_publication_durable(reopened)?;
    }
    if owner
        .pause_at(PhysicalRecoveryYieldpointStage::ObligationDischarge)
        .is_interrupted()
    {
        return Err(DischargeFailure::Indeterminate);
    }
    for (obligation, _) in settled {
        owner
            .discharge_prior_physical_work_obligation(media, obligation)
            .map_err(|_| DischargeFailure::Indeterminate)?;
    }
    Ok(())
}

impl AdjudicationBasis {
    fn from_recovery(
        reopened: &ReopenedPhysicalRecovery,
        checkpoint: SettledCheckpointResidue,
    ) -> Self {
        Self {
            published: matches!(
                reopened.publication_settlement.settlement(),
                PhysicalRecoveryPublicationSettlement::Completed(_)
            ),
            created: reopened
                .expectation
                .created_artifacts()
                .iter()
                .copied()
                .collect(),
            reached: reopened
                .expectation
                .referenced_artifacts()
                .iter()
                .chain(reopened.state.base.source_artifacts())
                .copied()
                .collect(),
            checkpoint,
        }
    }

    fn settlement(&self, target: PhysicalWorkRecoveryTarget) -> Option<Settlement> {
        match target {
            PhysicalWorkRecoveryTarget::Checkpoint { sequence, action } => {
                match checkpoint_obligation::adjudicate(self.checkpoint, sequence, action) {
                    CheckpointObligationVerdict::UnselectedCandidateRemoved => {
                        Some(Settlement::Settled)
                    }
                    CheckpointObligationVerdict::SelectedPublication => {
                        Some(Settlement::AfterSelectedCheckpointPublication)
                    }
                    CheckpointObligationVerdict::Retained => None,
                }
            }
            PhysicalWorkRecoveryTarget::WalArtifactInterval { .. } => Some(Settlement::Settled),
            _ => (self.published && adjudicated(target, &self.created, &self.reached))
                .then_some(Settlement::Settled),
        }
    }
}

/// The selected checkpoint's bytes were admitted by recovery; a published
/// checkpoint that no longer matches them, or cannot be read, is damage.
fn make_selected_checkpoint_publication_durable(
    reopened: &ReopenedPhysicalRecovery,
) -> Result<(), DischargeFailure> {
    let owner = reopened.state.coordination.owner();
    let selected = owner.checkpoint().ok_or(DischargeFailure::Damaged)?;
    owner
        .synchronize_selected_checkpoint_publication(&reopened.state.authority.media, selected)
        .map_err(|denial| match denial {
            SelectedCheckpointPublicationDenial::CheckpointMismatch
            | SelectedCheckpointPublicationDenial::Observation(_) => DischargeFailure::Damaged,
            SelectedCheckpointPublicationDenial::Synchronization(_) => {
                DischargeFailure::Indeterminate
            }
        })
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
