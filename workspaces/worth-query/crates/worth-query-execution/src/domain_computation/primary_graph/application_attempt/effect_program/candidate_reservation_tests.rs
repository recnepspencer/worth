use super::{CandidateItemKind, WorthQueryCandidateReservation};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidatePolicy,
    ApplicationCandidateRequirements, ApplicationCandidateResourceCeiling,
};

#[test]
fn absent_policy_still_reserves_actual_kinds_and_representation() {
    for requested in [requirements(2, 3, 7), requirements(5, 1, 19)] {
        let mut reservation = WorthQueryCandidateReservation::admit(
            requested,
            ApplicationCandidatePolicy::new(None, None),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            reservation.total_items(),
            requested.cardinality().maximum_creates() + requested.cardinality().maximum_writes()
        );
        let denied = reservation
            .charge_retained_representation(
                CandidateItemKind::Write,
                requested
                    .resources()
                    .maximum_retained_representation_bytes()
                    + 1,
                0,
            )
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
        );
        reservation
            .charge_many(
                CandidateItemKind::Create,
                requested.cardinality().maximum_creates(),
                0,
                0,
            )
            .unwrap();
        reservation
            .charge_retained_representation(
                CandidateItemKind::Write,
                requested
                    .resources()
                    .maximum_retained_representation_bytes(),
                0,
            )
            .unwrap();
        assert!(reservation.charge(CandidateItemKind::Create).is_err());
        assert!(reservation.charge_representation_only(1).is_err());
    }
}

#[test]
fn independent_finite_declaration_policies_and_checked_count_overflow_refuse() {
    let requested = requirements(3, 2, 7);
    for policy in [
        ApplicationCandidatePolicy::new(Some(requirements(2, 2, 7).cardinality()), None),
        ApplicationCandidatePolicy::new(
            None,
            Some(ApplicationCandidateResourceCeiling::representation_bytes(6)),
        ),
    ] {
        let denied = WorthQueryCandidateReservation::admit(requested, policy, None, None)
            .err()
            .unwrap();
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
        );
    }
    let overflowing = requirements(usize::MAX, 1, 0);
    let denied = WorthQueryCandidateReservation::admit(
        overflowing,
        ApplicationCandidatePolicy::new(None, None),
        None,
        None,
    )
    .err()
    .unwrap();
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
    );
}

#[test]
fn capacity_denies_before_a_reservation_exists() {
    let ceiling = requirements(1, 1, 64);
    let denial = WorthQueryCandidateReservation::admit(
        requirements(2, 1, 64),
        worth_query_declaration::facade::application_operation::ApplicationCandidatePolicy::finite(
            ceiling,
        ),
        Some(16),
        Some(64),
    )
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
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    let mut reservation = WorthQueryCandidateReservation::admit(
        requirements,
        worth_query_declaration::facade::application_operation::ApplicationCandidatePolicy::finite(
            requirements,
        ),
        Some(6),
        Some(64),
    )
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
    let requirement = requirements(0, 1, candidate_bytes);
    let runtime_bytes = u64::try_from(candidate_bytes).unwrap();
    let mut small = WorthQueryCandidateReservation::admit(
        requirement,
        worth_query_declaration::facade::application_operation::ApplicationCandidatePolicy::finite(
            requirement,
        ),
        Some(1),
        Some(runtime_bytes),
    )
    .expect("the small candidate reservation is admitted");
    small
        .charge_retained_representation(
            CandidateItemKind::Write,
            small_value.semantic_byte_width(),
            0,
        )
        .expect("the equal-cardinality small value fits");

    let mut oversized = WorthQueryCandidateReservation::admit(
        requirement,
        worth_query_declaration::facade::application_operation::ApplicationCandidatePolicy::finite(
            requirement,
        ),
        Some(1),
        Some(runtime_bytes),
    )
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
    let requirement = requirements(0, 2, bytes);
    let mut reservation = WorthQueryCandidateReservation::admit(
        requirement,
        worth_query_declaration::facade::application_operation::ApplicationCandidatePolicy::finite(
            requirement,
        ),
        Some(2),
        Some(u64::try_from(bytes).unwrap()),
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

fn requirements(creates: usize, writes: usize, bytes: usize) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(creates, 0, 0, 0, writes, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(bytes),
    )
}
