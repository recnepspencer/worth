use crate::mapping::{
    BridgeAspectRegistrationId, FrozenAspectMappingRegistry, SliceWideningPolicy,
    SubscriptionSliceKind, TruthDeltaSurfaceKind,
};
use crate::snapshot::SnapshotReadContract;

use super::surfaces::TruthDeltaSurface;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FineGrainedMatchStatus {
    Matched,
    WideningAdmitted,
    SuppressedByRegistrationPolicy,
    UnsupportedSurfaceCategory,
    AmbiguousRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FineGrainedMatchOutcome {
    Matched {
        truth_surface_kind: TruthDeltaSurfaceKind,
        aspect_registration_id: BridgeAspectRegistrationId,
        snapshot_read_contract: SnapshotReadContract,
        subscription_slice_kind: SubscriptionSliceKind,
    },
    WideningAdmitted {
        truth_surface_kind: TruthDeltaSurfaceKind,
        aspect_registration_id: BridgeAspectRegistrationId,
        snapshot_read_contract: SnapshotReadContract,
        subscription_slice_kind: SubscriptionSliceKind,
        widening_policy: SliceWideningPolicy,
    },
    SuppressedByRegistrationPolicy {
        truth_surface_kind: TruthDeltaSurfaceKind,
    },
}

impl FineGrainedMatchOutcome {
    pub fn status(&self) -> FineGrainedMatchStatus {
        match self {
            Self::Matched { .. } => FineGrainedMatchStatus::Matched,
            Self::WideningAdmitted { .. } => FineGrainedMatchStatus::WideningAdmitted,
            Self::SuppressedByRegistrationPolicy { .. } => {
                FineGrainedMatchStatus::SuppressedByRegistrationPolicy
            }
        }
    }

    pub fn subscription_slice_kind(&self) -> Option<&SubscriptionSliceKind> {
        match self {
            Self::Matched {
                subscription_slice_kind,
                ..
            }
            | Self::WideningAdmitted {
                subscription_slice_kind,
                ..
            } => Some(subscription_slice_kind),
            Self::SuppressedByRegistrationPolicy { .. } => None,
        }
    }

    pub fn snapshot_read_contract(&self) -> Option<&SnapshotReadContract> {
        match self {
            Self::Matched {
                snapshot_read_contract,
                ..
            }
            | Self::WideningAdmitted {
                snapshot_read_contract,
                ..
            } => Some(snapshot_read_contract),
            Self::SuppressedByRegistrationPolicy { .. } => None,
        }
    }
}

pub(crate) type FineGrainedSurfaceMatch = FineGrainedMatchOutcome;

pub(crate) fn classify_truth_delta_surface(
    surface: &TruthDeltaSurface,
    aspect_registry: &FrozenAspectMappingRegistry,
) -> FineGrainedSurfaceMatch {
    let Some(registration) = aspect_registry.lookup(
        surface.entity_identity().as_ref(),
        surface.aspect_key(),
        surface.field_locator().map(|locator| locator.field_path()),
        surface.surface_kind(),
    ) else {
        return FineGrainedSurfaceMatch::SuppressedByRegistrationPolicy {
            truth_surface_kind: surface.surface_kind(),
        };
    };

    match registration.widening_policy() {
        SliceWideningPolicy::Disallow => FineGrainedSurfaceMatch::Matched {
            truth_surface_kind: surface.surface_kind(),
            aspect_registration_id: registration.registration_id().clone(),
            snapshot_read_contract: registration.snapshot_read_contract().clone(),
            subscription_slice_kind: registration.subscription_slice_kind().clone(),
        },
        widening_policy => FineGrainedSurfaceMatch::WideningAdmitted {
            truth_surface_kind: surface.surface_kind(),
            aspect_registration_id: registration.registration_id().clone(),
            snapshot_read_contract: registration.snapshot_read_contract().clone(),
            subscription_slice_kind: registration.subscription_slice_kind().clone(),
            widening_policy,
        },
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
    };

    use crate::input::envelope::{
        BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
        BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeProducerMetadata,
    };
    use crate::mapping::{
        BridgeAspectRegistration, BridgeAspectRegistrationId, FrozenAspectMappingRegistry,
        MappingSelector, SliceWideningPolicy, SubscriptionSliceKind, TruthDeltaSurfaceKind,
        TruthPatchScope,
    };

    use super::{classify_truth_delta_surface, FineGrainedMatchStatus};
    use crate::routing::surfaces::{derive_normalized_truth_delta_surface_set, TruthDeltaSurface};

    fn registry(registrations: Vec<BridgeAspectRegistration>) -> FrozenAspectMappingRegistry {
        FrozenAspectMappingRegistry::freeze(registrations).expect("aspect registry should freeze")
    }

    #[test]
    fn classify_surface_as_suppressed_when_no_registration_matches() {
        let surface = field_surface("user", "profile", "name");

        let classification =
            classify_truth_delta_surface(&surface, &FrozenAspectMappingRegistry::default());

        assert_eq!(
            classification.status(),
            FineGrainedMatchStatus::SuppressedByRegistrationPolicy
        );
        assert!(classification.subscription_slice_kind().is_none());
    }

    #[test]
    fn classify_surface_as_matched_when_direct_registration_exists() {
        let surface = field_surface("user", "profile", "name");
        let registry = registry(vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("field"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                aspect_key("profile"),
                ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        )]);

        let classification = classify_truth_delta_surface(&surface, &registry);

        assert_eq!(classification.status(), FineGrainedMatchStatus::Matched);
        assert_eq!(
            classification.subscription_slice_kind(),
            Some(&SubscriptionSliceKind::SignalField)
        );
    }

    #[test]
    fn classify_surface_as_widening_when_registration_admits_widening() {
        let surface = field_surface("user", "profile", "name");
        let registry = registry(vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("field-widening"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                aspect_key("profile"),
                ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::RegisteredCoarseWidening,
            SliceWideningPolicy::RegisteredEntityCoarseWidening,
        )]);

        let classification = classify_truth_delta_surface(&surface, &registry);

        assert_eq!(
            classification.status(),
            FineGrainedMatchStatus::WideningAdmitted
        );
        assert_eq!(
            classification.subscription_slice_kind(),
            Some(&SubscriptionSliceKind::RegisteredCoarseWidening)
        );
    }

    fn aspect_key(value: &str) -> AspectKey {
        AspectKey::new(value).expect("valid truth delta surface aspect key")
    }

    fn field_surface(entity_identity: &str, aspect: &str, field: &str) -> TruthDeltaSurface {
        let envelope = BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
                BridgeProducerMetadata::bridge_harness_fixture(),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:routing-match"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch:routing-match"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:routing-match"),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                entity_identity,
                BridgeCommittedPatchTarget::entity_field_path(
                    AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(aspect)),
                    field_path(field),
                ),
            )],
        )
        .expect("matching test envelope should validate");
        derive_normalized_truth_delta_surface_set(&envelope).item_surfaces()[0].clone()
    }

    fn field_path(value: &str) -> CanonicalFieldPath {
        CanonicalFieldPath::single(
            FieldKey::new(value.to_owned()).expect("valid truth delta surface field key"),
        )
    }
}
