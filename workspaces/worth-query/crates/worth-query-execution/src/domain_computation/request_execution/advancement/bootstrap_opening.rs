//! Bootstrap custody borrows the caller's actual request and native accounting scope.
use super::{
    advancement_denial, ExecutionRequest, ExecutionWorkCeiling, OpeningCustody,
    WorthQueryAdvancementDenial, WorthQueryBootstrapAdvancementPhase,
};

/// Borrow original execution for installation or recovery. Native nested scopes inherit
/// remaining work, cancellation and deadlines; no lease or allowance is opened here.
/// The callback owns completion/error precedence and any performed recovery custody.
/// A callback that ran keeps its result even if a nested pattern stopped, so the
/// opening cannot discard an acknowledged or deferred native owner.
///
/// ```compile_fail,E0521
/// use worth_query_execution::facade::{application_contribution::with_bootstrap_advancement_from_request,
///     provider_session::ExecutionRequest};
/// fn escape(request: ExecutionRequest<'_, '_>) {
///     let _escaped = with_bootstrap_advancement_from_request(request, |phase| phase);
/// }
/// ```
/// ```
/// use worth_query_execution::facade::{application_contribution::with_bootstrap_advancement_from_request,
///     provider_session::ExecutionRequest};
/// fn borrow(request: ExecutionRequest<'_, '_>) {
///     let _ = with_bootstrap_advancement_from_request(request, |phase| {
///         phase.execution_request().consult()
///     });
/// }
/// ```
/// Bootstrap custody stays on the thread holding OpeningCustody and the native meter.
/// ```compile_fail,E0277
/// use worth_query_execution::facade::{application_contribution::with_bootstrap_advancement_from_request,
///     provider_session::ExecutionRequest};
/// fn move_to_worker(request: ExecutionRequest<'_, '_>) {
///     let _ = with_bootstrap_advancement_from_request(request, |phase| {
///         std::thread::scope(|threads| { threads.spawn(move || phase.execution_request().consult()); });
///     });
/// }
/// ```
pub fn with_bootstrap_advancement_from_request<R>(
    request: ExecutionRequest<'_, '_>,
    body: impl for<'scope> FnOnce(WorthQueryBootstrapAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    request.consult().map_err(advancement_denial)?;
    let mut unwind = None;
    let outcome = request.run(ExecutionWorkCeiling::new(request.work_ceiling()), |_| {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            body(WorthQueryBootstrapAdvancementPhase::new(request))
        })) {
            Ok(value) => Some(value),
            Err(payload) => {
                unwind = Some(payload);
                None
            }
        }
    });
    if let Some(payload) = unwind {
        std::panic::resume_unwind(payload);
    }
    outcome
        .map(|(value, _)| value.expect("completed bootstrap callback returned its value"))
        .map_err(advancement_denial)
}
