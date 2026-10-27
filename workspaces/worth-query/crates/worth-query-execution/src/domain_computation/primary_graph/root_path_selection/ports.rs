//! Private acquisition ports retain the concrete adapter's control and authority.
use super::super::read_observation::WorthQueryInvariantProjectionWork;
use super::*;
use worth_query_installation::facade::WorthQueryInstalledRootPathGuard;
use worth_relational::facade::runtime::{
    BoundedFrontierFieldEqualityTruthRead, PinnedStructuralWalkReads,
};

pub(in crate::domain_computation::primary_graph) enum RootPathReadObservation<'a> {
    SourceRead {
        subject: &'a str,
    },
    SourceCopy {
        units: usize,
        subject: &'a str,
    },
    Adjacency {
        lists: usize,
        records: usize,
        endpoints: usize,
        subject: &'a str,
    },
    Predicate {
        records: usize,
        matches: usize,
        fields: usize,
        subject: &'a str,
    },
    Walk {
        reads: PinnedStructuralWalkReads,
        units: usize,
        subject: &'a str,
    },
    Step {
        path_index: usize,
        step_index: usize,
        traversal: ApplicationQueryRootPathTraversal,
        direction: RelationDirection,
        reads: WorthQueryInvariantProjectionWork,
        subject: &'a str,
    },
}
/// `prepare_sources` must run exactly once, after the concrete guard admission
/// and before its physical equality read. No protected revision precedes admission.
pub(in crate::domain_computation::primary_graph) struct RootPathReadPorts<'a, E> {
    pub(in crate::domain_computation::primary_graph) checkpoint: &'a dyn Fn(&str) -> Result<(), E>,
    pub(in crate::domain_computation::primary_graph) read_guard_with_sources:
        &'a mut dyn FnMut(
            &WorthQueryInstalledRootPathGuard,
            &BTreeSet<EntityId>,
            &mut dyn FnMut() -> Result<(), RootPathReadDenial<E>>,
        )
            -> Result<BoundedFrontierFieldEqualityTruthRead, RootPathReadDenial<E>>,
    pub(in crate::domain_computation::primary_graph) observation:
        &'a mut dyn FnMut(RootPathReadObservation<'_>) -> Result<(), E>,
}
impl<E> RootPathReadPorts<'_, E> {
    pub(super) fn check(&self, subject: &str) -> Result<(), RootPathReadDenial<E>> {
        (self.checkpoint)(subject).map_err(RootPathReadDenial::Adapter)
    }
    pub(super) fn notify(
        &mut self,
        observation: RootPathReadObservation<'_>,
    ) -> Result<(), RootPathReadDenial<E>> {
        (self.observation)(observation).map_err(RootPathReadDenial::Adapter)
    }
}
