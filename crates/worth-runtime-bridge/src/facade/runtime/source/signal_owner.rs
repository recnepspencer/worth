use crate::facade::RuntimeBridge;
use crate::source::BridgeSignalRuntime;
use worth_signal::facade::SignalError;

impl RuntimeBridge {
    pub(super) fn with_signal_source<T>(
        &self,
        operation: impl FnOnce(&mut BridgeSignalRuntime) -> Result<T, SignalError>,
    ) -> Result<T, SignalError> {
        crate::source::with_async_request_signal_runtime(self.signal_runtime_key, operation)
            .map_err(|error| {
                SignalError::invalid_input(format!("Bridge Signal owner is unavailable: {error:?}"))
            })?
    }
}
