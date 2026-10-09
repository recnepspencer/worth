use super::{
    layout_customization_boundary, FutureLayoutCapabilityRequest, FutureLayoutCustomizationDenial,
    FutureLayoutCustomizationRequest, FutureLayoutWorkloadEnvelope,
};
use crate::strategy::registry::{LayoutAdmissionDenial, LayoutRequestedCapability};
use crate::strategy::tests_support::{admit_strategy_scope, root_manifest_scope};
use crate::strategy::LayoutStrategyFamily;
use worth_proof::TransitionOutcome;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

#[test]
fn customization_admits_supported_foreground_requests_as_registry_snapshots() {
    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let request = FutureLayoutCustomizationRequest::new(
        page_lifecycle,
        FutureLayoutCapabilityRequest::ordered_range(page_domain),
        FutureLayoutWorkloadEnvelope::foreground_bounded_traversal(),
    );

    match layout_customization_boundary().admit(request) {
        TransitionOutcome::Success(admitted) => {
            let snapshot = admitted.registry_snapshot();
            assert_eq!(
                snapshot.admitted_strategy().family(),
                LayoutStrategyFamily::BTreeRange
            );
            assert_eq!(
                snapshot.request().family(),
                LayoutStrategyFamily::BTreeRange
            );
        }
        other => panic!("supported customization request should admit: {other:?}"),
    }
}

#[test]
fn customization_denies_envelope_masquerade_and_unready_projection_requests() {
    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let masquerade = FutureLayoutCustomizationRequest::new(
        page_lifecycle,
        FutureLayoutCapabilityRequest::ordered_range(page_domain),
        FutureLayoutWorkloadEnvelope::verifier_corpus_inspection(),
    );
    let rebuildable_projection = FutureLayoutCustomizationRequest::new(
        page_lifecycle,
        FutureLayoutCapabilityRequest::rebuildable_projection(page_domain),
        FutureLayoutWorkloadEnvelope::background_rebuild_projection(),
    );

    assert_eq!(
        layout_customization_boundary().admit(masquerade),
        TransitionOutcome::Denied(
            FutureLayoutCustomizationDenial::WorkloadEnvelopeDoesNotSupportCapability {
                capability: FutureLayoutCapabilityRequest::ordered_range(page_domain),
                envelope: FutureLayoutWorkloadEnvelope::VerifierCorpusInspection,
            }
        )
    );
    assert_eq!(
        layout_customization_boundary().admit(rebuildable_projection),
        TransitionOutcome::Denied(
            FutureLayoutCustomizationDenial::RebuildableProjectionNotYetSupported {
                key_domain: page_domain.witness(),
            }
        )
    );
}

#[test]
fn customization_propagates_store_missing_layout_facts_honestly() {
    let (wal_lifecycle, wal_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PublicationWalIntent,
        StoreKeyScope::WalCheckpointEnvelope,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedWalRecord,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let unsupported_range = FutureLayoutCustomizationRequest::new(
        wal_lifecycle,
        FutureLayoutCapabilityRequest::ordered_range(wal_domain),
        FutureLayoutWorkloadEnvelope::foreground_bounded_traversal(),
    );
    let verifier_scan = FutureLayoutCustomizationRequest::new(
        wal_lifecycle,
        FutureLayoutCapabilityRequest::verifier_declared_scan(wal_domain),
        FutureLayoutWorkloadEnvelope::verifier_corpus_inspection(),
    );

    assert_eq!(
        layout_customization_boundary().admit(unsupported_range),
        TransitionOutcome::Denied(FutureLayoutCustomizationDenial::StoreAdmissionDenied(
            super::LayoutAdmissionDenialProjection::new(
                LayoutAdmissionDenial::StrategyDoesNotSupportRequestedCapability {
                    family: LayoutStrategyFamily::BaselineLsmWriteOptimized,
                    capability: LayoutRequestedCapability::OrderedRange,
                },
            )
        ))
    );
    assert_eq!(
        layout_customization_boundary().admit(verifier_scan),
        TransitionOutcome::Denied(FutureLayoutCustomizationDenial::StoreAdmissionDenied(
            super::LayoutAdmissionDenialProjection::new(
                LayoutAdmissionDenial::RequestedLaneDoesNotMatchFamilyLane {
                    family: LayoutStrategyFamily::BaselineLsmWriteOptimized,
                    requested_lane: crate::catalog::ArtifactFamilyAccessLane::VerifierPath,
                    declared_lane: crate::catalog::ArtifactFamilyAccessLane::HotPath,
                },
            )
        ))
    );
}

#[test]
fn customization_denies_key_domains_without_supported_store_strategy() {
    let (root_manifest_lifecycle, root_manifest_domain) = root_manifest_scope();
    let unsupported = FutureLayoutCustomizationRequest::new(
        root_manifest_lifecycle,
        FutureLayoutCapabilityRequest::point_lookup(root_manifest_domain),
        FutureLayoutWorkloadEnvelope::foreground_low_fanout(),
    );

    assert_eq!(
        layout_customization_boundary().admit(unsupported),
        TransitionOutcome::Denied(
            FutureLayoutCustomizationDenial::NoStrategySupportsRequestedCapability {
                capability: FutureLayoutCapabilityRequest::point_lookup(root_manifest_domain),
                key_domain: root_manifest_domain.witness(),
            }
        )
    );
}
