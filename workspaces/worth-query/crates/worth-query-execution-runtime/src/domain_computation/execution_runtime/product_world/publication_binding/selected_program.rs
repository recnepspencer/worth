//! The already observed branch programme is carried with this exact publication basis.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};

/// The rostered program actually carried by one selected product occurrence.
/// This is descriptive operator evidence; it grants no mutation or adoption
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedProgramInspection {
    identity: ApplicationProgramIdentity,
    revision: ApplicationProgramRevision,
}

impl WorthQuerySelectedProgramInspection {
    pub(crate) fn from_rostered(
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Self {
        Self { identity, revision }
    }

    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }

    pub const fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }
}

/// Crate-boundary port for constructing a rostered selected-program inspection
/// inside the Query crate. Never exported on `worth-query-execution::facade` or Host.
pub trait SelectedProgramInspectionInternalPort: Sized {
    fn from_rostered(
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Self;
}

impl SelectedProgramInspectionInternalPort for WorthQuerySelectedProgramInspection {
    fn from_rostered(
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Self {
        Self::from_rostered(identity, revision)
    }
}

pub enum SelectedProgramSourceRetentionDenial {
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
        selected: &WorthQuerySelectedProgramInspection,
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
    pub(crate) fn carry_selected_program_source_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), SelectedProgramSourceRetentionDenial> {
        let revision = match &prior.program_source {
            BranchProgramSourceCache::MemoryOnly => return Ok(()),
            BranchProgramSourceCache::Directory { selected, .. } => {
                let Some(selected) = selected else {
                    return Ok(());
                };
                selected.revision
            }
        };
        let slot = match &mut self.program_source {
            BranchProgramSourceCache::MemoryOnly => return Ok(()),
            BranchProgramSourceCache::Directory { selected, .. } => selected,
        };
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
