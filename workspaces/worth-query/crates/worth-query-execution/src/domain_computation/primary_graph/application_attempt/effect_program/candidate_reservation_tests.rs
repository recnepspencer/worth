use super::{CandidateItemKind, WorthQueryCandidateReservation};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling,
};

#[test]
fn capacity_denies_before_a_reservation_exists() {
    let ceiling = requirements(1, 1, 64, 8);
    let denial =
        WorthQueryCandidateReservation::admit(requirements(2, 1, 64, 8), ceiling, 8, 16, 64, 8)
            .err()
            .expect("the binding ceiling must deny excess creation");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
    );
}

#[test]
fn admitted_reservation_charges_each_effect_family_exactly() {
    let requirements = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(1, 1, 1, 1, 1, 1),
        ApplicationCandidateResourceCeiling::bounded(64, 8),
    );
    let mut reservation =
        WorthQueryCandidateReservation::admit(requirements, requirements, 8, 6, 64, 8)
            .expect("the exact finite reservation is admitted");
    for kind in [
        CandidateItemKind::Create,
        CandidateItemKind::Delete,
        CandidateItemKind::Link,
        CandidateItemKind::Unlink,
        CandidateItemKind::Write,
        CandidateItemKind::Emit,
    ] {
        reservation
            .charge(kind)
            .expect("the reserved item is available");
        let denial = reservation
            .charge(kind)
            .expect_err("the same family cannot exceed its reservation");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
        );
    }
}

#[test]
fn equal_cardinality_payload_twin_denies_bytes_without_consuming_reservation() {
    let small_value = worth_foundational::facade::AspectValue::UInt8(7);
    let oversized_value = worth_foundational::facade::AspectValue::String("large".into());
    let candidate_bytes = small_value.semantic_byte_width();
    assert!(oversized_value.semantic_byte_width() > candidate_bytes);
    let requirement = requirements(0, 1, candidate_bytes, 8);
    let runtime_bytes = u64::try_from(candidate_bytes).unwrap();
    let mut small =
        WorthQueryCandidateReservation::admit(requirement, requirement, 8, 1, runtime_bytes, 8)
            .expect("the small candidate reservation is admitted");
    small
        .charge_retained_representation(
            CandidateItemKind::Write,
            small_value.semantic_byte_width(),
            0,
        )
        .expect("the equal-cardinality small value fits");

    let mut oversized =
        WorthQueryCandidateReservation::admit(requirement, requirement, 8, 1, runtime_bytes, 8)
            .expect("the oversized twin starts from the same reservation");
    let denial = oversized
        .charge_retained_representation(
            CandidateItemKind::Write,
            oversized_value.semantic_byte_width(),
            0,
        )
        .expect_err("the equal-cardinality oversized value must be denied");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    oversized
        .charge_retained_representation(
            CandidateItemKind::Write,
            small_value.semantic_byte_width(),
            0,
        )
        .expect("denial retained no candidate and consumed no reservation");
}

#[test]
fn coalesced_write_replaces_its_retained_semantic_width() {
    let first = worth_foundational::facade::AspectValue::UInt64(7);
    let replacement = worth_foundational::facade::AspectValue::String("12345678".into());
    assert_eq!(
        first.semantic_byte_width(),
        replacement.semantic_byte_width()
    );
    let bytes = first.semantic_byte_width();
    let requirement = requirements(0, 2, bytes, 8);
    let mut reservation = WorthQueryCandidateReservation::admit(
        requirement,
        requirement,
        8,
        2,
        u64::try_from(bytes).unwrap(),
        8,
    )
    .expect("both writes share one retained field slot");
    reservation
        .charge_retained_representation(CandidateItemKind::Write, first.semantic_byte_width(), 0)
        .unwrap();
    reservation
        .charge_retained_representation(
            CandidateItemKind::Write,
            replacement.semantic_byte_width(),
            first.semantic_byte_width(),
        )
        .expect("replacement is charged once for the retained field slot");
}

fn requirements(
    creates: usize,
    writes: usize,
    bytes: usize,
    validator_work: usize,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(creates, 0, 0, 0, writes, 0),
        ApplicationCandidateResourceCeiling::bounded(bytes, validator_work),
    )
}

#[test]
fn derived_validator_work_obeys_host_and_deliberate_caps() {
    let automatic = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    let admit = |request, ceiling, derived, host| {
        WorthQueryCandidateReservation::admit(request, ceiling, derived, 1, 64, host)
    };
    let first = admit(automatic, automatic, 33, 40).unwrap();
    assert_eq!(first.validator_work_admission().maximum_work(), Some(33));
    // An installed invariant can increase its bound without editing the caller.
    let revised = admit(automatic, automatic, 39, 40).unwrap();
    assert_eq!(revised.validator_work_admission().maximum_work(), Some(39));
    assert!(admit(automatic, automatic, 41, 40).is_err());
    let restricted = requirements(0, 1, 64, 7);
    assert_eq!(
        admit(automatic, restricted, 33, 40)
            .unwrap()
            .validator_work_admission()
            .maximum_work(),
        Some(7)
    );
    assert_eq!(
        admit(restricted, automatic, 33, 40)
            .unwrap()
            .validator_work_admission()
            .maximum_work(),
        Some(7)
    );
    assert!(admit(requirements(0, 1, 64, 8), restricted, 33, 40).is_err());
}

#[test]
fn handler_caps_cannot_inflate_the_installed_validator_allowance() {
    let automatic = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    let requested = requirements(0, 1, 64, 50);
    for binding in [automatic, requirements(0, 1, 64, 100)] {
        let admitted = WorthQueryCandidateReservation::admit(requested, binding, 33, 1, 64, 33)
            .expect("a cap above installed need neither widens nor wastes host capacity");
        assert_eq!(admitted.validator_work_admission().maximum_work(), Some(33));
    }
    assert!(
        WorthQueryCandidateReservation::admit(requested, requirements(0, 1, 64, 40), 33, 1, 64, 33)
            .is_err(),
        "an explicit request still cannot exceed the binding cap"
    );
}
