//! Inverse predicate algebra; topology and retained membership belong to Query.
mod probe;
mod root;
mod tree;
use super::{
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexProbeValue as Probe,
    WorthQueryManagedIndexProvider, WorthQueryManagedIndexProviderStop as Stop,
    WorthQueryManagedIndexSourceRow as Row,
};
pub(in crate::domain_computation::primary_graph::application_query) use probe::RetainedPredicateProbe;
pub(super) use root::TypedPredicateFactory;
pub(in crate::domain_computation::primary_graph::application_query) use root::{
    PredicateFactory, PredicateRoot,
};
use std::cmp::Ordering;
pub(in crate::domain_computation::primary_graph) use tree::WatchSlot;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

/// A qualified fixed-data release algebra, also used during requestless removal.
/// Implementations must do bounded fixed-width operations only: no allocation,
/// dynamic state, Probe/Source loans, SDK/Query reentry, I/O or panics. Configuration
/// opts in to this contract; Clone alone does not qualify an implementation.
pub trait WorthQueryManagedIndexPredicateSummary: Clone + Eq + Send + Sync + 'static {
    fn merge(own: &Self, children: [Option<&Self>; 2]) -> Self;
}

impl WorthQueryManagedIndexPredicateSummary for () {
    fn merge(_: &Self, _: [Option<&Self>; 2]) {}
}

/// Optional inverse routing for the same actual installed forward provider.
/// Query owns the immutable admitted probes, their strict order, reachable
/// topology, extrema and slots. Providers can prune only a supplied enclosure;
/// Query applies matches_source itself before recording any reachable receiver.
pub trait WorthQueryManagedIndexPredicateProvider<S, I>:
    WorthQueryManagedIndexProvider<S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    type PredicateSummary: WorthQueryManagedIndexPredicateSummary;

    fn compare_probes(
        &self,
        a: &Probe<S, I>,
        b: &Probe<S, I>,
        control: &Control<'_>,
    ) -> Result<Ordering, Stop<Self::Denial>>;

    fn predicate_summary(
        &self,
        probe: &Probe<S, I>,
        control: &Control<'_>,
    ) -> Result<Self::PredicateSummary, Stop<Self::Denial>>;

    fn predicate_region_may_match(
        &self,
        summary: &Self::PredicateSummary,
        extrema: [&Probe<S, I>; 2],
        row: Row<'_, S, I>,
        control: &Control<'_>,
    ) -> Result<bool, Stop<Self::Denial>>;
}
