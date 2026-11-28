use crate::mapping::{SubscriptionSliceKind, TruthDeltaSurfaceKind};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, CanonicalFieldPath, FieldKey,
    LocatorAuthority, ScalarAspectType,
};

pub(in crate::builder::tests) fn native_prior_field_slice(
    entity_identity: &str,
    aspect: &str,
    field: &str,
) -> crate::continuity::PriorSubscriptionSlice {
    let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(aspect));
    let field_locator = AspectFieldLocator::from_aspect(
        aspect_locator.clone(),
        CanonicalFieldPath::single(
            FieldKey::new(field.to_owned()).expect("builder test field key should be valid"),
        ),
    );
    let projection_mask = AspectMask::new([field_locator.field_path().clone()]);
    let slice = crate::routing::BridgeSubscriptionSlice::from_continuity_parts(
        entity_identity.into(),
        aspect_locator,
        Some(field_locator),
        projection_mask,
        crate::snapshot::SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        crate::routing::FineGrainedMatchStatus::Matched,
    );

    crate::continuity::PriorSubscriptionSlice::new(
        crate::routing::BridgeSubscriptionSliceIdentity::admit_bridge_owned("slice:test"),
        &slice,
    )
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid builder test aspect key")
}
