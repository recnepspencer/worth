//! Borrow the exact already-judged payloads from the live publication owner.
use super::{
    preparation::PreparedManagedIndexSourceRoot, ManagedIndexSourceAvailability,
    ManagedIndexSourceProgram,
};
use crate::identity::data::EntityId;
use std::{any::Any, sync::Arc};

/// Native gives this loan only to its selected candidate's publication preflight.
/// It does not permit Source projection, invariant evaluation or public READ.
pub struct ManagedIndexSourcePublication<'a> {
    pub(crate) prepared: &'a super::staged::PreparedManagedIndexSources,
}

impl ManagedIndexSourcePublication<'_> {
    pub fn roots(&self) -> impl Iterator<Item = ManagedIndexSourcePublicationRoot<'_>> {
        self.prepared.programs.iter().flat_map(|program| {
            program
                .roots
                .iter()
                .map(|root| ManagedIndexSourcePublicationRoot { root })
        })
    }
}

pub struct ManagedIndexSourcePublicationRoot<'a> {
    root: &'a PreparedManagedIndexSourceRoot,
}
impl ManagedIndexSourcePublicationRoot<'_> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        &self.root.program
    }
    pub const fn scope(&self) -> EntityId {
        self.root.scope
    }
    pub const fn physical_availability(&self) -> ManagedIndexSourceAvailability {
        self.root.availability
    }
    pub(crate) const fn source_changed(&self) -> bool {
        self.root.source_changed
    }
    /// None is the exact Native retirement disposition, not empty membership.
    pub fn proposed(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.root.proposed.as_ref()
    }
}

impl super::NativeManagedSourceInterpretation {
    pub(crate) fn prepare_publication_loan(
        &self,
        binding: &crate::mvcc::publication::CandidateCompanionBinding,
    ) -> Result<ManagedIndexSourcePublication<'_>, crate::mvcc::CompanionPreflightStop> {
        use crate::mvcc::CompanionPreflightStop as Stop;
        let prepared = self.prepared.get().ok_or(Stop::ManagedSourceNotPrepared)?;
        let basis = self.physical_basis();
        if basis.identity().runtime_instance_id() != binding.runtime_instance_id
            || basis.identity().branch_id() != &binding.branch_id
            || basis.descriptor().root_identity() != binding.expected_root_id
        {
            return Err(Stop::SelectedSourceMismatch);
        }
        if !prepared.programs.is_empty()
            && prepared.registration_generation != binding.registration_generation
        {
            return Err(Stop::RegistrationChanged);
        }
        Ok(ManagedIndexSourcePublication { prepared })
    }
}
