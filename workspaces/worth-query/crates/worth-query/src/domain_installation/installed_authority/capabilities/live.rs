use std::marker::PhantomData;

use crate::ordinary::live::{
    declare_live, WorthQueryLiveDeclaration, WorthQueryLiveDeclarationStop,
    WorthQueryLiveOpenOutcome, WorthQueryLiveOpenStop, WorthQueryManagedLiveCheckpointOutcome,
    WorthQueryManagedLiveDelivery, WorthQueryManagedLiveHandle,
    WorthQueryManagedLiveLifecycleObservation,
};
use crate::ordinary::read::{WorthQueryDeclaredReadIntent, WorthQueryReadContextDeclaration};
use crate::runtime::{
    WorthQueryLiveReadResult, WorthQueryReadBuilder, WorthQueryReadDenial, WorthQueryRuntimeError,
    WorthQueryWorkspace,
};

use super::super::{
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainCapabilityKind,
    WorthQueryInstalledDomainCapabilityStop, WorthQueryInstalledDomainExecutionDrift,
    WorthQueryInstalledDomainExecutionReceipt, WorthQueryInstalledDomainHandle,
};
use super::{
    close_outcome, WorthQueryInstalledDomainLiveCheckpointOutcome,
    WorthQueryInstalledDomainLiveCheckpointStop, WorthQueryInstalledDomainLiveCloseOutcome,
    WorthQueryInstalledDomainLiveContinuation, WorthQueryInstalledDomainProjectionOutcome,
};

pub struct WorthQueryInstalledDomainLiveDeclaration<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    declaration: WorthQueryLiveDeclaration,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainLiveDeclaration<D> {
    pub fn using(
        self,
        context: impl Into<WorthQueryReadContextDeclaration>,
    ) -> WorthQueryInstalledDomainLiveRequest<D> {
        WorthQueryInstalledDomainLiveRequest {
            witness: self.witness,
            request: self.declaration.using(context),
            marker: PhantomData,
        }
    }
}

pub struct WorthQueryInstalledDomainLiveRequest<D> {
    witness: WorthQueryInstalledDomainAuthorityWitness,
    request: crate::ordinary::live::WorthQueryLiveRequest,
    marker: PhantomData<fn() -> D>,
}

impl<D: 'static> WorthQueryInstalledDomainLiveRequest<D> {
    pub fn open(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<
        WorthQueryInstalledDomainLiveOpenOutcome<D>,
        WorthQueryInstalledDomainCapabilityStop<WorthQueryInstalledDomainExecutionDrift>,
    > {
        let declaration_identity = WorthQueryInstalledDomainExecutionReceipt::label_identity(
            "live-declaration",
            self.request.declaration_identity().as_str(),
        );
        WorthQueryInstalledDomainExecutionDrift::validate::<D>(&self.witness, workspace).map_err(
            |drift| {
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.witness.clone(),
                    WorthQueryInstalledDomainCapabilityKind::LiveOpen,
                    declaration_identity.clone(),
                    drift,
                )
            },
        )?;
        let basis_identity = workspace
            .snapshot_identity()
            .map_err(|denial| {
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.witness.clone(),
                    WorthQueryInstalledDomainCapabilityKind::LiveOpen,
                    declaration_identity.clone(),
                    WorthQueryInstalledDomainExecutionDrift::closed(&self.witness, denial),
                )
            })?
            .evidence_identity();
        Ok(match self.request.open(workspace) {
            WorthQueryLiveOpenOutcome::Opened(completion) => {
                let context_receipt_digest = completion.context_receipt().digest().to_string();
                let handle = completion.into_handle();
                let receipt = WorthQueryInstalledDomainExecutionReceipt::new_managed_live(
                    self.witness,
                    WorthQueryInstalledDomainCapabilityKind::LiveOpen,
                    declaration_identity,
                    basis_identity,
                    WorthQueryInstalledDomainExecutionReceipt::label_identity(
                        "live-context-admission",
                        &context_receipt_digest,
                    ),
                    handle.resource_identity().clone(),
                );
                WorthQueryInstalledDomainLiveOpenOutcome::Opened(
                    WorthQueryInstalledDomainLiveHandle {
                        handle,
                        receipt,
                        marker: PhantomData,
                    },
                )
            }
            WorthQueryLiveOpenOutcome::Stopped(stop) => {
                WorthQueryInstalledDomainLiveOpenOutcome::Stopped(
                    WorthQueryInstalledDomainCapabilityStop::new(
                        self.witness,
                        WorthQueryInstalledDomainCapabilityKind::LiveOpen,
                        declaration_identity,
                        stop,
                    ),
                )
            }
        })
    }
}

pub enum WorthQueryInstalledDomainLiveOpenOutcome<D> {
    Opened(WorthQueryInstalledDomainLiveHandle<D>),
    Stopped(WorthQueryInstalledDomainCapabilityStop<WorthQueryLiveOpenStop>),
}

#[must_use = "installed live handles own a Query resource until closed"]
pub struct WorthQueryInstalledDomainLiveHandle<D> {
    pub(super) handle: WorthQueryManagedLiveHandle,
    pub(super) receipt: WorthQueryInstalledDomainExecutionReceipt,
    pub(super) marker: PhantomData<fn() -> D>,
}

impl<D: 'static> WorthQueryInstalledDomainLiveHandle<D> {
    pub fn name(&self) -> &str {
        self.handle.name()
    }

    pub fn installation_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn read(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryInstalledDomainLiveRead, WorthQueryInstalledDomainLiveOperationError>
    {
        self.validate(workspace, WorthQueryInstalledDomainCapabilityKind::LiveRead)?;
        let result = self.handle.read(workspace).map_err(|error| {
            self.runtime_stop(WorthQueryInstalledDomainCapabilityKind::LiveRead, error)
        })?;
        let receipt = self.receipt.derive(
            WorthQueryInstalledDomainCapabilityKind::LiveRead,
            WorthQueryInstalledDomainExecutionReceipt::label_identity(
                "live-read-result",
                result.receipt().result_digest(),
            ),
        );
        Ok(WorthQueryInstalledDomainLiveRead { result, receipt })
    }

    pub fn project(
        &self,
        read: &WorthQueryInstalledDomainLiveRead,
        declaration: crate::ordinary::read::WorthQueryProjectionDeclaration,
    ) -> WorthQueryInstalledDomainProjectionOutcome<D> {
        let outcome = self.handle.project(read.result(), declaration);
        let receipt = self.receipt.derive(
            WorthQueryInstalledDomainCapabilityKind::Projection,
            read.receipt().operational_identity().clone(),
        );
        WorthQueryInstalledDomainProjectionOutcome::from_outcome(outcome, receipt)
    }

    pub fn drain(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryInstalledDomainLiveDelivery, WorthQueryInstalledDomainLiveOperationError>
    {
        self.validate(
            workspace,
            WorthQueryInstalledDomainCapabilityKind::LiveDelivery,
        )?;
        let delivery = self.handle.drain(workspace).map_err(|error| {
            self.runtime_stop(WorthQueryInstalledDomainCapabilityKind::LiveDelivery, error)
        })?;
        let operational = delivery
            .batches()
            .last()
            .map(|batch| batch.delivery_batch_identity().clone())
            .unwrap_or_else(|| self.receipt.operational_identity().clone());
        Ok(WorthQueryInstalledDomainLiveDelivery {
            delivery,
            receipt: self.receipt.derive(
                WorthQueryInstalledDomainCapabilityKind::LiveDelivery,
                operational,
            ),
        })
    }

    pub fn observe(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<
        WorthQueryManagedLiveLifecycleObservation,
        WorthQueryInstalledDomainLiveOperationError,
    > {
        self.validate(
            workspace,
            WorthQueryInstalledDomainCapabilityKind::LiveObservation,
        )?;
        self.handle.observe(workspace).map_err(|error| {
            self.runtime_stop(
                WorthQueryInstalledDomainCapabilityKind::LiveObservation,
                error,
            )
        })
    }

    pub fn checkpoint(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryInstalledDomainLiveCheckpointOutcome<D> {
        if let Err(drift) = WorthQueryInstalledDomainExecutionDrift::validate::<D>(
            self.receipt.installed_authority(),
            workspace,
        ) {
            return WorthQueryInstalledDomainLiveCheckpointOutcome::AuthorityStopped(self, drift);
        }
        let receipt = self.receipt;
        match self.handle.checkpoint(workspace) {
            WorthQueryManagedLiveCheckpointOutcome::Checkpointed(completion) => {
                let operational = completion.checkpoint().continuation_identity().clone();
                WorthQueryInstalledDomainLiveCheckpointOutcome::Checkpointed(
                    WorthQueryInstalledDomainLiveContinuation {
                        continuation: completion.into_continuation(),
                        receipt: receipt.derive(
                            WorthQueryInstalledDomainCapabilityKind::LiveCheckpoint,
                            operational,
                        ),
                        marker: PhantomData,
                    },
                )
            }
            WorthQueryManagedLiveCheckpointOutcome::Stopped(stop) => {
                WorthQueryInstalledDomainLiveCheckpointOutcome::RuntimeStopped(
                    WorthQueryInstalledDomainLiveCheckpointStop {
                        stop,
                        receipt,
                        marker: PhantomData,
                    },
                )
            }
        }
    }

    pub fn close(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryInstalledDomainLiveCloseOutcome<D> {
        if let Err(drift) = WorthQueryInstalledDomainExecutionDrift::validate::<D>(
            self.receipt.installed_authority(),
            workspace,
        ) {
            return WorthQueryInstalledDomainLiveCloseOutcome::AuthorityStopped(self, drift);
        }
        close_outcome(self.handle.close(workspace), self.receipt)
    }

    fn validate(
        &self,
        workspace: &WorthQueryWorkspace,
        capability: WorthQueryInstalledDomainCapabilityKind,
    ) -> Result<(), WorthQueryInstalledDomainLiveOperationError> {
        WorthQueryInstalledDomainExecutionDrift::validate::<D>(
            self.receipt.installed_authority(),
            workspace,
        )
        .map_err(|drift| {
            WorthQueryInstalledDomainLiveOperationError::Authority(
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.receipt.installed_authority().clone(),
                    capability,
                    self.receipt.declaration_identity().clone(),
                    drift,
                ),
            )
        })
    }

    fn runtime_stop(
        &self,
        capability: WorthQueryInstalledDomainCapabilityKind,
        error: WorthQueryRuntimeError,
    ) -> WorthQueryInstalledDomainLiveOperationError {
        WorthQueryInstalledDomainLiveOperationError::Runtime(
            WorthQueryInstalledDomainCapabilityStop::new(
                self.receipt.installed_authority().clone(),
                capability,
                self.receipt.declaration_identity().clone(),
                error,
            ),
        )
    }
}

pub enum WorthQueryInstalledDomainLiveOperationError {
    Authority(WorthQueryInstalledDomainCapabilityStop<WorthQueryInstalledDomainExecutionDrift>),
    Runtime(WorthQueryInstalledDomainCapabilityStop<WorthQueryRuntimeError>),
}

pub struct WorthQueryInstalledDomainLiveRead {
    result: WorthQueryLiveReadResult,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl WorthQueryInstalledDomainLiveRead {
    pub fn result(&self) -> &WorthQueryLiveReadResult {
        &self.result
    }
    pub fn receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }
}

pub struct WorthQueryInstalledDomainLiveDelivery {
    delivery: WorthQueryManagedLiveDelivery,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl WorthQueryInstalledDomainLiveDelivery {
    pub fn delivery(&self) -> &WorthQueryManagedLiveDelivery {
        &self.delivery
    }
    pub fn receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }
}

impl<D> WorthQueryInstalledDomainHandle<D> {
    pub fn live(
        &self,
        name: impl Into<String>,
        author: impl FnOnce(
            WorthQueryReadBuilder<WorthQueryDeclaredReadIntent>,
        ) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial>,
    ) -> Result<WorthQueryInstalledDomainLiveDeclaration<D>, WorthQueryLiveDeclarationStop> {
        declare_live(name, author).map(|declaration| WorthQueryInstalledDomainLiveDeclaration {
            witness: self.authority_witness(),
            declaration,
            marker: PhantomData,
        })
    }
}
