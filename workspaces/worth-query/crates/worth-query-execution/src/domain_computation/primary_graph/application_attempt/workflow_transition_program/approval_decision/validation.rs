use super::*;

pub(super) fn validate_request_binding<Schema, Capability, Operation, Input, Scope, Spec, Program>(
    read_set: &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
    instance: &super::super::super::PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &super::super::super::PublishedWorkflowProposalRef,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    let identity = installed.approval_capability_identity_bytes::<Capability, Operation>();
    if instance.branch() != read_set.lease.product().product_branch()
        || !installed.approval_binding_matches::<Capability, Operation>()
        || read_set.admission.installed_capability_identity() != identity.copied()
        || Operation::IDENTIFIER != required.operation()
        || hex(identity.copied().unwrap_or_default()) != required.installed_capability_identity()
        || required.instance() != instance.entity_id()
        || proposal.branch() != instance.branch()
        || proposal.instance_entity_id() != instance.entity_id()
        || proposal.definition_entity_id() != instance.definition_entity_id()
    {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
            required.node_path(),
        ));
    }
    Ok(())
}

pub(super) fn validate_requirement_definition(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    required: &RequiredWorkflowApproval,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryApplicationAttemptDenial> {
    let mut matches = compiled
        .nodes()
        .filter(|node| node.path() == required.node_path());
    let (Some(node), None) = (matches.next(), matches.next()) else {
        return Err(affinity("approval requirement node is not singular"));
    };
    let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Approval {
        capability,
        capability_type,
        operation,
        installed_capability_identity,
    } = node.kind()
    else {
        return Err(affinity("approval requirement no longer names an approval node"));
    };
    let target = unique(
        compiled.approval_authority_targets(node.entity()),
        "authority target",
    )?;
    let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Operation {
        operation: target_operation,
        requires_workflow_authority: true,
        ..
    } = target.kind()
    else {
        return Err(affinity("approval authority target is not guarded"));
    };
    if capability != required.capability()
        || capability_type != required.capability_type()
        || operation != required.operation()
        || installed_capability_identity != required.installed_capability_identity()
        || target_operation != required.target_operation()
    {
        return Err(affinity("approval requirement differs from its definition"));
    }
    Ok(node.entity())
}

pub(super) fn unique<'a>(
    mut nodes: impl Iterator<Item = &'a crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNode>,
    kind: &str,
) -> Result<
    &'a crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNode,
    WorthQueryApplicationAttemptDenial,
> {
    match (nodes.next(), nodes.next()) {
        (Some(node), None) => Ok(node),
        _ => Err(affinity(format!("approval {kind} input is not singular"))),
    }
}

pub(super) fn validate_passing_evidence(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    join: worth_relational::facade::identity::EntityId,
    transitions: &[super::super::super::workflow_instance_observation::ObservedWorkflowTransition],
) -> Result<Vec<worth_relational::facade::identity::EntityId>, WorthQueryApplicationAttemptDenial> {
    let required = compiled.required_assessments(join).collect::<Vec<_>>();
    if required.len() < 2 {
        return Err(affinity("approval evidence inventory is incomplete"));
    }
    let mut evidence_entities = Vec::new();
    for node in required {
        let Some(evidence) =
            super::super::super::workflow_instance_observation::latest_assessment_evidence(
                transitions,
                node.entity(),
            )
        else {
            return Err(affinity("approval required evidence is absent"));
        };
        if !evidence.passing {
            return Err(affinity("approval required evidence is failing"));
        }
        evidence_entities.push(evidence.entity);
    }
    Ok(evidence_entities)
}

pub(super) fn affinity(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

pub(super) fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}
