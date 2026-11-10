//! Creation names and destination incarnations share the owner's restored issuer.
use super::{ProductBranchIdentity, ProductBranchIncarnation, ProductBranchName};
use super::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldOwnerRoot};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The branch identity is the owner plus the normalized name, so retiring
    /// and recreating one name yields the same identity with a new
    /// incarnation. Only the incarnation is drawn from the issuer.
    pub(in crate::lifecycle::owner) fn issue_branch_identities(
        &self,
        name: ProductBranchName,
    ) -> Result<(ProductBranchIdentity, ProductBranchIncarnation), ()> {
        let mut identities = self
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let incarnation = identities.branch_incarnation().map_err(|_| ())?;
        Ok((
            ProductBranchIdentity::issued(self.owner_identity(), name),
            incarnation,
        ))
    }

    pub(super) fn issue_creation_name_ordinal(
        &self,
    ) -> Result<u64, RuntimeWorldBranchAdmissionDenial> {
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        self.state
            .identities
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .branch_incarnation()
            .map(|reserved| reserved.ordinal())
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::IdentityExhausted)
    }
}

impl<D, I, E, Ctx, T> crate::lifecycle::RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Read-only diagnostic of the same owner's next name/incarnation coordinate.
    pub fn creation_name_frontier_for_test(
        &self,
    ) -> Result<u64, crate::lifecycle::RuntimeWorldOwnerUnavailable> {
        let owner = self.service()?;
        let identities = owner
            .state
            .identities
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        Ok(identities.branch_incarnation_frontier())
    }
}
