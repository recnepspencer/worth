use super::{
    WorthQueryGraphProviderCheckpoint, WorthQueryGraphProviderStep,
    WorthQueryGraphProviderStepDisposition,
};
use crate::domain_computation::WorthQueryGraphProviderFailure;

pub trait WorthQueryGraphProviderExecution: Send + 'static {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure>;

    fn suspend(
        &mut self,
    ) -> Result<Box<dyn WorthQueryGraphProviderCheckpoint>, WorthQueryGraphProviderFailure> {
        Err(WorthQueryGraphProviderFailure::new(
            "provider execution does not support checkpoint suspension",
        ))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure>;
}
