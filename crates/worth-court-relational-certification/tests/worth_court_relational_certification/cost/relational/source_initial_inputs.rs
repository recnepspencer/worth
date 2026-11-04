//! Production catalog work versus real schema-changing history and carrier bytes.
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_testkit_relational::{
    initial_input_source::{capture, schema_transition_history},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn cached_input_commitments_hash_each_schema_once_as_history_and_carrier_size_grow() {
    let small_short = measured_work(64, 8);
    let small_long = measured_work(64, 64);
    let large_short = measured_work(32_768, 8);
    let large_long = measured_work(32_768, 64);
    println!("commitment work: small/8={small_short}, small/64={small_long}, large/8={large_short}, large/64={large_long}");
    assert!(
        large_short > small_short + 32_000,
        "the actual large carriers are hashed"
    );
    let small_rows = small_long - small_short;
    let large_rows = large_long - large_short;
    assert!(small_rows > 0 && large_rows > 0);
    assert!(
        large_rows <= 56 * 1_024,
        "each additional row hashes only small role/policy DATA"
    );
    assert!(
        large_rows.abs_diff(small_rows) <= 56 * 64,
        "schema-byte cost cannot multiply by history"
    );
}
fn measured_work(name_bytes: usize, rows: usize) -> u64 {
    let setup = serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let (runtime, medium) = schema_transition_history(&setup_request, name_bytes, rows);
    let complete = capture(&runtime, &setup_request);
    let policy = serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (leaves, report) = request
        .run(ExecutionWorkCeiling::new(None), |_| {
            complete
                .source_artifact()
                .input_commitment_leaves_for_test(&request)
                .unwrap()
        })
        .unwrap();
    assert_eq!(leaves.len(), rows);
    for (position, leaf) in leaves.iter() {
        let bytes = medium.artifact_bytes(*position).unwrap();
        assert_eq!(*leaf, original_leaf(&bytes));
    }
    report.charged_work()
}
fn original_leaf(frame: &[u8]) -> [u8; 32] {
    // Independent MessagePack slicing, not the owner's semantic codec or hash grammar.
    let payload = &frame[66..];
    assert_eq!(payload[0], 0x83);
    let mut cursor = std::io::Cursor::new(payload);
    cursor.set_position(1);
    let mut found = None;
    for _ in 0..3 {
        let key: String = rmp_serde::from_read(&mut cursor).unwrap();
        let start = cursor.position() as usize;
        let _: serde::de::IgnoredAny = rmp_serde::from_read(&mut cursor).unwrap();
        let end = cursor.position() as usize;
        if key == "original_inputs" {
            let value = crate::source_wire_value::read(&payload[start..end]);
            assert!(
                !value["result_image"].is_null(),
                "each real commit changes its schema carrier"
            );
            assert_ne!(value["prestate_schema"], value["result_schema"]);
            found = Some(Sha256::digest(&payload[start..end]).into());
        }
    }
    assert_eq!(cursor.position() as usize, payload.len());
    found.expect("v5 original input map")
}
