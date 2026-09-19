#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryProductActivationDenial {
    CapacityExhausted,
    AllocationRejected,
    UnknownProductBranch,
    RegistryUnavailable,
    GateUnavailable,
    PublicationInProgress,
}
