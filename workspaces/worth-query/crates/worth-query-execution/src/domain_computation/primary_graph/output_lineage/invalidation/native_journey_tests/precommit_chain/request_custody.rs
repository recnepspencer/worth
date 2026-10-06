//! Request custody through two real retained native dependency closures.

use super::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop, InvalidationEditAdmission,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(super) fn assert_carried_native_verification(
    owner: &SourceInvalidationOwner,
    two_closures_work: u64,
    verify: impl Fn(
        &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
) {
    let retained_before = owner.resources.retained_capacity_bytes();
    let mut request = owner.read_admission(usize::try_from(two_closures_work + 12).unwrap());
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
