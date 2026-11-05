//! Only the current receiver arms its actual original recovered effect.
use super::*;
use crate::publication::{
    HomeTransitionEffectControlState, WorldHomeTransitionEffectControl,
    WorldHomeTransitionEffectInterruption,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn arm_home_transition_effect_control(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        claim: &WorldRecoveredUnpublishedEffects,
        mode: WorldHomeTransitionEffectInterruption,
    ) -> Result<WorldHomeTransitionEffectControl, Denial> {
        use crate::recovery::RuntimeWorldRecoveryDenial as Recovery;
        // Scope precedes payload inspection, resource admission and arming.
        self.owner_identity()
            .admit_incarnation(claim.identity().owner_identity())
            .map_err(|cause| {
                Denial::Recovery(match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        Recovery::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => Recovery::ForeignHandle,
                })
            })?;
        if !self.state.recovery.admits_recovered(claim)
            || !self
                .state
                .durable
                .as_ref()
                .is_some_and(|binding| std::sync::Arc::ptr_eq(binding, &claim.record.binding))
        {
            return Err(Denial::Recovery(Recovery::ForeignHandle));
        }
        let prepared = HomeTransitionEffectControlState::prepare(request, mode)?;
        let original = claim
            .record
            .native
            .lock()
            .unwrap_or_else(|cause| cause.into_inner());
        match &*original {
            RecoveredNativeCustody::Relational(native) => {
                if !native
                    .custody()
                    .effect()
                    .subject()
                    .association()
                    .is_some_and(|association| {
                        worth_foundational::facade::HomeProgramTransitionCaller::read(
                            association.as_bytes(),
                        )
                        .is_ok()
                    })
                {
                    return Err(Denial::NativeEffectRequired);
                }
            }
            RecoveredNativeCustody::OpenOnly => return Err(Denial::NativeEffectRequired),
            RecoveredNativeCustody::Continuing(_) | RecoveredNativeCustody::Working => {
                return Err(Denial::Recovery(Recovery::Busy))
            }
            RecoveredNativeCustody::Released => return Err(Denial::ReleaseInProgress),
        }
        let retired = {
            let mut slot = claim
                .record
                .control
                .lock()
                .unwrap_or_else(|cause| cause.into_inner());
            if slot.as_ref().is_some_and(|control| control.is_armed()) {
                return Err(Denial::Recovery(Recovery::Busy));
            }
            slot.replace(prepared.clone())
        };
        drop(original);
        drop(retired);
        Ok(prepared.guard())
    }
}
