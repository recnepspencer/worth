//! Check recorded original facts against the enclosing canonical publication.
use super::{
    RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCreatedEndpoint,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;

mod creations;

impl RelationalCommitSemanticResult {
    pub(crate) fn validate_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), Refusal> {
        if self.receipt != envelope.commit || self.history.target_branch != self.receipt.branch_id.0
        {
            return Err(Refusal::ReceiptMismatch);
        }
        let same_transition = match (&self.schema.transition, &envelope.schema_transition) {
            (None, None) => true,
            (Some(summary), Some(artifact)) => summary.matches_artifact(artifact),
            _ => false,
        };
        if !same_transition
            || self.schema.descriptor_semantics_version != envelope.descriptor_semantics_version
            || self.structural.descriptor_semantics_version != envelope.descriptor_semantics_version
        {
            return Err(Refusal::SchemaMismatch);
        }
        if self
            .changed_records
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| self.changed_records.binary_search(&patch.target).is_err())
            || self.change.changed_record_count != self.changed_records.len()
        {
            return Err(Refusal::ChangedRecordsMismatch);
        }
        if self.publication.patch_record_count != envelope.patch.authoritative_record_patches.len()
            || self.publication.lineage_event_count != envelope.lineage_events().len()
            || self.publication.merge_parent_count != self.receipt.parents.len().saturating_sub(1)
            || self.history.parent_count != self.receipt.parents.len()
            || self.history.merge_base_count != envelope.merge_base_commits.len()
            || self
                .structural
                .touched_partitions
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .aspects
                .touched_aspects
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(Refusal::PublicationMismatch);
        }
        self.validate_creations(envelope)?;
        // An original committed observation may name the predecessor version.
        // Its version is an execution fact, not the newly published version.
        if self
            .validation
            .iter()
            .any(|execution| !execution.is_consistent())
        {
            return Err(Refusal::ValidationMismatch);
        }
        Ok(())
    }

    fn validate_creations(&self, envelope: &CanonicalCommitEnvelope) -> Result<(), Refusal> {
        if self
            .created_entities
            .windows(2)
            .any(|pair| pair[0].key >= pair[1].key)
            || self
                .created_relations
                .windows(2)
                .any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(Refusal::NonCanonicalCorrespondence);
        }
        let patches = &envelope.patch.authoritative_record_patches;
        // A typed ordinal roster preserves FIRST matching Created patch even
        // for invalid duplicate targets; no private std node layout is assumed.
        let mut created_patches = Vec::with_capacity(patches.len());
        created_patches.extend(patches.iter().enumerate().filter_map(|(index, patch)| {
            (patch.structural_change == RecordStructuralChange::Created).then_some(index)
        }));
        created_patches.sort_unstable_by(|left, right| {
            patches[*left]
                .target
                .cmp(&patches[*right].target)
                .then(left.cmp(right))
        });
        let created_patch = |target: &RecordRef| {
            let slot = created_patches.partition_point(|index| &patches[*index].target < target);
            created_patches
                .get(slot)
                .map(|index| &patches[*index])
                .filter(|patch| &patch.target == target)
        };
        let count = self
            .created_entities
            .len()
            .checked_add(self.created_relations.len())
            .ok_or(Refusal::ConflictingCreateCorrespondence)?;
        let mut records = Vec::with_capacity(count);
        for created in &self.created_entities {
            if created.key.partition != created.assigned.partition_id {
                return Err(Refusal::CreatePartitionMismatch);
            }
            records.push(RecordRef::Entity(created.assigned));
        }
        let resolve = |reference: &RelationalCreatedEndpoint| match reference {
            RelationalCreatedEndpoint::Existing(id) => Some(*id),
            RelationalCreatedEndpoint::Created(key) => self
                .created_entities
                .binary_search_by(|created| created.key.cmp(key))
                .ok()
                .map(|index| self.created_entities[index].assigned),
        };
        for created in &self.created_relations {
            if created.key.partition != created.assigned.partition_id {
                return Err(Refusal::CreatePartitionMismatch);
            }
            let source = resolve(&created.key.source).ok_or(Refusal::CreatedEndpointMissing)?;
            let target = resolve(&created.key.target).ok_or(Refusal::CreatedEndpointMissing)?;
            records.push(RecordRef::Relation(created.assigned));
            let birth = created_patch(&RecordRef::Relation(created.assigned))
                .and_then(|patch| patch.relation_birth);
            if birth.is_none_or(|birth| {
                birth.kind != created.key.kind || birth.source != source || birth.target != target
            }) {
                return Err(Refusal::ConflictingCreateCorrespondence);
            }
        }
        records.sort_unstable();
        if records.windows(2).any(|pair| pair[0] == pair[1])
            || records.iter().any(|record| created_patch(record).is_none())
        {
            return Err(Refusal::ConflictingCreateCorrespondence);
        }
        creations::validate_original_keys(self, envelope)?;
        Ok(())
    }
}
