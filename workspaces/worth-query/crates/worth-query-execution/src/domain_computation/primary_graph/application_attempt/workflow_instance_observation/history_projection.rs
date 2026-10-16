//! Read-only row projections. Audit is admitted by the caller before any role read.
use super::read_fields::ReadFields;
use super::*;
use crate::domain_computation::primary_graph::application_discovery::{
    WorthQueryWorkflowApprovalRead as Approval, WorthQueryWorkflowHistoryAudit as Audit,
    WorthQueryWorkflowHistoryEntry as Entry, WorthQueryWorkflowReadDenial as Denial,
    WorthQueryWorkflowRecordedActor as Actor,
};
pub(in crate::domain_computation::primary_graph) fn observe(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    instance: EntityId,
    definition: EntityId,
    transition: EntityId,
    audited: bool,
    maximum_connections: usize,
) -> Result<Entry, Denial> {
    let mut facts = Vec::new();
    let settlement = settlement::observe_transition_state(
        fields.execution,
        fields.runtime,
        fields.snapshot(),
        layout,
        transition,
        &mut facts,
    )
    .map_err(Denial::Execution)?;
    let path = fields.text(
        settlement.node(),
        layout.node.entity_kind,
        &layout.node.path,
    )?;
    let proposal = super::super::workflow_proposal_program::observe_read_only_proposal(
        fields, layout, branch, instance, definition, transition,
    )?;
    let approval_entity = fields.one(layout.transition_approval_relation, transition)?;
    let approval = approval_entity
        .map(|approval| {
            observe_approval(
                fields,
                layout,
                branch,
                instance,
                definition,
                settlement.node(),
                approval,
                settlement.outcome(),
                maximum_connections,
            )
        })
        .transpose()?;
    let audit = if audited {
        let roles = observe_transition_roles(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            transition,
            settlement.operation_receipt_identity().is_some(),
            &mut facts,
        )
        .map_err(Denial::Execution)?;
        let proposer = proposal
            .as_ref()
            .map(|p| {
                role(
                    fields,
                    &layout.proposal.proposer,
                    p.entity_id(),
                    layout.proposal.entity_kind,
                    &mut facts,
                )
            })
            .transpose()?;
        let approver=approval_entity.map(|id|role(fields,&crate::domain_computation::primary_graph::workflow::actor::record::WorkflowActorFields {partition:layout.approval.approver_partition.clone(),slot:layout.approval.approver_slot.clone(),generation:layout.approval.approver_generation.clone()},id,layout.approval.entity_kind,&mut facts)).transpose()?;
        Audit::Visible {
            advancer: Actor::observed(roles.advancer),
            executor: roles.executor.map(Actor::observed),
            proposer,
            approver,
        }
    } else {
        Audit::Withheld
    };
    Ok(Entry {
        occurrence: settlement.occurrence(),
        path,
        outcome: settlement.outcome(),
        proposal,
        approval,
        audit,
    })
}
fn observe_approval(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    instance: EntityId,
    definition: EntityId,
    approval_node: EntityId,
    approval: EntityId,
    outcome:worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome,
    maximum_connections: usize,
) -> Result<Approval, Denial> {
    let kind = layout.approval.entity_kind;
    fields.protocol(approval, kind, &layout.approval.protocol_version)?;
    let decision = fields.text(approval, kind, &layout.approval.decision)?;
    use super::super::WorkflowApprovalDecision as Decision;
    let decision = match decision.bytes() {
        b"approve" => Decision::Approve,
        b"reject" => Decision::Reject,
        _ => return Err(Denial::Corrupt),
    };
    if decision.outcome() != outcome {
        return Err(Denial::Corrupt);
    }
    // Producer encoding names this full native EntityId, not the start-intent digest field.
    let expected_instance =
        crate::domain_computation::primary_graph::workflow::workflow_approval_instance_identity(
            instance,
        );
    let approved_instance = fields.text(approval, kind, &layout.approval.instance_identity)?;
    if approved_instance.bytes() != expected_instance.as_bytes() {
        return Err(Denial::Corrupt);
    }
    if fields.one(layout.instance_definition_relation, instance)? != Some(definition) {
        return Err(Denial::Corrupt);
    }
    let expected_definition = fields.text(
        definition,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
    )?;
    let recorded_definition =
        fields.text(approval, kind, &layout.approval.definition_content_identity)?;
    if expected_definition.bytes() != recorded_definition.bytes() {
        return Err(Denial::Corrupt);
    }
    let descriptor=crate::domain_computation::primary_graph::workflow::definition::retained_descriptor::approval(fields,layout,definition,approval_node,maximum_connections)?;
    let operation = fields.text(approval, kind, &layout.approval.approval_operation)?;
    let target_operation = fields.text(approval, kind, &layout.approval.target_operation)?;
    let capability = fields.text(
        approval,
        kind,
        &layout.approval.installed_capability_identity,
    )?;
    if !descriptor.matches_recorded(
        std::str::from_utf8(operation.bytes()).map_err(|_| Denial::Corrupt)?,
        std::str::from_utf8(target_operation.bytes()).map_err(|_| Denial::Corrupt)?,
        std::str::from_utf8(capability.bytes()).map_err(|_| Denial::Corrupt)?,
    ) {
        return Err(Denial::Corrupt);
    }
    let proposal = fields
        .one(layout.approval_proposal_relation, approval)?
        .ok_or(Denial::Corrupt)?;
    let proposal_transition = fields.incoming_one(layout.transition_proposal_relation, proposal)?;
    if fields.one(layout.transition_node_relation, proposal_transition)?
        != Some(descriptor.proposal_node)
    {
        return Err(Denial::Corrupt);
    }
    // Validate the referenced proposal even when its source transition is outside this bounded page.
    let validated = super::super::workflow_proposal_program::observe_read_only_proposal(
        fields,
        layout,
        branch,
        instance,
        definition,
        proposal_transition,
    )?
    .ok_or(Denial::Corrupt)?;
    if validated.entity_id() != proposal {
        return Err(Denial::Corrupt);
    }
    let proposal_identity = fields.text(
        proposal,
        layout.proposal.entity_kind,
        &layout.proposal.identity,
    )?;
    // The view does not decode the authority-producing authorization envelope.
    // Both approve and reject are recorded data, independent of current approval permission.
    Ok(Approval {
        decision,
        proposal_identity,
    })
}
fn role(
    fields: &ReadFields<'_, '_, '_>,
    locators:&crate::domain_computation::primary_graph::workflow::actor::record::WorkflowActorFields,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Actor, Denial> {
    let actor = locators
        .observe(
            fields.runtime,
            fields.execution,
            fields.snapshot(),
            entity,
            kind,
            facts,
        )
        .map_err(Denial::Execution)?;
    let actor=crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability(actor,true).map_err(Denial::Execution)?.ok_or(Denial::Corrupt)?;
    Ok(Actor::observed(actor))
}
