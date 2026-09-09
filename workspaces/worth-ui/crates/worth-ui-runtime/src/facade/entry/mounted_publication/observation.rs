pub(in crate::facade::entry) fn record_mounted_observation(
    host_exchange: &mut crate::host_exchange::WorthUiHostExchangeSessionState,
    observation: crate::mounting::UiMountedHostObservationTransition,
) {
    match observation {
        crate::mounting::UiMountedHostObservationTransition::NeverPresented(frame) => {
            host_exchange.record_never_presented_frame(frame);
        }
        crate::mounting::UiMountedHostObservationTransition::Rejected(frame) => {
            host_exchange.record_rejected_frame(frame);
        }
        crate::mounting::UiMountedHostObservationTransition::Indeterminate { frame, bindings } => {
            host_exchange.record_indeterminate_frame(frame, &bindings);
        }
    }
}
