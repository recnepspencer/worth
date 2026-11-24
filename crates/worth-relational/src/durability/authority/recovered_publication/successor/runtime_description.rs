use super::RelationalRecordedSuccessorRefusal;
use crate::branch::RecordedRelationalComponentReference as Recorded;
use crate::durability::authority::associated_effects::discovery::current_settlement;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::durability::authority::ReadmittedRelationalAssociatedEffect;
use crate::runtime::RelationalRuntime;
use worth_relational_durability::internal_port::durability::authority::associated_effects::{
    ReadmittedRelationalAssociatedEffectOwnerPort as _,
    RelationalAssociatedEffectSubjectOwnerPort as _,
};

impl RecoveredRelationalRuntimeAuthority {
    /// Describe the exact original successor from owner-recovered observations.
    /// This borrows the source image and grants no basis, settlement or adoption.
    pub fn recorded_publication_successor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        effect: &ReadmittedRelationalAssociatedEffect,
    ) -> Result<Recorded, RelationalRecordedSuccessorRefusal> {
        use RelationalRecordedSuccessorRefusal as Refusal;
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        if self.runtime_instance_id != runtime.runtime_instance_id()
            || !effect
                .subject()
                .belongs_to_runtime(runtime.runtime_instance_id())
        {
            return Err(Refusal::ForeignRecoveryRuntime);
        }
        let _operation = runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        if current_settlement(effect.subject(), runtime).is_none() {
            return Err(Refusal::SubjectMismatch);
        }
        let entry = effect.subject().source_entry();
        let observations = self
            .recovered_roots
            .publication_observations(entry.commit.commit_id)
            .ok_or(Refusal::MissingRecoveredImage)?;
        let selected =
            super::select_original_successor(execution, effect.branch(), entry, observations)?;
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        Ok(selected)
    }
}
