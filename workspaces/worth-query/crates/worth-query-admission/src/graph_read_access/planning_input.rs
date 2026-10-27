use worth_foundational::facade::{AspectKey, CanonicalDigestId, FieldKey};

use super::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphReadFanoutPosture,
    WorthQueryGraphReadLifecycleClass, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPredicateFamily, WorthQueryGraphReadResultPressure,
    WorthQueryGraphReadTraversalOperator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanningIdentity {
    read_graph_digest: CanonicalDigestId,
    access_shape_digest: CanonicalDigestId,
    selectivity_shape_digest: CanonicalDigestId,
    schema_basis_digest: CanonicalDigestId,
}

impl WorthQueryGraphReadPlanningIdentity {
    pub fn from_admitted_evidence(
        read_graph_digest: CanonicalDigestId,
        access_shape_digest: CanonicalDigestId,
        selectivity_shape_digest: CanonicalDigestId,
        schema_basis_digest: CanonicalDigestId,
    ) -> Self {
        Self {
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            schema_basis_digest,
        }
    }

    pub const fn read_graph_digest(&self) -> &CanonicalDigestId {
        &self.read_graph_digest
    }

    pub const fn access_shape_digest(&self) -> &CanonicalDigestId {
        &self.access_shape_digest
    }

    pub const fn selectivity_shape_digest(&self) -> &CanonicalDigestId {
        &self.selectivity_shape_digest
    }

    pub const fn schema_basis_digest(&self) -> &CanonicalDigestId {
        &self.schema_basis_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanningRelation {
    relation_name: String,
    direction: WorthQueryAdmittedGraphReadRelationDirection,
    depth: usize,
    operators: Vec<WorthQueryGraphReadTraversalOperator>,
}

impl WorthQueryGraphReadPlanningRelation {
    pub fn from_admitted_reference(
        relation_name: impl Into<String>,
        direction: WorthQueryAdmittedGraphReadRelationDirection,
        depth: usize,
        mut operators: Vec<WorthQueryGraphReadTraversalOperator>,
    ) -> Self {
        operators.sort_by_key(|operator| operator.as_str());
        operators.dedup();
        Self {
            relation_name: relation_name.into(),
            direction,
            depth,
            operators,
        }
    }

    pub fn relation_name(&self) -> &str {
        &self.relation_name
    }

    pub fn direction(&self) -> &WorthQueryAdmittedGraphReadRelationDirection {
        &self.direction
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }

    pub fn operators(&self) -> &[WorthQueryGraphReadTraversalOperator] {
        &self.operators
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanningPredicateField {
    aspect: AspectKey,
    field: FieldKey,
    native_family: String,
}

impl WorthQueryGraphReadPlanningPredicateField {
    pub fn from_admitted_field(
        aspect: AspectKey,
        field: FieldKey,
        native_family: impl Into<String>,
    ) -> Self {
        Self {
            aspect,
            field,
            native_family: native_family.into(),
        }
    }

    pub fn aspect(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field(&self) -> &FieldKey {
        &self.field
    }

    pub fn native_family(&self) -> &str {
        &self.native_family
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanningOrderingField {
    collection_path: String,
    aspect: AspectKey,
    field: FieldKey,
    direction: String,
    native_family: String,
}

impl WorthQueryGraphReadPlanningOrderingField {
    pub fn from_admitted_field(
        collection_path: impl Into<String>,
        aspect: AspectKey,
        field: FieldKey,
        direction: impl Into<String>,
        native_family: impl Into<String>,
    ) -> Self {
        Self {
            collection_path: collection_path.into(),
            aspect,
            field,
            direction: direction.into(),
            native_family: native_family.into(),
        }
    }

    pub fn collection_path(&self) -> &str {
        &self.collection_path
    }

    pub fn aspect(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field(&self) -> &FieldKey {
        &self.field
    }

    pub fn direction(&self) -> &str {
        &self.direction
    }

    pub fn native_family(&self) -> &str {
        &self.native_family
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanningShape {
    relations: Vec<WorthQueryGraphReadPlanningRelation>,
    fanout_posture: WorthQueryGraphReadFanoutPosture,
    predicate_family: WorthQueryGraphReadPredicateFamily,
    predicate_fields: Vec<WorthQueryGraphReadPlanningPredicateField>,
    ordering_posture: WorthQueryGraphReadOrderingPosture,
    ordering_fields: Vec<WorthQueryGraphReadPlanningOrderingField>,
    relationship_proof_required: bool,
    root_union_dedup_required: bool,
    component_selection_required: bool,
    lifecycle_class: WorthQueryGraphReadLifecycleClass,
    result_pressure: WorthQueryGraphReadResultPressure,
}

impl WorthQueryGraphReadPlanningShape {
    pub fn from_admitted_shape(
        relations: Vec<WorthQueryGraphReadPlanningRelation>,
        fanout_posture: WorthQueryGraphReadFanoutPosture,
        result_pressure: WorthQueryGraphReadResultPressure,
    ) -> Self {
        Self {
            relations,
            fanout_posture,
            predicate_family: WorthQueryGraphReadPredicateFamily::None,
            predicate_fields: Vec::new(),
            ordering_posture: WorthQueryGraphReadOrderingPosture::Unordered,
            ordering_fields: Vec::new(),
            relationship_proof_required: false,
            root_union_dedup_required: false,
            component_selection_required: false,
            lifecycle_class: WorthQueryGraphReadLifecycleClass::ReusableReadFamily,
            result_pressure,
        }
    }

    pub fn with_predicates(
        mut self,
        family: WorthQueryGraphReadPredicateFamily,
        fields: Vec<WorthQueryGraphReadPlanningPredicateField>,
    ) -> Self {
        self.predicate_family = family;
        self.predicate_fields = fields;
        self
    }

    pub fn with_ordering(
        mut self,
        posture: WorthQueryGraphReadOrderingPosture,
        fields: Vec<WorthQueryGraphReadPlanningOrderingField>,
    ) -> Self {
        self.ordering_posture = posture;
        self.ordering_fields = fields;
        self
    }

    pub fn with_relationship_proof_required(mut self, required: bool) -> Self {
        self.relationship_proof_required = required;
        self
    }

    pub fn with_component_selection_required(mut self, required: bool) -> Self {
        self.component_selection_required = required;
        self
    }
    pub const fn component_selection_required(&self) -> bool {
        self.component_selection_required
    }

    pub fn with_root_union_dedup_required(mut self, required: bool) -> Self {
        self.root_union_dedup_required = required;
        self
    }

    pub fn relations(&self) -> &[WorthQueryGraphReadPlanningRelation] {
        &self.relations
    }

    pub fn fanout_posture(&self) -> &WorthQueryGraphReadFanoutPosture {
        &self.fanout_posture
    }

    pub fn predicate_family(&self) -> &WorthQueryGraphReadPredicateFamily {
        &self.predicate_family
    }

    pub fn predicate_fields(&self) -> &[WorthQueryGraphReadPlanningPredicateField] {
        &self.predicate_fields
    }

    pub fn ordering_posture(&self) -> &WorthQueryGraphReadOrderingPosture {
        &self.ordering_posture
    }

    pub fn ordering_fields(&self) -> &[WorthQueryGraphReadPlanningOrderingField] {
        &self.ordering_fields
    }

    pub const fn relationship_proof_required(&self) -> bool {
        self.relationship_proof_required
    }

    pub const fn root_union_dedup_required(&self) -> bool {
        self.root_union_dedup_required
    }

    pub fn lifecycle_class(&self) -> &WorthQueryGraphReadLifecycleClass {
        &self.lifecycle_class
    }

    pub fn result_pressure(&self) -> &WorthQueryGraphReadResultPressure {
        &self.result_pressure
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCanonicalGraphReadPlanningInput {
    identity: WorthQueryGraphReadPlanningIdentity,
    shape: WorthQueryGraphReadPlanningShape,
    maximum_cardinality: Option<usize>,
    live_maintenance_required: bool,
}

impl WorthQueryCanonicalGraphReadPlanningInput {
    pub fn from_admitted_evidence(
        identity: WorthQueryGraphReadPlanningIdentity,
        shape: WorthQueryGraphReadPlanningShape,
    ) -> Self {
        Self {
            identity,
            shape,
            maximum_cardinality: None,
            live_maintenance_required: false,
        }
    }

    pub fn with_maximum_cardinality(mut self, maximum: usize) -> Self {
        self.maximum_cardinality = Some(maximum);
        self
    }

    pub fn with_live_maintenance_required(mut self, required: bool) -> Self {
        self.live_maintenance_required = required;
        self
    }

    pub fn identity(&self) -> &WorthQueryGraphReadPlanningIdentity {
        &self.identity
    }

    pub fn shape(&self) -> &WorthQueryGraphReadPlanningShape {
        &self.shape
    }

    pub const fn maximum_cardinality(&self) -> Option<usize> {
        self.maximum_cardinality
    }

    pub const fn live_maintenance_required(&self) -> bool {
        self.live_maintenance_required
    }
}
