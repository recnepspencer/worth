use sha2::{Digest, Sha256};
use worth_store_physical_format::PhysicalCheckpointIdentity;

use super::lease::PhysicalNamespaceDurableCheckpointGeneration;
use super::registry::{
    PhysicalMutationIdempotencyBindingState, PhysicalMutationIdempotencyRegistry,
    RebuiltPhysicalMutationBindingState,
};

mod decoding;
mod encoding;
#[cfg(test)]
#[path = "binding_compaction/tests.rs"]
mod tests;
pub(in crate::physical_runtime) use decoding::DecodedPhysicalMutationBindingRecord;
use encoding::{encode_group_sealed, encode_terminal, encode_unsealed, encode_wal_bound};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMutationBindingCompaction {
    checkpoint: PhysicalCheckpointIdentity,
    generation: PhysicalNamespaceDurableCheckpointGeneration,
    wal_cutoff_lsn_exclusive: u64,
    binding_count: u64,
    unresolved_binding_count: u64,
    terminal_binding_count: u64,
    records_digest: [u8; 32],
}

pub(in crate::physical_runtime) struct PendingPhysicalMutationBindingCompaction {
    authority: PhysicalMutationBindingCompaction,
    prior_generation: PhysicalNamespaceDurableCheckpointGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationBindingCompactionDenial {
    GenerationExhausted,
    BindingCountOverflow,
    RecordTooLarge,
    RegistryChanged,
    NamespaceSyncMismatch,
}

impl PhysicalMutationIdempotencyRegistry {
    pub(super) fn prepare_binding_compaction(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        wal_cutoff_lsn_exclusive: u64,
    ) -> Result<PendingPhysicalMutationBindingCompaction, PhysicalMutationBindingCompactionDenial>
    {
        let generation = self
            .generation
            .checked_successor()
            .ok_or(PhysicalMutationBindingCompactionDenial::GenerationExhausted)?;
        let mut unresolved_binding_count = 0_u64;
        let mut terminal_binding_count = 0_u64;
        let mut digest = Sha256::new();

        for state in self.bindings.values() {
            let Some(encoded) = encode_retained_record(state, generation) else {
                continue;
            };
            match state {
                PhysicalMutationIdempotencyBindingState::Unsealed(_) => {
                    unresolved_binding_count = increment(unresolved_binding_count)?;
                }
                PhysicalMutationIdempotencyBindingState::GroupSealed { .. } => {
                    unresolved_binding_count = increment(unresolved_binding_count)?;
                }
                PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { .. } => {
                    unresolved_binding_count = increment(unresolved_binding_count)?;
                }
                PhysicalMutationIdempotencyBindingState::WalBound { .. } => {
                    unresolved_binding_count = increment(unresolved_binding_count)?;
                }
                PhysicalMutationIdempotencyBindingState::Terminal { .. } => {
                    terminal_binding_count = increment(terminal_binding_count)?;
                }
            }
            if encoded.len() > worth_store_physical_format::MAX_CHECKPOINT_BINDING_RECORD_BYTES {
                return Err(PhysicalMutationBindingCompactionDenial::RecordTooLarge);
            }
            digest.update((encoded.len() as u64).to_le_bytes());
            digest.update(&encoded);
        }
        let binding_count = unresolved_binding_count
            .checked_add(terminal_binding_count)
            .ok_or(PhysicalMutationBindingCompactionDenial::BindingCountOverflow)?;
        Ok(PendingPhysicalMutationBindingCompaction {
            authority: PhysicalMutationBindingCompaction {
                checkpoint,
                generation,
                wal_cutoff_lsn_exclusive,
                binding_count,
                unresolved_binding_count,
                terminal_binding_count,
                records_digest: digest.finalize().into(),
            },
            prior_generation: self.generation,
        })
    }
}

impl PendingPhysicalMutationBindingCompaction {
    pub(in crate::physical_runtime) const fn generation(
        &self,
    ) -> PhysicalNamespaceDurableCheckpointGeneration {
        self.authority.generation
    }

    pub(in crate::physical_runtime) const fn wal_cutoff_lsn_exclusive(&self) -> u64 {
        self.authority.wal_cutoff_lsn_exclusive
    }

    pub(in crate::physical_runtime) const fn prior_generation(
        &self,
    ) -> PhysicalNamespaceDurableCheckpointGeneration {
        self.prior_generation
    }

    pub(super) fn for_each_record<E>(
        &self,
        registry: &PhysicalMutationIdempotencyRegistry,
        mut consume: impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<(), E> {
        for state in registry.bindings.values() {
            if let Some(encoded) = encode_retained_record(state, self.authority.generation) {
                consume(&encoded)?;
            }
        }
        Ok(())
    }

    pub(super) fn commit(
        self,
        registry: &mut PhysicalMutationIdempotencyRegistry,
        namespace_sync: &crate::physical_runtime::work::CompletedPhysicalCheckpointAction,
    ) -> Result<PhysicalMutationBindingCompaction, PhysicalMutationBindingCompactionDenial> {
        if namespace_sync.action()
            != crate::physical_runtime::PhysicalCheckpointRecoveryAction::SynchronizeNamespace
            || namespace_sync.role()
                != worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication
        {
            return Err(PhysicalMutationBindingCompactionDenial::NamespaceSyncMismatch);
        }
        if registry.generation != self.prior_generation {
            return Err(PhysicalMutationBindingCompactionDenial::RegistryChanged);
        }
        registry.bindings.retain(|_, state| {
            let PhysicalMutationIdempotencyBindingState::Terminal {
                basis,
                fate,
                last_compacted,
                ..
            } = state
            else {
                return true;
            };
            if fate.reclamation_eligible_at(
                basis.key().lease(),
                self.authority.generation,
                *last_compacted,
            ) {
                return false;
            }
            *last_compacted = Some(self.authority.generation);
            true
        });
        registry.generation = self.authority.generation;
        Ok(self.authority)
    }
}

fn encode_retained_record(
    state: &PhysicalMutationIdempotencyBindingState,
    generation: PhysicalNamespaceDurableCheckpointGeneration,
) -> Option<Vec<u8>> {
    match state {
        PhysicalMutationIdempotencyBindingState::Unsealed(basis) => Some(encode_unsealed(basis)),
        PhysicalMutationIdempotencyBindingState::GroupSealed { basis, group } => {
            Some(encode_group_sealed(basis, *group))
        }
        PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, prior } => {
            Some(match prior {
                RebuiltPhysicalMutationBindingState::Unsealed => encode_unsealed(basis),
                RebuiltPhysicalMutationBindingState::GroupSealed(group) => {
                    encode_group_sealed(basis, *group)
                }
            })
        }
        PhysicalMutationIdempotencyBindingState::WalBound { persisted, .. } => {
            Some(encode_wal_bound(persisted))
        }
        PhysicalMutationIdempotencyBindingState::Terminal {
            basis,
            fate,
            last_compacted,
        } => fate
            .requires_compaction_at(basis.key().lease(), generation, *last_compacted)
            .then(|| encode_terminal(basis, fate)),
    }
}

impl PhysicalMutationBindingCompaction {
    pub const fn checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }

    pub const fn generation(&self) -> PhysicalNamespaceDurableCheckpointGeneration {
        self.generation
    }

    pub const fn wal_cutoff_lsn_exclusive(&self) -> u64 {
        self.wal_cutoff_lsn_exclusive
    }

    pub const fn binding_count(&self) -> u64 {
        self.binding_count
    }

    pub const fn unresolved_binding_count(&self) -> u64 {
        self.unresolved_binding_count
    }

    pub const fn terminal_binding_count(&self) -> u64 {
        self.terminal_binding_count
    }

    pub const fn records_digest(&self) -> [u8; 32] {
        self.records_digest
    }
}

fn increment(value: u64) -> Result<u64, PhysicalMutationBindingCompactionDenial> {
    value
        .checked_add(1)
        .ok_or(PhysicalMutationBindingCompactionDenial::BindingCountOverflow)
}
