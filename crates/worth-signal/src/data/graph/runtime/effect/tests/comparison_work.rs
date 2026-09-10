use super::*;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::{ArtifactContinuityToken, OutputIdentity};
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::EvaluationWork;

#[test]
fn partition_count_matches_independent_set_with_exact_and_short_work() {
    use super::super::vocabulary::count_changed_partitions;
    for count in 0..64 {
        let regions: Vec<_> = (0..count)
            .map(|n| {
                ChangedRegion::new(format!("market-λ-{}", n % 7)).with_detail(format!("detail-{n}"))
            })
            .collect();
        let expected = regions
            .iter()
            .map(|region| region.partition.0.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u32;
        let mut work = Work::new(1_000_000);
        assert_eq!(
            count_changed_partitions(&regions, &mut EvaluationWork::Conditional(&mut work))
                .unwrap(),
            expected
        );
        let cost = work.visits();
        assert_eq!(
            count_changed_partitions(
                &regions,
                &mut EvaluationWork::Conditional(&mut Work::new(cost))
            )
            .unwrap(),
            expected
        );
        if cost > 0 {
            assert!(matches!(
                count_changed_partitions(
                    &regions,
                    &mut EvaluationWork::Conditional(&mut Work::new(cost - 1))
                ),
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
    }
    let short = [ChangedRegion::new("x"), ChangedRegion::new("x")];
    let mut work = Work::new(1000);
    count_changed_partitions(&short, &mut EvaluationWork::Conditional(&mut work)).unwrap();
    let long = [
        ChangedRegion::new("λ".repeat(1000)),
        ChangedRegion::new("λ".repeat(1000)),
    ];
    assert!(matches!(
        count_changed_partitions(
            &long,
            &mut EvaluationWork::Conditional(&mut Work::new(work.visits()))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
}

#[test]
fn output_identity_comparison_uses_shared_work_and_query_bytes() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let identity = OutputIdentity::new("same-λ".repeat(1000));
    let continuity = ArtifactContinuityToken::new("same-continuity".repeat(1000));
    let mut effect = test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.diagnostics = DiagnosticEnvelope::from_parts(
        Some(identity.clone()),
        Some(continuity.clone()),
        Vec::new(),
        Vec::new(),
    );
    let mut work = Work::new(1_000_000);
    let comparison = graph
        .compare_effect(
            &effect,
            Some(&identity),
            Some(&continuity),
            OutputEquivalencePolicy::OutputIdentity,
            &mut EvaluationWork::Conditional(&mut work),
        )
        .unwrap();
    assert!(comparison.output_identity_unchanged);
    assert!(comparison.continuity_token_unchanged);
    assert!(comparison.propagation_suppressed);
    let cost = work.visits();
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 7);
        work.reserve_visits(cost + 7 - available).unwrap();
        let result = graph.compare_effect(
            &effect,
            Some(&identity),
            Some(&continuity),
            OutputEquivalencePolicy::OutputIdentity,
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            assert!(result.unwrap().propagation_suppressed);
        } else {
            assert!(matches!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
    }
}
