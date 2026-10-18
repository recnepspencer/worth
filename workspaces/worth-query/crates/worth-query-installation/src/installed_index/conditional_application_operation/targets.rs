//! Complete installed conditional target locations for schema composition.
use super::*;
use crate::domain_operation::{
    WorthQueryConditionalNodeLocation, WorthQueryOperationWorkflowContract,
};

impl WorthQueryInstalledPackageIndex {
    /// Enumerates every operation and workflow-stage conditional target of
    /// this installed schema. These are read-only declarations, not execution
    /// authority; ordinary binding and runtime admission remain mandatory.
    pub fn installed_conditional_targets_for_schema(
        &self,
        owner: &str,
        schema: &str,
    ) -> Vec<(
        &WorthQueryPortableApplicationConditionalOperationBinding,
        WorthQueryConditionalNodeLocation,
    )> {
        let mut targets = Vec::new();
        for ((binding_owner, binding_schema, _), binding) in
            &self.conditional_application_operations
        {
            if binding_owner != owner || binding_schema != schema {
                continue;
            }
            let operation = self
                .domain_operations
                .get(&(owner.to_owned(), binding.domain_operation_slot().to_owned()))
                .expect("an admitted conditional binding retains its domain operation");
            let semantics = operation.definition().semantics();
            targets.extend(semantics.conditional_nodes.iter().map(|node| {
                (
                    binding,
                    WorthQueryConditionalNodeLocation::Operation {
                        node_identity: node.identity().to_owned(),
                    },
                )
            }));
            if let WorthQueryOperationWorkflowContract::Declared(workflow) = &semantics.workflow {
                for stage in workflow.stages() {
                    targets.extend(stage.semantics().conditional_nodes.iter().map(|node| {
                        (
                            binding,
                            WorthQueryConditionalNodeLocation::WorkflowStage {
                                stage_identity: stage.identity().to_owned(),
                                node_identity: node.identity().to_owned(),
                            },
                        )
                    }));
                }
            }
        }
        targets
    }
}
