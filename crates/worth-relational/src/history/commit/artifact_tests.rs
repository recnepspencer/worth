use std::sync::Arc;

use crate::tests::support::create_entity_outcome;

use super::{RelationalCommitArtifact, RelationalCommitArtifactDenial};

#[test]
fn recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = crate::tests::support::runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "preencoded-recovery-root");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("committed main has one root");
    let envelope = root
        .canonical_envelope()
        .cloned()
        .expect("committed root carries its canonical envelope");
    let preencoded = RelationalCommitArtifact::from_preencoded(
        crate::branch::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::clone(&envelope),
        )
        .unwrap(),
    )
    .unwrap();
    let legacy =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .unwrap();
    let relinked = preencoded
        .relink_preencoded_recovery(&envelope, Some(&root), None)
        .unwrap();
    assert!(Arc::ptr_eq(
        &preencoded.envelope_residency(),
        &relinked.envelope_residency()
    ));
    assert_eq!(relinked.identity(), legacy.identity());
    assert_eq!(relinked.parentage(), legacy.parentage());
    assert_eq!(relinked.roots(), legacy.roots());
    assert_eq!(
        relinked
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest"),
        legacy
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest")
    );

    let equal_but_unsealed = Arc::new(envelope.copy_original_for_test(execution));
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&equal_but_unsealed, Some(&root), None),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
    let mismatched_schema = crate::branch::RelationalBranchRootDescriptor::new(
        *legacy.roots().truth_root(),
        [0xAB; 32],
    );
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&envelope, None, Some(&mismatched_schema)),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
}
