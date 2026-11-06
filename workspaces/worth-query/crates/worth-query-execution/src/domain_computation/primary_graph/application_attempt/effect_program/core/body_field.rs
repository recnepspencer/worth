//! Installed record and operation-body fields retain one effect boundary.
use super::*;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_body_value(
        &mut self,
        target: Target<'_>,
        field: &ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, AspectValue>,
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
        self.write_required_value(field, *entity_id, locator, value)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_body_optional_field(
        &mut self,
        target: Target<'_>,
        field: &ApplicationFieldBindingLocus,
        value: Option<AspectValue>,
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
        if !self.layout.field_is_optional(field.entity(), &locator) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.field(),
            ));
        }
        self.write_optional_value(field, *entity_id, locator, value)
    }
}
