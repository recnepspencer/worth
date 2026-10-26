use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationToken, RuntimeWorldServiceDenial,
};

use super::{activation::WorthQueryProductActivationDenial, WorthQueryProductRuntime};
use crate::basis::WorthQueryProductBranchLease;

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationDenial {
    /// Atomic failed-reservation evidence for this runtime's activation gates.
    /// Counts the root, each installed branch, in-flight reservations, and
    /// retired occurrences while an operation still retains their gate.
    /// Pending owner cleanup alone holds no gate and is excluded after the
    /// last gate holder drops, even if component cleanup is still outstanding.
    /// Multiple holders of one gate count once. Unpublished World forks whose
    /// Query reservation was dropped are excluded. The refused fork is excluded.
    /// With no installed policy, the effective limit is usize::MAX.
    CoordinationCapacityExhausted {
        limit: usize,
        occupancy: usize,
    },
    CoordinationOccupancyOverflow,
    CoordinationAllocationRejected,
    CoordinationUnavailable,
    ProgramSupportUnavailable,
    World(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
}

impl WorthQueryProductRuntime {
    /// Reserve activation capacity before issuing a public destination name.
    pub(crate) fn reserve_product_branch_creation<'runtime>(
        &'runtime self,
        source: &'runtime WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
    ) -> Result<
        WorthQueryProductBranchCreationReservation<'runtime>,
        WorthQueryProductBranchCreationDenial,
    > {
        if source.observation().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchCreationDenial::World(
                RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchAdmissionDenial::ForeignOwner),
            ));
        }
        let reservation = self
            .activations
            .reserve_for_source_program(source_program)
            .map_err(|denial| match denial {
                WorthQueryProductActivationDenial::CapacityExhausted { limit, occupancy } => {
                    WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted {
                        limit,
                        occupancy,
                    }
                }
                WorthQueryProductActivationDenial::ProgramSupportUnavailable => {
                    WorthQueryProductBranchCreationDenial::ProgramSupportUnavailable
                }
                WorthQueryProductActivationDenial::OccupancyOverflow => {
                    WorthQueryProductBranchCreationDenial::CoordinationOccupancyOverflow
                }
                WorthQueryProductActivationDenial::AllocationRejected => {
                    WorthQueryProductBranchCreationDenial::CoordinationAllocationRejected
                }
                WorthQueryProductActivationDenial::UnknownProductBranch
                | WorthQueryProductActivationDenial::RegistryUnavailable
                | WorthQueryProductActivationDenial::GateUnavailable
                | WorthQueryProductActivationDenial::PublicationInProgress => {
                    WorthQueryProductBranchCreationDenial::CoordinationUnavailable
                }
            })?;
        Ok(WorthQueryProductBranchCreationReservation {
            runtime: self,
            source,
            activation: reservation,
        })
    }

    #[cfg(test)]
    pub(crate) fn create_product_branch(
        &self,
        source: &WorthQueryProductBranchLease,
        source_program: Option<&ApplicationProgramRevision>,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        self.reserve_product_branch_creation(source, source_program)?
            .create(intent, cancellation)
    }
}

/// Live activation custody admitted before destination construction or World effects.
pub(crate) struct WorthQueryProductBranchCreationReservation<'runtime> {
    runtime: &'runtime WorthQueryProductRuntime,
    source: &'runtime WorthQueryProductBranchLease,
    activation: super::activation::WorthQueryProductActivationReservation<'runtime>,
}

impl WorthQueryProductBranchCreationReservation<'_> {
    pub(crate) fn create(
        self,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        let outcome = self
            .runtime
            .owner
            .branch_port()
            .create_product_branch(self.source.observation().clone(), intent, cancellation)
            .map_err(WorthQueryProductBranchCreationDenial::World)?;
        if let RuntimeWorldBranchCreationOutcome::Performed(observation) = &outcome {
            self.activation.commit(observation);
        }
        Ok(outcome)
    }
}
