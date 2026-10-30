use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, CanonicalFieldPath, FieldKey,
    LocatorAuthority, ProjectionMask,
};

use crate::identity::{BridgeIdentity, SubscriptionSliceTargetIdentityTag};
use crate::input::envelope::BridgeCommittedPatchTarget;
use crate::mapping::SubscriptionSliceKind;

pub(crate) type BridgeSubscriptionSliceTargetIdentity =
    BridgeIdentity<SubscriptionSliceTargetIdentityTag>;
enum SubscriptionTargetMaskIdentityTag {}
type BridgeSubscriptionTargetMaskIdentity = BridgeIdentity<SubscriptionTargetMaskIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeSubscriptionDeliveryIntentClass {
    None,
    CanonicalMeaningfulChange,
}

impl BridgeSubscriptionDeliveryIntentClass {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CanonicalMeaningfulChange => "canonical_meaningful_change",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizedSubscriptionSliceIntentErrorKind {
    EmptyEntityIdentity,
    MissingFieldLocator,
    UnexpectedFieldLocator,
    ProjectionMaskTargetMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSubscriptionSliceIntentError {
    kind: NormalizedSubscriptionSliceIntentErrorKind,
}

impl NormalizedSubscriptionSliceIntentError {
    const fn new(kind: NormalizedSubscriptionSliceIntentErrorKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> NormalizedSubscriptionSliceIntentErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSubscriptionSliceIntent {
    entity_identity: Arc<str>,
    aspect_locator: AspectLocator,
    field_locator: Option<AspectFieldLocator>,
    projection_mask: AspectMask<ProjectionMask>,
    slice_target_identity: BridgeSubscriptionSliceTargetIdentity,
    native_target_basis: Arc<str>,
    surface_kind: crate::mapping::TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
    canonical_basis: Arc<str>,
}

impl NormalizedSubscriptionSliceIntent {
    pub fn try_new_entity_field(
        entity_identity: impl Into<Arc<str>>,
        aspect_key: AspectKey,
        field_key: FieldKey,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key);
        let field_locator = AspectFieldLocator::from_aspect(
            aspect_locator.clone(),
            CanonicalFieldPath::single(field_key),
        );
        let projection_mask = AspectMask::new([field_locator.field_path().clone()]);
        Self::try_new_native(
            entity_identity,
            aspect_locator,
            Some(field_locator),
            projection_mask,
            crate::mapping::TruthDeltaSurfaceKind::EntityField,
            slice_kind,
        )
    }

    pub fn try_new_entity_relation_endpoint(
        entity_identity: impl Into<Arc<str>>,
        aspect_key: AspectKey,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key);
        Self::try_new_native(
            entity_identity,
            aspect_locator,
            None,
            AspectMask::whole_aspect(),
            crate::mapping::TruthDeltaSurfaceKind::EntityRelationEndpoint,
            slice_kind,
        )
    }

    pub fn try_new_entity_region(
        entity_identity: impl Into<Arc<str>>,
        aspect_key: AspectKey,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key);
        Self::try_new_native(
            entity_identity,
            aspect_locator,
            None,
            AspectMask::whole_aspect(),
            crate::mapping::TruthDeltaSurfaceKind::EntityRegion,
            slice_kind,
        )
    }

    pub fn try_new_entity_partition(
        entity_identity: impl Into<Arc<str>>,
        aspect_key: AspectKey,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key);
        Self::try_new_native(
            entity_identity,
            aspect_locator,
            None,
            AspectMask::whole_aspect(),
            crate::mapping::TruthDeltaSurfaceKind::EntityPartition,
            slice_kind,
        )
    }

    pub fn try_new_entity_facet(
        entity_identity: impl Into<Arc<str>>,
        aspect_key: AspectKey,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, aspect_key);
        Self::try_new_native(
            entity_identity,
            aspect_locator,
            None,
            AspectMask::whole_aspect(),
            crate::mapping::TruthDeltaSurfaceKind::EntityFacet,
            slice_kind,
        )
    }

    pub fn try_new_native(
        entity_identity: impl Into<Arc<str>>,
        aspect_locator: AspectLocator,
        field_locator: Option<AspectFieldLocator>,
        projection_mask: AspectMask<ProjectionMask>,
        surface_kind: crate::mapping::TruthDeltaSurfaceKind,
        slice_kind: SubscriptionSliceKind,
    ) -> Result<Self, NormalizedSubscriptionSliceIntentError> {
        let entity_identity = entity_identity.into();
        if entity_identity.is_empty() {
            return Err(NormalizedSubscriptionSliceIntentError::new(
                NormalizedSubscriptionSliceIntentErrorKind::EmptyEntityIdentity,
            ));
        }
        validate_slice_target_shape(field_locator.as_ref(), &projection_mask, surface_kind)?;
        let committed_patch_target = committed_patch_target_for_slice_intent(
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
        let canonical_basis =
            subscription_slice_intent_canonical_basis(slice_target_identity.as_str());
        Ok(Self {
            entity_identity,
            aspect_locator,
            field_locator,
            projection_mask,
            slice_target_identity,
            native_target_basis: Arc::from(native_target_basis),
            surface_kind,
            slice_kind,
            canonical_basis: canonical_basis.into(),
        })
    }

    pub fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
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

    pub(crate) fn slice_target_identity(&self) -> &BridgeSubscriptionSliceTargetIdentity {
        &self.slice_target_identity
    }

    pub fn native_target_basis(&self) -> &str {
        self.native_target_basis.as_ref()
    }

    pub fn surface_kind(&self) -> crate::mapping::TruthDeltaSurfaceKind {
        self.surface_kind
    }

    pub fn slice_kind(&self) -> &SubscriptionSliceKind {
        &self.slice_kind
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

impl PartialOrd for NormalizedSubscriptionSliceIntent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalizedSubscriptionSliceIntent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canonical_basis.cmp(&other.canonical_basis)
    }
}

fn validate_slice_target_shape(
    field_locator: Option<&AspectFieldLocator>,
    projection_mask: &AspectMask<ProjectionMask>,
    surface_kind: crate::mapping::TruthDeltaSurfaceKind,
) -> Result<(), NormalizedSubscriptionSliceIntentError> {
    match (surface_kind, field_locator) {
        (crate::mapping::TruthDeltaSurfaceKind::EntityField, Some(locator)) => {
            if projection_mask.paths() == std::slice::from_ref(locator.field_path()) {
                Ok(())
            } else {
                Err(NormalizedSubscriptionSliceIntentError::new(
                    NormalizedSubscriptionSliceIntentErrorKind::ProjectionMaskTargetMismatch,
                ))
            }
        }
        (crate::mapping::TruthDeltaSurfaceKind::EntityField, None) => {
            Err(NormalizedSubscriptionSliceIntentError::new(
                NormalizedSubscriptionSliceIntentErrorKind::MissingFieldLocator,
            ))
        }
        (_, Some(_)) => Err(NormalizedSubscriptionSliceIntentError::new(
            NormalizedSubscriptionSliceIntentErrorKind::UnexpectedFieldLocator,
        )),
        (_, None) => {
            if projection_mask.is_whole_aspect() {
                Ok(())
            } else {
                Err(NormalizedSubscriptionSliceIntentError::new(
                    NormalizedSubscriptionSliceIntentErrorKind::ProjectionMaskTargetMismatch,
                ))
            }
        }
    }
}

pub(crate) fn subscription_slice_target_identity(
    entity_identity: &str,
    committed_patch_target: &BridgeCommittedPatchTarget,
    slice_kind: &SubscriptionSliceKind,
) -> BridgeSubscriptionSliceTargetIdentity {
    let target_mask_identity = subscription_target_mask_identity(committed_patch_target);
    let basis = subscription_slice_target_identity_basis(
        entity_identity,
        &target_mask_identity,
        slice_kind,
    );
    let digest = Sha256::digest(basis.as_bytes());
    BridgeSubscriptionSliceTargetIdentity::admit_bridge_owned(format!(
        "subscription-slice-target:sha256:{digest:x}"
    ))
}

fn subscription_target_mask_identity(
    committed_patch_target: &BridgeCommittedPatchTarget,
) -> BridgeSubscriptionTargetMaskIdentity {
    let basis = format!(
        "subscription-target-mask|target={}",
        committed_patch_target.canonical_basis()
    );
    let digest = Sha256::digest(basis.as_bytes());
    BridgeSubscriptionTargetMaskIdentity::admit_bridge_owned(format!(
        "subscription-target-mask:sha256:{digest:x}"
    ))
}

fn subscription_slice_target_identity_basis(
    entity_identity: &str,
    target_mask_identity: &BridgeSubscriptionTargetMaskIdentity,
    slice_kind: &SubscriptionSliceKind,
) -> String {
    format!(
        "subscription-slice-target|entity={}|target-proof={}|slice-kind={}",
        entity_identity,
        target_mask_identity.as_str(),
        subscription_slice_kind_label(slice_kind),
    )
}

fn subscription_slice_intent_canonical_basis(slice_target_identity: &str) -> String {
    format!("subscription-slice-intent|slice-target={slice_target_identity}")
}

fn committed_patch_target_for_slice_intent(
    aspect_locator: &AspectLocator,
    field_locator: Option<&AspectFieldLocator>,
    projection_mask: &AspectMask<ProjectionMask>,
    surface_kind: crate::mapping::TruthDeltaSurfaceKind,
) -> BridgeCommittedPatchTarget {
    debug_assert!(
        validate_slice_target_shape(field_locator, projection_mask, surface_kind).is_ok(),
        "subscription slice target proof requires an admitted native target shape"
    );
    BridgeCommittedPatchTarget::from_admitted_target_shape(
        aspect_locator.clone(),
        field_locator.cloned(),
        projection_mask,
        surface_kind,
    )
}

fn subscription_slice_kind_label(slice_kind: &SubscriptionSliceKind) -> &'static str {
    match slice_kind {
        SubscriptionSliceKind::SignalAspect => "signal-aspect",
        SubscriptionSliceKind::SignalField => "signal-field",
        SubscriptionSliceKind::SignalLens => "signal-lens",
        SubscriptionSliceKind::SignalRegion => "signal-region",
        SubscriptionSliceKind::SignalPartition => "signal-partition",
        SubscriptionSliceKind::SignalFacet => "signal-facet",
        SubscriptionSliceKind::SignalLifecycle => "signal-lifecycle",
        SubscriptionSliceKind::RegisteredCoarseWidening => "registered-coarse-widening",
    }
}

#[cfg(test)]
#[path = "slice_intent/identity_tests.rs"]
mod identity_tests;
