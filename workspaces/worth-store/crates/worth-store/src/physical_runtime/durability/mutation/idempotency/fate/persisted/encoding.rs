//! The persisted terminal-fate grammar for byte production and comparison.

use super::super::super::canonical_encoding::CanonicalBindingEncoding;
use super::*;

impl PersistedPhysicalMutationFate {
    pub(in crate::physical_runtime) fn encode(&self, target: &mut impl CanonicalBindingEncoding) {
        match self {
            Self::ProvenNoEffect(fate) => {
                target.push(1);
                target.push(fate.cause().encoding_code());
            }
            Self::Completed(completed) => {
                target.push(2);
                target.field(completed.binding.bytes());
                let breadth = completed.fact.breadth();
                target.write(&breadth.data_effect_count().to_le_bytes());
                target.write(&breadth.current_root_generation().to_le_bytes());
                target.write(
                    &u32::try_from(completed.fact.persisted_records().len())
                        .expect("admitted record count fits u32")
                        .to_le_bytes(),
                );
                for record in completed.fact.persisted_records() {
                    target.field(&record.allocation_epoch());
                    target.write(&record.ordinal().to_le_bytes());
                }
                for field in completed.fact.observation().persisted_fields() {
                    target.write(&field.to_le_bytes());
                }
            }
            Self::Indeterminate(indeterminate) => {
                target.push(3);
                target.push(indeterminate.fate.stage().encoding_code());
                target.write(&indeterminate.fate.completed_effect_count().to_le_bytes());
                match &indeterminate.basis {
                    PersistedIndeterminatePhysicalMutationBasis::Unsealed => target.push(1),
                    PersistedIndeterminatePhysicalMutationBasis::GroupSealed(group) => {
                        target.push(2);
                        encode_group(target, *group);
                    }
                    PersistedIndeterminatePhysicalMutationBasis::WalBound(binding) => {
                        target.push(3);
                        target.field(binding.bytes());
                    }
                }
            }
        }
    }
}

fn encode_group(
    target: &mut impl CanonicalBindingEncoding,
    group: PhysicalDurabilityGroupMemberBinding,
) {
    target.field(&group.group_identity().bytes());
    target.write(&group.ordinal().get().to_le_bytes());
    target.write(&group.member_count().get().to_le_bytes());
    target.field(&group.membership_digest());
}
