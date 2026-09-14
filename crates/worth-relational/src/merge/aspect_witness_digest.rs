use sha2::{Digest, Sha256};

use crate::aspect_wire::{encode_aspect_value, encode_string, encode_u32};
use crate::merge::aspect_components::MergeAspectComponent;
use crate::storage::data::RecordLifecycleState;
use worth_foundational::facade::AspectKey;

pub(crate) fn canonical_aspect_witness_digest(
    aspect_key: &AspectKey,
    source_component: &MergeAspectComponent,
    target_component: &MergeAspectComponent,
) -> String {
    sha256_hex(&canonical_aspect_witness_bytes(
        aspect_key,
        source_component,
        target_component,
    ))
}

fn canonical_aspect_witness_bytes(
    aspect_key: &AspectKey,
    source_component: &MergeAspectComponent,
    target_component: &MergeAspectComponent,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "merge.aspect_witness.v1");
    encode_string(&mut bytes, aspect_key.as_str());
    encode_merge_component(&mut bytes, "source", source_component);
    encode_merge_component(&mut bytes, "target", target_component);
    bytes
}

fn encode_merge_component(
    bytes: &mut Vec<u8>,
    side: &'static str,
    component: &MergeAspectComponent,
) {
    encode_string(bytes, side);
    match component {
        MergeAspectComponent::AspectValue(value) => {
            encode_string(bytes, "aspect_value");
            encode_length_prefixed_bytes(bytes, &encode_aspect_value(value));
        }
        MergeAspectComponent::StructValue(value) => {
            encode_string(bytes, "struct_value");
            encode_u32(bytes, value.fields().count() as u32);
            for (field, field_value) in value.fields() {
                encode_string(bytes, field.as_str());
                encode_length_prefixed_bytes(bytes, &encode_aspect_value(field_value));
            }
        }
        MergeAspectComponent::EntityEndpoint(entity_id) => {
            encode_string(bytes, "entity_endpoint");
            bytes.extend_from_slice(&entity_id.partition_id.0.to_le_bytes());
            bytes.extend_from_slice(&entity_id.local_slot.0.to_le_bytes());
            bytes.extend_from_slice(&entity_id.generation.0.to_le_bytes());
        }
        MergeAspectComponent::Lifecycle(state) => {
            encode_string(bytes, "lifecycle");
            bytes.push(lifecycle_state_tag(*state));
        }
    }
}

fn encode_length_prefixed_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    encode_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value);
}

fn lifecycle_state_tag(state: RecordLifecycleState) -> u8 {
    match state {
        RecordLifecycleState::Live => 0,
        RecordLifecycleState::DeletedRetained => 1,
        RecordLifecycleState::RetainedDanglingForAudit => 2,
        RecordLifecycleState::PinnedBySnapshot => 3,
        RecordLifecycleState::PinnedByBranch => 4,
        RecordLifecycleState::PinnedByReplayRetention => 5,
        RecordLifecycleState::Reclaimable => 6,
        RecordLifecycleState::Reusable => 7,
        RecordLifecycleState::MaterializationUnavailable => 8,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{aspects, AspectValue, CanonicalF32};

    use super::*;

    #[test]
    fn aspect_witness_digest_uses_canonical_aspect_value_bytes() {
        let aspect_key = AspectKey::new("merge.name").expect("valid aspect key");
        let digest = canonical_aspect_witness_digest(
            &aspect_key,
            &MergeAspectComponent::AspectValue(AspectValue::String("same".into())),
            &MergeAspectComponent::AspectValue(AspectValue::String("same".into())),
        );

        let mut expected_bytes = Vec::new();
        encode_string(&mut expected_bytes, "merge.aspect_witness.v1");
        encode_string(&mut expected_bytes, "merge.name");
        encode_string(&mut expected_bytes, "source");
        encode_string(&mut expected_bytes, "aspect_value");
        encode_length_prefixed_bytes(
            &mut expected_bytes,
            &encode_aspect_value(&AspectValue::String("same".into())),
        );
        encode_string(&mut expected_bytes, "target");
        encode_string(&mut expected_bytes, "aspect_value");
        encode_length_prefixed_bytes(
            &mut expected_bytes,
            &encode_aspect_value(&AspectValue::String("same".into())),
        );

        assert_eq!(digest, sha256_hex(&expected_bytes));
    }

    #[test]
    fn struct_witness_digest_canonicalizes_field_values_without_serde_tags() {
        let aspect_key = AspectKey::new("merge.summary").expect("valid aspect key");
        let struct_value = aspects()
            .vocabulary()
            .struct_value()
            .with_field("count", AspectValue::Int64(9))
            .with_field("ratio", AspectValue::Float32(CanonicalF32::from_f32(1.25)))
            .finish()
            .expect("struct value");
        let witness_bytes = canonical_aspect_witness_bytes(
            &aspect_key,
            &MergeAspectComponent::StructValue(struct_value.clone()),
            &MergeAspectComponent::StructValue(struct_value),
        );

        let digest = sha256_hex(&witness_bytes);
        assert_eq!(digest.len(), 64);
        assert!(witness_bytes
            .windows([5, 9, 0, 0, 0, 0, 0, 0, 0].len())
            .any(|window| window == [5, 9, 0, 0, 0, 0, 0, 0, 0].as_slice()));
        assert!(witness_bytes
            .windows([10, 0, 0, 160, 63].len())
            .any(|window| window == [10, 0, 0, 160, 63].as_slice()));
    }
}
