//! Descriptive ordinal routing established by completion, never read authority. Moved from
//! `primary_graph::application_attempt::read_set::application_projection`; the attempt keeps the
//! routing that builds and refreshes it (`CompletedFactProjectionRouting`).
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub struct CompletedFactProjection {
    locators: BTreeMap<Arc<str>, usize>,
    by_ordinal: Vec<Arc<str>>,
    count: usize,
    physical: bool,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
    scopes: crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes,
    preimages: BTreeMap<
        worth_relational::facade::identity::EntityId,
        BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<usize>>>>,
    >,
}

impl PartialEq for CompletedFactProjection {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.locators == other.locators
    }
}

impl Eq for CompletedFactProjection {}

impl CompletedFactProjection {
    /// The routing completion derived from an admitted body; only the attempt's completion
    /// routing builds it, after funding `bytes` and checking every locator.
    pub(in crate::domain_computation::primary_graph) fn from_routing(
        locators: BTreeMap<Arc<str>, usize>,
        by_ordinal: Vec<Arc<str>>,
        physical: bool,
        bytes: u64,
        memory: Option<ExecutionMemoryReservation>,
        scopes: crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes,
        preimages: BTreeMap<
            worth_relational::facade::identity::EntityId,
            BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<usize>>>>,
        >,
    ) -> Self {
        Self {
            locators,
            by_ordinal,
            count: scopes.len(),
            physical,
            bytes,
            _memory: memory,
            scopes,
            preimages,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn scopes(&self) -> &crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes {
        &self.scopes
    }
    pub(in crate::domain_computation) fn count(&self) -> usize {
        self.count
    }
    pub(in crate::domain_computation) fn physical(&self) -> bool {
        self.physical
    }
    pub(in crate::domain_computation) fn ordinal(&self, locator: &str) -> Option<usize> {
        self.locators.get(locator).copied()
    }
    pub(in crate::domain_computation) fn locator(&self, ordinal: usize) -> Option<&str> {
        self.by_ordinal.get(ordinal).map(AsRef::as_ref)
    }
    pub(in crate::domain_computation) fn locators(&self) -> impl Iterator<Item = (&str, usize)> {
        self.locators
            .iter()
            .map(|(key, ordinal)| (key.as_ref(), *ordinal))
    }
    pub(in crate::domain_computation::primary_graph) fn bytes(&self) -> u64 {
        self.bytes
    }
    pub(in crate::domain_computation::primary_graph) fn preimage_ordinals(
        &self,
        entity: worth_relational::facade::identity::EntityId,
        locus: &worth_query_installation::facade::InstalledPreImageLocus,
    ) -> &[usize] {
        self.preimages
            .get(&entity)
            .and_then(|entities| entities.get(locus.entity()))
            .and_then(|aspects| aspects.get(locus.aspect()))
            .and_then(|fields| fields.get(locus.field()))
            .map_or(&[], Vec::as_slice)
    }
}
