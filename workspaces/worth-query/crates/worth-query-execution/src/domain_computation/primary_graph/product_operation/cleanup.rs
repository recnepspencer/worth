//! Application-owned completion of product cleanup across Query and Bridge.

use std::sync::{Arc, RwLock};

use worth_runtime_bridge::facade::{BridgeConditionalDenial, BridgeSealedRuntimeAssembly};

use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductBranchOwnerCleanupWork,
};

#[derive(Debug)]
pub enum WorthQueryApplicationProductBranchCleanupDenial {
    SignalBasisObservation(worth_signal::facade::branch::SignalBranchBasisObservationDenial),
    ConditionalRetirement(BridgeConditionalDenial),
    Product(WorthQueryProductBranchOwnerCleanupDenial),
}

/// Retains every owner needed to finish one product cleanup obligation.
#[must_use = "product cleanup must be completed or retained for retry"]
pub struct WorthQueryApplicationProductBranchCleanup {
    product: WorthQueryProductBranchOwnerCleanup,
    conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
}

impl std::fmt::Debug for WorthQueryApplicationProductBranchCleanup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationProductBranchCleanup")
            .field("pending", &self.product.pending_work())
            .finish_non_exhaustive()
    }
}

impl WorthQueryApplicationProductBranchCleanup {
    pub(in crate::domain_computation) fn new(
        product: WorthQueryProductBranchOwnerCleanup,
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Self {
        Self {
            product,
            conditional,
        }
    }

    pub fn pending_work(&self) -> Vec<WorthQueryProductBranchOwnerCleanupWork> {
        self.product.pending_work()
    }

    pub fn retry(
        self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryApplicationProductBranchCleanupFailure,
    > {
        let Self {
            product,
            conditional,
        } = self;
        if let Err(denial) = product.release_retired_history() {
            return Err(Self::failed(
                product,
                conditional,
                WorthQueryApplicationProductBranchCleanupDenial::Product(denial),
            ));
        }
        let bases = match product.pending_signal_bases() {
            Ok(bases) => bases,
            Err(denial) => {
                return Err(Self::failed(
                    product,
                    conditional,
                    WorthQueryApplicationProductBranchCleanupDenial::SignalBasisObservation(denial),
                ));
            }
        };
        for basis in bases {
            let result = {
                conditional
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .retire_owned_conditionals_on_signal_branch(&basis)
            };
            if let Err(denial) = result {
                return Err(Self::failed(
                    product,
                    conditional,
                    WorthQueryApplicationProductBranchCleanupDenial::ConditionalRetirement(denial),
                ));
            }
        }
        match product.retry() {
            Ok(receipt) => Ok(receipt),
            Err(failure) => {
                let denial = failure.denial();
                Err(Self::failed(
                    failure.into_cleanup(),
                    conditional,
                    WorthQueryApplicationProductBranchCleanupDenial::Product(denial),
                ))
            }
        }
    }

    fn failed(
        product: WorthQueryProductBranchOwnerCleanup,
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
        denial: WorthQueryApplicationProductBranchCleanupDenial,
    ) -> WorthQueryApplicationProductBranchCleanupFailure {
        WorthQueryApplicationProductBranchCleanupFailure {
            denial,
            cleanup: Self {
                product,
                conditional,
            },
        }
    }
}

#[derive(Debug)]
pub struct WorthQueryApplicationProductBranchCleanupFailure {
    denial: WorthQueryApplicationProductBranchCleanupDenial,
    cleanup: WorthQueryApplicationProductBranchCleanup,
}

impl WorthQueryApplicationProductBranchCleanupFailure {
    pub const fn denial(&self) -> &WorthQueryApplicationProductBranchCleanupDenial {
        &self.denial
    }

    pub fn into_cleanup(self) -> WorthQueryApplicationProductBranchCleanup {
        self.cleanup
    }
}
