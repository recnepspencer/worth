//! Read-only census of checkpoint capture custody at the durability owner.
//!
//! Each observation is one owner visit that copies counts, coordinates and
//! weak allocation identities. It retains nothing, advances no selection or
//! acknowledgement, and grants no currentness, retirement or read authority.
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use std::any::Any;
use std::sync::{Arc, Weak};

/// Weak identity of one owner allocation. It never extends that allocation.
#[derive(Debug, Clone)]
pub struct RelationalAllocationWitness(Weak<dyn Any + Send + Sync>);

impl RelationalAllocationWitness {
    pub(crate) fn of<T: Any + Send + Sync>(owner: &Arc<T>) -> Self {
        let weak: Weak<dyn Any + Send + Sync> =
            Arc::downgrade(owner) as Weak<dyn Any + Send + Sync>;
        Self(weak)
    }
    /// Whether some owner still retains the witnessed allocation.
    pub fn is_retained(&self) -> bool {
        self.0.strong_count() != 0
    }
    /// Whether both witnesses name the same allocation.
    pub fn same_allocation(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

/// Custody posture of one retained canonical commit envelope cell.
#[derive(Debug, Clone)]
pub struct RelationalCommitEnvelopeCustody {
    pub commit_id: crate::history::data::CommitId,
    /// The sealed positioned envelope is resident in this cell.
    pub resident: bool,
    /// The selected checkpoint directory backing this cell, when placed.
    pub selected_directory: Option<RelationalAllocationWitness>,
    /// The retained journal or selected-member index owner backing this cell.
    pub source_placement: Option<RelationalAllocationWitness>,
}

/// The selected checkpoint directory the runtime currently serves from.
#[derive(Debug, Clone)]
pub struct RelationalSelectedCheckpointObservation {
    pub directory: RelationalAllocationWitness,
    pub covered_position: PatchStreamPosition,
    pub index_definition_frontier: u64,
}

/// Checkpoint selection and catalog acceptance, read as two owner visits.
#[derive(Debug, Clone)]
pub struct RelationalCheckpointSelectionCensus {
    pub selected: Option<RelationalSelectedCheckpointObservation>,
    /// The index-definition frontier whose catalog acceptance has completed.
    pub acknowledged_index_definition_frontier: u64,
}

impl RelationalRuntime {
    /// Census every retained commit envelope cell, in catalog order. Cost:
    /// O(retained commits) lock visits; no envelope or member is loaded.
    pub fn observe_commit_envelope_custody(&self) -> Vec<RelationalCommitEnvelopeCustody> {
        self.history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| {
                let cell = artifact.envelope_residency();
                RelationalCommitEnvelopeCustody {
                    commit_id: artifact.commit_id(),
                    resident: cell.positioned_if_resident().is_some(),
                    selected_directory: cell
                        .selected_directory()
                        .map(|directory| RelationalAllocationWitness::of(&directory)),
                    source_placement: cell
                        .source_placement()
                        .map(|(entry, _)| entry.allocation_witness()),
                }
            })
            .collect()
    }

    /// Read one retained commit's original canonical frame from its memory
    /// original or its selected checkpoint member only: never by re-encoding a
    /// resident body and never through the journal. A journal-backed cell
    /// refuses with `SourceUnavailable`. `None` names no retained commit.
    pub fn read_original_commit_artifact_without_journal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: crate::history::data::CommitId,
    ) -> Result<
        Option<crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact>,
        crate::durability::data::RelationalRecoveryStop,
    > {
        self.history
            .commit_artifact(commit_id)
            .map(|artifact| {
                artifact
                    .envelope_residency()
                    .require_original_artifact(execution, None)
            })
            .transpose()
    }

    /// Whether this runtime's attached durable read port is exactly `source`.
    pub fn reads_durable_source(
        &self,
        source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> bool {
        self.durability
            .cold_source()
            .is_some_and(|attached| std::ptr::addr_eq(Arc::as_ptr(&attached), Arc::as_ptr(source)))
    }

    /// Observe the served checkpoint selection and acknowledged catalog frontier.
    pub fn observe_checkpoint_selection(&self) -> RelationalCheckpointSelectionCensus {
        let selection = self.durability.checkpoint_selection();
        RelationalCheckpointSelectionCensus {
            selected: selection.snapshot().map(|directory| {
                RelationalSelectedCheckpointObservation {
                    directory: RelationalAllocationWitness::of(&directory),
                    covered_position: directory.manifest.covered_position,
                    index_definition_frontier: directory.manifest.index_definition_frontier,
                }
            }),
            acknowledged_index_definition_frontier: selection
                .acknowledged_index_definition_frontier(),
        }
    }
}
