//! Read-only locator projection at one validated native instance/transition basis.
use super::{
    reference::{ProposalText, PublishedWorkflowProposalRef},
    *,
};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::read_fields::{identity, ReadFields},
    application_discovery::WorthQueryWorkflowReadDenial as Denial,
    workflow::schema::WorthQueryWorkflowLayout,
};
pub(in crate::domain_computation::primary_graph) fn observe(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    instance: EntityId,
    definition: EntityId,
    transition: EntityId,
) -> Result<Option<PublishedWorkflowProposalRef>, Denial> {
    let Some(entity) = fields.one(layout.transition_proposal_relation, transition)? else {
        return Ok(None);
    };
    if fields.incoming_one(layout.transition_proposal_relation, entity)? != transition
        || fields.incoming_one(layout.instance_transition_relation, transition)? != instance
    {
        return Err(Denial::Corrupt);
    }
    let kind = layout.proposal.entity_kind;
    fields.protocol(entity, kind, &layout.proposal.protocol_version)?;
    let id = fields.text(entity, kind, &layout.proposal.identity)?;
    identity(id.bytes())?;
    let operation = fields.text(entity, kind, &layout.proposal.operation)?;
    let input_type = fields.text(entity, kind, &layout.proposal.input_type)?;
    let node_path = fields.text(entity, kind, &layout.proposal.node_path)?;
    if fields.one(layout.instance_definition_relation, instance)? != Some(definition) {
        return Err(Denial::Corrupt);
    }
    let native_node = fields
        .one(layout.transition_node_relation, transition)?
        .ok_or(Denial::Corrupt)?;
    fields.protocol(
        transition,
        layout.transition.entity_kind,
        &layout.transition.protocol_version,
    )?;
    let node =
        crate::domain_computation::primary_graph::workflow::definition::retained_descriptor::node(
            fields,
            layout,
            definition,
            native_node,
        )?;
    let selected=crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowProposal::from_node(&node).ok_or(Denial::Corrupt)?;
    if !selected.matches_recorded(
        std::str::from_utf8(operation.bytes()).map_err(|_| Denial::Corrupt)?,
        std::str::from_utf8(input_type.bytes()).map_err(|_| Denial::Corrupt)?,
        std::str::from_utf8(node_path.bytes()).map_err(|_| Denial::Corrupt)?,
    ) {
        return Err(Denial::Corrupt);
    }
    let input_identity = identity(
        fields
            .text(entity, kind, &layout.proposal.input_identity)?
            .bytes(),
    )?;
    let source_identity = fields
        .optional_text(entity, kind, &layout.proposal.source_identity)?
        .map(|v| identity(v.bytes()))
        .transpose()?;
    Ok(Some(PublishedWorkflowProposalRef {
        branch,
        entity,
        instance,
        definition,
        identity: ProposalText::Read(id),
        operation: ProposalText::Read(operation),
        input_type: ProposalText::Read(input_type),
        input_identity,
        source_identity,
        node_path: ProposalText::Read(node_path),
    }))
}
