//! Capture canonical meaning from the original Native graph selection.
use super::RelationalOwnedModelGraphSelection;
use crate::history::data::canonical_commit_envelope::owned_graph_selection::{
    CanonicalOwnedGraphSelection, SelectedEntity,
};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::{
    ConflictClass, MutationStateInconsistencyEvidence,
};
use crate::transactions::data::outcomes::transaction_commit_error::{
    origin_denial, TransactionCommitError,
};
use crate::transactions::data::primitives::RecordRef;

impl RelationalOwnedModelGraphSelection {
    pub(crate) fn canonical_meaning(
        &self,
    ) -> Result<CanonicalOwnedGraphSelection, TransactionCommitError> {
        let entities = self.entities();
        let mut selected_entities = Vec::new();
        for &id in self.restored_entities() {
            let kind = entities
                .selected_basis()
                .inner
                .root
                .get_current_partition(id.partition_id)
                .and_then(|region| region.entity(id))
                .filter(|slot| slot.is_live())
                .and_then(|slot| slot.kind_id())
                .ok_or_else(|| {
                    TransactionCommitError::conflict(CommitConflict::new(
                        ConflictClass::MutationStateInconsistency {
                            detail: "prepared graph selected entity unavailable".into(),
                            evidence:
                                MutationStateInconsistencyEvidence::MaterializationTransition {
                                    record: RecordRef::Entity(id),
                                },
                        },
                    ))
                })?;
            selected_entities.push(SelectedEntity { id, kind });
        }
        let selected_basis = entities
            .selected_basis()
            .descriptor()
            .in_canonical_frame()
            .map_err(|_| origin_denial("graph selection basis cannot enter canonical frame"))?;
        Ok(CanonicalOwnedGraphSelection::from_observed(
            selected_basis,
            entities.model(),
            self.current_entities().iter().copied().collect(),
            self.transfer_participants().iter().copied().collect(),
            selected_entities,
            self.current_relations().values().copied().collect(),
            self.selected_relations().values().copied().collect(),
        ))
    }
}
