use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::durability::{
    rebuild_idempotency, reopen_binding_compaction, reopen_wal_inventory,
    PhysicalDurabilityRuntimeOwner, PhysicalWalBindingReopenCutoff, PhysicalWalRuntimeOwner,
    ReopenedPhysicalBindingCompaction, ReopenedPhysicalDurabilityRuntimeOwner,
};
use crate::physical_runtime::{
    PhysicalBindingCompactionReopenFailure, PhysicalIdempotencyReopenFailure,
    PhysicalSignalProfileIdentity, PhysicalWalOpenFailure, RuntimeIdentity,
};

pub(in crate::physical_runtime) struct PhysicalDurabilityReopenBasis {
    rebuilt: crate::physical_runtime::durability::RebuiltPhysicalMutationIdempotency,
    wal: PhysicalWalRuntimeOwner,
    unresolved_retirements: Vec<crate::physical_runtime::durability::RetirementRecord>,
}

pub(in crate::physical_runtime) struct ReopenedPhysicalDurabilityOwners {
    pub(in crate::physical_runtime) durability: ReopenedPhysicalDurabilityRuntimeOwner,
    pub(in crate::physical_runtime) wal: PhysicalWalRuntimeOwner,
    pub(in crate::physical_runtime) unresolved_retirements:
        Vec<crate::physical_runtime::durability::RetirementRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDurabilityStateReopenFailure {
    Checkpoint(PhysicalBindingCompactionReopenFailure),
    Wal(PhysicalWalOpenFailure),
    Idempotency(PhysicalIdempotencyReopenFailure),
}

pub(in crate::physical_runtime) fn reopen_durability_basis(
    media: &QualifiedFilesystemMedia,
    runtime: RuntimeIdentity,
    signal_profile: PhysicalSignalProfileIdentity,
    durability: &PhysicalDurabilityRuntimeOwner,
) -> Result<PhysicalDurabilityReopenBasis, PhysicalDurabilityStateReopenFailure> {
    let observation = durability.observation();
    let checkpoint = reopen_binding_compaction(media)
        .map_err(PhysicalDurabilityStateReopenFailure::Checkpoint)?;
    let cutoff = match checkpoint {
        ReopenedPhysicalBindingCompaction::GenerationZero => {
            PhysicalWalBindingReopenCutoff::GenerationZero
        }
        ReopenedPhysicalBindingCompaction::NamespaceDurable(ref reopened) => {
            PhysicalWalBindingReopenCutoff::after_checkpoint(reopened.wal_cutoff_lsn_exclusive())
        }
    };
    let mut inventory = reopen_wal_inventory(media, observation.wal_policy(), cutoff)
        .map_err(PhysicalDurabilityStateReopenFailure::Wal)?;
    let members = inventory.take_members();
    let retirement_spans = inventory.take_retirement_spans();
    let records = inventory.take_retirement_records();
    let locations = inventory.take_retirement_locations();
    let located = records
        .into_iter()
        .zip(locations)
        .map(|(record, (start, end))| (record, start, end))
        .collect::<Vec<_>>();
    let unresolved_retirements = crate::physical_runtime::durability::unresolved_retirements(
        located.iter().map(|(record, _, _)| *record).collect(),
    );
    let retirement_holds =
        crate::physical_runtime::durability::unresolved_retirement_holds(located);
    let rebuilt = rebuild_idempotency(
        media,
        runtime,
        observation.policy_identity(),
        observation.idempotency_policy(),
        &checkpoint,
        members,
        retirement_spans,
    )
    .map_err(PhysicalDurabilityStateReopenFailure::Idempotency)?;
    let wal = PhysicalWalRuntimeOwner::from_reopened(
        media,
        runtime,
        signal_profile,
        observation.wal_policy(),
        inventory,
    );
    wal.install_retirement_holds(retirement_holds);
    Ok(PhysicalDurabilityReopenBasis {
        rebuilt,
        wal,
        unresolved_retirements,
    })
}

impl PhysicalDurabilityReopenBasis {
    pub(in crate::physical_runtime) fn install(
        self,
        durability: PhysicalDurabilityRuntimeOwner,
    ) -> ReopenedPhysicalDurabilityOwners {
        ReopenedPhysicalDurabilityOwners {
            durability: durability.install_rebuilt_idempotency(self.rebuilt),
            wal: self.wal,
            unresolved_retirements: self.unresolved_retirements,
        }
    }
}
