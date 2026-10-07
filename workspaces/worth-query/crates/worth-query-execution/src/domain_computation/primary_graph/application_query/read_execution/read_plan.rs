//! Concrete, pinned read data shared by ordinary and reconstruction kernels.

use super::super::{
    disclosure::WorthQueryApplicationQueryGovernance, WorthQueryAdmittedApplicationQueryPlan,
};
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::{identity::EntityId, snapshots::SnapshotHandle};

pub(in crate::domain_computation::primary_graph::application_query) struct ReadPlan<'a> {
    pub(in crate::domain_computation::primary_graph::application_query) name: &'a str,
    pub(in crate::domain_computation::primary_graph::application_query) contract:
        &'a WorthQueryInstalledGraphReadContract,
    pub(in crate::domain_computation::primary_graph::application_query) root: EntityId,
    pub(in crate::domain_computation::primary_graph::application_query) snapshot:
        &'a SnapshotHandle,
    pub(in crate::domain_computation::primary_graph::application_query) parameters:
        &'a WorthQueryAdmittedApplicationQueryParameters,
    pub(in crate::domain_computation::primary_graph::application_query) governance:
        &'a WorthQueryApplicationQueryGovernance,
    pub(in crate::domain_computation::primary_graph::application_query) maximum_work: usize,
    pub(in crate::domain_computation::primary_graph::application_query) maximum_result_count: usize,
}

impl<'a> ReadPlan<'a> {
    pub(in crate::domain_computation::primary_graph::application_query) fn of<
        S,
        Q,
        P,
        R,
        A,
        I,
        C,
    >(
        plan: &'a WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    ) -> Self {
        Self {
            name: plan.query.name(),
            contract: plan.query.read_family_binding().planning_contract(),
            root: plan.scope.entity_id(),
            snapshot: plan.basis.snapshot_handle(),
            parameters: &plan.parameters,
            governance: &plan.governance,
            maximum_work: plan.controls.maximum_work().get(),
            maximum_result_count: plan.controls.maximum_result_count().get(),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn public_worker<
        S,
        Q,
        P,
        R,
        A,
        I,
        C,
    >(
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
            name: _name,
            contract: _contract,
            root: _root,
            snapshot: _snapshot,
            parameters: _parameters,
            governance: _governance,
            maximum_work: _maximum_work,
            maximum_result_count: _maximum_result_count,
        } = self;
        0
    }
}
