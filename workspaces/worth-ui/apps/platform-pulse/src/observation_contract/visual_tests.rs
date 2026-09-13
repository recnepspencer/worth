use super::lifecycle::PlatformPulseLifecycleObservation;
use super::projection::{
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
    PlatformPulseVisualObservationState,
};
use super::visual::{
    PlatformPulseVisualCoordinateObservation, PlatformPulseVisualCoordinateOrientationObservation,
    PlatformPulseVisualCoordinateRoundingObservation,
    PlatformPulseVisualPixelColorSpaceObservation, PlatformPulseVisualPixelObservation,
    PlatformPulseVisualSnapshotAffinityObservation, PlatformPulseVisualSnapshotCaptured,
    PlatformPulseVisualSnapshotRelationObservation,
};
use super::{
    PlatformPulseLifecycleObservationEnvelope, PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION,
};

#[test]
fn current_schema_snapshot_observation_round_trips_without_pixel_payload_bytes() {
    let (mut stream, _) = PlatformPulseLifecycleObservationStream::start();
    let envelope = stream
        .next_envelope(PlatformPulseLifecycleObservation::VisualSnapshotCaptured(
            snapshot_observation(),
        ))
        .expect("bounded fixture projects");
    let encoded = envelope.encode_prefixed_line().expect("bounded v3 encodes");
    assert!(encoded.len() < 4_096);
    assert!(!encoded.contains("rgba"));
    assert!(!encoded.contains("screenshot"));
    let decoded =
        PlatformPulseLifecycleObservationEnvelope::decode_prefixed_line(&encoded).expect("v3");
    assert_eq!(
        decoded.protocol().schema_version(),
        PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION
    );
    assert_eq!(decoded, envelope);
}

#[test]
fn replacement_rejects_a_partially_observed_visual_pulse() {
    let partial = PlatformPulseVisualObservationState::SnapshotCaptured {
        snapshot: 7,
        frame: 11,
    };
    assert_eq!(
        partial.after_replacement(12),
        Err(PlatformPulseLifecycleObservationProjectionDenial::VisualPulseIncomplete)
    );
}

#[test]
fn cleared_overlay_advances_to_exact_successor_snapshot_basis() {
    let cleared = PlatformPulseVisualObservationState::OverlayCleared {
        snapshot: 7,
        snapshot_frame: 11,
        overlay: 13,
        published_frame: 12,
        cleared_frame: 14,
    };
    assert_eq!(
        cleared.after_replacement(15),
        Ok(
            PlatformPulseVisualObservationState::AwaitingSuccessorSnapshot {
                predecessor_snapshot: 7,
                predecessor_frame: 11,
                successor_frame: 15,
            }
        )
    );
}

#[test]
fn retired_visual_pulse_rebases_without_inventing_a_predecessor_retirement() {
    assert_eq!(
        PlatformPulseVisualObservationState::Retired.after_content_publication(21),
        Ok(PlatformPulseVisualObservationState::AwaitingRefreshSnapshot { refresh_frame: 21 })
    );
    assert_eq!(
        PlatformPulseVisualObservationState::Retired.after_replacement(22),
        Ok(PlatformPulseVisualObservationState::AwaitingRefreshSnapshot { refresh_frame: 22 })
    );
}

#[test]
fn refreshed_visual_pulse_requires_retirement_before_its_next_capture() {
    let awaiting = PlatformPulseVisualObservationState::Refreshed {
        snapshot: 17,
        frame: 19,
    }
    .after_content_publication(23)
    .expect("a current retained snapshot can enter refresh retirement");
    assert_eq!(
        awaiting,
        PlatformPulseVisualObservationState::AwaitingRefreshRetirement {
            snapshot: 17,
            snapshot_frame: 19,
            refresh_frame: 23,
        }
    );
    assert_eq!(awaiting.after_content_publication(23), Ok(awaiting));
}

#[test]
fn in_flight_refresh_coalesces_only_monotonically_newer_content_frames() {
    let initial = PlatformPulseVisualObservationState::AwaitingSnapshot { frame: 13 };
    assert_eq!(
        initial.after_content_publication(17),
        Ok(PlatformPulseVisualObservationState::AwaitingSnapshot { frame: 17 })
    );
    assert!(initial.after_content_publication(12).is_err());

    let overlay = PlatformPulseVisualObservationState::OverlayPublished {
        snapshot: 7,
        snapshot_frame: 11,
        overlay: 13,
        published_frame: 17,
    };
    assert_eq!(overlay.after_content_publication(19), Ok(overlay));
    assert!(overlay.after_content_publication(16).is_err());

    let retained = PlatformPulseVisualObservationState::AwaitingRefreshRetirement {
        snapshot: 17,
        snapshot_frame: 19,
        refresh_frame: 23,
    };
    assert_eq!(
        retained.after_content_publication(29),
        Ok(
            PlatformPulseVisualObservationState::AwaitingRefreshRetirement {
                snapshot: 17,
                snapshot_frame: 19,
                refresh_frame: 29,
            }
        )
    );
    assert_eq!(
        retained.after_content_publication(22),
        Err(PlatformPulseLifecycleObservationProjectionDenial::VisualPulseIncomplete)
    );

    let rebasing =
        PlatformPulseVisualObservationState::AwaitingRefreshSnapshot { refresh_frame: 31 };
    assert_eq!(
        rebasing.after_content_publication(37),
        Ok(PlatformPulseVisualObservationState::AwaitingRefreshSnapshot { refresh_frame: 37 })
    );
}

#[test]
fn content_replacement_interrupts_pending_comparison_without_losing_retirement_evidence() {
    let awaiting = PlatformPulseVisualObservationState::AwaitingSuccessorSnapshot {
        predecessor_snapshot: 17,
        predecessor_frame: 19,
        successor_frame: 23,
    };
    assert_eq!(
        awaiting.after_content_publication(29),
        Ok(
            PlatformPulseVisualObservationState::AwaitingRefreshRetirement {
                snapshot: 17,
                snapshot_frame: 19,
                refresh_frame: 29,
            }
        )
    );
    assert_eq!(
        awaiting.after_content_publication(22),
        Err(PlatformPulseLifecycleObservationProjectionDenial::VisualPulseIncomplete)
    );
}

#[test]
fn refreshed_snapshot_accepts_a_current_runtime_owned_successor_after_its_initiating_frame() {
    let awaiting =
        PlatformPulseVisualObservationState::AwaitingRefreshSnapshot { refresh_frame: 139 };

    assert_eq!(
        awaiting.after_refreshed_snapshot(41, 144, true),
        Ok(PlatformPulseVisualObservationState::Refreshed {
            snapshot: 41,
            frame: 144,
        })
    );
    assert_eq!(
        awaiting.after_refreshed_snapshot(41, 138, true),
        Err(PlatformPulseLifecycleObservationProjectionDenial::
            VisualRefreshSnapshotAffinityMismatch {
                expected_frame: 139,
                observed_frame: 138,
                observed_current: true,
            })
    );
    assert_eq!(
        awaiting.after_refreshed_snapshot(41, 144, false),
        Err(PlatformPulseLifecycleObservationProjectionDenial::
            VisualRefreshSnapshotAffinityMismatch {
                expected_frame: 139,
                observed_frame: 144,
                observed_current: false,
            })
    );
}

fn snapshot_observation() -> PlatformPulseVisualSnapshotCaptured {
    PlatformPulseVisualSnapshotCaptured {
        affinity: PlatformPulseVisualSnapshotAffinityObservation {
            snapshot: 7,
            presentation_attempt: 8,
            frame: 9,
            semantic_surface: 10,
            host_surface: 11,
            binding_generation: 12,
            presentation_epoch: 13,
            relation: PlatformPulseVisualSnapshotRelationObservation::Current,
        },
        captured_client_extent: [0, 0, 160, 96],
        coordinates: PlatformPulseVisualCoordinateObservation {
            native_client_origin: [20, 30],
            client_physical_dimensions: [160, 96],
            viewport_logical_dimension_bits: [160.0_f32.to_bits(), 96.0_f32.to_bits()],
            scale_bits: [1.0_f32.to_bits(), 1.0_f32.to_bits()],
            translation_bits: [0.0_f32.to_bits(), 0.0_f32.to_bits()],
            orientation: PlatformPulseVisualCoordinateOrientationObservation::TopLeftOrigin,
            rounding: PlatformPulseVisualCoordinateRoundingObservation::PixelCenterNearest,
        },
        pixels: PlatformPulseVisualPixelObservation {
            dimensions: [160, 96],
            stride: 640,
            byte_count: 61_440,
            color_space: PlatformPulseVisualPixelColorSpaceObservation::Srgb,
        },
        visible_region_count: 2,
        hit_test_region_count: 2,
        cost_counters: [2, 1, 2, 8, 61_440, 61_440, 61_440, 1, 0, 1, 4_096],
    }
}
