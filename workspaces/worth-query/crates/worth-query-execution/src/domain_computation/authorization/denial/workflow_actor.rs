//! Keep the original native source/storage stop behind the admission refusal.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as NativeDenial,
    WorthQueryApplicationAttemptDenialKind as NativeKind,
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(super) struct WorkflowActorNativeDenial(Arc<NativeDenial>);
impl PartialEq for WorkflowActorNativeDenial {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WorkflowActorNativeDenial {}
impl WorthQueryOperationAuthorizationDenial {
    pub(in crate::domain_computation) fn workflow_actor_native(cause: NativeDenial) -> Self {
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
        let kind = match cause.kind() {
            NativeKind::Cancelled => {
                WorthQueryOperationAuthorizationDenialKind::Stopped(Err(Control::Cancelled))
            }
            NativeKind::DeadlineExceeded => {
                WorthQueryOperationAuthorizationDenialKind::Stopped(Err(Control::TimedOut))
            }
            NativeKind::ExecutionRequest(kind) => {
                WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(kind))
            }
            NativeKind::AllocationDenied
            | NativeKind::RetainedSourceStorageDenied
            | NativeKind::DecisionFactBudgetExceeded => {
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::WorkflowActorEvidenceUnavailable,
                )
            }
            _ => WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::WorkflowActorSourceInvalid,
            ),
        };
        let mut denial = Self::new(kind, cause.subject());
        denial.workflow_actor_cause = Some(WorkflowActorNativeDenial(Arc::new(cause)));
        denial
    }
    /// The original native selection or storage denial, when admission stopped there.
    pub fn workflow_actor_source_denial(&self) -> Option<&NativeDenial> {
        self.workflow_actor_cause
            .as_ref()
            .map(|cause| cause.0.as_ref())
    }
}

impl worth_execution::ChargedBytes for WorkflowActorNativeDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // Arc retains the native value and its nested payload through every clone.
        (std::mem::size_of::<NativeDenial>() as u64)
            .saturating_add((2 * std::mem::size_of::<usize>()) as u64)
            .saturating_add(worth_execution::ChargedBytes::additional_charged_bytes(
                self.0.as_ref(),
            ))
    }
}
