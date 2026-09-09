use super::*;
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn retained_node_payload_copy_is_admitted_before_mutation() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let payload = "nested-payload".repeat(1000);
    // Storage copy fixture: these labels make no claim of performed evaluation.
    graph.cold_mut(node).unwrap().causality = Some(crate::data::trace::CausalityMetadata {
        kind: payload.clone(),
        fields: [(payload.clone(), payload.clone())].into_iter().collect(),
    });
    let mut runtime = crate::data::trace::RuntimeArtifactState::default();
    runtime.warm_mut().output_identity = Some(crate::data::output::OutputIdentity::new(&payload));
    graph.warm_mut(node).unwrap().runtime_artifact_state = Some(runtime);
    graph
        .warm_mut(node)
        .unwrap()
        .dirty_partition_scope_payload
        .push((
            crate::data::aspect::Aspect::new(0),
            crate::data::output::PartitionSubscription::whole_partition(payload.as_str()),
        ));
    let mut exclusive = Work::new(0);
    graph
        .admit_effect_node_copy_work(node, &mut EvaluationWork::Conditional(&mut exclusive))
        .unwrap();
    let _retained = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let previous = graph.arena.cold.fork_persistent();
    let mut measured = Work::new(usize::MAX);
    graph
        .admit_effect_node_copy_work(node, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    assert!(cost >= 5 * payload.len());
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 19);
        work.reserve_visits(cost + 19 - available).unwrap();
        let result =
            graph.admit_effect_node_copy_work(node, &mut EvaluationWork::Conditional(&mut work));
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 19);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 19
                })
            );
        }
        assert_eq!(
            graph
                .cold_ref(node)
                .unwrap()
                .unwrap()
                .causality
                .as_ref()
                .unwrap()
                .kind,
            payload
        );
    }
    graph
        .cold_mut(node)
        .unwrap()
        .causality
        .as_mut()
        .unwrap()
        .kind
        .clear();
    assert_eq!(
        previous
            .get(node.index() as usize)
            .unwrap()
            .as_ref()
            .unwrap()
            .causality
            .as_ref()
            .unwrap()
            .kind,
        payload
    );
}
