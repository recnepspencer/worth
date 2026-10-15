use super::bundles::{
    bounded_materialization_region_bundle, bridge_slice_incompatibility_rejection_bundle,
    broad_control_bundle, cdc_stream_contract_bundle, detail_region_convergence_bundle,
    detail_region_widening_bundle, forbidden_broad_success_lane_rejection_bundle,
    forbidden_locality_widening_rejection_bundle,
    forbidden_stream_width_overflow_success_rejection_bundle,
    forbidden_stream_window_overflow_success_rejection_bundle, locality_breadth_budget_bundle,
    locality_work_avoided_bundle, off_region_suppression_bundle,
    ordered_collection_partition_bundle, raw_partition_leakage_rejection_bundle,
    raw_stream_member_forbidden_rejection_bundle, raw_stream_member_leakage_rejection_bundle,
    stream_contract_bundle, stream_member_width_budget_bundle,
    unsupported_locality_family_rejection_bundle, unsupported_locality_predicate_rejection_bundle,
    unsupported_stream_consumer_rejection_bundle,
};
use crate::harness::live_certification::{
    LiveBundleFamily, LiveCertificationBundle, LiveFailureClass, LiveHostileExpectation,
    LiveOutcomeKind, LivePerturbationClass, LiveRejectionBundle,
};
use crate::harness::profiles::CertificationProfile;

#[derive(Clone, Copy)]
pub(super) enum DigestRelation {
    MatchesDeliveryDigest,
    DiffersFromDeliveryDigest,
}

pub(super) struct CanonicalRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: LivePerturbationClass,
    pub hostile_expectation: LiveHostileExpectation,
    pub control_lane: fn(CertificationProfile) -> LiveCertificationBundle,
    pub hostile_lane: fn(CertificationProfile) -> LiveCertificationBundle,
    pub parity_lane: fn(CertificationProfile) -> LiveCertificationBundle,
    pub family: LiveBundleFamily,
    pub outcome_kind: LiveOutcomeKind,
    pub digest_relation: DigestRelation,
}

pub(super) struct RejectionRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: LivePerturbationClass,
    pub control_lane: fn(CertificationProfile) -> LiveCertificationBundle,
    pub hostile_lane: fn(CertificationProfile) -> LiveRejectionBundle,
    pub parity_lane: fn(CertificationProfile) -> LiveCertificationBundle,
    pub failure_class: LiveFailureClass,
    pub control_family: LiveBundleFamily,
    pub failure_digest_fragment: &'static str,
}

macro_rules! canonical_row_specs {
    ($(
        {
            row_name: $row_name:literal,
            perturbation_class: $perturbation_class:expr,
            hostile_expectation: $hostile_expectation:expr,
            control_lane: $control_lane:path,
            hostile_lane: $hostile_lane:path,
            parity_lane: $parity_lane:path,
            family: $family:expr,
            outcome_kind: $outcome_kind:expr,
            digest_relation: $digest_relation:expr
        }
    ),+ $(,)?) => {
        pub(super) const CANONICAL_ROW_SPECS: &[CanonicalRowSpec] = &[
            $(
                CanonicalRowSpec {
                    row_name: $row_name,
                    perturbation_class: $perturbation_class,
                    hostile_expectation: $hostile_expectation,
                    control_lane: $control_lane,
                    hostile_lane: $hostile_lane,
                    parity_lane: $parity_lane,
                    family: $family,
                    outcome_kind: $outcome_kind,
                    digest_relation: $digest_relation,
                },
            )+
        ];
    };
}

macro_rules! rejection_row_specs {
    ($(
        {
            row_name: $row_name:literal,
            perturbation_class: $perturbation_class:expr,
            control_lane: $control_lane:path,
            hostile_lane: $hostile_lane:path,
            parity_lane: $parity_lane:path,
            failure_class: $failure_class:expr,
            control_family: $control_family:expr,
            failure_digest_fragment: $failure_digest_fragment:literal
        }
    ),+ $(,)?) => {
        pub(super) const REJECTION_ROW_SPECS: &[RejectionRowSpec] = &[
            $(
                RejectionRowSpec {
                    row_name: $row_name,
                    perturbation_class: $perturbation_class,
                    control_lane: $control_lane,
                    hostile_lane: $hostile_lane,
                    parity_lane: $parity_lane,
                    failure_class: $failure_class,
                    control_family: $control_family,
                    failure_digest_fragment: $failure_digest_fragment,
                },
            )+
        ];
    };
}

canonical_row_specs![
    {
        row_name: "region-live-convergence",
        perturbation_class: LivePerturbationClass::RegionScopedConvergenceParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: detail_region_convergence_bundle,
        hostile_lane: detail_region_convergence_bundle,
        parity_lane: detail_region_convergence_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "off-region-suppression-parity",
        perturbation_class: LivePerturbationClass::OffRegionSuppressionParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: off_region_suppression_bundle,
        hostile_lane: off_region_suppression_bundle,
        parity_lane: off_region_suppression_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Suppressed,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "collection-partition-hit",
        perturbation_class: LivePerturbationClass::CollectionPartitionParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: ordered_collection_partition_bundle,
        hostile_lane: ordered_collection_partition_bundle,
        parity_lane: ordered_collection_partition_bundle,
        family: LiveBundleFamily::OrderedCollection,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "bounded-materialization-region-hit",
        perturbation_class: LivePerturbationClass::BoundedMaterializationRegionParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: bounded_materialization_region_bundle,
        hostile_lane: bounded_materialization_region_bundle,
        parity_lane: bounded_materialization_region_bundle,
        family: LiveBundleFamily::BoundedMaterialization,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "detail-region-single-peer-widening",
        perturbation_class: LivePerturbationClass::LocalityWideningAdmissionParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: detail_region_widening_bundle,
        hostile_lane: detail_region_widening_bundle,
        parity_lane: detail_region_widening_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "broad-vs-region-narrowing-control",
        perturbation_class: LivePerturbationClass::BroadVsRegionParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: broad_control_bundle,
        hostile_lane: detail_region_convergence_bundle,
        parity_lane: detail_region_convergence_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "stream-contract-delivery-parity",
        perturbation_class: LivePerturbationClass::StreamContractParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: stream_contract_bundle,
        hostile_lane: stream_contract_bundle,
        parity_lane: stream_contract_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::StreamLoweredDelivery,
        digest_relation: DigestRelation::DiffersFromDeliveryDigest
    },
    {
        row_name: "cdc-stream-lowered-parity",
        perturbation_class: LivePerturbationClass::CdcStreamLoweredParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: cdc_stream_contract_bundle,
        hostile_lane: cdc_stream_contract_bundle,
        parity_lane: cdc_stream_contract_bundle,
        family: LiveBundleFamily::OrderedCollection,
        outcome_kind: LiveOutcomeKind::StreamLoweredDelivery,
        digest_relation: DigestRelation::DiffersFromDeliveryDigest
    },
    {
        row_name: "locality-breadth-budget-enforcement",
        perturbation_class: LivePerturbationClass::LocalityBreadthBudgetEnforcement,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: locality_breadth_budget_bundle,
        hostile_lane: locality_breadth_budget_bundle,
        parity_lane: locality_breadth_budget_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Patch,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    },
    {
        row_name: "stream-member-width-budget-enforcement",
        perturbation_class: LivePerturbationClass::StreamMemberWidthBudgetEnforcement,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: stream_member_width_budget_bundle,
        hostile_lane: stream_member_width_budget_bundle,
        parity_lane: stream_member_width_budget_bundle,
        family: LiveBundleFamily::OrderedCollection,
        outcome_kind: LiveOutcomeKind::StreamLoweredDelivery,
        digest_relation: DigestRelation::DiffersFromDeliveryDigest
    },
    {
        row_name: "locality-work-avoided-counter-parity",
        perturbation_class: LivePerturbationClass::LocalityWorkAvoidedParity,
        hostile_expectation: LiveHostileExpectation::EquivalentToControl,
        control_lane: locality_work_avoided_bundle,
        hostile_lane: locality_work_avoided_bundle,
        parity_lane: locality_work_avoided_bundle,
        family: LiveBundleFamily::Detail,
        outcome_kind: LiveOutcomeKind::Suppressed,
        digest_relation: DigestRelation::MatchesDeliveryDigest
    }
];

rejection_row_specs![
    {
        row_name: "unsupported-locality-family",
        perturbation_class: LivePerturbationClass::UnsupportedLocalityFamilyRejection,
        control_lane: broad_control_bundle,
        hostile_lane: unsupported_locality_family_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::UnsupportedLocalityFamily,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "UnsupportedLiveCollectionFamily"
    },
    {
        row_name: "unsupported-locality-predicate",
        perturbation_class: LivePerturbationClass::UnsupportedLocalityPredicateRejection,
        control_lane: broad_control_bundle,
        hostile_lane: unsupported_locality_predicate_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::UnsupportedLocalityPredicate,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "UnsupportedLocalityPredicate"
    },
    {
        row_name: "unsupported-stream-consumer-contract",
        perturbation_class: LivePerturbationClass::UnsupportedStreamConsumerRejection,
        control_lane: stream_contract_bundle,
        hostile_lane: unsupported_stream_consumer_rejection_bundle,
        parity_lane: stream_contract_bundle,
        failure_class: LiveFailureClass::UnsupportedStreamConsumerContract,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "UnsupportedStreamConsumerShape"
    },
    {
        row_name: "raw-partition-event-leakage-forbidden",
        perturbation_class: LivePerturbationClass::RawPartitionLeakageRejection,
        control_lane: broad_control_bundle,
        hostile_lane: raw_partition_leakage_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::RawPartitionEventLeakageForbidden,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "WideningDenied"
    },
    {
        row_name: "collection-cross-partition-denied",
        perturbation_class: LivePerturbationClass::ForbiddenLocalityWideningRejection,
        control_lane: stream_member_width_budget_bundle,
        hostile_lane: forbidden_locality_widening_rejection_bundle,
        parity_lane: stream_member_width_budget_bundle,
        failure_class: LiveFailureClass::ForbiddenLocalityWidening,
        control_family: LiveBundleFamily::OrderedCollection,
        failure_digest_fragment: "WideningDenied"
    },
    {
        row_name: "raw-stream-member-leakage-forbidden",
        perturbation_class: LivePerturbationClass::RawStreamMemberLeakageRejection,
        control_lane: stream_contract_bundle,
        hostile_lane: raw_stream_member_leakage_rejection_bundle,
        parity_lane: stream_contract_bundle,
        failure_class: LiveFailureClass::RawStreamMemberLeakageForbidden,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "UnsupportedStreamConsumerShape"
    },
    {
        row_name: "raw-stream-member-forbidden",
        perturbation_class: LivePerturbationClass::RawStreamMemberForbiddenRejection,
        control_lane: stream_member_width_budget_bundle,
        hostile_lane: raw_stream_member_forbidden_rejection_bundle,
        parity_lane: stream_member_width_budget_bundle,
        failure_class: LiveFailureClass::RawStreamMemberForbidden,
        control_family: LiveBundleFamily::OrderedCollection,
        failure_digest_fragment: "UnsupportedStreamConsumerShape"
    },
    {
        row_name: "forbidden-locality-widening",
        perturbation_class: LivePerturbationClass::ForbiddenLocalityWideningRejection,
        control_lane: broad_control_bundle,
        hostile_lane: forbidden_locality_widening_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::ForbiddenLocalityWidening,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "WideningDenied"
    },
    {
        row_name: "forbidden-broad-success-lane",
        perturbation_class: LivePerturbationClass::ForbiddenBroadSuccessLaneRejection,
        control_lane: broad_control_bundle,
        hostile_lane: forbidden_broad_success_lane_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::ForbiddenBroadSuccessLane,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "LocalityBreadthBudgetExceeded"
    },
    {
        row_name: "forbidden-stream-width-overflow-success",
        perturbation_class: LivePerturbationClass::ForbiddenStreamWidthOverflowSuccessRejection,
        control_lane: stream_member_width_budget_bundle,
        hostile_lane: forbidden_stream_width_overflow_success_rejection_bundle,
        parity_lane: stream_member_width_budget_bundle,
        failure_class: LiveFailureClass::ForbiddenStreamWindowOverflowSuccess,
        control_family: LiveBundleFamily::OrderedCollection,
        failure_digest_fragment: "StreamMemberWidthBudgetExceeded"
    },
    {
        row_name: "forbidden-stream-window-overflow-success",
        perturbation_class: LivePerturbationClass::ForbiddenStreamWindowOverflowSuccessRejection,
        control_lane: detail_region_widening_bundle,
        hostile_lane: forbidden_stream_window_overflow_success_rejection_bundle,
        parity_lane: detail_region_widening_bundle,
        failure_class: LiveFailureClass::ForbiddenStreamWindowOverflowSuccess,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "StreamWindowWidthBudgetExceeded"
    },
    {
        row_name: "bridge-slice-incompatibility-denied",
        perturbation_class: LivePerturbationClass::BridgeSliceIncompatibilityRejection,
        control_lane: broad_control_bundle,
        hostile_lane: bridge_slice_incompatibility_rejection_bundle,
        parity_lane: broad_control_bundle,
        failure_class: LiveFailureClass::BridgeSliceIncompatibilityDenied,
        control_family: LiveBundleFamily::Detail,
        failure_digest_fragment: "BridgeSliceIncompatibility"
    }
];
