use super::*;
use crate::expression::model::{ConditionSpec, Expr, IdentitySpec};
use crate::recipe::model::RecipeSpec;
use crate::runtime::core::RuntimeCore;
use crate::runtime::policy::{RuntimePolicyPreset, RuntimePolicySpec};

fn compute_without_publication(core: &RuntimeCore, node: NodeId) -> EvaluationOutput {
    let graph = core.runtime.observe().graph().graph();
    let mut view = EvaluationContext::new(graph, node, &core.store);
    evaluate_node(
        &mut view,
        &core.store,
        &core.callback_diagnostics,
        &core.nodes_by_id,
    )
    .unwrap()
}

fn expected_output(core: &RuntimeCore, change: OutputChange) -> EvaluationOutput {
    let store = core.store.lock().unwrap();
    let recipe = store.recipes.get("candidate").unwrap();
    let mut result = NodeEvaluationResult::from_version(recipe.version).with_output_change(change);
    if let Some(identity) = &recipe.output_identity {
        result = result.with_output_identity(identity.clone());
    }
    EvaluationOutput::from_result(result)
}

#[test]
fn unpublished_recipe_retry_uses_graph_baseline_even_when_condition_is_false() {
    for preset in [
        RuntimePolicyPreset::Operational,
        RuntimePolicyPreset::Forensic,
    ] {
        for condition in [true, false] {
            for identity in [None, Some(IdentitySpec::Exact)] {
                let mut core = RuntimeCore::new(RuntimePolicySpec {
                    preset: preset.clone(),
                })
                .unwrap();
                core.define_recipe(RecipeSpec {
                    id: "candidate".into(),
                    reads: Vec::new(),
                    expr: Expr::Value {
                        value: SignalValue::Number(7.0),
                    },
                    when: Some(ConditionSpec {
                        expr: Expr::Value {
                            value: SignalValue::Bool(condition),
                        },
                    }),
                    identity,
                    produces_aspects: None,
                })
                .unwrap();
                let node = core.catalog.get("candidate").unwrap().node;
                let before = core
                    .runtime
                    .observe()
                    .graph()
                    .graph()
                    .node_aspect_version(node)
                    .unwrap();
                let first = compute_without_publication(&core, node);
                assert_eq!(first, expected_output(&core, OutputChange::Replaced));
                let retry = compute_without_publication(&core, node);
                assert_eq!(retry, first);
                assert_eq!(
                    core.runtime
                        .observe()
                        .graph()
                        .graph()
                        .node_aspect_version(node)
                        .unwrap(),
                    before
                );
                assert!(core
                    .runtime
                    .observe()
                    .graph()
                    .runtime_artifact_warm(node)
                    .unwrap()
                    .is_none());
                assert_eq!(
                    core.read_value("candidate").unwrap(),
                    SignalValue::Number(7.0)
                );
                let after = core
                    .runtime
                    .observe()
                    .graph()
                    .graph()
                    .node_aspect_version(node)
                    .unwrap();
                assert_ne!(after, before);
                assert_eq!(
                    compute_without_publication(&core, node),
                    expected_output(&core, OutputChange::Unchanged)
                );
            }
        }
    }
}
