use super::*;

#[test]
fn platform_pulse_profile_and_zero_axis_denials_are_exact() {
    let profile = UiObservationProfile::platform_pulse();
    assert_eq!(profile.admitted_per_turn(), 8);
    assert_eq!(profile.retained_bytes_per_turn(), 65_536);
    assert_eq!(profile.queued_during_effecting_rebind(), 16);

    let baseline = UiObservationProfileInput {
        admitted_per_turn: 1,
        retained_bytes_per_turn: 1,
        queued_during_effecting_rebind: 1,
    };
    for (input, expected) in [
        (
            UiObservationProfileInput {
                admitted_per_turn: 0,
                ..baseline
            },
            UiObservationProfileConstructionDenial::EmptyTurnCapacity,
        ),
        (
            UiObservationProfileInput {
                retained_bytes_per_turn: 0,
                ..baseline
            },
            UiObservationProfileConstructionDenial::EmptyByteCapacity,
        ),
        (
            UiObservationProfileInput {
                queued_during_effecting_rebind: 0,
                ..baseline
            },
            UiObservationProfileConstructionDenial::EmptyQueueCapacity,
        ),
    ] {
        assert_eq!(UiObservationProfile::bounded(input), Err(expected));
    }
}

#[test]
fn family_definitions_preserve_closed_owner_and_framework_order() {
    let expected = [
        (
            UiObservationFamily::AuthoredSource,
            UiObservationOwner::SourceIngress,
        ),
        (
            UiObservationFamily::HostViewport,
            UiObservationOwner::HostViewport,
        ),
        (
            UiObservationFamily::HostDeviceScale,
            UiObservationOwner::HostDeviceScale,
        ),
        (
            UiObservationFamily::PointerPresenceTarget,
            UiObservationOwner::PointerPresenceRuntimeState,
        ),
        (
            UiObservationFamily::Measurement,
            UiObservationOwner::MeasurementExchange,
        ),
        (UiObservationFamily::Query, UiObservationOwner::QueryBinding),
        (
            UiObservationFamily::IntentPosture,
            UiObservationOwner::IntentRuntime,
        ),
        (
            UiObservationFamily::CommittedScrollExtent,
            UiObservationOwner::ScrollRuntimeState,
        ),
        (
            UiObservationFamily::CommittedPortalAnchor,
            UiObservationOwner::PortalRuntimeState,
        ),
        (
            UiObservationFamily::CommittedFocus,
            UiObservationOwner::FocusRuntimeState,
        ),
        (
            UiObservationFamily::CommittedSelection,
            UiObservationOwner::SelectionRuntimeState,
        ),
        (
            UiObservationFamily::CommittedMotionTrack,
            UiObservationOwner::MotionRuntimeState,
        ),
        (
            UiObservationFamily::CommittedCommandRoute,
            UiObservationOwner::CommandRoutingRuntimeState,
        ),
    ];
    for (rank, (family, owner)) in expected.into_iter().enumerate() {
        let definition = family.definition();
        assert_eq!(definition.family(), family);
        assert_eq!(definition.owner(), owner);
        assert_eq!(usize::from(definition.framework_rank()), rank);
    }

    assert_eq!(
        UiObservationFamily::Query.definition().reset_policy(),
        UiObservationResetPolicy::OwnerIssuedReset
    );
    assert_eq!(
        UiObservationFamily::HostViewport
            .definition()
            .coalescing_policy(),
        UiObservationCoalescingPolicy::OwnerEquivalentOnly
    );
    assert_eq!(
        UiObservationFamily::AuthoredSource
            .definition()
            .coalescing_policy(),
        UiObservationCoalescingPolicy::Forbidden
    );
    let intent_posture = UiObservationFamily::IntentPosture.definition();
    assert_eq!(
        intent_posture.loss_policy(),
        UiObservationLossPolicy::Lossless
    );
    assert_eq!(
        intent_posture.reset_policy(),
        UiObservationResetPolicy::NoReset
    );
    assert_eq!(
        intent_posture.coalescing_policy(),
        UiObservationCoalescingPolicy::Forbidden
    );

    for family in [
        UiObservationFamily::PointerPresenceTarget,
        UiObservationFamily::CommittedFocus,
        UiObservationFamily::CommittedSelection,
        UiObservationFamily::CommittedMotionTrack,
        UiObservationFamily::CommittedCommandRoute,
    ] {
        let definition = family.definition();
        assert_eq!(definition.loss_policy(), UiObservationLossPolicy::Lossless);
        assert_eq!(definition.reset_policy(), UiObservationResetPolicy::NoReset);
        assert_eq!(
            definition.coalescing_policy(),
            UiObservationCoalescingPolicy::OwnerEquivalentOnly
        );
    }
}
