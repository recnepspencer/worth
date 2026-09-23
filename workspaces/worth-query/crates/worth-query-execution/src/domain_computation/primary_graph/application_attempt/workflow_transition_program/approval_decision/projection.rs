use worth_foundational::facade::AspectValue;

use super::*;

pub(super) fn prepare(
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    meaning: &crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning,
) -> Result<publication::PreparedWorkflowApprovalProjection, WorthQueryApplicationAttemptDenial> {
    let mut fields = crate::domain_computation::primary_graph::workflow::workflow_approval_fields(
        layout, meaning,
    );
    let required_fields = [
        layout.approval.authorization_decision.clone(),
        layout.approval.authorization_sample.clone(),
    ];
    for locator in &required_fields {
        fields.remove(locator);
    }
    let AspectValue::UInt64(expiry) = meaning.expiry else {
        return Err(affinity("workflow approval expiry has the wrong type"));
    };
    Ok(publication::PreparedWorkflowApprovalProjection {
        identity: meaning.identity_text.clone(),
        identity_locator: layout.approval.identity.clone(),
        fields: fields.into_iter().collect(),
        required_fields: required_fields.into(),
        decision: meaning.decision,
        proposal: meaning.proposal,
        evidence: meaning.evidence.clone(),
        approver: meaning.approver,
        purpose: meaning.purpose.clone(),
        expiry,
        transition_relation: layout.transition_approval_relation,
        proposal_relation: layout.approval_proposal_relation,
        evidence_relation: layout.approval_evidence_relation,
    })
}
