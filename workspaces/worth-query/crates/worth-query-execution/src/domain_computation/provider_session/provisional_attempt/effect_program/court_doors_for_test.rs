//! Sealed test doors over this module's private members.
use super::*;

impl WorthQuerySessionEffectAuthority<'_> {
    #[doc(hidden)]
    pub fn lower_shared_provisional_program_for_test(
        &self,
        read_set: &WorthQueryFreshDecisionReadSet,
        steps: Arc<[WorthQueryProvisionalEffectStep]>,
    ) -> Result<WorthQueryLoweredProvisionalEffectProgram, WorthQueryProvisionalFailure> {
        Self::lower_shared_provisional_program(self, read_set, steps)
    }
}
