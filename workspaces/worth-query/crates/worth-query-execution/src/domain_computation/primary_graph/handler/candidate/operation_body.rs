//! Execution-only native body write into the existing candidate owner.
use super::OutputRoleUse;
use super::{
    ApplicationMutationBinding, ApplicationSchema, CandidateWriter,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    CandidateWriter<'_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_creation_unlink<Entity>(
        &mut self,
        scope: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        predecessor: &WorthQueryApplicationEffectEntity<Schema, ()>,
        link: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLink,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let target = link
            .replacement_target()
            .expect("installed replacement posture");
        self.candidate.unlink_at_target(&target, scope, predecessor)
    }
    pub(in crate::domain_computation::primary_graph) fn body_signing_slot(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, ()>,
        spec: &worth_query_declaration::facade::application_schema::ApplicationOperationBodySigningSpec,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.reserve_body_signing_slot(target, spec)
    }

    pub(in crate::domain_computation::primary_graph) fn body_creation_link<Entity>(
        &mut self,
        scope: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        created: &WorthQueryApplicationEffectEntity<Schema, ()>,
        link: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLink,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        use worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkDirection as Direction;
        match link.direction {
            Direction::FromScope => self.candidate.link_at_target(
                &link.program_target(),
                link.key.as_str(),
                scope,
                created,
            ),
            Direction::ToScope => self.candidate.link_at_target(
                &link.program_target(),
                link.key.as_str(),
                created,
                scope,
            ),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn body_output<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        role: OutputRoleUse,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.bind_output(role, target)
    }
    pub(in crate::domain_computation::primary_graph) fn body_input_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.write_body_input_field(target, field, value)
    }
    pub(in crate::domain_computation::primary_graph) fn body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.write_body_field(target, field, value)
    }

    pub(in crate::domain_computation::primary_graph) fn body_create_in_scope<Entity>(
        &mut self,
        scope: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        entity: &str,
        key: &str,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, ()>, WorthQueryApplicationAttemptDenial>
    {
        self.candidate
            .create_body_entity_in_context(scope, entity, key)
    }

    pub(in crate::domain_computation::primary_graph) fn body_initialize_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.initialize_body_field(target, field, value)
    }

    pub(in crate::domain_computation::primary_graph) fn body_optional_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Option<AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate
            .write_body_optional_field(target, field, value)
    }
}
