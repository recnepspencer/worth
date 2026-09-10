use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_identity_basis::{
    canonical_indexed_operation_material, canonical_operation_material, graph_call_kind_material,
    operation_result_state_material, workflow_warning_material,
};
use crate::identity::hash_parts;

use super::{WorthQueryWorkflowRun, WorthQueryWorkflowStageReceipt};

pub(super) fn semantic_trace_identity<D, O, F, L: BasisOperationLane>(
    run: &WorthQueryWorkflowRun<D, O, F, L>,
) -> String {
    let mut semantic_parts = run
        .receipts
        .iter()
        .map(stage_semantic_part)
        .collect::<Vec<_>>();
    semantic_parts.sort();
    hash_parts(&[
        "worth_query_workflow_semantic_trace_v1".into(),
        format!("operation:{}", run.bound.definition().canonical_identity()),
        format!(
            "operation_conditional:{}",
            canonical_indexed_operation_material(
                "workflow.operation.conditional",
                run.operation_conditional_provenance().iter().map(
                    super::super::workflow_conditional_trace::conditional_trace_semantic_material,
                ),
            )
        ),
        format!("stages:{}", semantic_parts.join("|")),
    ])
}

fn stage_semantic_part(receipt: &WorthQueryWorkflowStageReceipt) -> String {
    canonical_operation_material(vec![
        ("stage.identity", receipt.stage_identity.clone()),
        (
            "stage.predecessors",
            canonical_indexed_operation_material(
                "stage.predecessor",
                receipt.predecessor_stage_identities.iter().cloned(),
            ),
        ),
        (
            "stage.result_state",
            operation_result_state_material(receipt.result_state).into(),
        ),
        (
            "stage.output",
            crate::domain_installation::operation_identity_basis::workflow_semantic_value_material(
                &receipt.output_semantics,
            ),
        ),
        (
            "stage.warnings",
            canonical_indexed_operation_material(
                "stage.warning",
                receipt.warnings.iter().map(workflow_warning_material),
            ),
        ),
        (
            "stage.domain_evidence",
            receipt
                .domain_evidence()
                .map(super::super::WorthQueryAdmittedDomainEvidence::replay_meaning)
                .map(|meaning| meaning.semantic_material())
                .unwrap_or_else(|| "not-required".into()),
        ),
        (
            "stage.graph",
            canonical_indexed_operation_material(
                "stage.graph.receipt",
                receipt.graph_receipts.iter().map(|graph| {
                    canonical_operation_material(vec![
                        ("graph.role", graph.role().into()),
                        ("graph.kind", graph_call_kind_material(graph.kind()).into()),
                        ("graph.evidence", graph.evidence_identity().into()),
                        (
                            "graph.projection",
                            graph
                                .graph_read_product()
                                .map(|projection| projection.call_identity())
                                .unwrap_or("not-projected")
                                .into(),
                        ),
                    ])
                }),
            ),
        ),
        (
            "stage.reads",
            canonical_indexed_operation_material(
                "stage.read",
                receipt.primary_read_evidence.iter().map(|read| {
                    canonical_operation_material(vec![
                        ("read.role", read.role().into()),
                        ("read.result", read.read_receipt().result_digest().into()),
                    ])
                }),
            ),
        ),
        (
            "stage.effects",
            canonical_indexed_operation_material(
                "stage.effect",
                receipt.effect_evidence.iter().map(|effect| {
                    canonical_operation_material(vec![
                        ("effect.family", effect.family().as_str().into()),
                        ("effect.receipt", effect.receipt_identity().into()),
                    ])
                }),
            ),
        ),
        (
            "stage.invariants",
            canonical_indexed_operation_material(
                "stage.invariant",
                receipt.invariant_outcomes.iter().map(|outcome| {
                    canonical_operation_material(vec![
                        ("invariant.role", outcome.invariant_role().into()),
                        (
                            "invariant.installed",
                            outcome.installed_invariant_identity().into(),
                        ),
                    ])
                }),
            ),
        ),
    ])
}
