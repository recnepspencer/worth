use super::{
    UiNativeClientShutdownObservation, UiNativeEventLoopApplication, UiNativeEventLoopClient,
    UiNativeEventLoopClientCleanup,
};

pub(super) struct UiNativeEventLoopTerminalEvidence {
    pub client_cleanup: Option<Box<dyn UiNativeEventLoopClientCleanup>>,
    pub client_shutdown: Option<UiNativeClientShutdownObservation>,
    pub peak_census: crate::native::UiNativeResourceCensus,
    pub terminal_census: crate::native::UiNativeResourceCensus,
    pub client_closed: bool,
    pub client_resources_complete: bool,
    pub cleanup_complete: bool,
    pub shutdown_overlap: super::UiNativeEventLoopShutdownOverlapObservation,
}

pub(super) fn close<Client: UiNativeEventLoopClient>(
    application: &mut UiNativeEventLoopApplication<Client>,
    host_peak_census: crate::native::UiNativeResourceCensus,
) -> UiNativeEventLoopTerminalEvidence {
    let client_close =
        super::client_close::close(application.client.take(), &application.readiness);
    let client_cleanup = client_close.cleanup;
    let client_shutdown = client_close.shutdown;
    let shutdown_overlap = client_close.overlap;
    let client_closed = client_cleanup.is_none();
    let peak_census = client_shutdown.as_ref().map_or(host_peak_census, |client| {
        host_peak_census.with_client_peak(client.resources())
    });
    crate::native::resize_trace::peaks(peak_census);
    let mut expected_readiness = vec![
        application.readiness_owner,
        application.physical_readiness_owner,
        application.input_readiness_owner,
    ];
    expected_readiness.extend(application.application_readiness_owners.iter().copied());
    let readiness_closure = application.readiness.close_exact(&expected_readiness);
    drop(application.pointer_input.take());
    let mut shared = application.shared.borrow_mut();
    shared
        .resources
        .release_all(application.loop_resources.drain(..))
        .expect("event-loop owners must remain exact");
    let host_census = shared.close();
    let terminal_census = client_shutdown.as_ref().map_or(host_census, |client| {
        host_census.with_client_terminal(client.resources())
    });
    let client_resources_complete = client_shutdown
        .as_ref()
        .is_none_or(UiNativeClientShutdownObservation::terminal_resources_complete);
    drop(shared);
    UiNativeEventLoopTerminalEvidence {
        client_cleanup,
        client_shutdown,
        peak_census,
        terminal_census,
        client_closed,
        client_resources_complete,
        cleanup_complete: super::terminal_cleanup::terminal_cleanup_complete(
            client_closed,
            client_resources_complete,
            readiness_closure.is_complete(),
            &terminal_census,
        ),
        shutdown_overlap,
    }
}
