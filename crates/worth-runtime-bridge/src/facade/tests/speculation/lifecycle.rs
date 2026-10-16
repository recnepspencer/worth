use crate::facade::tests::runtime;
use crate::facade::tests::speculation::preview_declaration;
use crate::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewResidueClass, BridgeRuntimePolicy,
};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn runtime_activates_and_discards_preview_session_with_zero_authoritative_residue() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:analysis"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");

    let (active, execution_record) = runtime.activate_preview_session(admitted, 3, 1, 2);
    assert_eq!(execution_record.counters().preview_artifact_count(), 3);

    let (discarded, discard_record) = runtime
        .discard_preview_session(
            active,
            &execution_record,
            vec![
                BridgePreviewResidueClass::PreviewExecutionRetained,
                BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
                BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
            ],
        )
        .expect("preview discard should succeed");

    assert_eq!(
        discarded.lifecycle_state_kind(),
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert_eq!(
        discard_record
            .residue_report()
            .authoritative_residue_count(),
        0
    );
    assert_eq!(discard_record.counters().destroyed_artifact_count(), 1);
    assert_eq!(runtime.diagnostics().preview_execution_records().len(), 1);
    assert_eq!(runtime.diagnostics().preview_discard_records().len(), 1);
    assert_eq!(
        runtime
            .diagnostics()
            .preview_discard_record_for_session_identity(discarded.session_identity())
            .expect("discard diagnostics should be retained")
            .record_identity(),
        discard_record.record_identity()
    );
}

#[test]
fn runtime_rejects_preview_discard_when_authoritative_residue_remains() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:authority"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 2, 1, 1);

    let original_digest = active.digest().to_owned();
    let (active, error) = runtime
        .discard_preview_session(
            active,
            &execution_record,
            vec![BridgePreviewResidueClass::AuthoritativeRoutingResidue],
        )
        .expect_err("authoritative residue must block discard");

    assert_eq!(
        error.kind(),
        crate::error::BridgeSpeculationErrorKind::PreviewResidueClassificationMismatch
    );
    assert_eq!(active.digest(), original_digest);
    assert!(runtime.diagnostics().preview_discard_records().is_empty());
    let (discarded, _) = runtime
        .discard_preview_session(active, &execution_record, Vec::new())
        .unwrap();
    assert_eq!(
        discarded.session_identity().as_str(),
        "preview-session:authority"
    );
    assert_eq!(runtime.diagnostics().preview_execution_records().len(), 1);
    assert_eq!(runtime.diagnostics().preview_discard_records().len(), 1);
}
