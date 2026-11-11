//! Concrete portable declaration backing; nested FieldKey storage is separate.
use super::PortableFieldDeclaration;
use std::alloc::Layout;

pub(super) fn bytes(capacity: usize) -> Option<usize> {
    Some(
        Layout::array::<PortableFieldDeclaration>(capacity)
            .ok()?
            .size(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspects::portable::contract_candidate::{
        PortableAspectContract, PortableAspectEvolution, PortableAspectShape,
        PortableFieldRequirement,
    };
    use crate::aspects::{
        AbsenceLaw, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, FieldKey,
    };
    use crate::values::ScalarAspectType;

    #[test]
    fn roster_quote_uses_actual_private_slot_capacity_and_checks_layout_overflow() {
        for capacity in [0, 1, 3, 8, 17] {
            assert_eq!(
                PortableAspectContract::decode_field_roster_storage_bytes(capacity),
                Some(capacity * size_of::<PortableFieldDeclaration>())
            );
        }
        assert!(PortableAspectContract::decode_field_roster_storage_bytes(usize::MAX).is_none());
        let beyond_layout = (isize::MAX as usize / size_of::<PortableFieldDeclaration>()) + 1;
        assert!(PortableAspectContract::decode_field_roster_storage_bytes(beyond_layout).is_none());
    }

    #[test]
    fn field_count_excludes_spare_capacity_while_roster_quote_excludes_nested_keys() {
        let scalar = AspectContract::scalar(
            AspectKey::new("count").unwrap(),
            AspectIdentity(9),
            AspectContractRevision(1),
            ScalarAspectType::UInt64,
        );
        let mut candidate = PortableAspectContract::from_contract(&scalar);
        assert_eq!(candidate.struct_field_count(), 0);
        let mut name = String::with_capacity(512);
        name.push_str("wide_key");
        let mut fields = Vec::with_capacity(8);
        fields.push(PortableFieldDeclaration {
            key: FieldKey::new(name).unwrap(),
            value_type: ScalarAspectType::UInt64,
            requirement: PortableFieldRequirement::Required,
            absence: AbsenceLaw::Required,
            evolution: PortableAspectEvolution::Frozen,
            codec: None,
        });
        let capacity = fields.capacity();
        assert!(fields[0].key.owned_allocation_capacity_bytes() >= 512);
        candidate.shape = PortableAspectShape::Struct(fields);
        assert_eq!(candidate.struct_field_count(), 1);
        assert_eq!(
            PortableAspectContract::decode_field_roster_storage_bytes(capacity),
            Some(capacity * size_of::<PortableFieldDeclaration>())
        );
    }
}
