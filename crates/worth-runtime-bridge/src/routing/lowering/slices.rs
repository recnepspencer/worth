use std::sync::Arc;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, ProjectionMask,
};

use crate::mapping::SubscriptionSliceKind;
use crate::mapping::TruthDeltaSurfaceKind;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::routing::matching::FineGrainedMatchStatus;
use crate::routing::surfaces::TruthDeltaSurface;
use crate::snapshot::SnapshotReadContract;
use crate::subscription::{
    subscription_slice_target_identity, BridgeSubscriptionSliceTargetIdentity,
};

use super::slice_support::{
    assert_subscription_slice_target_shape, subscription_committed_patch_target,
    subscription_slice_canonical_basis,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionSlice {
    entity_identity: Arc<str>,
    relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    aspect_locator: AspectLocator,
    field_locator: Option<AspectFieldLocator>,
    projection_mask: AspectMask<ProjectionMask>,
    slice_target_identity: BridgeSubscriptionSliceTargetIdentity,
    snapshot_read_contract: SnapshotReadContract,
    native_target_basis: Arc<str>,
    surface_kind: TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
    match_status: FineGrainedMatchStatus,
    canonical_basis: Arc<str>,
}

impl BridgeSubscriptionSlice {
    pub(crate) fn from_truth_delta_surface(
        surface: &TruthDeltaSurface,
        snapshot_read_contract: SnapshotReadContract,
        slice_kind: SubscriptionSliceKind,
        match_status: FineGrainedMatchStatus,
    ) -> Self {
        let projection_mask = surface.projection_mask().clone();
        let slice_target_identity = subscription_slice_target_identity(
            surface.entity_identity(),
            surface.target(),
            &slice_kind,
        );
        let canonical_basis = subscription_slice_canonical_basis(
            slice_target_identity.as_str(),
            snapshot_read_contract.canonical_basis(),
            match_status,
        );
        Self {
            entity_identity: Arc::from(surface.entity_identity()),
            relational_record_identity: surface.relational_record_identity_parts(),
            aspect_locator: surface.aspect_locator().clone(),
            field_locator: surface.field_locator().cloned(),
            projection_mask,
            slice_target_identity,
            snapshot_read_contract,
            native_target_basis: Arc::from(surface.native_target_basis()),
            surface_kind: surface.surface_kind(),
            slice_kind,
            match_status,
            canonical_basis: canonical_basis.into(),
        }
    }

    pub(crate) fn from_continuity_parts(
        entity_identity: impl Into<Arc<str>>,
        aspect_locator: AspectLocator,
        field_locator: Option<AspectFieldLocator>,
        projection_mask: AspectMask<ProjectionMask>,
        snapshot_read_contract: SnapshotReadContract,
        surface_kind: TruthDeltaSurfaceKind,
        slice_kind: SubscriptionSliceKind,
        match_status: FineGrainedMatchStatus,
    ) -> Self {
        let entity_identity = entity_identity.into();
        assert_subscription_slice_target_shape(
            field_locator.as_ref(),
            &projection_mask,
            surface_kind,
        );
        let committed_patch_target = subscription_committed_patch_target(
            &aspect_locator,
            field_locator.as_ref(),
            &projection_mask,
            surface_kind,
        );
        let native_target_basis = committed_patch_target.canonical_basis();
        let slice_target_identity = subscription_slice_target_identity(
            entity_identity.as_ref(),
            &committed_patch_target,
            &slice_kind,
        );
        let canonical_basis = subscription_slice_canonical_basis(
            slice_target_identity.as_str(),
            snapshot_read_contract.canonical_basis(),
            match_status,
        );
        Self {
            entity_identity,
            relational_record_identity: None,
            aspect_locator,
            field_locator,
            projection_mask,
            slice_target_identity,
            snapshot_read_contract,
            native_target_basis: Arc::from(native_target_basis),
            surface_kind,
            slice_kind,
            match_status,
            canonical_basis: canonical_basis.into(),
        }
    }

    pub fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
    }

    pub fn relational_record_identity_parts(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        self.relational_record_identity
    }

    pub fn aspect_key(&self) -> &AspectKey {
        self.aspect_locator.aspect_key()
    }

    pub fn aspect_locator(&self) -> &AspectLocator {
        &self.aspect_locator
    }

    pub fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.field_locator.as_ref()
    }

    pub fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    #[cfg(test)]
    pub(crate) fn slice_target_identity(&self) -> &BridgeSubscriptionSliceTargetIdentity {
        &self.slice_target_identity
    }

    pub fn snapshot_read_contract(&self) -> &SnapshotReadContract {
        &self.snapshot_read_contract
    }

    pub(crate) fn native_target_basis(&self) -> &str {
        self.native_target_basis.as_ref()
    }

    pub fn surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.surface_kind
    }

    pub fn slice_kind(&self) -> &SubscriptionSliceKind {
        &self.slice_kind
    }

    pub fn match_status(&self) -> FineGrainedMatchStatus {
        self.match_status
    }

    pub(crate) fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

impl PartialOrd for BridgeSubscriptionSlice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BridgeSubscriptionSlice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canonical_basis.cmp(&other.canonical_basis)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSubscriptionSlices {
    slices: Arc<[BridgeSubscriptionSlice]>,
}

impl CanonicalSubscriptionSlices {
    pub(crate) fn new(slices: Vec<BridgeSubscriptionSlice>) -> Self {
        Self {
            slices: Arc::from(slices),
        }
    }

    pub fn slices(&self) -> &[BridgeSubscriptionSlice] {
        &self.slices
    }

    pub(crate) fn shared(&self) -> &Arc<[BridgeSubscriptionSlice]> {
        &self.slices
    }

    pub fn len(&self) -> usize {
        self.slices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, AspectLocator, AspectMask, CanonicalFieldPath, FieldKey,
        LocatorAuthority, ProjectionMask, ScalarAspectType,
    };

    use super::BridgeSubscriptionSlice;
    use crate::mapping::{SubscriptionSliceKind, TruthDeltaSurfaceKind};
    use crate::routing::matching::FineGrainedMatchStatus;
    use crate::snapshot::SnapshotReadContract;
    use crate::subscription::NormalizedSubscriptionSliceIntent;

    #[test]
    fn declaration_intent_and_lowered_slice_share_admitted_target_mask_basis() {
        let field_intent = NormalizedSubscriptionSliceIntent::try_new_entity_field(
            "user",
            aspect_key("profile"),
            field_key("name"),
            SubscriptionSliceKind::SignalField,
        )
        .expect("field intent should validate");
        let field_slice = BridgeSubscriptionSlice::from_continuity_parts(
            "user",
            aspect_locator("profile"),
            Some(field_locator("profile", "name")),
            AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(field_key("name"))]),
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            FineGrainedMatchStatus::Matched,
        );

        assert_eq!(
            field_intent.native_target_basis(),
            field_slice.native_target_basis()
        );
        assert_eq!(
            field_intent.projection_mask(),
            field_slice.projection_mask()
        );
        assert_eq!(
            field_intent.slice_target_identity(),
            field_slice.slice_target_identity()
        );
        assert!(field_slice
            .slice_target_identity()
            .as_str()
            .starts_with("subscription-slice-target:sha256:"));
        assert!(!field_slice
            .canonical_basis()
            .contains("committed-patch-target"));
        assert!(field_slice
            .native_target_basis()
            .contains("projection-mask="));

        for (whole_intent, surface_kind, slice_kind) in [
            (
                NormalizedSubscriptionSliceIntent::try_new_entity_relation_endpoint(
                    "user",
                    aspect_key("profile"),
                    SubscriptionSliceKind::SignalLens,
                )
                .expect("relation-endpoint intent should validate"),
                TruthDeltaSurfaceKind::EntityRelationEndpoint,
                SubscriptionSliceKind::SignalLens,
            ),
            (
                NormalizedSubscriptionSliceIntent::try_new_entity_region(
                    "user",
                    aspect_key("profile"),
                    SubscriptionSliceKind::SignalRegion,
                )
                .expect("region intent should validate"),
                TruthDeltaSurfaceKind::EntityRegion,
                SubscriptionSliceKind::SignalRegion,
            ),
            (
                NormalizedSubscriptionSliceIntent::try_new_entity_partition(
                    "user",
                    aspect_key("profile"),
                    SubscriptionSliceKind::SignalPartition,
                )
                .expect("partition intent should validate"),
                TruthDeltaSurfaceKind::EntityPartition,
                SubscriptionSliceKind::SignalPartition,
            ),
            (
                NormalizedSubscriptionSliceIntent::try_new_entity_facet(
                    "user",
                    aspect_key("profile"),
                    SubscriptionSliceKind::SignalFacet,
                )
                .expect("facet intent should validate"),
                TruthDeltaSurfaceKind::EntityFacet,
                SubscriptionSliceKind::SignalFacet,
            ),
        ] {
            let whole_slice = BridgeSubscriptionSlice::from_continuity_parts(
                "user",
                aspect_locator("profile"),
                None,
                AspectMask::whole_aspect(),
                SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
                surface_kind,
                slice_kind,
                FineGrainedMatchStatus::Matched,
            );

            assert_eq!(
                whole_intent.native_target_basis(),
                whole_slice.native_target_basis()
            );
            assert_eq!(
                whole_intent.projection_mask(),
                whole_slice.projection_mask()
            );
            assert_eq!(
                whole_intent.slice_target_identity(),
                whole_slice.slice_target_identity()
            );
            assert!(whole_slice.projection_mask().is_whole_aspect());
        }
    }

    fn aspect_key(value: &str) -> AspectKey {
        AspectKey::new(value).expect("valid subscription slice aspect key")
    }

    fn aspect_locator(value: &str) -> AspectLocator {
        AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(value))
    }

    fn field_key(value: &str) -> FieldKey {
        FieldKey::new(value.to_owned()).expect("valid subscription slice field key")
    }

    fn field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
        AspectFieldLocator::from_aspect(
            aspect_locator(aspect),
            CanonicalFieldPath::single(field_key(field)),
        )
    }
}
