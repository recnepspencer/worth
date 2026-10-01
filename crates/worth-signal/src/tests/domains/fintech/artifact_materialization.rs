use crate::facade::*;

use super::scales::FintechScale;
use super::scenarios::setup_world;

#[test]
fn fintech_retained_and_reconstructed_artifacts_agree_across_runtime_policies() {
    let mut retained_world = setup_world();
    retained_world.assert_shape(FintechScale::smoke());
    retained_world.runtime.set_runtime_policy(
        SignalRuntimePolicy::development()
            .with_history_limit(8)
            .with_detail_limit(4),
    );

    let mut reconstructed_world = setup_world();
    reconstructed_world.assert_shape(FintechScale::smoke());
    reconstructed_world.runtime.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_history_limit(8)
            .with_detail_limit(4),
    );

    let retained_audit = retained_world.read_primary_audit_surface().unwrap();
    let reconstructed_audit = reconstructed_world.read_primary_audit_surface().unwrap();
    assert_eq!(retained_audit, reconstructed_audit);

    let retained_node = retained_world.top_desk();
    let reconstructed_node = reconstructed_world.top_desk();

    let retained_explanation = retained_world
        .runtime
        .observe()
        .materialize()
        .retained_explanation_artifact(retained_node)
        .expect("development policy should retain explanations eagerly");
    let retained_provenance = retained_world
        .runtime
        .observe()
        .materialize()
        .retained_provenance_artifact(retained_node)
        .expect("development policy should retain provenance eagerly");

    assert!(reconstructed_world
        .runtime
        .observe()
        .materialize()
        .retained_explanation_artifact(reconstructed_node)
        .is_none());
    assert!(reconstructed_world
        .runtime
        .observe()
        .materialize()
        .retained_provenance_artifact(reconstructed_node)
        .is_none());

    let reconstructed_explanation = reconstructed_world
        .runtime
        .observe()
        .materialize()
        .reconstruct_explanation_artifact(reconstructed_node)
        .unwrap();
    let reconstructed_provenance = reconstructed_world
        .runtime
        .observe()
        .materialize()
        .reconstruct_provenance_artifact(reconstructed_node)
        .unwrap();

    assert_eq!(retained_explanation.node, reconstructed_explanation.node);
    assert_eq!(
        retained_explanation.upstream.len(),
        reconstructed_explanation.upstream.len()
    );
    assert_eq!(
        retained_provenance.vertices.len(),
        reconstructed_provenance.vertices.len()
    );
    assert!(reconstructed_provenance
        .vertices
        .iter()
        .any(|vertex| vertex.node == reconstructed_node));
}
