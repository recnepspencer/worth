//! Projects one landed workflow approval without folding request stops.
use super::*;
pub(super) fn project_approval(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    transition: worth_relational::facade::identity::EntityId,
    approval: PreparedWorkflowApprovalProjection,
) -> Result<
    Option<PerformedWorkflowApproval>,
    worth_relational::facade::transactions::RelationalExecutionDenialCause,
> {
    let expected = AspectValue::String(InternedString::Raw(approval.identity.clone()));
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            (receipt
                .committed_changes()
                .committed_field_values(entity, &[&approval.identity_locator])
                == Some(vec![expected.clone()]))
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    let [entity] = candidates.as_slice() else {
        return Ok(None);
    };
    let locators = approval
        .fields
        .iter()
        .map(|(locator, _)| locator)
        .collect::<Vec<_>>();
    let expected_fields = approval
        .fields
        .iter()
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();
    if receipt
        .committed_changes()
        .committed_field_values(*entity, &locators)
        != Some(expected_fields)
    {
        return Ok(None);
    }
    let required = approval.required_fields.iter().collect::<Vec<_>>();
    if receipt
        .committed_changes()
        .committed_field_values(*entity, &required)
        .is_none()
    {
        return Ok(None);
    }
    // The committing branch's own state: a fork's approval never lands in
    // main's edition.
    let Ok(committed) = runtime
        .read_truth()
        .try_project_historical_version(receipt.committed_changes().commit_reference().version_id)
    else {
        return Ok(None);
    };
    // One unit for the adjacency list and two per expected relation: an
    // unexpected extra relation exhausts the bound and fails closed.
    let exact_targets = |kind, from, expected: usize| {
        committed
            .bounded_outgoing_relations_for_frontier(
                execution,
                &std::collections::BTreeSet::from([from]),
                kind,
                expected.saturating_mul(2).saturating_add(1),
            )
            .map(|answer| {
                answer.ok().map(|read| {
                    read.into_records()
                        .into_iter()
                        .map(|record| record.target)
                        .collect::<Vec<_>>()
                })
            })
    };
    if exact_targets(approval.transition_relation, transition, 1)?.as_deref() != Some(&[*entity])
        || exact_targets(approval.proposal_relation, *entity, 1)?.as_deref()
            != Some(&[approval.proposal])
    {
        return Ok(None);
    }
    let mut expected_evidence = approval.evidence.to_vec();
    expected_evidence.sort_unstable();
    let Some(mut actual_evidence) =
        exact_targets(approval.evidence_relation, *entity, expected_evidence.len())?
    else {
        return Ok(None);
    };
    actual_evidence.sort_unstable();
    if actual_evidence != expected_evidence {
        return Ok(None);
    }
    Ok(Some(PerformedWorkflowApproval {
        entity: *entity,
        identity: approval.identity,
        decision: approval.decision,
        proposal: approval.proposal,
        evidence: approval.evidence,
        approver: approval.approver,
        purpose: approval.purpose,
        expiry: approval.expiry,
    }))
}
