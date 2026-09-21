use super::WorthQueryApplicationWorkflowResourceCeiling;

#[test]
fn workflow_resource_ceiling_covers_every_definition_limit() {
    let ceiling =
        WorthQueryApplicationWorkflowResourceCeiling::new(10, 20, 3, 4, 4096, 8, 32, 1024)
            .expect("all workflow ceilings are nonzero");
    assert_eq!(ceiling.maximum_definition_nodes(), 10);
    assert_eq!(ceiling.maximum_definition_connections(), 20);
    assert_eq!(ceiling.maximum_definition_effects(), 3);
    assert_eq!(ceiling.maximum_component_depth(), 4);
    assert_eq!(ceiling.maximum_canonical_bytes(), 4096);
}

#[test]
fn zero_component_or_canonical_ceiling_is_rejected() {
    assert!(
        WorthQueryApplicationWorkflowResourceCeiling::new(10, 20, 3, 0, 4096, 8, 32, 1024)
            .is_none()
    );
    assert!(
        WorthQueryApplicationWorkflowResourceCeiling::new(10, 20, 3, 4, 0, 8, 32, 1024).is_none()
    );
    assert!(
        WorthQueryApplicationWorkflowResourceCeiling::new(10, 20, 3, 4, 4096, 8, 0, 1024).is_none()
    );
}
