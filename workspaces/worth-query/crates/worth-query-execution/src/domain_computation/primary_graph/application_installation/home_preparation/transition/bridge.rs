//! B mappings retain the original Home's Bridge and Signal ownership.
use super::*;
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
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
