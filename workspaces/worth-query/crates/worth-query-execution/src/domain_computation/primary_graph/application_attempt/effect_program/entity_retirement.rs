//! Typed writers and installed retirement recipes share exact delete admission.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::model::WorthQueryApplicationEffectEntity;
use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind;
impl<S, O, I, Scope> WorthQueryApplicationEffectProgramBuilder<S, O, I, Scope> {
    pub(in crate::domain_computation::primary_graph) fn delete_at_target<E>(
        &mut self,
        program_target: &ApplicationOperationProgramTarget,
        target: &WorthQueryApplicationEffectEntity<S, E>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let ApplicationOperationProgramTarget::Delete { entity } = program_target else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                "entity retirement",
            ));
        };
        self.validate_target(target, entity)?;
        self.admit_program_target(program_target)?;
        let EntityReference::Existing(entity_id) = target.reference else {
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
