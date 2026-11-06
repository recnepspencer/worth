//! Value protocol checks; these descriptions deliberately grant no authority.
use super::*;
use crate::facade::*;

fn plan() -> HomeProgramTransitionPlanDescription {
    let home = HomeIdentity::from_observed_bytes([1; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    HomeProgramTransitionPlanDescription::from_recorded(
        HomeProgramTransitionIdentity::from_recorded(home, NonZeroU64::new(3).unwrap()),
        DurableProductBranchReference::from_recorded(
            DurableProductBranchIdentity::from_recorded(owner, "main".to_owned()).unwrap(),
            DurableProductBranchIncarnation::from_recorded(owner, 1),
            DurableProductReferenceGeneration::from_recorded(2),
            DurableCompositeCommitIdentity::from_recorded(owner, 4),
        )
        .unwrap(),
        HomeProgramTransitionPlanDigest::from_recorded([5; 32]),
        3,
        HomeProgramTransitionTargetMeaning::from_recorded(
            1,
            HomeProgramTransitionCatalogDigest::from_recorded([7; 32]),
        ),
    )
    .unwrap()
}

fn frame(step: u32) -> Vec<u8> {
    let plan = plan();
    let source = plan.predecessor();
    let original = CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(source.branch().owner(), 9),
        source.branch().clone(),
        source.incarnation(),
    )
    .unwrap();
    let described = HomeProgramTransitionCaller::for_step(
        &plan,
        HomeProgramTransitionStepAssociation::from_recorded(plan.identity(), step),
        &original,
        b"owner-encoded original",
    )
    .unwrap();
    [described.header().as_slice(), described.original()].concat()
}

#[test]
fn home_step_envelopes_preserve_the_original_bytes_and_distinguish_all_execution_roles() {
    let expected = [
        HomeProgramTransitionStepRole::BeforeTarget,
        HomeProgramTransitionStepRole::Target,
        HomeProgramTransitionStepRole::AfterTarget,
    ];
    for (step, role) in expected.into_iter().enumerate() {
        let bytes = frame(step as u32);
        let read = HomeProgramTransitionCaller::read(&bytes).unwrap();
        assert_eq!(read.step().step(), step as u32);
        assert_eq!(read.role(), role);
        assert_eq!(read.catalog(), plan().target_meaning().catalog());
        assert_eq!(read.original(), b"owner-encoded original");
        assert_eq!(read.header().as_slice(), &bytes[..HEADER]);
    }
    let first = frame(0);
    let second = frame(1);
    assert_ne!(
        HomeProgramTransitionCaller::read(&first)
            .unwrap()
            .commitment(),
        HomeProgramTransitionCaller::read(&second)
            .unwrap()
            .commitment()
    );
    assert_eq!(
        plan().step_role(3),
        Err(HomeProgramTransitionDescriptionDenial::StepOutOfRange)
    );
}

#[test]
fn unsupported_home_step_versions_are_recognized_and_refused_in_the_same_protocol_family() {
    let mut bytes = frame(1);
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    assert!(HomeProgramTransitionCaller::recognizes(&bytes));
    assert!(matches!(
        HomeProgramTransitionCaller::read(&bytes),
        Err(HomeProgramTransitionCallerDenial::UnsupportedVersion(_))
    ));
}

#[test]
fn malformed_home_step_roles_lengths_and_zero_identities_never_parse_as_an_original() {
    for end in 0..HEADER {
        assert!(HomeProgramTransitionCaller::read(&frame(1)[..end]).is_err());
    }
    for (start, end) in [(12, 28), (28, 36), (105, 113)] {
        let mut bytes = frame(1);
        bytes[start..end].fill(0);
        assert!(HomeProgramTransitionCaller::read(&bytes).is_err());
    }
    let mut role = frame(1);
    role[40] = 3;
    assert_eq!(
        HomeProgramTransitionCaller::read(&role),
        Err(HomeProgramTransitionCallerDenial::Malformed)
    );
    let mut trailing = frame(1);
    trailing.push(0);
    assert_eq!(
        HomeProgramTransitionCaller::read(&trailing),
        Err(HomeProgramTransitionCallerDenial::Length)
    );
}
