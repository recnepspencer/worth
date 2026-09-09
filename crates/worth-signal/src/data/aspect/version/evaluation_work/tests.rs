use super::*;
use crate::data::aspect::Aspect;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

fn version(n: u64) -> AspectVersion {
    AspectVersion::zero().with(Aspect::new(0), n)
}

fn fixture(unrelated: &str) -> PartitionVersionOverrides {
    let mut values = PartitionVersionOverrides::default();
    for partition in ["selected", unrelated] {
        for detail in ["first", "second", "third"] {
            values.apply_evaluation(
                version(1),
                &[ChangedRegion::new(partition).with_detail(detail)],
            );
        }
    }
    values
}

#[test]
fn scoped_evaluation_range_matches_independent_full_map_reference() {
    for case in 0..32 {
        let source = fixture("unrelated");
        let mut expected_partitions = source.partitions.clone();
        let mut expected_details = source.details.clone();
        let regions: Vec<_> = (0..case)
            .map(|n| {
                let partition = if n % 3 == 0 { "selected" } else { "new" };
                if n % 4 == 0 {
                    ChangedRegion::new(partition)
                } else {
                    ChangedRegion::new(partition).with_detail(format!("detail-{}", n % 5))
                }
            })
            .collect();
        // Deliberately full-map reference: no ordered range, same public meaning.
        for region in &regions {
            expected_partitions.insert(region.partition.clone(), version(9));
            if let Some(detail) = &region.detail {
                expected_details.insert(
                    PartitionSubscription::partition_and_detail(
                        region.partition.clone(),
                        detail.clone(),
                    ),
                    version(9),
                );
            } else {
                for (scope, value) in &mut expected_details {
                    if scope.partition == region.partition {
                        *value = version(9);
                    }
                }
            }
        }
        let mut candidate = source.clone();
        let mut work = Work::new(100_000_000);
        candidate
            .admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut work))
            .unwrap();
        assert_eq!(candidate, source);
        candidate.apply_evaluation(version(9), &regions);
        assert_eq!(candidate.partitions, expected_partitions);
        assert_eq!(candidate.details, expected_details);
        for detail in ["first", "second", "third"] {
            assert_eq!(
                candidate.scoped_or_global(
                    &PartitionSubscription::partition_and_detail("unrelated", detail),
                    version(0)
                ),
                version(1)
            );
        }
    }
}

#[test]
fn scoped_evaluation_admits_exact_shared_work_before_mutation() {
    let source = fixture("unrelated");
    let regions = [
        ChangedRegion::new("selected").with_detail("new"),
        ChangedRegion::new("selected"),
    ];
    let mut measured = Work::new(100_000_000);
    source
        .admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 31);
        work.reserve_visits(cost + 31 - available).unwrap();
        let mut candidate = source.clone();
        let result =
            candidate.admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut work));
        assert_eq!(candidate, source);
        if available == cost {
            result.unwrap();
            candidate.apply_evaluation(version(9), &regions);
            assert_eq!(
                candidate.scoped_or_global(
                    &PartitionSubscription::partition_and_detail("selected", "new"),
                    version(0)
                ),
                version(9)
            );
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 31
                })
            );
        }
    }
}

#[test]
fn scoped_evaluation_does_not_charge_unrelated_payloads_as_matching_details() {
    let regions = [ChangedRegion::new("selected")];
    let mut costs = Vec::new();
    for unrelated in ["z".to_owned(), "z".repeat(10000)] {
        let values = fixture(&unrelated);
        let mut work = Work::new(100_000_000);
        values
            .admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut work))
            .unwrap();
        costs.push(work.visits());
    }
    assert_eq!(costs[0], costs[1]);
    let values = fixture("unrelated");
    let mut short = Work::new(100_000_000);
    values
        .admit_evaluation_work(
            &[ChangedRegion::new("x").with_detail("x")],
            &mut EvaluationWork::Conditional(&mut short),
        )
        .unwrap();
    assert!(matches!(
        values.admit_evaluation_work(
            &[ChangedRegion::new("x".repeat(10000)).with_detail("x".repeat(10000))],
            &mut EvaluationWork::Conditional(&mut Work::new(short.visits()))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
}
