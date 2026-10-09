mod encoding;

use std::num::NonZeroU32;
use std::sync::Arc;

use worth_store_physical_format::PersistedRecordIdentity;

use super::super::persisted_binding::{
    CanonicalBindingCursor, PhysicalBindingDecodingContext, PhysicalPersistedBindingDecodeDenial,
};
use super::super::{
    registry::PhysicalMutationBindingBasis, PersistedPhysicalMutationAttemptBinding,
    PhysicalNamespaceDurableCheckpointGeneration,
};
use crate::physical_runtime::{
    CompletedPhysicalMutationFact, IndeterminatePhysicalMutation,
    PhysicalDurabilityGroupMemberBinding, PhysicalMutationIdempotencyLease,
    PhysicalMutationIndeterminateStage, PhysicalMutationProvenNoEffectCause,
    PhysicalMutationRequestFingerprint, ProvenNoEffectPhysicalMutation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PersistedPhysicalMutationFate {
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Completed(PersistedCompletedPhysicalMutation),
    Indeterminate(PersistedIndeterminatePhysicalMutation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PersistedCompletedPhysicalMutation {
    binding: PersistedPhysicalMutationAttemptBinding,
    fact: Arc<CompletedPhysicalMutationFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PersistedIndeterminatePhysicalMutation {
    basis: PersistedIndeterminatePhysicalMutationBasis,
    fate: IndeterminatePhysicalMutation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PersistedIndeterminatePhysicalMutationBasis {
    Unsealed,
    GroupSealed(PhysicalDurabilityGroupMemberBinding),
    WalBound(PersistedPhysicalMutationAttemptBinding),
}

pub(in crate::physical_runtime) enum DuplicatePhysicalMutationTerminal {
    Completed(Arc<CompletedPhysicalMutationFact>),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

impl PersistedCompletedPhysicalMutation {
    pub(in crate::physical_runtime::durability) fn into_parts(
        self,
    ) -> (
        PersistedPhysicalMutationAttemptBinding,
        Arc<CompletedPhysicalMutationFact>,
    ) {
        (self.binding, self.fact)
    }
}

impl PersistedIndeterminatePhysicalMutation {
    pub(in crate::physical_runtime::durability) fn into_parts(
        self,
    ) -> (
        PersistedIndeterminatePhysicalMutationBasis,
        IndeterminatePhysicalMutation,
    ) {
        (self.basis, self.fate)
    }
}

impl PersistedPhysicalMutationFate {
    pub(in crate::physical_runtime) fn indeterminate_wal_binding(
        &self,
    ) -> Option<&PersistedPhysicalMutationAttemptBinding> {
        let Self::Indeterminate(indeterminate) = self else {
            return None;
        };
        let PersistedIndeterminatePhysicalMutationBasis::WalBound(binding) = &indeterminate.basis
        else {
            return None;
        };
        Some(binding)
    }

    pub(in crate::physical_runtime) const fn proven_no_effect(
        terminal: ProvenNoEffectPhysicalMutation,
    ) -> Self {
        Self::ProvenNoEffect(terminal)
    }

    pub(in crate::physical_runtime) fn completed(
        binding: PersistedPhysicalMutationAttemptBinding,
        fact: Arc<CompletedPhysicalMutationFact>,
    ) -> Self {
        Self::Completed(PersistedCompletedPhysicalMutation { binding, fact })
    }

    pub(in crate::physical_runtime) const fn indeterminate(
        basis: PersistedIndeterminatePhysicalMutationBasis,
        fate: IndeterminatePhysicalMutation,
    ) -> Self {
        Self::Indeterminate(PersistedIndeterminatePhysicalMutation { basis, fate })
    }

    pub(in crate::physical_runtime) fn duplicate_observation(
        &self,
        fingerprint: PhysicalMutationRequestFingerprint,
    ) -> Option<DuplicatePhysicalMutationTerminal> {
        let matches = match self {
            Self::ProvenNoEffect(fate) => fate.request_fingerprint() == fingerprint,
            Self::Completed(fate) => fate.fact.request_fingerprint() == fingerprint,
            Self::Indeterminate(fate) => fate.fate.request_fingerprint() == fingerprint,
        };
        matches.then(|| match self {
            Self::ProvenNoEffect(fate) => {
                DuplicatePhysicalMutationTerminal::ProvenNoEffect(fate.clone())
            }
            Self::Completed(fate) => {
                DuplicatePhysicalMutationTerminal::Completed(Arc::clone(&fate.fact))
            }
            Self::Indeterminate(fate) => {
                DuplicatePhysicalMutationTerminal::Indeterminate(fate.fate.clone())
            }
        })
    }

    pub(in crate::physical_runtime) const fn requires_compaction_at(
        &self,
        lease: PhysicalMutationIdempotencyLease,
        generation: PhysicalNamespaceDurableCheckpointGeneration,
        last_compacted: Option<PhysicalNamespaceDurableCheckpointGeneration>,
    ) -> bool {
        !self.reclamation_eligible_at(lease, generation, last_compacted)
    }

    pub(in crate::physical_runtime) const fn reclamation_eligible_at(
        &self,
        lease: PhysicalMutationIdempotencyLease,
        generation: PhysicalNamespaceDurableCheckpointGeneration,
        last_compacted: Option<PhysicalNamespaceDurableCheckpointGeneration>,
    ) -> bool {
        lease.is_expired_at(generation) && last_compacted.is_some()
    }

    pub(in crate::physical_runtime) fn as_proven_no_effect(
        &self,
    ) -> Option<ProvenNoEffectPhysicalMutation> {
        match self {
            Self::ProvenNoEffect(terminal) => Some(terminal.clone()),
            Self::Completed(_) | Self::Indeterminate(_) => None,
        }
    }

    pub(in crate::physical_runtime) fn decode(
        cursor: &mut CanonicalBindingCursor<'_>,
        basis: &PhysicalMutationBindingBasis,
        context: PhysicalBindingDecodingContext,
    ) -> Result<Option<Self>, PhysicalPersistedBindingDecodeDenial> {
        let class = cursor.byte()?;
        match class {
            1 => {
                let cause = PhysicalMutationProvenNoEffectCause::decode(cursor.byte()?);
                Ok(cause.map(|cause| {
                    Self::proven_no_effect(ProvenNoEffectPhysicalMutation::before_group_seal(
                        basis.key().identity(),
                        basis.fingerprint(),
                        basis.mutation(),
                        cause,
                    ))
                }))
            }
            2 => Self::decode_completed(cursor, basis, context).map(Some),
            3 => Self::decode_indeterminate(cursor, basis, context).map(Some),
            _ => Ok(None),
        }
    }

    fn decode_completed(
        cursor: &mut CanonicalBindingCursor<'_>,
        basis: &PhysicalMutationBindingBasis,
        context: PhysicalBindingDecodingContext,
    ) -> Result<Self, PhysicalPersistedBindingDecodeDenial> {
        let binding = PersistedPhysicalMutationAttemptBinding::decode_from_compaction(
            cursor.field()?,
            context,
        )?;
        require_binding_matches(&binding, basis)?;
        let data_effect_count = cursor.u32()?;
        let current_root_generation = cursor.u64()?;
        let record_count = cursor.u32()?;
        let record_bytes = (record_count as usize)
            .checked_mul(32)
            .and_then(|bytes| bytes.checked_add(13 * 8))
            .ok_or(PhysicalPersistedBindingDecodeDenial::FieldLengthOverflow)?;
        if record_bytes > cursor.remaining_bytes() {
            return Err(PhysicalPersistedBindingDecodeDenial::Truncated);
        }
        let requested = (record_count as u64)
            .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)
            .ok_or(PhysicalPersistedBindingDecodeDenial::FieldLengthOverflow)?;
        let mut records = Vec::new();
        records
            .try_reserve_exact(record_count as usize)
            .map_err(|cause| PhysicalPersistedBindingDecodeDenial::Allocation {
                requested,
                cause,
            })?;
        if records.capacity() > record_count as usize {
            return Err(
                PhysicalPersistedBindingDecodeDenial::AllocatorExceededReservation {
                    requested,
                    actual: records.capacity() as u64
                        * std::mem::size_of::<PersistedRecordIdentity>() as u64,
                },
            );
        }
        for _ in 0..record_count {
            let allocation_epoch = cursor.array_field()?;
            let ordinal = cursor.u64()?;
            records.push(
                PersistedRecordIdentity::new(allocation_epoch, ordinal)
                    .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidIdentity)?,
            );
        }
        let mut fields = [0; 13];
        for field in &mut fields {
            *field = cursor.u64()?;
        }
        let observation =
            crate::physical_runtime::RecordAppendObservation::from_persisted_fields(fields);
        let fact = CompletedPhysicalMutationFact::from_persisted_terminal(
            &binding,
            data_effect_count,
            current_root_generation,
            records.into_boxed_slice(),
            observation,
        );
        Ok(Self::completed(binding, fact))
    }

    fn decode_indeterminate(
        cursor: &mut CanonicalBindingCursor<'_>,
        basis: &PhysicalMutationBindingBasis,
        context: PhysicalBindingDecodingContext,
    ) -> Result<Self, PhysicalPersistedBindingDecodeDenial> {
        let stage = PhysicalMutationIndeterminateStage::decode(cursor.byte()?)
            .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidIdentity)?;
        let completed_effects = cursor.u32()? as usize;
        let persisted_basis = match cursor.byte()? {
            1 => PersistedIndeterminatePhysicalMutationBasis::Unsealed,
            2 => PersistedIndeterminatePhysicalMutationBasis::GroupSealed(decode_group(
                cursor, basis,
            )?),
            3 => {
                let binding = PersistedPhysicalMutationAttemptBinding::decode_from_compaction(
                    cursor.field()?,
                    context,
                )?;
                require_binding_matches(&binding, basis)?;
                PersistedIndeterminatePhysicalMutationBasis::WalBound(binding)
            }
            _ => return Err(PhysicalPersistedBindingDecodeDenial::InvalidIdentity),
        };
        Ok(Self::indeterminate(
            persisted_basis,
            IndeterminatePhysicalMutation::possible_effect(
                basis.mutation(),
                basis.key().identity(),
                basis.fingerprint(),
                stage,
                completed_effects,
            ),
        ))
    }
}

fn decode_group(
    cursor: &mut CanonicalBindingCursor<'_>,
    basis: &PhysicalMutationBindingBasis,
) -> Result<PhysicalDurabilityGroupMemberBinding, PhysicalPersistedBindingDecodeDenial> {
    let identity = crate::physical_runtime::PhysicalDurabilityGroupIdentity::from_reopened(
        cursor.array_field()?,
    );
    let ordinal = NonZeroU32::new(cursor.u32()?)
        .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)?;
    let count = NonZeroU32::new(cursor.u32()?)
        .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)?;
    PhysicalDurabilityGroupMemberBinding::from_reopened(
        identity,
        crate::physical_runtime::PhysicalWalMemberIdentity::for_mutation(basis.mutation()),
        ordinal,
        count,
        cursor.array_field()?,
    )
    .ok_or(PhysicalPersistedBindingDecodeDenial::InvalidGroupBinding)
}

fn require_binding_matches(
    binding: &PersistedPhysicalMutationAttemptBinding,
    basis: &PhysicalMutationBindingBasis,
) -> Result<(), PhysicalPersistedBindingDecodeDenial> {
    if binding.idempotency_identity() == basis.key().identity()
        && binding.fingerprint() == basis.fingerprint()
        && binding.mutation() == basis.mutation()
    {
        Ok(())
    } else {
        Err(PhysicalPersistedBindingDecodeDenial::NonCanonicalEncoding)
    }
}

impl PhysicalMutationBindingBasis {
    pub(in crate::physical_runtime) fn matches_terminal(
        &self,
        terminal: &IndeterminatePhysicalMutation,
    ) -> bool {
        self.key().identity() == terminal.idempotency_identity()
            && self.fingerprint() == terminal.request_fingerprint()
            && self.mutation() == terminal.mutation_identity()
    }
}
