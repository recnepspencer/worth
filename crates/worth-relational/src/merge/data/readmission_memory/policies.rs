//! Strategy row admissions follow the same actual immutable policy owners.
use super::{attach_policy_name, MergeLeafMemory};
use crate::merge::data::{RelationalMergeAspectPolicyWitnessRow, RelationalMergeStrategyWitness};

pub(crate) struct MergePolicyRowMemory {
    pub(crate) resolutions: MergeLeafMemory,
    pub(crate) resolution_names: Vec<Option<MergeLeafMemory>>,
    pub(crate) policies: MergeLeafMemory,
    pub(crate) policy_names: Vec<Option<MergeLeafMemory>>,
}
impl MergePolicyRowMemory {
    fn attach(&mut self, row: &RelationalMergeAspectPolicyWitnessRow) -> Result<(), &'static str> {
        if row.aspect_resolutions().len() != self.resolution_names.len()
            || row.applied_policies().len() != self.policy_names.len()
        {
            return Err("strategy policy admission count mismatch");
        }
        for (memory, resolution) in self
            .resolution_names
            .iter_mut()
            .zip(row.aspect_resolutions())
        {
            if let Some(policy) = &resolution.applied_policy {
                attach_policy_name(memory, policy)?;
            } else if memory.is_some() {
                return Err("unexpected absent policy admission");
            }
        }
        for (memory, policy) in self.policy_names.iter_mut().zip(row.applied_policies()) {
            attach_policy_name(memory, &policy.policy)?;
        }
        self.resolutions
            .install(row.aspect_resolutions().len(), |memory| {
                row.retain_aspect_resolutions_memory(memory)
            })?;
        self.policies
            .install(row.applied_policies().len(), |memory| {
                row.retain_applied_policies_memory(memory)
            })
    }
}

pub(crate) struct MergeStrategyMemory {
    pub(crate) aspects: MergeLeafMemory,
    pub(crate) rows: Vec<MergePolicyRowMemory>,
    pub(crate) topology: MergeLeafMemory,
    pub(crate) deletion: MergeLeafMemory,
}
impl MergeStrategyMemory {
    pub(super) fn attach(
        &mut self,
        witness: &RelationalMergeStrategyWitness,
    ) -> Result<(), &'static str> {
        if self.rows.len() != witness.aspect_policy_rows().len() {
            return Err("strategy admission count mismatch");
        }
        for (memory, row) in self.rows.iter_mut().zip(witness.aspect_policy_rows()) {
            memory.attach(row)?;
        }
        self.aspects
            .install(witness.aspect_policy_rows().len(), |memory| {
                witness.retain_aspect_policy_rows_memory(memory)
            })?;
        self.topology
            .install(witness.topology_rows().len(), |memory| {
                witness.retain_topology_rows_memory(memory)
            })?;
        self.deletion
            .install(witness.deletion_rows().len(), |memory| {
                witness.retain_deletion_rows_memory(memory)
            })
    }
}
