use std::marker::PhantomData;

use crate::ordinary::workflow::{
    WorthQueryWorkflowContext, WorthQueryWorkflowDeclaration, WorthQueryWorkflowOutcome,
    WorthQueryWorkflowStop,
};
use crate::runtime::WorthQueryWorkspace;

use super::super::{
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainCapabilityKind,
    WorthQueryInstalledDomainCapabilityStop, WorthQueryInstalledDomainExecutionDrift,
    WorthQueryInstalledDomainExecutionReceipt,
};

pub struct WorthQueryInstalledDomainWorkflowDeclaration<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    declaration: WorthQueryWorkflowDeclaration,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainWorkflowDeclaration<D> {
    pub(super) fn new(
        witness: WorthQueryInstalledDomainAuthorityWitness,
        declaration: WorthQueryWorkflowDeclaration,
    ) -> Self {
        Self {
            witness,
            declaration,
            marker: PhantomData,
        }
    }

    pub fn with_rich_inspection(mut self) -> Self {
        self.declaration = self.declaration.with_rich_inspection();
        self
    }

    pub fn using(
        self,
        context: WorthQueryWorkflowContext,
    ) -> WorthQueryInstalledDomainWorkflowRequest<D> {
        let declaration_identity = self.declaration.identity().evidence_identity().clone();
        WorthQueryInstalledDomainWorkflowRequest {
            witness: self.witness,
            declaration_identity,
            request: self.declaration.using(context),
            marker: PhantomData,
        }
    }
}

pub struct WorthQueryInstalledDomainWorkflowRequest<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    declaration_identity: crate::WorthQueryEvidenceIdentity,
    request: crate::ordinary::workflow::WorthQueryWorkflowRequest,
    marker: PhantomData<fn() -> D>,
}

impl<D: 'static> WorthQueryInstalledDomainWorkflowRequest<D> {
    pub fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<
        WorthQueryInstalledDomainWorkflowOutcome,
        WorthQueryInstalledDomainCapabilityStop<WorthQueryInstalledDomainExecutionDrift>,
    > {
        WorthQueryInstalledDomainExecutionDrift::validate::<D>(&self.witness, workspace).map_err(
            |drift| {
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.witness.clone(),
                    WorthQueryInstalledDomainCapabilityKind::Workflow,
                    self.declaration_identity.clone(),
                    drift,
                )
            },
        )?;
        Ok(match self.request.run(execution, workspace) {
            WorthQueryWorkflowOutcome::Completed(completion) => {
                let receipt = WorthQueryInstalledDomainExecutionReceipt::new(
                    self.witness,
                    WorthQueryInstalledDomainCapabilityKind::Workflow,
                    self.declaration_identity,
                    completion
                        .promotion_eligibility()
                        .snapshot_identity()
                        .evidence_identity(),
                    completion.aftermath().receipt_identity().clone(),
                );
                WorthQueryInstalledDomainWorkflowOutcome::Completed(
                    WorthQueryInstalledDomainWorkflowCompletion {
                        completion,
                        receipt,
                    },
                )
            }
            WorthQueryWorkflowOutcome::Stopped(stop) => {
                WorthQueryInstalledDomainWorkflowOutcome::Stopped(
                    WorthQueryInstalledDomainCapabilityStop::new(
                        self.witness,
                        WorthQueryInstalledDomainCapabilityKind::Workflow,
                        self.declaration_identity,
                        stop,
                    ),
                )
            }
        })
    }
}

pub enum WorthQueryInstalledDomainWorkflowOutcome {
    Completed(WorthQueryInstalledDomainWorkflowCompletion),
    Stopped(WorthQueryInstalledDomainCapabilityStop<WorthQueryWorkflowStop>),
}

impl WorthQueryInstalledDomainWorkflowOutcome {
    pub fn completed(&self) -> Option<&WorthQueryInstalledDomainWorkflowCompletion> {
        match self {
            Self::Completed(completion) => Some(completion),
            Self::Stopped(_) => None,
        }
    }

    pub fn stop(&self) -> Option<&WorthQueryInstalledDomainCapabilityStop<WorthQueryWorkflowStop>> {
        match self {
            Self::Completed(_) => None,
            Self::Stopped(stop) => Some(stop),
        }
    }
}

pub struct WorthQueryInstalledDomainWorkflowCompletion {
    completion: crate::ordinary::workflow::WorthQueryWorkflowCompletion,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl WorthQueryInstalledDomainWorkflowCompletion {
    pub fn completion(&self) -> &crate::ordinary::workflow::WorthQueryWorkflowCompletion {
        &self.completion
    }

    pub fn receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }
}
