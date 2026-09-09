use super::*;
use crate::data::retained_storage::RetainedStoragePreparation as Preparation;

#[test]
fn candidate_normalization_matches_ordered_set_and_denies_before_mutation() {
    for count in 0..128 {
        let input: Vec<_> = (0..count)
            .map(|n| NodeId::new((n * 37 % 29) as u32, (n % 3) as u32))
            .collect();
        let expected: Vec<_> = input
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut work = Preparation::new(1_000_000);
        let mut actual = input.clone();
        normalize(&mut actual, &mut EvaluationWork::Conditional(&mut work)).unwrap();
        assert_eq!(actual, expected);
        let cost = work.visits();
        let mut exact = input.clone();
        normalize(
            &mut exact,
            &mut EvaluationWork::Conditional(&mut Preparation::new(cost)),
        )
        .unwrap();
        assert_eq!(exact, expected);
        if cost > 0 {
            let mut denied = input.clone();
            assert!(matches!(
                normalize(
                    &mut denied,
                    &mut EvaluationWork::Conditional(&mut Preparation::new(cost - 1))
                ),
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
            assert_eq!(denied, input);
        }
    }
}

#[test]
fn candidate_growth_denies_overflow_and_exhaustion_before_reserving_storage() {
    let mut candidates = vec![NodeId::new(3, 2)];
    let capacity = candidates.capacity();
    for count in [10, usize::MAX] {
        let mut work = Preparation::new(0);
        assert!(matches!(
            reserve_additional(
                &mut candidates,
                count,
                &mut EvaluationWork::Conditional(&mut work)
            ),
            Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits: 0 })
        ));
        assert_eq!(candidates, [NodeId::new(3, 2)]);
        assert_eq!(candidates.capacity(), capacity);
        assert_eq!(work.visits(), 0);
    }
}
