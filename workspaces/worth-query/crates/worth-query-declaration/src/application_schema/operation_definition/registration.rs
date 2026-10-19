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
        if let Some(recipe) = definition.external_emission_recipe {
            self.member_provenance.register_body_external_effect(recipe);
        }
        self.push_member_in_place(ApplicationSchemaMember::Operation {
            operation: operation.clone(),
            input_type: definition.input_type,
        });
        if let Some(body) = definition.body {
            let mut reads = body
                .scope_fields()
                .iter()
                .map(|step| step.decision_read())
                .collect::<std::collections::BTreeSet<_>>();
            if let Some(scope) = body
                .explicit_scope_entity()
                .filter(|_| body.scope_fields().is_empty())
            {
                reads.insert(
                    super::super::ApplicationOperationDecisionReadTarget::Entity {
                        entity: scope.to_owned(),
                    },
                );
            }
            reads.extend(
                body.outputs()
                    .iter()
                    .flat_map(|output| output.target.decision_reads()),
            );
            reads.extend(
                body.observation_operands()
                    .flat_map(|operand| operand.source().decision_reads()),
            );
            reads.extend(
                body.retirements()
                    .iter()
                    .flat_map(|rule| rule.decision_reads()),
            );
            reads.extend(
                body.path_input_expectations()
                    .iter()
                    .flat_map(|guard| guard.source.decision_reads()),
            );
            reads.extend(
                body.path_value_assignments()
                    .iter()
                    .flat_map(|step| step.target.decision_reads()),
            );
            reads.extend(
                body.reference_inputs()
                    .iter()
                    .flat_map(|reference| reference.decision_reads()),
            );
            for creation in body
                .creation()
                .into_iter()
                .flat_map(|c| c.initialization_sites())
            {
                if creation.reuses_current_if_unchanged() {
                    reads.extend(creation.fields().iter().map(|field| {
                        super::super::ApplicationOperationDecisionReadTarget::Field {
                            entity: field.field().entity().to_owned(),
                            aspect: field.field().aspect().to_owned(),
                            field: field.field().field().to_owned(),
                        }
                    }));
                }
                reads.extend(
                    creation
                        .links()
                        .iter()
                        .flat_map(|link| link.replacement_reads()),
                );
                reads.extend(
                    creation
                        .fields()
                        .iter()
                        .filter_map(|step| step.source_read()),
                );
                reads.extend(
                    creation
                        .fields()
                        .iter()
                        .filter_map(|step| step.related_source())
                        .flat_map(|source| source.decision_reads()),
                );
                reads.extend(
                    creation
                        .fields()
                        .iter()
                        .filter_map(|step| step.path_source())
                        .flat_map(|source| source.decision_reads()),
                );
            }
            reads.extend(
                body.creation()
                    .into_iter()
                    .flat_map(|c| c.initialization_sites())
                    .flat_map(|c| c.path_links())
                    .flat_map(|p| p.decision_reads()),
            );
            for target in reads {
                self.push_member_in_place(ApplicationSchemaMember::OperationDecisionRead {
                    operation: operation.clone(),
                    target,
                });
            }
            let mut programs = body
                .scope_fields()
                .iter()
                .filter_map(|step| step.write_target())
                .chain(
                    body.path_value_assignments()
                        .iter()
                        .map(|step| step.write_target()),
                )
                .collect::<std::collections::BTreeSet<_>>();
            for site in body
                .creation()
                .into_iter()
                .flat_map(|root| root.initialization_sites())
            {
                programs.insert(super::super::ApplicationOperationProgramTarget::Create {
                    entity: site.entity().to_owned(),
                });
                programs.extend(site.fields().iter().map(|field| field.write_target()));
                programs.extend(site.path_links().iter().map(|p| p.link.program_target()));
                programs.extend(site.links().iter().flat_map(|link| {
                    std::iter::once(link.program_target())
                        .chain(link.replacement_target())
                        .chain(link.retirement_target())
                }));
            }
            programs.extend(
                body.retirements()
                    .iter()
                    .flat_map(|rule| rule.program_targets()),
            );
            programs.extend(
                body.reference_inputs()
                    .iter()
                    .flat_map(|reference| reference.program_targets()),
            );
            if let Some(emission) = body.external_emission() {
                programs.insert(super::super::ApplicationOperationProgramTarget::Emit {
                    effect: emission.effect.clone(),
                });
            }
            for target in programs {
                self.push_member_in_place(ApplicationSchemaMember::OperationProgram {
                    operation: operation.clone(),
                    target,
                });
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
