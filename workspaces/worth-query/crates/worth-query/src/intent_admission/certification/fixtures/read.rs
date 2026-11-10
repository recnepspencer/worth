use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::facade::foundation::{
    basis_lifecycle, preflight_execution_basis, resolve_snapshot_basis, BasisAuthorityFamily,
    BasisResolutionMode, ExecutionBasisIntent, ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::facade::policy::{
    admit_query_basis_context, QueryContextBindingSource, ScopedQueryBasisContext,
};
use crate::facade::runtime::{
    WorthQueryIntentAdmissionDecision, WorthQueryReadFamily, WorthQueryReadResult,
    WorthQueryWorkspace,
};
use crate::intent_admission::{WorthQueryAdmittedIntentPlan, WorthQueryRawIntentAdmissionRequest};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView};

use super::{certification_snapshot_identity, runtime::certification_runtime};

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedReadIntentFixture {
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) plan:
        crate::intent_admission::WorthQueryReadExecutionPlan,
    pub(in crate::intent_admission::certification) handoff:
        crate::intent_admission::WorthQueryReadExecutionHandoff,
    pub(in crate::intent_admission::certification) binding:
        crate::intent_admission::WorthQueryReadExecutionBinding,
    pub(in crate::intent_admission::certification) trace:
        crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope,
    pub(in crate::intent_admission::certification) result: WorthQueryReadResult,
}

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct ReadDelegationParityFixture {
    pub(in crate::intent_admission::certification) current_legacy: WorthQueryReadResult,
    pub(in crate::intent_admission::certification) current_canonical: WorthQueryReadResult,
    pub(in crate::intent_admission::certification) basis_legacy: WorthQueryReadResult,
    pub(in crate::intent_admission::certification) basis_canonical: WorthQueryReadResult,
}

pub(in crate::intent_admission::certification) fn certified_read_intent_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CertifiedReadIntentFixture {
    let mut workspace = certification_read_workspace(execution, "certification-read-intent");
    let family = certification_read_family(&mut workspace, "certification-read-family");
    let review = workspace
        .review_read_execution(family.clone(), None)
        .expect("certification read review should succeed");
    let request = review.request().clone();
    let plan = match review.decision().clone() {
        WorthQueryIntentAdmissionDecision::Admitted(
            WorthQueryAdmittedIntentPlan::ReadExecution(plan),
        ) => plan,
        other => panic!("expected admitted read decision, got {other:?}"),
    };
    let handoff = workspace
        .resolve_reviewed_admitted_read_execution_handoff(review.clone())
        .expect("read handoff should admit");
    let binding = workspace
        .into_runtime_read_execution_binding(handoff.clone())
        .expect("read binding should prepare");
    let result = workspace
        .execute_bound_read_execution(execution, binding.clone())
        .expect("read binding should execute");
    let trace = result
        .receipt()
        .decision_trace_envelope()
        .expect("read result should retain decision trace")
        .clone();
    CertifiedReadIntentFixture {
        request,
        plan,
        handoff,
        binding,
        trace,
        result,
    }
}

pub(in crate::intent_admission::certification) fn read_delegation_parity_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ReadDelegationParityFixture {
    let mut delegated_current = certification_read_workspace(execution, "delegated-read-current");
    let current_family = certification_read_family(&mut delegated_current, "tasks");
    let current_legacy = delegated_current
        .execute_read_family(execution, &current_family)
        .expect("delegated current read should execute");

    let mut canonical_current = certification_read_workspace(execution, "canonical-read-current");
    let canonical_current_family = certification_read_family(&mut canonical_current, "tasks");
    let current_review = canonical_current
        .review_read_execution(canonical_current_family, None)
        .expect("canonical current review should succeed");
    let current_handoff = canonical_current
        .resolve_reviewed_admitted_read_execution_handoff(current_review)
        .expect("canonical current handoff should admit");
    let current_binding = canonical_current
        .into_runtime_read_execution_binding(current_handoff)
        .expect("canonical current binding should prepare");
    let current_canonical = canonical_current
        .execute_bound_read_execution(execution, current_binding)
        .expect("canonical current read should execute");

    let mut delegated_basis = certification_read_workspace(execution, "delegated-read-basis");
    let basis_family = certification_read_family(&mut delegated_basis, "tasks");
    let basis_context = current_context_for_family(
        &basis_family,
        certification_snapshot_identity("certification-read-basis"),
    );
    let basis_legacy = delegated_basis
        .execute_read_family_in_basis_context(execution, &basis_family, &basis_context)
        .expect("delegated basis read should execute");

    let mut canonical_basis = certification_read_workspace(execution, "canonical-read-basis");
    let canonical_basis_family = certification_read_family(&mut canonical_basis, "tasks");
    let canonical_context = current_context_for_family(
        &canonical_basis_family,
        certification_snapshot_identity("certification-read-basis"),
    );
    let basis_review = canonical_basis
        .review_read_execution(canonical_basis_family, Some(canonical_context))
        .expect("canonical basis review should succeed");
    let basis_handoff = canonical_basis
        .resolve_reviewed_admitted_read_execution_handoff(basis_review)
        .expect("canonical basis handoff should admit");
    let basis_binding = canonical_basis
        .into_runtime_read_execution_binding(basis_handoff)
        .expect("canonical basis binding should prepare");
    let basis_canonical = canonical_basis
        .execute_bound_read_execution(execution, basis_binding)
        .expect("canonical basis read should execute");

    ReadDelegationParityFixture {
        current_legacy,
        current_canonical,
        basis_legacy,
        basis_canonical,
    }
}

fn certification_read_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> WorthQueryWorkspace {
    certification_read_runtime(request_execution)
        .workspace(name)
        .expect("certification read workspace should build")
}

fn certification_read_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::runtime::WorthQueryRuntime {
    certification_runtime(request_execution)
}

fn certification_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_detail(
                "user",
                certification_manager_schema(),
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
        .expect("certification read family should define")
}

fn current_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_identity: WorthQuerySnapshotIdentity,
) -> ScopedQueryBasisContext {
    let intent = ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::CurrentHead,
        false,
    );
    let identity = ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        None,
        snapshot_identity.evidence_identity(),
        family.read_graph().schema_basis().clone(),
        SnapshotLineageClass::CurrentHead,
    );
    let basis = resolve_snapshot_basis(intent, identity, BasisResolutionMode::RuntimeDirect)
        .expect("runtime basis should resolve");
    let preflight = preflight_execution_basis(family.read_graph().execution_plan().clone(), basis)
        .expect("family preflight should build");
    admit_query_basis_context(
        basis_lifecycle().current_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("current basis context should admit")
}

fn certification_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "certification-read-composition",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            1,
        )],
    )
}
