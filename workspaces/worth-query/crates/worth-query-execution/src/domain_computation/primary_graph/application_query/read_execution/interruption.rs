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

/// Safe-point authority is injected by the calling read door. A prepared worker
/// has no Query request scope; both its checkpoints and charges use its map.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph::application_query) enum ReadInterruption<'a> {
    Query(&'a WorthQueryRequestScope),
    Execution(&'a dyn Fn(&str) -> Result<(), WorthQueryApplicationReadExecutionDenial>),
}
impl ReadInterruption<'_> {
    pub(super) fn checkpoint(
        self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        match self {
            Self::Query(request) => checkpoint(request, subject),
            Self::Execution(check) => check(subject),
        }
    }
}
