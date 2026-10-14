//! Installed body effects use native target, footprint and reservation checks.
use super::{
    denial, ApplicationOperationProgramTarget, EntityReference, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn write_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.field(),
            ));
        };
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        if self.layout.field_is_optional(field.entity(), &locator) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.field(),
            ));
        }
        self.write_required_value(field, entity_id, locator, std::borrow::Cow::Borrowed(value))
    }
}
