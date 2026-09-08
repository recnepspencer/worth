mod checkpoint_scope;
mod contract;
mod denial;
mod effect_contract;
mod intent;
mod resource_demand;
mod root_publication_scope;
mod scope;
mod wal_append_scope;
mod wal_barrier_scope;
mod wal_reclamation_scope;

pub(in crate::physical_runtime) use checkpoint_scope::{
    PhysicalCheckpointWorkAction, PhysicalCheckpointWorkScope,
};
pub use contract::{
    PhysicalWorkDurabilityRequirement, PhysicalWorkEffectClass, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition,
};
pub use denial::PhysicalWorkDeclarationDenial;
pub use intent::PhysicalWorkIntent;
pub(in crate::physical_runtime) use intent::PhysicalWorkIntentParts;
pub(in crate::physical_runtime) use resource_demand::PhysicalWorkResourceDemand;
pub(in crate::physical_runtime) use root_publication_scope::{
    PhysicalRootPublicationWorkAction, PhysicalRootPublicationWorkScope,
};
pub use scope::PhysicalWorkScope;
pub use wal_append_scope::{PhysicalWalAppendScope, PhysicalWalFrameWriteDisposition};
pub use wal_barrier_scope::PhysicalWalBarrierScope;
pub(in crate::physical_runtime) use wal_reclamation_scope::PhysicalWalReclamationScope;

mod inspection_digest;
#[cfg(test)]
mod tests;
