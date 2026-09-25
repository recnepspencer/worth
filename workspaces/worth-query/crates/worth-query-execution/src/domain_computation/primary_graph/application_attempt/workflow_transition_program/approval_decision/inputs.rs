use super::*;

pub(super) struct ObservedApprovalEvidence {
    pub(super) node_path: String,
    pub(super) evidence:
        super::super::super::workflow_instance_observation::ObservedWorkflowAssessmentEvidence,
}

pub(super) struct ValidatedApprovalInputs {
    pub(super) proposal_entity: worth_relational::facade::identity::EntityId,
    pub(super) evidence: Vec<ObservedApprovalEvidence>,
    pub(super) currentness: Vec<super::super::super::WorthQueryApplicationObservedFact>,
    pub(super) facts: Vec<super::super::super::WorthQueryApplicationObservedFact>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    graph_layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    resource: worth_relational::facade::identity::EntityId,
    instance: &super::super::super::PublishedWorkflowInstanceRef,
    progress: &crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress,
    approval: worth_relational::facade::identity::EntityId,
    proposal: Option<&super::super::super::PublishedWorkflowProposalRef>,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<ValidatedApprovalInputs, WorthQueryApplicationAttemptDenial> {
    let proposal_source = unique(compiled.approval_proposal_sources(approval), "proposal")?;
    if proposal.is_some_and(|proposal| proposal.node_path() != proposal_source.path()) {
        return Err(affinity(
            "approval proposal source differs from authored input",
        ));
    }
    let proposal_transition = progress
        .latest_transition(proposal_source.entity())
        .ok_or_else(|| affinity("approval proposal transition is absent"))?;
    let (proposal_entity, proposal_identity, mut facts) = handle.with_runtime(|runtime| {
        super::super::super::workflow_instance_observation::observe_retained_workflow_proposal_identity(
            runtime,
            snapshot,
            layout,
            proposal_transition,
            maximum_facts,
        )
    })?;
    if proposal.is_some_and(|proposal| {
        proposal_entity != proposal.entity_id() || proposal_identity != proposal.identity()
    }) {
        return Err(affinity("approval proposal binding changed"));
    }

    let evidence_source = unique(compiled.approval_evidence_sources(approval), "evidence")?;
    if !matches!(
        evidence_source.kind(),
        crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::EvidenceJoin { .. }
    ) {
        return Err(affinity("approval evidence source is not a join"));
    }
    let join_transition = progress
        .latest_transition(evidence_source.entity())
        .ok_or_else(|| affinity("approval joined evidence is absent"))?;
    handle.with_runtime(|runtime| {
        super::super::super::workflow_instance_observation::observe_retained_transition(
            runtime,
            snapshot,
            layout,
            join_transition,
            &mut facts,
        )
    })?;
    if join_transition.settlement().outcome()
        != worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::EvidenceSatisfied
    {
        return Err(affinity("approval joined evidence is absent"));
    }
    enforce_fact_budget(facts.len(), maximum_facts)?;

    let authored = compiled
        .required_assessments(evidence_source.entity())
        .collect::<Vec<_>>();
    if authored.len() < 2 {
        return Err(affinity("approval evidence inventory is incomplete"));
    }
    let mut required = Vec::new();
    let mut observed_proposals =
        super::super::assessment_coverage::AssessmentProposalObservations::default();
    for node in authored {
        let remaining = maximum_facts.saturating_sub(facts.len());
        let subject = super::super::assessment_coverage::observe_subject_cached(
            compiled,
            layout,
            progress,
            node,
            resource,
            handle,
            snapshot,
            remaining,
            &mut observed_proposals,
        )?;
        let remaining =
            maximum_facts.saturating_sub(facts.len().saturating_add(subject.facts.len()));
        let observed = handle.with_runtime(|runtime| {
            super::super::assessment_applicability::observe(
                node,
                graph_layout,
                runtime,
                snapshot,
                subject.resource,
                subject.related,
                remaining,
            )
        })?;
        facts.extend(subject.facts);
        facts.extend(observed.facts);
        if observed.applicable {
            required.push((node, subject.coverage));
        }
    }
    enforce_fact_budget(facts.len(), maximum_facts)?;
    if required.is_empty() {
        return Err(affinity(
            "approval has no currently applicable authored requirement",
        ));
    }
    let mut evidence = Vec::with_capacity(required.len());
    let mut currentness = Vec::new();
    for (node, coverage) in required {
        let locator = progress
            .latest_assessment_evidence(node.entity())
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete,
                    "approval required evidence is absent",
                )
            })?;
        let remaining = maximum_facts.saturating_sub(facts.len());
        let (observed, mut retained_facts) = handle.with_runtime(|runtime| {
            super::super::super::workflow_instance_observation::observe_retained_assessment_evidence(
                runtime,
                snapshot,
                layout,
                locator,
                remaining,
            )
        })?;
        facts.append(&mut retained_facts);
        let remaining = maximum_facts.saturating_sub(facts.len());
        let coverage_state = super::super::assessment_coverage::observe(
            node, &coverage, &observed, layout, handle, snapshot, remaining,
        )?;
        facts.extend(coverage_state.facts);
        if !coverage_state.current {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                "approval required evidence is stale",
            ));
        }
        if !observed.passing {
            return Err(affinity("approval required evidence is failing"));
        }
        currentness.extend(coverage_state.dependencies);
        evidence.push(ObservedApprovalEvidence {
            node_path: node.path().to_owned(),
            evidence: observed,
        });
    }
    if proposal
        .is_some_and(|proposal| proposal.definition_entity_id() != instance.definition_entity_id())
    {
        return Err(affinity("approval proposal definition changed"));
    }
    enforce_fact_budget(facts.len(), maximum_facts)?;
    Ok(ValidatedApprovalInputs {
        proposal_entity,
        evidence,
        currentness,
        facts,
    })
}

fn enforce_fact_budget(
    observed: usize,
    maximum: usize,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if observed > maximum {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
            "workflow approval input fact budget",
        ));
    }
    Ok(())
}
