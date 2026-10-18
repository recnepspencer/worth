use super::evidence::{
    certification_bundle, compile_fail_rejection, denied_bridge_parity_failure,
    denied_cross_family_scope_failure, denied_runtime_certification_failure,
    denied_support_failure,
};
use super::lanes::{lane_for, CertifiedLaneArtifacts, LaneScenario};
use crate::harness::certification::{HostileExpectation, ParityAnchor, RejectionCertificationRow};
use crate::live::LiveQueryFamily;
use crate::subscription::CoverageResolutionPosture;
use crate::view_shape_live::LiveViewShapeFamily;

use super::{
    MilestoneNineThreeCertificationBundle, MilestoneNineThreeCertificationRow,
    MilestoneNineThreePerturbationClass, MilestoneNineThreeRejectionBundle,
};

pub(super) fn canonical_rows() -> Vec<MilestoneNineThreeCertificationRow> {
    let detail = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        1,
    );
    let inspector = lane_for(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::InspectorDetailFocused),
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        2,
    );
    let ordered = lane_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        3,
    );
    let grouped = lane_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        4,
    );
    let bounded = lane_for(
        LiveQueryFamily::BoundedMaterialization,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        5,
    );
    let continuation = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::Continuation,
        CoverageResolutionPosture::IndexedCoverageSet,
        6,
    );
    let preview = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::PreviewDiscard,
        CoverageResolutionPosture::IndexedCoverageSet,
        7,
    );
    let churn_control = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        8,
    );
    let churn_hostile = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        9,
    );
    let debt = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::MatrixScanDebtExplicit,
        1,
    );

    vec![
        admitted_row(
            "detail-family-support-and-parity",
            MilestoneNineThreePerturbationClass::DetailFamilySupportAndParity,
            HostileExpectation::EquivalentToControl,
            &detail,
            &detail,
            &detail,
        ),
        admitted_row(
            "inspector-family-support-and-parity",
            MilestoneNineThreePerturbationClass::InspectorFamilySupportAndParity,
            HostileExpectation::DistinctFromControl,
            &detail,
            &inspector,
            &inspector,
        ),
        admitted_row(
            "ordered-collection-family-support-and-parity",
            MilestoneNineThreePerturbationClass::OrderedCollectionFamilySupportAndParity,
            HostileExpectation::EquivalentToControl,
            &ordered,
            &ordered,
            &ordered,
        ),
        admitted_row(
            "grouped-collection-family-support-and-parity",
            MilestoneNineThreePerturbationClass::GroupedCollectionFamilySupportAndParity,
            HostileExpectation::DistinctFromControl,
            &ordered,
            &grouped,
            &grouped,
        ),
        admitted_row(
            "bounded-materialization-family-support-and-parity",
            MilestoneNineThreePerturbationClass::BoundedMaterializationFamilySupportAndParity,
            HostileExpectation::EquivalentToControl,
            &bounded,
            &bounded,
            &bounded,
        ),
        admitted_row(
            "preview-family-lifecycle-certification-bundle",
            MilestoneNineThreePerturbationClass::PreviewFamilyLifecycleCertificationBundle,
            HostileExpectation::DistinctFromControl,
            &detail,
            &preview,
            &preview,
        ),
        admitted_row(
            "continuation-family-support-sync",
            MilestoneNineThreePerturbationClass::ContinuationFamilySupportSync,
            HostileExpectation::DistinctFromControl,
            &detail,
            &continuation,
            &continuation,
        ),
        admitted_row(
            "family-coverage-certification-closure",
            MilestoneNineThreePerturbationClass::FamilyCoverageCertificationClosure,
            HostileExpectation::EquivalentToControl,
            &detail,
            &detail,
            &detail,
        ),
        admitted_row(
            "declaration-family-drift-vs-lifecycle-churn-distinctness",
            MilestoneNineThreePerturbationClass::DeclarationFamilyDriftVsLifecycleChurnDistinctness,
            HostileExpectation::DistinctFromControl,
            &churn_control,
            &churn_hostile,
            &churn_control,
        ),
        admitted_row(
            "basis-policy-viewshape-family-coverage-closure",
            MilestoneNineThreePerturbationClass::BasisPolicyViewshapeFamilyCoverageClosure,
            HostileExpectation::DistinctFromControl,
            &detail,
            &grouped,
            &grouped,
        ),
        admitted_row(
            "support-matrix-scale-honesty",
            MilestoneNineThreePerturbationClass::SupportMatrixScaleHonesty,
            HostileExpectation::DistinctFromControl,
            &detail,
            &debt,
            &detail,
        ),
    ]
}

pub(super) fn rejection_rows() -> Vec<
    RejectionCertificationRow<
        MilestoneNineThreePerturbationClass,
        MilestoneNineThreeCertificationBundle,
        MilestoneNineThreeRejectionBundle,
    >,
> {
    let detail = lane_for(
        LiveQueryFamily::Detail,
        None,
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        11,
    );
    let collection = lane_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        LaneScenario::ActiveLifecycle,
        CoverageResolutionPosture::IndexedCoverageSet,
        12,
    );

    vec![
        rejection_row(
            "uncertified-family-support-overclaim-forbidden",
            MilestoneNineThreePerturbationClass::UncertifiedFamilySupportOverclaimForbidden,
            &detail,
            denied_support_failure(&detail),
            &detail,
        ),
        rejection_row(
            "store-backed-restart-support-overclaim-forbidden",
            MilestoneNineThreePerturbationClass::StoreBackedRestartSupportOverclaimForbidden,
            &detail,
            compile_fail_rejection("subscription_support_report_durable_overclaim_forbidden.rs"),
            &detail,
        ),
        rejection_row(
            "durable-replay-support-overclaim-forbidden",
            MilestoneNineThreePerturbationClass::DurableReplaySupportOverclaimForbidden,
            &detail,
            compile_fail_rejection("subscription_support_report_durable_overclaim_forbidden.rs"),
            &detail,
        ),
        rejection_row(
            "bridge-parity-declaration-source-mismatch",
            MilestoneNineThreePerturbationClass::BridgeParityDeclarationSourceMismatch,
            &detail,
            denied_bridge_parity_failure(&detail, &collection),
            &detail,
        ),
        rejection_row(
            "bridge-parity-signal-strategy-source-mismatch",
            MilestoneNineThreePerturbationClass::BridgeParitySignalStrategySourceMismatch,
            &detail,
            compile_fail_rejection(
                "subscription_bridge_parity_mismatched_signal_strategy_forbidden.rs",
            ),
            &detail,
        ),
        rejection_row(
            "diagnostic-bundle-missing-hostile-row-forbidden",
            MilestoneNineThreePerturbationClass::DiagnosticBundleMissingHostileRowForbidden,
            &detail,
            denied_runtime_certification_failure(&detail),
            &detail,
        ),
        rejection_row(
            "runtime-certification-cross-family-row-mix-forbidden",
            MilestoneNineThreePerturbationClass::RuntimeCertificationCrossFamilyRowMixForbidden,
            &detail,
            denied_cross_family_scope_failure(&detail, &collection),
            &detail,
        ),
        rejection_row(
            "generic-family-certification-shortcut-forbidden",
            MilestoneNineThreePerturbationClass::GenericFamilyCertificationShortcutForbidden,
            &detail,
            compile_fail_rejection(
                "subscription_runtime_certification_uncertified_family_forbidden.rs",
            ),
            &detail,
        ),
    ]
}

fn admitted_row(
    row_name: &'static str,
    perturbation_class: MilestoneNineThreePerturbationClass,
    hostile_expectation: HostileExpectation,
    control: &CertifiedLaneArtifacts,
    hostile: &CertifiedLaneArtifacts,
    parity: &CertifiedLaneArtifacts,
) -> MilestoneNineThreeCertificationRow {
    MilestoneNineThreeCertificationRow {
        row_name,
        perturbation_class,
        hostile_expectation,
        parity_anchor: if hostile
            .runtime_bundle
            .runtime_certification_bundle_projection()
            .label()
            == parity
                .runtime_bundle
                .runtime_certification_bundle_projection()
                .label()
        {
            ParityAnchor::Hostile
        } else {
            ParityAnchor::Control
        },
        control_lane: certification_bundle(control),
        hostile_lane: certification_bundle(hostile),
        parity_lane: certification_bundle(parity),
    }
}

fn rejection_row(
    row_name: &'static str,
    perturbation_class: MilestoneNineThreePerturbationClass,
    control: &CertifiedLaneArtifacts,
    hostile: MilestoneNineThreeRejectionBundle,
    parity: &CertifiedLaneArtifacts,
) -> RejectionCertificationRow<
    MilestoneNineThreePerturbationClass,
    MilestoneNineThreeCertificationBundle,
    MilestoneNineThreeRejectionBundle,
> {
    RejectionCertificationRow {
        row_name,
        perturbation_class,
        control_lane: certification_bundle(control),
        hostile_lane: hostile,
        parity_lane: certification_bundle(parity),
    }
}
