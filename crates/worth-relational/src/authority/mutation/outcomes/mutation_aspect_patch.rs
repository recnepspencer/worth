//! One original mutation patch keeps its own admitted allocation until consumed.
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AuthoritativeRecordAspectPatch;

pub(crate) struct MutationAspectPatch {
    patch: AuthoritativeRecordAspectPatch,
    // Last: this reservation never outlives the data by dropping first.
    _funding: Option<ExecutionMemoryReservation>,
}
impl MutationAspectPatch {
    pub(crate) fn as_patch(&self) -> &AuthoritativeRecordAspectPatch {
        &self.patch
    }
    /// Existing independent ordinary production; this constructor qualifies no funding.
    pub(crate) fn from_ordinary(patch: AuthoritativeRecordAspectPatch) -> Self {
        Self {
            patch,
            _funding: None,
        }
    }
    pub(in crate::authority::mutation) fn controlled(
        patch: AuthoritativeRecordAspectPatch,
        funding: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            patch,
            _funding: funding,
        }
    }
}
impl std::fmt::Debug for MutationAspectPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.patch.fmt(f)
    }
}
impl PartialEq for MutationAspectPatch {
    fn eq(&self, other: &Self) -> bool {
        self.patch == other.patch
    }
}
impl Eq for MutationAspectPatch {}
