use super::super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use super::ApplicationOperationDefinition;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Registers one completed operation definition as an atomic semantic unit.
    pub fn operation<Operation, Input>(
        mut self,
        definition: ApplicationOperationDefinition<Schema, Operation, Input>,
    ) -> Self
    where
        Operation: 'static,
        Input: 'static,
    {
        let operation = definition.operation.to_string();
        self.member_provenance
            .register_operation::<Operation, Input>(
                definition.operation,
                definition.input_type.clone(),
            );
        self.push_member_in_place(ApplicationSchemaMember::Operation {
            operation: operation.clone(),
            input_type: definition.input_type,
        });
        if let Some(external_effect) = definition.external_effect {
            if let Some(inbound) = external_effect.inbound {
                self.push_member_in_place(ApplicationSchemaMember::OperationInboundOccurrence {
                    operation: operation.clone(),
                    effect: external_effect.effect.clone(),
                    protocol: inbound.protocol,
                    source_identity: inbound.source_identity,
                    limits: inbound.limits,
                });
            }
            self.push_member_in_place(ApplicationSchemaMember::OperationExternalEffect {
                operation: operation.clone(),
                effect: external_effect.effect,
                rust_payload_type: external_effect.rust_payload_type,
                protocol: external_effect.protocol,
                maximum_payload_bytes: external_effect.maximum_payload_bytes,
                correlation_family: external_effect.correlation_family,
            });
        }
        if let Some(contract) = definition.aftermath {
            self.push_member_in_place(ApplicationSchemaMember::OperationAftermath {
                operation,
                contract,
            });
        }
        self
    }
}
