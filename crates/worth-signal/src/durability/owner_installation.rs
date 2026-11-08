//! An installation owner observes actual recovered source before service begins.
use super::{SignalDurableBranchDescriptor, SignalReadmittedConditionalContract};
use crate::data::graph::SignalGraph;

/// Borrowed installation inputs from the original branch being readmitted.
/// The graph is cold and immutable; this view grants no publication service.
pub struct SignalAuthoredBranchInstallation<'a> {
    graph: &'a SignalGraph,
    descriptor: &'a SignalDurableBranchDescriptor,
    contracts: &'a [SignalReadmittedConditionalContract],
    bridge_section: &'a [u8],
}
impl<'a> SignalAuthoredBranchInstallation<'a> {
    pub(crate) fn new(
        graph: &'a SignalGraph,
        descriptor: &'a SignalDurableBranchDescriptor,
        contracts: &'a [SignalReadmittedConditionalContract],
        bridge_section: &'a [u8],
    ) -> Self {
        Self {
            graph,
            descriptor,
            contracts,
            bridge_section,
        }
    }
    pub fn graph(&self) -> &SignalGraph {
        self.graph
    }
    pub fn descriptor(&self) -> &SignalDurableBranchDescriptor {
        self.descriptor
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
