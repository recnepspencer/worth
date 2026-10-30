#[test]
fn rejected_batch_input_is_not_copied_before_cardinality_admission() {
    let parent = tempfile::tempdir().unwrap();
    let output =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "batch_admission_probe",
            &parent.path().join("unused-store"),
            None,
        );
    assert!(output
        .lines()
        .any(|line| line == "C5_BATCH_ADMISSION 0 0 true"));
}

#[test]
fn impossible_inline_geometry_is_rejected_before_allocation_or_media_effects() {
    let parent = tempfile::tempdir().unwrap();
    let output =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "geometry_admission_probe",
            &parent.path().join("store"),
            None,
        );
    assert!(
        output
            .lines()
            .any(|line| line == "C5_GEOMETRY_ADMISSION 0 0 0 true"),
        "unexpected geometry admission evidence: {output}"
    );
}
