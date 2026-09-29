use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};

use super::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};

#[cfg(test)]
mod tests;

/// A safe point between bounded kernel/native steps, never an interrupt inside
/// an opaque native call. Every meter uses the same request-owned authority.
pub(super) fn checkpoint(
    request: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    #[cfg(test)]
    tests::visit_checkpoint(subject);
    match request.interruption() {
        None => Ok(()),
        Some(WorthQueryRequestInterruption::Cancelled) => Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::Cancelled,
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded,
            subject,
        )),
    }
}
