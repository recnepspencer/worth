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

pub type SourceBinding<S, I> = <I as ApplicationManagedIndexBinding<S>>::SourceBinding;
pub type SourceQuery<S, I> = <SourceBinding<S, I> as ApplicationQueryBinding<S>>::Query;
pub type SourcePlan<'a, S, I> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, SourceQuery<S, I>,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    <SourceBinding<S, I> as ApplicationQueryBinding<S>>::Principal,
    <SourceBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<SourceBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;

pub struct SourceImage<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    pub rebuild_loan: Option<worth_relational::facade::runtime::ManagedIndexSourceRebuild>,
    pub installed: WorthQueryInstalledManagedIndex<S, I>,
    pub material: Arc<SourceMaterial>,
    pub dependencies: Arc<super::super::observed_source::ManagedSourceDependencies>,
    // Keep the original observations and their custody for this image's lifetime.
    pub _sources: Vec<WorthQueryObservedSource<SourceQuery<S, I>>>,
    pub _result_set: WorthQueryObservedResultSet<SourceQuery<S, I>>,
    pub _affinity: WorthQueryApplicationQueryRequestAffinity,
    pub terminal: Arc<WorthQueryCompletedSourceRead>,
    pub authorization:
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    pub _allocation: Arc<SourceFactsAllocation>,
}

/// Immutable disclosed rows carry no completed READ, World or Native root custody.
pub struct SourceMaterial {
    pub tree: WorthQueryApplicationDisclosedProjectionTree,
    pub order: worth_execution::ExecutionArray<usize>,
    pub _allocation: Arc<SourceFactsAllocation>,
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> SourceImage<S, I> {
    pub fn view<'a>(
        &'a self,
        bindings: &'a WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> SourceImageView<'a> {
        SourceImageView {
            owner: self.terminal.session_identity(),
            rows: super::source_rows::SourceImageRows::Completed {
                tree: &self.material.tree,
                order: &self.material.order,
            },
            governance: &WorthQueryApplicationQueryGovernance::Public,
            bindings,
            source_budget: None,
        }
    }
}
