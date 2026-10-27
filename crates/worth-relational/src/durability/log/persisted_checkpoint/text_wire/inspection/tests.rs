//! Independent authenticated-frame rewrites exercise the codec's closed schema.
use super::*;
use crate::durability::log::native_file_codec::CHECKPOINT_MAGIC;
use crate::tests::support::*;
fn original(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalNativeCheckpoint, crate::identity::data::EntityId) {
    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "inspected-record");
    (
        runtime
            .durability_authority()
            .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        entity,
    )
}
fn map_bytes(fields: BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![0xde];
    bytes.extend(u16::try_from(fields.len()).unwrap().to_be_bytes());
    for (key, value) in fields {
        bytes.extend(rmp_serde::to_vec(&key).unwrap());
        bytes.extend(value);
    }
    bytes
}
fn map(bytes: &[u8], edit: impl FnOnce(&mut BTreeMap<String, Vec<u8>>)) -> Vec<u8> {
    let mut value = fields(bytes)
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, v.to_vec()))
        .collect();
    edit(&mut value);
    map_bytes(value)
}
fn rewritten(
    bytes: &[u8],
    allocator: bool,
    edit: impl FnOnce(&mut BTreeMap<String, Vec<u8>>),
) -> Vec<u8> {
    let envelope = fields(&bytes[CHECKPOINT_MAGIC.len()..]).unwrap();
    let checkpoint = map(envelope["checkpoint"], |stored| {
        if allocator {
            let value = map(&stored["record_identity"], edit);
            stored.insert("record_identity".into(), value);
        } else {
            edit(stored);
        }
    });
    let mut result = CHECKPOINT_MAGIC.to_vec();
    result.push(0x82);
    // The actual native grammar requires the version first.
    result.extend(rmp_serde::to_vec("checkpoint_protocol_version").unwrap());
    result.extend(envelope["checkpoint_protocol_version"]);
    result.extend(rmp_serde::to_vec("checkpoint").unwrap());
    result.extend(checkpoint);
    result
}
/// Change an encoding's integer/length width without changing its value.
fn wider(old: &[u8]) -> Option<Vec<u8>> {
    let mut result = match old[0] {
        n @ 0x00..=0x7f => {
            let mut v = vec![0xcf];
            v.extend(u64::from(n).to_be_bytes());
            return Some(v);
        }
        n @ 0x80..=0x8f => {
            let mut v = vec![0xde];
            v.extend(u16::from(n - 0x80).to_be_bytes());
            v
        }
        n @ 0x90..=0x9f => {
            let mut v = vec![0xdc];
            v.extend(u16::from(n - 0x90).to_be_bytes());
            v
        }
        n @ 0xa0..=0xbf => vec![0xd9, n - 0xa0],
        0xdc | 0xde => {
            let mut v = vec![old[0] + 1];
            v.extend(u32::from(u16::from_be_bytes(old[1..3].try_into().unwrap())).to_be_bytes());
            v.extend(&old[3..]);
            return Some(v);
        }
        0xd9 => {
            let mut v = vec![0xda];
            v.extend(u16::from(old[1]).to_be_bytes());
            v.extend(&old[2..]);
            return Some(v);
        }
        _ => return None,
    };
    result.extend(&old[1..]);
    Some(result)
}
#[test]
fn inspection_compares_every_native_model_value_as_original_bytes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (before, entity) = original(&execution);
    let decoded = before.inspect(&execution).unwrap();
    assert!(
        decoded.live_records.contains(&CheckpointRecordIdentity {
            class: RecordAllocationClass::Entity,
            partition_id: entity.partition_value(),
            slot: entity.local_slot_value(),
            generation: entity.generation_value(),
        }),
        "the census uses the actual committed identity"
    );
    let envelope = fields(&before.bytes()[CHECKPOINT_MAGIC.len()..]).unwrap();
    for (key, old) in fields(envelope["checkpoint"]).unwrap() {
        if key == "record_identity" {
            continue;
        }
        let equivalent = wider(old);
        let after = rewritten(before.bytes(), false, |fields| {
            fields.insert(
                key.clone(),
                equivalent.clone().unwrap_or_else(|| vec![0xc3]),
            );
        });
        match RelationalNativeCheckpoint::inspect_bytes(&execution, &after) {
            Ok(after) => assert_ne!(decoded.model, after.model, "ignored model field {key}"),
            Err(error) => assert!(
                equivalent.is_none(),
                "equivalent {key} encoding refused: {error:?}"
            ),
        }
    }
}
#[test]
fn inspection_refuses_unknown_root_allocator_and_row_fields() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (before, _) = original(&execution);
    for allocator in [false, true] {
        let after = rewritten(before.bytes(), allocator, |fields| {
            fields.insert("future_field".into(), vec![0xc0]);
        });
        assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err());
    }
    for key in ["append_frontiers", "generation_high_water"] {
        let after = rewritten(before.bytes(), true, |fields| {
            let mut rows: Vec<serde_json::Value> = rmp_serde::from_slice(&fields[key]).unwrap();
            assert!(!rows.is_empty());
            rows[0]["future_field"] = true.into();
            fields.insert(key.into(), rmp_serde::to_vec_named(&rows).unwrap());
        });
        assert!(
            RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err(),
            "unknown field in {key} row"
        );
    }
}
