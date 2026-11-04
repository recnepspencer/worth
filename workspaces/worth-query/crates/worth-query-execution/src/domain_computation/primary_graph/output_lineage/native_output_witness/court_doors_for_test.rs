//! Sealed test doors over this module's private members.
use super::*;

impl PreparedNativeOutputWitness {
    #[doc(hidden)]
    pub fn roles_for_test(&self) -> &Vec<RoleProbe> {
        &self.roles
    }
}

impl PreparedNativeOutputWitness {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        runtime: &RelationalRuntime,
        committed: &SnapshotHandle,
    ) -> Option<Arc<OnceLock<SealedNativeOutputWitness>>> {
        Self::finish(self, correspondence, runtime, committed)
    }
}

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn prepare_fact_projection_for_test(
        &self,
    ) -> Result<Option<PreparedNativeOutputFacts<'_>>, CompanionPreflightStop> {
        Self::prepare_fact_projection(self)
    }
}

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn unchanged_in_for_test(
        &self,
        runtime: &RelationalRuntime,
        selected: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, CompanionPreflightStop> {
        Self::unchanged_in(self, runtime, selected, execution)
    }
}
