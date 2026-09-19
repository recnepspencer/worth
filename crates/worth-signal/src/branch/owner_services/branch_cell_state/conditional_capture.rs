use super::SignalBranchCellState;
use crate::branch::owner_services::conditional_execution::{
    SignalConditionalBasisCaptureDenial, SignalConditionalServiceIssuanceDenial,
    SignalRetainedExecutionBasis,
};
use crate::data::retained_storage::{RetainedStoragePreparation, SignalConditionalRetentionLedger};
use std::sync::Arc;

pub(super) struct SignalPublishedConditionalExecutionBasis {
    generation: u64,
    basis: Arc<SignalRetainedExecutionBasis>,
}

impl SignalPublishedConditionalExecutionBasis {
    fn new(generation: u64, basis: Arc<SignalRetainedExecutionBasis>) -> Self {
        Self { generation, basis }
    }

    fn snapshot(&self, generation: u64) -> Option<Arc<SignalRetainedExecutionBasis>> {
        (self.generation == generation).then(|| Arc::clone(&self.basis))
    }
}

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Caller holds the exact admitted cell after definition claimant validation.
    pub(in crate::branch::owner_services) fn capture_conditional_basis(
        &mut self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
    ) -> Result<Arc<SignalRetainedExecutionBasis>, SignalConditionalServiceIssuanceDenial> {
        if let Some(current) = self.current_conditional_basis() {
            return Ok(current);
        }
        let basis = self
            .capture_current_conditional_basis(ledger)
            .map_err(map_issuance_capture_denial)?;
        self.publish_conditional_basis(Arc::clone(&basis));
        Ok(basis)
    }

    pub(in crate::branch::owner_services) fn current_conditional_basis(
        &self,
    ) -> Option<Arc<SignalRetainedExecutionBasis>> {
        self.conditional_execution_basis
            .as_ref()
            .and_then(|published| published.snapshot(self.head_generation))
    }

    pub(in crate::branch::owner_services) fn capture_conditional_basis_from_graph(
        &self,
        graph: &mut crate::data::graph::SignalGraph,
        ledger: &Arc<SignalConditionalRetentionLedger>,
    ) -> Result<Arc<SignalRetainedExecutionBasis>, SignalConditionalBasisCaptureDenial> {
        let maximum_visits = graph
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        SignalRetainedExecutionBasis::capture(
            graph,
            ledger,
            &mut RetainedStoragePreparation::new(maximum_visits),
        )
        .map(Arc::new)
    }

    pub(in crate::branch::owner_services) fn publish_conditional_basis(
        &mut self,
        basis: Arc<SignalRetainedExecutionBasis>,
    ) {
        self.conditional_execution_basis = Some(SignalPublishedConditionalExecutionBasis::new(
            self.head_generation,
            basis,
        ));
    }

    fn capture_current_conditional_basis(
        &mut self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
    ) -> Result<Arc<SignalRetainedExecutionBasis>, SignalConditionalBasisCaptureDenial> {
        let maximum_visits = self
            .state
            .graph()
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        SignalRetainedExecutionBasis::capture(
            self.state.graph_mut(),
            ledger,
            &mut RetainedStoragePreparation::new(maximum_visits),
        )
        .map(Arc::new)
    }
}

fn map_issuance_capture_denial(
    denial: SignalConditionalBasisCaptureDenial,
) -> SignalConditionalServiceIssuanceDenial {
    match denial {
        SignalConditionalBasisCaptureDenial::CapacityExhausted => {
            SignalConditionalServiceIssuanceDenial::CaptureCapacityExhausted
        }
        SignalConditionalBasisCaptureDenial::WorkExhausted { maximum_visits } => {
            SignalConditionalServiceIssuanceDenial::CaptureWorkExhausted { maximum_visits }
        }
        SignalConditionalBasisCaptureDenial::OwnerUnavailable => {
            SignalConditionalServiceIssuanceDenial::OwnerUnavailable(
                crate::branch::owner_services::SignalOwnerUnavailable,
            )
        }
        SignalConditionalBasisCaptureDenial::Unavailable => {
            SignalConditionalServiceIssuanceDenial::CaptureUnavailable
        }
    }
}
