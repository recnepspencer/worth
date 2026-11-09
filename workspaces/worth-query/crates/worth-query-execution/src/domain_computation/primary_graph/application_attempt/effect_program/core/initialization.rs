//! New-entity fields use the same admitted column and same-program target.
use super::*;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn initialize_field(
        &mut self,
        target: Target<'_>,
        field: ColumnIdentity,
        encoder: &dyn FieldEncoder,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let column = self.layout.record_column(field)?;
        let value = encoder.encode().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                column.path(),
            )
        })?;
        self.initialize_encoded_field(target, &field.locus(), std::borrow::Cow::Owned(value))
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn initialize_encoded_field(
        &mut self,
        target: Target<'_>,
        field: &ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.reserve_effects(0)?;
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        })?;
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let Some(created_effect) = target.created_effect else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        let Some(WorthQueryApplicationRealizedEffect::CreateEntity { .. }) =
            self.effects.get(created_effect)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        let presence = self.layout.record_presence_locator(field).cloned();
        let WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. } =
            &mut self.effects[created_effect]
        else {
            unreachable!("validated created effect changed before field retention");
        };
        fields.insert(locator, value.into_owned());
        if let Some(presence) = presence {
            fields.insert(
                presence,
                worth_foundational::facade::AspectValue::Bool(true),
            );
        }
        Ok(())
    }
}
