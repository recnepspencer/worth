use crate::basis::{BasisAuthorityFamily, ExecutionPreflightBundle};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::WorkflowCounters;
use worth_relational::facade::history::BranchId;

use super::context_binding::{WorkflowBasisFamily, WorkflowContextBinding};
use super::context_identity::{
    apply_binding_scope_field, binding_scope_for_context_binding,
    workflow_canonical_query_digest_evidence, workflow_context_basis_identity,
    workflow_context_binding_identity, workflow_context_query_identity,
    workflow_context_source_identity, workflow_plan_digest_evidence, workflow_scope_from_label,
    WorkflowBindingScopeField,
};

fn synthetic_runtime_workflow_query_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    runtime_snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::WorkflowContextBinding,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "synthetic_runtime_workflow_query_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label),
        binding_scope,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("runtime_snapshot"),
        runtime_snapshot_identity,
    )
    .seal()
}

fn synthetic_runtime_workflow_source_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    runtime_snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::WorkflowContextBinding,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "synthetic_runtime_workflow_source_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label),
        binding_scope,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("runtime_snapshot"),
        runtime_snapshot_identity,
    )
    .seal()
}

fn synthetic_runtime_workflow_basis_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    runtime_snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::WorkflowContextBinding,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "synthetic_runtime_workflow_basis_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("basis_family"),
            WorkflowBasisFamily::RuntimePreflight.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label),
        binding_scope,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("runtime_snapshot"),
        runtime_snapshot_identity,
    )
    .seal()
}

pub(crate) fn synthetic_runtime_workflow_binding_for_snapshot_identity(
    source_label: &str,
    runtime_snapshot_identity: WorthQuerySnapshotIdentity,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    synthetic_runtime_workflow_binding_scoped_for_snapshot_identity(
        source_label,
        "unscoped",
        runtime_snapshot_identity,
    )
}

pub(crate) fn synthetic_runtime_workflow_binding_scoped_for_snapshot_identity(
    source_label: &str,
    binding_scope_label: &str,
    runtime_snapshot_identity: WorthQuerySnapshotIdentity,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    let binding_scope = workflow_scope_from_label(binding_scope_label);
    synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity(
        source_label,
        &binding_scope,
        runtime_snapshot_identity,
    )
}

pub(crate) fn synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    runtime_snapshot_identity: WorthQuerySnapshotIdentity,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity(
        source_label,
        binding_scope,
        runtime_snapshot_identity,
        BranchId("main".to_string()),
    )
}

pub(crate) fn synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_identity(
    source_label: &str,
    binding_scope_label: &str,
    runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    runtime_target_branch: BranchId,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    let binding_scope = workflow_scope_from_label(binding_scope_label);
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity(
        source_label,
        &binding_scope,
        runtime_snapshot_identity,
        runtime_target_branch,
    )
}

pub(crate) fn synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    runtime_target_branch: BranchId,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    let runtime_snapshot_evidence = runtime_snapshot_identity.evidence_identity();
    let source_identity = synthetic_runtime_workflow_source_identity(
        source_label,
        binding_scope,
        &runtime_snapshot_evidence,
    )?;
    let query_identity = synthetic_runtime_workflow_query_identity(
        source_label,
        binding_scope,
        &runtime_snapshot_evidence,
    )?;
    let basis_identity = synthetic_runtime_workflow_basis_identity(
        source_label,
        binding_scope,
        &runtime_snapshot_evidence,
    )?;
    let binding_identity = workflow_context_binding_identity(
        &source_identity,
        &query_identity,
        WorkflowBasisFamily::RuntimePreflight,
        &basis_identity,
        Some(&runtime_snapshot_identity),
        binding_scope_for_context_binding(binding_scope),
    )?;
    Ok(WorkflowContextBinding {
        binding_identity,
        source_identity,
        query_identity,
        basis_family: WorkflowBasisFamily::RuntimePreflight,
        basis_identity,
        runtime_snapshot_identity: Some(runtime_snapshot_identity),
        runtime_target_branch: Some(runtime_target_branch),
        preview_evaluation_class: None,
        preview_request_family: None,
        preview_session_identity: None,
        counters: WorkflowCounters {
            workflow_basis_binding_count: 1,
            workflow_basis_binding_width: 1,
            workflow_executor_rediscovery_count: 0,
            ..WorkflowCounters::default()
        },
    })
}

pub(crate) fn scoped_runtime_preflight_workflow_binding_for_binding_identity(
    preflight: &ExecutionPreflightBundle,
    binding_scope_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorkflowContextBinding, super::admission_reporting::WorkflowAdmissionError> {
    let mut binding = bind_runtime_preflight(preflight)?;
    let binding_scope = WorkflowBindingScopeField::Identity(binding_scope_identity);
    let error_counters = binding.counters.clone();
    let result: Result<WorkflowContextBinding, IdentityDerivationDenial> = (|| {
        binding.binding_identity = workflow_context_binding_identity(
            &binding.source_identity,
            &binding.query_identity,
            binding.basis_family.clone(),
            &binding.basis_identity,
            binding.runtime_snapshot_identity.as_ref(),
            Some(&binding_scope),
        )?;
        Ok(binding)
    })();
    result.map_err(|cause| {
        super::admission_reporting::WorkflowAdmissionError::identity_refused(cause, error_counters)
    })
}

pub(super) fn bind_runtime_preflight(
    preflight: &ExecutionPreflightBundle,
) -> Result<WorkflowContextBinding, super::admission_reporting::WorkflowAdmissionError> {
    if preflight.basis().identity().authority_family() != &BasisAuthorityFamily::Runtime {
        return Err(super::admission_reporting::WorkflowAdmissionError::new(
            super::admission_reporting::WorkflowAdmissionFailureClass::InvalidBasisPairing,
            "workflow binding requires a runtime-backed execution preflight basis",
            super::admission_reporting::WorkflowPredictionDriftOutcome::WithinBudget,
            WorkflowCounters {
                workflow_basis_binding_count: 1,
                workflow_basis_binding_width: 1,
                workflow_denial_count: 1,
                workflow_executor_rediscovery_count: 0,
                ..WorkflowCounters::default()
            },
        ));
    }

    let plan_query = preflight.plan().query();
    let source_identity =
        workflow_context_source_identity(&workflow_plan_digest_evidence(plan_query.plan_digest()));
    let query_identity = workflow_context_query_identity(
        &workflow_canonical_query_digest_evidence(plan_query.canonical_query_digest()),
    );
    let basis_identity = workflow_context_basis_identity(
        &WorkflowBasisFamily::RuntimePreflight,
        preflight.basis().proof().identity(),
    );
    let runtime_snapshot_identity = WorthQuerySnapshotIdentity::preview(
        preflight.basis().identity().snapshot_identity().clone(),
    );
    let error_counters = WorkflowCounters {
        workflow_basis_binding_count: 1,
        workflow_basis_binding_width: 1,
        workflow_executor_rediscovery_count: 0,
        ..WorkflowCounters::default()
    };
    let result: Result<WorkflowContextBinding, IdentityDerivationDenial> = (|| {
        let binding_identity = workflow_context_binding_identity(
            &source_identity,
            &query_identity,
            WorkflowBasisFamily::RuntimePreflight,
            &basis_identity,
            Some(&runtime_snapshot_identity),
            None,
        )?;

        Ok(WorkflowContextBinding {
            binding_identity,
            source_identity,
            query_identity,
            basis_family: WorkflowBasisFamily::RuntimePreflight,
            basis_identity,
            runtime_snapshot_identity: Some(runtime_snapshot_identity),
            runtime_target_branch: Some(BranchId("main".to_string())),
            preview_evaluation_class: None,
            preview_request_family: None,
            preview_session_identity: None,
            counters: error_counters.clone(),
        })
    })();
    result.map_err(|cause| {
        super::admission_reporting::WorkflowAdmissionError::identity_refused(cause, error_counters)
    })
}
