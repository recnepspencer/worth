use crate::branch::ProductBranchObservation;

use super::RuntimeWorldOwnerRoot;

#[derive(Debug)]
pub enum RuntimeWorldOwnedAsyncRequestAdmissionDenial {
    ForeignOwner,
    RelationalSourceMismatch,
    NoCommittedSource,
    Bridge(worth_runtime_bridge::facade::BridgeAsyncRequestIdentityRejection),
}

#[derive(Debug)]
pub enum RuntimeWorldOwnedAsyncRevalidationDenial {
    ForeignOwner,
    RelationalSourceMismatch,
    NoCommittedSource,
    Bridge(worth_runtime_bridge::facade::BridgeAsyncCompletionRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeWorldOwnedAsyncProductValidationDenial {
    ForeignOwner,
    RelationalSourceMismatch,
    NoCommittedSource,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn admit_managed_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        relational_source: &worth_runtime_bridge::facade::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
        RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        let truth_basis = self
            .validate_owned_async_product(observation, relational_source)
            .map_err(request_product_denial)?;
        bridge
            .admit_managed_owned_async_request_identity(
                declaration,
                observation.basis().signal_basis(),
                truth_basis,
            )
            .map_err(RuntimeWorldOwnedAsyncRequestAdmissionDenial::Bridge)
    }

    pub(crate) fn admit_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        relational_source: &worth_runtime_bridge::facade::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        let truth_basis = self
            .validate_owned_async_product(observation, relational_source)
            .map_err(request_product_denial)?;
        bridge
            .admit_owned_async_request_identity(
                declaration,
                observation.basis().signal_basis(),
                truth_basis,
            )
            .map_err(RuntimeWorldOwnedAsyncRequestAdmissionDenial::Bridge)
    }

    pub(crate) fn revalidate_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        observation: &ProductBranchObservation,
        relational_source: &worth_runtime_bridge::facade::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRevalidationAdmission,
        RuntimeWorldOwnedAsyncRevalidationDenial,
    > {
        let truth_basis = self
            .validate_owned_async_product(observation, relational_source)
            .map_err(|denial| match denial {
                RuntimeWorldOwnedAsyncProductValidationDenial::ForeignOwner => {
                    RuntimeWorldOwnedAsyncRevalidationDenial::ForeignOwner
                }
                RuntimeWorldOwnedAsyncProductValidationDenial::RelationalSourceMismatch => {
                    RuntimeWorldOwnedAsyncRevalidationDenial::RelationalSourceMismatch
                }
                RuntimeWorldOwnedAsyncProductValidationDenial::NoCommittedSource => {
                    RuntimeWorldOwnedAsyncRevalidationDenial::NoCommittedSource
                }
            })?;
        bridge
            .revalidate_owned_async_request(
                request,
                observation.basis().signal_basis(),
                truth_basis,
            )
            .map_err(RuntimeWorldOwnedAsyncRevalidationDenial::Bridge)
    }

    fn validate_owned_async_product(
        &self,
        observation: &ProductBranchObservation,
        relational_source: &worth_runtime_bridge::facade::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeAsyncRequestTruthViewBasis,
        RuntimeWorldOwnedAsyncProductValidationDenial,
    > {
        if observation.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldOwnedAsyncProductValidationDenial::ForeignOwner);
        }
        if !relational_source.admits_basis(observation.basis().relational_basis()) {
            return Err(RuntimeWorldOwnedAsyncProductValidationDenial::RelationalSourceMismatch);
        }
        let relational = observation.basis().relational_basis().observation();
        let commit = relational
            .commit_id()
            .ok_or(RuntimeWorldOwnedAsyncProductValidationDenial::NoCommittedSource)?;
        Ok(
            worth_runtime_bridge::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
                worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id(
                    relational.identity().branch_id().0.clone(),
                ),
                worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                    commit.0,
                ),
                relational_source.snapshot_identity().clone(),
            ),
        )
    }
}

fn request_product_denial(
    denial: RuntimeWorldOwnedAsyncProductValidationDenial,
) -> RuntimeWorldOwnedAsyncRequestAdmissionDenial {
    match denial {
        RuntimeWorldOwnedAsyncProductValidationDenial::ForeignOwner => {
            RuntimeWorldOwnedAsyncRequestAdmissionDenial::ForeignOwner
        }
        RuntimeWorldOwnedAsyncProductValidationDenial::RelationalSourceMismatch => {
            RuntimeWorldOwnedAsyncRequestAdmissionDenial::RelationalSourceMismatch
        }
        RuntimeWorldOwnedAsyncProductValidationDenial::NoCommittedSource => {
            RuntimeWorldOwnedAsyncRequestAdmissionDenial::NoCommittedSource
        }
    }
}
