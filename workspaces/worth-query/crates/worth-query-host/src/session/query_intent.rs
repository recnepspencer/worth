//! One named bound for every query a session reads or keeps index-ready, so
//! generic application code never restates projection and scope-resolution
//! where-clauses.
use sha2::{Digest, Sha256};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexQueryIntent, ApplicationQueryBinding, ApplicationQueryIntent,
        ApplicationQueryScopeBinding, ApplicationQueryScopeResolution,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationScalarValueBinding, ApplicationSchema,
    },
};
use worth_query_execution::facade::primary_graph::{
    compute_admitted_request_binding_digest, WorthQueryApplicationProjection,
    WorthQueryApplicationRootWindow, WorthQueryManagedIndex,
};
use worth_query_publication::facade::{
    application_entry::{
        WorthQueryApplicationReadObservation, WorthQueryApplicationRequestQueryDenial,
    },
    domain_computation::WorthQueryPublishedApplicationResult,
};

use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};

/// A query a [`WorthQueryHostSession`] reads: every
/// `ApplicationQueryIntent` whose scope resolves for its principal and whose
/// rows project implements it.
///
/// ```
/// use worth_query_host::facade::{WorthQueryHostSession, WorthQuerySessionQuery};
/// use worth_query_host::facade::declaration::{
///     application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
/// };
/// fn read<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, Q: WorthQuerySessionQuery<S>>(
///     session: &WorthQueryHostSession<'_, S, P>,
///     intent: Q,
/// ) -> Option<Vec<Q::Row>> {
///     session.query(intent).ok()
/// }
/// ```
pub trait WorthQuerySessionQuery<Schema: ApplicationSchema>:
    ApplicationQueryIntent<Schema>
{
    /// The projected row type.
    type Row;

    #[doc(hidden)]
    fn execute_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn execute_at_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn execute_window_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn execute_window_in_session_with_execution<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    >;

    #[doc(hidden)]
    fn derive_historical_query_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial>
    where
        Self: Clone;

    #[doc(hidden)]
    fn admitted_request_binding_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial>
    where
        Self: Clone;

    #[doc(hidden)]
    fn installed_contract_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial>;
}

impl<Schema, Intent> WorthQuerySessionQuery<Schema> for Intent
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    type Row = QueryRow<Schema, Intent>;

    fn execute_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = request.query(self);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn execute_at_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = request.at(observation).query(self);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn execute_window_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = match at {
            Some(observation) => request.at(observation).query(self),
            None => request.query(self),
        }
        .window(window);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn execute_window_in_session_with_execution<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        at: Option<&WorthQueryApplicationReadObservation>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Self>, Self::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let query_req = match at {
            Some(observation) => request.at(observation).query(self),
            None => request.query(self),
        }
        .window(window);
        let query_req = match session.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        session
            .home
            .primary_graph()
            .with_scoped_execution_request_advancement(
                *execution,
                session.settings.policy,
                &scope,
                |phase| query_req.execute_in_advancement(&phase),
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    fn derive_historical_query_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial>
    where
        Self: Clone,
    {
        let contract_digest = self.installed_contract_digest(session)?;
        let request_binding_digest = self.admitted_request_binding_digest(session)?;
        let desc = <QueryBinding<Schema, Self> as ApplicationQueryBinding<Schema>>::descriptor();
        let mut h = Sha256::new();
        h.update(b"worth.query.historical.binding-contract.v4");
        h.update(contract_digest);
        h.update(request_binding_digest);
        hash_str(&mut h, desc.identity());
        hash_str(&mut h, desc.query_name());
        hash_str(&mut h, desc.query_identity().as_str());
        hash_str(&mut h, desc.input_identity().as_str());
        hash_str(&mut h, desc.parameter_identity().as_str());
        hash_str(&mut h, desc.result_identity().as_str());
        hash_str(&mut h, desc.scope_identity().as_str());
        let locus = desc.scope().field().locus();
        hash_str(&mut h, locus.entity());
        hash_str(&mut h, locus.aspect());
        hash_str(&mut h, locus.field());
        hash_str(&mut h, desc.principal().name());
        Ok(h.finalize().into())
    }

    fn admitted_request_binding_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial>
    where
        Self: Clone,
    {
        let binding = session
            .home
            .primary_graph()
            .installed_schema()
            .installed_query_binding::<QueryBinding<Schema, Self>>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let installed_query = binding.query();
        let params = self
            .prepare_parameters()
            .map_err(WorthQueryApplicationRequestQueryDenial::InputPreparation)?;
        let scope_binding = self.clone().into_scope();
        let Some((_field, scope_value_opt)) =
            <<QueryBinding<Schema, Self> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Self> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >>::into_request_parts(scope_binding)
        else {
            return Err(WorthQueryApplicationRequestQueryDenial::RequestMode);
        };
        let encoded_scope: Option<AspectValue> = match scope_value_opt {
            Some(val) => {
                type FieldOf<S, I> = <<QueryBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Field;
                type BindingOf<S, I> = <FieldOf<S, I> as RecordSelection>::Binding;
                let encoded =
                    <BindingOf<Schema, Self> as ApplicationScalarValueBinding>::encode(&val)
                        .map_err(|err| {
                            WorthQueryApplicationRequestQueryDenial::InputPreparation(err.into())
                        })?;
                Some(encoded)
            }
            None => None,
        };
        let param_refs: Vec<(&str, &AspectValue)> = params
            .bindings()
            .iter()
            .map(|(name, value)| (*name, value))
            .collect();
        let external = session.principal().identity();
        Ok(compute_admitted_request_binding_digest(
            installed_query.read_graph().digest().bytes(),
            installed_query.identity().as_bytes(),
            &param_refs,
            installed_query.scope_entity(),
            encoded_scope.as_ref(),
            external.issuer(),
            external.subject(),
        ))
    }

    fn installed_contract_digest<Program>(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<[u8; 32], WorthQueryApplicationRequestQueryDenial> {
        let binding = session
            .home
            .primary_graph()
            .installed_schema()
            .installed_query_binding::<QueryBinding<Schema, Self>>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        Ok(*binding.query().read_graph().digest().bytes())
    }
}

fn hash_str(h: &mut Sha256, s: &str) {
    h.update((s.len() as u32).to_le_bytes());
    h.update(s.as_bytes());
}

/// A query answered by a managed index the session can make ready.
pub trait WorthQuerySessionIndexedQuery<Schema: ApplicationSchema>:
    ApplicationManagedIndexQueryIntent<Schema>
{
    #[doc(hidden)]
    fn ensure_ready_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<WorthQueryManagedIndex<Schema, Self::Index>, WorthQueryApplicationRequestQueryDenial>;
}

impl<Schema, Intent> WorthQuerySessionIndexedQuery<Schema> for Intent
where
    Schema: ApplicationSchema,
    Intent: ApplicationManagedIndexQueryIntent<Schema>,
    <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    fn ensure_ready_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<WorthQueryManagedIndex<Schema, Self::Index>, WorthQueryApplicationRequestQueryDenial>
    {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        session
            .with_funded_advancement(&scope, |phase| {
                request
                    .query(self)
                    .ensure_index_ready_in_advancement::<Intent::Index>(&phase)
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
