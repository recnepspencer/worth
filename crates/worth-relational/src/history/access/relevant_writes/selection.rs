use crate::history::data::RelationalTouchAdjacencyDirection;
use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::DerivedIndexId;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::transactions::data::RecordRef;
use worth_foundational::facade::{AspectFieldLocator, AspectKey, CanonicalFieldPath};

/// Complete meaning of the supported installed entity-field index. Descriptive
/// coordinates never grant source access or establish an index definition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct RelationalAuthoredEntityIndexDefinition {
    pub index_id: DerivedIndexId,
    pub name: String,
    pub branch_scoped: bool,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
    pub locator: AspectFieldLocator,
}

impl RelationalAuthoredEntityIndexDefinition {
    /// Coordinate grammar only. Actual installation and canonical coverage
    /// remain receiving-owner requirements; this never reissues authority.
    pub(crate) fn supports_locator(locator: &AspectFieldLocator) -> bool {
        matches!(
            locator.aspect().authority(),
            worth_foundational::facade::LocatorAuthority::Authoritative
                | worth_foundational::facade::LocatorAuthority::Planned
        ) && locator.field_path().fields().len() == 1
    }

    pub(crate) fn owned_bytes(&self) -> u64 {
        (self.name.capacity() as u64)
            .saturating_add(
                self.locator
                    .aspect()
                    .aspect_key()
                    .owned_allocation_capacity_bytes() as u64,
            )
            .saturating_add(self.locator.field_path().owned_allocation_capacity_bytes() as u64)
    }
}

/// Descriptive write scope. It grants no record, history, or disclosure access.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationalAuthoredWriteScope {
    Lifecycle,
    WholeAspect(AspectKey),
    Field(AspectKey, CanonicalFieldPath),
}

/// An exact record coordinate or full equality-index predicate. These select
/// authored history only after the runtime admits both actual source roots.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationalAuthoredWriteSelector {
    /// Complete live membership, including future entrants into an empty list.
    Adjacency {
        anchor: EntityId,
        kind: KindId,
        direction: RelationalTouchAdjacencyDirection,
    },
    Record {
        record: RecordRef,
        scope: RelationalAuthoredWriteScope,
    },
    EntityFieldIndex {
        definition: RelationalAuthoredEntityIndexDefinition,
        kind: KindId,
        value: AuthoritativeFieldComparisonKey,
    },
}

impl worth_execution::ChargedBytes for RelationalAuthoredWriteSelector {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Adjacency { .. } => 0,
            Self::EntityFieldIndex {
                definition, value, ..
            } => definition
                .owned_bytes()
                .saturating_add(value.owned_allocation_capacity_bytes()),
            Self::Record {
                scope: RelationalAuthoredWriteScope::Lifecycle,
                ..
            } => 0,
            Self::Record {
                scope: RelationalAuthoredWriteScope::WholeAspect(aspect),
                ..
            } => aspect.owned_allocation_capacity_bytes() as u64,
            Self::Record {
                scope: RelationalAuthoredWriteScope::Field(aspect, path),
                ..
            } => (aspect.owned_allocation_capacity_bytes() as u64)
                .saturating_add(path.owned_allocation_capacity_bytes() as u64),
        }
    }
}
