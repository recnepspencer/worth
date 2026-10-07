use super::super::support::*;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::facade::foundation::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    preflight_execution_basis, resolve_historical_materialization_path, resolve_snapshot_basis,
    BasisAuthorityFamily, BasisResolutionMode, ExecutionBasisIntent,
    HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathReuseDescriptor, ResolvedSnapshotIdentity,
    SnapshotLineageClass,
};
use crate::facade::policy::{
    admit_preview_workflow_foundation, bind_preflight_to_preview_session, PreviewEvaluationClass,
    PreviewSessionQueryContext, QueryContextBindingSource, QueryContextFamily,
    ScopedQueryBasisContext,
};
use crate::facade::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_legacy_query_basis_context,
    QueryBasisContextRequest,
};
use crate::harness::fixtures::preview_bridge::active_preview_artifacts;
use crate::runtime::tests::support::{
    test_has_native_field_prefix, test_native_scalar_value, test_native_string_value,
};
use crate::runtime::{
    WorthQueryReadDenialKind, WorthQueryReadExecutionEngine, WorthQueryReadFamily,
    WorthQueryRuntimeError, WorthQueryWorkspace,
};

#[test]
fn execute_read_family_in_basis_context_preserves_current_context_receipt() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-current-basis-context")
        .expect("read-backed runtime should open a workspace");
    let family = identity_read_family(&mut workspace, "current-context-family");
    let context = current_context_for_family(&family, "snapshot-current-context");

    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("basis-context read-family execution should succeed");

    assert_eq!(result.receipt().query_digest(), context.query_digest());
    assert_eq!(result.receipt().basis_digest(), context.basis_digest());
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimeCurrent
    );
    assert_eq!(
        result.receipt().read_graph_digest(),
        family.read_graph().digest()
    );
    assert_runtime_materialized_rows(result.rows());
}

#[test]
fn execute_read_family_in_basis_context_preserves_branch_context_receipt() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-branch-basis-context")
        .expect("read-backed runtime should open a workspace");
    let family = identity_read_family(&mut workspace, "branch-context-family");
    let context = branch_context_for_family(&family, "branch:snapshot-family");

    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("branch basis-context read-family execution should succeed");

    assert_eq!(context.family(), &QueryContextFamily::BranchHead);
    assert_eq!(result.receipt().query_digest(), context.query_digest());
    assert_eq!(result.receipt().basis_digest(), context.basis_digest());
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimeBranch
    );
    assert_context_materialized_rows(result.rows(), &context);
}

#[test]
fn execute_read_family_in_basis_context_preserves_unbound_historical_context_receipt() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-historical-basis-context")
        .expect("read-backed runtime should open a workspace");
    let family = identity_read_family(&mut workspace, "historical-context-family");
    let context = retained_historical_context_for_family(&family, "history:snapshot-family");

    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("historical basis-context read-family execution should succeed");

    assert_eq!(context.family(), &QueryContextFamily::HistoricalSnapshot);
    assert_eq!(result.receipt().query_digest(), context.query_digest());
    assert_eq!(result.receipt().basis_digest(), context.basis_digest());
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimeHistorical
    );
    assert_context_materialized_rows(result.rows(), &context);
}

#[test]
fn execute_read_family_in_basis_context_materializes_runtime_rows_for_bound_historical_snapshot() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-bound-historical-basis-context")
        .expect("read-backed runtime should open a workspace");
    let family = identity_read_family(&mut workspace, "bound-historical-context-family");
    let snapshot_identity = workspace
        .snapshot_identity()
        .expect("the fixture owner remains open");
    let snapshot_evidence_identity = snapshot_identity.evidence_identity();
    let context =
        retained_historical_context_for_family(&family, snapshot_evidence_identity.as_str());

    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("workspace-bound historical basis-context read-family execution should succeed");

    assert_eq!(context.family(), &QueryContextFamily::HistoricalSnapshot);
    assert_eq!(result.receipt().query_digest(), context.query_digest());
    assert_eq!(result.receipt().basis_digest(), context.basis_digest());
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimeHistorical
    );
    assert_runtime_materialized_rows(result.rows());
}

#[test]
fn execute_read_family_in_basis_context_preserves_preview_derived_context_receipt() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-preview-basis-context")
        .expect("read-backed runtime should open a workspace");
    let family = identity_read_family(&mut workspace, "preview-context-family");
    let context = preview_derived_context_for_family(&family, "preview-context-family");

    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("preview-derived basis-context read-family execution should succeed");

    assert_eq!(
        context.family(),
        &QueryContextFamily::PreviewDerivedHistorical
    );
    assert_eq!(result.receipt().query_digest(), context.query_digest());
    assert_eq!(result.receipt().basis_digest(), context.basis_digest());
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimePreviewDerived
    );
    assert_context_materialized_rows(result.rows(), &context);
}

#[test]
fn execute_read_family_in_basis_context_denies_query_digest_substitution() {
    let mut workspace = read_runtime()
        .workspace("runtime.read-composition.family-basis-context-substitution")
        .expect("read-backed runtime should open a workspace");
    let requested_family = identity_read_family(&mut workspace, "requested-family");
    let unrelated_family = profile_read_family(&mut workspace, "unrelated-family");
    let unrelated_context = current_context_for_family(&unrelated_family, "snapshot-unrelated");

    let error = workspace
        .execute_read_family_in_basis_context(&requested_family, &unrelated_context)
        .expect_err("mismatched basis contexts must deny before execution");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                &WorthQueryReadDenialKind::BasisPreflightDenied
            );
            assert!(denial.message().contains("does not match"));
        }
        other => panic!("expected read-composition basis denial, got {other:?}"),
    }
}

fn identity_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_detail(
                "user",
                manager_schema(),
                |query| {
                    query.project(
                        AspectFieldSelector::new("identity", "id")
                            .expect("identity projection should build"),
                    )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect("read family should define")
}

fn profile_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_detail(
                "user",
                manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("profile projection should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new(
                                "profile",
                                "display_name",
                                "display_name",
                            )
                            .expect("profile result-shape field should build"),
                        )
                },
            )
        })
        .expect("profile read family should define")
}

fn current_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let preflight =
        runtime_preflight_for_family(family, snapshot_token, SnapshotLineageClass::CurrentHead);
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::current_branch_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("current basis context should bind");
    admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("current basis context should admit")
}

fn branch_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let preflight =
        runtime_preflight_for_family(family, snapshot_token, SnapshotLineageClass::CurrentHead);
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::branch_head(snapshot_token),
        QueryContextBindingSource::RuntimeBranch(&preflight),
    )
    .expect("branch basis context should bind");
    admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("branch basis context should admit")
}

fn retained_historical_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let query_preflight =
        runtime_preflight_for_family(family, snapshot_token, SnapshotLineageClass::CurrentHead);
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        snapshot_token,
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    );
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        snapshot_token,
        HistoricalPathReuseDescriptor::retained_reuse(),
    );
    let admission =
        admit_historical_evaluation_path(request, capability).expect("history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test(snapshot_token),
    )
    .expect("history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot(snapshot_token),
        QueryContextBindingSource::Historical {
            query_preflight: &query_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect("historical basis context should bind");
    admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("historical basis context should admit")
}

fn preview_derived_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let preflight =
        runtime_preflight_for_family(family, snapshot_token, SnapshotLineageClass::CurrentHead);
    let (_runtime, active, execution_record) = active_preview_artifacts(snapshot_token);
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("preview binding should succeed");
    let foundation =
        admit_preview_workflow_foundation(&binding).expect("preview foundation should admit");
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::preview_derived_historical(
            foundation
                .preview_session_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting()
                .to_string(),
        ),
        QueryContextBindingSource::PreviewDerivedHistorical(&foundation),
    )
    .expect("preview-derived basis context should bind");
    admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("preview-derived basis context should admit")
}

fn runtime_preflight_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
    lineage_class: SnapshotLineageClass,
) -> crate::facade::foundation::ExecutionPreflightBundle {
    let intent =
        ExecutionBasisIntent::new(BasisAuthorityFamily::Runtime, lineage_class.clone(), false);
    let identity = ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        None,
        crate::memory_workspace::admit_external_snapshot_label(snapshot_token).evidence_identity(),
        family.read_graph().schema_basis().clone(),
        lineage_class,
    );
    let basis = resolve_snapshot_basis(intent, identity, BasisResolutionMode::RuntimeDirect)
        .expect("runtime basis should resolve");
    preflight_execution_basis(family.read_graph().execution_plan().clone(), basis)
        .expect("family preflight should build")
}

fn assert_context_materialized_rows(
    rows: &[crate::facade::foundation::WorthQueryEntity],
    context: &ScopedQueryBasisContext,
) {
    assert!(!rows.is_empty());
    assert!(rows
        .iter()
        .all(|row| test_native_scalar_value(row, "query_context.basis_digest").is_some()));
    assert!(rows.iter().all(|row| {
        test_native_string_value(row, "query_context.basis_digest")
            .as_deref()
            .is_some_and(|digest| digest == context.basis_digest())
    }));
    assert!(rows.iter().all(|row| {
        test_native_string_value(row, "query_context.query_digest")
            .as_deref()
            .is_some_and(|digest| digest == context.query_digest())
    }));
    assert!(rows
        .iter()
        .all(|row| row.identity().relational_record_parts().is_none()));
}

fn assert_runtime_materialized_rows(rows: &[crate::facade::foundation::WorthQueryEntity]) {
    assert!(!rows.is_empty());
    assert!(rows
        .iter()
        .all(|row| test_native_scalar_value(row, "query_context.basis_digest").is_none()));
    assert!(rows
        .iter()
        .any(|row| test_has_native_field_prefix(row, "identity")));
    assert!(rows.iter().all(|row| !row
        .identity()
        .evidence_identity()
        .as_str()
        .starts_with("query-context:")));
}
