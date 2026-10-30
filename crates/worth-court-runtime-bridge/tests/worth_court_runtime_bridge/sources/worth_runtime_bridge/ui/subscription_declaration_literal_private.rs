use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeSubscriptionDeclaration, BridgeSubscriptionDeclarationFamilyKind,
    BridgeSubscriptionDeclarationIdentity, BridgeSubscriptionDeliveryIntentClass,
    NormalizedSubscriptionSliceIntent,
};

fn main() {
    let _ = BridgeSubscriptionDeclaration {
        declaration_identity: BridgeSubscriptionDeclarationIdentity::new("subscription:literal"),
        requested_family_kind: BridgeSubscriptionDeclarationFamilyKind::DetailExact,
        delivery_intent_class: BridgeSubscriptionDeliveryIntentClass::None,
        normalized_slice_intents: Arc::<[NormalizedSubscriptionSliceIntent]>::from([]),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
