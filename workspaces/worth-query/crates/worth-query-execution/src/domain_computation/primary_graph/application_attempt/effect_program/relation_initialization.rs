use super::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_installation::facade::{
    ApplicationOperationProgramTarget as Target, ApplicationRelationRef, OperationLinks,
    WorthQueryInstalledApplicationValueBindingCatalog,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn link_for_initialization<
        Relation,
        From,
        To,
    >(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<
        WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationLinks<Operation>,
    {
        let position = self.link_at_target_index(
            &Target::Link {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
            },
            key,
            from,
            to,
        )?;
        original.consult().map_err(|cause| {
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause.into(),relation.name())
        })?;
        Ok(WorthQueryApplicationCreatedRelation {
            relation,
            created_effect: position,
            program: Arc::clone(&self.program),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn initialize_relation_field<
        Relation,
        From,
        To,
        Field,
    >(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        field: Field,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Operation>,
        Field: worth_query_declaration::facade::application_schema::fragment::RelationSelection<
            Entity = Relation,
            From = From,
            To = To,
        >,
    {
        self.native_effects().initialize_relation_field(
            original, bindings, super::core::CreatedRelation {
                name: target.relation.name(), from: target.relation.from(), to: target.relation.to(),
                program: &target.program, created_effect: target.created_effect,
            }, field.identity(), (field.from(), field.to()),
            worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding::matches_binding::<Field::Binding>,
            value.into_foundational_value(),
        )
    }
}
