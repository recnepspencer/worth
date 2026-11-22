//! One conditional's portable definition and its exact local field expectations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationConditionalPackageContract {
    domain_operation: WorthQueryPortableDomainOperationDefinition,
    operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
    target: WorthQueryConditionalNodeLocation,
    pub(crate) fields: Vec<super::super::native_field_contract::FieldExpectation>,
    pub(crate) delivery_fields: std::collections::BTreeSet<
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

    pub fn domain_operation(&self) -> &WorthQueryPortableDomainOperationDefinition {
        &self.domain_operation
    }

    pub fn operation_binding(&self) -> &WorthQueryPortableApplicationConditionalOperationBinding {
        &self.operation_binding
    }

    pub fn node_identity(&self) -> &str {
        self.target.node_identity()
    }

    pub fn target(&self) -> &WorthQueryConditionalNodeLocation {
        &self.target
    }

    pub fn seal<Schema: ApplicationSchema>(
        &self,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryOutputReadinessContractDenial> {
        for field in &self.fields {
            field.seal(installed)?;
        }
        Ok(())
    }
}

mod court_doors_for_test;

/// Crate-boundary internal port for
/// `WorthQueryApplicationConditionalPackageContract` field expectations.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ConditionalPackageContractInternalPort {
    fn fields_ref(&self) -> &[super::super::native_field_contract::FieldExpectation];
    fn fields_mut(&mut self) -> &mut Vec<super::super::native_field_contract::FieldExpectation>;
    fn delivery_fields_ref(
        &self,
    ) -> &std::collections::BTreeSet<
        worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    >;
    fn delivery_fields_mut(
        &mut self,
    ) -> &mut std::collections::BTreeSet<
        worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    >;
}

impl ConditionalPackageContractInternalPort for WorthQueryApplicationConditionalPackageContract {
    fn fields_ref(&self) -> &[super::super::native_field_contract::FieldExpectation] {
        &self.fields
    }

    fn fields_mut(&mut self) -> &mut Vec<super::super::native_field_contract::FieldExpectation> {
        &mut self.fields
    }

    fn delivery_fields_ref(
        &self,
    ) -> &std::collections::BTreeSet<
        worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    > {
        &self.delivery_fields
    }

    fn delivery_fields_mut(
        &mut self,
    ) -> &mut std::collections::BTreeSet<
        worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    > {
        &mut self.delivery_fields
    }
}
