//! Another owner's authored section is rebuilt against an actual private fork.
use super::{
    SignalAuthoredBranchInstallation, SignalBridgeAuthoredSection,
    SignalDefinitionTransitionArtifact, SignalReadmittedConditionalContract,
};
use crate::branch::{owner_services::SignalDurableAdvanceDenial as D, SignalBranchObservation};
use crate::data::{conditional_execution::InstalledSignalConditionalContract, graph::SignalGraph};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalConditionalDefinitionOccurrence,
    DurableSignalNodeIdentity, SignalBranchAdvanceIdentity,
};

pub struct SignalForkPublicationContext<'a> {
    pub(crate) original: SignalBranchAdvanceIdentity,
    pub(crate) branch: DurableSignalBranchIdentity,
    pub(crate) observation: &'a SignalBranchObservation,
    pub(crate) graph: &'a SignalGraph,
    pub(crate) predecessor: &'a SignalDefinitionTransitionArtifact,
    pub(crate) occurrences: &'a [DurableSignalConditionalDefinitionOccurrence],
}
impl SignalForkPublicationContext<'_> {
    pub fn original(&self) -> SignalBranchAdvanceIdentity {
        self.original
    }
    pub fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub fn observation(&self) -> &SignalBranchObservation {
        self.observation
    }
    pub fn predecessor(&self) -> &SignalDefinitionTransitionArtifact {
        self.predecessor
    }
    pub fn occurrences(&self) -> &[DurableSignalConditionalDefinitionOccurrence] {
        self.occurrences
    }
    /// Only actual inherited installation can become target projection input.
    pub fn readmit_inherited_contract(
        &self,
        contract: &InstalledSignalConditionalContract,
    ) -> Result<SignalReadmittedConditionalContract, D> {
        if contract.bind_for_conditional_service(self.graph).is_none() {
            return Err(D::ConditionalDefinitionMismatch);
        }
        let node = DurableSignalNodeIdentity::from_recorded(
            self.branch,
            contract.node().index(),
            contract.node().generation(),
        );
        let row = self
            .occurrences
            .binary_search_by_key(&node, |row| row.node())
            .map_err(|_| D::ConditionalDefinitionMismatch)?;
        Ok(SignalReadmittedConditionalContract::new(
            self.occurrences[row],
            contract.clone(),
        ))
    }
    pub fn installation<'a>(
        &'a self,
        contracts: &'a [SignalReadmittedConditionalContract],
        section: &'a [u8],
    ) -> Result<SignalAuthoredBranchInstallation<'a>, D> {
        if contracts.len() != self.occurrences.len() {
            return Err(D::ConditionalDefinitionMismatch);
        }
        for (contract, occurrence) in contracts.iter().zip(self.occurrences) {
            if contract.occurrence() != *occurrence
                || contract
                    .installed()
                    .bind_for_conditional_service(self.graph)
                    .is_none()
                || contract.installed().node().index() != occurrence.node().index()
                || contract.installed().node().generation() != occurrence.node().generation()
            {
                return Err(D::ConditionalDefinitionMismatch);
            }
        }
        Ok(SignalAuthoredBranchInstallation::new(
            self.graph,
            self.branch,
            contracts,
            section,
        ))
    }
}

pub trait SignalDurableForkPublication: super::SignalDurableProjectionPublication {
    fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalForkPublicationContext<'_>,
    ) -> Result<SignalBridgeAuthoredSection, D>;
}

pub struct SignalOnlyForkPublication;
impl SignalDurableForkPublication for SignalOnlyForkPublication {
    fn prepare(
        &mut self,
        _: &ExecutionRequest<'_, '_>,
        context: SignalForkPublicationContext<'_>,
    ) -> Result<SignalBridgeAuthoredSection, D> {
        if !context.predecessor.bridge_bytes().is_empty() {
            return Err(D::DefinitionSectionRefused);
        }
        Ok(context.predecessor.bridge_section())
    }
}
impl super::SignalDurableProjectionPublication for SignalOnlyForkPublication {
    fn committed(self: Box<Self>, _: &crate::branch::owner_services::SignalDurableBranchHandle) {}
}
