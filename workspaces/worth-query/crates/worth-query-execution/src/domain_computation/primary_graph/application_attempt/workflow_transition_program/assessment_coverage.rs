use std::collections::BTreeMap;

use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::{
    workflow_instance_observation::{
        observe_evidence_dependencies, observe_retained_workflow_proposal_identity,
        ObservedWorkflowAssessmentEvidence,
    },
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{
        CompiledWorkflowAssessmentApplicability, CompiledWorkflowDefinition, CompiledWorkflowNode,
        CompiledWorkflowNodeKind,
    },
    instance::WorkflowInstanceProgress,
    proposal::{observe_workflow_proposal_coverages, WorkflowProposalCoverageMeaning},
    schema::WorthQueryWorkflowLayout,
};

pub(super) struct ObservedAssessmentSubject {
    pub(super) coverage: WorkflowProposalCoverageMeaning,
    pub(super) resource: EntityId,
    pub(super) related: Option<EntityId>,
    pub(super) proposal_identity: String,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) struct ObservedAssessmentCoverage {
    pub(super) current: bool,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) dependencies: Vec<WorthQueryApplicationObservedFact>,
}

#[derive(Default)]
pub(super) struct AssessmentProposalObservations {
    by_source: BTreeMap<EntityId, ObservedProposalCoverage>,
}

struct ObservedProposalCoverage {
    identity: String,
    coverages: Vec<WorkflowProposalCoverageMeaning>,
}

/// The authored source and published proposal, not the admission request, own
/// the assessment subject. Resource coverage must agree with admitted scope.
#[allow(clippy::too_many_arguments)]
pub(super) fn observe_subject(
    compiled: &CompiledWorkflowDefinition,
    layout: &WorthQueryWorkflowLayout,
    progress: &WorkflowInstanceProgress,
    node: &CompiledWorkflowNode,
    admitted_resource: EntityId,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<ObservedAssessmentSubject, WorthQueryApplicationAttemptDenial> {
    observe_subject_cached(
        compiled,
        layout,
        progress,
        node,
        admitted_resource,
        handle,
        snapshot,
        maximum_facts,
        &mut AssessmentProposalObservations::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_subject_cached(
    compiled: &CompiledWorkflowDefinition,
    layout: &WorthQueryWorkflowLayout,
    progress: &WorkflowInstanceProgress,
    node: &CompiledWorkflowNode,
    admitted_resource: EntityId,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
    observed_proposals: &mut AssessmentProposalObservations,
) -> Result<ObservedAssessmentSubject, WorthQueryApplicationAttemptDenial> {
    let CompiledWorkflowNodeKind::Assessment {
        subject,
        applicability,
        ..
    } = node.kind()
    else {
        return Err(mismatch("required evidence node is not an assessment"));
    };
    let mut sources = compiled.assessment_subject_sources(node.entity());
    let source = sources
        .next()
        .ok_or_else(|| mismatch("assessment proposal source is absent"))?;
    if sources.next().is_some() {
        return Err(mismatch("assessment proposal source is ambiguous"));
    }
    let mut facts = Vec::new();
    if !observed_proposals.by_source.contains_key(&source.entity()) {
        let source_transition = progress
            .latest_transition(source.entity())
            .ok_or_else(|| mismatch("assessment proposal transition is absent"))?;
        let (proposal, identity, mut proposal_facts) = handle.with_runtime(|runtime| {
            observe_retained_workflow_proposal_identity(
                runtime,
                snapshot,
                layout,
                source_transition,
                maximum_facts,
            )
        })?;
        let (coverages, coverage_facts) = handle.with_runtime(|runtime| {
            observe_workflow_proposal_coverages(
                runtime,
                snapshot,
                layout,
                proposal,
                maximum_facts.saturating_sub(proposal_facts.len()),
            )
        })?;
        proposal_facts.extend(coverage_facts);
        enforce_budget(proposal_facts.len(), maximum_facts)?;
        observed_proposals.by_source.insert(
            source.entity(),
            ObservedProposalCoverage {
                identity,
                coverages,
            },
        );
        facts = proposal_facts;
    }
    let observed = observed_proposals
        .by_source
        .get(&source.entity())
        .expect("proposal observation was inserted above");
    let coverage = observed
        .coverages
        .iter()
        .find(|coverage| &coverage.selector == subject)
        .cloned()
        .ok_or_else(|| mismatch("assessment subject is absent from proposal coverage"))?;
    if matches!(subject, ApplicationWorkflowSubjectSelector::Resource)
        && coverage.subject != admitted_resource
    {
        return Err(mismatch(
            "assessment proposal resource differs from admitted scope",
        ));
    }
    if matches!(
        applicability,
        CompiledWorkflowAssessmentApplicability::WhenRelatedRelationPresent { .. }
    ) {
        let resource = observed
            .coverages
            .iter()
            .find(|coverage| coverage.selector == ApplicationWorkflowSubjectSelector::Resource)
            .ok_or_else(|| mismatch("assessment proposal resource coverage is absent"))?;
        if resource.subject != admitted_resource {
            return Err(mismatch(
                "assessment proposal resource differs from admitted scope",
            ));
        }
    }
    enforce_budget(facts.len(), maximum_facts)?;
    let related =
        matches!(subject, ApplicationWorkflowSubjectSelector::Related).then_some(coverage.subject);
    Ok(ObservedAssessmentSubject {
        coverage,
        resource: admitted_resource,
        related,
        proposal_identity: observed.identity.clone(),
        facts,
    })
}

/// A retained node locator is only a lookup. The authored contract, selected
/// proposal coverage, and native dependency versions decide reuse.
pub(super) fn observe(
    node: &CompiledWorkflowNode,
    coverage: &WorkflowProposalCoverageMeaning,
    evidence: &ObservedWorkflowAssessmentEvidence,
    layout: &WorthQueryWorkflowLayout,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<ObservedAssessmentCoverage, WorthQueryApplicationAttemptDenial> {
    let CompiledWorkflowNodeKind::Assessment {
        query,
        parameter_type,
        result_type,
        binding,
        ..
    } = node.kind()
    else {
        return Err(mismatch("required evidence node is not an assessment"));
    };
    if evidence.query != *query
        || evidence.parameter_type != *parameter_type
        || evidence.result_type != *result_type
        || evidence.binding != *binding
        || evidence.proposal_identity.is_empty()
    {
        return Err(mismatch(
            "assessment evidence contract differs from its authored requirement",
        ));
    }
    if evidence.subject != coverage.subject || evidence.coverage_identity != coverage.identity {
        return Ok(ObservedAssessmentCoverage {
            current: false,
            facts: Vec::new(),
            dependencies: Vec::new(),
        });
    }
    let (dependencies, facts, current) =
        observe_native_dependencies(evidence, layout, handle, snapshot, maximum_facts)?;
    enforce_budget(facts.len(), maximum_facts)?;
    if !current {
        return Ok(ObservedAssessmentCoverage {
            current: false,
            facts,
            dependencies: Vec::new(),
        });
    }
    Ok(ObservedAssessmentCoverage {
        current: true,
        facts,
        dependencies,
    })
}

pub(super) fn observe_native_dependencies(
    evidence: &ObservedWorkflowAssessmentEvidence,
    layout: &WorthQueryWorkflowLayout,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<
    (
        Vec<WorthQueryApplicationObservedFact>,
        Vec<WorthQueryApplicationObservedFact>,
        bool,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let mut facts = Vec::new();
    let dependencies = handle.with_runtime(|runtime| {
        observe_evidence_dependencies(
            runtime,
            snapshot,
            layout,
            evidence.entity,
            maximum_facts,
            &mut facts,
        )
    })?;
    let current = handle.with_runtime(|runtime| {
        dependencies
            .iter()
            .all(|fact| fact.remains_equal_in(runtime, snapshot))
    });
    Ok((dependencies, facts, current))
}

fn enforce_budget(
    observed: usize,
    maximum: usize,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if observed > maximum {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
            "workflow assessment coverage fact budget",
        ));
    }
    Ok(())
}

fn mismatch(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
