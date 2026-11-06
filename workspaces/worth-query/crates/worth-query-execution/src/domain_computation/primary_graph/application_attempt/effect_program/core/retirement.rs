//! Entity retirement checks and effect retention.
use super::*;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn delete_at_target(
        &mut self,
        program_target: &ApplicationOperationProgramTarget,
        target: Target<'_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let ApplicationOperationProgramTarget::Delete { entity } = program_target else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                "entity retirement",
            ));
        };
        self.validate_target(target, entity)?;
        self.admit_program_target(program_target)?;
        let EntityReference::Existing(entity_id) = *target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ));
        };
        if self.effects.iter().any(|effect| matches!(effect, WorthQueryApplicationRealizedEffect::RemoveAspect { entity_id: target, .. } if *target == entity_id)) {
            return Err(denial(WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep, entity));
        }
        self.reserve_effects(1)?;
        self.effects
            .push(WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id });
        Ok(())
    }
}
