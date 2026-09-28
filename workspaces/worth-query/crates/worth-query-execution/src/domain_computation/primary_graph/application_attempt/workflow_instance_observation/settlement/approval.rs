use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

use super::{
    adjacency_with_kind, denial, exact, observed_text, observed_u64,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationAttemptDenial,
    workflow::schema::WorthQueryWorkflowLayout,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_retained_approval_binding(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    graph_layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    transition: EntityId,
    expected_proposal: EntityId,
    expected_evidence: &[EntityId],
    target_operation: &str,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<
    crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
    WorthQueryApplicationAttemptDenial,
> {
    let approvals = adjacency_with_kind(
        runtime,
        snapshot,
        layout.transition_approval_relation,
        transition,
        2,
        "workflow transition approval relation is unavailable",
        facts,
    )?;
    let [approval] = approvals.as_slice() else {
        return Err(denial("workflow transition approval is not singular"));
    };
    let kind = layout.approval.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: *approval,
        kind,
    });
    exact(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        facts,
    )?;
    exact(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.decision,
        super::text("approve".to_owned()),
        facts,
    )?;
    exact(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.target_operation,
        super::text(target_operation.to_owned()),
        facts,
    )?;
    let request = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_request,
        facts,
    )?;
    let request = crate::domain_computation::authorization::WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&request)
        .map_err(|()| denial("workflow approval authorization request is invalid"))?;
    let principal = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_principal,
        facts,
    )?;
    let principal: crate::domain_computation::primary_graph::WorthQueryDurablePrincipalCurrentness =
        serde_json::from_str(&principal)
            .map_err(|_| denial("workflow approval principal evidence is invalid"))?;
    if !principal.is_portable() {
        return Err(denial(
            "workflow approval principal evidence is nonportable",
        ));
    }
    if !principal.remains_current_in(runtime, snapshot, graph_layout) {
        return Err(WorthQueryApplicationAttemptDenial::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::WorkflowApprovalPrincipalStale,
            "workflow approval principal evidence is stale",
        ));
    }
    let capability = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.installed_capability_identity,
        facts,
    )?;
    let request_capability = request
        .capability_identity()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let approver = observed_entity(
        runtime,
        snapshot,
        *approval,
        kind,
        [
            &layout.approval.approver_partition,
            &layout.approval.approver_slot,
            &layout.approval.approver_generation,
        ],
        facts,
    )?;
    let scope = observed_entity(
        runtime,
        snapshot,
        *approval,
        kind,
        [
            &layout.approval.scope_partition,
            &layout.approval.scope_slot,
            &layout.approval.scope_generation,
        ],
        facts,
    )?;
    let action = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.action,
        facts,
    )?;
    let purpose = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.purpose,
        facts,
    )?;
    if request_capability != capability
        || request.principal() != approver
        || principal.principal() != approver
        || request.resource() != scope
        || action != format!("{:?}", request.action())
        || purpose != format!("{:?}", request.purpose())
    {
        return Err(denial(
            "workflow approval authorization request conflicts with approval",
        ));
    }
    let grant = observed_entity(
        runtime,
        snapshot,
        *approval,
        kind,
        [
            &layout.approval.grant_partition,
            &layout.approval.grant_slot,
            &layout.approval.grant_generation,
        ],
        facts,
    )?;
    // This per-observation identity is audit provenance, not a durable permit:
    // fresh readmission mints a different observation identity.
    let audit_observation_identity = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_decision,
        facts,
    )?;
    if audit_observation_identity.len() != 64
        || !audit_observation_identity
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(denial("workflow approval observation identity is invalid"));
    }
    let policy = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.capability_authority_identity,
        facts,
    )?;
    if policy.is_empty() {
        return Err(denial(
            "workflow approval capability authority identity is absent",
        ));
    }
    let lineage = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_lineage,
        facts,
    )?;
    if lineage.len() > crate::domain_computation::authorization::MAXIMUM_PORTABLE_BYTES {
        return Err(denial(
            "workflow approval capability lineage exceeds its bound",
        ));
    }
    let lineage: crate::domain_computation::authorization::WorthQueryDurableCapabilityLineage =
        serde_json::from_str(&lineage)
            .map_err(|_| denial("workflow approval capability lineage is invalid"))?;
    let support = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_support,
        facts,
    )?;
    let support =
        crate::domain_computation::authorization::decode_workflow_approval_support(&support)
            .map_err(|()| denial("workflow approval capability support is invalid"))?;
    let dependencies = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.authorization_dependencies,
        facts,
    )?;
    let dependencies =
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalDependencies::decode(
            &dependencies,
        )
        .map_err(|()| denial("workflow approval native authority dependencies are invalid"))?;
    if dependencies.support_present() != support.is_some() {
        return Err(denial(
            "workflow approval support dependencies conflict with approval",
        ));
    }
    let timeline = observed_text(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.validity_timeline,
        facts,
    )?;
    let timeline = match timeline.as_str() {
        "unix-epoch-seconds" => worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
        "unix-epoch-milliseconds" => worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
        _ => return Err(denial("workflow approval validity timeline is invalid")),
    };
    let expiry = observed_u64(
        runtime,
        snapshot,
        *approval,
        kind,
        &layout.approval.expiry,
        facts,
    )?;
    let proposals = adjacency_with_kind(
        runtime,
        snapshot,
        layout.approval_proposal_relation,
        *approval,
        2,
        "workflow approval proposal relation is unavailable",
        facts,
    )?;
    if proposals.as_slice() != [expected_proposal] {
        return Err(denial("workflow approval proposal binding changed"));
    }
    let mut evidence = adjacency_with_kind(
        runtime,
        snapshot,
        layout.approval_evidence_relation,
        *approval,
        expected_evidence.len().saturating_mul(2).saturating_add(1),
        "workflow approval evidence relation is unavailable",
        facts,
    )?;
    evidence.sort();
    let mut expected = expected_evidence.to_vec();
    expected.sort();
    if evidence != expected {
        return Err(denial("workflow approval evidence binding changed"));
    }
    Ok(
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis::from_retained_approval(
            request, principal, grant, policy, lineage, support, dependencies, timeline, expiry,
        ),
    )
}

fn observed_entity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locators: [&worth_foundational::facade::AspectFieldLocator; 3],
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let partition = observed_u64(runtime, snapshot, entity, kind, locators[0], facts)?;
    let slot = observed_u64(runtime, snapshot, entity, kind, locators[1], facts)?;
    let generation = observed_u64(runtime, snapshot, entity, kind, locators[2], facts)?;
    Ok(EntityId::new(
        worth_relational::facade::identity::PartitionId(
            u32::try_from(partition)
                .map_err(|_| denial("workflow approval partition is invalid"))?,
        ),
        slot,
        u32::try_from(generation).map_err(|_| denial("workflow approval generation is invalid"))?,
    ))
}
