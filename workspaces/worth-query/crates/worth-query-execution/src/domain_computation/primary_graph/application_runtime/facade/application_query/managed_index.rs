//! The runtime's consumer methods over `managed_index` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndex;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexWatch;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexWatchOutcome;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn execute_application_query_binding_one_shot<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<WorthQueryApplicationOneShotResult<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryManagedIndexDenial>
    where
        B: ApplicationQueryBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::ReceiverOps<S>>::execute_application_query_binding_one_shot(self, phase, binding, external, plan)
    }

    pub fn evict_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::EvictionOps<S>>::evict_application_managed_index(self, phase, binding, plan, index)
    }

    pub fn open_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<WorthQueryManagedIndex<S, I>, WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::LifecycleOps<S>>::open_application_managed_index(self, phase, binding, plan)
    }

    pub fn rebuild_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::LifecycleOps<S>>::rebuild_application_managed_index(self, phase, binding, external, plan, index)
    }

    pub fn next_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        watch: &mut WorthQueryManagedIndexWatch<S, I>,
    ) -> Result<WorthQueryManagedIndexWatchOutcome<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryManagedIndexDenial>
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NextOps<S>>::next_application_managed_index_watch(self, phase, binding, external, plan, watch)
    }

    pub fn open_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<
        (
            WorthQueryApplicationOneShotResult<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>,
            WorthQueryManagedIndexWatch<S, I>,
        ),
        WorthQueryManagedIndexDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpenOps<S>>::open_application_managed_index_watch(self, phase, binding, external, plan)
    }
}
