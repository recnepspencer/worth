//! Detached complete Native meaning; identity visits use the run's one bijection.
use super::comparison::{HostDivergence, ProjectionPart};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use serde::{Deserialize, Serialize};
use worth_relational::facade::{history::RelationalCommitReceipt, transactions::*};
mod record_identity;
mod validation;
pub(crate) use record_identity::{correspond_receipt, correspond_record};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSemanticProjection {
    receipt: RelationalCommitReceipt,
    created_entities: Vec<RelationalCreatedEntityCorrespondence>,
    created_relations: Vec<RelationalCreatedRelationCorrespondence>,
    changed_records: Vec<RecordRef>,
    structural: RelationalCommitStructuralMeaning,
    schema: CommitSchemaSummary,
    history: CommitHistorySummary,
    change: CommitChangeSummary,
    aspects: CommitAspectSummary,
    publication: RelationalCommitPublicationMeaning,
    validation: Vec<RelationalInvariantExecutionMeaning>,
}
impl NativeSemanticProjection {
    pub fn observe(value: &RelationalCommitSemanticResult) -> Self {
        // Strict shape decoding fails if Native adds meaning this observer omits.
        serde_json::from_value(serde_json::to_value(value).expect("Native meaning serializes"))
            .expect("the complete Native semantic schema is projected")
    }
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        &self.receipt
    }
}
pub(crate) fn bad() -> HostDivergence {
    HostDivergence::Meaning {
        part: ProjectionPart::NativeSemantic,
        index: 0,
    }
}
pub fn compare(
    map: &mut DurableIdentityCorrespondence,
    home: &NativeSemanticProjection,
    reference: &NativeSemanticProjection,
) -> Result<(), HostDivergence> {
    let mut h = home.clone();
    if h.history.target_branch != h.receipt.branch_id.0
        || reference.history.target_branch != reference.receipt.branch_id.0
    {
        return Err(bad());
    }
    correspond_receipt(map, &mut h.receipt, &reference.receipt)?;
    h.history.target_branch = reference.history.target_branch.clone();
    if h.created_entities.len() != reference.created_entities.len()
        || h.created_relations.len() != reference.created_relations.len()
        || h.changed_records.len() != reference.changed_records.len()
    {
        return Err(bad());
    }
    for (h, r) in h
        .created_entities
        .iter_mut()
        .zip(&reference.created_entities)
    {
        map.admit(
            Id::NativePartition(h.key.partition),
            Id::NativePartition(r.key.partition),
        )?;
        h.key.partition = r.key.partition;
        if h.key != r.key {
            return Err(bad());
        }
        map.admit(Id::Entity(h.assigned), Id::Entity(r.assigned))?;
        h.assigned = r.assigned;
    }
    for (h, r) in h
        .created_relations
        .iter_mut()
        .zip(&reference.created_relations)
    {
        map.admit(
            Id::NativePartition(h.key.partition),
            Id::NativePartition(r.key.partition),
        )?;
        h.key.partition = r.key.partition;
        record_identity::correspond_endpoint(map, &mut h.key.source, &r.key.source)?;
        record_identity::correspond_endpoint(map, &mut h.key.target, &r.key.target)?;
        if h.key != r.key {
            return Err(bad());
        }
        map.admit(Id::Relation(h.assigned), Id::Relation(r.assigned))?;
        h.assigned = r.assigned;
    }
    for (h, r) in h.changed_records.iter_mut().zip(&reference.changed_records) {
        correspond_record(map, h, r)?;
    }
    if h.structural.touched_partitions.len() != reference.structural.touched_partitions.len() {
        return Err(bad());
    }
    for (h, r) in h
        .structural
        .touched_partitions
        .iter_mut()
        .zip(&reference.structural.touched_partitions)
    {
        map.admit(Id::NativePartition(*h), Id::NativePartition(*r))?;
        *h = *r;
    }
    validation::correspond(map, &mut h.validation, &reference.validation)?;
    // Every non-identity value, order, field and optional presence remains exact.
    if h != *reference {
        return Err(bad());
    }
    Ok(())
}
