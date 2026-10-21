use crate::application_operation::{
    WorthQueryApplicationConditionalOperationBinding,
    WorthQueryApplicationOperationInstallationDenialKind,
    WorthQueryConditionalApplicationOperationDenial,
    WorthQueryConditionalApplicationOperationDenialKind,
    WorthQueryInstalledApplicationConditionalOperation, WorthQueryInstalledApplicationOperation,
    WorthQueryPortableApplicationConditionalOperationBinding,
};
use crate::installed_domain_operation::WorthQueryInstalledDomainOperationAuthority;

use super::WorthQueryInstalledPackageIndex;
mod targets;

impl WorthQueryInstalledPackageIndex {
    pub fn bind_conditional_application_operation<Schema, ApplicationOperation, Input, D, O, F>(
        &self,
        application_operation: WorthQueryInstalledApplicationOperation<
            Schema,
            ApplicationOperation,
            Input,
        >,
        binding: &WorthQueryApplicationConditionalOperationBinding<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
        >,
    ) -> Result<
        WorthQueryInstalledApplicationConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
        >,
        WorthQueryConditionalApplicationOperationDenial,
    > {
        validate_application_operation_currentness(self, &application_operation)?;
        let installed_binding = resolve_installed_binding(self, &application_operation, binding)?;
        let domain_operation = resolve_bound_domain_operation(
            self,
            application_operation.owner(),
            &installed_binding,
        )?;
        Ok(WorthQueryInstalledApplicationConditionalOperation::new(
            application_operation,
            domain_operation,
            installed_binding,
        ))
    }

    pub fn validate_conditional_application_node<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    >(
        &self,
        node: &crate::application_operation::WorthQueryInstalledApplicationConditionalNode<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            N,
        >,
    ) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
        validate_application_operation_currentness(self, node.operation().application_operation())?;
        validate_node_binding(self, node)?;
        validate_node_declaration(self, node)
    }

    pub fn installed_conditional_node_count_for_schema(&self, owner: &str, schema: &str) -> usize {
        self.conditional_application_operations
            .iter()
            .filter(|((binding_owner, binding_schema, _), _)| {
                binding_owner == owner && binding_schema == schema
            })
            .filter_map(|(_, binding)| {
                self.domain_operations.get(&(
                    owner.to_string(),
                    binding.domain_operation_slot().to_string(),
                ))
            })
            .map(|operation| conditional_node_count(operation.definition().semantics()))
            .sum()
    }
}

fn validate_node_binding<Schema, ApplicationOperation, Input, D, O, F, N>(
    index: &WorthQueryInstalledPackageIndex,
    node: &crate::application_operation::WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    >,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    let operation = node.operation();
    let application = operation.application_operation();
    let key = (
        application.owner().to_string(),
        application.schema_name().to_string(),
        application.operation().to_string(),
    );
    let installed = index
        .conditional_application_operations
        .get(&key)
        .ok_or_else(|| {
            conditional_denial(
                WorthQueryConditionalApplicationOperationDenialKind::BindingNotInstalled,
                application.operation(),
            )
        })?;
    if installed != operation.binding() {
        return Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::BindingMeaningChanged,
            application.operation(),
        ));
    }
    Ok(())
}

fn validate_node_declaration<Schema, ApplicationOperation, Input, D, O, F, N>(
    index: &WorthQueryInstalledPackageIndex,
    node: &crate::application_operation::WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
    >,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    let operation = node.operation();
    let installed_domain = index
        .domain_operation(
            operation.application_operation().owner(),
            operation.binding().domain_operation_slot(),
        )
        .map_err(|_| {
            conditional_denial(
                WorthQueryConditionalApplicationOperationDenialKind::DomainOperationNotInstalled,
                operation.binding().domain_operation_slot(),
            )
        })?;
    if installed_domain.definition().canonical_identity()
        != operation.binding().domain_operation_canonical_identity()
    {
        return Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::DomainOperationChanged,
            operation.binding().domain_operation_slot(),
        ));
    }
    let declaration = installed_domain
        .conditional_node_declaration(node.location())
        .map_err(|_| {
            conditional_denial(
                WorthQueryConditionalApplicationOperationDenialKind::NodeNotDeclared,
                node.location().node_identity(),
            )
        })?;
    if declaration != *node.declaration() {
        return Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::NodeMeaningChanged,
            node.location().node_identity(),
        ));
    }
    Ok(())
}

fn conditional_node_count(
    semantics: &crate::domain_operation::WorthQueryDomainOperationSemanticClosure,
) -> usize {
    let operation_nodes = semantics.conditional_nodes.len();
    let workflow_nodes = match &semantics.workflow {
        crate::domain_operation::WorthQueryOperationWorkflowContract::NotRequired => 0,
        crate::domain_operation::WorthQueryOperationWorkflowContract::Declared(workflow) => {
            workflow
                .stages()
                .iter()
                .map(|stage| stage.semantics().conditional_nodes.len())
                .sum()
        }
    };
    operation_nodes + workflow_nodes
}

fn conditional_denial(
    kind: WorthQueryConditionalApplicationOperationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryConditionalApplicationOperationDenial {
    WorthQueryConditionalApplicationOperationDenial::new(kind, subject)
}

fn validate_application_operation_currentness<Schema, ApplicationOperation, Input>(
    index: &WorthQueryInstalledPackageIndex,
    application_operation: &WorthQueryInstalledApplicationOperation<
        Schema,
        ApplicationOperation,
        Input,
    >,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    index
        .validate_application_operation(application_operation)
        .map_err(|denial| {
            let kind = match denial.kind() {
                WorthQueryApplicationOperationInstallationDenialKind::ForeignRuntime => {
                    WorthQueryConditionalApplicationOperationDenialKind::ForeignRuntime
                }
                WorthQueryApplicationOperationInstallationDenialKind::StaleGeneration => {
                    WorthQueryConditionalApplicationOperationDenialKind::StaleGeneration
                }
                _ => {
                    WorthQueryConditionalApplicationOperationDenialKind::ApplicationOperationChanged
                }
            };
            WorthQueryConditionalApplicationOperationDenial::new(kind, denial.operation())
        })
}

fn resolve_installed_binding<Schema, ApplicationOperation, Input, D, O, F>(
    index: &WorthQueryInstalledPackageIndex,
    application_operation: &WorthQueryInstalledApplicationOperation<
        Schema,
        ApplicationOperation,
        Input,
    >,
    binding: &WorthQueryApplicationConditionalOperationBinding<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
    >,
) -> Result<
    WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryConditionalApplicationOperationDenial,
> {
    let key = (
        application_operation.owner().to_string(),
        application_operation.schema_name().to_string(),
        application_operation.operation().to_string(),
    );
    let installed = index
        .conditional_application_operations
        .get(&key)
        .ok_or_else(|| {
            WorthQueryConditionalApplicationOperationDenial::new(
                WorthQueryConditionalApplicationOperationDenialKind::BindingNotInstalled,
                application_operation.operation(),
            )
        })?;
    if installed != binding.portable() {
        return Err(WorthQueryConditionalApplicationOperationDenial::new(
            WorthQueryConditionalApplicationOperationDenialKind::BindingMeaningChanged,
            application_operation.operation(),
        ));
    }
    Ok(installed.clone())
}

fn resolve_bound_domain_operation(
    index: &WorthQueryInstalledPackageIndex,
    owner: &str,
    binding: &WorthQueryPortableApplicationConditionalOperationBinding,
) -> Result<
    WorthQueryInstalledDomainOperationAuthority,
    WorthQueryConditionalApplicationOperationDenial,
> {
    let domain_operation = index
        .domain_operation(owner, binding.domain_operation_slot())
        .map_err(|_| {
            WorthQueryConditionalApplicationOperationDenial::new(
                WorthQueryConditionalApplicationOperationDenialKind::DomainOperationNotInstalled,
                binding.domain_operation_slot(),
            )
        })?;
    if domain_operation.definition().canonical_identity()
        != binding.domain_operation_canonical_identity()
    {
        return Err(WorthQueryConditionalApplicationOperationDenial::new(
            WorthQueryConditionalApplicationOperationDenialKind::DomainOperationChanged,
            binding.domain_operation_slot(),
        ));
    }
    Ok(domain_operation)
}
