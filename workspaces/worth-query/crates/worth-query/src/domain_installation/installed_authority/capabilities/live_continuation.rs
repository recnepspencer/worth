use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::marker::PhantomData;

use crate::ordinary::live::{
    WorthQueryManagedLiveCheckpointStop, WorthQueryManagedLiveCloseOutcome,
    WorthQueryManagedLiveCloseReceipt, WorthQueryManagedLiveCloseStop,
    WorthQueryManagedLiveContinuation, WorthQueryManagedLiveResumeOutcome,
    WorthQueryManagedLiveResumeReceipt, WorthQueryManagedLiveResumeStop,
};
use crate::runtime::WorthQueryWorkspace;

use super::super::{
    WorthQueryInstalledDomainCapabilityKind, WorthQueryInstalledDomainExecutionDrift,
    WorthQueryInstalledDomainExecutionReceipt,
};
use super::WorthQueryInstalledDomainLiveHandle;

pub enum WorthQueryInstalledDomainLiveCheckpointOutcome<D> {
    Checkpointed(WorthQueryInstalledDomainLiveContinuation<D>),
    RuntimeStopped(WorthQueryInstalledDomainLiveCheckpointStop<D>),
    AuthorityStopped(
        WorthQueryInstalledDomainLiveHandle<D>,
        WorthQueryInstalledDomainExecutionDrift,
    ),
}

pub struct WorthQueryInstalledDomainLiveCheckpointStop<D> {
    pub(super) stop: WorthQueryManagedLiveCheckpointStop,
    pub(super) receipt: WorthQueryInstalledDomainExecutionReceipt,
    pub(super) marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainLiveCheckpointStop<D> {
    pub fn installation_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn error(&self) -> &crate::runtime::WorthQueryRuntimeError {
        self.stop.error()
    }

    pub fn into_handle(self) -> WorthQueryInstalledDomainLiveHandle<D> {
        WorthQueryInstalledDomainLiveHandle {
            handle: self.stop.into_handle(),
            receipt: self.receipt,
            marker: PhantomData,
        }
    }
}

#[must_use = "installed live continuations retain a Query resource until resumed or dropped"]
pub struct WorthQueryInstalledDomainLiveContinuation<D> {
    pub(super) continuation: WorthQueryManagedLiveContinuation,
    pub(super) receipt: WorthQueryInstalledDomainExecutionReceipt,
    pub(super) marker: PhantomData<fn() -> D>,
}

impl<D: 'static> WorthQueryInstalledDomainLiveContinuation<D> {
    pub fn checkpoint_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn resume(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryInstalledDomainLiveResumeOutcome<D>, IdentityDerivationDenial> {
        if let Err(drift) = WorthQueryInstalledDomainExecutionDrift::validate::<D>(
            self.receipt.installed_authority(),
            workspace,
        ) {
            return Ok(WorthQueryInstalledDomainLiveResumeOutcome::AuthorityStopped(self, drift));
        }
        let receipt = self.receipt;
        Ok(match self.continuation.resume(workspace)? {
            WorthQueryManagedLiveResumeOutcome::Resumed(completion) => {
                let resume_receipt = completion.receipt().clone();
                let execution_receipt = receipt.derive(
                    WorthQueryInstalledDomainCapabilityKind::LiveResume,
                    resume_receipt.resume_identity().clone(),
                );
                WorthQueryInstalledDomainLiveResumeOutcome::Resumed(
                    WorthQueryInstalledDomainLiveResumeCompletion {
                        handle: WorthQueryInstalledDomainLiveHandle {
                            handle: completion.into_handle(),
                            receipt: execution_receipt.clone(),
                            marker: PhantomData,
                        },
                        resume_receipt,
                        execution_receipt,
                    },
                )
            }
            WorthQueryManagedLiveResumeOutcome::Stopped(stop) => {
                WorthQueryInstalledDomainLiveResumeOutcome::RuntimeStopped(
                    WorthQueryInstalledDomainLiveResumeStop {
                        stop,
                        receipt,
                        marker: PhantomData,
                    },
                )
            }
        })?
    }
}

pub enum WorthQueryInstalledDomainLiveResumeOutcome<D> {
    Resumed(WorthQueryInstalledDomainLiveResumeCompletion<D>),
    RuntimeStopped(WorthQueryInstalledDomainLiveResumeStop<D>),
    AuthorityStopped(
        WorthQueryInstalledDomainLiveContinuation<D>,
        WorthQueryInstalledDomainExecutionDrift,
    ),
}

pub struct WorthQueryInstalledDomainLiveResumeCompletion<D> {
    handle: WorthQueryInstalledDomainLiveHandle<D>,
    resume_receipt: WorthQueryManagedLiveResumeReceipt,
    execution_receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl<D> WorthQueryInstalledDomainLiveResumeCompletion<D> {
    pub fn handle(&self) -> &WorthQueryInstalledDomainLiveHandle<D> {
        &self.handle
    }
    pub fn resume_receipt(&self) -> &WorthQueryManagedLiveResumeReceipt {
        &self.resume_receipt
    }
    pub fn execution_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.execution_receipt
    }
    pub fn into_handle(self) -> WorthQueryInstalledDomainLiveHandle<D> {
        self.handle
    }
}

pub struct WorthQueryInstalledDomainLiveResumeStop<D> {
    stop: WorthQueryManagedLiveResumeStop,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainLiveResumeStop<D> {
    pub fn installation_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn stop(&self) -> &WorthQueryManagedLiveResumeStop {
        &self.stop
    }
    pub fn into_continuation(self) -> WorthQueryInstalledDomainLiveContinuation<D> {
        WorthQueryInstalledDomainLiveContinuation {
            continuation: self.stop.into_continuation(),
            receipt: self.receipt,
            marker: PhantomData,
        }
    }
}

pub enum WorthQueryInstalledDomainLiveCloseOutcome<D> {
    Closed(WorthQueryInstalledDomainLiveCloseReceipt),
    RuntimeStopped(WorthQueryInstalledDomainLiveCloseStop<D>),
    AuthorityStopped(
        WorthQueryInstalledDomainLiveHandle<D>,
        WorthQueryInstalledDomainExecutionDrift,
    ),
}

pub struct WorthQueryInstalledDomainLiveCloseReceipt {
    close_receipt: WorthQueryManagedLiveCloseReceipt,
    execution_receipt: WorthQueryInstalledDomainExecutionReceipt,
}

impl WorthQueryInstalledDomainLiveCloseReceipt {
    pub fn close_receipt(&self) -> &WorthQueryManagedLiveCloseReceipt {
        &self.close_receipt
    }
    pub fn execution_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.execution_receipt
    }
}

pub struct WorthQueryInstalledDomainLiveCloseStop<D> {
    stop: WorthQueryManagedLiveCloseStop,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainLiveCloseStop<D> {
    pub fn installation_receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn error(&self) -> &crate::runtime::WorthQueryRuntimeError {
        self.stop.error()
    }
    pub fn into_handle(self) -> WorthQueryInstalledDomainLiveHandle<D> {
        WorthQueryInstalledDomainLiveHandle {
            handle: self.stop.into_handle(),
            receipt: self.receipt,
            marker: PhantomData,
        }
    }
}

pub(super) fn close_outcome<D>(
    outcome: WorthQueryManagedLiveCloseOutcome,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
) -> WorthQueryInstalledDomainLiveCloseOutcome<D> {
    match outcome {
        WorthQueryManagedLiveCloseOutcome::Closed(close_receipt) => {
            let execution_receipt = receipt.derive(
                WorthQueryInstalledDomainCapabilityKind::LiveClose,
                close_receipt.closeout_identity().clone(),
            );
            WorthQueryInstalledDomainLiveCloseOutcome::Closed(
                WorthQueryInstalledDomainLiveCloseReceipt {
                    close_receipt,
                    execution_receipt,
                },
            )
        }
        WorthQueryManagedLiveCloseOutcome::Stopped(stop) => {
            WorthQueryInstalledDomainLiveCloseOutcome::RuntimeStopped(
                WorthQueryInstalledDomainLiveCloseStop {
                    stop,
                    receipt,
                    marker: PhantomData,
                },
            )
        }
    }
}
