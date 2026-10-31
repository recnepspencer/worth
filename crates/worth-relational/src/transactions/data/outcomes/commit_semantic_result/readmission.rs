//! Check recorded original facts against the enclosing canonical publication.
use super::{
    RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCreatedEndpoint,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use std::collections::{BTreeMap, BTreeSet};

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
        let schema = crate::transactions::data::CommitSchemaSummary {
            transition: envelope
                .schema_transition
                .as_ref()
                .map(crate::schema::data::SchemaTransitionSummary::from_artifact),
            descriptor_semantics_version: envelope.descriptor_semantics_version,
        };
        if self.schema != schema
            || self.structural.descriptor_semantics_version != envelope.descriptor_semantics_version
        {
            return Err(Refusal::SchemaMismatch);
        }
        let actual_records = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .map(|patch| patch.target.clone())
            .collect::<BTreeSet<_>>();
        if self
            .changed_records
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || !actual_records.is_subset(
                &self
                    .changed_records
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
            )
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
        let mut entities = BTreeMap::new();
        let mut records = BTreeSet::new();
        for created in &self.created_entities {
            if created.key.partition != created.assigned.partition_id {
                return Err(Refusal::CreatePartitionMismatch);
            }
            if !records.insert(RecordRef::Entity(created.assigned)) {
                return Err(Refusal::ConflictingCreateCorrespondence);
            }
            entities.insert(&created.key, created.assigned);
        }
        let resolve = |reference: &RelationalCreatedEndpoint| match reference {
            RelationalCreatedEndpoint::Existing(id) => Some(*id),
            RelationalCreatedEndpoint::Created(key) => entities.get(key).copied(),
        };
        for created in &self.created_relations {
            if created.key.partition != created.assigned.partition_id {
                return Err(Refusal::CreatePartitionMismatch);
            }
            let source = resolve(&created.key.source).ok_or(Refusal::CreatedEndpointMissing)?;
            let target = resolve(&created.key.target).ok_or(Refusal::CreatedEndpointMissing)?;
            if !records.insert(RecordRef::Relation(created.assigned)) {
                return Err(Refusal::ConflictingCreateCorrespondence);
            }
            let birth = envelope
                .patch
                .authoritative_record_patches
                .iter()
                .find(|patch| {
                    patch.target == RecordRef::Relation(created.assigned)
                        && patch.structural_change == RecordStructuralChange::Created
                })
                .and_then(|patch| patch.relation_birth);
            if birth.is_none_or(|birth| {
                birth.kind != created.key.kind || birth.source != source || birth.target != target
            }) {
                return Err(Refusal::ConflictingCreateCorrespondence);
            }
        }
        let actual = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .filter(|patch| patch.structural_change == RecordStructuralChange::Created)
            .map(|patch| patch.target.clone())
            .collect::<BTreeSet<_>>();
        if !records.is_subset(&actual) {
            return Err(Refusal::ConflictingCreateCorrespondence);
        }
        creations::validate_original_keys(self, envelope)?;
        Ok(())
    }
}
