use worth_runtime_bridge::facade::RuntimeWorldCorrespondenceAdmissionDenial;

#[test]
fn bridge_generation_drift_is_a_distinct_denial_axis() {
    let denial = RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
        expected_generation: 12,
        actual_generation: 11,
    };

    assert!(matches!(
        denial,
        RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
            expected_generation: 12,
            actual_generation: 11,
        }
    ));
    assert_ne!(
        denial,
        RuntimeWorldCorrespondenceAdmissionDenial::InstalledCorrespondenceNotCurrent
    );
}
