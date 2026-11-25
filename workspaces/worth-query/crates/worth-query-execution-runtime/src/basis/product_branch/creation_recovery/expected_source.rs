//! Same-original publication with the exact expected-source lease still in custody.
use super::*;
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryProductRuntime,
    primary_graph::WorthQueryAdvancementPhase,
};

pub trait ProductBranchCreationRecoveryInternalPort: Sized {
    fn continue_with_expected_source(
        self,
        runtime: &WorthQueryProductRuntime,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (
            WorthQueryProductBranchCreationContinuation,
            WorthQueryProductBranchLease,
        ),
        WorthQueryProductBranchCreationRecoveryFailure,
    >;
}

impl ProductBranchCreationRecoveryInternalPort for WorthQueryProductBranchCreationRecovery {
    fn continue_with_expected_source(
        mut self,
        runtime: &WorthQueryProductRuntime,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (
            WorthQueryProductBranchCreationContinuation,
            WorthQueryProductBranchLease,
        ),
        WorthQueryProductBranchCreationRecoveryFailure,
    > {
        if self.runtime.fields().owner.owner_identity() != runtime.fields().owner.owner_identity() {
            return Err(WorthQueryProductBranchCreationRecoveryFailure {
                denial: WorthQueryProductBranchRecoveryDenial::ForeignRecovery,
                recovery: self,
            });
        }
        let Some(source) = self.expected_source.take() else {
            return Err(WorthQueryProductBranchCreationRecoveryFailure {
                denial: WorthQueryProductBranchRecoveryDenial::OriginalSourceUnavailable,
                recovery: self,
            });
        };
        match self.continue_in_advancement(phase) {
            Ok(outcome) => Ok((outcome, *source)),
            Err(mut failure) => {
                failure.recovery.expected_source = Some(source);
                Err(failure)
            }
        }
    }
}
