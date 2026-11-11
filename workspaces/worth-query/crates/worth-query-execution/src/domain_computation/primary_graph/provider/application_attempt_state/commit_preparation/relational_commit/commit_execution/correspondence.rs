//! Canonical output correspondence is admitted before Native's World effect.
use super::*;
pub(super) fn admit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &mut WorthQueryPrimaryGraphApplicationAttempt,
    members: &mut [crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    candidate: &worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
    prepared_group_publications: Option<
        &worth_relational::facade::mvcc::RelationalPreparedOutputPublications,
    >,
) -> Result<(), crate::domain_computation::WorthQueryProviderSessionCommitStop> {
    let correspondence_denial = |(cause, role)| {
        crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
            WorthQueryProviderSessionFailure::new(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::OutputCorrespondenceMismatch { cause },
                WorthQueryProviderSessionProtocolStage::Commit,
                format!("canonical output correspondence differs at role {role}"),Default::default(),
            )
        )
    };
    if let Some(prepared) = prepared_group_publications {
        for member in members {
            if !member
                .owned
                .as_ref()
                .expect("actual participant")
                .commit
                .as_ref()
                .expect("prepared participant")
                .publication_required
            {
                // Native allocation correspondence was admitted from the real
                // ordinary part before candidate preparation, with no origin.
                continue;
            }
            let crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::Producer { producer, meaning } = &member.kind else { continue };
            member
                .owned
                .as_mut()
                .expect("actual prepared coupled writer")
                .attempt
                .admit_owned_output_correspondence(prepared, producer, meaning)
                .map_err(correspondence_denial)?;
        }
    } else if let Some(publisher) = &provider.graph.application_publication {
        let publications = candidate
            .prepare_required_output_publications(execution, &publisher.native)
            .map_err(transaction_commit_stop)?;
        attempt
            .admit_output_correspondence(&publications)
            .map_err(correspondence_denial)?;
    }
    Ok(())
}
