//! One decode's original admissions transfer to the actual shared merge leaves.
use super::{AspectMergePolicyKind, IdentityBasisKind};
use crate::transactions::data::PublishedMergeExecutionAuthority;
use worth_execution::ExecutionMemoryReservation;

mod policies;
pub(crate) use policies::{MergePolicyRowMemory, MergeStrategyMemory};

pub(crate) struct MergeLeafMemory {
    count: usize,
    bytes: u64,
    memory: Option<ExecutionMemoryReservation>,
}
impl MergeLeafMemory {
    pub(crate) fn new(
        count: usize,
        bytes: u64,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            count,
            bytes,
            memory,
        }
    }
    pub(crate) fn install(
        &mut self,
        count: usize,
        install: impl FnOnce(
            Option<ExecutionMemoryReservation>,
        ) -> Result<(), Option<ExecutionMemoryReservation>>,
    ) -> Result<(), &'static str> {
        if count != self.count
            || self
                .memory
                .as_ref()
                .is_some_and(|held| held.bytes() != self.bytes)
        {
            return Err("merge leaf admission does not match its original decoded backing");
        }
        // Only this one-decode private handoff can initialize the cells. Any
        // legitimate internal alias shares an already installed original grant;
        // its redundant, unused admission can be released without losing data.
        if let Err(unused) = install(self.memory.take()) {
            drop(unused);
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct MergeIdentityMemory {
    pub(crate) keys: Option<MergeLeafMemory>,
    pub(crate) custom_name: Option<MergeLeafMemory>,
}
impl MergeIdentityMemory {
    pub(crate) fn attach(&mut self, basis: &IdentityBasisKind) -> Result<(), &'static str> {
        match basis {
            IdentityBasisKind::DeclaredKeySet(keys) if self.custom_name.is_none() => self
                .keys
                .as_mut()
                .ok_or("missing identity key-set admission")?
                .install(keys.len(), |memory| keys.retain_decoded_memory(memory)),
            IdentityBasisKind::Custom(custom) if self.keys.is_none() => self
                .custom_name
                .as_mut()
                .ok_or("missing identity name admission")?
                .install(custom.name.len(), |memory| {
                    custom.name.retain_decoded_memory(memory)
                }),
            IdentityBasisKind::StorageIdentity
            | IdentityBasisKind::LineageIdentity
            | IdentityBasisKind::StructuralFingerprint
                if self.keys.is_none() && self.custom_name.is_none() =>
            {
                Ok(())
            }
            _ => Err("merge identity admission kind does not match decoded meaning"),
        }
    }
}

pub(crate) fn attach_policy_name(
    memory: &mut Option<MergeLeafMemory>,
    policy: &AspectMergePolicyKind,
) -> Result<(), &'static str> {
    match (memory.as_mut(), policy) {
        (Some(memory), AspectMergePolicyKind::Custom(custom)) => memory
            .install(custom.name.len(), |memory| {
                custom.name.retain_decoded_memory(memory)
            }),
        (None, AspectMergePolicyKind::Custom(_)) | (Some(_), _) => {
            Err("merge policy name admission mismatch")
        }
        (None, _) => Ok(()),
    }
}

pub(crate) struct MergeBasisMemory {
    pub(crate) left: MergeLeafMemory,
    pub(crate) right: MergeLeafMemory,
}
impl MergeBasisMemory {
    fn attach(
        &mut self,
        basis: &crate::history::data::RelationalMergeBranchBasis,
    ) -> Result<(), &'static str> {
        let base = basis.merge_base();
        self.left
            .install(base.supporting_left_ancestors.len(), |memory| {
                base.supporting_left_ancestors.retain_decoded_memory(memory)
            })?;
        self.right
            .install(base.supporting_right_ancestors.len(), |memory| {
                base.supporting_right_ancestors
                    .retain_decoded_memory(memory)
            })
    }
}

pub(crate) struct MergeReadmissionMemory {
    pub(crate) basis: MergeBasisMemory,
    pub(crate) correspondence: MergeLeafMemory,
    pub(crate) correspondence_identities: Vec<MergeIdentityMemory>,
    pub(crate) schema: MergeLeafMemory,
    pub(crate) schema_identities: Vec<MergeIdentityMemory>,
    pub(crate) strategy: MergeStrategyMemory,
    pub(crate) packet_basis: MergeBasisMemory,
    pub(crate) packet_surface: MergeLeafMemory,
    // Every transient tree buffer and untransferred leaf admission precedes
    // this ticket. The tree never owns publication or service authority.
    pub(crate) _tree_memory: Option<ExecutionMemoryReservation>,
}
impl MergeReadmissionMemory {
    pub(crate) fn attach(
        &mut self,
        authority: &PublishedMergeExecutionAuthority,
    ) -> Result<(), &'static str> {
        let summary = &authority.execution_summary;
        self.basis.attach(&summary.branch_basis)?;
        let witness = &summary.correspondence_witness;
        let rows = witness.rows();
        if rows.len() != self.correspondence_identities.len() {
            return Err("correspondence admission count mismatch");
        }
        for (memory, row) in self.correspondence_identities.iter_mut().zip(rows) {
            memory.attach(row.authority_basis())?;
        }
        self.correspondence
            .install(rows.len(), |memory| witness.retain_rows_memory(memory))?;
        let witness = &summary.schema_reconciliation_witness;
        let rows = witness.rows();
        if rows.len() != self.schema_identities.len() {
            return Err("schema admission count mismatch");
        }
        for (memory, row) in self.schema_identities.iter_mut().zip(rows) {
            if let Some(link) = row.correspondence_linkage() {
                memory.attach(&link.basis)?;
            } else if memory.keys.is_some() || memory.custom_name.is_some() {
                return Err("unexpected schema linkage admission");
            }
        }
        self.schema
            .install(rows.len(), |memory| witness.retain_rows_memory(memory))?;
        self.strategy.attach(&summary.strategy_witness)?;
        self.packet_basis
            .attach(summary.proof_packet.branch_basis())?;
        let packet = &summary.proof_packet;
        self.packet_surface
            .install(packet.admitted_merge_surface().len(), |memory| {
                packet.retain_admitted_merge_surface_memory(memory)
            })
    }
}
