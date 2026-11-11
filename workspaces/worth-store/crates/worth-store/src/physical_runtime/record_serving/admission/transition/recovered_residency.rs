//! A recovered seal moves the original pool; ordinary opens admit a new owner.

use super::super::recovered_custody::RecoveredCheckpointCustodyEvidence;
use crate::physical_runtime::{
    instance::PhysicalResidencyOwner,
    record_serving::{AdmittedPhysicalRecordResidencyPolicy, RecordBootstrapDenial},
    RecoveredPhysicalCheckpointCustody,
};

pub(super) fn admit(
    custody: Option<RecoveredPhysicalCheckpointCustody>,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) -> Result<
    (
        PhysicalResidencyOwner,
        Option<RecoveredCheckpointCustodyEvidence>,
    ),
    RecordBootstrapDenial,
> {
    match custody {
        #[cfg(feature = "recovery-runtime-owner")]
        Some(seal) => {
            let (owner, evidence) = seal.into_serving_parts();
            owner.validate_recovered_policy(store, policy)?;
            Ok((owner, Some(evidence)))
        }
        #[cfg(not(feature = "recovery-runtime-owner"))]
        Some(seal) => seal.without_recovery_owner(),
        None => PhysicalResidencyOwner::admit(store, policy)
            .map(|owner| (owner, None))
            .map_err(RecordBootstrapDenial::from_residency),
    }
}
