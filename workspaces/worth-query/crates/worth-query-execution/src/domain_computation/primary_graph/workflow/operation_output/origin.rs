//! Descriptive workflow occurrence retained by the operation that actually ran.
use crate::domain_computation::primary_graph::workflow::{
    instance::WorkflowOperationSettlementBasis, schema::WorthQueryWorkflowLayout,
};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
};
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowOperationOrigin {
    instance: EntityId,
    transition: String,
    operation_receipt_identity: [u8; 32],
    layout: WorthQueryWorkflowLayout,
    authority_fact_range: std::ops::Range<usize>,
}

impl WorkflowOperationOrigin {
    pub(in crate::domain_computation::primary_graph) fn from_settlement(
        basis: &WorkflowOperationSettlementBasis,
        layout: &WorthQueryWorkflowLayout,
        authority_fact_range: std::ops::Range<usize>,
        operation_receipt_identity: [u8; 32],
    ) -> Self {
        Self {
            instance: basis.instance(),
            transition: basis.identity().to_owned(),
            operation_receipt_identity,
            layout: layout.clone(),
            authority_fact_range,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn instance(&self) -> EntityId {
        self.instance
    }

    pub(in crate::domain_computation::primary_graph) fn transition(&self) -> &str {
        &self.transition
    }

    pub(in crate::domain_computation::primary_graph) fn operation_receipt_identity(
        &self,
    ) -> [u8; 32] {
        self.operation_receipt_identity
    }

    pub(in crate::domain_computation::primary_graph) fn semantic_facts(
        &self,
        facts: &FactPayload,
        control: StorageControl<'_, '_>,
    ) -> Result<FactPayload, StoreDenial> {
        semantic_facts(facts, self.authority_fact_range.clone(), control)
    }

    pub(in crate::domain_computation::primary_graph) fn layout(&self) -> &WorthQueryWorkflowLayout {
        &self.layout
    }
}

impl PartialEq for WorkflowOperationOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
            && self.transition == other.transition
            && self.operation_receipt_identity == other.operation_receipt_identity
            && self.authority_fact_range == other.authority_fact_range
            && std::ptr::eq(&*self.layout, &*other.layout)
    }
}
impl Eq for WorkflowOperationOrigin {}

fn semantic_facts(
    facts: &FactPayload,
    authority: std::ops::Range<usize>,
    control: StorageControl<'_, '_>,
) -> Result<FactPayload, StoreDenial> {
    if authority.start > authority.end || authority.end > facts.len() {
        return Err(StoreDenial::ConflictingBody);
    }
    if authority.is_empty() {
        return Ok(facts.clone());
    }
    facts
        .slice(0..authority.start)
        .append_completed(facts.slice(authority.end..facts.len()), control)
}

#[cfg(test)]
mod tests;
