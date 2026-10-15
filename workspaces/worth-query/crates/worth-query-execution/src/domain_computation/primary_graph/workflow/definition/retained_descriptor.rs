//! Exact retained authored descriptor validation without progression or compilation-cache mutation.
use super::{
    compile_node, observe_connection, CompiledWorkflowConnectionKind, CompiledWorkflowNode,
    CompiledWorkflowNodeKind,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        observe_adjacency, workflow_instance_observation::read_fields::ReadFields,
        WorthQueryApplicationAdjacencyDirection,
    },
    application_discovery::WorthQueryWorkflowReadDenial as Denial,
    workflow::{
        instance::{SelectedWorkflowApproval, SelectedWorkflowProposal},
        schema::WorthQueryWorkflowLayout,
    },
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowDataFlow;
use worth_relational::facade::identity::{EntityId, KindId};
/// A decoded node whose native membership belongs to this exact definition.
pub(in crate::domain_computation::primary_graph) fn node(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    node: EntityId,
) -> Result<CompiledWorkflowNode, Denial> {
    if fields.incoming_one(layout.definition_node_relation, node)? != definition {
        return Err(Denial::Corrupt);
    }
    compile_node(
        fields.execution,
        fields.runtime,
        fields.snapshot(),
        layout,
        node,
        &mut Vec::new(),
    )
    .map_err(Denial::Execution)
}
/// Full authored approval context, not an approval authority or a current grant.
pub(in crate::domain_computation::primary_graph) struct ApprovalDescriptor {
    approval: CompiledWorkflowNode,
    target: CompiledWorkflowNode,
    pub(in crate::domain_computation::primary_graph) proposal_node: EntityId,
}
impl ApprovalDescriptor {
    pub(in crate::domain_computation::primary_graph) fn matches_recorded(
        &self,
        operation: &str,
        target_operation: &str,
        capability: &str,
    ) -> bool {
        let CompiledWorkflowNodeKind::Approval {
            operation: expected,
            installed_capability_identity,
            ..
        } = self.approval.kind()
        else {
            return false;
        };
        expected == operation
            && installed_capability_identity == capability
            && SelectedWorkflowApproval::guarded_target(&self.target)
                .is_ok_and(|expected| expected == target_operation)
    }
}
pub(in crate::domain_computation::primary_graph) fn approval(
    fields: &ReadFields<'_, '_, '_>,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    approval_node: EntityId,
    maximum_connections: usize,
) -> Result<ApprovalDescriptor, Denial> {
    let approval = node(fields, layout, definition, approval_node)?;
    if !matches!(approval.kind(), CompiledWorkflowNodeKind::Approval { .. }) {
        return Err(Denial::Corrupt);
    }
    let bound = usize::try_from(fields.u64(
        definition,
        layout.definition.entity_kind,
        &layout.definition.maximum_connections,
    )?)
    .map_err(|_| Denial::Corrupt)?;
    if bound == 0 {
        return Err(Denial::Corrupt);
    }
    if bound > maximum_connections {
        return Err(Denial::PolicyMismatch);
    }
    let mut target = None;
    let mut proposal_node = None;
    for (relation, outgoing) in [
        (layout.connection_source_relation, true),
        (layout.connection_target_relation, false),
    ] {
        for relation in incoming(fields, relation, approval_node, bound)? {
            let connection = relation.from;
            if fields.incoming_one(layout.definition_connection_relation, connection)? != definition
            {
                return Err(Denial::Corrupt);
            }
            let decoded: super::CompiledWorkflowConnection = observe_connection(
                fields.execution,
                fields.runtime,
                fields.snapshot(),
                layout,
                connection,
                &mut Vec::new(),
            )
            .map_err(Denial::Execution)?;
            for endpoint in [decoded.source(), decoded.target()] {
                if fields.incoming_one(layout.definition_node_relation, endpoint)? != definition {
                    return Err(Denial::Corrupt);
                }
            }
            match (outgoing, decoded.kind()) {
                (
                    true,
                    CompiledWorkflowConnectionKind::Data(
                        ApplicationWorkflowDataFlow::ApprovalAuthority,
                    ),
                ) => {
                    let id = decoded.target();
                    let candidate = node(fields, layout, definition, id)?;
                    SelectedWorkflowApproval::guarded_target(&candidate)
                        .map_err(Denial::Execution)?;
                    if target.replace(candidate).is_some() {
                        return Err(Denial::Corrupt);
                    }
                }
                (
                    false,
                    CompiledWorkflowConnectionKind::Data(
                        ApplicationWorkflowDataFlow::ProposalSubject,
                    ),
                ) => {
                    let id = decoded.source();
                    let candidate = node(fields, layout, definition, id)?;
                    if SelectedWorkflowProposal::from_node(&candidate).is_none()
                        || proposal_node.replace(id).is_some()
                    {
                        return Err(Denial::Corrupt);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(ApprovalDescriptor {
        approval,
        target: target.ok_or(Denial::Corrupt)?,
        proposal_node: proposal_node.ok_or(Denial::Corrupt)?,
    })
}
fn incoming(fields:&ReadFields<'_, '_, '_>,kind:KindId,anchor:EntityId,bound:usize)->Result<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedRelation>,Denial>{
    let rows = observe_adjacency(
        fields.runtime,
        fields.execution,
        fields.snapshot(),
        kind,
        anchor,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        bound.checked_add(1).ok_or(Denial::Corrupt)?,
    )
    .map_err(Denial::Execution)?
    .ok_or(Denial::BasisUnavailable)?;
    if rows.len() > bound {
        return Err(Denial::Corrupt);
    }
    Ok(rows)
}
