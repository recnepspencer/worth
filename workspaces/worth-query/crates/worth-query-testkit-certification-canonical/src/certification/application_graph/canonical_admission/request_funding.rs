//! Decoder requests borrow a bounded descendant after independently funded installation.
use super::host::{Host, WorthQueryAdvancementPhase, WorthQueryRequestScope};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

pub fn with_request<R>(
    host: &Host,
    scope: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> R {
    let runtime = host.program.runtime();
    runtime
        .with_application_advancement(scope, |phase| {
            let original = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let child = original
                .in_scope(|parent| {
                    let parent = parent.expect("native installation lends its admitted request");
                    parent
                        .child(LeaseRequest {
                            policy: ExecutionRequestPolicy::new(
                                parent.policy().posture(),
                                parent.policy().determinism(),
                                ExecutionBudget::new(
                                    std::num::NonZeroUsize::MIN,
                                    Some(64 * 1024 * 1024),
                                    Some(host.request_work),
                                ),
                            ),
                            cancellation: scope.cancellation().execution_token(),
                            deadline: scope.deadline(),
                        })
                        .unwrap()
                })
                .unwrap();
            ExecutionRequest::leased(&child)
                .run(ExecutionWorkCeiling::new(Some(host.request_work)), |_| {
                    body(phase)
                })
                .expect("the original bounded request executes")
                .0
        })
        .expect("the installed runtime lends its native phase")
}
