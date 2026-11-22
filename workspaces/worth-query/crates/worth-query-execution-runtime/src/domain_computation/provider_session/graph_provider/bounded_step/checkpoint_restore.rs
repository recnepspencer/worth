use super::WorthQueryGraphProviderExecution;
use crate::domain_computation::WorthQueryGraphProviderFailure;

pub enum WorthQueryProviderCheckpointRestoreInvocation {
    Returned(Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure>),
    Panicked,
}
