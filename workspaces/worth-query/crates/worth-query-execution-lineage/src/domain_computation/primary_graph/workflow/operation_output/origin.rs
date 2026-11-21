//! Pure workflow operation origin record.
use worth_query_execution_layout::facade::workflow_schema::WorthQueryWorkflowLayout;
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug)]
pub struct WorkflowOperationOrigin {
    pub instance: EntityId,
    pub transition: String,
    pub operation_receipt_identity: [u8; 32],
    pub layout: WorthQueryWorkflowLayout,
    pub authority_fact_range: std::ops::Range<usize>,
}

impl WorkflowOperationOrigin {
    pub fn instance(&self) -> EntityId {
        self.instance
    }

    pub fn transition(&self) -> &str {
        &self.transition
    }

    pub fn operation_receipt_identity(&self) -> &[u8; 32] {
        &self.operation_receipt_identity
    }

    pub fn layout(&self) -> &WorthQueryWorkflowLayout {
        &self.layout
    }

    pub fn authority_fact_range(&self) -> std::ops::Range<usize> {
        self.authority_fact_range.clone()
    }

    pub fn semantic_facts(
        &self,
        facts: &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    > {
        let authority = self.authority_fact_range.clone();
        if authority.start > authority.end || authority.end > facts.len() {
            return Err(
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::ConflictingBody,
            );
        }
        if authority.is_empty() {
            return Ok(facts.clone());
        }
        facts
            .slice(0..authority.start)
            .append_completed(facts.slice(authority.end..facts.len()), control)
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
