use super::WorthQueryApplicationAttemptDenial;
use super::WorthQueryApplicationEffectProgramBuilder;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(super) fn reserve_effects(
        &mut self,
        additional: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.native_effects().reserve_effects(additional)
    }
}
