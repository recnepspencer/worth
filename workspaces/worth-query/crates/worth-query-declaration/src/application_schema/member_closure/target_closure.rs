use super::ClosureIndex;
use crate::application_schema::{
    ApplicationMutationPreconditionTarget, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget, ApplicationSchemaMember,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

impl ClosureIndex<'_> {
    pub(super) fn external_effect_dependencies_exist(
        &self,
        operation: &str,
        effect: &str,
        payload_type: &WorthQueryPortableTypeIdentity,
    ) -> bool {
        self.operations.contains(operation)
            && self.members.iter().any(|member| {
                matches!(
                    member,
                    ApplicationSchemaMember::Effect {
                        effect: installed,
                        payload_type: installed_payload,
                    } if installed == effect && installed_payload == payload_type
                )
            })
            && self.members.iter().any(|member| {
                matches!(
                    member,
                    ApplicationSchemaMember::OperationProgram {
                        operation: installed,
                        target: ApplicationOperationProgramTarget::Emit { effect: emitted },
                    } if installed == operation && emitted == effect
                )
            })
    }

    pub(super) fn program_target_exists(
        &self,
        operation: &str,
        target: &ApplicationOperationProgramTarget,
    ) -> bool {
        match target {
            ApplicationOperationProgramTarget::Create { entity }
            | ApplicationOperationProgramTarget::Delete { entity } => {
                self.entities.contains(entity.as_str())
            }
            ApplicationOperationProgramTarget::Write {
                entity,
                aspect,
                field,
            } => self
                .fields
                .contains(&(entity.as_str(), aspect.as_str(), field.as_str())),
            ApplicationOperationProgramTarget::Link { relation, from, to }
            | ApplicationOperationProgramTarget::Unlink { relation, from, to } => self
                .relations
                .contains(&(relation.as_str(), from.as_str(), to.as_str())),
            ApplicationOperationProgramTarget::Emit { effect } => {
                self.effects.contains(effect.as_str())
            }
            ApplicationOperationProgramTarget::InitializeRelationField { relation, from, to, aspect, field } => self.members.iter().any(|member| matches!(member,
                ApplicationSchemaMember::RelationAspect { definition }
                if { let parts = definition.parts(); parts.relation == *relation && parts.from == *from && parts.to == *to && parts.aspect == *aspect && parts.fields.iter().any(|value| value.field == *field) }
            )) && self.members.iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target: ApplicationOperationProgramTarget::Link { relation: installed, from: source, to: destination } } if candidate == operation && installed == relation && source == from && destination == to)),
        }
    }

    pub(super) fn decision_read_target_exists(
        &self,
        target: &ApplicationOperationDecisionReadTarget,
    ) -> bool {
        match target {
            ApplicationOperationDecisionReadTarget::Entity { entity } => {
                self.entities.contains(entity.as_str())
            }
            ApplicationOperationDecisionReadTarget::Field {
                entity,
                aspect,
                field,
            } => self
                .fields
                .contains(&(entity.as_str(), aspect.as_str(), field.as_str())),
            ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => self
                .relations
                .contains(&(relation.as_str(), from.as_str(), to.as_str())),
        }
    }

    pub(super) fn precondition_target_is_decision_read(
        &self,
        operation: &str,
        target: &ApplicationMutationPreconditionTarget,
    ) -> bool {
        self.members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::OperationDecisionRead {
                    operation: candidate_operation,
                    target: ApplicationOperationDecisionReadTarget::Field {
                        entity,
                        aspect,
                        field,
                    },
                } if candidate_operation == operation
                    && entity == target.entity()
                    && aspect == target.aspect()
                    && field == target.field_name()
            )
        })
    }
}
