//! Candidate-lived prepared roots; each observation receives its own loan.
use super::preparation::PreparedSourceProgram;

pub struct PreparedManagedIndexSources {
    pub registration_generation: u64,
    pub programs: Vec<PreparedSourceProgram>,
}

impl std::fmt::Debug for PreparedManagedIndexSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedManagedIndexSources")
            .field("programs", &self.programs.len())
            .finish_non_exhaustive()
    }
}
