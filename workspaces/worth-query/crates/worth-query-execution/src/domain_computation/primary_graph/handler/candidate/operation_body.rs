//! Execution-only native body write into the existing candidate owner.
use super::{
    ApplicationMutationBinding, ApplicationSchema, CandidateWriter,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    CandidateWriter<'_, Schema, Binding>
{
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
}
