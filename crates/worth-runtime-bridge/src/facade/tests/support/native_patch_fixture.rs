use crate::input::envelope::BridgeCommittedPatchItem;
use worth_foundational::facade::{AspectKey, AspectLocator, CanonicalFieldPath, FieldKey};

pub(in crate::facade::tests) fn native_profile_name_patch_item() -> BridgeCommittedPatchItem {
    BridgeCommittedPatchItem::with_target(
        "entity-1",
        crate::facade::BridgeCommittedPatchTarget::entity_field_path(
            AspectLocator::new(
                worth_foundational::facade::LocatorAuthority::Authoritative,
                AspectKey::new("profile").expect("valid bridge patch aspect key"),
            ),
            CanonicalFieldPath::single(
                FieldKey::new("name".to_owned()).expect("valid foundational field key"),
            ),
        ),
    )
}
