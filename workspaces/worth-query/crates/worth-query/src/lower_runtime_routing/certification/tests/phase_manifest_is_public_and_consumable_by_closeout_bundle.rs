use super::*;

#[test]
fn phase_manifest_is_public_and_consumable_by_closeout_bundle() {
    let manifest = worth_query_lower_runtime_phase_manifest();

    assert_eq!(
        manifest.manifest_digest(),
        worth_query_lower_runtime_phase_artifact_manifest_digest()
    );
    assert_eq!(
        manifest.typestate_transition_digest(),
        worth_query_lower_runtime_typestate_transition_digest()
    );
    assert_eq!(
        manifest
            .rows()
            .last()
            .expect("manifest rows")
            .next_consumer(),
        "runtime-api-public-stabilization gate"
    );
}
