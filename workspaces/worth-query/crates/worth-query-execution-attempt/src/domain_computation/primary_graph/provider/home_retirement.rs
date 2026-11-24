//! Retain the original provider while its live inventories and source retire.
use super::super::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCloseCustody;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationLandedCommit,
    output_lineage::CoupledOutputCompletion,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_relational::facade::runtime::RelationalRuntime;

pub struct PrimaryProviderCustody {
    serving: Option<Arc<WorthQueryPrimaryGraphProvider>>,
    retiring: Option<WorthQueryPrimaryGraphProvider>,
    stage: ProviderRetirementStage,
    /// What the latest admitted preflight proved about dispatch custody.
    dispatch_custody: QueryDispatchCloseCustody,
    /// Coupled-completion receipts detached during preflight and held in
    /// prepared-close custody until World seals or serving resumes.
    detached_coupled: Vec<(CoupledOutputCompletion, WorthQueryApplicationLandedCommit)>,
}
enum ProviderRetirementStage {
    OutstandingSources,
    TerminalSources,
    ReceiptBases,
    Publications,
    CorrectionMaterial,
    Source,
    Complete,
}

impl PrimaryProviderCustody {
    pub fn retained_for_teardown(&self) -> Option<Arc<WorthQueryPrimaryGraphProvider>> {
        self.serving.clone()
    }

    pub fn new(provider: Arc<WorthQueryPrimaryGraphProvider>) -> Self {
        Self {
            serving: Some(provider),
            retiring: None,
            stage: ProviderRetirementStage::OutstandingSources,
            dispatch_custody: QueryDispatchCloseCustody::ProcessLocal,
            detached_coupled: Vec::new(),
        }
    }

    /// Restore coupled-completion receipts if World refuses to seal and this
    /// Home returns to live service.
    pub fn restore_after_refused_home_retirement(&mut self) {
        for (completion, landed) in self.detached_coupled.drain(..) {
            completion.restore_receipt_after_refused_home_retirement(landed);
        }
    }

    pub fn preflight(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        dispatch_custody: QueryDispatchCloseCustody,
        transport: &crate::domain_computation::primary_graph::application_runtime::inbound_publication::InstalledTransportCloseCustody,
    ) -> Result<(), Denial> {
        self.restore_after_refused_home_retirement();
        self.dispatch_custody = QueryDispatchCloseCustody::ProcessLocal;
        let detached = self
            .serving
            .as_ref()
            .expect("only serving provider preflights")
            .preflight_home_retirement(request, &dispatch_custody, transport)?;
        self.detached_coupled = detached;
        self.dispatch_custody = dispatch_custody;
        Ok(())
    }
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        if matches!(self.stage, ProviderRetirementStage::Complete) {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if !self.detached_coupled.is_empty() {
            self.detached_coupled.clear();
        }
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
            ProviderRetirementStage::OutstandingSources => {
                if provider
                    .outstanding_dispatch
                    .advance_source_retirement(request)?
                    == Progress::Complete
                {
                    self.stage = ProviderRetirementStage::TerminalSources;
                }
                Ok(Progress::Advanced)
            }
            ProviderRetirementStage::TerminalSources => {
                if provider
                    .inbound_terminal_index
                    .advance_source_retirement(request)?
                    == Progress::Complete
                {
                    self.stage = ProviderRetirementStage::ReceiptBases;
                }
                Ok(Progress::Advanced)
            }
            ProviderRetirementStage::ReceiptBases => {
                let progress = provider
                    .receipt_basis_retention
                    .try_lock()
                    .map_err(|_| Denial::Busy(Owner::ReceiptBases))?
                    .advance_home_retirement(request, &self.dispatch_custody)?;
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
                    self.stage = ProviderRetirementStage::CorrectionMaterial;
                }
                Ok(Progress::Advanced)
            }
            ProviderRetirementStage::CorrectionMaterial => {
                provider
                    .settle_expired_correction_meanings(request)
                    .map_err(Denial::CorrectionRetention)?;
                if let Some(owner) = provider.correction_material.get_mut() {
                    owner
                        .finish_retirement(request)
                        .map_err(Denial::CorrectionMaterial)?;
                }
                self.stage = ProviderRetirementStage::Source;
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
    fn preflight_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        dispatch_custody: &QueryDispatchCloseCustody,
        transport: &crate::domain_computation::primary_graph::application_runtime::inbound_publication::InstalledTransportCloseCustody,
    ) -> Result<Vec<(CoupledOutputCompletion, WorthQueryApplicationLandedCommit)>, Denial> {
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
        let detached_coupled = {
            let lineage = self
                .graph
                .output_lineage_ref()
                .try_lock()
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
            if lineage.has_pending_coupled_outputs() {
                return Err(Denial::Busy(Owner::InstalledProducers));
            }
            lineage
                .detach_coupled_completion_receipts_for_home_retirement()
                .map_err(|()| Denial::Busy(Owner::PublicationReceipts))?
        };
        let result = (|| {
            self.completed_commit_evidence
                .try_lock()
                .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?
                .preflight_home_retirement(request)?;
            self.outstanding_dispatch
                .preflight_home_retirement(dispatch_custody, transport)?;
            self.receipt_basis_retention
                .try_lock()
                .map_err(|_| Denial::Busy(Owner::ReceiptBases))?
                .preflight_home_retirement(dispatch_custody)?;
            self.graph
                .source_owner_ref()
                .preflight_home_retirement(request)
                .map_err(Into::into)
        })();
        match result {
            Ok(()) => Ok(detached_coupled),
            Err(cause) => {
                for (completion, landed) in detached_coupled {
                    completion.restore_receipt_after_refused_home_retirement(landed);
                }
                Err(cause)
            }
        }
    }
}
