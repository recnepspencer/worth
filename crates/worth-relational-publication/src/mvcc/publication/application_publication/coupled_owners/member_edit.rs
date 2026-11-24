//! A live owner's edit retains member lifetime custody without becoming that member.
use super::*;

impl AuthoredPublicationReads<'_> {
    /// Bind the admitted edit source to an already-issued member closure.
    /// Retirement/restoration evidence and mandatory linked writers stay exact;
    /// new operand routes use the edit's ordinary current-owner admission.
    pub fn readmit_member_edit_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        mut members: RelationalAuthoredCoupledOwners,
        source: EntityId,
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        if !members.stages.is_empty() || members.corrective_root.is_some() {
            return Err(origin_denial(
                "member edit source must precede later stages",
            ));
        }
        let edit =
            self.readmit_edit_coupled_owners(execution, installed, &members.before, source)?;
        let source = match members.source {
            CoupledSource::Retirement {
                anchor,
                birth,
                co_anchors,
                roles,
            } => CoupledSource::MemberEditRetirement {
                source,
                anchor,
                birth,
                co_anchors,
                roles,
            },
            CoupledSource::Restoration { plan, .. } => {
                CoupledSource::MemberEditRestoration { source, plan }
            }
            _ => {
                return Err(origin_denial(
                    "member edit requires genuine member lifecycle custody",
                ))
            }
        };
        execution
            .run_native_with_checkpoint(|context| {
                for current in edit.current {
                    checkpoint(context, members.current.len() as u64 + 1)
                        .map_err(custody::execution_stop)?;
                    if !members
                        .current
                        .iter()
                        .any(|known| known.origin.same_publication(&current.origin))
                    {
                        members.current.push(current);
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0?;
        members.deferred_edges.extend(edit.deferred_edges);
        members.source = source;
        Ok(members)
    }
}
