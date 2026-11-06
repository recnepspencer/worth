//! Header validation is envelope work, independent of origin lookup multiplicity.
mod journey;
mod republication;
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn restoration_header_is_validated_once_and_work_grows_linearly() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    for axis in 0..3 {
        let mut work = Vec::new();
        for scale in [2, 4, 8] {
            let (publications, members, edges) = match axis {
                0 => (scale, 2, 4),
                1 => (2, scale, 4),
                _ => (2, 4, scale),
            };
            let journey = journey::run(execution, publications, members, edges);
            let once = journey
                .restored
                .publication()
                .envelope
                .output_origin_header_work_for_test(execution, 1)
                .unwrap();
            let repeated = journey
                .restored
                .publication()
                .envelope
                .output_origin_header_work_for_test(execution, scale)
                .unwrap();
            assert_eq!(once.0, 1, "one envelope has one restoration header");
            assert_eq!(repeated, once, "incident lookups must carry validation");
            eprintln!(
                "axis={axis} scale={scale} validations={} header_work={}",
                once.0, once.1
            );
            work.push(once.1);
        }
        assert!(
            work[0] < work[1] && work[1] < work[2],
            "each axis must perform observable header work: {work:?}"
        );
        assert!(
            work[2] - work[1] <= 2 * (work[1] - work[0]),
            "doubling one axis must have linear growth: {work:?}"
        );
    }
}

#[test]
fn missing_restoration_entity_mapping_is_refused_before_origin_lookup() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let journey = journey::run(execution, 2, 2, 4);
    let denied = journey
        .restored
        .publication()
        .envelope
        .output_origin_header_with_missing_entity_for_test(execution)
        .unwrap_err();
    assert!(
        denied.contains("restoration omits exact recorded members"),
        "{denied}"
    );
}

#[test]
fn public_create_grow_retire_restore_relation_scale_report() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let journey = journey::run(execution, 10, 8, 36);
    for (phase, elapsed) in ["create", "grow", "retire", "undo_retirement"]
        .into_iter()
        .zip(journey.seconds)
    {
        eprintln!(
            "restoration_scale phase={phase} publications=10 restored_relations=720 seconds={:.6}",
            elapsed.as_secs_f64()
        );
    }
}
