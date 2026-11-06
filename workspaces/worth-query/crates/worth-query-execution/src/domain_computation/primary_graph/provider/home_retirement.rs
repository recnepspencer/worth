//! Retain the original provider while its live inventories and source retire.
use super::super::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use super::WorthQueryPrimaryGraphProvider;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;

pub(crate) struct PrimaryProviderCustody {
    serving: Option<Arc<WorthQueryPrimaryGraphProvider>>,
    retiring: Option<WorthQueryPrimaryGraphProvider>,
    stage: ProviderRetirementStage,
}
enum ProviderRetirementStage {
    ReceiptBases,
    Publications,
    Source,
    Complete,
}

impl PrimaryProviderCustody {
    pub(in crate::domain_computation::primary_graph) fn retained_for_teardown(
        &self,
    ) -> Option<Arc<WorthQueryPrimaryGraphProvider>> {
        self.serving.clone()
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        provider: Arc<WorthQueryPrimaryGraphProvider>,
    ) -> Self {
        Self {
            serving: Some(provider),
            retiring: None,
            stage: ProviderRetirementStage::ReceiptBases,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn preflight(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.serving
            .as_ref()
            .expect("only serving provider preflights")
            .preflight_home_retirement(request)
    }
    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        if matches!(self.stage, ProviderRetirementStage::Complete) {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if let Some(provider) = self.serving.take() {
            match Arc::try_unwrap(provider) {
                Ok(provider) => self.retiring = Some(provider),
                Err(provider) => {
                    self.serving = Some(provider);
                    return Err(Denial::Busy(Owner::GraphProvider));
                }
            }
            return Ok(Progress::Advanced);
        }
        let provider = self
            .retiring
            .as_mut()
            .expect("same original provider remains owned");
        match self.stage {
            ProviderRetirementStage::ReceiptBases => {
                let progress = provider
                    .receipt_basis_retention
                    .try_lock()
                    .map_err(|_| Denial::Busy(Owner::ReceiptBases))?
                    .advance_home_retirement(request)?;
                if progress == Progress::Complete {
                    self.stage = ProviderRetirementStage::Publications;
                }
                Ok(Progress::Advanced)
            }
            ProviderRetirementStage::Publications => {
                let progress = provider
                    .completed_commit_evidence
                    .try_lock()
                    .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?
                    .advance_home_retirement(request, native)?;
                if progress == Progress::Complete {
                    self.stage = ProviderRetirementStage::Source;
                }
                Ok(Progress::Advanced)
            }
            ProviderRetirementStage::Source => {
                let progress = provider.graph.advance_home_retirement(request, native)?;
                if progress == Progress::Complete {
                    self.stage = ProviderRetirementStage::Complete;
                }
                Ok(progress)
            }
            ProviderRetirementStage::Complete => Ok(Progress::Complete),
        }
    }
}
impl std::ops::Deref for PrimaryProviderCustody {
    type Target = Arc<WorthQueryPrimaryGraphProvider>;
    fn deref(&self) -> &Self::Target {
        self.serving
            .as_ref()
            .expect("retiring Query exposes no provider")
    }
}

impl WorthQueryPrimaryGraphProvider {
    fn preflight_home_retirement(&self, request: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        let attempts = self
            .attempts
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        if attempts.home_close_has_work() {
            return Err(Denial::Busy(Owner::GraphProvider));
        }
        drop(attempts);
        let pending = self
            .pending_application_publications
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        if pending.home_close_has_work() {
            return Err(Denial::Busy(Owner::GraphProvider));
        }
        drop(pending);
        let unpublished = self
            .unpublished_idempotency
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        if unpublished.home_close_has_work() {
            return Err(Denial::Busy(Owner::GraphProvider));
        }
        drop(unpublished);
        self.completed_commit_evidence
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?
            .preflight_home_retirement(request)?;
        self.outstanding_dispatch.preflight_home_retirement()?;
        self.receipt_basis_retention
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::ReceiptBases))?
            .preflight_home_retirement()?;
        self.graph.source_owner.preflight_home_retirement(request)
    }
}
