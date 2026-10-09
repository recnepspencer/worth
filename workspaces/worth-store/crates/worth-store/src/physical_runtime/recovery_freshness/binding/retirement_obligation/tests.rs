use crate::physical_runtime::durability::{RetiredArtifact, RetirementRecord};

use super::super::StoreRecoveryBindingSampleDenial as Denial;
use super::{fold_retirement_records, StoreRecoveryRetiredArtifact};

// Local semantic vectors are not native funding evidence.
fn fold(records: Vec<RetirementRecord>) -> Vec<super::StoreRecoveryRetirementObligation> {
    let output = Vec::with_capacity(records.len());
    fold_retirement_records(records.into_iter().enumerate().collect(), output).unwrap()
}

#[test]
fn completion_removes_the_reconstructed_obligation() {
    let intent = RetirementRecord {
        artifact: RetiredArtifact::Segment {
            segment: 1,
            generation: 7,
        },
        completion: false,
        source_root: 3,
        bytes: 32,
        release: None,
    };
    let completion = RetirementRecord {
        completion: true,
        ..intent
    };
    assert_eq!(fold(vec![intent]).len(), 1);
    assert!(fold(vec![intent, completion]).is_empty());
}

fn range() -> worth_store_physical_format::ExtentArenaRange {
    worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
        4096,
        8192,
    )
    .unwrap()
}

#[test]
fn an_extent_obligation_keeps_its_artifact_kind() {
    let intent = RetirementRecord {
        artifact: RetiredArtifact::Extent {
            extent: 1,
            generation: 7,
            range: range(),
        },
        completion: false,
        source_root: 3,
        bytes: 32,
        release: Some(
            crate::physical_runtime::durability::RetirementReleaseProjection::new(
                4, 5, [7; 32], 4096, 9,
            )
            .unwrap(),
        ),
    };
    let obligations = fold(vec![intent]);
    assert_eq!(
        obligations[0].artifact(),
        StoreRecoveryRetiredArtifact::Extent {
            extent: 1,
            generation: 7,
            range: range()
        }
    );
    assert_eq!(obligations[0].artifact().segment(), None);
}

#[test]
fn first_intent_survives_duplicates_until_exact_completion_then_reintent_wins() {
    let first = RetirementRecord {
        artifact: RetiredArtifact::Segment {
            segment: 1,
            generation: 7,
        },
        completion: false,
        source_root: 3,
        bytes: 32,
        release: None,
    };
    let duplicate = RetirementRecord {
        source_root: 4,
        ..first
    };
    let wrong_completion = RetirementRecord {
        completion: true,
        ..duplicate
    };
    let prefix = fold(vec![first, duplicate, wrong_completion]);
    assert_eq!(prefix.len(), 1);
    assert_eq!(prefix[0].source_root(), 3);
    let completion = RetirementRecord {
        completion: true,
        ..first
    };
    let replacement = RetirementRecord {
        source_root: 5,
        bytes: 64,
        ..first
    };
    let output = fold(vec![
        wrong_completion,
        first,
        duplicate,
        completion,
        replacement,
    ]);
    assert_eq!(output.len(), 1);
    assert_eq!((output[0].source_root(), output[0].bytes()), (5, 64));
}

#[test]
fn artifact_sort_preserves_per_artifact_arrival_order_and_output_backing() {
    let first = RetirementRecord {
        artifact: RetiredArtifact::Segment {
            segment: 1,
            generation: 7,
        },
        completion: false,
        source_root: 3,
        bytes: 32,
        release: None,
    };
    let second = RetirementRecord {
        artifact: RetiredArtifact::Segment {
            segment: 2,
            generation: 7,
        },
        ..first
    };
    // Physical collection order can differ from the carried arrival ordinal.
    let records = vec![
        (
            2,
            RetirementRecord {
                completion: true,
                ..first
            },
        ),
        (1, first),
        (0, second),
    ];
    let output = Vec::with_capacity(records.len());
    let pointer = output.as_ptr();
    let capacity = output.capacity();
    let output = fold_retirement_records(records, output).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].artifact().segment(), Some(2));
    assert_eq!(output.as_ptr(), pointer);
    assert_eq!(output.capacity(), capacity);
    let output = fold(vec![second, first]);
    assert_eq!(
        output
            .iter()
            .map(|item| item.artifact().segment())
            .collect::<Vec<_>>(),
        [Some(1), Some(2)]
    );
}

#[test]
fn prepared_output_must_be_empty_and_cover_the_whole_record_roster() {
    let record = RetirementRecord {
        artifact: RetiredArtifact::Segment {
            segment: 1,
            generation: 7,
        },
        completion: false,
        source_root: 3,
        bytes: 32,
        release: None,
    };
    assert_eq!(
        fold_retirement_records(vec![(0, record)], Vec::new()),
        Err(Denial::RecoveryMemoryLimit)
    );
    assert_eq!(
        fold_retirement_records(Vec::new(), fold(vec![record])),
        Err(Denial::InvalidWalMember)
    );
    assert!(fold_retirement_records(Vec::new(), Vec::new())
        .unwrap()
        .is_empty());
}
