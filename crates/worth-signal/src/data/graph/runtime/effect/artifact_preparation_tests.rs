use super::*;
use crate::data::trace::ColdArtifactIntent;
use crate::facade::SignalRuntimePolicy;

#[test]
fn artifact_preparation_preserves_retain_reconstruct_and_omit_policy() {
    use ArtifactRetentionPolicy::{Omit, Reconstruct, Retain};
    for explanation in [Retain, Reconstruct, Omit] {
        for provenance in [Retain, Reconstruct, Omit] {
            for has_intent in [false, true] {
                let mut graph = SignalGraph::new();
                graph.set_runtime_policy(
                    SignalRuntimePolicy::development()
                        .with_explanation_retention(explanation)
                        .with_provenance_retention(provenance),
                );
                let intent = has_intent.then(|| ColdArtifactIntent {
                    labels: smallvec::smallvec!["prepared".into()],
                    ..Default::default()
                });
                let prepared = graph
                    .prepare_effect_artifact_write(
                        Some(HotArtifactWrite {
                            runtime: None,
                            cold_intent: intent,
                        }),
                        &mut EvaluationWork::Ordinary,
                    )
                    .unwrap();
                let retained = has_intent && (explanation == Retain || provenance == Retain);
                let bypassed =
                    !retained && (has_intent || (explanation == Omit && provenance == Omit));
                assert_eq!(prepared.retained.is_some(), retained);
                assert_eq!(
                    graph
                        .telemetry()
                        .storage
                        .hot_write_cold_record_materialization_count,
                    u64::from(retained)
                );
                assert_eq!(
                    graph.telemetry().storage.hot_write_cold_bypass_count,
                    u64::from(bypassed)
                );
            }
        }
    }
}

#[test]
fn conditional_artifact_storage_denies_before_materialization_and_moves_label_payloads() {
    use crate::data::retained_storage::RetainedStoragePreparation;
    for count in [1, 8] {
        let maximum = count + 8;
        for admitted in [false, true] {
            let mut graph = SignalGraph::new();
            graph.set_runtime_policy(SignalRuntimePolicy::forensic());
            let labels: smallvec::SmallVec<[_; 4]> = (0..count)
                .map(|n| format!("label-{n}-{}", "λ".repeat(1000)))
                .collect();
            let pointers: Vec<_> = labels.iter().map(|label| label.as_ptr()).collect();
            let write = HotArtifactWrite {
                runtime: None,
                cold_intent: Some(ColdArtifactIntent {
                    labels,
                    ..Default::default()
                }),
            };
            let limit = maximum - usize::from(!admitted);
            let mut work = RetainedStoragePreparation::new(limit);
            let result = graph.prepare_effect_artifact_write(
                Some(write),
                &mut EvaluationWork::Conditional(&mut work),
            );
            if admitted {
                let retained = result.unwrap().retained.unwrap();
                assert_eq!(
                    retained
                        .labels
                        .iter()
                        .map(|label| label.as_ptr())
                        .collect::<Vec<_>>(),
                    pointers
                );
                assert_eq!(work.visits(), maximum);
            } else {
                assert!(
                    matches!(result, Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }) if maximum_visits == limit)
                );
                assert_eq!(work.visits(), 0);
            }
            assert_eq!(
                graph
                    .telemetry()
                    .storage
                    .hot_write_cold_record_materialization_count,
                u64::from(admitted)
            );
        }
    }
}
