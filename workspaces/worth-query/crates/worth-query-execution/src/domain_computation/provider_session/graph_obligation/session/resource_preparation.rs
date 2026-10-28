//! Resource intent binds the actual installed leaf before opening Product work.
use super::*;
use worth_query_declaration::facade::application_resource::ApplicationResourceBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationResource, WorthQueryResourceAccessPurpose,
};

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_resource<Schema, Binding>(
        reserved: WorthQueryPreparedProductReadSessionIdentity,
        resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
        purpose: WorthQueryResourceAccessPurpose,
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        authorization_product: &crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        port: WorthQueryGraphReadOwnerPort,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationResourceBinding<Schema>,
    {
        let contract = resource.retain_access_contract(purpose);
        if plan.intent().kind()
            != worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkIntentKind::ResourceRead
            || plan.binding_identity() != resource.binding_identity()
            || plan.obligation_identity() != contract.graph_obligations().identity()
            || !resource.owns_access_contract(&contract)
        {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        let obligation = contract.graph_obligations().identity().clone();
        Self::start_product_read(
            reserved,
            ProductReadIntent::Resource(contract),
            plan,
            runtime,
            resource.binding_identity(),
            &obligation,
            Binding::IDENTITY,
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
