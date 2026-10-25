//! Certified footer windows must retain the full-stream grammar without retaining prior bytes.
use super::super::{
    literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, HEADER},
    support::*,
};
use worth_store_physical_integrity::*;

fn prefix() -> PhysicalIntegrityScrubValidator {
    let mut validator = PhysicalIntegrityScrubValidator::new();
    for (scope, bytes) in [
        (header_scope_staged(), HEADER.as_slice()),
        (dirty_scope(identity()), DIRTY_BASIS.as_slice()),
        (compaction_scope(identity()), BINDING_COMPACTION.as_slice()),
        (
            binding_scope(identity(), BINDING.len() as u64),
            BINDING.as_slice(),
        ),
    ] {
        let (result, _) = validator.inspect(PhysicalIntegrityScrubWindow::new(
            0,
            scope,
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        ));
        assert!(matches!(
            result.outcome(),
            PhysicalIntegrityObservationOutcome::Intact(_)
        ));
    }
    validator
}

fn scope(certificates: &[u8], footer: &[u8]) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_footer(
        identity(),
        PhysicalByteRange::new(
            FOOTER_OFFSET + certificates.len() as u64,
            footer.len() as u64,
        )
        .unwrap(),
    )
}

fn window<'a>(
    scope: PhysicalArtifactScope,
    certificates: &'a [u8],
    footer: &'a [u8],
) -> PhysicalIntegrityScrubWindow<'a> {
    PhysicalIntegrityScrubWindow::checkpoint_footer_with_certificates(
        4,
        scope,
        UntrustedPhysicalArtifact::from_bounded_bytes(footer),
        UntrustedPhysicalArtifact::from_bounded_bytes(certificates),
    )
    .unwrap()
}

pub(super) fn assert_admitted(frames: &[Vec<u8>], footer: &[u8]) {
    let certificates = frames.concat();
    let scope = scope(&certificates, footer);
    let footer_window = window(scope, &certificates, footer);
    let mut validator = prefix();
    let (result, counters) = validator.inspect(footer_window.reborrow());
    assert_eq!(
        result.outcome(),
        PhysicalIntegrityObservationOutcome::Intact(scope)
    );
    assert_eq!(
        counters.inspected_bytes(),
        (certificates.len() + footer.len()) as u64
    );
    assert!(
        matches!(
            validator.inspect(footer_window).0.outcome(),
            PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_))
        ),
        "a certified footer cannot replay consumed prefix context"
    );

    for context in [&[][..], &certificates[..certificates.len() - 1]] {
        let mut validator = prefix();
        let (result, _) = validator.inspect(window(scope, context, footer));
        assert!(
            matches!(
                result.outcome(),
                PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(
                    _
                ))
            ),
            "missing or truncated certificate context cannot validate a footer"
        );
    }
    let mut corrupted = certificates.clone();
    corrupted[16] ^= 1;
    let (result, _) = prefix().inspect(window(scope, &corrupted, footer));
    assert!(matches!(result.outcome(),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(damage))
        if damage.cause() == PhysicalDamageCause::SequenceMismatch));
    assert!(
        matches!(
            PhysicalIntegrityScrubValidator::new()
                .inspect(footer_window)
                .0
                .outcome(),
            PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_))
        ),
        "certificates cannot replace the admitted header and binding windows"
    );
}

pub(super) fn assert_denied(frames: &[Vec<u8>], footer: &[u8]) {
    let certificates = frames.concat();
    let scope = scope(&certificates, footer);
    let (result, _) = prefix().inspect(window(scope, &certificates, footer));
    let PhysicalIntegrityObservationOutcome::Rejected(rejection) = result.outcome() else {
        panic!("bounded scrub admitted an ambiguous certificate roster");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}
