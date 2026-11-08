//! Another owner's authored section follows the original Signal publication task.
use super::{SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact};
use crate::branch::owner_services::{SignalDurableAdvanceDenial, SignalDurableBranchHandle};
use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::conditional_execution::{
    InstalledSignalConditionalContract, SignalConditionalContractDefinition,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalConditionalDefinitionOccurrence, SignalBranchAdvanceIdentity,
    SignalDefinitionTransitionIdentity,
};

/// Actual issued identities and the contract installed on the isolated candidate.
pub struct SignalDefinitionPublicationContext<'a> {
    pub(crate) original: SignalBranchAdvanceIdentity,
    pub(crate) identity: SignalDefinitionTransitionIdentity,
    pub(crate) occurrence: DurableSignalConditionalDefinitionOccurrence,
    pub(crate) predecessor: &'a SignalDefinitionTransitionArtifact,
    pub(crate) contract: &'a InstalledSignalConditionalContract,
    pub(crate) custody: &'a crate::branch::owner_services::conditional_execution::SignalConditionalInstallationCustody,
}
impl SignalDefinitionPublicationContext<'_> {
    pub fn custody(
        &self,
    ) -> &crate::branch::owner_services::conditional_execution::SignalConditionalInstallationCustody
    {
        self.custody
    }
    pub fn original(&self) -> SignalBranchAdvanceIdentity {
        self.original
    }
    pub fn identity(&self) -> SignalDefinitionTransitionIdentity {
        self.identity
    }
    pub fn occurrence(&self) -> DurableSignalConditionalDefinitionOccurrence {
        self.occurrence
    }
    pub fn predecessor(&self) -> &SignalDefinitionTransitionArtifact {
        self.predecessor
    }
    pub fn contract(&self) -> &InstalledSignalConditionalContract {
        self.contract
    }
}

/// Prepare all fallible derived work before media publication. The original task
/// retains this object across caller drop/resume and consumes it after cutover.
pub trait SignalDurableDefinitionPublication: super::SignalDurableProjectionPublication {
    fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalDefinitionPublicationContext<'_>,
    ) -> Result<SignalBridgeAuthoredSection, SignalDurableAdvanceDenial>;
}

/// Signal-only publication cannot silently carry another owner's stale section.
pub struct SignalOnlyDefinitionPublication;
impl SignalDurableDefinitionPublication for SignalOnlyDefinitionPublication {
    fn prepare(
        &mut self,
        _: &ExecutionRequest<'_, '_>,
        context: SignalDefinitionPublicationContext<'_>,
    ) -> Result<SignalBridgeAuthoredSection, SignalDurableAdvanceDenial> {
        if !context.predecessor().bridge_bytes().is_empty() {
            return Err(SignalDurableAdvanceDenial::DefinitionSectionRefused);
        }
        Ok(context.predecessor().bridge_section())
    }
}

pub(crate) struct SignalDurableConditionalDefinitionChange {
    pub(crate) claimant: SignalAspectLoweringOwner,
    pub(crate) predecessor: InstalledSignalConditionalContract,
    pub(crate) definition: SignalConditionalContractDefinition,
    pub(crate) publication: Box<dyn SignalDurableDefinitionPublication>,
}

impl super::SignalDurableProjectionPublication for SignalOnlyDefinitionPublication {
    fn committed(self: Box<Self>, _: &SignalDurableBranchHandle) {}
}
