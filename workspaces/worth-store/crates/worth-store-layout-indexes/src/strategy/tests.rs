#[test]
fn denies_unsupported_or_incomplete_strategy_claims_before_declaration() {
    use super::tests_support::{admit_strategy_scope, root_manifest_scope};
    use crate::strategy::registry::LayoutAdmissionDenial;
    use crate::strategy::registry::{
        layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
    };
    use crate::strategy::LayoutStrategyFamily;
    use crate::{ArtifactFamilyAccessLane, StrategyDenial};

    use worth_store_contracts::DurableArtifactFamilyId;
    use worth_store_security::{
        StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
        StoreKeyScope, StoreTenantScope,
    };

    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let (root_lifecycle, root_domain) = root_manifest_scope();
    assert_eq!(
        layout_admission_registry()
            .admit(LayoutAdmissionRequest::from_admitted(
                page_lifecycle,
                page_domain,
                LayoutStrategyFamily::StreamingCursorIndex,
                LayoutRequestedCapability::point_lookup(),
                ArtifactFamilyAccessLane::HotPath,
            ))
            .unwrap_err(),
        LayoutAdmissionDenial::StrategyVocabularyDenied(StrategyDenial::UnsupportedFamily,)
    );
    assert_eq!(
        layout_admission_registry()
            .admit(LayoutAdmissionRequest::from_admitted(
                root_lifecycle,
                root_domain,
                LayoutStrategyFamily::BaselineBTreeRange,
                LayoutRequestedCapability::point_lookup(),
                ArtifactFamilyAccessLane::MaintenancePath,
            ))
            .unwrap_err(),
        LayoutAdmissionDenial::StrategyVocabularyDenied(
            StrategyDenial::PhysicalKeyDomainDoesNotSupportBaselineBTree,
        )
    );
    assert_eq!(
        layout_admission_registry()
            .admit(LayoutAdmissionRequest::from_admitted(
                page_lifecycle,
                root_domain,
                LayoutStrategyFamily::BaselineBTreeRange,
                LayoutRequestedCapability::point_lookup(),
                ArtifactFamilyAccessLane::HotPath,
            ))
            .unwrap_err(),
        LayoutAdmissionDenial::StrategyVocabularyDenied(
            StrategyDenial::FamilyDoesNotMatchKeyDomain,
        )
    );
    assert_eq!(
        layout_admission_registry()
            .admit(LayoutAdmissionRequest::from_admitted(
                page_lifecycle,
                page_domain,
                LayoutStrategyFamily::ExactScan,
                LayoutRequestedCapability::exact_scan(),
                ArtifactFamilyAccessLane::HotPath,
            ))
            .unwrap_err(),
        LayoutAdmissionDenial::StrategyVocabularyDenied(StrategyDenial::UnsupportedFamily,)
    );
    assert_eq!(
        layout_admission_registry()
            .admit(LayoutAdmissionRequest::from_admitted(
                page_lifecycle,
                page_domain,
                LayoutStrategyFamily::ManifestTable,
                LayoutRequestedCapability::point_lookup(),
                ArtifactFamilyAccessLane::HotPath,
            ))
            .unwrap_err(),
        LayoutAdmissionDenial::StrategyVocabularyDenied(StrategyDenial::UnsupportedFamily,)
    );
}
#[test]
fn admission_binds_counter_profiles_and_posture_to_strategy_families() {
    use super::tests_support::{admit_btree_page_strategy, admit_lsm_wal_strategy};
    use crate::{
        AccessShapeDetail, ArtifactFamilyAccessLane, MaterializationStateClass,
        StrategyAmplificationProfile, StrategyLocalityProfile, StrategyLookupInvariant,
        StrategyPublicationInvariant, StrategyRebuildSourceRequirement,
    };

    let btree = admit_btree_page_strategy();
    let lsm = admit_lsm_wal_strategy();
    let btree_suite = btree.invariant_suite();
    let lsm_suite = lsm.invariant_suite();

    assert_eq!(
        btree_suite.lookup_invariant(),
        StrategyLookupInvariant::SeparatorDirectedLookup
    );
    assert_eq!(
        lsm_suite.publication_invariant(),
        StrategyPublicationInvariant::ManifestPublication
    );
    assert!(btree.supports_point_access());
    assert!(btree.supports_range_access());
    assert!(btree.supports_prefix_access());
    assert!(!btree.supports_streaming_access());
    assert!(lsm.supports_point_access());
    assert!(!lsm.supports_range_access());
    assert!(btree.allows_access_lane(ArtifactFamilyAccessLane::HotPath));
    assert!(btree.allows_access_lane(ArtifactFamilyAccessLane::MaintenancePath));
    assert!(!btree.allows_access_lane(ArtifactFamilyAccessLane::VerifierPath));
    assert_eq!(
        btree.locality_profile(),
        StrategyLocalityProfile::OrderedPageLocality
    );
    assert_eq!(
        lsm.locality_profile(),
        StrategyLocalityProfile::BufferedRunLocality
    );
    assert_eq!(
        btree.amplification_profile(),
        StrategyAmplificationProfile::SplitMergeBounded
    );
    assert_eq!(
        lsm.amplification_profile(),
        StrategyAmplificationProfile::CompactionWriteAmplified
    );
    assert_eq!(
        btree.rebuild_source_requirement(),
        StrategyRebuildSourceRequirement::PhysicalSnapshotReplay
    );
    assert_eq!(
        lsm.rebuild_source_requirement(),
        StrategyRebuildSourceRequirement::WalReplay
    );
    assert!(btree.supports_materialization_state(MaterializationStateClass::Exact));
    assert!(lsm.supports_materialization_state(MaterializationStateClass::Lagged));
    assert!(!lsm.supports_materialization_state(MaterializationStateClass::Exact));
    assert!(btree.canonical_key_encoding().is_some());
    assert!(btree.comparator_law().is_some());
    assert!(btree.prefix_law().is_some());
    assert!(btree.range_bound_law().is_some());
    assert!(lsm.canonical_key_encoding().is_some());
    assert!(lsm.comparator_law().is_some());
    assert!(lsm.prefix_law().is_none());
    assert!(lsm.range_bound_law().is_none());
    assert_eq!(btree.planned_counter_envelope(), None);
    assert_eq!(
        lsm.planned_counter_envelope()
            .expect("lsm strategy should declare planned counters")
            .aggregate_profile(),
        lsm.declared_counter_profile()
    );
    assert_eq!(
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PointLookup)
            .expect("btree point counters should be available")
            .aggregate_profile(),
        crate::strategy::StrategyCounterProfile::new(1, 0, 1, 1, 3)
    );
    assert_eq!(
        btree.declared_counter_profile(),
        btree_suite.counter_evidence().aggregate_profile()
    );
    assert!(btree
        .planned_counter_envelope_for(AccessShapeDetail::RangeLookup(
            crate::access::shape::RangeBasis::CanonicalRangeBounds
        ))
        .is_some());
    assert!(btree
        .planned_counter_envelope_for(AccessShapeDetail::PrefixLookup(
            crate::access::shape::PrefixBasis::CanonicalPrefixBounds
        ))
        .is_some());

    let btree_evidence = btree_suite.counter_evidence();
    let lsm_evidence = lsm_suite.counter_evidence();

    assert_eq!(btree_evidence.lookup(), None);
    assert_eq!(
        btree_evidence
            .point_lookup()
            .expect("btree point declarative envelope should exist"),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PointLookup)
            .expect("btree point counters should be available")
    );
    assert_eq!(
        btree_evidence
            .range_lookup()
            .expect("btree range declarative envelope should exist"),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::RangeLookup(
                crate::access::shape::RangeBasis::CanonicalRangeBounds
            ))
            .expect("btree range counters should be available")
    );
    assert_eq!(
        btree_evidence
            .prefix_lookup()
            .expect("btree prefix declarative envelope should exist"),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PrefixLookup(
                crate::access::shape::PrefixBasis::CanonicalPrefixBounds
            ))
            .expect("btree prefix counters should be available")
    );
    assert_eq!(
        btree_evidence.publication(),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PointLookup)
            .expect("btree point counters should be available")
            .publication()
    );
    assert_eq!(
        btree_evidence.recovery(),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PointLookup)
            .expect("btree point counters should be available")
            .recovery()
    );
    assert_eq!(
        lsm_evidence
            .lookup()
            .expect("lsm lookup envelope should remain singular"),
        lsm.planned_counter_envelope()
            .expect("lsm strategy should declare planned counters")
    );
    assert_eq!(
        lsm_evidence.publication(),
        lsm.planned_counter_envelope()
            .expect("lsm strategy should declare planned counters")
            .publication()
    );
    assert_eq!(
        lsm_evidence.recovery(),
        lsm.planned_counter_envelope()
            .expect("lsm strategy should declare planned counters")
            .recovery()
    );
}

#[test]
fn strategy_identity_preserves_family_and_lane_posture() {
    use super::tests_support::admit_strategy_scope;
    use crate::catalog::ArtifactFamilyAccessLane;
    use crate::strategy::registry::{
        layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
    };
    use crate::strategy::LayoutStrategyFamily;

    use worth_store_contracts::DurableArtifactFamilyId;
    use worth_store_security::{
        StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
        StoreKeyScope, StoreTenantScope,
    };

    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let (segment_lifecycle, segment_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalSegment,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );

    let page_snapshot = layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            page_lifecycle,
            page_domain,
            LayoutStrategyFamily::BaselineBTreeRange,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .unwrap();
    let page = page_snapshot.admitted_strategy();
    let segment_snapshot = layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            segment_lifecycle,
            segment_domain,
            LayoutStrategyFamily::BaselineBTreeRange,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .unwrap();
    let segment = segment_snapshot.admitted_strategy();

    assert_ne!(page.key_domain(), segment.key_domain());
    assert_eq!(page.family(), segment.family());
    assert_eq!(
        page.declared_access_lane(),
        ArtifactFamilyAccessLane::HotPath
    );
}

#[test]
fn btree_strategy_counter_surface_requires_shape_specific_lookup_truth() {
    use super::tests_support::admit_btree_page_strategy;
    use crate::{AccessShapeDetail, PrefixBasis, RangeBasis};

    let btree = admit_btree_page_strategy();
    let evidence = btree.invariant_suite().counter_evidence();

    assert_eq!(btree.planned_counter_envelope(), None);
    assert_eq!(evidence.lookup(), None);
    assert_eq!(
        btree.planned_counter_envelope_for(AccessShapeDetail::PointLookup),
        evidence.point_lookup()
    );
    assert_eq!(
        evidence
            .range_lookup()
            .expect("range declarative counters should exist"),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::RangeLookup(
                RangeBasis::CanonicalRangeBounds
            ))
            .expect("range counters should be available")
    );
    assert_eq!(
        evidence
            .prefix_lookup()
            .expect("prefix declarative counters should exist"),
        btree
            .planned_counter_envelope_for(AccessShapeDetail::PrefixLookup(
                PrefixBasis::CanonicalPrefixBounds
            ))
            .expect("prefix counters should be available")
    );
    assert_eq!(
        evidence.aggregate_profile(),
        crate::strategy::StrategyCounterProfile::new(1, 1, 1, 1, 3)
    );
}
