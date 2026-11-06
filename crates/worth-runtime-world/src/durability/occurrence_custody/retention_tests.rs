//! Pure prepaid-buffer/name ownership, not Native or physical publication proof.
use super::*;
use worth_foundational::facade::*;

fn open_description() -> WorldOpenAttemptRecord {
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(0),
    );
    let branch =
        DurableProductBranchIdentity::from_recorded(owner, "untrusted-value".into()).unwrap();
    let incarnation = DurableProductBranchIncarnation::from_recorded(owner, 1);
    WorldOpenAttemptRecord {
        component_intent: super::super::WorldOpenComponentIntent::RelationalOnly,
        association: CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, 1),
            branch.clone(),
            incarnation,
        )
        .unwrap(),
        commit: DurableCompositeCommitIdentity::from_recorded(owner, 2),
        custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, 1),
        expected: super::super::DurableWorldHead {
            branch,
            incarnation,
            generation: DurableProductReferenceGeneration::from_recorded(0),
            commit: DurableCompositeCommitIdentity::from_recorded(owner, 1),
        },
        program_revision: [3; 32],
    }
}

#[test]
fn occurrence_name_loans_restore_the_same_prepaid_open_when_filling_unwinds() {
    let description = open_description();
    let mut destination = Some(description.clone());
    let name = destination
        .as_ref()
        .unwrap()
        .association
        .branch()
        .name()
        .as_ptr();
    let expected_name = destination
        .as_ref()
        .unwrap()
        .expected
        .branch
        .name()
        .as_ptr();
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let loan = OpenRecordLoan::take(
            &mut destination,
            WorldRelationalEvidence {
                commit: 1,
                position: 1,
                digest: [5; 32],
            },
            HomeIncarnation::initial(),
        );
        assert!(matches!(loan.record(), WorldRecord::Occurrence { .. }));
        panic!("exercise actual loan unwinding without a Native or journal effect");
    }));
    assert!(unwound.is_err());
    assert_eq!(destination.as_ref(), Some(&description));
    assert_eq!(
        destination
            .as_ref()
            .unwrap()
            .association
            .branch()
            .name()
            .as_ptr(),
        name
    );
    assert_eq!(
        destination
            .as_ref()
            .unwrap()
            .expected
            .branch
            .name()
            .as_ptr(),
        expected_name
    );
}

#[test]
fn a_consumed_occurrence_destination_refusal_requires_reopen() {
    let mut posture = super::byte_destination::DestinationPosture::Available;
    posture.consume().unwrap();
    assert!(matches!(
        posture,
        super::byte_destination::DestinationPosture::Consumed
    ));
    assert!(matches!(
        posture.consume(),
        Err(WorldDurableMediumDenial::OccurrenceReopenRequired)
    ));
    assert!(matches!(
        posture.consume(),
        Err(WorldDurableMediumDenial::OccurrenceReopenRequired)
    ));
}
