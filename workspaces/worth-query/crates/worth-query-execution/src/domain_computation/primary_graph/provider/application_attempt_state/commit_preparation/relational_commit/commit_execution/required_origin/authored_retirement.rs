//! Ordinary admitted Retire correspondence succeeds Native's complete selected publication.
use super::*;

pub(super) fn bind(
    proposal: ValidatedRelationalProposal,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<
    ValidatedRelationalProposal,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let Some(publisher) = &provider.graph.application_publication else {
        return Ok(proposal);
    };
    let targets = attempt.native_publication_roles();
    if targets.len() == 0
        || targets
            .clone()
            .any(|(_, posture, _, _)| posture != Posture::Retire)
    {
        return Ok(proposal);
    }
    let mismatch = || native_output_witness_stop(CompanionPreflightStop::SelectedSourceMismatch);
    let basis = attempt
        .affinity()
        .product_publication()
        .observation()
        .basis()
        .relational_basis();
    provider.graph.with_runtime(execution, |runtime| {
        let positioned = runtime
            .read_truth()
            .positioned_snapshot(before)
            .map_err(|_| mismatch())?;
        if positioned.root_id() != basis.descriptor().root_identity()
            || positioned.runtime_instance_id() != basis.descriptor().runtime_instance_id()
            || positioned.branch_id() != basis.descriptor().branch_id()
        {
            return Err(mismatch());
        }
        Ok(())
    })?;
    let mut roles = Vec::with_capacity(targets.len());
    for (role, _, name, target) in targets {
        roles.push(RequiredOutputRoleTarget {
            role: role.to_owned(),
            entity_name: name.to_owned(),
            kind: provider
                .graph
                .layout
                .entity_kind(name)
                .ok_or_else(mismatch)?,
            posture: RequiredOutputRolePosture::Retire,
            target: target.clone(),
        });
    }
    let previous = provider
        .graph
        .with_runtime(execution, |runtime| {
            runtime.readmit_authored_output_retirement(execution, &publisher.native, basis, &roles)
        })
        .map_err(transaction_commit_stop)?;
    let Some(predecessor) = previous else {
        return Ok(proposal);
    };
    let admission = publisher
        .owner
        .admit_authored_output_retirement(predecessor.authority())
        .ok_or_else(mismatch)?;
    proposal
        .bind_required_output_publication(
            &publisher.native,
            RequiredOutputPublication::AuthoredRetirement {
                admission,
                predecessor,
                roles,
            },
        )
        .map_err(transaction_commit_stop)
}
