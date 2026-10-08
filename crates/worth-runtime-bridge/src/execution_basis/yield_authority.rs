use super::finalization::finalize_signal_request;
use super::{
    BridgeBoundExecutionBasis, BridgeExecutionBasisFinalizationFailure,
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisIdentity,
    BridgeExecutionBasisTerminalDisposition, BridgeManagedExecutionIntent,
    BridgeManagedExecutionStepContract,
};

pub struct BridgeYieldedExecutionBasis {
    pub(super) basis: BridgeBoundExecutionBasis,
    receipt: BridgeExecutionBasisFinalizationReceipt,
}

impl BridgeBoundExecutionBasis {
    pub fn yield_execution_basis(
        mut self,
    ) -> Result<BridgeYieldedExecutionBasis, BridgeExecutionBasisFinalizationFailure> {
        if self.step_contract().is_none() {
            return Err(BridgeExecutionBasisFinalizationFailure::new(
                super::BridgeExecutionBasisFinalizationFailureKind::AtomicExecutionUnsupported,
                "atomic execution cannot yield",
                self,
            ));
        }
        let (signal_terminal, signal_transition_performed) = match finalize_signal_request(
            &self,
            BridgeExecutionBasisTerminalDisposition::Yielded,
        ) {
            Ok(finalization) => finalization,
            Err((kind, detail)) => {
                return Err(BridgeExecutionBasisFinalizationFailure::new(
                    kind, detail, self,
                ));
            }
        };
        self.signal_terminalized = true;
        let reservation_released = self
            .reservation
            .take()
            .is_some_and(super::reservation::BridgeExecutionBasisReservation::release);
        let receipt = BridgeExecutionBasisFinalizationReceipt::new(
            self.identity.clone(),
            self.managed_intent.identity().clone(),
            signal_terminal,
            BridgeExecutionBasisTerminalDisposition::Yielded,
            signal_transition_performed,
            reservation_released,
        );
        Ok(BridgeYieldedExecutionBasis {
            basis: self,
            receipt,
        })
    }
}

impl BridgeYieldedExecutionBasis {
    pub fn basis_identity(&self) -> &BridgeExecutionBasisIdentity {
        self.basis.identity()
    }

    pub fn managed_intent(&self) -> &BridgeManagedExecutionIntent {
        self.basis.managed_intent()
    }

    pub fn step_contract(&self) -> &BridgeManagedExecutionStepContract {
        self.basis
            .step_contract()
            .expect("yield admission proves managed step authority")
    }

    pub fn receipt(&self) -> &BridgeExecutionBasisFinalizationReceipt {
        &self.receipt
    }

    pub fn basis_request_identity(&self) -> &str {
        self.basis.request().digest()
    }

    pub fn release(self) -> BridgeExecutionBasisFinalizationReceipt {
        self.receipt
    }

    pub(super) fn take_observation(&mut self) -> crate::snapshot::MaterializedTruthViewObservation {
        self.basis.take_observation()
    }
}
