//! Request custody through two real retained native dependency closures.

use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::{
    ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    InvalidationEditAdmission,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;

/// The least meter on which two closures both answer exceeds the least on
/// which one does, in work and in scratch: the second closure's comparison
/// draws from what the first spent. A fresh allowance per closure would
/// answer both on the least meter for one. Recording rows comes after the
/// answer, so the meter one short of the least for two stops the second
/// closure's comparison itself.
pub(super) fn assert_closures_share_the_meter(
    two_closures_work: u64,
    two_closures_bytes: u64,
    verify: impl Fn(
        &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
) {
    let meter = |work, bytes| {
        InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: work,
            maximum_preparation_bytes: bytes,
        })
    };
    let answers = |mut meter: InvalidationEditAdmission, closures: usize| {
        (0..closures).all(|_| verify(&mut meter) == Ok(ConsumedOutputVerification::Current))
    };
    let least_work = |closures| {
        least(two_closures_work, |work| {
            answers(meter(work, two_closures_bytes), closures)
        })
    };
    let least_bytes = |closures| {
        least(two_closures_bytes, |bytes| {
            answers(meter(two_closures_work, bytes), closures)
        })
    };
    let (one_work, two_work) = (least_work(1), least_work(2));
    let (one_bytes, two_bytes) = (least_bytes(1), least_bytes(2));
    assert!(two_work > one_work && two_bytes > one_bytes);
    for (mut short, stop) in [
        (
            meter(two_work - 1, two_closures_bytes),
            ConsumedOutputVerificationStop::WorkExhausted,
        ),
        (
            meter(two_closures_work, two_bytes - 1),
            ConsumedOutputVerificationStop::Unavailable,
        ),
    ] {
        assert_eq!(verify(&mut short), Ok(ConsumedOutputVerification::Current));
        assert_eq!(verify(&mut short), Err(stop));
    }
}

/// The least of `1..=maximum` that `answers`, which only more can satisfy.
fn least(maximum: u64, answers: impl Fn(u64) -> bool) -> u64 {
    assert!(answers(maximum));
    let (mut low, mut high) = (1, maximum);
    while low < high {
        let middle = low + (high - low) / 2;
        if answers(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

pub(super) fn assert_carried_native_verification(
    owner: &SourceInvalidationOwner,
    two_closures_work: u64,
    verify: impl Fn(
        &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
) {
    let retained_before = owner.resources.retained_capacity_bytes();
    let mut request = owner.edit_admission_within(
        std::num::NonZeroUsize::new(usize::try_from(two_closures_work + 12).unwrap()).unwrap(),
    );
    request.charge_external_work(5).unwrap();
    let mut publication = request
        .carry_for_publication(owner)
        .unwrap()
        .into_admission();
    assert_eq!(
        verify(&mut publication),
        Ok(ConsumedOutputVerification::Current)
    );
    assert!(
        request.charged_work() > 6,
        "native verification spends the carried request"
    );
    request.charge_external_work(6).unwrap();
    assert_eq!(
        verify(&mut publication),
        Ok(ConsumedOutputVerification::Current)
    );
    assert_eq!(request.charged_work(), two_closures_work + 12);
    assert!(matches!(
        publication.charge_external_work(1),
        Err(CompanionPreflightStop::WorkExhausted { .. })
    ));
    assert_eq!(
        request.remaining_work(),
        0,
        "later preparation cannot remint the spent allowance"
    );
    drop(publication);
    assert!(owner.resources.retained_capacity_bytes() > retained_before);
    drop(request);
    assert_eq!(owner.resources.retained_capacity_bytes(), retained_before);
}

/// The backing consumed edges are held in is paid by the meter the request
/// hands it, never by a fresh allowance: the request's meter is charged its
/// bytes in work, and one a unit short stops for work.
pub(super) fn assert_backing_paid_by_the_request(
    owner: &SourceInvalidationOwner,
    edges: &[ConsumedOutputEvidence],
) {
    let admit = |meter: &mut InvalidationEditAdmission| {
        ConsumedOutputEvidence::admit_backing(&mut edges.to_vec(), owner, meter)
    };
    let mut request = owner.edit_admission();
    assert_eq!(admit(&mut request), Ok(()));
    let (work, bytes) = (request.charged_work(), request.charged_bytes());
    assert!(work > 0, "the request's meter pays the backing");
    let meter = |work| {
        InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: work,
            maximum_preparation_bytes: bytes,
        })
    };
    assert_eq!(admit(&mut meter(work)), Ok(()));
    assert_eq!(
        admit(&mut meter(work - 1)),
        Err(ConsumedOutputVerificationStop::WorkExhausted)
    );
}
