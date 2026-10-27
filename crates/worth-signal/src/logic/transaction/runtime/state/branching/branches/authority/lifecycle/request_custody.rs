use super::*;
use crate::data::resource::InFlightResourceRequest;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn require_owned_async_handle(
        &self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let actual = self
            .derived
            .resource
            .in_flight_request_optional(handle, None)
            .map(InFlightResourceRequest::node);
        if actual == Some(node) {
            Ok(())
        } else {
            Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch)
        }
    }

    pub(super) fn require_in_flight(
        &mut self,
        handle: ResourceRequestHandle,
    ) -> Result<InFlightResourceRequest, SignalConditionalServiceExecutionDenial> {
        let capture = self.captures_telemetry();
        self.derived
            .resource
            .in_flight_request_optional(
                handle,
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .cloned()
            .ok_or_else(|| {
                SignalConditionalServiceExecutionDenial::SlotAdmission(
                    crate::data::error::SignalError::invalid_input(
                        "owned async lifecycle admission lost its in-flight state",
                    ),
                )
            })
    }
}
