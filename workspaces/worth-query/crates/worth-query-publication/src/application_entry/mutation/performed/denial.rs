#[derive(Debug)]
pub enum WorthQueryPerformedMutationExecutionDenial {
    ForeignProgram,
    MissingConnection,
    Connection(
        worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputConnectionDenial,
    ),
    Mutation(crate::application_entry::WorthQueryApplicationRequestMutationDenial),
}

#[derive(Debug)]
pub enum WorthQueryRequiredOutputPreparationDenial {
    ForeignProgram,
    MissingConnection,
    MissingPerformedDelivery,
    MissingSource,
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
