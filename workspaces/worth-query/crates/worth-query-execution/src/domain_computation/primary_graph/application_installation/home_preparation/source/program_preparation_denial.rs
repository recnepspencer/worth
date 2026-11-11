//! The original owner cause remains typed across programme preparation refusal.
#[derive(Debug)]
pub enum QueryHomeSignalProgramPreparationDenial {
    Source(super::WorthQueryPrimaryGraphInstallationDenial),
    Producer(super::WorthQueryPrimaryGraphInstallationDenial),
    PrimaryGraph(super::WorthQueryPrimaryGraphInstallationDenial),
    Conditional(Box<crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial>),
    Bridge(Box<worth_runtime_bridge::facade::BridgeConditionalDenial>),
    Authored(Box<worth_runtime_bridge::facade::BridgeAuthoredReadmissionDenial>),
    TemporalFamilyNotReadmitted,
}
