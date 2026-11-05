use super::WorthQueryGraphProviderExecution;
use crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure;

pub(crate) enum WorthQueryProviderCheckpointRestoreInvocation {
    Returned(Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure>),
    Panicked,
}
