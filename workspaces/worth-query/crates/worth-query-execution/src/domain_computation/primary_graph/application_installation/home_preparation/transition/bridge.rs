//! B mappings retain the original Home's Bridge and Signal ownership.
use super::*;
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    /// D16's present handoff is Relational-only. Reject future Signal programme
    /// migration before publishing an intent or changing the descriptor binding.
    pub fn require_relational_programme_transition(
        &self,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if !self._pending._state.pending.producers.is_empty()
            || !self._pending._state.pending.conditionals.is_empty()
        {
            return Err(super::author_source::denied(
                "authored Signal programme migration requires its later migration phase",
            ));
        }
        Ok(())
    }
    pub fn prepare_home_program_bridge(
        &self,
        incumbent: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> Result<worth_runtime_bridge::facade::RuntimeBridge, WorthQueryPrimaryGraphInstallationDenial>
    {
        super::super::super::super::managed_bridge::home_definition::prepare_home_program_mappings(
            incumbent,
            &self._pending._state.installed,
            &self._pending._layout,
        )
    }
}
