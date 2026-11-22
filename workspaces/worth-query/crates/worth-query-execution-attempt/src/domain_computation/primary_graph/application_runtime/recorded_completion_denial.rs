#[derive(Debug)]
pub enum WorthQueryRecordedCompletionDenial {
    World(worth_runtime_world::facade::RuntimeWorldRecoveryDenial),
    Pending,
    Refused(worth_runtime_world::facade::WorldRecoveredContinuationDenial),
    EvidenceRequired,
    Indeterminate,
    Source(
        crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionReadDenial,
    ),
    Terminal(
        crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial,
    ),
    PublicationMismatch,
}
