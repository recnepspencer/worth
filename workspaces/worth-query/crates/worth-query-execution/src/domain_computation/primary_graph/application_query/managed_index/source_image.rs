//! Immutable disclosed source data, issued only by completed source execution.
use super::source_rows::SourceImageView;
use crate::domain_computation::primary_graph::application_query::{
    admitted_result::WorthQueryApplicationQueryRequestAffinity,
    disclosure::WorthQueryApplicationQueryGovernance,
    projection::WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryObservedResultSet, WorthQueryObservedSource,
};
use crate::domain_computation::{
    primary_graph::source_facts::SourceFactsAllocation,
    provider_session::WorthQueryCompletedSourceRead,
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryScopeBinding,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBindingCatalog, WorthQueryInstalledManagedIndex,
};

pub(super) type SourceBinding<S, I> = <I as ApplicationManagedIndexBinding<S>>::SourceBinding;
pub(super) type SourceQuery<S, I> = <SourceBinding<S, I> as ApplicationQueryBinding<S>>::Query;
pub(super) type SourcePlan<'a, S, I> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, SourceQuery<S, I>,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    <SourceBinding<S, I> as ApplicationQueryBinding<S>>::Principal,
    <SourceBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;

pub(super) struct SourceImage<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    pub(super) installed: WorthQueryInstalledManagedIndex<S, I>,
    pub(super) tree: WorthQueryApplicationDisclosedProjectionTree,
    pub(super) order: worth_execution::ExecutionArray<usize>,
    pub(super) dependencies: Arc<super::super::observed_source::ManagedSourceDependencies>,
    // Keep the original observations and their custody for this image's lifetime.
    pub(super) _sources: Vec<WorthQueryObservedSource<SourceQuery<S, I>>>,
    pub(super) _result_set: WorthQueryObservedResultSet<SourceQuery<S, I>>,
    pub(super) _affinity: WorthQueryApplicationQueryRequestAffinity,
    pub(super) terminal: Arc<WorthQueryCompletedSourceRead>,
    pub(super) _allocation: Arc<SourceFactsAllocation>,
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> SourceImage<S, I> {
    pub(super) fn view<'a>(
        &'a self,
        bindings: &'a WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> SourceImageView<'a> {
        SourceImageView {
            owner: self.terminal.session_identity(),
            tree: &self.tree,
            order: &self.order,
            governance: &WorthQueryApplicationQueryGovernance::Public,
            bindings,
        }
    }
}
