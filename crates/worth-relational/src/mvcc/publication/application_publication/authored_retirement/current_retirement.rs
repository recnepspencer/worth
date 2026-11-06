//! Live opted-in retirement custody comes from current facts before legacy history.
use super::*;
impl RelationalRuntime {
    pub(super) fn current_retirement_custody(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<
        Option<Arc<crate::publication::ownership::CurrentPublicationRoster>>,
        TransactionCommitError,
    > {
        if installed.source_scope_ownership.is_empty() {
            return Ok(None);
        }
        let records = roles
            .iter()
            .filter_map(|r| match r.target {
                EntityReference::Existing(id) => {
                    Some(crate::transactions::data::RecordRef::Entity(id))
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
        current: Option<&crate::publication::ownership::CurrentPublicationRoster>,
    ) -> Result<(), TransactionCommitError> {
        let Some(current) = current else {
            return Ok(());
        };
        let selected = execution
            .run_native_with_checkpoint(|context| {
                crate::history::data::select_owned_scope_dependent(
                    crate::history::data::OwnedScopeDependentSelection::LiveRecord(
                        &mut |origin, context| {
                            for role in roles {
                                crate::execution::request_stage::checkpoint(
                                    context,
                                    origin.members.len() as u64 + 1,
                                )
                                .map_err(|cause| {
                                    crate::history::data::CodecRetirementClosureStop::Request(
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
                    current,
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
            return Err(TransactionCommitError::publication(crate::publication::data::PublicationError::owned_publication(
                crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade{
                    publication:origin.commit.commit_id,owner:origin.meaning.source_scope,
                },subject,origin.meaning.source_scope)));
        }
        Ok(())
    }
}
