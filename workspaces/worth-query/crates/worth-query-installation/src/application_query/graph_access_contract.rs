use std::sync::Arc;

use worth_foundational::facade::{AspectKey, CanonicalDigestId, FieldKey, ScalarAspectType};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection, ApplicationQueryResultSlotKey,
    ApplicationQueryResultTraversalDirection,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

mod compilation;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledGraphProjection {
    slot_key: Arc<ApplicationQueryResultSlotKey>,
    parent_path: Arc<str>,
    result_path: Arc<str>,
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    output_name: String,
    scalar_family: ScalarAspectType,
    value_type: WorthQueryPortableTypeIdentity,
    presence: ApplicationFieldPresence,
}

impl WorthQueryInstalledGraphProjection {
    pub fn slot_key_identity(&self) -> Arc<ApplicationQueryResultSlotKey> {
        Arc::clone(&self.slot_key)
    }

    pub fn parent_path(&self) -> &str {
        &self.parent_path
    }

    pub fn result_path(&self) -> &str {
        &self.result_path
    }

    pub fn result_path_identity(&self) -> Arc<str> {
        Arc::clone(&self.result_path)
    }

    pub fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn slot_type(&self) -> &str {
        self.slot_type.as_str()
    }

    pub fn portable_slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot_type.clone()
    }

    pub fn slot_type_identity(&self) -> Arc<str> {
        Arc::from(self.slot_type.as_str())
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        self.aspect.as_str()
    }

    pub fn field(&self) -> &str {
        self.field.as_str()
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub fn value_type(&self) -> &str {
        self.value_type.as_str()
    }

    pub const fn presence(&self) -> ApplicationFieldPresence {
        self.presence
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledGraphRelation {
    slot_key: Arc<ApplicationQueryResultSlotKey>,
    parent_path: Arc<str>,
    result_path: Arc<str>,
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    relation: String,
    from: String,
    to: String,
    direction: ApplicationQueryResultTraversalDirection,
    output_name: String,
    cardinality: ApplicationQueryCardinality,
    predicate: Option<WorthQueryInstalledGraphPredicate>,
    depth: usize,
}

impl WorthQueryInstalledGraphRelation {
    pub fn slot_key_identity(&self) -> Arc<ApplicationQueryResultSlotKey> {
        Arc::clone(&self.slot_key)
    }

    pub fn parent_path(&self) -> &str {
        &self.parent_path
    }

    pub fn result_path(&self) -> &str {
        &self.result_path
    }

    pub fn result_path_identity(&self) -> Arc<str> {
        Arc::clone(&self.result_path)
    }

    pub fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn slot_type(&self) -> &str {
        self.slot_type.as_str()
    }

    pub fn portable_slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot_type.clone()
    }

    pub fn slot_type_identity(&self) -> Arc<str> {
        Arc::from(self.slot_type.as_str())
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> ApplicationQueryResultTraversalDirection {
        self.direction
    }

    pub fn parent_entity(&self) -> &str {
        match self.direction {
            ApplicationQueryResultTraversalDirection::Forward => &self.from,
            ApplicationQueryResultTraversalDirection::Reverse => &self.to,
        }
    }

    pub fn child_entity(&self) -> &str {
        match self.direction {
            ApplicationQueryResultTraversalDirection::Forward => &self.to,
            ApplicationQueryResultTraversalDirection::Reverse => &self.from,
        }
    }

    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        self.cardinality
    }

    pub fn predicate(&self) -> Option<&WorthQueryInstalledGraphPredicate> {
        self.predicate.as_ref()
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledGraphPredicate {
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    parameter: String,
    scalar_family: ScalarAspectType,
}

impl WorthQueryInstalledGraphPredicate {
    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, self.aspect.as_str(), self.field.as_str())
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn parameter(&self) -> &str {
        &self.parameter
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledGraphOrdering {
    result_path: String,
    collection_path: String,
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    output_name: String,
    direction: ApplicationQueryOrderingDirection,
    scalar_family: ScalarAspectType,
    value_type: WorthQueryPortableTypeIdentity,
}

impl WorthQueryInstalledGraphOrdering {
    pub fn result_path(&self) -> &str {
        &self.result_path
    }

    pub fn collection_path(&self) -> &str {
        &self.collection_path
    }

    pub fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn slot_type(&self) -> &str {
        self.slot_type.as_str()
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, self.aspect.as_str(), self.field.as_str())
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    pub const fn direction(&self) -> ApplicationQueryOrderingDirection {
        self.direction
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub fn value_type(&self) -> &str {
        self.value_type.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryInstalledGraphReadMeaning {
    pub(super) schema_basis_digest: worth_foundational::facade::CanonicalDigestId,
    pub(super) root_entity: String,
    pub(super) cardinality: ApplicationQueryCardinality,
    pub(super) projections: Vec<WorthQueryInstalledGraphProjection>,
    pub(super) relations: Vec<WorthQueryInstalledGraphRelation>,
    pub(super) root_paths: Vec<super::WorthQueryInstalledRootPath>,
    pub(super) predicates: Vec<WorthQueryInstalledGraphPredicate>,
    pub(super) ordering: Vec<WorthQueryInstalledGraphOrdering>,
    pub(super) maximum_traversal_depth: usize,
    pub(super) maximum_result_count: usize,
}

pub(super) struct WorthQueryInstalledGraphPlanningPreparation<'a> {
    pub(super) meaning: &'a WorthQueryInstalledGraphReadMeaning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphReadContract {
    canonical: super::WorthQueryApplicationCanonicalArtifact,
    planning: super::WorthQueryApplicationCanonicalArtifact,
    meaning: WorthQueryInstalledGraphReadMeaning,
}

impl WorthQueryInstalledGraphReadContract {
    pub(super) fn meaning(&self) -> &WorthQueryInstalledGraphReadMeaning {
        &self.meaning
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        self.canonical.digest()
    }

    pub fn canonical_basis(&self) -> &super::WorthQueryApplicationCanonicalArtifact {
        &self.canonical
    }

    pub fn canonical_planning_basis(&self) -> &super::WorthQueryApplicationCanonicalArtifact {
        &self.planning
    }

    pub const fn schema_basis_digest(&self) -> &worth_foundational::facade::CanonicalDigestId {
        &self.meaning.schema_basis_digest
    }

    pub fn root_entity(&self) -> &str {
        &self.meaning.root_entity
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        self.meaning.cardinality
    }

    pub fn projections(&self) -> &[WorthQueryInstalledGraphProjection] {
        &self.meaning.projections
    }

    pub fn relations(&self) -> &[WorthQueryInstalledGraphRelation] {
        &self.meaning.relations
    }

    pub fn root_paths(&self) -> &[super::WorthQueryInstalledRootPath] {
        &self.meaning.root_paths
    }

    pub fn predicates(&self) -> &[WorthQueryInstalledGraphPredicate] {
        &self.meaning.predicates
    }

    pub fn ordering(&self) -> &[WorthQueryInstalledGraphOrdering] {
        &self.meaning.ordering
    }

    pub const fn maximum_traversal_depth(&self) -> usize {
        self.meaning.maximum_traversal_depth
    }

    pub const fn maximum_result_count(&self) -> usize {
        self.meaning.maximum_result_count
    }
}
