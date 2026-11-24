//! Continuable unpublished dispatch correlations resolve strictly once published.
use super::recovered::{describe, native_denial, world_denial, RecoveredDispatchSelector};
use super::*;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use worth_foundational::facade::{DurableCompositePublicationAttemptIdentity, WorldOriginalFate};
use worth_runtime_world::facade::{
    RuntimeWorldInspectionPort, RuntimeWorldObservationPort, RuntimeWorldServiceDenial,
    WorldReadmittedPublicationDenial,
};

#[derive(Clone)]
pub struct PendingRecoveredDispatch {
    pub(super) selector: RecoveredDispatchSelector,
    pub(super) attempt: DurableCompositePublicationAttemptIdentity,
}

impl PendingRecoveredDispatch {
    pub fn selector(&self) -> &RecoveredDispatchSelector {
        &self.selector
    }
    pub fn same_pending(&self, other: &Self) -> bool {
        self.attempt == other.attempt && self.selector.same_source(&other.selector)
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn resolve_pending_recovered_dispatch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &ExternalEffectCorrelationIdentity,
        pending: PendingRecoveredDispatch,
        ports: (&RuntimeWorldObservationPort, &RuntimeWorldInspectionPort),
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let (observation, inspection) = ports;
        let resolved = self.graph.with_runtime(execution, |runtime| {
            let native = runtime
                .durability_authority()
                .readmit_associated_effect(execution, &pending.selector.subject)
                .map_err(native_denial)?;
            let original = match observation.readmitted_native_publication(execution, native) {
                Ok(original) => original,
                Err(RuntimeWorldServiceDenial::Denied(
                    WorldReadmittedPublicationDenial::NotReadmitted,
                )) => {
                    let history = inspection
                        .observe_program_activation_history(execution)
                        .map_err(|_| Denial::NotAuthoritative)?;
                    let Some(state) = history.original_state(pending.attempt) else {
                        return Err(Denial::NotAuthoritative);
                    };
                    return match state.fate() {
                        WorldOriginalFate::Open | WorldOriginalFate::Continued => {
                            Err(Denial::PendingPublication)
                        }
                        WorldOriginalFate::Released => Ok(None),
                        WorldOriginalFate::Published => Err(Denial::NotAuthoritative),
                    };
                }
                Err(cause) => return Err(world_denial(cause)),
            };
            let description = describe(&original, inspection)?;
            let record = self.original_record(
                execution,
                &original,
                pending.selector.entity,
                &description,
            )?;
            if record != pending.selector.record {
                return Err(Denial::RecordMismatch);
            }
            self.read_recovered_with_original(
                execution,
                runtime,
                &pending.selector,
                &original,
                description,
            )
            .map(Some)
        })?;
        match resolved {
            Some(owner) => {
                self.completed_commit_evidence
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .promote_pending_recovered_dispatch(*correlation, pending.selector)
                    .map_err(|()| Denial::AmbiguousCorrelation)?;
                Ok(Some(owner))
            }
            None => {
                self.completed_commit_evidence
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove_released_pending_dispatch(correlation);
                Ok(None)
            }
        }
    }
}
