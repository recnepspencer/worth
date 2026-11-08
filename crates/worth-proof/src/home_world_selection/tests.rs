use super::*;
fn owner(home: u8, incarnation: u64) -> HomeWorldSelectionOwner {
    HomeWorldSelectionOwner::from_retained_home_owner(
        [home; 16],
        HomeIncarnationFence::from_owner(std::num::NonZeroU64::new(incarnation).unwrap()),
        Arc::new(()),
    )
}
#[test]
fn world_source_and_completion_require_the_same_home_owner_and_incarnation() {
    let original = owner(1, 1);
    let current = original.authority();
    let (source, completion) = original.admit_source(37);
    let verified = current.verify_source(source).unwrap();
    assert_eq!(*verified.source(), 37);
    assert_eq!(current.admit_completion(&completion.complete()), Ok(()));
    let (foreign, completion) = owner(2, 1).admit_source(37);
    assert_eq!(
        current.verify_source(foreign).unwrap_err(),
        HomeWorldSelectionAdmissionDenial::ForeignOwner
    );
    assert_eq!(
        current.admit_completion(&completion.complete()),
        Err(HomeWorldSelectionAdmissionDenial::ForeignOwner)
    );
    let (same_home_foreign_owner, _) = owner(1, 1).admit_source(37);
    assert_eq!(
        current.verify_source(same_home_foreign_owner).unwrap_err(),
        HomeWorldSelectionAdmissionDenial::ForeignOwner
    );
    let next = owner(1, 2).authority();
    let prior = owner(1, 1);
    let (source, completion) = prior.admit_source(37);
    assert_eq!(
        next.verify_source(source).unwrap_err(),
        HomeWorldSelectionAdmissionDenial::StaleIncarnation
    );
    assert_eq!(
        next.admit_completion(&completion.complete()),
        Err(HomeWorldSelectionAdmissionDenial::StaleIncarnation)
    );
}

#[test]
fn world_source_eligibility_requires_the_original_shared_source_instead_of_equal_data() {
    let issuer = owner(1, 1);
    let authority = issuer.authority();
    let original = Arc::new(String::from("protected original source"));
    let equal_copy = Arc::new(original.as_ref().clone());
    let (admitted, _) = issuer.admit_source(original.clone());
    let verified = authority.verify_source(admitted).unwrap();
    assert_eq!(verified.require_original_source(&original.clone()), Ok(()));
    assert_eq!(
        verified.require_original_source(&equal_copy),
        Err(HomeWorldSelectionAdmissionDenial::ForeignSource)
    );
}
