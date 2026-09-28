use std::sync::Arc;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, ProjectionMask,
};

use crate::mapping::SubscriptionSliceKind;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::routing::{
    BridgeSubscriptionSlice, BridgeSubscriptionSliceIdentity, FineGrainedMatchStatus,
};
use crate::snapshot::SnapshotReadContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorSubscriptionSlice {
    prior_subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    entity_identity: Arc<str>,
    relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    aspect_locator: AspectLocator,
    field_locator: Option<AspectFieldLocator>,
    projection_mask: AspectMask<ProjectionMask>,
    snapshot_read_contract: SnapshotReadContract,
    native_target_basis: Arc<str>,
    prior_slice_canonical_basis: Arc<str>,
    surface_kind: crate::mapping::TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
    match_status: FineGrainedMatchStatus,
}

impl PriorSubscriptionSlice {
    pub(crate) fn new(
        prior_subscription_slice_identity: BridgeSubscriptionSliceIdentity,
        slice: &BridgeSubscriptionSlice,
    ) -> Self {
        Self {
            prior_subscription_slice_identity,
            entity_identity: Arc::from(slice.entity_identity()),
            relational_record_identity: slice.relational_record_identity_parts(),
            aspect_locator: slice.aspect_locator().clone(),
            field_locator: slice.field_locator().cloned(),
            projection_mask: slice.projection_mask().clone(),
            snapshot_read_contract: slice.snapshot_read_contract().clone(),
            native_target_basis: Arc::from(slice.native_target_basis()),
            prior_slice_canonical_basis: Arc::from(slice.canonical_basis()),
            surface_kind: slice.surface_kind(),
            slice_kind: slice.slice_kind().clone(),
            match_status: slice.match_status(),
        }
    }

    pub fn prior_subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.prior_subscription_slice_identity
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

    pub(crate) fn aspect_locator(&self) -> &AspectLocator {
        &self.aspect_locator
    }

    pub(crate) fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.field_locator.as_ref()
    }

    pub(crate) fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    pub(crate) fn snapshot_read_contract(&self) -> &SnapshotReadContract {
        &self.snapshot_read_contract
    }

    #[cfg(test)]
    pub(crate) fn native_target_basis(&self) -> &str {
        self.native_target_basis.as_ref()
    }

    pub(crate) fn prior_slice_canonical_basis(&self) -> &str {
        self.prior_slice_canonical_basis.as_ref()
    }

    pub(crate) fn surface_kind(&self) -> crate::mapping::TruthDeltaSurfaceKind {
        self.surface_kind
    }

    pub fn slice_kind(&self) -> SubscriptionSliceKind {
        self.slice_kind.clone()
    }

    pub fn match_status(&self) -> FineGrainedMatchStatus {
        self.match_status
    }

    pub fn canonical_basis(&self) -> String {
        format!(
            "prior-slice|slice-set={}|slice={}|entity={}|read-contract={}|kind={:?}|match={:?}",
            self.prior_subscription_slice_identity.as_str(),
            self.prior_slice_canonical_basis(),
            self.entity_identity(),
            self.snapshot_read_contract().canonical_basis(),
            self.slice_kind(),
            self.match_status(),
        )
    }

    pub(crate) fn logical_dedup_basis(&self) -> String {
        format!(
            "prior-slice-logical|entity={}|slice={}|read-contract={}|kind={:?}|match={:?}",
            self.entity_identity(),
            self.prior_slice_canonical_basis(),
            self.snapshot_read_contract().canonical_basis(),
            self.slice_kind(),
            self.match_status(),
        )
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, AspectLocator, AspectMask, CanonicalFieldPath, FieldKey,
        LocatorAuthority, ScalarAspectType,
    };

    use super::PriorSubscriptionSlice;
    use crate::mapping::{SubscriptionSliceKind, TruthDeltaSurfaceKind};
    use crate::routing::{
        BridgeSubscriptionSlice, BridgeSubscriptionSliceIdentity, FineGrainedMatchStatus,
    };
    use crate::snapshot::SnapshotReadContract;

    #[test]
    fn prior_slice_bases_consume_slice_proof_not_native_target_basis() {
        let slice = BridgeSubscriptionSlice::from_continuity_parts(
            "entity-1",
            aspect_locator("profile"),
            Some(field_locator("profile", "name")),
            AspectMask::new([CanonicalFieldPath::single(field_key("name"))]),
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            FineGrainedMatchStatus::Matched,
        );
        let prior_slice = PriorSubscriptionSlice::new(
            BridgeSubscriptionSliceIdentity::admit_bridge_owned("slice-set:a"),
            &slice,
        );

        let canonical_basis = prior_slice.canonical_basis();
        assert!(
            canonical_basis.contains(slice.canonical_basis()),
            "prior-slice canonical basis must consume retained slice proof: {canonical_basis}"
        );
        assert!(
            !canonical_basis.contains(prior_slice.native_target_basis()),
            "prior-slice canonical basis must not reopen native target basis: {canonical_basis}"
        );

        let logical_basis = prior_slice.logical_dedup_basis();
        assert!(
            logical_basis.contains(slice.canonical_basis()),
            "prior-slice logical dedup basis must consume retained slice proof: {logical_basis}"
        );
        assert!(
            !logical_basis.contains(prior_slice.native_target_basis()),
            "prior-slice logical dedup basis must not reopen native target basis: {logical_basis}"
        );
    }

    fn aspect_key(value: &str) -> AspectKey {
        AspectKey::new(value).expect("valid prior-slice test aspect key")
    }

    fn aspect_locator(value: &str) -> AspectLocator {
        AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(value))
    }

    fn field_key(value: &str) -> FieldKey {
        FieldKey::new(value.to_owned()).expect("valid prior-slice test field key")
    }

    fn field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
        AspectFieldLocator::from_aspect(
            aspect_locator(aspect),
            CanonicalFieldPath::single(field_key(field)),
        )
    }
}
