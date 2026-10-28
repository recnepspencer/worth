//! Query intent around the sole admitted Product-read startup.
use super::product_read::ProductReadIntent;
use super::*;

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
            WorthQueryPreparedProductReadSessionIdentity::reserve()?,
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
        reserved: WorthQueryPreparedProductReadSessionIdentity,
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
        if plan.intent().kind()
            != worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkIntentKind::ApplicationQueryRead
        {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        Self::start_product_read(
            reserved,
            ProductReadIntent::Query,
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
}
