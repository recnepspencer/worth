#[allow(unused_imports)]
use crate::checkpoint_validation::scrub_streaming::*;

#[test]
fn bounded_windows_validate_literal_selective_aggregates_without_retaining_bytes() {
    let mut validator = PhysicalIntegrityScrubValidator::new();
    prefix(&mut validator);
    assert!(matches!(
        inspect(&mut validator, footer_scope(identity()), &FOOTER),
        PhysicalIntegrityObservationOutcome::Intact(_)
    ));
    assert!(
        matches!(
            inspect(&mut validator, footer_scope(identity()), &FOOTER),
            PhysicalIntegrityObservationOutcome::Rejected(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
            )
        ),
        "footer cannot replay consumed context"
    );
}

#[test]
fn resealed_footer_digest_damage_is_not_a_valid_envelope_shortcut() {
    let mut validator = PhysicalIntegrityScrubValidator::new();
    prefix(&mut validator);
    let mut footer = FOOTER;
    footer[48] ^= 1;
    reseal_record(&mut footer);
    let outcome = inspect(&mut validator, footer_scope(identity()), &footer);
    assert!(
        matches!(outcome, PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage))
        if damage.cause() == worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch
        && damage.damaged_range() == field_range(footer_scope(identity()), 48, 32))
    );
}

#[test]
fn missing_or_failed_middle_window_cannot_borrow_prior_aggregate() {
    let mut validator = PhysicalIntegrityScrubValidator::new();
    assert!(matches!(
        inspect(&mut validator, header_scope_staged(), &HEADER),
        PhysicalIntegrityObservationOutcome::Intact(_)
    ));
    assert!(matches!(
        inspect(
            &mut validator,
            compaction_scope(identity()),
            &BINDING_COMPACTION
        ),
        PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
        )
    ));
    assert!(matches!(
        inspect(&mut validator, footer_scope(identity()), &FOOTER),
        PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
        )
    ));
    prefix(&mut validator);
    validator.invalidate(binding_scope(identity(), BINDING.len() as u64));
    assert!(matches!(
        inspect(&mut validator, footer_scope(identity()), &FOOTER),
        PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
        )
    ));
}

#[test]
fn healthy_omitted_or_other_checkpoint_context_is_unknown_not_damage() {
    let mut validator = PhysicalIntegrityScrubValidator::new();
    inspect(&mut validator, header_scope_staged(), &HEADER);
    assert!(matches!(
        inspect(&mut validator, footer_scope(identity()), &FOOTER),
        PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
        )
    ));
    inspect(&mut validator, header_scope_staged(), &HEADER);
    let other = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store(),
        std::num::NonZeroU64::new(8).unwrap(),
    );
    assert!(matches!(
        inspect(&mut validator, dirty_scope(other), &DIRTY_BASIS),
        PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)
        )
    ));
}
