//! Certified footer windows must retain the full-stream grammar without retaining prior bytes.
pub(crate) use worth_store_physical_integrity::*;
pub(crate) use {
    crate::checkpoint_validation::support::assert_damage,
    crate::checkpoint_validation::support::binding_scope,
    crate::checkpoint_validation::support::compaction_scope,
    crate::checkpoint_validation::support::dirty_scope,
    crate::checkpoint_validation::support::header_scope_staged,
    crate::checkpoint_validation::support::identity,
    crate::checkpoint_validation::support::FOOTER_OFFSET,
    crate::literal_vectors::checkpoint_validation::BINDING,
    crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION,
    crate::literal_vectors::checkpoint_validation::DIRTY_BASIS,
    crate::literal_vectors::checkpoint_validation::HEADER,
    worth_store_physical_integrity::PhysicalArtifactScope,
    worth_store_physical_integrity::PhysicalBlastRadius,
    worth_store_physical_integrity::PhysicalByteRange,
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
    worth_store_physical_integrity::UntrustedPhysicalArtifact,
};

pub(crate) fn prefix() -> PhysicalIntegrityScrubValidator {
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
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        ));
        assert!(matches!(
            result.outcome(),
            PhysicalIntegrityObservationOutcome::Intact(_)
        ));
    }
    validator
}

pub(crate) fn scope(certificates: &[u8], footer: &[u8]) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        identity(),
        worth_store_physical_integrity::PhysicalByteRange::new(
            FOOTER_OFFSET + certificates.len() as u64,
            footer.len() as u64,
        )
        .unwrap(),
    )
}

pub(crate) fn window<'a>(
    scope: PhysicalArtifactScope,
    certificates: &'a [u8],
    footer: &'a [u8],
) -> PhysicalIntegrityScrubWindow<'a> {
    PhysicalIntegrityScrubWindow::checkpoint_footer_with_certificates(
        4,
        scope,
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(footer),
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(certificates),
    )
    .unwrap()
}

pub(crate) fn assert_admitted(frames: &[Vec<u8>], footer: &[u8]) {
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
            PhysicalIntegrityObservationOutcome::Rejected(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
            )
        ),
        "a certified footer cannot replay consumed prefix context"
    );

    for context in [&[][..], &certificates[..certificates.len() - 1]] {
        let mut validator = prefix();
        let (result, _) = validator.inspect(window(scope, context, footer));
        assert!(
            matches!(
                result.outcome(),
                PhysicalIntegrityObservationOutcome::Rejected(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
                )
            ),
            "missing or truncated certificate context cannot validate a footer"
        );
    }
    let mut corrupted = certificates.clone();
    corrupted[16] ^= 1;
    let (result, _) = prefix().inspect(window(scope, &corrupted, footer));
    assert!(matches!(result.outcome(),
        PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage))
        if damage.cause() == worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch));
    assert!(
        matches!(
            PhysicalIntegrityScrubValidator::new()
                .inspect(footer_window)
                .0
                .outcome(),
            PhysicalIntegrityObservationOutcome::Rejected(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
            )
        ),
        "certificates cannot replace the admitted header and binding windows"
    );
}

pub(crate) fn assert_denied(frames: &[Vec<u8>], footer: &[u8]) {
    let certificates = frames.concat();
    let scope = scope(&certificates, footer);
    let (result, _) = prefix().inspect(window(scope, &certificates, footer));
    let PhysicalIntegrityObservationOutcome::Rejected(rejection) = result.outcome() else {
        panic!("bounded scrub admitted an ambiguous certificate roster");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
