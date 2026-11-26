//! Concrete, pinned read data shared by ordinary and reconstruction kernels.

use super::super::{
    disclosure::WorthQueryApplicationQueryGovernance, WorthQueryAdmittedApplicationQueryPlan,
};
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::{identity::EntityId, snapshots::SnapshotHandle};

pub struct ReadPlan<'a> {
    pub(crate) parent_session:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub(crate) managed_rows: Option<&'a super::super::managed_index::selection::SelectionRows>,
    pub(crate) applied_managed_rows:
        Option<&'a super::super::managed_index::applied_read::AppliedSelectionRows<'a>>,
    pub(crate) binding: &'a worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub(crate) name: &'a str,
    pub(crate) contract: &'a WorthQueryInstalledGraphReadContract,
    pub(crate) root: EntityId,
    pub(crate) snapshot: &'a SnapshotHandle,
    pub(crate) parameters: &'a WorthQueryAdmittedApplicationQueryParameters,
    pub(crate) governance: &'a WorthQueryApplicationQueryGovernance,
    pub(crate) material_source: Option<
        &'a std::sync::Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    >,

    pub(crate) maximum_result_count: usize,
    pub(crate) root_window: Option<&'a super::super::root_window::WorthQueryApplicationRootWindow>,
}

impl<'a> ReadPlan<'a> {
    pub fn of<S, Q, P, R, A, I, C>(
        plan: &'a WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    ) -> Self {
        Self {
            parent_session: plan.graph_work.identity(),
            managed_rows: None,
            applied_managed_rows: None,
            binding: plan.query.binding_identity(),
            name: plan.query.name(),
            contract: plan.query.read_family_binding().planning_contract(),
            root: plan.scope.entity_id(),
            snapshot: plan.basis.snapshot_handle(),
            parameters: &plan.parameters,
            governance: &plan.governance,
            material_source: None,
            maximum_result_count: plan.controls.maximum_result_count().get(),
            root_window: plan.root_window.as_ref(),
        }
    }
    pub fn public_worker<S, Q, P, R, A, I, C>(
        plan: &'a WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    ) -> Self {
        let mut read = Self::of(plan);
        // Admission has proved Public. Do not carry governed authorization,
        // which can itself retain a Query request scope, onto this worker.
        read.governance = &WorthQueryApplicationQueryGovernance::Public;
        read
    }
}

impl worth_execution::ChargedBytes for ReadPlan<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        // Borrowed installed/pinned values and inline limits allocate nothing.
        let Self {
            parent_session: _parent_session,
            managed_rows: _managed_rows,
            applied_managed_rows: _applied_managed_rows,
            binding: _binding,
            name: _name,
            contract: _contract,
            root: _root,
            snapshot: _snapshot,
            parameters: _parameters,
            governance: _governance,
            material_source: _material_source,

            maximum_result_count: _maximum_result_count,
            root_window: _root_window,
        } = self;
        0
    }
}

impl ReadPlan<'_> {
    /// After session custody accepts the same exact root/World selection,
    /// Native work uses the shared capability's operational snapshot. The
    /// session retains and seals its own original receipt after the join.
    pub fn with_snapshot<'b>(&'b self, snapshot: &'b SnapshotHandle) -> ReadPlan<'b> {
        ReadPlan {
            parent_session: self.parent_session,
            managed_rows: self.managed_rows,
            applied_managed_rows: self.applied_managed_rows,
            binding: self.binding,
            name: self.name,
            contract: self.contract,
            root: self.root,
            snapshot,
            parameters: self.parameters,
            governance: self.governance,
            material_source: self.material_source,

            maximum_result_count: self.maximum_result_count,
            root_window: self.root_window,
        }
    }
}

impl ReadPlan<'_> {
    pub fn snapshot_unavailable_denial(&self) -> super::WorthQueryApplicationReadExecutionDenial {
        if self.contract.orbit().is_some() || !self.contract.root_paths().is_empty() {
            super::read_execution_denial(
                super::WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
                self.name,
            )
        } else {
            super::read_execution_denial(
                super::WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                self.name,
            )
        }
    }
    pub fn predicate_unavailable_denial(&self) -> super::WorthQueryApplicationReadExecutionDenial {
        super::read_execution_denial(
            super::WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            match self.contract.predicates() {
                [predicate] => predicate.field().2,
                _ => self.name,
            },
        )
    }
}
