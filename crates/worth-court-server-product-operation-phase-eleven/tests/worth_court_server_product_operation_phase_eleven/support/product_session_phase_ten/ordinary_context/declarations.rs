pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc, std::sync::Barrier, std::sync::Mutex};

pub(crate) use {worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerProductSessionCreationRequest, worth_server::WorthServerProductSessionDenialCode, worth_server::WorthServerProductSessionLifecycle, worth_server::WorthServerProductSessionTermination, worth_server::WorthServerProductSessionTerminationKind, worth_server::WorthServerProductSessionTerminationObserver};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture as fixture};

pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server_with_clock_and_observers, worth_testkit_server::shared_support_product_session_phase_ten_fixture::direct_session, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_product_mutation_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_product_read_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_session_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::preview_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::session_backed_editor_registration, worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock};

#[derive(Debug, Default)]
pub(crate) struct RecordingTerminationObserver {
    pub(crate) events: Mutex<Vec<(String, WorthServerProductSessionTerminationKind)>>,
}

impl WorthServerProductSessionTerminationObserver for RecordingTerminationObserver {
    fn observe_termination(&self, termination: &WorthServerProductSessionTermination) {
        self.events.lock().expect("observer lock").push((
            termination.session().identity().as_str().to_string(),
            termination.kind(),
        ));
    }
}