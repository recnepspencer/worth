use worth_ui_inspection::{
    UiInspectionQuery, UiInspectionScope, UiInspectionScopeSupportRow, UiInspectionSupportReason,
    UiInspectionSupportReport, UiInspectionSupportWorld, UiInspectionTarget,
};

use crate::facade::inspection_bridge::boundary_access::{
    aspect_inspection_boundary, authored_inspection_boundary, graph_inspection_boundary,
};
use crate::facade::inspection_bridge::dispatch::{
    classify_inspection_dispatch, InspectionDispatchLane,
};
use crate::facade::WorthUiApp;

pub(crate) fn inspection_support_report_for(
    app: &WorthUiApp,
    query: &UiInspectionQuery,
) -> UiInspectionSupportReport {
    match classify_inspection_dispatch(query) {
        InspectionDispatchLane::ProductRootOrDeclaredSurface => match query.target() {
            UiInspectionTarget::ProductRoot => app.inspection_support_report(query.scope()),
            UiInspectionTarget::DeclaredSurface {
                module_path,
                declaration_index,
            } => declared_surface_inspection_support_report(
                app,
                module_path,
                *declaration_index,
                query.scope(),
            ),
            UiInspectionTarget::GraphNodeIdentity { .. }
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => unsupported_support_report(query.scope()),
        },
        InspectionDispatchLane::AuthoredLookup => authored_inspection_boundary(app)
            .support_report_for(query)
            .unwrap_or_else(|| unsupported_support_report(query.scope())),
        InspectionDispatchLane::GraphNodeIdentity => graph_inspection_boundary(app)
            .support_report_for(query)
            .unwrap_or_else(|| unsupported_support_report(query.scope())),
        InspectionDispatchLane::AspectEvidence => aspect_inspection_boundary(app)
            .support_report_for(query)
            .unwrap_or_else(|| unsupported_support_report(query.scope())),
        InspectionDispatchLane::PlanningScope | InspectionDispatchLane::MeasurementScope => {
            app.inspection_support_report(query.scope())
        }
        InspectionDispatchLane::RetainedObligation | InspectionDispatchLane::UnsupportedTarget => {
            unsupported_support_report(query.scope())
        }
    }
}

fn declared_surface_inspection_support_report(
    app: &WorthUiApp,
    module_path: &str,
    declaration_index: usize,
    scope: UiInspectionScope,
) -> UiInspectionSupportReport {
    app.declaration_artifacts()
        .iter()
        .find(|artifact| {
            let provenance = artifact.provenance().source_provenance();
            provenance.module_path() == module_path
                && provenance.declaration_index() == declaration_index
        })
        .and_then(|artifact| artifact.support_snapshot().ok())
        .map(|snapshot| snapshot.inspection_rows(scope))
        .filter(|rows| !rows.is_empty())
        .map(|rows| UiInspectionSupportReport::from_scope_rows(scope, rows.as_ref()))
        .unwrap_or_else(|| unsupported_support_report(scope))
}

fn unsupported_support_report(scope: UiInspectionScope) -> UiInspectionSupportReport {
    let rows = [UiInspectionScopeSupportRow::unsupported(
        "inspection",
        scope,
        UiInspectionSupportReason::TargetOutsideInspectionBoundary,
        None,
        UiInspectionSupportWorld::Authoritative,
    )];
    UiInspectionSupportReport::from_scope_rows(scope, &rows)
}
