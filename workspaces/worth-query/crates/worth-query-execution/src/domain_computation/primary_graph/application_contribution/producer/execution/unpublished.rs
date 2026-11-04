//! The original producer request survives an unpublished World result.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;
use crate::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecovery,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use std::any::Any;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::read_cleanup::WorthQueryOutputReadCloseDenial,
    crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryPerformed,
};
mod input;
mod recovery;
pub(super) use input::prepare_input;
pub(in crate::domain_computation::primary_graph) use recovery::RecoveredProducerPublication;
type Original = Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication>>;

/// Both allocations precede effects. None is exposed only through this prepared
/// type; only an actual unpublished result can initialize its continuation.
pub(in crate::domain_computation::primary_graph) struct PreparedProducerPublication {
    input: Box<dyn Any + Send + Sync>,
    slot: Box<Option<ContinuationState>>,
    edition: super::super::InstalledProducerEdition,
    mode: WorthQueryProducerCommitAuthority,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Fixed-size registry custody for the original preallocated cold cell.
pub(in crate::domain_computation::primary_graph) struct PendingProducerPublication {
    slot: Box<Option<ContinuationState>>,
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution)
struct ContinuationState
{
    input: Option<Box<dyn Any + Send + Sync>>,
    edition: super::super::InstalledProducerEdition,
    mode: WorthQueryProducerCommitAuthority,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    state: PublicationState,
    prior_cleanup: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    original_publication: Original,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

enum PublicationState {
    Unpublished(WorthQueryProductUnpublishedRecovery),
    Performed {
        outcome: WorthQueryManagedApplicationRecoveryPerformed,
        carrier: Option<WorthQueryRecoveredProgramOutputSource>,
    },
    Closing,
    Closed,
}

impl PreparedProducerPublication {
    pub(super) fn unpublished(
        self,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        original: WorthQueryProductUnpublishedApplication,
    ) -> PendingProducerPublication {
        let Self {
            input,
            mut slot,
            edition,
            mode,
            memory,
        } = self;
        *slot = Some(ContinuationState {
            input: Some(input),
            edition,
            mode,
            idempotency,
            state: PublicationState::Unpublished(original.into_recovery()),
            prior_cleanup: None,
            original_publication: None,
            _memory: memory,
        });
        PendingProducerPublication { slot }
    }
}

impl PendingProducerPublication {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn state_mut(
        &mut self,
    ) -> &mut ContinuationState {
        self.slot
            .as_mut()
            .as_mut()
            .expect("only actual unpublished outcomes issue this owner")
    }
    fn state(&self) -> &ContinuationState {
        self.slot
            .as_ref()
            .as_ref()
            .expect("only actual unpublished outcomes issue this owner")
    }
    pub(in crate::domain_computation::primary_graph) fn retain_original_publication(
        &mut self,
        original: Original,
    ) {
        self.state_mut().original_publication = original;
    }
    pub(in crate::domain_computation::primary_graph) fn is_closed(&self) -> bool {
        let state = self.state();
        matches!(state.state, PublicationState::Closed) && state.prior_cleanup.is_none()
    }
    pub(in crate::domain_computation::primary_graph) fn publication(
        &self,
    ) -> Option<&WorthQueryCommittedProductPublication> {
        match &self.state().state {
            PublicationState::Performed {
                carrier: Some(carrier),
                ..
            } => Some(carrier.publication()),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn close<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        self.state_mut().close(phase, runtime)
    }
}

impl ContinuationState {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn admits_selected(
        &self,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match &self.state {
            PublicationState::Unpublished(recovery) => recovery
                .inspect()
                .map(|original| original.expected_product() == selected)
                .map_err(|stop| {
                    denial(
                        WorthQueryOutputDemandDenialKind::WorldRecovery(stop),
                        "producer publication",
                    )
                }),
            PublicationState::Performed {
                carrier: Some(carrier),
                ..
            } => Ok(carrier.product_branch().occurrence() == selected.lifecycle_incarnation()),
            _ => Ok(false),
        }
    }

    fn close<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        self.close_prior(phase, runtime)?;
        if matches!(self.state, PublicationState::Closing) {
            self.finish_close();
            return Ok(());
        }
        let prior = match &mut self.state {
            PublicationState::Performed { outcome, .. } => outcome.take_original_cleanup_failure(),
            _ => None,
        };
        if let Some(prior) = prior {
            self.prior_cleanup = Some(prior);
            self.close_prior(phase, runtime)?;
        }
        let recovery = match &mut self.state {
            PublicationState::Unpublished(recovery) => Some(recovery.clone()),
            PublicationState::Performed { outcome, .. } => outcome.take_original_cleanup(),
            PublicationState::Closed | PublicationState::Closing => None,
        };
        if let Some(recovery) = recovery
            && let Err(failure) =
                runtime.close_product_publication_recovery_in_advancement(phase, recovery)
        {
            let denial = close_failure(&failure);
            self.prior_cleanup = Some(failure);
            self.state = PublicationState::Closing;
            return Err(denial);
        }
        self.finish_close();
        Ok(())
    }

    fn finish_close(&mut self) {
        self.state = PublicationState::Closed;
        // Called while the checkpoint is lent outside registry guards.
        drop(self.input.take());
    }

    fn close_prior<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        let Some(failure) = self.prior_cleanup.take() else {
            return Ok(());
        };
        use WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;
        let result = match failure {
            Failure::Recovery(failure) => runtime
                .close_product_publication_recovery_in_advancement(phase, failure.into_recovery()),
            Failure::OwnerCleanup(failure) => failure
                .into_cleanup()
                .retry_in_advancement(phase)
                .map_err(Failure::OwnerCleanup),
        };
        match result {
            Ok(_) => Ok(()),
            Err(failure) => {
                let denial = close_failure(&failure);
                self.prior_cleanup = Some(failure);
                Err(denial)
            }
        }
    }
}

fn close_failure(
    failure: &WorthQueryProductUnpublishedRecoveryReleaseFailure,
) -> WorthQueryOutputReadCloseDenial {
    match failure {
        WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure) => {
            WorthQueryOutputReadCloseDenial::Recovery(failure.denial())
        }
        WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failure) => {
            WorthQueryOutputReadCloseDenial::Owner(failure.denial())
        }
    }
}
