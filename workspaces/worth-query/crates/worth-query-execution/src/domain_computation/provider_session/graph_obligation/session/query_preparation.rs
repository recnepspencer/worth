use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisSelectionIdentity,
};
use worth_query_admission::facade::graph_obligation::WorthQueryAdmittedGraphWorkPlan;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryGraphReadOwnerPort, WorthQueryGraphWorkAccessContextAffinity,
    WorthQueryGraphWorkBasis, WorthQueryGraphWorkBranchAffinity,
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
    WorthQueryManagedGraphWorkSession, WorthQueryManagedGraphWorkSessionStartDenial,
};

mod admitted;
pub(in crate::domain_computation) use admitted::WorthQueryAdmittedQuerySessionStartStop;

/// One reserved identity for a freshly admitted Query permission. The same
/// identity starts graph work only after that permission admits its Product.
pub(in crate::domain_computation) struct WorthQueryPreparedQuerySessionIdentity {
    identity: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
}

impl WorthQueryPreparedQuerySessionIdentity {
    pub(in crate::domain_computation) fn reserve(
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let identity = WorthQueryGraphWorkSessionIdentity::mint()
            .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted)?;
        let managed_run = WorthQueryGraphWorkManagedRunIdentity::mint()
            .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted)?;
        Ok(Self {
            identity,
            managed_run,
        })
    }

    pub(in crate::domain_computation) const fn identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.identity
    }

    pub(in crate::domain_computation) const fn managed_run_identity(
        &self,
    ) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }
}

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_query(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        authorization_product: &crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        port: WorthQueryGraphReadOwnerPort,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        Self::start_query_with_reserved_identity(
            WorthQueryPreparedQuerySessionIdentity::reserve()?,
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            basis,
            product,
            authorization_product,
            provider,
            port,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_query_with_reserved_identity(
        reserved: WorthQueryPreparedQuerySessionIdentity,
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        authorization_product: &crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        port: WorthQueryGraphReadOwnerPort,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let branch = WorthQueryGraphWorkBranchAffinity::from_query_basis(basis);
        let authorization_branch =
            WorthQueryGraphWorkBranchAffinity::from_product(authorization_product);
        let selected_product = match basis.selection() {
            WorthQueryApplicationBasisSelectionIdentity::Product(identity) => identity,
            WorthQueryApplicationBasisSelectionIdentity::Relational => {
                return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
            }
        };
        let carried_product = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            product.observation(),
        );
        if !branch.admits_query_basis(basis) || selected_product != &carried_product {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
        }
        Self::start(
            reserved.identity,
            Some(reserved.managed_run),
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            branch,
            authorization_branch,
            WorthQueryGraphWorkBasis::Query {
                identity: basis.clone(),
                product,
                port,
            },
            provider,
        )
    }
}
