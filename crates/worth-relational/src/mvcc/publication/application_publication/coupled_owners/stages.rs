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
        let mut owners = self.readmit_edit_coupled_owners(execution, installed, basis, prefix)?;
        for source in later {
            let stage = self.readmit_edit_coupled_owners(execution, installed, basis, *source)?;
            execution
                .run_native_with_checkpoint(|context| {
                    for owner in stage.owners {
                        checkpoint(context, owners.owners.len() as u64 + 1)
                            .map_err(custody::execution_stop)?;
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
