use crate::physical_runtime::{
    PhysicalDurabilityGroupAdmissionDenial, PhysicalWalGroupAppendFailureCause, RecordAppendDenial,
};

/// Live producer detail for an owner-settled pre-seal admission denial.
///
/// This is diagnostic context, not a second mutation fate or retry authority.
/// Persisted and duplicate observations may carry only the broad fate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalMutationPreSealAdmissionDetail {
    RecordPlanning(RecordAppendDenial),
    WalAdmission(PhysicalWalGroupAppendFailureCause),
    GroupAdmission(PhysicalDurabilityGroupAdmissionDenial),
    PublicationReclaimFenced,
}
