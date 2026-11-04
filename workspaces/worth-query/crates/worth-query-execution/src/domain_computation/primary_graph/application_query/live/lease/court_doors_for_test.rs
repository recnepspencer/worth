//! Sealed test doors over this module's private members.
use super::*;

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn basis_for_test(&self) -> &Option<WorthQueryManagedLowerExecutionBasis> {
        &self.basis
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn controls_for_test(&self) -> &WorthQueryApplicationLiveControls {
        &self.controls
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn initial_read_work_for_test(
        &self,
    ) -> &Option<crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork> {
        &self.initial_read_work
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn queue_for_test(
        &self,
    ) -> &WorthQueryLiveCauseQueue<
        <Binding::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    > {
        &self.queue
    }
}

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding>
    WorthQueryApplicationLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn initial_read_work_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork>
    {
        &mut self.initial_read_work
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn queue_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryLiveCauseQueue<
        <Binding::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    > {
        &mut self.queue
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn basis_mut_for_test(&mut self) -> &mut Option<WorthQueryManagedLowerExecutionBasis> {
        &mut self.basis
    }
}

impl<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
>
    WorthQueryApplicationLiveLease<
        'runtime,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    /// The cause queue and the native basis, both mutable: one borrow, two fields.
    pub fn queue_basis_mut_for_test(
        &mut self,
    ) -> (
        &mut WorthQueryLiveCauseQueue<
            <Binding::PayloadBinding as ApplicationStructuredValueBinding>::Value,
        >,
        &mut Option<WorthQueryManagedLowerExecutionBasis>,
    ) {
        (&mut self.queue, &mut self.basis)
    }
}
