#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryProductActivationDenial {
    CapacityExhausted { limit: usize, occupancy: usize },
    OccupancyOverflow,
    AllocationRejected,
    UnknownProductBranch,
    RegistryUnavailable,
    GateUnavailable,
    PublicationInProgress,
    ProgramSupportUnavailable,
}
