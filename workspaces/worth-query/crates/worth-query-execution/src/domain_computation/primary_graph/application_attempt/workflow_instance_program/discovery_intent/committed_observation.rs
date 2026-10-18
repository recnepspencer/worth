//! Closed passive comparison with the actual performed native start artifact.
use super::WorthQueryWorkflowInstanceDiscoveryIntent;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::PerformedWorkflowInstanceStart;
use crate::domain_computation::primary_graph::schema_layout::planned_field_locator;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::publication::RecordStructuralChange;

impl<Schema> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    /// Compares original locator data with actual created fields and commit binding.
    /// This observes immutable evidence only and creates no reference or permission.
    #[doc(hidden)]
    pub fn matches_performed_start_for_test(
        &self,
        performed: &PerformedWorkflowInstanceStart,
    ) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
        let receipt = performed.receipt();
        let changes = receipt.committed_changes();
        let entity = performed.instance().entity_id();
        if !changes.entity_changes().any(|(candidate, change)| {
            candidate == entity && change == RecordStructuralChange::Created
        }) {
            return Ok(false);
        }
        let fields = [
            "identity",
            "subject-partition",
            "subject-slot",
            "subject-generation",
        ]
        .map(|field| planned_field_locator("workflow-instance", field));
        let [identity, partition, slot, generation] = fields;
        let fields = [identity?, partition?, slot?, generation?];
        let values = changes.committed_field_values(entity, &fields.each_ref());
        let subject = self.subject_entity_id();
        let expected = vec![
            AspectValue::String(InternedString::Raw(self.lookup_identity().to_owned())),
            AspectValue::UInt64(subject.partition_value_u64()),
            AspectValue::UInt64(subject.local_slot_value()),
            AspectValue::UInt64(u64::from(subject.generation_value())),
        ];
        let scope = receipt.principal_scope().scope();
        let binding = receipt.idempotency_binding();
        Ok(values == Some(expected)
            && scope.partition_id() == subject.partition_value()
            && scope.local_slot() == subject.local_slot_value()
            && scope.generation() == subject.generation_value()
            // Rebinding the recorded slot is equal only when that actual slot
            // already contains the original digest (an absent slot also fails).
            && binding == binding.bind_workflow_instance(self.intent_identity()))
    }
}
