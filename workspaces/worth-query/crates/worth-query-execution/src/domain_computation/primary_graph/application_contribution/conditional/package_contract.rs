//! One conditional's portable definition and its exact local field expectations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationConditionalPackageContract {
    domain_operation: WorthQueryPortableDomainOperationDefinition,
    operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
    target: WorthQueryConditionalNodeLocation,
    pub(super) fields: Vec<output_readiness_contract::typed::FieldExpectation>,
    pub(super) delivery_fields: std::collections::BTreeSet<
        worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    >,
}

impl WorthQueryApplicationConditionalPackageContract {
    pub fn new(
        domain_operation: WorthQueryPortableDomainOperationDefinition,
        operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
        node_identity: impl Into<String>,
    ) -> Self {
        Self {
            domain_operation,
            operation_binding,
            fields: Vec::new(),
            delivery_fields: std::collections::BTreeSet::new(),
            target: WorthQueryConditionalNodeLocation::Operation {
                node_identity: node_identity.into(),
            },
        }
    }

    /// Authors a workflow-stage location for this declaration's node. This is
    /// selection only; complete installed coverage and native binding validate it.
    pub fn in_workflow_stage(mut self, stage_identity: impl Into<String>) -> Self {
        self.target = WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity: stage_identity.into(),
            node_identity: self.target.node_identity().to_owned(),
        };
        self
    }

    pub(in super::super) fn domain_operation(
        &self,
    ) -> &WorthQueryPortableDomainOperationDefinition {
        &self.domain_operation
    }

    pub(in super::super) fn operation_binding(
        &self,
    ) -> &WorthQueryPortableApplicationConditionalOperationBinding {
        &self.operation_binding
    }

    pub(in super::super) fn node_identity(&self) -> &str {
        self.target.node_identity()
    }

    pub(in super::super) fn target(&self) -> &WorthQueryConditionalNodeLocation {
        &self.target
    }

    pub(in super::super) fn seal<Schema: ApplicationSchema>(
        &self,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryOutputReadinessContractDenial> {
        for field in &self.fields {
            field.seal(installed)?;
        }
        Ok(())
    }
}
