use super::*;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{DependencyRevision, OutputCommitOrdinal};
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use std::collections::BTreeMap;

fn cause(key: u32, ordinal: u64) -> Cause {
    Cause::new(
        1,
        NodeId::new(1, 0),
        DependencyRevision(1),
        NodeId::new(key, 0),
        crate::data::aspect::Aspect::new(1),
        Some(
            crate::data::output::PartitionSubscription::partition_and_detail(
                "scope".repeat(1000),
                "detail",
            ),
        ),
        0,
        OutputCommitOrdinal(ordinal),
        1,
        Default::default(),
    )
}
#[test]
fn normalization_preserves_first_duplicate_and_shared_work() {
    for seed in 0..32u32 {
        let input: Vec<_> = (0..24u32)
            .map(|i| cause((i * 13 + seed) % 9, u64::from(i + 1)))
            .collect();
        let mut oracle = BTreeMap::new();
        for cause in &input {
            oracle
                .entry(cause.key.clone())
                .or_insert_with(|| cause.clone());
        }
        let expected: Vec<_> = oracle.into_values().collect();
        let mut measured = Work::new(usize::MAX);
        let actual = NormalizedCauseSet::prepare(
            input.clone(),
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
        assert_eq!(&*actual, expected.as_slice());
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let mut work = Work::new(cost + 9);
            work.reserve_visits(cost + 9 - available).unwrap();
            let result = NormalizedCauseSet::prepare(
                input.clone(),
                &mut EvaluationWork::Conditional(&mut work),
            );
            if available == cost {
                assert_eq!(&*result.unwrap(), expected.as_slice());
                assert_eq!(work.visits(), cost + 9);
            } else {
                assert!(matches!(
                    result,
                    Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
                ));
            }
        }
    }
}
#[test]
fn canonical_input_keeps_its_allocation() {
    let input = vec![cause(1, 1), cause(2, 2)];
    let pointer = input.as_ptr();
    let capacity = input.capacity();
    let normalized = NormalizedCauseSet::prepare(input, &mut EvaluationWork::Ordinary)
        .unwrap()
        .into_vec();
    assert_eq!(normalized.as_ptr(), pointer);
    assert_eq!(normalized.capacity(), capacity);
    assert!(
        NormalizedCauseSet::prepare(Vec::new(), &mut EvaluationWork::Ordinary)
            .unwrap()
            .is_empty()
    );
}
