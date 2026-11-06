//! Typed writers and installed retirement recipes share exact delete admission.
use super::*;
impl<S, O, I, Scope> WorthQueryApplicationEffectProgramBuilder<S, O, I, Scope> {
    pub(in crate::domain_computation::primary_graph) fn delete_at_target<E>(
        &mut self,
        program_target: &ApplicationOperationProgramTarget,
        target: &WorthQueryApplicationEffectEntity<S, E>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.native_effects()
            .delete_at_target(program_target, target.borrowed())
    }
}
