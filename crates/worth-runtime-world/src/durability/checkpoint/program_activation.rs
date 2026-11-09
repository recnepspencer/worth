//! Read-only program facts from the same protected owner image and tails.
use super::{WorldCheckpointImage, WorldCheckpointReadSource};
use crate::durability::{
    DecodedWorldRecord, WorldDurableHomeBinding, WorldDurableMediumBinding, WorldReadmissionDenial,
    WorldRecord,
};
use std::sync::Arc;
use worth_foundational::facade::DurableCompositeCommitIdentity;

#[derive(Debug)]
pub struct WorldProgramActivationHistory {
    pub(super) image: WorldCheckpointImage,
    _source: Box<dyn WorldCheckpointReadSource>,
    binding: Arc<WorldDurableMediumBinding>,
}
/// Borrows the original admitted record; it grants neither a writer nor a receipt.
pub struct WorldProgramActivationHistoryEntry<'a> {
    record: &'a DecodedWorldRecord,
    binding: &'a WorldDurableHomeBinding,
}
impl WorldProgramActivationHistory {
    pub(super) fn capture(
        binding: Arc<WorldDurableMediumBinding>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldReadmissionDenial> {
        let source = binding
            .capture_checkpoint_source(request)
            .map_err(WorldReadmissionDenial::Medium)?;
        let (image, _) = WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            binding.binding(),
            request,
            None,
        )?;
        Ok(Self {
            image,
            _source: source,
            binding,
        })
    }
    pub fn entry(
        &self,
        commit: &crate::identity::CompositeCommitIdentity,
    ) -> Result<Option<WorldProgramActivationHistoryEntry<'_>>, WorldReadmissionDenial> {
        let owner = commit.owner_identity();
        if owner.durable() != Some(self.binding.binding().owner())
            || owner.home_incarnation_fence().is_none_or(|fence| {
                fence
                    .admit(self.binding.binding().incarnation().fence())
                    .is_err()
            })
        {
            return Err(WorldReadmissionDenial::ForeignOwner);
        }
        let key = DurableCompositeCommitIdentity::from_recorded(
            self.binding.binding().owner(),
            commit.ordinal(),
        );
        Ok(self
            .image
            .history
            .get(&key)
            .map(|head| WorldProgramActivationHistoryEntry {
                record: head.program_record(),
                binding: self.binding.binding(),
            }))
    }
}
impl WorldProgramActivationHistoryEntry<'_> {
    /// The original's actual execution attribution, unchanged by D16's target.
    pub fn recorded_revision(&self) -> [u8; 32] {
        match &self.record.record {
            WorldRecord::Branch {
                program_revision, ..
            } => *program_revision,
            WorldRecord::OpenAttempt(open) => open.program_revision,
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::Continued { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
    /// Branch activation before this effect. Exact acknowledged canonical D16
    /// steps may run under A attribution after installing B; ordinary effects
    /// must match their recorded revision without an exemption.
    pub fn expected_activation(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], WorldReadmissionDenial> {
        match &self.record.record {
            WorldRecord::Branch {
                program_revision, ..
            } => Ok(*program_revision),
            WorldRecord::OpenAttempt(open) => match &self.binding.lineage {
                Some(lineage) => lineage.recorded_activation_for_original(
                    &open.association,
                    open.program_revision,
                    request,
                ),
                None => Ok(open.program_revision),
            },
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::Continued { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
    pub fn predecessor(&self) -> Option<DurableCompositeCommitIdentity> {
        match &self.record.record {
            WorldRecord::Branch { .. } => None,
            WorldRecord::OpenAttempt(open) => Some(open.expected.commit),
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::Continued { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
}
