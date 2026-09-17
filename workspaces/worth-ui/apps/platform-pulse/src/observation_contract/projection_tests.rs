use super::{
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
};

#[test]
fn process_start_origin_and_terminal_progression_are_monotonic_and_closed() {
    let (mut stream, started) = PlatformPulseLifecycleObservationStream::start();
    assert_eq!(started.sequence().value(), 1);
    let terminal = stream
        .project_native_event_loop_failure()
        .expect("terminal event");
    assert_eq!(terminal.sequence().value(), 2);
    assert_eq!(
        stream.project_source_worker_panic(),
        Err(PlatformPulseLifecycleObservationProjectionDenial::StreamTerminated)
    );
}
