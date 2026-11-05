//! Sealed test doors over a source custody's private members.
use super::*;

impl SourceCustody {
    #[doc(hidden)]
    pub fn cleanup_for_test(&self) -> &cleanup::CustodyCleanupLinks {
        &self.cleanup
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn new_for_test(
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        root_kind: PreparedOutputRootKind,
        source: Option<WorthQueryPerformedOutputDemandSource>,
        discovery: Option<DiscoveredSourceRecovery>,
        consumed_sources: Vec<SourceEpoch>,
        retired_sources: Vec<(SourceEpoch, WorthQueryOutputDemandDenial)>,
        bound_sources: Option<Vec<BoundOutputSource>>,
        retired: Option<WorthQueryOutputDemandDenial>,
        token_count: usize,
        cleanup: cleanup::CustodyCleanupLinks,
        completed: bool,
    ) -> Self {
        Self {
            occurrence,
            root_kind,
            source,
            discovery,
            consumed_sources,
            retired_sources,
            bound_sources,
            retired,
            token_count,
            cleanup,
            completed,
        }
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn bound_sources_for_test(&self) -> &Option<Vec<BoundOutputSource>> {
        &self.bound_sources
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn completed_for_test(&self) -> &bool {
        &self.completed
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn consumed_sources_for_test(&self) -> &Vec<SourceEpoch> {
        &self.consumed_sources
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn occurrence_for_test(&self) -> &worth_runtime_world::facade::ProductBranchIncarnation {
        &self.occurrence
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn retired_for_test(&self) -> &Option<WorthQueryOutputDemandDenial> {
        &self.retired
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn root_kind_for_test(&self) -> &PreparedOutputRootKind {
        &self.root_kind
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &Option<WorthQueryPerformedOutputDemandSource> {
        &self.source
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn token_count_for_test(&self) -> &usize {
        &self.token_count
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn completed_mut_for_test(&mut self) -> &mut bool {
        &mut self.completed
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn retired_mut_for_test(&mut self) -> &mut Option<WorthQueryOutputDemandDenial> {
        &mut self.retired
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn token_count_mut_for_test(&mut self) -> &mut usize {
        &mut self.token_count
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn source_denial_for_test(
        &self,
        identity: &SourceEpoch,
    ) -> Option<WorthQueryOutputDemandDenial> {
        Self::source_denial(self, identity)
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn retired_sources_for_test(&self) -> &Vec<(SourceEpoch, WorthQueryOutputDemandDenial)> {
        &self.retired_sources
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn consumed_sources_mut_for_test(&mut self) -> &mut Vec<SourceEpoch> {
        &mut self.consumed_sources
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn prepared_count_for_test(&self) -> usize {
        Self::prepared_count(self)
    }
}
