use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryBatchAdmission, WorthQueryApplicationQueryBatchResourceDenial,
};

/// Refusal of independently scoped ordinary batch admission: either the
/// original authorization/planning cause or the shared custody allowance.
/// Neither variant carries a partial query result or grants access authority.
#[derive(Debug)]
pub enum WorthQueryRetainedBatchQueryAdmissionDenial {
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Resource(WorthQueryApplicationQueryBatchResourceDenial),
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// Admits another independently scoped ordinary read at the same two
    /// issued selections. This does not select latest, combine scopes, or grant
    /// an authorization exemption. The consuming scalar owner still performs
    /// its complete ordinary admission and disclosure decision.
    pub fn admit_retained_application_query_from_selection<
        'read,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'read self,
        retained: &mut WorthQuerySelectedProductOperation<'runtime, Schema>,
        query: &'read WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'read,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryProductQueryControls<'read>,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'read,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryRetainedBatchQueryAdmissionDenial,
    > {
        use WorthQueryRetainedBatchQueryAdmissionDenial::{Admission, Resource};
        let unavailable = || {
            WorthQueryApplicationQueryAdmissionDenial::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
            query.name(),
        )
        };
        let application: &'read crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema> = self.application();
        if !std::ptr::eq(application, retained.application())
            || self.product().branch_identity() != retained.product().branch_identity()
        {
            return Err(Admission(unavailable()));
        }
        let security = application
            .product_runtime
            .security_observation_for(self.product())
            .map_err(|_| Admission(unavailable()))?;
        let planned_item = batch
            .take_planned_read(
                application.runtime.authority_identity().as_u64(),
                &application.installed_schema.binding_identity(),
                query.identity(),
                controls.maximum_work.get(),
                controls.maximum_results.get(),
            )
            .map_err(Resource)?;
        let application_basis = retained
            .application_basis_mut()
            .retain_in_batch(batch)
            .map_err(Resource)?;
        let product = retained.product().retained_clone();
        let mut plan = application
            .admit_application_query(
                query,
                access,
                parameters,
                WorthQueryApplicationQueryControls::retained_product_one_shot(
                    security,
                    product,
                    application_basis,
                    controls.maximum_results,
                    controls.maximum_work,
                    controls.request,
                )
                .limit_inline_result_bytes(batch.maximum_result_bytes_per_item()),
            )
            .map_err(Admission)?;
        plan.carry_planned_batch_item(planned_item);
        Ok(plan)
    }
}
