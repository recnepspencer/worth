use super::support::*;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::facade::certification::{
    certify_intent_admission, worth_query_intent_admission_certification_output_manifest,
    worth_query_intent_admission_closeout_extension_outputs,
    worth_query_intent_admission_mutation_audit,
    worth_query_intent_admission_required_certification_outputs,
    worth_query_intent_admission_support_matrix,
};
use crate::facade::foundation::{
    preflight_execution_basis, resolve_snapshot_basis, BasisAuthorityFamily, BasisResolutionMode,
    ExecutionBasisIntent, ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::facade::policy::{QueryContextBindingSource, ScopedQueryBasisContext};
use crate::facade::runtime::{
    worth_query_intent_admission_coverage_inventory, worth_query_intent_admission_family_inventory,
    WorthQueryAdmittedIntentExecutionHandoff, WorthQueryAuthoritativeIntentExecutionHandoff,
    WorthQueryAuthorityLane, WorthQueryEffectTriggeredIntentExecutionHandoff,
    WorthQueryIntentAdmissionAuthorityLaneEligibility, WorthQueryIntentAdmissionBasisEligibility,
    WorthQueryIntentAdmissionCapabilityEligibility, WorthQueryIntentAdmissionCoverageStatus,
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionFamily, WorthQueryIntentAdmissionInvariantEligibility,
    WorthQueryIntentAdmissionPolicyEligibility, WorthQueryIntentAdmissionPreDecisionPosture,
    WorthQueryIntentAdmissionProjectionSourceEligibility,
    WorthQueryIntentAdmissionRoutingSupportEligibility, WorthQueryIntentAdmissionSlopeLane,
    WorthQueryIntentAdmissionSourceLaneEligibility, WorthQueryIntentAdmissionSupportEligibility,
    WorthQueryIntentAdmissionSupportPosture, WorthQueryIntentAdmissionWidthRunScale,
    WorthQueryIntentConsumerOutcomeClass, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryIntentDecisionTraceEnvelopeKind, WorthQueryIntentDecisionTraceEvidence,
    WorthQueryIntentDecisionTraceEvidenceOwner, WorthQueryIntentDecisionTraceStage,
    WorthQueryIntentDeclaration, WorthQueryIntentNonAdmittedStop, WorthQueryIntentSourceLane,
    WorthQueryWorkspace,
};
use crate::facade::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_legacy_query_basis_context,
    QueryBasisContextRequest,
};
use crate::intent_admission::admit_runtime_intent_request;
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView};
use std::cell::Cell;
use std::rc::Rc;

mod certification;
mod dx;
mod execution;
mod inventory;
mod inventory_mutation;
mod phases;

pub(in crate::runtime::tests) fn intent_runtime_with_authority<
    T: WorthQueryIntentAuthorityAdapter + 'static,
>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    authority: T,
) -> WorthQueryRuntime {
    bridge_runtime_with_support_and_intent_authority(
        request_execution,
        intent_support_profile(),
        authority,
    )
}

fn trace_stages(
    envelope: &WorthQueryIntentDecisionTraceEnvelope,
) -> Vec<WorthQueryIntentDecisionTraceStage> {
    envelope.rows().iter().map(|row| row.stage()).collect()
}

fn read_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ),
    )
}

fn manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition",
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

fn identity_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> crate::runtime::WorthQueryReadFamily {
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

fn current_context_for_family(
    family: &crate::runtime::WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let intent = ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::CurrentHead,
        false,
    );
    let identity = ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        None,
        crate::memory_workspace::admit_external_snapshot_label(snapshot_token).evidence_identity(),
        family.read_graph().schema_basis().clone(),
        SnapshotLineageClass::CurrentHead,
    );
    let basis = resolve_snapshot_basis(intent, identity, BasisResolutionMode::RuntimeDirect)
        .expect("runtime basis should resolve");
    let preflight = preflight_execution_basis(family.read_graph().execution_plan().clone(), basis)
        .expect("family preflight should build");
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::current_branch_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("current basis context should bind");
    admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("current basis context should admit")
}

fn profile_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> crate::runtime::WorthQueryReadFamily {
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
