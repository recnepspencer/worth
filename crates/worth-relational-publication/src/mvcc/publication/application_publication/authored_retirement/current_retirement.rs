//! Live opted-in retirement custody comes from current facts before legacy history.
use super::*;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
impl AuthoredPublicationReads<'_> {
    pub(super) fn current_retirement_custody(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<
        Option<Arc<branch::publication::ownership::read::roster::CurrentPublicationRoster>>,
        TransactionCommitError,
    > {
        if installed.source_scope_ownership.is_empty() {
            return Ok(None);
        }
        let records = roles
            .iter()
            .filter_map(|r| match r.target {
                EntityReference::Existing(id) => {
                    Some(contracts::transactions::data::primitives::RecordRef::Entity(id))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        crate::publication::ownership::select_current(execution, basis, &records).map(Some)
    }

    pub(super) fn refuse_live_owned_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        roles: &[RequiredOutputRoleTarget],
        current: Option<&branch::publication::ownership::read::roster::CurrentPublicationRoster>,
    ) -> Result<(), TransactionCommitError> {
        let Some(current) = current else {
            return Ok(());
        };
        let selected = execution
            .run_native_with_checkpoint(|context| {
                branch::history::data::codec_retirement_closure::scope_dependents::select_owned_scope_dependent(
                    branch::history::data::codec_retirement_closure::scope_dependents::OwnedScopeDependentSelection::LiveRecord(
                        &mut |origin, context| {
                            for role in roles {
                                contracts::execution::request_stage::checkpoint(
                                    context,
                                    origin.members.len() as u64 + 1,
                                )
                                .map_err(|cause| {
                                    branch::history::data::codec_retirement_closure::CodecRetirementClosureStop::Request(
                                        cause.into(),
                                    )
                                })?;
                                if role.posture == RequiredOutputRolePosture::Retire
                                    && origin.members.iter().any(|member| {
                                        member.posture != RequiredOutputRolePosture::Retire
                                            && role.target == EntityReference::Existing(member.id)
                                    })
                                {
                                    return Ok(true);
                                }
                            }
                            Ok(false)
                        },
                    ),
                    current.origins(),
                    context,
                )
            })
            .map_err(custody::execution_stop)?
            .0
            .map_err(|failure| custody::cold_stop(failure.stop))?;
        if let Some(origin) = selected {
            let subject = origin
                .outputs
                .first()
                .copied()
                .ok_or_else(|| origin_denial("live publication has no owned output"))?;
            return Err(TransactionCommitError::publication(contracts::publication::data::publication_error::PublicationError::owned_publication(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade{
                    publication:origin.commit.commit_id,owner:origin.meaning.source_scope,
                },subject,origin.meaning.source_scope)));
        }
        Ok(())
    }
}
