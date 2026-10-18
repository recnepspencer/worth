use crate::harness::certification::{
    digest_parts, HostileExpectation, ParityAnchor, RejectionCertificationRow,
};
use crate::saved_query::{SavedQueryFailureClass, SavedQueryReuseDenial, SavedQueryReuseOutcome};

use super::axes::{MilestoneNineFiveFailureClass, MilestoneNineFivePerturbationClass};
use super::digests::application_support_report;
use super::fixtures::saved_query::{
    erased_basis_family_reuse, focused_inspector_target_reuse,
    freeze_future_preserving_detail_saved_query, freeze_future_preserving_grouped_saved_query,
};
use super::lanes::builders::{
    direct_detail_live_lane, direct_table_lane, grouped_ordinary_lane, grouped_preserved_lane,
    named_scope_table_lane, preserved_detail_lane, template_detail_live_lane,
    template_public_bridge_table_lane,
};
use super::row::{
    MilestoneNineFiveHostileLaneBundle, MilestoneNineFiveHostileRejectionBundle,
    MilestoneNineFiveHostileRow,
};
use crate::projection_consumption::ProjectionConsumptionCertifiedSourceSurface;

const CAPABILITY_IDENTITY: &str = "milestone_nine_five_cross_lane";

pub fn canonical_rows() -> Vec<MilestoneNineFiveHostileRow> {
    let direct_table_retained = direct_table_lane(
        ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding,
    );
    let direct_table_live =
        direct_table_lane(ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding);
    let named_scope_table = named_scope_table_lane();
    let direct_detail_live = direct_detail_live_lane();
    let template_detail_live = template_detail_live_lane();
    let grouped_ordinary = grouped_ordinary_lane();
    let grouped_preserved = grouped_preserved_lane();
    let template_public_bridge_table = template_public_bridge_table_lane();

    vec![
        admitted_row(
            "named-scope-table-retained-derived-parity",
            MilestoneNineFivePerturbationClass::NamedScopeTableRetainedDerivedParity,
            HostileExpectation::EquivalentToControl,
            &direct_table_retained,
            &named_scope_table,
            &direct_table_retained,
        ),
        admitted_row(
            "template-detail-live-artifact-parity",
            MilestoneNineFivePerturbationClass::TemplateDetailLiveArtifactParity,
            HostileExpectation::EquivalentToControl,
            &direct_detail_live,
            &template_detail_live,
            &direct_detail_live,
        ),
        admitted_row(
            "retained-vs-live-projection-contract-distinctness",
            MilestoneNineFivePerturbationClass::RetainedVsLiveProjectionContractDistinctness,
            HostileExpectation::DistinctFromControl,
            &direct_table_retained,
            &direct_table_live,
            &direct_table_live,
        ),
        admitted_row(
            "grouped-view-family-preserved-reuse-distinctness",
            MilestoneNineFivePerturbationClass::GroupedViewFamilyPreservedReuseDistinctness,
            HostileExpectation::DistinctFromControl,
            &direct_table_retained,
            &grouped_preserved,
            &grouped_preserved,
        ),
        admitted_row(
            "grouped-ordinary-vs-preserved-reuse-distinctness",
            MilestoneNineFivePerturbationClass::GroupedOrdinaryVsPreservedReuseDistinctness,
            HostileExpectation::DistinctFromControl,
            &grouped_ordinary,
            &grouped_preserved,
            &grouped_preserved,
        ),
        admitted_row(
            "public-bridge-bootstrap-fixed-under-template-composition",
            MilestoneNineFivePerturbationClass::PublicBridgeBootstrapFixedUnderTemplateComposition,
            HostileExpectation::EquivalentToControl,
            &direct_table_live,
            &template_public_bridge_table,
            &direct_table_live,
        ),
    ]
}

pub fn rejection_rows() -> Vec<
    RejectionCertificationRow<
        MilestoneNineFivePerturbationClass,
        MilestoneNineFiveHostileLaneBundle,
        MilestoneNineFiveHostileRejectionBundle,
    >,
> {
    let grouped_preserved = grouped_preserved_lane();
    let preserved_detail = preserved_detail_lane();
    let support_digest = application_support_report().report_digest().to_string();

    vec![
        rejection_row(
            "grouped-preserved-reuse-basis-erasure-denied",
            MilestoneNineFivePerturbationClass::GroupedPreservedReuseBasisErasureDenied,
            &grouped_preserved,
            denial_bundle(erased_basis_family_reuse(
                &freeze_future_preserving_grouped_saved_query(&support_digest, CAPABILITY_IDENTITY),
            )),
            &grouped_preserved,
        ),
        rejection_row(
            "inspector-target-preserved-reuse-downcast-denied",
            MilestoneNineFivePerturbationClass::InspectorTargetPreservedReuseDowncastDenied,
            &preserved_detail,
            denial_bundle(focused_inspector_target_reuse(
                &freeze_future_preserving_detail_saved_query(&support_digest, CAPABILITY_IDENTITY),
            )),
            &preserved_detail,
        ),
    ]
}

fn admitted_row(
    row_name: &'static str,
    perturbation_class: MilestoneNineFivePerturbationClass,
    hostile_expectation: HostileExpectation,
    control_lane: &MilestoneNineFiveHostileLaneBundle,
    hostile_lane: &MilestoneNineFiveHostileLaneBundle,
    parity_lane: &MilestoneNineFiveHostileLaneBundle,
) -> MilestoneNineFiveHostileRow {
    MilestoneNineFiveHostileRow {
        row_name,
        perturbation_class,
        hostile_expectation,
        parity_anchor: if parity_lane.semantic_signature() == hostile_lane.semantic_signature() {
            ParityAnchor::Hostile
        } else {
            ParityAnchor::Control
        },
        control_lane: control_lane.clone(),
        hostile_lane: hostile_lane.clone(),
        parity_lane: parity_lane.clone(),
    }
}

fn rejection_row(
    row_name: &'static str,
    perturbation_class: MilestoneNineFivePerturbationClass,
    control_lane: &MilestoneNineFiveHostileLaneBundle,
    hostile_lane: MilestoneNineFiveHostileRejectionBundle,
    parity_lane: &MilestoneNineFiveHostileLaneBundle,
) -> RejectionCertificationRow<
    MilestoneNineFivePerturbationClass,
    MilestoneNineFiveHostileLaneBundle,
    MilestoneNineFiveHostileRejectionBundle,
> {
    RejectionCertificationRow {
        row_name,
        perturbation_class,
        control_lane: control_lane.clone(),
        hostile_lane,
        parity_lane: parity_lane.clone(),
    }
}

fn denial_bundle(outcome: SavedQueryReuseOutcome) -> MilestoneNineFiveHostileRejectionBundle {
    let SavedQueryReuseOutcome::Denied(denial) = outcome else {
        panic!("rejection row requires a denied reuse outcome");
    };
    build_denial_bundle(&denial)
}

fn build_denial_bundle(denial: &SavedQueryReuseDenial) -> MilestoneNineFiveHostileRejectionBundle {
    assert_eq!(
        denial.failure_class(),
        &SavedQueryFailureClass::IllegalSemanticDrift
    );
    let illegal_row_count = denial
        .matrix()
        .rows()
        .iter()
        .filter(|row| {
            row.legality() == crate::saved_query::SavedQueryRebindingLegality::IllegalSemanticDrift
        })
        .count();
    MilestoneNineFiveHostileRejectionBundle {
        failure_class: MilestoneNineFiveFailureClass::PreservedReuseDriftDenied,
        failure_kind: "illegal_semantic_drift".to_string(),
        failure_digest: digest_parts(&[
            "failure:illegal_semantic_drift".to_string(),
            format!("matrix:{}", denial.matrix().digest()),
            format!(
                "temporal_async:{}",
                denial.temporal_async_surface_posture().as_str()
            ),
            format!("message:{}", denial.message()),
        ]),
        reuse_matrix_digest: denial.matrix().digest().to_string(),
        temporal_async_surface_posture: denial
            .temporal_async_surface_posture()
            .as_str()
            .to_string(),
        counter_snapshot: format!("illegal_rows={illegal_row_count};residue=0"),
    }
}
