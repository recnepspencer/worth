use super::*;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
#[test]
fn projection_matches_actual_scoped_writes() {
    let aspect = Aspect::new(1);
    let old = AspectVersion::zero().with(aspect, 3);
    let next = old.with(aspect, 9);
    let mut basis = PartitionVersionOverrides::default();
    basis.apply_evaluation(
        old,
        &[
            ChangedRegion::new("p").with_detail("old"),
            ChangedRegion::new("q"),
        ],
    );
    let mut unusual = PartitionSubscription::partition_and_detail("p", "old");
    unusual.match_mode = PartitionMatchMode::WholePartition;
    basis.details.insert(unusual.clone(), old);
    let queries = [
        None,
        Some(PartitionSubscription::whole_partition("p")),
        Some(PartitionSubscription::partition_and_detail("p", "old")),
        Some(PartitionSubscription::partition_and_detail("p", "new")),
        Some(unusual),
        Some(PartitionSubscription::whole_partition("q")),
        Some(PartitionSubscription::whole_partition("absent")),
    ];
    let choices = [
        ChangedRegion::new("p"),
        ChangedRegion::new("p").with_detail("old"),
        ChangedRegion::new("p").with_detail("new"),
        ChangedRegion::new("q"),
    ];
    for mask in 0..16 {
        let regions: Vec<_> = choices
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, r)| r.clone())
            .collect();
        let mut applied = basis.clone();
        applied.apply_evaluation(next, &regions);
        for query in &queries {
            let mut measured = Work::new(usize::MAX);
            let projected = basis
                .version_after_evaluation(
                    aspect,
                    query.as_ref(),
                    next,
                    &regions,
                    &mut EvaluationWork::Conditional(&mut measured),
                )
                .unwrap();
            assert_eq!(
                projected,
                applied.version_for_scope(aspect, query.as_ref(), next)
            );
            let mut short = Work::new(measured.visits() - 1);
            assert!(basis
                .version_after_evaluation(
                    aspect,
                    query.as_ref(),
                    next,
                    &regions,
                    &mut EvaluationWork::Conditional(&mut short)
                )
                .is_err());
        }
    }
}
