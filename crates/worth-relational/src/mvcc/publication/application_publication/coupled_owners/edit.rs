//! Ordinary edit custody is selected by the current ownership owner.
use super::*;
use crate::storage::overlay::CurrentPartitionAccess;

impl RelationalRuntime {
    /// Admit synchronous writers for an ordinary authored source. Unlike a
    /// retirement, an edit does not require every current owner to republish:
    /// Query may prove an output current without appending any commands.
    pub fn readmit_edit_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        source: EntityId,
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        let foreign = || {
            TransactionCommitError::publication(PublicationError::authored_retirement(
                Cause::ForeignCoupledOwner,
                RecordRef::Entity(source),
            ))
        };
        if installed.runtime_instance_id != self.runtime_instance_id()
            || basis.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || !basis.is_current()
        {
            return Err(foreign());
        }
        basis
            .inner
            .root
            .demand_head_partition(execution, source.partition_id)
            .map_err(custody::execution_stop)?;
        if !basis
            .inner
            .root
            .get_current_partition(source.partition_id)
            .and_then(|partition| partition.entity_arena().get(&source))
            .is_some_and(|row| row.is_live())
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(
                    Cause::CoupledSourceNotLive,
                    RecordRef::Entity(source),
                ),
            ));
        }
        let current = execution
            .run_native_with_checkpoint(|context| {
                crate::history::data::source_scope_closure::current(
                    execution,
                    basis,
                    Some(installed),
                    &[RecordRef::Entity(source)],
                    context,
                )
            })
            .map_err(custody::execution_stop)?
            .0?;
        let owners = current
            .iter()
            .map(|selected| {
                self.readmit_canonical_publication(execution, installed, basis, &selected.origin)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let deferred_edges = owners
            .iter()
            .flat_map(|owner| owner.origin.relations.iter().map(|edge| edge.id))
            .collect();
        Ok(RelationalAuthoredCoupledOwners {
            before: basis.clone(),
            source: CoupledSource::Edit(source),
            retirements: Vec::new(),
            owners,
            current,
            deferred_edges,
            stages: Vec::new(),
            corrective_root: None,
        })
    }
}
