use worth_store_physical_format::physical_work_obligation::{
    physical_work_obligation_v6_scope_digest, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::PhysicalWorkObligationIdentity;

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn physical_work_obligation(
        store: StableStoreIdentity,
        identity: PhysicalWorkObligationIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::PhysicalWorkObligation(identity),
            range,
        )
    }

    pub const fn physical_work_obligation_identity(self) -> Option<PhysicalWorkObligationIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::PhysicalWorkObligation(identity) => Some(identity),
            _ => None,
        }
    }

    pub(crate) fn physical_work_exact_scope_digest(self) -> Option<[u8; 32]> {
        let identity = self.physical_work_obligation_identity()?;
        let mut preimage = [0_u8; 65];
        preimage[..8].copy_from_slice(b"C9PWOSCP");
        preimage[8..24].copy_from_slice(&self.store.bytes());
        preimage[24] = PHYSICAL_WORK_OBLIGATION_V6_VERSION;
        preimage[25..33].copy_from_slice(&identity.runtime().get().to_le_bytes());
        preimage[33..41].copy_from_slice(&identity.generation().get().to_le_bytes());
        preimage[41..49].copy_from_slice(&identity.operation().get().to_le_bytes());
        preimage[49..57].copy_from_slice(&self.range.offset().to_le_bytes());
        preimage[57..65].copy_from_slice(&self.range.length().to_le_bytes());
        Some(physical_work_obligation_v6_scope_digest(&preimage))
    }
}
