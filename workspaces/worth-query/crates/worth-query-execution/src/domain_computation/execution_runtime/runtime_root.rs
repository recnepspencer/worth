use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryInstalledPackageIndexDenial, WorthQueryInstalledPackageIndexRelation,
};

use super::{
    WorthQueryApplicationCandidateResourceProfile, WorthQueryApplicationQueryResourceProfile,
    WorthQueryExecutionRuntimeInstallation, WorthQueryRuntimeAuthorityIdentity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphIntegrationHandle,
};

/// Execution-owned root for one installed Query operating world.
///
/// The root retains the exact installed package index that later operation
/// binding and provider routing consume. It is not reconstructible from an
/// index digest or workspace label.
pub struct WorthQueryExecutionRuntime {
    authority_identity: WorthQueryRuntimeAuthorityIdentity,
    installed_packages: Arc<WorthQueryInstalledPackageIndex>,
    current_generation: Arc<AtomicU64>,
    primary_graph: Option<WorthQueryPrimaryGraph>,
    application_query_resources: WorthQueryApplicationQueryResourceProfile,
    application_candidate_resources: WorthQueryApplicationCandidateResourceProfile,
}

/// Move-only construction authority for one execution runtime.
///
/// Provider installation may borrow its identities, but only consuming the
/// installer can publish the runtime root.
pub struct WorthQueryExecutionRuntimeInstaller {
    authority_identity: WorthQueryRuntimeAuthorityIdentity,
    installation_runtime: WorthQueryInstallationRuntimeIdentity,
    application_query_resources: WorthQueryApplicationQueryResourceProfile,
    application_candidate_resources: WorthQueryApplicationCandidateResourceProfile,
}

impl WorthQueryExecutionRuntimeInstaller {
    pub fn new() -> Self {
        Self {
            authority_identity: WorthQueryRuntimeAuthorityIdentity::mint(),
            installation_runtime: WorthQueryInstallationRuntimeIdentity::fresh(),
            application_query_resources: WorthQueryApplicationQueryResourceProfile::default(),
            application_candidate_resources: WorthQueryApplicationCandidateResourceProfile::default(
            ),
        }
    }

    pub fn application_query_resources(
        mut self,
        profile: WorthQueryApplicationQueryResourceProfile,
    ) -> Self {
        self.application_query_resources = profile;
        self
    }

    pub fn application_candidate_resources(
        mut self,
        profile: WorthQueryApplicationCandidateResourceProfile,
    ) -> Self {
        self.application_candidate_resources = profile;
        self
    }

    pub fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.authority_identity
    }

    pub fn installation_runtime(&self) -> &WorthQueryInstallationRuntimeIdentity {
        &self.installation_runtime
    }

    pub fn install(
        self,
        generation: WorthQueryInstallationGeneration,
        packages: impl IntoIterator<Item = WorthQueryAdmittedPortableDomainPackage>,
    ) -> Result<WorthQueryExecutionRuntimeInstallation, WorthQueryInstalledPackageIndexDenial> {
        let retained_installation_runtime = self
            .installation_runtime
            .retain_for_execution_installation();
        let installed_packages = WorthQueryInstalledPackageIndex::build(
            self.installation_runtime,
            generation,
            packages,
        )?;
        Ok(WorthQueryExecutionRuntimeInstallation::new(
            WorthQueryExecutionRuntime {
                authority_identity: self.authority_identity,
                current_generation: Arc::new(AtomicU64::new(generation.ordinal())),
                installed_packages: Arc::new(installed_packages),
                primary_graph: None,
                application_query_resources: self.application_query_resources,
                application_candidate_resources: self.application_candidate_resources,
            },
            retained_installation_runtime,
        ))
    }
}

impl Default for WorthQueryExecutionRuntimeInstaller {
    fn default() -> Self {
        Self::new()
    }
}

impl WorthQueryExecutionRuntime {
    pub fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.authority_identity
    }

    pub fn installed_packages(&self) -> &WorthQueryInstalledPackageIndex {
        &self.installed_packages
    }

    pub fn retain_installed_packages(&self) -> Arc<WorthQueryInstalledPackageIndex> {
        Arc::clone(&self.installed_packages)
    }

    pub fn primary_graph(&self) -> Option<&WorthQueryPrimaryGraph> {
        self.primary_graph.as_ref()
    }

    pub const fn application_query_resource_profile(
        &self,
    ) -> WorthQueryApplicationQueryResourceProfile {
        self.application_query_resources
    }

    pub const fn application_candidate_resource_profile(
        &self,
    ) -> WorthQueryApplicationCandidateResourceProfile {
        self.application_candidate_resources
    }

    pub(crate) fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<WorthQueryPrimaryGraphIntegrationHandle> {
        self.primary_graph
            .as_ref()
            .map(WorthQueryPrimaryGraph::integration_handle)
    }

    pub(crate) fn install_primary_graph(&mut self, graph: WorthQueryPrimaryGraph) {
        self.primary_graph = Some(graph);
    }

    pub(crate) fn retain_current_generation(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.current_generation)
    }

    pub fn commit_successor_installation(
        &mut self,
        successor: Arc<WorthQueryInstalledPackageIndex>,
    ) -> Result<(), WorthQueryExecutionInstallationCommitDenial> {
        match self.installed_packages.relation_to(&successor) {
            WorthQueryInstalledPackageIndexRelation::ExactSuccessor => {
                self.current_generation
                    .store(successor.generation().ordinal(), Ordering::Release);
                self.installed_packages = successor;
                Ok(())
            }
            WorthQueryInstalledPackageIndexRelation::ForeignRuntime => {
                Err(WorthQueryExecutionInstallationCommitDenial::ForeignRuntime)
            }
            WorthQueryInstalledPackageIndexRelation::EquivalentGeneration
            | WorthQueryInstalledPackageIndexRelation::SameGenerationMeaningChanged
            | WorthQueryInstalledPackageIndexRelation::NonSuccessorGeneration => {
                Err(WorthQueryExecutionInstallationCommitDenial::ExactSuccessorRequired)
            }
        }
    }

    pub fn replace_rebuilt_installation(
        &mut self,
        rebuilt: Arc<WorthQueryInstalledPackageIndex>,
    ) -> Result<(), WorthQueryExecutionInstallationCommitDenial> {
        match self.installed_packages.relation_to(&rebuilt) {
            WorthQueryInstalledPackageIndexRelation::EquivalentGeneration => {
                self.current_generation
                    .store(rebuilt.generation().ordinal(), Ordering::Release);
                self.installed_packages = rebuilt;
                Ok(())
            }
            WorthQueryInstalledPackageIndexRelation::ForeignRuntime => {
                Err(WorthQueryExecutionInstallationCommitDenial::ForeignRuntime)
            }
            WorthQueryInstalledPackageIndexRelation::SameGenerationMeaningChanged
            | WorthQueryInstalledPackageIndexRelation::ExactSuccessor
            | WorthQueryInstalledPackageIndexRelation::NonSuccessorGeneration => {
                Err(WorthQueryExecutionInstallationCommitDenial::EquivalentRebuildRequired)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionInstallationCommitDenial {
    EquivalentRebuildRequired,
    ExactSuccessorRequired,
    ForeignRuntime,
}
