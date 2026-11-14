//! One edit envelope admits every authored stage's source before any handler runs.
use super::*;

impl RelationalRuntime {
    /// Admit synchronous writers for an edit envelope. `prefix` is the authored
    /// prefix's source; `later` are the sources of stages 2..k, which Native
    /// then accepts once each, in this order. Each source is admitted exactly
    /// as a single edit is; the owner set is their union, with a route current
    /// for two sources admitted once.
    pub fn readmit_edit_coupled_stage_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        prefix: EntityId,
        later: &[EntityId],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        let owners = self.readmit_edit_coupled_owners(execution, installed, basis, prefix)?;
        self.readmit_later_authored_stage_owners(execution, installed, owners, later)
    }

    /// Admit the complete later-source roster without replacing the prefix's
    /// actual Edit/Retirement/Restoration custody. Every source uses the same C.
    pub fn readmit_later_authored_stage_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        mut owners: RelationalAuthoredCoupledOwners,
        later: &[EntityId],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id()
            || owners.before.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || !owners.before.is_current()
            || !owners.stages.is_empty()
        {
            return Err(origin_denial(
                "later authored stages have a different admission basis",
            ));
        }
        for source in later {
            let stage =
                self.readmit_edit_coupled_owners(execution, installed, &owners.before, *source)?;
            execution
                .run_native_with_checkpoint(|context| {
                    let selected = if matches!(owners.source, CoupledSource::Edit(_)) {
                        None
                    } else {
                        Some(
                            crate::history::data::source_scope_closure::later_source_owners(
                                *source,
                                &stage.current,
                                context,
                            )
                            .map_err(custody::cold_stop)?,
                        )
                    };
                    for owner in stage.owners {
                        if let Some(selected) = &selected {
                            checkpoint(context, selected.len() as u64 + 1)
                                .map_err(custody::execution_stop)?;
                            if !selected
                                .iter()
                                .any(|known| known.origin.same_publication(&owner.origin))
                            {
                                continue;
                            }
                        }
                        checkpoint(
                            context,
                            (owners.owners.len() + owners.retirements.len()) as u64 + 1,
                        )
                        .map_err(custody::execution_stop)?;
                        // A later source's current closure can also see a route
                        // genuinely retired by this prefix. Its actual retirement
                        // custody must not become a required surviving writer.
                        if owners
                            .retirements
                            .iter()
                            .any(|retired| retired.origin.same_publication(&owner.origin))
                        {
                            continue;
                        }
                        if !owners
                            .owners
                            .iter()
                            .any(|known| known.origin.same_publication(&owner.origin))
                        {
                            owners
                                .deferred_edges
                                .extend(owner.origin.relations.iter().map(|edge| edge.id));
                            owners.owners.push(owner);
                        }
                    }
                    for current in stage.current {
                        checkpoint(context, owners.current.len() as u64 + 1)
                            .map_err(custody::execution_stop)?;
                        if !owners
                            .current
                            .iter()
                            .any(|known| known.origin.same_publication(&current.origin))
                        {
                            owners.current.push(current);
                        }
                    }
                    Ok::<_, TransactionCommitError>(())
                })
                .map_err(custody::execution_stop)?
                .0?;
            owners.stages.push(*source);
        }
        Ok(owners)
    }
}
