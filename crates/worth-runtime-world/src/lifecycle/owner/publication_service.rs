use super::RuntimeWorldOwnerRoot;

#[cfg(test)]
use crate::branch::ProductBranchReferenceCell;
#[cfg(test)]
use crate::lifecycle::ports::RuntimeWorldProductPublicationService;
use crate::publication::RuntimeWorldPublicationOutcome;
#[cfg(test)]
use crate::publication::{CompositeLateCancellationPosture, CompositePublicationReady};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn finish_publication(
        &self,
        outcome: crate::publication::OwnerExecutionOutcome,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> RuntimeWorldPublicationOutcome {
        use crate::publication::OwnerExecutionOutcome;
        let settlement = match outcome {
            OwnerExecutionOutcome::NoEffect(value) => {
                return RuntimeWorldPublicationOutcome::NoEffect(value)
            }
            OwnerExecutionOutcome::ProductUnpublished(value) => {
                return RuntimeWorldPublicationOutcome::ProductUnpublished(value)
            }
            OwnerExecutionOutcome::Settled(value) => value,
        };
        let successor = settlement
            .successor_basis()
            .expect("owner settlement carries its admitted successor")
            .clone();
        let ready = match settlement.ready(successor) {
            Ok(ready) => ready,
            Err(record) => return RuntimeWorldPublicationOutcome::ProductUnpublished(record),
        };
        let Some(cell) = self
            .state
            .branches
            .branch_cell(ready.expected_head().branch_identity())
        else {
            return ready.retain(crate::recovery::ProductUnpublishedCause::StaleProductHead);
        };
        ready.publish_controlled(
            &cell,
            cancellation,
            &self.state.clock,
            #[cfg(feature = "test-operation-control")]
            &self.state.operation_control,
        )
    }

    /// Recover the original committed delivery after caller loss. This reads
    /// history and claims delivery; it performs no component work or CAS.
    #[cfg(test)]
    pub(crate) fn recover_performed_publication(
        &self,
        identity: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        Option<crate::publication::PerformedCompositePublication>,
        crate::history::CompositeHistoryCatalogDenial,
    > {
        self.state
            .history
            .claim_performed_publication(identity)
            .map(|claim| claim.map(crate::publication::PerformedCompositePublication::owner_issued))
    }
}

#[cfg(test)]
impl<D, I, E, Ctx, T> RuntimeWorldProductPublicationService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Publish the ready commit into the branch cell.
    ///
    /// The cell is the only authority for the head. Exact reuse resolves its
    /// commit from the observation the cell issued, so a movement has nothing
    /// to report to the registry and there is no window in which a derived
    /// index lags the head the cell already carries.
    fn publish(
        &self,
        ready: CompositePublicationReady,
        cell: &ProductBranchReferenceCell,
        late_cancellation: CompositeLateCancellationPosture,
    ) -> RuntimeWorldPublicationOutcome {
        ready.publish(cell, late_cancellation)
    }
}

#[cfg(test)]
#[path = "publication_service_tests.rs"]
mod tests;
