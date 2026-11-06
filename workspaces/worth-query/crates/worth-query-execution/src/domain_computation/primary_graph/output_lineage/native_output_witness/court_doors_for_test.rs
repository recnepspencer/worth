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

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn roles_for_test(&self) -> &Vec<RoleProbe> {
        &self.roles
    }
}

impl RoleProbe {
    #[doc(hidden)]
    pub fn kind_mut_for_test(&mut self) -> &mut KindId {
        &mut self.kind
    }
}

impl RoleProbe {
    #[doc(hidden)]
    pub fn posture_mut_for_test(&mut self) -> &mut WorthQueryApplicationOutputPosture {
        &mut self.posture
    }
}

impl PreparedNativeOutputFacts<'_> {
    #[doc(hidden)]
    pub fn append_into_for_test(self, facts: &mut Vec<Fact>) {
        Self::append_into(self, facts)
    }
}

impl PreparedNativeOutputFacts<'_> {
    #[doc(hidden)]
    pub fn count_for_test(&self) -> usize {
        Self::count(self)
    }
}

impl RoleProbe {
    #[doc(hidden)]
    pub fn retirement_for_test(&self) -> &Option<RetirementMetadata> {
        &self.retirement
    }
}

impl RoleProbe {
    #[doc(hidden)]
    pub fn retirement_mut_for_test(&mut self) -> &mut Option<RetirementMetadata> {
        &mut self.retirement
    }
}

impl PreparedNativeOutputWitness {
    #[doc(hidden)]
    pub fn roles_mut_for_test(&mut self) -> &mut Vec<RoleProbe> {
        &mut self.roles
    }
}

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn roles_mut_for_test(&mut self) -> &mut Vec<RoleProbe> {
        &mut self.roles
    }
}
