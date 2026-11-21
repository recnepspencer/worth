pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;

#[derive(Debug)]
pub enum WorthQuerySelectedProgramOwnerDenial {
    ProductSelection(
        crate::domain_computation::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    InstalledOwnerUnavailable,
    UndeclaredOutputRoot,
}

/// A retirement's inventory denial or the exact request that stopped reading it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramSupportRetirementFailure {
    Durability(super::query_obligations::QueryObligationDenial),
    Policy(worth_query_installation::facade::WorthQueryProgramSupportRetirementDenial),
    ReadExecution {
        stop: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
        inventory:
            worth_query_installation::facade::WorthQueryProgramSupportPartialRetirementInventory,
    },
}

impl From<worth_query_installation::facade::WorthQueryProgramSupportRetirementDenial>
    for WorthQueryProgramSupportRetirementFailure
{
    fn from(
        denial: worth_query_installation::facade::WorthQueryProgramSupportRetirementDenial,
    ) -> Self {
        Self::Policy(denial)
    }
}
