use crate::merge::aspect_components::MergeAspectComponent;
use worth_relational_contracts::internal_port::aspect_wire::aspect_value_canonical_codec::writing::encode_aspect_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DeclaredIdentityDigest(pub(crate) u128);

pub(crate) fn declared_identity_signature(
    components: &[MergeAspectComponent],
) -> Option<DeclaredIdentityDigest> {
    let mut hash = FNV_OFFSET;
    for component in components {
        mix_identity_component(&mut hash, component)?;
    }
    Some(DeclaredIdentityDigest(hash))
}

const FNV_OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
const FNV_PRIME: u128 = 0x0000000001000000000000000000013B;

fn mix_identity_component(hash: &mut u128, component: &MergeAspectComponent) -> Option<()> {
    match component {
        MergeAspectComponent::AspectValue(value) => {
            mix_identity_bytes(hash, b"aspect_value");
            let encoded_value = encode_aspect_value(value);
            mix_identity_bytes(hash, &encoded_value);
        }
        MergeAspectComponent::StructValue(value) => {
            mix_identity_bytes(hash, b"struct_value");
            for (field, field_value) in value.fields() {
                mix_identity_bytes(hash, field.as_str().as_bytes());
                let encoded_value = encode_aspect_value(field_value);
                mix_identity_bytes(hash, &encoded_value);
            }
        }
        MergeAspectComponent::EntityEndpoint(entity_id) => {
            mix_identity_bytes(hash, b"endpoint");
            mix_identity_bytes(hash, &entity_id.partition_id.0.to_le_bytes());
            mix_identity_bytes(hash, &entity_id.local_slot.0.to_le_bytes());
            mix_identity_bytes(hash, &entity_id.generation.0.to_le_bytes());
        }
        MergeAspectComponent::Lifecycle(state) => {
            mix_identity_bytes(hash, b"lifecycle");
            mix_identity_bytes(hash, format!("{state:?}").as_bytes());
        }
    }
    Some(())
}

fn mix_identity_bytes(hash: &mut u128, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= *byte as u128;
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
    *hash ^= 0xff_u128;
    *hash = hash.wrapping_mul(FNV_PRIME);
}
