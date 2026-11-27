use std::marker::PhantomData;

use super::super::value_decode::ValueDecodeSession;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationCardinality,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversal, ExactlyOneResult,
    ManyResults, OptionalOneResult,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_execution_naming::facade::ObservedEntityNames;
use worth_query_installation::facade::ApplicationReadableScalarValueBinding;
use worth_relational::facade::identity::EntityId;

pub mod denial;
pub mod disclosed;
pub mod required_field;
pub use denial::{
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionDenialKind,
};
pub mod optional_aspect;
pub mod persistent_name;
pub mod prerequisite_selection;
pub use prerequisite_selection::WorthQueryPrerequisiteSelection;
pub mod projected_tree;
pub use projected_tree::content::ContentWriter;
pub use projected_tree::content_storage::ContentScratch;
pub use projected_tree::content_storage::ProjectionContentStorageFault;
pub use projected_tree::ProjectionContentComparisonFault;
pub use projected_tree::{OwnedProjectionRows, ProjectionDisposal};
pub mod retained_projection;

pub use disclosed::{WorthQueryApplicationDisclosed, WorthQueryApplicationOmission};

pub use projected_tree::{
    WorthQueryApplicationDisclosedProjectionNode, WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryApplicationProjectedField, WorthQueryApplicationProjectedRelation,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationWorkingProjectionTree,
};

/// Query-owned authoritative row presented to domain projection code.
///
/// Construction is private. Reads require query-bound result selectors whose
/// slot, path meaning, field or relation contract, and cardinality match the
/// installed application query.
pub struct WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    node: WorthQueryApplicationDisclosedProjectionNode<'row>,
    governance: &'row dyn crate::ports::QueryGovernancePort,
    decode: &'row ValueDecodeSession<'row>,
    names: Option<&'row ObservedEntityNames>,
    _marker: PhantomData<fn() -> (Schema, Query)>,
}

/// The rows of a many-valued relation, read from a projection row.
///
/// Iterate to get one projection row per related result.
pub struct WorthQueryApplicationProjectionRows<'row, Schema, Query> {
    rows: &'row [WorthQueryApplicationProjectionNode],
    disclosure_parent: WorthQueryApplicationDisclosedProjectionNode<'row>,
    governance: &'row dyn crate::ports::QueryGovernancePort,
    decode: &'row ValueDecodeSession<'row>,
    names: Option<&'row ObservedEntityNames>,
    _marker: PhantomData<fn() -> (Schema, Query)>,
}

impl<'row, Schema, Query> WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    /// Returns the authorized graph identity represented by this projected row.
    ///
    /// Domain projectors can use this to distinguish repeated traversal of one
    /// entity from traversal of distinct entities without inferring identity
    /// from field values.
    pub const fn entity_id(&self) -> EntityId {
        self.node.entity_id()
    }

    pub fn field<Slot, Entity, Aspect, Field, Value>(
        &self,
        selector: ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>,
    ) -> Result<Value, WorthQueryApplicationProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,

        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }

    pub fn optional_field<Slot, Entity, Aspect, Field, Value>(
        &self,
        selector: ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>,
    ) -> Result<Option<Value>, WorthQueryApplicationProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_optional_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }

    pub fn optional<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            OptionalOneResult,
        >,
    ) -> Result<
        Option<WorthQueryApplicationProjectionRow<'_, Schema, Query>>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        match relation.rows() {
            [] => Ok(None),
            [row] => Ok(Some(self.related(row))),
            _ => Err(relation_cardinality_denial(relation)),
        }
    }

    pub fn one<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ExactlyOneResult,
        >,
    ) -> Result<
        WorthQueryApplicationProjectionRow<'_, Schema, Query>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        match relation.rows() {
            [row] => Ok(self.related(row)),
            _ => Err(relation_cardinality_denial(relation)),
        }
    }

    pub fn many<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ManyResults,
        >,
    ) -> Result<
        WorthQueryApplicationProjectionRows<'_, Schema, Query>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        Ok(WorthQueryApplicationProjectionRows {
            rows: relation.rows(),
            disclosure_parent: self.node,
            governance: self.governance,
            decode: self.decode,
            names: self.names,
            _marker: PhantomData,
        })
    }

    fn relation<Slot, Relation, From, To, Direction, Cardinality>(
        &self,
        selector: &ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
    ) -> Result<&WorthQueryApplicationProjectedRelation, WorthQueryApplicationProjectionDenial>
    where
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_relation(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::RelationOmitted)
    }

    pub const fn new(
        node: WorthQueryApplicationDisclosedProjectionNode<'row>,
        governance: &'row dyn crate::ports::QueryGovernancePort,
        decode: &'row ValueDecodeSession<'row>,
    ) -> Self {
        Self {
            node,
            governance,
            decode,
            names: None,
            _marker: PhantomData,
        }
    }

    /// Lends the persistent names the same read observed for this row's
    /// records, which its name slots and its related rows then project.
    pub const fn with_persistent_names(mut self, names: Option<&'row ObservedEntityNames>) -> Self {
        self.names = names;
        self
    }

    /// A related row under this one, with the same disclosure and names.
    pub(crate) fn related(
        &self,
        row: &'row WorthQueryApplicationProjectionNode,
    ) -> WorthQueryApplicationProjectionRow<'row, Schema, Query> {
        WorthQueryApplicationProjectionRow::new(self.node.child(row), self.governance, self.decode)
            .with_persistent_names(self.names)
    }
}

impl<'row, Schema, Query> WorthQueryApplicationProjectionRows<'row, Schema, Query> {
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationProjectionRow<'_, Schema, Query>> {
        self.rows.iter().map(|row| {
            WorthQueryApplicationProjectionRow::new(
                self.disclosure_parent.child(row),
                self.governance,
                self.decode,
            )
            .with_persistent_names(self.names)
        })
    }
}

/// Application code that turns one query result row into a typed value.
///
/// Implement `project` by reading fields and relations through the row's
/// query-bound selectors. Returning a denial refuses the whole read.
pub trait WorthQueryApplicationProjection<Schema, Query>: Sized {
    /// Whether `project` reads a row's persistent name, here or in a related
    /// row. A read then observes its rows' names in the same pass; a
    /// projection that leaves this `false` pays nothing and its name reads
    /// refuse `PersistentNamesUnavailable`.
    const PROJECTS_PERSISTENT_NAMES: bool = false;
    /// Whether `project` reads a row's group name (`group_name`), here or
    /// in a related row. Its rows' footprints then keep their members' group
    /// membership, which regrouping changes without touching any record. A
    /// hand-written projection that projects names is assumed to read group
    /// names too, so it never loses that dependency.
    const PROJECTS_GROUP_NAMES: bool = Self::PROJECTS_PERSISTENT_NAMES;

    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial>;
}

fn projection_denial(
    kind: WorthQueryApplicationProjectionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationProjectionDenial {
    WorthQueryApplicationProjectionDenial::new(kind, subject)
}

fn relation_cardinality_denial(
    relation: &WorthQueryApplicationProjectedRelation,
) -> WorthQueryApplicationProjectionDenial {
    projection_denial(
        WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch,
        relation.result_path(),
    )
}
