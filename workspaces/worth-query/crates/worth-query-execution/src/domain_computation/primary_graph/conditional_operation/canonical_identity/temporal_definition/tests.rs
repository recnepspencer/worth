//! Key composition over canonical definition DATA and exact location.
use super::*;
#[test]
fn temporal_node_keys_bind_operation_definition_and_exact_declared_location() {
    let location = WorthQueryConditionalNodeLocation::operation("gate").unwrap();
    let derive = |operation: [u8; 32],
                  domain: &str,
                  slot: &str,
                  location: &WorthQueryConditionalNodeLocation| {
        prepare_temporal_node_definition(CanonicalDigestId::new(operation), domain, slot, location)
            .unwrap()
            .0
    };
    let original = derive([1; 32], "canonical-domain-definition", "slot", &location);
    assert_eq!(
        original,
        derive([1; 32], "canonical-domain-definition", "slot", &location)
    );
    assert_ne!(
        original,
        derive([2; 32], "canonical-domain-definition", "slot", &location)
    );
    assert_ne!(
        original,
        derive([1; 32], "changed-domain-definition", "slot", &location)
    );
    assert_ne!(
        original,
        derive(
            [1; 32],
            "canonical-domain-definition",
            "other-slot",
            &location
        )
    );
    assert_ne!(
        original,
        derive(
            [1; 32],
            "canonical-domain-definition",
            "slot",
            &WorthQueryConditionalNodeLocation::workflow_stage("stage", "gate").unwrap()
        )
    );
    assert_ne!(
        original,
        derive(
            [1; 32],
            "canonical-domain-definition",
            "slot",
            &WorthQueryConditionalNodeLocation::operation("another-gate").unwrap()
        )
    );
}
