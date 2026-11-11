use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::projection_consumption::ProjectionConsumptionCertifiedSourceSurface;

use super::super::axes::{
    MilestoneNineFiveBootstrapAxis, MilestoneNineFiveCompositionAxis,
    MilestoneNineFiveProjectionAxis, MilestoneNineFiveReuseAxis, MilestoneNineFiveViewAxis,
};
use super::super::digests::{
    application_support_report, projection_bundle, projection_surface_digest,
};
use super::super::fixtures::canonical::{
    basis_aware_composed_collection, direct_collection, direct_detail, named_scope_collection,
    template_collection, template_detail,
};
use super::super::fixtures::saved_query::{
    exact_saved_query_reuse, freeze_future_preserving_detail_saved_query,
    freeze_future_preserving_grouped_saved_query, freeze_ordinary_detail_saved_query,
    freeze_ordinary_grouped_saved_query,
};
use super::super::fixtures::views::{detail_view, grouped_view, table_view};
use super::super::row::MilestoneNineFiveHostileLaneBundle;
use super::bundle_parts::{bootstrap_axis_for, lane_bundle, projection_axis_for};

const CAPABILITY_IDENTITY: &str = "milestone_nine_five_cross_lane";

pub fn direct_table_lane(
    projection_surface: ProjectionConsumptionCertifiedSourceSurface,
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let canonical = direct_collection();
    let view = table_view(&canonical);
    let saved = freeze_ordinary_grouped_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::Direct,
        MilestoneNineFiveViewAxis::Table,
        projection_axis_for(projection_surface),
        MilestoneNineFiveReuseAxis::Ordinary,
        bootstrap_axis_for(projection_surface),
        canonical.query().digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        digest_parts(&[
            format!("query:{}", canonical.query().digest().as_str()),
            format!("shape:{}", canonical.result_shape().digest().as_str()),
        ]),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(&projection_bundle()?, projection_surface),
        &saved,
        &reuse,
    )
}

pub fn named_scope_table_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let composed = named_scope_collection();
    let view = table_view(composed.canonical());
    let saved = freeze_ordinary_grouped_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::NamedScopeExpansion,
        MilestoneNineFiveViewAxis::Table,
        MilestoneNineFiveProjectionAxis::RetainedDerivedArtifactBinding,
        MilestoneNineFiveReuseAxis::Ordinary,
        MilestoneNineFiveBootstrapAxis::RuntimeBackedDefaultFacade,
        composed.canonical().query().digest().as_str().to_string(),
        composed
            .canonical()
            .result_shape()
            .digest()
            .as_str()
            .to_string(),
        composed
            .composition()
            .composition_digest()
            .as_str()
            .to_string(),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding,
        ),
        &saved,
        &reuse,
    )
}

pub fn direct_detail_live_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let canonical = direct_detail();
    let view = detail_view(&canonical);
    let saved = freeze_ordinary_detail_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::Direct,
        MilestoneNineFiveViewAxis::Detail,
        MilestoneNineFiveProjectionAxis::LiveArtifactBinding,
        MilestoneNineFiveReuseAxis::Ordinary,
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract,
        canonical.query().digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        digest_parts(&[
            format!("query:{}", canonical.query().digest().as_str()),
            format!("shape:{}", canonical.result_shape().digest().as_str()),
        ]),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding,
        ),
        &saved,
        &reuse,
    )
}

pub fn template_detail_live_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let composed = template_detail();
    let view = detail_view(composed.canonical());
    let saved = freeze_ordinary_detail_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::TemplateInstantiation,
        MilestoneNineFiveViewAxis::Detail,
        MilestoneNineFiveProjectionAxis::LiveArtifactBinding,
        MilestoneNineFiveReuseAxis::Ordinary,
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract,
        composed.canonical().query().digest().as_str().to_string(),
        composed
            .canonical()
            .result_shape()
            .digest()
            .as_str()
            .to_string(),
        composed
            .composition()
            .composition_digest()
            .as_str()
            .to_string(),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding,
        ),
        &saved,
        &reuse,
    )
}

pub fn grouped_ordinary_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let canonical = direct_collection();
    let view = grouped_view(&canonical);
    let saved = freeze_ordinary_grouped_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::Direct,
        MilestoneNineFiveViewAxis::KanbanGrouped,
        MilestoneNineFiveProjectionAxis::RelationalGroupedProjection,
        MilestoneNineFiveReuseAxis::Ordinary,
        MilestoneNineFiveBootstrapAxis::RuntimeBackedDefaultFacade,
        canonical.query().digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        digest_parts(&[
            format!("query:{}", canonical.query().digest().as_str()),
            format!("shape:{}", canonical.result_shape().digest().as_str()),
        ]),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection,
        ),
        &saved,
        &reuse,
    )
}

pub fn grouped_preserved_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let composed = basis_aware_composed_collection();
    let view = grouped_view(composed.canonical());
    let saved = freeze_future_preserving_grouped_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::BasisAwareComposition,
        MilestoneNineFiveViewAxis::KanbanGrouped,
        MilestoneNineFiveProjectionAxis::RelationalGroupedProjection,
        MilestoneNineFiveReuseAxis::FuturePreserving,
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract,
        composed.canonical().query().digest().as_str().to_string(),
        composed
            .canonical()
            .result_shape()
            .digest()
            .as_str()
            .to_string(),
        composed
            .composition()
            .composition_digest()
            .as_str()
            .to_string(),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection,
        ),
        &saved,
        &reuse,
    )
}

pub fn template_public_bridge_table_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let composed = template_collection();
    let view = table_view(composed.canonical());
    let saved = freeze_ordinary_grouped_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::TemplateInstantiation,
        MilestoneNineFiveViewAxis::Table,
        MilestoneNineFiveProjectionAxis::LiveArtifactBinding,
        MilestoneNineFiveReuseAxis::Ordinary,
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract,
        composed.canonical().query().digest().as_str().to_string(),
        composed
            .canonical()
            .result_shape()
            .digest()
            .as_str()
            .to_string(),
        composed
            .composition()
            .composition_digest()
            .as_str()
            .to_string(),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding,
        ),
        &saved,
        &reuse,
    )
}

pub fn preserved_detail_lane(
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let saved = freeze_future_preserving_detail_saved_query(
        application_support_report().report_digest(),
        CAPABILITY_IDENTITY,
    );
    let reuse = exact_saved_query_reuse(&saved);
    let canonical = direct_detail();
    let view = detail_view(&canonical);
    lane_bundle(
        MilestoneNineFiveCompositionAxis::BasisAwareComposition,
        MilestoneNineFiveViewAxis::Detail,
        MilestoneNineFiveProjectionAxis::LiveArtifactBinding,
        MilestoneNineFiveReuseAxis::FuturePreserving,
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract,
        saved
            .metadata()
            .canonical_query_digest()
            .as_str()
            .to_string(),
        saved
            .metadata()
            .canonical_result_shape_digest()
            .as_str()
            .to_string(),
        saved.metadata().composition_digest().as_str().to_string(),
        view.view_shape_digest().as_str().to_string(),
        view.view_plan_digest().as_str().to_string(),
        projection_surface_digest(
            &projection_bundle()?,
            ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding,
        ),
        &saved,
        &reuse,
    )
}
