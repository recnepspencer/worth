//! Runtime allocation changes cannot enter recorded logical reference meaning.
use super::*;
use serde_json::Value;

fn projections(
    request: &ExecutionRequest<'_, '_>,
) -> (
    Value,
    Value,
    worth_relational::facade::branch::RecordedRelationalComponentReference,
) {
    let (runtime, _) = initialized(request);
    let commit = create_entity_outcome(request, &runtime, "reference-before");
    release_test_commit_snapshot(&runtime, &commit);
    let expected = current(request, &runtime);
    let commit = create_entity_outcome(request, &runtime, "reference-after");
    release_test_commit_snapshot(&runtime, &commit);
    let captured = capture(&runtime, request);
    let image = captured
        .admission_image()
        .seal_home_cut(&cut(&captured), request)
        .unwrap();
    let bytes = image.as_bytes();
    let cut_end = 64 + u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let mut at = cut_end;
    let (_, roots) = group(bytes, &mut at);
    let roots = crate::source_wire_value::read(roots);
    let mut axes = None;
    for expected in 1..=2 {
        assert_eq!(
            u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()),
            expected
        );
        at += 16;
        for tag in 1..=8 {
            let (actual_tag, payload) = group(bytes, &mut at);
            assert_eq!(actual_tag, tag);
            if expected == 2 && tag == 2 {
                axes = Some(crate::source_wire_value::read(payload));
            }
        }
    }
    assert_eq!(at, bytes.len());
    (axes.unwrap(), roots, expected)
}
pub(super) fn group<'a>(bytes: &'a [u8], at: &mut usize) -> (u8, &'a [u8]) {
    let tag = bytes[*at];
    let length = u64::from_le_bytes(bytes[*at + 1..*at + 9].try_into().unwrap()) as usize;
    let start = *at + 41;
    *at = start + length;
    (tag, &bytes[start..*at])
}
pub(super) fn no_runtime_identity(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                assert_ne!(key, "runtime_instance_id");
                assert_ne!(key, "root_identity");
                no_runtime_identity(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                no_runtime_identity(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[test]
fn checkpoint_image_records_logical_reference_coordinates_across_runtime_instances() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let first = projections(request);
    // Consume another actual runtime allocation between identical histories.
    drop(initialized(request));
    let second = projections(request);
    for (axes, roots, expected) in [&first, &second] {
        no_runtime_identity(axes);
        no_runtime_identity(roots);
        let reference = &axes["predecessor"]["reference"];
        assert_eq!(reference["observation"]["branch"], "main");
        let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(target) =
            expected.target()
        else {
            panic!("the first genuine commit is canonical");
        };
        assert_eq!(
            reference["observation"]["target"]["Canonical"]["commit"],
            target.receipt().commit_id.0
        );
        assert_eq!(
            reference["observation"]["generation"],
            expected.descriptor().reference().generation().get()
        );
        assert_eq!(
            reference["truth_version"],
            expected.descriptor().truth_version().as_u64()
        );
        assert_eq!(axes["branch"], "main");
    }
    assert_eq!(
        first, second,
        "logical reference projection is independent of runtime allocation"
    );
}
