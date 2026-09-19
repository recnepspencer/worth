use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationConditionalNode,
};

use super::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use crate::domain_computation::primary_graph::application_contribution::{
    TypedPendingOutputReadiness, WorthQueryApplicationProducerBinding,
};

impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn bind_output_readiness<Producer, ApplicationOperation, Input, D, O, F, Node>(
        &mut self,
        node: WorthQueryInstalledApplicationConditionalNode<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
        >,
        delivery_dependency_ordinal: usize,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
        ApplicationOperation: 'static,
        Input: 'static,
        D: 'static,
        O: 'static,
        F: 'static,
        Node: 'static,
    {
        let scope = self.application_binding_scope.as_ref().ok_or_else(|| {
            denial("output readiness must be bound inside an application contribution")
        })?;
        validate_output_readiness_authority(
            &scope.required_producers,
            Producer::IDENTITY,
            self.output_producers.contains_producer::<Producer>(),
            std::any::TypeId::of::<ApplicationOperation>(),
            std::any::TypeId::of::<
                <Producer::Operation as ApplicationMutationBinding<Schema>>::Operation,
            >(),
        )?;
        if delivery_dependency_ordinal >= node.declaration().dependencies().len() {
            return Err(denial(format!(
                "output readiness route `{}` has an invalid delivery dependency ordinal",
                Producer::IDENTITY,
            )));
        }
        if node.operation().binding() != &scope.binding
            || node.location().node_identity() != scope.node_identity
        {
            return Err(denial(scope.node_identity.clone()));
        }
        self.output_readiness
            .push(Box::new(TypedPendingOutputReadiness::new(
                Producer::IDENTITY,
                delivery_dependency_ordinal,
                node,
            )));
        Ok(())
    }
}

fn denial(subject: impl Into<String>) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
        subject,
    )
}

fn validate_output_readiness_authority(
    required_producers: &[String],
    producer_identity: &'static str,
    installed_producer_matches: bool,
    application_operation: std::any::TypeId,
    producer_application_operation: std::any::TypeId,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    if required_producers != [producer_identity] {
        return Err(denial(format!(
            "output readiness route `{producer_identity}` does not declare its exact producer dependency"
        )));
    }
    if !installed_producer_matches {
        return Err(denial(format!(
            "output readiness route `{producer_identity}` does not match the installed producer type and meaning"
        )));
    }
    if application_operation != producer_application_operation {
        return Err(denial(format!(
            "output readiness route `{producer_identity}` does not match the producer application operation"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::validate_output_readiness_authority;

    #[test]
    fn output_readiness_authority_rejects_another_application_operation() {
        let denial = validate_output_readiness_authority(
            &["producer".into()],
            "producer",
            true,
            TypeId::of::<DeclaredOperation>(),
            TypeId::of::<ProducerOperation>(),
        )
        .unwrap_err();

        assert!(denial
            .subject()
            .contains("does not match the producer application operation"));
    }

    #[test]
    fn output_readiness_authority_rejects_a_substituted_producer_type() {
        let denial = validate_output_readiness_authority(
            &["producer".into()],
            "producer",
            false,
            TypeId::of::<ProducerOperation>(),
            TypeId::of::<ProducerOperation>(),
        )
        .unwrap_err();

        assert!(denial
            .subject()
            .contains("does not match the installed producer type and meaning"));
    }

    struct DeclaredOperation;
    struct ProducerOperation;
}
