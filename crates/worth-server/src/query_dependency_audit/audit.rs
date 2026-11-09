use worth_proof::TransitionReadiness;
use worth_query::facade::consumer_kit::{
    project_workspace_support_snapshot, support_pinning_contract, WorthQueryPinnedSupportStatus,
    WorthQueryPinnedTeachingPosture, WorthQuerySupportPinContractBuilder,
    WorthQuerySupportPinningError,
};
use worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily;

use crate::{
    config::WorthServerQueryHandoffConfig, query_handoff::WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceBindingError, WorthServerRequestContextFacade,
    WorthServerRequestContextInput,
};

use super::{
    covered_path_inventory, covered_paths, WorthServerQueryDependencyAuditProvenance,
    WorthServerQueryDependencyAuditReceipt, WorthServerQueryDependencyAuditRow,
    WorthServerQueryDependencyAuditRowId, WorthServerQueryDependencyClosurePosture,
    WorthServerQueryDependencyConsumerKitPosture, WorthServerQueryDependencyCoveredPath,
    WorthServerQueryDependencyScopePosture, WorthServerQueryDependencySupportPinProvenance,
};

pub(crate) fn run_query_dependency_audit(
    request_contexts: &WorthServerRequestContextFacade,
    query_handoff_config: &WorthServerQueryHandoffConfig,
) -> WorthServerQueryDependencyAuditReceipt {
    let inventory = covered_path_inventory();
    let mut rows = covered_paths()
        .into_iter()
        .map(|path| classify_path(request_contexts, query_handoff_config, path))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.row_id().cmp(right.row_id()));
    WorthServerQueryDependencyAuditReceipt::new(inventory, rows)
}

fn classify_path(
    request_contexts: &WorthServerRequestContextFacade,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    path: WorthServerQueryDependencyCoveredPath,
) -> WorthServerQueryDependencyAuditRow {
    let (consumer_kit_posture, closure_posture, provenance, reason) =
        classify_query_bound_path(request_contexts, query_handoff_config, &path);
    WorthServerQueryDependencyAuditRow::new(super::WorthServerQueryDependencyAuditRowParts {
        row_id: WorthServerQueryDependencyAuditRowId::new(path.row_id),
        path_kind: path.path_kind,
        runtime_readiness: path.runtime_readiness,
        consumer_kit_posture,
        scope_posture: scope_posture_for(path.path_kind),
        closure_posture,
        ordinary_path: path.ordinary_path,
        provenance,
        reason,
    })
}

fn classify_query_bound_path(
    request_contexts: &WorthServerRequestContextFacade,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    path: &WorthServerQueryDependencyCoveredPath,
) -> (
    WorthServerQueryDependencyConsumerKitPosture,
    WorthServerQueryDependencyClosurePosture,
    WorthServerQueryDependencyAuditProvenance,
    String,
) {
    match evaluate_query_bound_support(request_contexts, query_handoff_config, path) {
        Ok(provenance) if provenance.blocking_finding_count() == 0 => (
            WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningAdopted,
            WorthServerQueryDependencyClosurePosture::Ready,
            WorthServerQueryDependencyAuditProvenance::QuerySupportPin(provenance.clone()),
            format!(
                "workspace `{}` snapshot `{}` satisfied {} required families with report `{}`",
                provenance.workspace_name(),
                provenance.support_snapshot_digest(),
                provenance.required_families().len(),
                provenance.report_digest()
            ),
        ),
        Ok(provenance) => (
            WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningBlocked,
            WorthServerQueryDependencyClosurePosture::Blocked,
            WorthServerQueryDependencyAuditProvenance::QuerySupportPin(provenance.clone()),
            format!(
                "workspace `{}` snapshot `{}` support pin findings: {} blocking across report `{}`",
                provenance.workspace_name(),
                provenance.support_snapshot_digest(),
                provenance.blocking_finding_count(),
                provenance.report_digest()
            ),
        ),
        Err(reason) => (
            WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningBlocked,
            WorthServerQueryDependencyClosurePosture::Blocked,
            WorthServerQueryDependencyAuditProvenance::QuerySupportPin(
                WorthServerQueryDependencySupportPinProvenance::new(
                    super::WorthServerQueryDependencySupportPinProvenanceParts {
                        workspace_name: "unbound-workspace".to_string(),
                        required_families: path.required_query_families.to_vec(),
                        support_matrix_digest: reason.clone(),
                        support_snapshot_digest: reason.clone(),
                        contract_digest: reason.clone(),
                        report_digest: reason.clone(),
                        blocking_finding_count: usize::MAX,
                        matched_required_count: 0,
                    },
                ),
            ),
            reason,
        ),
    }
}

fn evaluate_query_bound_support(
    request_contexts: &WorthServerRequestContextFacade,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    path: &WorthServerQueryDependencyCoveredPath,
) -> Result<WorthServerQueryDependencySupportPinProvenance, String> {
    let request_input = WorthServerRequestContextInput::builder()
        .with_surface_family(path.surface_family())
        .with_transport_class(path.transport_class())
        .with_authenticated_principal_id("audit-principal")
        .with_tenant_id("audit-tenant")
        .with_workspace_id("audit-workspace")
        .build()
        .map_err(|error| format!("request-context-input-error: {error:?}"))?;
    let resolved = match request_contexts.resolve(request_input) {
        TransitionReadiness::Ready(resolved) => resolved,
        other => {
            return Err(format!("request-context-resolution-not-ready: {other:?}"));
        }
    };
    let binding_request =
        WorthServerQueryWorkspaceBindingRequest::for_query_handoff(resolved, path.operation());
    let workspace = query_handoff_config
        .workspace_provider()
        .bind_workspace(&binding_request)
        .map_err(binding_error_reason)?;
    let snapshot = project_workspace_support_snapshot(&workspace)
        .map_err(|cause| format!("support-snapshot-identity-refused: {cause:?}"))?;
    let mut builder = support_pinning_contract(path.row_id)
        .against_snapshot(&snapshot)
        .map_err(|error| format!("support-pinning-contract-error: {error:?}"))?;
    for family in path.required_query_families {
        builder = require_supported_family(builder, *family)
            .map_err(|error| format!("support-pinning-require-family-error: {error:?}"))?;
    }
    let contract = builder
        .seal()
        .map_err(|error| format!("support-pinning-seal-error: {error:?}"))?;
    let report = contract
        .evaluate_snapshot(&snapshot)
        .map_err(|error| format!("support-pinning-evaluation-error: {error:?}"))?;

    Ok(WorthServerQueryDependencySupportPinProvenance::new(
        super::WorthServerQueryDependencySupportPinProvenanceParts {
            workspace_name: workspace.name().to_string(),
            required_families: path.required_query_families.to_vec(),
            support_matrix_digest: snapshot.source_matrix_digest().to_string(),
            support_snapshot_digest: snapshot.snapshot_digest().to_string(),
            contract_digest: report.contract_digest().to_string(),
            report_digest: report.report_digest().to_string(),
            blocking_finding_count: report.blocking_finding_count(),
            matched_required_count: report.matched_required_count(),
        },
    ))
}

fn binding_error_reason(error: WorthServerQueryWorkspaceBindingError) -> String {
    format!(
        "workspace-binding-failed:{}:{}",
        error.stage(),
        error.message()
    )
}

fn scope_posture_for(
    path_kind: super::WorthServerQueryDependencyAuditPathKind,
) -> WorthServerQueryDependencyScopePosture {
    match path_kind {
        super::WorthServerQueryDependencyAuditPathKind::DirectDeclarationSupportPosture => {
            WorthServerQueryDependencyScopePosture::ConsumerKitScoped
        }
        _ => WorthServerQueryDependencyScopePosture::QueryFamilyScoped,
    }
}

fn require_supported_family(
    builder: WorthQuerySupportPinContractBuilder,
    family: WorthQueryRuntimeFacadeFamily,
) -> Result<WorthQuerySupportPinContractBuilder, WorthQuerySupportPinningError> {
    builder.require_family(family, |row| {
        row.status(WorthQueryPinnedSupportStatus::Supported)
            .teaching_posture(WorthQueryPinnedTeachingPosture::OrdinaryRuntimeDx)
            .bind_live_row_digest()
    })
}
