use std::sync::Arc;

use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};

use crate::identity::{
    BridgeIdentity, FrozenMappingRegistrationIdentityTag, MappingIdTag, SignalInvalidationScopeTag,
};
use crate::mapping::widening::BridgeMappingWideningClass;
use crate::mapping::TruthDeltaSurfaceKind;
use crate::snapshot::SnapshotReadContract;

pub type BridgeMappingId = BridgeIdentity<MappingIdTag>;
pub type BridgeFrozenMappingRegistrationIdentity =
    BridgeIdentity<FrozenMappingRegistrationIdentityTag>;

impl BridgeMappingId {
    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MappingSelector {
    Any,
    Exact(Arc<str>),
}

impl MappingSelector {
    pub fn exact(value: impl Into<Arc<str>>) -> Self {
        Self::Exact(value.into())
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub(crate) fn matches(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => expected.as_ref() == value,
        }
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::Exact(left), Self::Exact(right)) => left == right,
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AspectKeySelector {
    Any,
    Exact(AspectKey),
}

impl AspectKeySelector {
    pub fn exact(aspect_key: AspectKey) -> Self {
        Self::Exact(aspect_key)
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub(crate) fn matches(&self, aspect_key: &AspectKey) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => expected == aspect_key,
        }
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::Exact(left), Self::Exact(right)) => left == right,
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TruthPatchTargetSelector {
    Any,
    AuthoritativeAspect,
    EntityField(CanonicalFieldPath),
    EntityRelationEndpoint,
    EntityRegion,
    EntityPartition,
    EntityFacet,
    LifecycleTransition,
}

impl TruthPatchTargetSelector {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn authoritative_aspect() -> Self {
        Self::AuthoritativeAspect
    }

    pub fn entity_field(field_key: FieldKey) -> Self {
        Self::EntityField(CanonicalFieldPath::single(field_key))
    }

    pub fn entity_field_path(field_path: CanonicalFieldPath) -> Self {
        Self::EntityField(field_path)
    }

    pub fn relation_endpoint() -> Self {
        Self::EntityRelationEndpoint
    }

    pub fn region() -> Self {
        Self::EntityRegion
    }

    pub fn partition() -> Self {
        Self::EntityPartition
    }

    pub fn facet() -> Self {
        Self::EntityFacet
    }

    pub fn lifecycle_transition() -> Self {
        Self::LifecycleTransition
    }

    pub(crate) fn matches<T>(&self, target: &T) -> bool
    where
        T: TruthPatchTargetView,
    {
        match self {
            Self::Any => true,
            Self::AuthoritativeAspect => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::AuthoritativeAspect
            }
            Self::EntityField(expected_path) => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::EntityField
                    && target.truth_field_path() == Some(expected_path)
            }
            Self::EntityRelationEndpoint => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::EntityRelationEndpoint
            }
            Self::EntityRegion => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::EntityRegion
            }
            Self::EntityPartition => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::EntityPartition
            }
            Self::EntityFacet => target.truth_surface_kind() == TruthDeltaSurfaceKind::EntityFacet,
            Self::LifecycleTransition => {
                target.truth_surface_kind() == TruthDeltaSurfaceKind::LifecycleTransition
            }
        }
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::AuthoritativeAspect, Self::AuthoritativeAspect) => true,
            (Self::EntityField(left), Self::EntityField(right)) => left == right,
            (Self::EntityRelationEndpoint, Self::EntityRelationEndpoint)
            | (Self::EntityRegion, Self::EntityRegion)
            | (Self::EntityPartition, Self::EntityPartition)
            | (Self::EntityFacet, Self::EntityFacet) => true,
            (Self::LifecycleTransition, Self::LifecycleTransition) => true,
            _ => false,
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        !matches!(self, Self::Any)
    }

    pub(crate) fn canonical_basis(&self) -> Arc<str> {
        match self {
            Self::Any => Arc::from("target-selector|kind=any"),
            Self::AuthoritativeAspect => Arc::from("target-selector|kind=authoritative-aspect"),
            Self::EntityField(path) => Arc::from(format!(
                "target-selector|kind=entity-field|field-path={}",
                path.fields()
                    .iter()
                    .map(FieldKey::as_str)
                    .collect::<Vec<_>>()
                    .join(".")
            )),
            Self::EntityRelationEndpoint => {
                Arc::from("target-selector|kind=entity-relation-endpoint")
            }
            Self::EntityRegion => Arc::from("target-selector|kind=entity-region"),
            Self::EntityPartition => Arc::from("target-selector|kind=entity-partition"),
            Self::EntityFacet => Arc::from("target-selector|kind=entity-facet"),
            Self::LifecycleTransition => Arc::from("target-selector|kind=lifecycle-transition"),
        }
    }
}

pub(crate) trait TruthPatchTargetView {
    fn truth_surface_kind(&self) -> TruthDeltaSurfaceKind;
    fn truth_field_path(&self) -> Option<&CanonicalFieldPath>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TruthPatchScope {
    entity_selector: MappingSelector,
    aspect_selector: AspectKeySelector,
    target_selector: TruthPatchTargetSelector,
}

impl TruthPatchScope {
    pub fn new(
        entity_selector: MappingSelector,
        aspect_selector: AspectKeySelector,
        target_selector: TruthPatchTargetSelector,
    ) -> Self {
        Self {
            entity_selector,
            aspect_selector,
            target_selector,
        }
    }

    pub fn for_entity_field(
        entity_selector: MappingSelector,
        aspect_key: AspectKey,
        field_key: FieldKey,
    ) -> Self {
        Self::new(
            entity_selector,
            AspectKeySelector::exact(aspect_key),
            TruthPatchTargetSelector::entity_field(field_key),
        )
    }

    pub fn for_target(
        entity_selector: MappingSelector,
        aspect_key: AspectKey,
        target_selector: TruthPatchTargetSelector,
    ) -> Self {
        Self::new(
            entity_selector,
            AspectKeySelector::exact(aspect_key),
            target_selector,
        )
    }

    pub fn entity_selector(&self) -> &MappingSelector {
        &self.entity_selector
    }

    pub fn aspect_selector(&self) -> &AspectKeySelector {
        &self.aspect_selector
    }

    pub fn target_selector(&self) -> &TruthPatchTargetSelector {
        &self.target_selector
    }

    pub(crate) fn specificity_rank(&self) -> u8 {
        u8::from(self.entity_selector.is_exact())
            + u8::from(self.aspect_selector.is_exact())
            + u8::from(self.target_selector.is_exact())
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        self.entity_selector.overlaps(&other.entity_selector)
            && self.aspect_selector.overlaps(&other.aspect_selector)
            && self.target_selector.overlaps(&other.target_selector)
    }

    pub(crate) fn widening_class(&self) -> Option<BridgeMappingWideningClass> {
        let widened_entity = !self.entity_selector.is_exact();
        let widened_aspect = !self.aspect_selector.is_exact();
        let widened_surface = !self.target_selector.is_exact();

        match (widened_entity, widened_aspect, widened_surface) {
            (false, false, false) => None,
            (true, false, false) => Some(BridgeMappingWideningClass::Entity),
            (false, true, false) => Some(BridgeMappingWideningClass::Aspect),
            (false, false, true) => Some(BridgeMappingWideningClass::Surface),
            (true, true, false) => Some(BridgeMappingWideningClass::EntityAspect),
            (true, false, true) => Some(BridgeMappingWideningClass::EntitySurface),
            (false, true, true) => Some(BridgeMappingWideningClass::AspectSurface),
            (true, true, true) => Some(BridgeMappingWideningClass::EntityAspectSurface),
        }
    }
}

pub type SignalInvalidationScope = BridgeIdentity<SignalInvalidationScopeTag>;

impl SignalInvalidationScope {
    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoarseRoutingMode {
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMappingRegistration {
    mapping_id: BridgeMappingId,
    truth_scope: TruthPatchScope,
    snapshot_read_contract: SnapshotReadContract,
    signal_scope: SignalInvalidationScope,
    routing_mode: CoarseRoutingMode,
    version_comparator: Option<worth_signal::facade::VersionComparatorPolicy>,
}

impl BridgeMappingRegistration {
    pub fn new(
        mapping_id: BridgeMappingId,
        truth_scope: TruthPatchScope,
        snapshot_read_contract: SnapshotReadContract,
        signal_scope: SignalInvalidationScope,
        routing_mode: CoarseRoutingMode,
    ) -> Self {
        Self {
            mapping_id,
            truth_scope,
            snapshot_read_contract,
            signal_scope,
            routing_mode,
            version_comparator: None,
        }
    }

    pub fn with_version_comparator(
        mut self,
        policy: worth_signal::facade::VersionComparatorPolicy,
    ) -> Self {
        self.version_comparator = Some(policy);
        self
    }

    pub fn version_comparator(&self) -> Option<&worth_signal::facade::VersionComparatorPolicy> {
        self.version_comparator.as_ref()
    }

    pub fn mapping_id(&self) -> &BridgeMappingId {
        &self.mapping_id
    }

    pub fn truth_scope(&self) -> &TruthPatchScope {
        &self.truth_scope
    }

    pub fn snapshot_read_contract(&self) -> &SnapshotReadContract {
        &self.snapshot_read_contract
    }

    pub fn signal_scope(&self) -> &SignalInvalidationScope {
        &self.signal_scope
    }

    pub fn routing_mode(&self) -> CoarseRoutingMode {
        self.routing_mode
    }

    pub(crate) fn semantic_duplicate_of(&self, other: &Self) -> bool {
        self.truth_scope == other.truth_scope
            && self.snapshot_read_contract == other.snapshot_read_contract
            && self.signal_scope == other.signal_scope
            && self.routing_mode == other.routing_mode
    }
}
