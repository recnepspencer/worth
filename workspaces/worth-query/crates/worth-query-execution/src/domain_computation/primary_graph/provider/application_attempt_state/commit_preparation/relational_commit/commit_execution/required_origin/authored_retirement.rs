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
    let anchor = authored_anchor(attempt)?;
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
    let mut roles = Vec::with_capacity(targets.len());
    for (role, posture, name, target) in targets {
        roles.push(RequiredOutputRoleTarget {
            role: role.to_owned(),
            entity_name: name.to_owned(),
            kind: provider
                .graph
                .layout
                .entity_kind(name)
                .ok_or_else(mismatch)?,
            posture: match posture {
                Posture::Create => RequiredOutputRolePosture::Create,
                Posture::Preserve => RequiredOutputRolePosture::Preserve,
                Posture::Retire => RequiredOutputRolePosture::Retire,
            },
            target: target.clone(),
        });
    }
    if let Some(anchor) = anchor {
        let cascade = provider
            .graph
            .with_runtime(execution, |runtime| {
                runtime.readmit_authored_source_retirement(
                    execution,
                    &publisher.native,
                    basis,
                    anchor,
                    &roles,
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

/// The installed correction declaration chooses the role. Roster order and
/// reference reachability never choose an authored source.
fn authored_anchor(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<
    Option<worth_relational::facade::identity::EntityId>,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    if let Some(anchor) = attempt
        .aftermath_causality()
        .and_then(|causality| causality.prepared_correction().authored_retirement_anchor())
    {
        return Ok(Some(anchor));
    }
    let identity = attempt
        .original_authority()
        .retained_governed_input_carrier()
        .and_then(|input| input.correction())
        .and_then(|original| original.association().declaration().authored_identity());
    let Some(identity) = identity else {
        return Ok(None);
    };
    let parts = identity.parts();
    if parts.source_posture != Posture::Retire {
        return Ok(None);
    }
    let mut matches = attempt
        .native_publication_roles()
        .filter(|(role, posture, _, _)| *role == parts.source_role && *posture == Posture::Retire);
    let target = matches.next().and_then(|(_, _, _, target)| match target {
        EntityReference::Existing(id) => Some(*id),
        _ => None,
    });
    if target.is_none() || matches.next().is_some() {
        return Err(native_output_witness_stop(
            CompanionPreflightStop::SelectedSourceMismatch,
        ));
    }
    Ok(target)
}
