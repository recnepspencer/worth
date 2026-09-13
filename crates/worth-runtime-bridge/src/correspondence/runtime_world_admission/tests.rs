use super::RuntimeWorldCorrespondenceAdmissionDenial;

#[test]
fn admission_denial_names_generation_rebind_without_a_relational_adapter() {
    let drift = RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
        expected_generation: 8,
        actual_generation: 7,
    };

    assert!(matches!(
        drift,
        RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
            expected_generation: 8,
            actual_generation: 7,
        }
    ));
    assert_ne!(
        drift,
        RuntimeWorldCorrespondenceAdmissionDenial::InstalledCorrespondenceNotCurrent
    );
}

#[test]
fn admission_denials_keep_foreign_missing_and_stale_facts_distinct() {
    let foreign = RuntimeWorldCorrespondenceAdmissionDenial::ForeignBridgeRuntime {
        expected_runtime_key: 11,
        actual_runtime_key: 12,
    };
    let missing = RuntimeWorldCorrespondenceAdmissionDenial::InstalledCorrespondenceNotCurrent;
    let drift = RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
        expected_generation: 8,
        actual_generation: 7,
    };

    assert_ne!(foreign, missing);
    assert_ne!(foreign, drift);
    assert_ne!(missing, drift);
    assert!(foreign.to_string().contains("expected 11"));
    assert!(missing.to_string().contains("not current"));
    assert!(drift.to_string().contains("generation 7"));
}
