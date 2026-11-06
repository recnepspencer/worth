//! Admit the actual newly created source without replacing its Native producer.
use super::DurabilityRecoveryAuthority;
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RecoveryPlan, RelationalRecoveryStop as Stop,
};
use crate::durability::medium::RelationalDurableReadProtection;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod initial_inputs;
mod receipt;
mod stop;
pub use receipt::FreshRelationalSourceAdmission;
pub use stop::RelationalFreshSourceAdmissionStop;
#[cfg(test)]
mod tests;

impl DurabilityRecoveryAuthority<'_> {
    /// Closed Store ingress: physical ProvenAbsent custody selects this door.
    /// Empty coverage alone never grants permission to choose fresh admission.
    pub fn admit_fresh_home_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<FreshRelationalSourceAdmission, RelationalFreshSourceAdmissionStop> {
        crate::execution::check_request(execution).map_err(Stop::Request)?;
        let admitted =
            super::recovery::admit_recovery_or_emit(self.runtime, plan).map_err(Stop::from)?;
        // This owner admitted the plain plan above; an opaque prepared Home
        // recovery cannot enter fresh-source admission.
        let (plan, _prepared_home_schemas) = admitted.into_parts();
        if plan.checkpoint.is_some()
            || !plan.tail_log.is_empty()
            || !plan.branch_transitions.transitions().is_empty()
            || !plan.restore_authoritative_envelope_commit_ids.is_empty()
        {
            return Err(refused("fresh source contains durable history").into());
        }
        let source = plan
            .cold_source
            .as_ref()
            .ok_or_else(|| refused("fresh source is absent"))?;
        let actual = self
            .runtime
            .durability
            .cold_source()
            .ok_or_else(|| refused("fresh runtime has no attached source"))?;
        if !Arc::ptr_eq(source, &actual) {
            return Err(refused("fresh source belongs to another attachment").into());
        }
        // Provider locks and protected source observation precede logical epochs.
        let mut protection =
            RelationalDurableReadProtection::capture(source, execution).map_err(|cause| {
                crate::durability::checkpoint_history::capture::owned_stop(
                    cause,
                    crate::storage::data::RelationalStorageSubject::Envelope(
                        crate::publication::patch::data::PatchStreamPosition(0),
                    ),
                )
            })?;
        if protection.bounds().canonical.0 != 0
            || protection.bounds().transitions.0 != 0
            || protection.bounds().selected_checkpoint
        {
            return Err(refused("fresh protected source has a nonempty head").into());
        }
        if initial_inputs::ensure_original_initialization(
            self.runtime,
            execution,
            source,
            &protection,
        )? {
            // This original owner just published Birth. Observe its own actual
            // successor cut; existing-source recovery never takes this route.
            protection =
                RelationalDurableReadProtection::capture(source, execution).map_err(|cause| {
                    crate::durability::checkpoint_history::capture::owned_stop(
                        cause,
                        crate::storage::data::RelationalStorageSubject::Envelope(
                            crate::publication::patch::data::PatchStreamPosition(0),
                        ),
                    )
                })?;
            initial_inputs::ensure_original_initialization(
                self.runtime,
                execution,
                source,
                &protection,
            )?;
        }
        let receipt = FreshRelationalSourceAdmission::capture(self.runtime, execution)?;
        crate::execution::check_request(execution).map_err(Stop::Request)?;
        // Existing source/medium/root bindings remain unchanged. Protected-read
        // abandonment has its existing ordinary disposal boundary, not retirement.
        drop(protection);
        Ok(receipt)
    }
}
fn refused(detail: &'static str) -> Stop {
    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into()
}
