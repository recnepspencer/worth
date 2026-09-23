use super::{
    UiHostObservationMountedBasis, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UiHostScrollDeltaSource,
    UiHostScrollDeltaTargetAffinity, UiHostScrollLineCountBasis, UiHostSurfacePosition,
    UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
};
use crate::{
    UiHostPresentationEpoch, UiHostSurfaceIdentity, UiMountedFrameIdentity,
    UiMountedNodeReceiptIdentity, UiSurfaceBindingGeneration,
};

#[test]
fn scroll_identity_covers_source_phase_precision_delta_and_target_form() {
    let presentation = presentation();
    let position = UiHostSurfacePosition::viewport_logical(12_000, 34_000);
    let receipt = UiMountedNodeReceiptIdentity::mint_unbound().unwrap();
    let mounted = UiHostObservationMountedBasis::new(receipt.mounted_instance(), receipt);
    let base = scroll(
        UiHostScrollDeltaSource::PointerWheel,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaPrecision::Pixel,
        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
        4_000,
        -8_000,
    );
    assert_eq!(base.encoded_len(), 70);
    assert_eq!(base.coalescing_identity(), None);
    let successor_frame_presentation = UiHostObservationPresentationBasis::new(
        presentation.host_surface(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        presentation.binding(),
        presentation.epoch(),
    );
    let successor_surface_presentation = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        presentation.frame(),
        presentation.binding(),
        presentation.epoch(),
    );
    let successor_binding_presentation = UiHostObservationPresentationBasis::new(
        presentation.host_surface(),
        presentation.frame(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        presentation.epoch(),
    );
    let successor_epoch_presentation = UiHostObservationPresentationBasis::new(
        presentation.host_surface(),
        presentation.frame(),
        presentation.binding(),
        UiHostPresentationEpoch::issued_by_host(2),
    );
    let variants = [
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(
                successor_frame_presentation,
                position,
            ),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(
                successor_surface_presentation,
                position,
            ),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(
                successor_binding_presentation,
                position,
            ),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(
                successor_epoch_presentation,
                position,
            ),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::Touch,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Ended,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: 3,
                basis: UiHostScrollLineCountBasis::PlatformReported,
            },
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: 4,
                basis: UiHostScrollLineCountBasis::PlatformReported,
            },
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: 3,
                basis: UiHostScrollLineCountBasis::DefaultedAfterMissing,
            },
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: 3,
                basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
            },
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Page,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(
                presentation,
                UiHostSurfacePosition::viewport_logical(12_001, 34_000),
            ),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_mounted_target(presentation, mounted),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::presented_surface_fallback(presentation),
            4_000,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_001,
            -8_000,
        ),
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position),
            4_000,
            -8_001,
        ),
    ];
    for variant in variants {
        assert_ne!(base.integrity_digest(), variant.integrity_digest());
    }
}

#[test]
fn scroll_target_accessors_preserve_honest_adapter_knowledge() {
    let presentation = presentation();
    let position = UiHostSurfacePosition::viewport_logical(7, 9);
    let exact = UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position);
    let fallback = UiHostScrollDeltaTargetAffinity::presented_surface_fallback(presentation);

    assert_eq!(exact.position(), Some(position));
    assert_eq!(exact.mounted_target(), None);
    assert!(!exact.is_surface_fallback());
    assert_eq!(fallback.position(), None);
    assert_eq!(fallback.mounted_target(), None);
    assert!(fallback.is_surface_fallback());
    assert_eq!(
        scroll(
            UiHostScrollDeltaSource::PointerWheel,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                basis: UiHostScrollLineCountBasis::DefaultedAfterMissing,
            },
            fallback,
            0,
            1,
        )
        .encoded_len(),
        55,
    );
}

/// A line precision declares the line count it applied, and pays for it.
///
/// The reader recovers the notch count by dividing the carried lines by the
/// count the host states, so the count is part of the observation rather than
/// shared knowledge, and the batch byte budget charges for the three extra
/// bytes it occupies. Pixel and page precisions carry no count and cost what
/// they always cost.
#[test]
fn precision_cost_and_stated_line_count_travel_with_the_observation() {
    let presentation = presentation();
    let position = UiHostSurfacePosition::viewport_logical(12_000, 34_000);
    let target = UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position);
    let line = UiHostScrollDeltaPrecision::Line {
        platform_lines_per_notch: 5,
        basis: UiHostScrollLineCountBasis::PlatformReported,
    };

    assert_eq!(line.lines_per_notch(), Some(5));
    assert_eq!(
        line.line_count_basis(),
        Some(UiHostScrollLineCountBasis::PlatformReported)
    );
    assert_eq!(UiHostScrollDeltaPrecision::Page.lines_per_notch(), None);
    assert_eq!(UiHostScrollDeltaPrecision::Page.line_count_basis(), None);
    assert_eq!(UiHostScrollDeltaPrecision::Pixel.lines_per_notch(), None);
    assert_eq!(UiHostScrollDeltaPrecision::Pixel.line_count_basis(), None);
    assert_eq!(UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH, 3);

    let pixel = scroll(
        UiHostScrollDeltaSource::PointerWheel,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaPrecision::Pixel,
        target,
        4_000,
        -8_000,
    );
    let page = scroll(
        UiHostScrollDeltaSource::PointerWheel,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaPrecision::Page,
        target,
        4_000,
        -8_000,
    );
    let lines = scroll(
        UiHostScrollDeltaSource::PointerWheel,
        UiHostScrollDeltaPhase::Updated,
        line,
        target,
        4_000,
        -8_000,
    );

    assert_eq!(page.encoded_len(), pixel.encoded_len());
    assert_eq!(lines.encoded_len(), pixel.encoded_len() + 3);
}

fn presentation() -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(1),
    )
}

fn scroll(
    source: UiHostScrollDeltaSource,
    phase: UiHostScrollDeltaPhase,
    precision: UiHostScrollDeltaPrecision,
    target: UiHostScrollDeltaTargetAffinity,
    x_subpixels: i64,
    y_subpixels: i64,
) -> UiHostObservationPayload {
    UiHostObservationPayload::ScrollDelta {
        source,
        phase,
        precision,
        target,
        x_subpixels,
        y_subpixels,
    }
}
