use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationalAdjacencyDirection, RelationalRuntime,
};
use worth_relational::facade::storage::RecordLifecycleState;

use crate::domain_computation::primary_graph::application_attempt::{
    fact::observe_adjacency_checked, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowAssessmentApplicability, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};

pub(super) struct ObservedAssessmentApplicability {
    pub(super) applicable: bool,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

/// Requirement membership comes from the authored rule and current native graph,
/// never from the number of assessment evidence facts already published.
#[allow(clippy::too_many_arguments)]
pub(super) fn observe(
    node: &CompiledWorkflowNode,
    layout: &WorthQueryPrimaryGraphLayout,
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    resource: EntityId,
    related: Option<EntityId>,
    remaining_decision_facts: usize,
) -> Result<ObservedAssessmentApplicability, WorthQueryApplicationAttemptDenial> {
    let CompiledWorkflowNodeKind::Assessment {
        subject,
        applicability,
        ..
    } = node.kind()
    else {
        return Err(denial("applicability source is not an authored assessment"));
    };
    match applicability {
        CompiledWorkflowAssessmentApplicability::Always => Ok(ObservedAssessmentApplicability {
            applicable: true,
            facts: Vec::new(),
        }),
        CompiledWorkflowAssessmentApplicability::WhenRelatedRelationPresent {
            relation,
            from,
            to,
        } => {
            if !matches!(subject, ApplicationWorkflowSubjectSelector::Related) {
                return Err(denial("conditional assessment does not select Related"));
            }
            let related = related.ok_or_else(|| denial("related assessment subject is absent"))?;
            let binding = layout
                .relation(relation)
                .ok_or_else(|| denial("applicability relation is not installed"))?;
            if layout.entity_name(binding.from) != Some(from.as_str())
                || layout.entity_name(binding.to) != Some(to.as_str())
            {
                return Err(denial(
                    "applicability relation endpoints differ from authored types",
                ));
            }
            // This rule admits only installed 0..1 outgoing relations. The
            // native adjacency observation then examines at most one edge
            // (two scalar work units: relation and returned endpoint), and
            // its revision also captures absence, insertion, removal, and ABA.
            if binding.source_max != Some(1) {
                return Err(denial(
                    "applicability relation must have source maximum one",
                ));
            }
            if remaining_decision_facts < 3 {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                    "applicability relation requires three decision facts",
                ));
            }
            let view = runtime
                .read_truth()
                .project_snapshot(snapshot)
                .ok_or_else(|| denial("applicability snapshot is unavailable"))?;
            for (entity, kind) in [(resource, binding.from), (related, binding.to)] {
                let matches = view.entity_record_with_projection_scope(
                    entity,
                    ProjectionAspectScope::empty(),
                    |record| {
                        Some(
                            record.kind_id() == kind
                                && record.lifecycle() == RecordLifecycleState::Live,
                        )
                    },
                ) == Some(true);
                if !matches {
                    return Err(denial(
                        "applicability endpoint is absent or has another kind",
                    ));
                }
            }
            let native_revision = view
                .bounded_adjacency_structural_revision(
                    resource,
                    binding.kind,
                    RelationalAdjacencyDirection::Outgoing,
                    1,
                )
                .map_err(|_| denial("applicability native revision is unavailable"))?
                .revision();
            let relations = observe_adjacency_checked(
                runtime,
                snapshot,
                binding.kind,
                resource,
                WorthQueryApplicationAdjacencyDirection::Outgoing,
                2,
            )
            .map_err(|_| {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                    "applicability relation scan exceeds admitted work",
                )
            })?;
            let applicable = relations.iter().any(|edge| edge.to == related);
            Ok(ObservedAssessmentApplicability {
                applicable,
                facts: vec![
                    WorthQueryApplicationObservedFact::Entity {
                        entity_id: resource,
                        kind: binding.from,
                    },
                    WorthQueryApplicationObservedFact::Entity {
                        entity_id: related,
                        kind: binding.to,
                    },
                    WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                        relation_kind: binding.kind,
                        anchor: resource,
                        direction: RelationalAdjacencyDirection::Outgoing,
                        native_revision,
                        comparison_work_limit: 1,
                        endpoints: Vec::new(),
                    },
                ],
            })
        }
    }
}

fn denial(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
