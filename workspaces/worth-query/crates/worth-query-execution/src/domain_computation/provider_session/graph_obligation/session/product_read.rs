//! One admitted current Product basis shared by distinct sealed read intents.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity;
use worth_query_installation::facade::WorthQueryInstalledResourceAccessContract;

pub(super) struct ProductReadBasis {
    pub(super) identity: WorthQueryApplicationBasisIdentity,
    // Retain query-truth custody separately from current authorization custody.
    pub(super) _product: crate::basis::WorthQueryProductObservationLease,
    pub(super) authorization_product: crate::basis::WorthQueryProductObservationLease,
    pub(super) port: WorthQueryGraphReadOwnerPort,
}

pub(super) enum ProductReadIntent {
    Query,
    Inspection(std::sync::Arc<
        crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget,
    >),
    Resource(std::sync::Arc<WorthQueryInstalledResourceAccessContract>),
}

/// Reserved before an authorization observation, consumed by its actual session.
pub(in crate::domain_computation) struct WorthQueryPreparedProductReadSessionIdentity {
    identity: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
}

impl WorthQueryPreparedProductReadSessionIdentity {
    pub(in crate::domain_computation) fn reserve(
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        Ok(Self {
            identity: WorthQueryGraphWorkSessionIdentity::mint()
                .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted)?,
            managed_run: WorthQueryGraphWorkManagedRunIdentity::mint()
                .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted)?,
        })
    }

    pub(super) fn into_identities(
        self,
    ) -> (
        WorthQueryGraphWorkSessionIdentity,
        WorthQueryGraphWorkManagedRunIdentity,
    ) {
        (self.identity, self.managed_run)
    }
    pub(in crate::domain_computation) const fn identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.identity
    }
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn resource_read_binding(
        &self,
    ) -> Option<(
        &std::sync::Arc<WorthQueryInstalledResourceAccessContract>,
        &WorthQueryApplicationBasisIdentity,
    )> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead {
                intent: ProductReadIntent::Resource(contract),
                read,
            } => Some((contract, &read.identity)),
            _ => None,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_product_read(
        reserved: WorthQueryPreparedProductReadSessionIdentity,
        intent: ProductReadIntent,
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
            WorthQueryGraphWorkBasis::ProductRead {
                intent,
                read: ProductReadBasis {
                    identity: basis.clone(),
                    _product: product,
                    authorization_product: authorization_product.retained_clone(),
                    port,
                },
            },
            provider,
        )
    }

    pub(super) fn query_read_basis(
        &self,
    ) -> Result<&ProductReadBasis, WorthQueryManagedGraphReadDenial> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead {
                intent: ProductReadIntent::Query,
                read,
            } => Ok(read),
            _ => Err(WorthQueryManagedGraphReadDenial::ForeignReadIntent),
        }
    }
}
