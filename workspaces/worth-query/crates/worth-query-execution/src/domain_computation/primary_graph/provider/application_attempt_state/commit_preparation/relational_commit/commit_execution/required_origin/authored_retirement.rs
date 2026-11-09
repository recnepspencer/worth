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
    let anchor = crate::domain_computation::primary_graph::provider::authored_retirement_selection::authored_anchor(attempt).map_err(native_output_witness_stop)?;
    let all_retire = targets
        .clone()
        .all(|(_, posture, _, _)| posture == Posture::Retire);
    if targets.len() == 0
        || (anchor.is_none()
            && !targets
                .clone()
                .any(|(_, posture, _, _)| posture == Posture::Retire))
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
    let (roles, _) =
        crate::domain_computation::primary_graph::provider::native_publication_roles::collect(
            &provider.graph.layout,
            targets,
        )
        .ok_or_else(mismatch)?;
    if let Some((anchor, co_anchors)) = anchor {
        let retiring = roles
            .iter()
            .filter(|role| role.posture == RequiredOutputRolePosture::Retire)
            .cloned()
            .collect::<Vec<_>>();
        let cascade = provider
            .graph
            .with_runtime(execution, |runtime| {
                runtime.readmit_authored_source_retirement_with_co_anchors(
                    execution,
                    &publisher.native,
                    basis,
                    anchor,
                    &co_anchors,
                    &retiring,
                )
            })
            .map_err(transaction_commit_stop)?;
        if let Some(predecessor) = cascade {
            let admission = publisher
                .owner
                .admit_authored_output_retirements(predecessor.authorities())
                .ok_or_else(mismatch)?;
            return proposal
                .bind_required_output_publication(
                    &publisher.native,
                    RequiredOutputPublication::AuthoredCascadeRetirement {
                        admission,
                        predecessor,
                    },
                )
                .map_err(transaction_commit_stop);
        }
    }
    // A mixed roster cannot bypass an opted-in cascade. If Native found no
    // cascade, the independent legacy path retains its original posture rule.
    if !all_retire {
        let retiring = roles
            .into_iter()
            .filter(|role| role.posture == RequiredOutputRolePosture::Retire)
            .collect::<Vec<_>>();
        if !retiring.is_empty() {
            provider
                .graph
                .with_runtime(execution, |runtime| {
                    runtime.refuse_owned_output_retirement(
                        execution,
                        &publisher.native,
                        basis,
                        &retiring,
                    )
                })
                .map_err(transaction_commit_stop)?;
        }
        return Ok(proposal);
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
