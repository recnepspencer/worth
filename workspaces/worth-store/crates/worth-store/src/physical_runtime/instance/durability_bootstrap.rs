use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::durability::{
    rebuild_idempotency, reopen_wal_inventory, CheckpointCustodyCandidate, CheckpointCustodyOrigin,
    PhysicalDurabilityRuntimeOwner, PhysicalWalBindingReopenCutoff, PhysicalWalRuntimeOwner,
    PreparedRecoveredCheckpointCustody, ReopenedPhysicalBindingCompaction,
    ReopenedPhysicalDurabilityRuntimeOwner,
};
use crate::physical_runtime::{
    PhysicalBindingCompactionReopenFailure, PhysicalIdempotencyReopenFailure,
    PhysicalSignalProfileIdentity, PhysicalWalOpenFailure, RuntimeIdentity,
};

pub(in crate::physical_runtime) struct PhysicalDurabilityReopenBasis {
    rebuilt: crate::physical_runtime::durability::RebuiltPhysicalMutationIdempotency,
    wal: PhysicalWalRuntimeOwner,
    unresolved_retirements: Vec<crate::physical_runtime::durability::RetirementRecord>,
    selected_checkpoint_sequence: u64,
    checkpoint_custody_origin: CheckpointCustodyOrigin,
}

/// The selected checkpoint as ordinary open read it, the custody open
/// verified against the loaded root, and any C.8 recovered custody.
pub(in crate::physical_runtime) struct OpenedCheckpointCustody {
    pub(in crate::physical_runtime) checkpoint:
        Result<ReopenedPhysicalBindingCompaction, PhysicalBindingCompactionReopenFailure>,
    pub(in crate::physical_runtime) candidate: CheckpointCustodyCandidate,
    pub(in crate::physical_runtime) recovered: Option<PreparedRecoveredCheckpointCustody>,
}

pub(in crate::physical_runtime) struct ReopenedPhysicalDurabilityOwners {
    pub(in crate::physical_runtime) durability: ReopenedPhysicalDurabilityRuntimeOwner,
    pub(in crate::physical_runtime) wal: PhysicalWalRuntimeOwner,
    pub(in crate::physical_runtime) unresolved_retirements:
        Vec<crate::physical_runtime::durability::RetirementRecord>,
    pub(in crate::physical_runtime) selected_checkpoint_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDurabilityStateReopenFailure {
    PublicationRetentionRejected,
    Checkpoint(PhysicalBindingCompactionReopenFailure),
    Wal(PhysicalWalOpenFailure),
    Idempotency(PhysicalIdempotencyReopenFailure),
    /// The current root carries a tier anchor whose clean reopen custody the
    /// retained WAL suffix does not admit; C.8 recovery must install it.
    RecoveredCheckpointCustodyRequired,
}

pub(in crate::physical_runtime) fn reopen_durability_basis(
    media: &QualifiedFilesystemMedia,
    runtime: RuntimeIdentity,
    signal_profile: PhysicalSignalProfileIdentity,
    durability: &PhysicalDurabilityRuntimeOwner,
    record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    reopen_grant: &worth_store_buffer_pool::OperationAllocationGrant,
    custody: &OpenedCheckpointCustody,
) -> Result<PhysicalDurabilityReopenBasis, PhysicalDurabilityStateReopenFailure> {
    let observation = durability.observation();
    let checkpoint = custody
        .checkpoint
        .as_ref()
        .map_err(|failure| PhysicalDurabilityStateReopenFailure::Checkpoint(*failure))?;
    let selected_checkpoint_sequence = match checkpoint {
        ReopenedPhysicalBindingCompaction::GenerationZero => 0,
        ReopenedPhysicalBindingCompaction::NamespaceDurable(reopened) => {
            reopened.rebuild_basis().checkpoint().sequence().get()
        }
    };
    let cutoff = match checkpoint {
        ReopenedPhysicalBindingCompaction::GenerationZero => {
            PhysicalWalBindingReopenCutoff::GenerationZero
        }
        ReopenedPhysicalBindingCompaction::NamespaceDurable(reopened) => {
            PhysicalWalBindingReopenCutoff::after_checkpoint(reopened.wal_cutoff_lsn_exclusive())
        }
    };
    let binding_context = crate::physical_runtime::durability::PhysicalBindingDecodingContext::new(
        media.store_identity(),
        observation.policy_identity(),
        observation.idempotency_policy(),
    );
    let mut inventory = reopen_wal_inventory(
        media,
        observation.wal_policy(),
        cutoff,
        record_format,
        binding_context,
        reopen_grant,
    )
    .map_err(PhysicalDurabilityStateReopenFailure::Wal)?;
    let checkpoint_custody_origin = custody.candidate.admit(inventory.release_evidence());
    if custody.candidate.requires_clean_custody()
        && !matches!(
            checkpoint_custody_origin,
            CheckpointCustodyOrigin::CleanReopen(_)
        )
    {
        return Err(PhysicalDurabilityStateReopenFailure::RecoveredCheckpointCustodyRequired);
    }
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
        checkpoint,
        members,
        retirement_spans,
    )
    .map_err(PhysicalDurabilityStateReopenFailure::Idempotency)?;
    if let Some(pending) = custody
        .recovered
        .as_ref()
        .and_then(|custody| custody.pending_wal_release())
    {
        rebuilt
            .reconcile_verified_pending_release(pending)
            .map_err(PhysicalDurabilityStateReopenFailure::Idempotency)?;
    }
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
        selected_checkpoint_sequence,
        checkpoint_custody_origin,
    })
}

impl PhysicalDurabilityReopenBasis {
    pub(in crate::physical_runtime) fn wal(&self) -> &PhysicalWalRuntimeOwner {
        &self.wal
    }

    pub(in crate::physical_runtime) const fn checkpoint_custody_origin(
        &self,
    ) -> CheckpointCustodyOrigin {
        self.checkpoint_custody_origin
    }

    pub(in crate::physical_runtime) fn install(
        self,
        durability: PhysicalDurabilityRuntimeOwner,
    ) -> ReopenedPhysicalDurabilityOwners {
        ReopenedPhysicalDurabilityOwners {
            durability: durability.install_rebuilt_idempotency(self.rebuilt),
            wal: self.wal,
            unresolved_retirements: self.unresolved_retirements,
            selected_checkpoint_sequence: self.selected_checkpoint_sequence,
        }
    }
}
