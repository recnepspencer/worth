use worth_foundational::facade::{
    AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_runtime_bridge::facade::{BridgeCommittedPatchItem, BridgeCommittedPatchTarget};


fn main() {
    let _item = BridgeCommittedPatchItem {
        entity_identity: sealed_authority_placeholder(),
        target: BridgeCommittedPatchTarget::entity_field_path(
            AspectLocator::new(
                LocatorAuthority::Authoritative,
                AspectKey::new("profile").expect("valid native aspect key"),
            ),
            CanonicalFieldPath::single(
                FieldKey::new("name".to_owned()).expect("valid native field key"),
            ),
        ),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
