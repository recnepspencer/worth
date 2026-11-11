//! The already observed branch programme is carried with this exact publication basis.
use super::*;
pub(crate) enum SelectedProgramSourceRetentionDenial {
    Head(worth_runtime_world::facade::WorldDurableMediumDenial),
    ConflictingObservation,
}
#[derive(Clone)]
pub(super) struct SelectedBranchProgramSource {
    revision: [u8; 32],
    commitment: [u8; 32],
}
#[derive(Clone)]
pub(super) enum BranchProgramSourceCache {
    MemoryOnly,
    Directory {
        owner: Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        selected: Option<SelectedBranchProgramSource>,
    },
}
impl BranchProgramSourceCache {
    pub(super) fn for_owner(
        owner: Option<Arc<worth_proof::InstalledBranchProgramSourceOwner>>,
    ) -> Self {
        match owner {
            None => Self::MemoryOnly,
            Some(owner) => Self::Directory {
                owner,
                selected: None,
            },
        }
    }
}
impl WorthQueryProductPublicationBinding {
    pub(crate) fn retain_selected_program_source(
        &mut self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspection,
    ) -> Result<(), SelectedProgramSourceRetentionDenial> {
        let slot = match &mut self.program_source {
            BranchProgramSourceCache::MemoryOnly => return Ok(()),
            BranchProgramSourceCache::Directory { selected, .. } => selected,
        };
        let revision = *selected.revision().as_bytes();
        let commitment = worth_runtime_world::facade::world_branch_program_source_commitment(
            &self.observation,
            revision,
        )
        .map_err(SelectedProgramSourceRetentionDenial::Head)?;
        let candidate = SelectedBranchProgramSource {
            revision,
            commitment,
        };
        match slot {
            None => {
                *slot = Some(candidate);
                Ok(())
            }
            Some(original)
                if (original.revision, original.commitment)
                    == (candidate.revision, candidate.commitment) =>
            {
                Ok(())
            }
            Some(_) => Err(SelectedProgramSourceRetentionDenial::ConflictingObservation),
        }
    }
    pub(crate) fn admitted_program_source(
        &self,
    ) -> Result<
        worth_proof::AdmittedBranchProgramSource,
        worth_runtime_world::facade::WorldOpenAttemptRefusal,
    > {
        let (owner, selected) = match &self.program_source {
            BranchProgramSourceCache::MemoryOnly => return Err(
                worth_runtime_world::facade::WorldOpenAttemptRefusal::ProgramSourceOwnerRequired,
            ),
            BranchProgramSourceCache::Directory { owner, selected } => (owner, selected),
        };
        let selected = selected.as_ref().ok_or(
            worth_runtime_world::facade::WorldOpenAttemptRefusal::ProgramSourceOwnerRequired,
        )?;
        Ok(owner.admit_source(selected.revision, selected.commitment))
    }
}

impl std::fmt::Debug for SelectedProgramSourceRetentionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Head(cause) => f.debug_tuple("Head").field(cause).finish(),
            Self::ConflictingObservation => f.write_str("ConflictingObservation"),
        }
    }
}
