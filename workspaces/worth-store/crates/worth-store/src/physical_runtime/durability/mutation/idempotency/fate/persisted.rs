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

    pub(in crate::physical_runtime) fn encode(&self, target: &mut Vec<u8>) {
        match self {
            Self::ProvenNoEffect(fate) => {
                target.push(1);
                target.push(fate.cause().encoding_code());
            }
            Self::Completed(completed) => {
                target.push(2);
                write_field(target, completed.binding.bytes());
                let breadth = completed.fact.breadth();
                target.extend_from_slice(&breadth.data_effect_count().to_le_bytes());
                target.extend_from_slice(&breadth.current_root_generation().to_le_bytes());
                target.extend_from_slice(
                    &u32::try_from(completed.fact.persisted_records().len())
                        .expect("admitted record count fits u32")
                        .to_le_bytes(),
                );
                for record in completed.fact.persisted_records() {
                    write_field(target, &record.allocation_epoch());
                    target.extend_from_slice(&record.ordinal().to_le_bytes());
                }
                for field in completed.fact.observation().persisted_fields() {
                    target.extend_from_slice(&field.to_le_bytes());
                }
            }
            Self::Indeterminate(indeterminate) => {
                target.push(3);
                target.push(indeterminate.fate.stage().encoding_code());
                target
                    .extend_from_slice(&indeterminate.fate.completed_effect_count().to_le_bytes());
                match &indeterminate.basis {
                    PersistedIndeterminatePhysicalMutationBasis::Unsealed => target.push(1),
                    PersistedIndeterminatePhysicalMutationBasis::GroupSealed(group) => {
                        target.push(2);
                        encode_group(target, *group);
                    }
                    PersistedIndeterminatePhysicalMutationBasis::WalBound(binding) => {
                        target.push(3);
                        write_field(target, binding.bytes());
                    }
                }
            }
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
        let mut records = Vec::with_capacity(record_count as usize);
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

fn encode_group(target: &mut Vec<u8>, group: PhysicalDurabilityGroupMemberBinding) {
    write_field(target, &group.group_identity().bytes());
    target.extend_from_slice(&group.ordinal().get().to_le_bytes());
    target.extend_from_slice(&group.member_count().get().to_le_bytes());
    write_field(target, &group.membership_digest());
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

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
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
