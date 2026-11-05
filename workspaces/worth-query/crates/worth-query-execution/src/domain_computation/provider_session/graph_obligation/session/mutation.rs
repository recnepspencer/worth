//! Mutation truth and its authorization source have separate owner custody.
use super::*;
use crate::basis::WorthQueryProductObservationLease;

pub(super) struct MutationProductBasis {
    pub(super) lease: Option<WorthQueryApplicationSnapshotLease>,
    control: Option<WorthQueryProductObservationLease>,
}

impl MutationProductBasis {
    pub(super) fn authorization_product(&self) -> Option<&WorthQueryProductObservationLease> {
        match &self.control {
            Some(control) => Some(control),
            None => self
                .lease
                .as_ref()
                .map(|lease| lease.product().read_lease_ref()),
        }
    }

    pub(super) fn admits_replacement_authorization(
        &self,
        authorization: &WorthQueryGraphWorkBranchAffinity,
        replacement: &WorthQueryApplicationSnapshotLease,
    ) -> bool {
        match &self.control {
            Some(control) => {
                *authorization == WorthQueryGraphWorkBranchAffinity::from_product(control)
            }
            None => authorization.admits_snapshot(replacement.snapshot()),
        }
    }
}

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        Self::start_mutation_with_control(
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            lease,
            None,
            provider,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation_with_control(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        control: Option<WorthQueryProductObservationLease>,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let branch = WorthQueryGraphWorkBranchAffinity::from_snapshot(lease.snapshot());
        if !branch.admits_snapshot(lease.snapshot()) {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
        }
        let authorization = control.as_ref().map_or_else(
            || branch.clone(),
            WorthQueryGraphWorkBranchAffinity::from_product,
        );
        Self::start(
            WorthQueryGraphWorkSessionIdentity::mint()
                .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted)?,
            None,
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            branch,
            authorization,
            WorthQueryGraphWorkBasis::Mutation(MutationProductBasis {
                lease: Some(lease),
                control,
            }),
            provider,
        )
    }
}
