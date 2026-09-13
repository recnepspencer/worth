use super::*;

const PORTAL_JOURNEY_CEILING: Duration = Duration::from_secs(45);

pub(in crate::courtroom) fn complete_portal_open() -> PulseExecutableWorld<Published<PortalReady>> {
    let initial = launch_initial(
        CanonicalPlatformPulse::checked_in(),
        PORTAL_JOURNEY_CEILING,
        None,
    );
    let deadline = initial.native_journey_started() + PORTAL_JOURNEY_CEILING;
    let input_reached =
        super::super::platform_pulse_input::reach_native_input_observed(initial, deadline, |_| {});
    let first_current = input_reached
        .publish_first_query_value(QueryStatusV1)
        .unwrap_or_else(|failure| panic!("publish first Query world input: {failure}"))
        .await_first_query_value(deadline)
        .unwrap_or_else(|failure| panic!("first Query value reaches native pixels: {failure}"));
    assert_eq!(first_current.query_evidence().issued_sequence(), 8);
    assert_eq!(first_current.query_evidence().published_sequence(), 9);

    let (visualized, _) = publish_visual_identity(first_current, None);
    publish_second_current(visualized)
}
