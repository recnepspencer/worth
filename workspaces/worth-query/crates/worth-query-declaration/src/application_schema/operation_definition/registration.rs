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
        if let Some(body) = definition.body {
            let reads = body
                .scope_fields()
                .iter()
                .map(|step| step.decision_read())
                .collect::<std::collections::BTreeSet<_>>();
            for target in reads {
                self.push_member_in_place(ApplicationSchemaMember::OperationDecisionRead {
                    operation: operation.clone(),
                    target,
                });
            }
            for target in body
                .scope_fields()
                .iter()
                .filter_map(|step| step.write_target())
            {
                self.push_member_in_place(ApplicationSchemaMember::OperationProgram {
                    operation: operation.clone(),
                    target,
                });
            }
            if let Some(creation) = body.creation() {
                self.push_member_in_place(ApplicationSchemaMember::OperationProgram {
                    operation: operation.clone(),
                    target: super::super::ApplicationOperationProgramTarget::Create {
                        entity: creation.entity().to_owned(),
                    },
                });
                for field in creation.fields() {
                    self.push_member_in_place(ApplicationSchemaMember::OperationProgram {
                        operation: operation.clone(),
                        target: field.write_target(),
                    });
                }
            }
            self.push_member_in_place(ApplicationSchemaMember::OperationBody {
                operation: operation.clone(),
                body,
            });
        }
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
