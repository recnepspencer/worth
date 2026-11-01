/// Why a mutation with required program outputs was refused before its commit.
#[derive(Debug)]
pub enum WorthQueryPerformedMutationExecutionDenial {
    Action(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    ForeignProgram,
    UndeclaredOutputRoot,
    Connection(
        worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputConnectionDenial,
    ),
    Mutation(crate::application_entry::WorthQueryApplicationRequestMutationDenial),
}

/// Why a landed mutation's required outputs could not start. `recovery_posture` says
/// whether a fresh request can retry.
#[derive(Debug)]
pub enum WorthQueryRequiredOutputPreparationDenial {
    ForeignProgram,
    UndeclaredOutputRoot,
    MissingPerformedDelivery,
    MissingSource,
    /// Complete original discovery could not be retained or combined.
    Dependencies(worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial),
    SourceQuery(crate::application_entry::WorthQueryApplicationRequestQueryDenial),
    ReadObservation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    DemandExecution(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial),
    Connection(
        worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputConnectionDenial,
    ),
    Closed,
}

mod recovery_posture;
pub use recovery_posture::WorthQueryRequiredOutputRecoveryPosture;

impl std::fmt::Display for WorthQueryPerformedMutationExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "performed mutation denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryPerformedMutationExecutionDenial {}

impl std::fmt::Display for WorthQueryRequiredOutputPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "required output preparation denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryRequiredOutputPreparationDenial {}

impl WorthQueryRequiredOutputPreparationDenial {
    pub(in crate::application_entry) fn advancement(
        cause: worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::Demand(
            crate::application_entry::WorthQueryApplicationOutputDemandDenial::advancement(cause),
        )
    }
}

impl WorthQueryRequiredOutputPreparationDenial {
    fn native_demand_mut(
        &mut self,
    ) -> Option<&mut worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial>
    {
        use crate::application_entry::{
            WorthQueryApplicationOutputDemandDenial as Demand,
            WorthQueryApplicationRequestQueryDenial as Query,
        };
        use worth_query_execution::publication_boundary::document_history::{
            WorthQueryHistoryOutputAdvanceDenial as Advance,
            WorthQueryHistoryOutputReadDenial as Read,
        };
        match self {
            Self::DemandExecution(cause)
            | Self::Demand(Demand::Demand(cause))
            | Self::Demand(Demand::History(Advance::Demand(cause)))
            | Self::Demand(Demand::HistoryRead(Read::Demand(cause)))
            | Self::SourceQuery(Query::HistoryOutput(Read::Demand(cause)))
            | Self::Demand(Demand::Source(Query::HistoryOutput(Read::Demand(cause)))) => {
                Some(cause)
            }
            _ => None,
        }
    }

    pub(in crate::application_entry) fn take_managed_cleanup(
        &mut self,
    ) -> Option<
        Box<worth_query_execution::facade::primary_graph::WorthQueryApplicationCleanupRequired>,
    > {
        self.native_demand_mut()
            .and_then(|cause| cause.take_managed_cleanup())
    }

    pub(in crate::application_entry) fn take_read_cleanup(
        &mut self,
    ) -> Option<worth_query_execution::facade::primary_graph::WorthQueryOutputReadCleanup> {
        self.native_demand_mut()
            .and_then(|cause| cause.take_read_cleanup())
    }
}
