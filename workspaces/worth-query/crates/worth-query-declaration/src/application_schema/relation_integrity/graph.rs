//! Graph laws shared by stored relations and derived endpoint sets.
use super::{
    ApplicationRelationCardinality, ApplicationRelationEndpoints,
    ApplicationRelationIntegrityRecord, ApplicationRelationMinimumReachabilityRecord,
};

/// Laws over canonical endpoint pairs. Duplicate supporting paths contribute
/// dependencies, not additional degree or reachability members.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationGraphLaws {
    pub endpoints: ApplicationRelationEndpoints,
    pub cardinality: ApplicationRelationCardinality,
    pub acyclic: bool,
    pub symmetric: bool,
    pub orbit: bool,
    pub same_partition_endpoints: bool,
    pub reaches_at_least: Option<ApplicationRelationMinimumReachabilityRecord>,
}

impl ApplicationRelationGraphLaws {
    /// Allows unbounded degree in one context, with self edges permitted.
    pub const fn unbounded() -> Self {
        Self {
            endpoints: ApplicationRelationEndpoints::same_context(true),
            cardinality: ApplicationRelationCardinality::unbounded(),
            acyclic: false,
            symmetric: false,
            orbit: false,
            same_partition_endpoints: false,
            reaches_at_least: None,
        }
    }

    /// Requires the canonical endpoint set to be acyclic.
    pub const fn acyclic(mut self) -> Self {
        self.acyclic = true;
        self
    }

    /// Requires a homogeneous successor orbit.
    pub const fn orbit(mut self) -> Self {
        self.orbit = true;
        self
    }

    pub(crate) fn borrowed(&self) -> ApplicationRelationGraphLawRef<'_> {
        ApplicationRelationGraphLawRef {
            endpoints: self.endpoints,
            cardinality: self.cardinality,
            acyclic: self.acyclic,
            symmetric: self.symmetric,
            orbit: self.orbit,
            same_partition_endpoints: self.same_partition_endpoints,
            reaches_at_least: self.reaches_at_least.as_ref(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ApplicationRelationGraphLawRef<'a> {
    pub(crate) endpoints: ApplicationRelationEndpoints,
    pub(crate) cardinality: ApplicationRelationCardinality,
    pub(crate) acyclic: bool,
    pub(crate) symmetric: bool,
    pub(crate) orbit: bool,
    pub(crate) same_partition_endpoints: bool,
    pub(crate) reaches_at_least: Option<&'a ApplicationRelationMinimumReachabilityRecord>,
}

impl ApplicationRelationIntegrityRecord {
    pub(crate) fn graph_laws(&self) -> ApplicationRelationGraphLawRef<'_> {
        ApplicationRelationGraphLawRef {
            endpoints: self.endpoints,
            cardinality: self.cardinality,
            acyclic: self.acyclic,
            symmetric: self.symmetric,
            orbit: self.orbit,
            same_partition_endpoints: self.same_partition_endpoints,
            reaches_at_least: self.reaches_at_least.as_ref(),
        }
    }
}

impl ApplicationRelationGraphLawRef<'_> {
    pub(crate) fn is_valid(&self, from: &str, to: &str) -> bool {
        self.cardinality.is_valid() && (!self.orbit || from == to)
    }

    pub(crate) fn minimum_kinds_are_valid(
        &self,
        entities: &std::collections::BTreeSet<&str>,
    ) -> bool {
        self.reaches_at_least
            .is_none_or(|minimum| super::minimum::minimum_kinds_are_valid(minimum, entities))
    }
}
