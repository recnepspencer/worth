use super::*;
use crate::data::aspect::AspectVersion;
use crate::data::output::ChangedRegion;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::DiagnosticEnvelope;

fn packet(
    mask: AspectMask,
    exact: Vec<(Aspect, ChangedRegion)>,
    legacy: Vec<ChangedRegion>,
) -> (SignalGraph, ApplyCommitPacket) {
    let mut graph = SignalGraph::new();
    let node = graph.node().produces_aspects(mask).build();
    let mut effect = super::super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.operational.aspect_version =
        AspectVersion::from_updates([(Aspect::new(0), 9), (Aspect::new(2), 17)]);
    effect.operational.changed_aspect_regions = exact;
    effect.diagnostics = DiagnosticEnvelope::from_parts(None, None, legacy, Vec::new());
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    (graph, apply)
}

#[test]
fn budgeted_delta_matches_independent_grouping_with_mixed_scope_precision() {
    for count in 0..24 {
        for mask in [
            AspectMask::ALL,
            AspectMask::from_aspect(Aspect::new(0)),
            AspectMask::EMPTY,
        ] {
            let exact = (0..count)
                .rev()
                .map(|n| {
                    (
                        // Include unchanged and undeclared aspects, duplicates and details.
                        Aspect::new((n % 3) as u8),
                        if n % 2 == 0 {
                            ChangedRegion::new(format!("p-{}", n % 5))
                        } else {
                            ChangedRegion::new(format!("p-{}", n % 5)).with_detail("detail-λ")
                        },
                    )
                })
                .collect();
            let legacy = if count % 2 == 0 {
                vec![
                    ChangedRegion::new("fallback"),
                    ChangedRegion::new("fallback"),
                ]
            } else {
                Vec::new()
            };
            let (graph, apply) = packet(mask, exact, legacy);
            let expected = ProducedAspectDelta::from_committed_result(
                apply.effect.operational.node,
                graph.cause_sets.reserve_output_commit_ordinal(),
                AspectVersion::zero(),
                apply.effect.operational.aspect_version,
                mask,
                apply.effect.changed_aspect_regions(),
                apply.effect.changed_regions(),
            );
            let mut measured = Work::new(10_000_000);
            let actual = graph
                .prepare_produced_delta(&apply, &mut EvaluationWork::Conditional(&mut measured))
                .unwrap();
            assert_eq!(actual, expected);
            let cost = measured.visits();
            for available in [cost - 1, cost] {
                let mut work = Work::new(cost + 7);
                work.reserve_visits(cost + 7 - available).unwrap();
                let actual = graph
                    .prepare_produced_delta(&apply, &mut EvaluationWork::Conditional(&mut work));
                if available == cost {
                    assert_eq!(actual.unwrap(), expected);
                } else {
                    assert_eq!(
                        actual,
                        Err(SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: cost + 7
                        })
                    );
                }
                assert_eq!(
                    graph
                        .node_aspect_version(apply.effect.operational.node)
                        .unwrap(),
                    AspectVersion::zero()
                );
                assert_eq!(graph.cause_sets.output_commit_ordinal_for_test(), 0);
            }
        }
    }
}

#[test]
fn delta_copy_and_normalization_charge_selected_payload_bytes() {
    let (short_graph, short) = packet(AspectMask::ALL, Vec::new(), vec![ChangedRegion::new("x")]);
    let mut measured = Work::new(1_000_000);
    short_graph
        .prepare_produced_delta(&short, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let text = "same-λ".repeat(2_000);
    let (graph, apply) = packet(
        AspectMask::ALL,
        Vec::new(),
        vec![ChangedRegion::new(text.as_str()).with_detail(&text)],
    );
    assert!(matches!(
        graph.prepare_produced_delta(
            &apply,
            &mut EvaluationWork::Conditional(&mut Work::new(measured.visits()))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    let mut work = Work::new(10_000_000);
    let actual = graph
        .prepare_produced_delta(&apply, &mut EvaluationWork::Conditional(&mut work))
        .unwrap()
        .unwrap();
    assert_eq!(actual.changes.as_slice().len(), 2);
    assert!(
        work.visits() >= 4 * text.len(),
        "each changed aspect owns both strings"
    );
}
