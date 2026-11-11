//! An installation owner observes actual recovered source before service begins.
use super::SignalReadmittedConditionalContract;
use crate::data::graph::SignalGraph;
use worth_foundational::facade::DurableSignalBranchIdentity;

/// Read-only source from a recovered branch or a private owner fork.
/// This descriptive view grants no serving or publication authority.
pub struct SignalAuthoredBranchInstallation<'a> {
    graph: &'a SignalGraph,
    branch: DurableSignalBranchIdentity,
    contracts: &'a [SignalReadmittedConditionalContract],
    bridge_section: &'a [u8],
}
impl<'a> SignalAuthoredBranchInstallation<'a> {
    pub(crate) fn new(
        graph: &'a SignalGraph,
        branch: DurableSignalBranchIdentity,
        contracts: &'a [SignalReadmittedConditionalContract],
        bridge_section: &'a [u8],
    ) -> Self {
        Self {
            graph,
            branch,
            contracts,
            bridge_section,
        }
    }
    pub fn graph(&self) -> &SignalGraph {
        self.graph
    }
    pub fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub fn contracts(&self) -> &[SignalReadmittedConditionalContract] {
        self.contracts
    }
    pub fn bridge_section(&self) -> &[u8] {
        self.bridge_section
    }
}

/// The installing owner retains its own typed reason when it refuses a branch.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SignalAuthoredInstallationRefused;

/// Rebind another owner's declarations to the actual immutable recovered graph.
/// Signal invokes this once for each live branch, before admitting any service.
pub trait SignalAuthoredOwnerInstallation {
    fn readmit_branch(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredInstallationRefused>;
}

/// A composition whose only authored owner is Signal has no Bridge installation.
pub struct SignalOnlyOwnerInstallation;
impl SignalAuthoredOwnerInstallation for SignalOnlyOwnerInstallation {
    fn readmit_branch(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredInstallationRefused> {
        if branch.bridge_section().is_empty() {
            Ok(())
        } else {
            Err(SignalAuthoredInstallationRefused)
        }
    }
}
