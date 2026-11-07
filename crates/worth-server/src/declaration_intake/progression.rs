use worth_query::facade::consumer_kit::{
    project_workspace_support_snapshot, support_pinning_contract, WorthQueryPinnedSupportStatus,
    WorthQueryPinnedTeachingPosture,
};
use worth_query::facade::runtime::{WorthQueryRuntimeFacadeFamily, WorthQueryWorkspace};

use crate::{
    config::WorthServerQueryHandoffConfig, WorthServerAdmission,
    WorthServerQueryWorkspaceBindingRequest,
};

use super::{
    WorthServerDirectDeclaration, WorthServerDirectDeclarationDenial,
    WorthServerDirectSupportSnapshot, WorthServerPreparedDirectDeclaration,
};

pub(crate) fn prepare_direct_declaration(
    config: &WorthServerQueryHandoffConfig,
    admission: WorthServerAdmission,
    declaration: WorthServerDirectDeclaration,
) -> Result<WorthServerPreparedDirectDeclaration, WorthServerDirectDeclarationDenial> {
    validate_declaration_identity(&admission, &declaration)?;
    let workspace = bind_workspace(config, &admission, &declaration)?;
    let support_snapshot = build_support_snapshot(
        admission.request_context().diagnostics_profile(),
        &workspace,
        declaration.clone(),
    )?;
    let declaration_digest = declaration_digest(&workspace, &declaration);

    Ok(WorthServerPreparedDirectDeclaration::new(
        admission,
        declaration,
        workspace,
        declaration_digest,
        support_snapshot,
    ))
}

fn validate_declaration_identity(
    admission: &WorthServerAdmission,
    declaration: &WorthServerDirectDeclaration,
) -> Result<(), WorthServerDirectDeclarationDenial> {
    if declaration.source().has_blank_binding_label() {
        return Err(
            WorthServerDirectDeclarationDenial::invalid_declaration_identity(
                admission.request_context().diagnostics_profile(),
                format!(
                    "{} declaration identity cannot be blank",
                    declaration.source().kind().as_str()
                ),
            ),
        );
    }

    Ok(())
}

fn bind_workspace(
    config: &WorthServerQueryHandoffConfig,
    admission: &WorthServerAdmission,
    declaration: &WorthServerDirectDeclaration,
) -> Result<WorthQueryWorkspace, WorthServerDirectDeclarationDenial> {
    let binding_request = WorthServerQueryWorkspaceBindingRequest::for_direct_declaration(
        admission.resolved_request_context().clone(),
        declaration.source().kind(),
        declaration.source().binding_label(),
    );
    config
        .workspace_provider()
        .bind_workspace(&binding_request)
        .map_err(|error| {
            WorthServerDirectDeclarationDenial::workspace_binding_failed(
                admission.request_context().diagnostics_profile(),
                format!("{}: {}", error.stage(), error.message()),
            )
        })
}

fn build_support_snapshot(
    diagnostics_profile: worth_foundational::DiagnosticRichnessProfile,
    workspace: &WorthQueryWorkspace,
    declaration: WorthServerDirectDeclaration,
) -> Result<WorthServerDirectSupportSnapshot, WorthServerDirectDeclarationDenial> {
    let support_snapshot = project_workspace_support_snapshot(workspace).map_err(|cause| {
        WorthServerDirectDeclarationDenial::EvidenceIdentityDerivationDenied {
            diagnostics_profile,
            cause,
            detail: "support snapshot identity exceeded canonical digest bounds".into(),
        }
    })?;
    let read_family_row = support_snapshot
        .rows()
        .iter()
        .find(|row| row.facade_family() == Some(WorthQueryRuntimeFacadeFamily::Read.as_str()))
        .cloned()
        .ok_or_else(|| {
            WorthServerDirectDeclarationDenial::query_facade_family_not_admitted(
                diagnostics_profile,
                "support snapshot did not expose the read facade family row",
                WorthServerDirectSupportSnapshot::new(
                    declaration.clone(),
                    support_snapshot.rows().first().cloned().expect(
                        "support snapshot should expose at least one row when building direct support posture",
                    ),
                    None,
                    support_snapshot.source_matrix_digest().to_string(),
                    support_snapshot.snapshot_digest().to_string(),
                    "direct-declaration-read-row-missing".to_string(),
                    false,
                ),
            )
        })?;
    let read_family_contract = workspace
        .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Read)
        .ok();
    let pin_report = support_pinning_contract("worth-server.direct-declaration-intake")
        .against_snapshot(&support_snapshot)
        .and_then(|builder| {
            builder.require_family(WorthQueryRuntimeFacadeFamily::Read, |row| {
                row.status(WorthQueryPinnedSupportStatus::Supported)
                    .teaching_posture(WorthQueryPinnedTeachingPosture::OrdinaryRuntimeDx)
                    .bind_live_row_digest()
            })
        })
        .and_then(|builder| builder.seal())
        .and_then(|contract| contract.evaluate_snapshot(&support_snapshot))
        .map_err(|error| {
            WorthServerDirectDeclarationDenial::query_facade_family_not_admitted(
                diagnostics_profile,
                format!("direct declaration support pinning failed: {error}"),
                WorthServerDirectSupportSnapshot::new(
                    declaration.clone(),
                    read_family_row.clone(),
                    read_family_contract.clone(),
                    support_snapshot.source_matrix_digest().to_string(),
                    support_snapshot.snapshot_digest().to_string(),
                    "direct-declaration-support-pin-evaluation-failed".to_string(),
                    false,
                ),
            )
        })?;

    Ok(WorthServerDirectSupportSnapshot::new(
        declaration,
        read_family_row,
        read_family_contract,
        support_snapshot.source_matrix_digest().to_string(),
        support_snapshot.snapshot_digest().to_string(),
        pin_report.report_digest().to_string(),
        pin_report.satisfied(),
    ))
}

fn declaration_digest(
    workspace: &WorthQueryWorkspace,
    declaration: &WorthServerDirectDeclaration,
) -> String {
    format!(
        "worth-server-direct-declaration-v1|workspace:{}|source:{}|view-shape:{}",
        workspace.name(),
        declaration.source().canonical_label(),
        declaration.view_shape().as_str(),
    )
}
