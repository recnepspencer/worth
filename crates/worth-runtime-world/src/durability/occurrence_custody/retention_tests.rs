//! Pure prepaid-buffer/name ownership, not Native or physical publication proof.
use super::*;
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionPhysicalAllocationDenialKind,
    ExecutionRequest, SerialMemoryBudget, SerialRequest,
};
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
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    let (prepared, open) =
        super::super::PreparedWorldArtifact::prepare_occurrence(&description, &request).unwrap();
    let charged = budget.charged_memory_bytes();
    assert!(charged > 0);
    let mut destination = Some(open);
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
    assert_eq!(budget.charged_memory_bytes(), charged);
    drop(destination);
    drop(prepared);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn a_consumed_occurrence_destination_refusal_requires_reopen_and_releases_its_byte_grant() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    let mut destination = OccurrenceByteDestination::prepare(2, &request).unwrap();
    assert!(budget.charged_memory_bytes() >= 2);
    destination
        .backing()
        .unwrap()
        .extend_from_slice(&[1])
        .unwrap();
    destination.backing().unwrap().check_live().unwrap();
    let refused = destination.seal().unwrap_err();
    match refused {
        WorldDurableMediumDenial::Allocation(cause) => {
            assert_eq!(
                cause.classify(),
                ExecutionAllocationDenialClass::Physical(
                    ExecutionPhysicalAllocationDenialKind::IncompleteSeal
                )
            )
        }
        other => panic!("the actual consuming seal must retain its precise cause: {other:?}"),
    }
    assert!(matches!(
        destination.backing(),
        Err(WorldDurableMediumDenial::OccurrenceReopenRequired)
    ));
    assert!(matches!(
        destination.seal(),
        Err(WorldDurableMediumDenial::OccurrenceReopenRequired)
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
