//! The no-effect decision samples the installed clock after contact.
use super::*;
use std::time::{Duration, UNIX_EPOCH};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::authorization_time::AuthorizationTimeController;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::provider::committed_dispatch_outbox::test_support::commit_observe_and_admit_with_clock_fixture;

struct RejectingClockTransport {
    clock: AuthorizationTimeController,
    unavailable: bool,
}
impl WorthQueryExternalEffectTransport for RejectingClockTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        if self.unavailable {
            self.clock.script([]);
        } else {
            self.clock.hold(UNIX_EPOCH + Duration::from_millis(23_456));
        }
        WorthQueryExternalTransportOutcome::Rejected
    }
}

#[test]
fn rejection_samples_postcontact_time_and_preserves_no_effect_when_time_is_unavailable() {
    for (seed, unavailable) in [(32, false), (33, true)] {
        let (admitted, clock) = commit_observe_and_admit_with_clock_fixture(seed);
        clock.hold(UNIX_EPOCH + Duration::from_millis(12_345));
        let dispatch = dispatch_external_effect(
            &RejectingClockTransport { clock, unavailable },
            admitted,
            || {},
        )
        .unwrap();
        let no_effect = dispatch.posture().proven_no_effect().unwrap();
        assert_eq!(
            no_effect.cause(),
            ExternalEffectNoEffectCause::PayloadRejected
        );
        assert_eq!(
            no_effect.attempt_posture(),
            dispatch.causal_ladder().attempt()
        );
        match (unavailable, no_effect.decision_time()) {
            (true, None) => {}
            (false, Some(time)) => {
                assert_eq!(
                    time.timeline(),
                    ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds
                );
                assert_eq!(time.value(), &AspectValue::UInt64(23_456));
            }
            (unavailable, time) => {
                panic!("unexpected decision time: unavailable={unavailable}, time={time:?}")
            }
        }
        assert!(!dispatch.is_external_completion());
        assert_eq!(dispatch.fault(), None);
    }
}
