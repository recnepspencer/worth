//! Live stage values stay private until their real group publication completes.
use super::*;
pub struct RetainedAuthoredStageResult<R> {
    pub result: R,
    pub identity: ApplicationEditStageIdentity,
    pub receipt: Option<WorthQueryApplicationCommitReceipt>,
}
/// A genuine domain refusal retains its original value and the stage that
/// returned it, independently of cleanup from the abandoned candidate.
pub struct AuthoredStageDomainDenial {
    pub identity: ApplicationEditStageIdentity,
    pub binding: &'static str,
    pub denial: Box<dyn Any + Send>,
}

/// Original non-domain execution refusal, with its stage and interruption posture.
#[derive(Debug)]
pub enum AuthoredStageExecutionCause {
    Cancelled,
    DeadlineExceeded,
    Handler(HandlerExecutionDenial),
    Request(MutationHandlerExecutionDenial),
}
#[derive(Debug)]
pub struct AuthoredStageExecutionDenial {
    pub identity: ApplicationEditStageIdentity,
    pub binding: &'static str,
    pub cause: AuthoredStageExecutionCause,
}
