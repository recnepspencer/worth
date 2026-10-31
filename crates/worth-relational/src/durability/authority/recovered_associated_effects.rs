//! Recovery/runtime affinity admission before associated inventory discovery.
use super::{
    RecoveredRelationalRuntimeAuthority, RelationalAssociatedEffectRefusal,
    RelationalAssociatedEffectSubject,
};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

impl RecoveredRelationalRuntimeAuthority {
    /// Inspect the complete existing associated inventory of this exact recovered
    /// runtime. Affinity is checked even when that inventory is empty.
    ///
    /// This historical recovery authority does not fence later live publication
    /// or issue an absence witness. The closed readmission composition supplies
    /// exclusion before exposing the recovered runtime to ordinary callers.
    pub fn discover_recovered_associated_effects(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Result<Vec<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(RelationalAssociatedEffectRefusal::ForeignRuntime);
        }
        runtime
            .durability_authority()
            .discover_all_associated_effects(execution)
    }
}
