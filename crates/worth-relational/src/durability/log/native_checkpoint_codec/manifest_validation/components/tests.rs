//! Mutate only exact headers in a real typed checkpoint wire.
use crate::durability::log::native_checkpoint_codec::{
    checkpoint_wire::CheckpointWire, decode_checkpoint, CHECKPOINT_MAGIC,
};
use crate::tests::support::*;

fn encode(wire: &CheckpointWire<'_>) -> Vec<u8> {
    let mut bytes = CHECKPOINT_MAGIC.to_vec();
    bytes.extend(rmp_serde::to_vec_named(wire).unwrap());
    bytes
}
fn offset(bytes: &[u8], selected: &[u8]) -> usize {
    let offsets = bytes
        .windows(selected.len())
        .enumerate()
        .filter_map(|(offset, span)| (span == selected).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 1, "select the actual exact wire field once");
    offsets[0]
}
fn omit<T: serde::Serialize, C: serde::Serialize>(
    bytes: &mut Vec<u8>,
    container: &C,
    name: &str,
    value: &T,
) {
    let container = rmp_serde::to_vec_named(container).unwrap();
    let container_offset = offset(bytes, &container);
    let mut pair = rmp_serde::to_vec_named(name).unwrap();
    pair.extend(rmp_serde::to_vec_named(value).unwrap());
    let field_offset = container_offset + offset(&container, &pair);
    bytes.drain(field_offset..field_offset + pair.len());
    // Independent MessagePack map-length repair leaves every other typed
    // value, including u128 durable identity fields, exactly as encoded.
    match bytes[container_offset] {
        marker @ 0x81..=0x8f => bytes[container_offset] = marker - 1,
        0xde => {
            let count = u16::from_be_bytes(
                bytes[container_offset + 1..container_offset + 3]
                    .try_into()
                    .unwrap(),
            );
            bytes[container_offset + 1..container_offset + 3]
                .copy_from_slice(&(count - 1).to_be_bytes());
        }
        0xdf => {
            let count = u32::from_be_bytes(
                bytes[container_offset + 1..container_offset + 5]
                    .try_into()
                    .unwrap(),
            );
            bytes[container_offset + 1..container_offset + 5]
                .copy_from_slice(&(count - 1).to_be_bytes());
        }
        marker => panic!("expected actual named map, got {marker:x}"),
    }
}
#[test]
fn recorded_component_checkpoint_header_tamper_refuses_without_defaults_or_foreign_binding() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "actual-header-source");
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let payload = &native.bytes()[CHECKPOINT_MAGIC.len()..];
    let original: CheckpointWire<'_> = rmp_serde::from_slice(payload).unwrap();
    decode_checkpoint(execution, &encode(&original)).unwrap();
    for change in 0..6 {
        let mut wire: CheckpointWire<'_> = rmp_serde::from_slice(payload).unwrap();
        let mut bytes = encode(&wire);
        let mut components = wire.manifest.roots[0].recorded_components.clone();
        match change {
            0 => omit(
                &mut bytes,
                &wire.manifest,
                "retained_empty_components",
                &wire.manifest.retained_empty_components,
            ),
            1 => omit(
                &mut bytes,
                &wire.manifest.roots[0],
                "recorded_components",
                &wire.manifest.roots[0].recorded_components,
            ),
            2 => {
                let duplicate = components[0].clone();
                components.push(duplicate);
            }
            3..=5 => {
                // The descriptor alone has no opaque u128 record inventories.
                let mut descriptor = serde_json::to_value(&components[0]).unwrap();
                match change {
                    3 => descriptor["runtime_instance_id"] = serde_json::json!(42),
                    4 => {
                        let byte = &mut descriptor["reference"]["target"]["Basis"]["roots"]
                            ["truth_root"][0];
                        *byte = serde_json::json!(byte.as_u64().unwrap() ^ 1);
                    }
                    5 => {
                        let byte = &mut descriptor["schema_commitment"][0];
                        *byte = serde_json::json!(byte.as_u64().unwrap() ^ 1);
                    }
                    _ => unreachable!(),
                }
                components[0] = serde_json::from_value(descriptor).unwrap();
            }
            _ => unreachable!(),
        }
        if change >= 2 {
            let mut manifest = wire.manifest.ordinary_owned_draft();
            let mut roots = manifest.roots.iter().cloned().collect::<Vec<_>>();
            let original = &roots[0];
            roots[0] = crate::durability::data::RelationalCheckpointRoot::from_ordinary_parts(
                original.commit_id,
                components,
                original.schema_carrier_digest,
                original.regions.clone(),
                original.content.clone(),
            );
            manifest.roots = crate::durability::data::CheckpointRoots::from_ordinary(roots);
            wire.manifest = manifest.seal_ordinary();
            bytes = encode(&wire);
        }
        let error = expect_durability_failure(decode_checkpoint(execution, &bytes).unwrap_err());
        assert_eq!(
            error.class,
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
            "header mutation {change}: {error:?}"
        );
        assert!(
            error.detail.contains("component") || error.detail.contains("recorded_components"),
            "header mutation {change}: {error:?}"
        );
    }
}
