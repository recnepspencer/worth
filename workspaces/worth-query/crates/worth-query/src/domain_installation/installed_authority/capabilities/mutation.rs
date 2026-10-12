use std::marker::PhantomData;

use crate::ordinary::mutation::{
    declare as declare_mutation, WorthQueryMutationContext, WorthQueryMutationDeclaration,
    WorthQueryMutationDeclarationStop, WorthQueryMutationOutcome, WorthQueryMutationStop,
};
use crate::ordinary::workflow::declare as declare_workflow;
use crate::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryRuntimeError, WorthQueryWorkspace,
    WorthQueryWriteCommand,
};
use crate::session_label::WorthQuerySessionLabel;

use super::super::{
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainCapabilityKind,
    WorthQueryInstalledDomainCapabilityStop, WorthQueryInstalledDomainExecutionDrift,
    WorthQueryInstalledDomainExecutionReceipt, WorthQueryInstalledDomainHandle,
};
use super::workflow::WorthQueryInstalledDomainWorkflowDeclaration;

pub struct WorthQueryInstalledDomainMutationDeclaration<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    declaration: WorthQueryMutationDeclaration,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainMutationDeclaration<D> {
    pub fn with_rich_inspection(mut self) -> Self {
        self.declaration = self.declaration.with_rich_inspection();
        self
    }

    pub fn using(
        self,
        context: WorthQueryMutationContext,
    ) -> WorthQueryInstalledDomainMutationRequest<D> {
        let declaration_identity = self.declaration.identity().evidence_identity().clone();
        WorthQueryInstalledDomainMutationRequest {
            witness: self.witness,
            declaration_identity,
            request: self.declaration.using(context),
            marker: PhantomData,
        }
    }

    pub fn workflow(
        self,
        label: WorthQuerySessionLabel,
    ) -> WorthQueryInstalledDomainWorkflowDeclaration<D> {
        WorthQueryInstalledDomainWorkflowDeclaration::new(
            self.witness,
            declare_workflow(label, self.declaration),
        )
    }
}

pub struct WorthQueryInstalledDomainMutationRequest<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    declaration_identity: crate::WorthQueryEvidenceIdentity,
    request: crate::ordinary::mutation::WorthQueryMutationRequest,
    marker: PhantomData<fn() -> D>,
}

impl<D: 'static> WorthQueryInstalledDomainMutationRequest<D> {
    pub fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<
        WorthQueryInstalledDomainMutationOutcome,
        WorthQueryInstalledDomainCapabilityStop<WorthQueryInstalledDomainExecutionDrift>,
    > {
        WorthQueryInstalledDomainExecutionDrift::validate::<D>(&self.witness, workspace).map_err(
            |drift| {
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.witness.clone(),
                    WorthQueryInstalledDomainCapabilityKind::Mutation,
                    self.declaration_identity.clone(),
                    drift,
                )
            },
        )?;
        Ok(match self.request.run(execution, workspace) {
            WorthQueryMutationOutcome::Completed(completion) => {
                let receipt = WorthQueryInstalledDomainExecutionReceipt::new(
                    self.witness,
                    WorthQueryInstalledDomainCapabilityKind::Mutation,
                    self.declaration_identity,
                    completion.receipt().snapshot_evidence_identity().clone(),
                    completion.receipt().commit_evidence_identity().clone(),
                );
                WorthQueryInstalledDomainMutationOutcome::Completed(
                    WorthQueryInstalledDomainMutationCompletion {
                        completion,
                        receipt,
                    },
                )
            }
            WorthQueryMutationOutcome::Stopped(stop) => {
                WorthQueryInstalledDomainMutationOutcome::Stopped(
                    WorthQueryInstalledDomainCapabilityStop::new(
                        self.witness,
                        WorthQueryInstalledDomainCapabilityKind::Mutation,
                        self.declaration_identity,
                        stop,
                    ),
                )
            }
        })
    }
}

pub enum WorthQueryInstalledDomainMutationOutcome {
    Completed(WorthQueryInstalledDomainMutationCompletion),
    Stopped(WorthQueryInstalledDomainCapabilityStop<WorthQueryMutationStop>),
}

impl WorthQueryInstalledDomainMutationOutcome {
    pub fn completed(&self) -> Option<&WorthQueryInstalledDomainMutationCompletion> {
        match self {
            Self::Completed(completion) => Some(completion),
            Self::Stopped(_) => None,
        }
    }

    pub fn stop(&self) -> Option<&WorthQueryInstalledDomainCapabilityStop<WorthQueryMutationStop>> {
        match self {
            Self::Completed(_) => None,
            Self::Stopped(stop) => Some(stop),
        }
    }
}

pub struct WorthQueryInstalledDomainMutationCompletion {
    completion: crate::ordinary::mutation::WorthQueryMutationCompletion,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl WorthQueryInstalledDomainMutationCompletion {
    pub fn completion(&self) -> &crate::ordinary::mutation::WorthQueryMutationCompletion {
        &self.completion
    }

    pub fn receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }
}

impl<D> WorthQueryInstalledDomainHandle<D> {
    pub fn mutation<E>(
        &self,
        author: impl FnOnce(WorthQueryAspectMutationBuilder) -> Result<WorthQueryWriteCommand, E>,
    ) -> Result<WorthQueryInstalledDomainMutationDeclaration<D>, WorthQueryMutationDeclarationStop>
    where
        E: Into<Box<WorthQueryRuntimeError>>,
    {
        declare_mutation(author).map(|declaration| WorthQueryInstalledDomainMutationDeclaration {
            witness: self.authority_witness(),
            declaration,
            marker: PhantomData,
        })
    }
}
