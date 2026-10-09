use super::*;
use crate::package::WorthQueryPortableDefinition;

fn package_with_semantics(length: usize) -> WorthQueryPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.identity-budget",
        1,
        0,
    ))
    .definition(WorthQueryPortableDefinition::invariant(
        "identity-budget",
        "x".repeat(length),
    ))
}

#[test]
fn package_identity_admits_exact_byte_ceiling_and_denies_one_byte_more() {
    let baseline = package_with_semantics(0);
    let (_, work) =
        canonical_identity_with_maximum_bytes(&baseline, INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .unwrap();
    let mut filler = INSTALLATION_MAXIMUM_CANONICAL_BYTES - work.canonical_encoded_bytes();
    let exact_work = loop {
        match canonical_identity_with_maximum_bytes(
            &package_with_semantics(filler),
            INSTALLATION_MAXIMUM_CANONICAL_BYTES,
        ) {
            Ok((_, work))
                if work.canonical_encoded_bytes() == INSTALLATION_MAXIMUM_CANONICAL_BYTES =>
            {
                break work
            }
            Ok((_, work)) => {
                filler += INSTALLATION_MAXIMUM_CANONICAL_BYTES - work.canonical_encoded_bytes();
            }
            Err(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
                attempted, ..
            }) => {
                filler -= attempted - INSTALLATION_MAXIMUM_CANONICAL_BYTES;
            }
            Err(denial) => panic!("unexpected exact-boundary denial: {denial:?}"),
        }
    };
    assert_eq!(
        exact_work.canonical_encoded_bytes(),
        INSTALLATION_MAXIMUM_CANONICAL_BYTES
    );

    let denial = canonical_identity_with_maximum_bytes(
        &package_with_semantics(filler + 1),
        INSTALLATION_MAXIMUM_CANONICAL_BYTES,
    )
    .unwrap_err();
    assert_eq!(
        denial,
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            maximum: INSTALLATION_MAXIMUM_CANONICAL_BYTES,
            attempted: INSTALLATION_MAXIMUM_CANONICAL_BYTES + 1,
        }
    );
}

#[test]
fn raising_work_ceiling_does_not_change_package_identity() {
    let fixture = package_with_semantics(1024);
    let (old_identity, _) =
        canonical_identity_with_maximum_bytes(&fixture, 4 * 1_024 * 1_024).unwrap();
    let (new_identity, _) =
        canonical_identity_with_maximum_bytes(&fixture, INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .unwrap();
    assert_eq!(old_identity, new_identity);
}

#[test]
fn package_identity_admits_more_than_the_former_entry_ceiling() {
    let mut fixture = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.entry-budget",
        1,
        0,
    ));
    for index in 0..33_000 {
        fixture = fixture.permits_contribution(format!("category-{index}"));
    }
    let validated = fixture.validate().unwrap();
    let work = validated.canonical_work();
    assert_eq!(work.canonical_entries(), 33_003);
    assert!(work.canonical_encoded_bytes() <= INSTALLATION_MAXIMUM_CANONICAL_BYTES);
}

#[test]
fn package_entry_breadth_is_denied_before_canonical_construction() {
    let mut fixture = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.entry-budget",
        1,
        0,
    ));
    let maximum =
        CanonicalDigestWorkBudget::for_encoded_byte_ceiling(INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .unwrap()
            .maximum_entry_count();
    for index in 0..=maximum {
        fixture = fixture.permits_contribution(format!("category-{index}"));
    }
    let denial = fixture.validate().unwrap_err();
    assert_eq!(
        denial.kind(),
        super::super::WorthQueryPortablePackageValidationDenialKind::CanonicalEntryBudgetExceeded,
    );
    assert_eq!(denial.maximum_canonical_entries(), Some(maximum));
    assert_eq!(denial.attempted_canonical_entries(), Some(maximum + 4));
}
