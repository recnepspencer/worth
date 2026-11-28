use crate::error::{
    BridgeErrorContext, BridgePatchTargetCoordinate, BridgeRouteError, BridgeRouteErrorKind,
};
use crate::input::envelope::{BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem};
use crate::mapping::{
    BridgeMappingLookup, BridgeMappingWideningClass, FrozenAspectMappingRegistry,
    FrozenBridgeMappingRegistration, FrozenMappingRegistry,
};
use crate::routing::counters::BridgeRoutingCounters;
use crate::routing::matching::{
    classify_truth_delta_surface, FineGrainedMatchStatus, FineGrainedSurfaceMatch,
};
use crate::routing::surfaces::{
    derive_normalized_truth_delta_surface_set, truth_delta_surface_count, TruthDeltaSurface,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EligibleRouteEntry {
    item: BridgeCommittedPatchItem,
    normalized_surface: TruthDeltaSurface,
    registration: FrozenBridgeMappingRegistration,
    widening_class: Option<BridgeMappingWideningClass>,
    fine_grained_match: FineGrainedSurfaceMatch,
}

impl EligibleRouteEntry {
    pub(crate) fn item(&self) -> &BridgeCommittedPatchItem {
        &self.item
    }

    pub(crate) fn normalized_surface(&self) -> &TruthDeltaSurface {
        &self.normalized_surface
    }

    pub(crate) fn registration(&self) -> &FrozenBridgeMappingRegistration {
        &self.registration
    }

    pub(crate) fn widening_class(&self) -> Option<BridgeMappingWideningClass> {
        self.widening_class
    }

    pub(crate) fn fine_grained_match(&self) -> &FineGrainedSurfaceMatch {
        &self.fine_grained_match
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EligibleRouteRequest {
    envelope: BridgeCommittedPatchEnvelope,
    entries: Vec<EligibleRouteEntry>,
    counters: BridgeRoutingCounters,
}

impl EligibleRouteRequest {
    pub(crate) fn entries(&self) -> &[EligibleRouteEntry] {
        &self.entries
    }

    pub(crate) fn counters(&self) -> BridgeRoutingCounters {
        self.counters
    }
}

pub(crate) fn validate_route_request(
    envelope: BridgeCommittedPatchEnvelope,
    registry: &FrozenMappingRegistry,
    aspect_registry: &FrozenAspectMappingRegistry,
) -> Result<EligibleRouteRequest, BridgeRouteError> {
    let truth_delta_surface_set = derive_normalized_truth_delta_surface_set(&envelope);
    let mut entries = Vec::with_capacity(envelope.patch_body().canonical_items().len());
    let mut counters = BridgeRoutingCounters::from_patch_counts(
        envelope.patch_summary().patch_item_count(),
        envelope.patch_summary().normalized_patch_item_count(),
    )
    .with_truth_delta_surface_counts(
        truth_delta_surface_count(&envelope),
        truth_delta_surface_set.len(),
    );

    for (item, normalized_surface) in envelope
        .patch_body()
        .canonical_items()
        .iter()
        .zip(truth_delta_surface_set.item_surfaces().iter())
    {
        let fine_grained_match = classify_truth_delta_surface(normalized_surface, aspect_registry);
        counters = match fine_grained_match.status() {
            FineGrainedMatchStatus::Matched => counters.with_planned_slice_match(),
            FineGrainedMatchStatus::WideningAdmitted => {
                counters.with_planned_slice_match().with_slice_widening()
            }
            FineGrainedMatchStatus::SuppressedByRegistrationPolicy => {
                counters.with_slice_suppression()
            }
            FineGrainedMatchStatus::UnsupportedSurfaceCategory
            | FineGrainedMatchStatus::AmbiguousRegistration => counters,
        };
        counters = counters.with_mapping_lookup();
        match registry.lookup_truth_surface(normalized_surface) {
            BridgeMappingLookup::Exact { resolved } => {
                for registration in resolved.registrations() {
                    entries.push(EligibleRouteEntry {
                        item: item.clone(),
                        normalized_surface: normalized_surface.clone(),
                        registration: registration.clone(),
                        widening_class: None,
                        fine_grained_match: fine_grained_match.clone(),
                    });
                }
            }
            BridgeMappingLookup::Widening { resolved } => {
                counters = counters.with_mapping_widening();
                let widening_class = resolved
                    .registrations()
                    .next()
                    .and_then(FrozenBridgeMappingRegistration::widening_class);
                for registration in resolved.registrations() {
                    entries.push(EligibleRouteEntry {
                        item: item.clone(),
                        normalized_surface: normalized_surface.clone(),
                        registration: registration.clone(),
                        widening_class,
                        fine_grained_match: fine_grained_match.clone(),
                    });
                }
            }
            BridgeMappingLookup::Missing => {
                return Err(BridgeRouteError::new(
                    BridgeRouteErrorKind::MissingMappingRegistration,
                    format!(
                        "No bridge mapping registration matched committed patch item `{}/{}/{}`.",
                        item.entity_identity().as_ref(),
                        item.aspect_key().as_str(),
                        item.target_canonical_basis()
                    ),
                )
                .with_context(BridgeErrorContext::routing(
                    BridgePatchTargetCoordinate::new(item.entity_identity(), item.target().clone()),
                )));
            }
        }
    }

    debug_assert_eq!(
        envelope.patch_body().canonical_items().len(),
        truth_delta_surface_set.item_surfaces().len(),
        "normalized truth-delta surface derivation must preserve canonical patch-item cardinality"
    );

    Ok(EligibleRouteRequest {
        envelope,
        entries,
        counters,
    })
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
    };

    use super::validate_route_request;
    use crate::error::BridgeRouteErrorKind;
    use crate::input::envelope::{
        BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
        BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeProducerMetadata,
    };
    use crate::mapping::{
        BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeMappingId,
        BridgeMappingRegistration, CoarseRoutingMode, FrozenAspectMappingRegistry,
        FrozenMappingRegistry, MappingSelector, SignalInvalidationScope, SliceWideningPolicy,
        SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
    };

    #[test]
    fn missing_mapping_registration_retains_native_patch_target_coordinate() {
        let envelope = BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
                BridgeProducerMetadata::bridge_harness_fixture(),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                BridgeCommittedPatchTarget::entity_field_path(
                    AspectLocator::new(LocatorAuthority::Authoritative, aspect_key("profile")),
                    CanonicalFieldPath::single(field_key("name")),
                ),
            )],
        )
        .expect("native patch envelope should construct");

        let error = validate_route_request(
            envelope,
            &FrozenMappingRegistry::default(),
            &FrozenAspectMappingRegistry::default(),
        )
        .expect_err("missing route registration must preserve patch target context");

        assert_eq!(
            error.kind(),
            BridgeRouteErrorKind::MissingMappingRegistration
        );
        let coordinate = error
            .context()
            .patch_target_coordinate()
            .expect("missing mapping should retain native patch target coordinate");
        assert_eq!(coordinate.entity_identity(), "entity-1");
        assert_eq!(coordinate.aspect_key().as_str(), "profile");
        assert_eq!(
            coordinate.aspect_locator().authority(),
            LocatorAuthority::Authoritative
        );
        assert_eq!(
            coordinate
                .field_locator()
                .expect("field target coordinate should retain field locator")
                .field_path()
                .fields()[0]
                .as_str(),
            "name"
        );
        assert_eq!(
            coordinate.surface_kind(),
            crate::mapping::TruthDeltaSurfaceKind::EntityField
        );
        assert_eq!(
            coordinate.target().surface_kind(),
            crate::mapping::TruthDeltaSurfaceKind::EntityField
        );
        assert!(!coordinate.target().projection_mask().is_whole_aspect());
        assert!(coordinate
            .target_canonical_basis()
            .contains("kind=entity-field"));
    }

    #[test]
    fn route_eligibility_admits_full_native_whole_target_matrix() {
        let cases = [
            (
                BridgeCommittedPatchTarget::entity_relation_endpoint(aspect_locator("profile")),
                TruthPatchTargetSelector::relation_endpoint(),
                TruthDeltaSurfaceKind::EntityRelationEndpoint,
                SubscriptionSliceKind::SignalLens,
            ),
            (
                BridgeCommittedPatchTarget::entity_region(aspect_locator("profile")),
                TruthPatchTargetSelector::region(),
                TruthDeltaSurfaceKind::EntityRegion,
                SubscriptionSliceKind::SignalRegion,
            ),
            (
                BridgeCommittedPatchTarget::entity_partition(aspect_locator("profile")),
                TruthPatchTargetSelector::partition(),
                TruthDeltaSurfaceKind::EntityPartition,
                SubscriptionSliceKind::SignalPartition,
            ),
            (
                BridgeCommittedPatchTarget::entity_facet(aspect_locator("profile")),
                TruthPatchTargetSelector::facet(),
                TruthDeltaSurfaceKind::EntityFacet,
                SubscriptionSliceKind::SignalFacet,
            ),
        ];

        for (target, target_selector, surface_kind, slice_kind) in cases {
            let envelope = envelope_for_target(target);
            let mapping_registry = mapping_registry(target_selector.clone());
            let aspect_registry = aspect_registry(target_selector, surface_kind, slice_kind);

            let eligible = validate_route_request(envelope, &mapping_registry, &aspect_registry)
                .expect("registered whole-aspect native target should be route eligible");

            assert_eq!(eligible.entries().len(), 1);
            let entry = &eligible.entries()[0];
            assert_eq!(entry.normalized_surface().surface_kind(), surface_kind);
            assert_eq!(entry.normalized_surface().field_locator(), None);
            assert_eq!(entry.item().surface_kind(), surface_kind);
            assert_eq!(entry.item().field_locator(), None);
            assert_eq!(
                entry.fine_grained_match().status(),
                crate::routing::matching::FineGrainedMatchStatus::Matched
            );
            assert!(entry
                .normalized_surface()
                .native_target_basis()
                .contains("projection-mask="));
            assert!(!entry
                .normalized_surface()
                .surface_identity()
                .as_str()
                .contains("field:"));
        }
    }

    fn aspect_key(value: &str) -> AspectKey {
        AspectKey::new(value).expect("valid patch target aspect key")
    }

    fn aspect_locator(value: &str) -> AspectLocator {
        AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(value))
    }

    fn field_key(value: &str) -> FieldKey {
        FieldKey::new(value.to_owned()).expect("valid patch target field key")
    }

    fn envelope_for_target(target: BridgeCommittedPatchTarget) -> BridgeCommittedPatchEnvelope {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
                BridgeProducerMetadata::bridge_harness_fixture(),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            ),
            vec![BridgeCommittedPatchItem::with_target("entity-1", target)],
        )
        .expect("native target envelope should construct")
    }

    fn mapping_registry(target_selector: TruthPatchTargetSelector) -> FrozenMappingRegistry {
        FrozenMappingRegistry::freeze(vec![BridgeMappingRegistration::new(
            BridgeMappingId::admit_bridge_owned("native-target-route"),
            TruthPatchScope::for_target(
                MappingSelector::exact("entity-1"),
                aspect_key("profile"),
                target_selector,
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                aspect_key("profile"),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::admit_bridge_owned("signal.native-target"),
            CoarseRoutingMode::Direct,
        )])
        .expect("mapping registry should freeze")
    }

    fn aspect_registry(
        target_selector: TruthPatchTargetSelector,
        surface_kind: TruthDeltaSurfaceKind,
        slice_kind: SubscriptionSliceKind,
    ) -> FrozenAspectMappingRegistry {
        FrozenAspectMappingRegistry::freeze(vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("native-target-aspect"),
            TruthPatchScope::for_target(
                MappingSelector::exact("entity-1"),
                aspect_key("profile"),
                target_selector,
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                aspect_key("profile"),
                ScalarAspectType::String,
            ),
            surface_kind,
            slice_kind,
            SliceWideningPolicy::Disallow,
        )])
        .expect("aspect registry should freeze")
    }
}
