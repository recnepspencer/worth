#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductActivationDenial {
    CapacityExhausted,
    AllocationRejected,
    UnknownProductBranch,
    RegistryUnavailable,
    GateUnavailable,
    PublicationInProgress,
    ProgramSupportUnavailable,
}
