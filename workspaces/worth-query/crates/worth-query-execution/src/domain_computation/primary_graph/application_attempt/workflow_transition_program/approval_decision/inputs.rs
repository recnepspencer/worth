use super::*;

pub(super) struct ObservedApprovalEvidence {
    pub(super) node_path: String,
    pub(super) evidence:
        super::super::super::workflow_instance_observation::ObservedWorkflowAssessmentEvidence,
}

pub(super) struct ValidatedApprovalInputs {
    pub(super) evidence: Vec<ObservedApprovalEvidence>,
    pub(super) currentness: Vec<super::super::super::WorthQueryApplicationObservedFact>,
    pub(super) facts: Vec<super::super::super::WorthQueryApplicationObservedFact>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    instance: &super::super::super::PublishedWorkflowInstanceRef,
    progress: &crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress,
    approval: worth_relational::facade::identity::EntityId,
    proposal: &super::super::super::PublishedWorkflowProposalRef,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<ValidatedApprovalInputs, WorthQueryApplicationAttemptDenial> {
    let proposal_source = unique(compiled.approval_proposal_sources(approval), "proposal")?;
    if proposal.node_path() != proposal_source.path() {
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
    if proposal_entity != proposal.entity_id() || proposal_identity != proposal.identity() {
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

    let required = compiled
        .required_assessments(evidence_source.entity())
        .collect::<Vec<_>>();
    if required.len() < 2 {
        return Err(affinity("approval evidence inventory is incomplete"));
    }
    let mut evidence = Vec::with_capacity(required.len());
    let mut currentness = Vec::new();
    for node in required {
        let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Assessment {
            query,
            parameter_type,
            result_type,
            binding,
            ..
        } = node.kind() else {
            return Err(affinity("approval evidence requirement is not an assessment"));
        };
        let locator = progress
            .latest_assessment_evidence(node.entity())
            .ok_or_else(|| affinity("approval required evidence is absent"))?;
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
        if observed.query != *query
            || observed.parameter_type != *parameter_type
            || observed.result_type != *result_type
            || observed.binding != *binding
        {
            return Err(affinity(
                "approval evidence contract differs from its authored requirement",
            ));
        }
        if !observed.passing {
            return Err(affinity("approval required evidence is failing"));
        }
        let remaining = maximum_facts.saturating_sub(facts.len());
        currentness.extend(handle.with_runtime(|runtime| {
            super::super::super::workflow_instance_observation::observe_evidence_dependencies(
                runtime,
                snapshot,
                layout,
                observed.entity,
                remaining,
                &mut facts,
            )
        })?);
        evidence.push(ObservedApprovalEvidence {
            node_path: node.path().to_owned(),
            evidence: observed,
        });
    }
    if proposal.definition_entity_id() != instance.definition_entity_id() {
        return Err(affinity("approval proposal definition changed"));
    }
    enforce_fact_budget(facts.len(), maximum_facts)?;
    Ok(ValidatedApprovalInputs {
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
