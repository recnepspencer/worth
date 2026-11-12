use super::*;
use crate::CanonicalDigestId;

#[test]
fn an_uninitialized_home_address_is_not_a_home_identity() {
    assert_eq!(
        HomeIdentity::from_observed_bytes([0; 16]),
        Err(HomeIdentityDenial::Zero)
    );
    let bytes = [7; 16];
    assert_eq!(
        HomeIdentity::from_observed_bytes(bytes).unwrap().bytes(),
        bytes
    );
}

#[test]
fn recorded_incarnations_are_nonzero_and_exhaustion_never_wraps() {
    assert_eq!(HomeIncarnation::from_recorded(0), None);
    assert_eq!(HomeIncarnation::initial().successor().unwrap().get(), 2);
    assert_eq!(
        HomeIncarnation::from_recorded(u64::MAX)
            .unwrap()
            .successor(),
        Err(HomeIncarnationExhausted)
    );
}

#[test]
fn durable_owner_tags_keep_the_declared_wire_assignments() {
    let assignments = [
        (DurableOwnerFamily::Relational, 1),
        (DurableOwnerFamily::Signal, 2),
        (DurableOwnerFamily::Bridge, 3),
        (DurableOwnerFamily::World, 4),
        (DurableOwnerFamily::Query, 5),
        (DurableOwnerFamily::Dispatch, 6),
        (DurableOwnerFamily::Correction, 7),
    ];
    for (family, tag) in assignments {
        assert_eq!(family.tag(), tag);
        assert_eq!(DurableOwnerFamily::from_tag(tag), Ok(family));
    }
    for observed in [0, 8, u16::MAX] {
        assert_eq!(
            DurableOwnerFamily::from_tag(observed),
            Err(UnsupportedDurableOwnerTag { observed })
        );
    }
}

#[test]
fn a_home_binding_has_one_canonical_roster_and_cannot_select_an_absent_program() {
    let earlier = HomeProgramBinding::new("earlier".into(), CanonicalDigestId::new([1; 32]));
    let later = HomeProgramBinding::new("later".into(), CanonicalDigestId::new([2; 32]));
    let package = CanonicalDigestId::new([3; 32]);
    let schema = CanonicalDigestId::new([4; 32]);
    let binding = ApplicationHomeBinding::new(
        package,
        schema,
        later.clone(),
        vec![later.clone(), earlier.clone()],
    )
    .unwrap();
    assert_eq!(binding.roster(), &[earlier.clone(), later.clone()]);
    assert_eq!(binding.program(later.revision()), Some(&later));
    assert_eq!(
        ApplicationHomeBinding::new(package, schema, later.clone(), vec![earlier.clone()]),
        Err(ApplicationHomeBindingDescriptionDenial::SelectedProgramNotRostered)
    );
    assert_eq!(
        ApplicationHomeBinding::new(
            package,
            schema,
            earlier.clone(),
            vec![earlier.clone(), earlier]
        ),
        Err(ApplicationHomeBindingDescriptionDenial::DuplicateRevision)
    );
}
