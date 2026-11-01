//! Immutable installed Source requirements, independent of rule execution.
use super::ManagedIndexSourceProgram;
use crate::validation::data::{CustomInvariantAccessContract, CustomInvariantRegistration};
use std::{num::NonZeroU64, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct SourcePreparationGroup {
    programs: Arc<[ManagedIndexSourceProgram]>,
    access: CustomInvariantAccessContract,
    maximum_work_units: Option<NonZeroU64>,
}

impl SourcePreparationGroup {
    /// Freeze only actual requirements. Relative rule order remains the Source
    /// owner order; rules without programs cannot contribute a posting or slot.
    pub(crate) fn from_invariant(registration: &CustomInvariantRegistration) -> Option<Self> {
        if registration.managed_sources.is_empty() {
            return None;
        }
        Some(Self {
            programs: registration.managed_sources.clone(),
            access: registration.access_contract().clone(),
            maximum_work_units: registration.maximum_work_units(),
        })
    }

    pub(crate) fn programs(&self) -> &[ManagedIndexSourceProgram] {
        &self.programs
    }

    pub(crate) fn access_contract(&self) -> &CustomInvariantAccessContract {
        &self.access
    }

    pub(crate) fn maximum_work_units(&self) -> Option<NonZeroU64> {
        self.maximum_work_units
    }
}
