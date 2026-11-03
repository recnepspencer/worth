//! Sealed test doors over this module's private members.
use super::*;

impl QueryRequestExecution<'_> {
    #[doc(hidden)]
    pub fn run_advancement_for_test<R>(
        &self, owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>, source_instance: Option<u64>, body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        Self::run_advancement(self, owner, source_instance, body)
    }
}
