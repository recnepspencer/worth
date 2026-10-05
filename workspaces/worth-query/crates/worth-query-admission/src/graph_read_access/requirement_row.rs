use super::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphReadAccessComplexityContract,
    WorthQueryGraphReadAccessInvalidationBasis, WorthQueryGraphReadAccessMemoryEstimateBasis,
    WorthQueryGraphReadAccessRebuildBasis, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadLifecycleClass,
    WorthQueryGraphReadOperationCapabilityRequirement, WorthQueryGraphReadOrderingFieldAuthority,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadPredicateFieldAuthority, WorthQueryGraphReadRelationAuthority,
    WorthQueryGraphReadResultPressure, WorthQueryGraphReadTraversalOperator,
};

mod digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessRequirementRow {
    kind: WorthQueryGraphReadAccessRequirementKind,
    rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
    relation_name: Option<String>,
    relation_authority: Option<WorthQueryGraphReadRelationAuthority>,
    relation_direction: Option<WorthQueryAdmittedGraphReadRelationDirection>,
    relation_depth: Option<usize>,
    fanout_posture: Option<WorthQueryGraphReadFanoutPosture>,
    predicate_family: Option<WorthQueryGraphReadPredicateFamily>,
    predicate_field_authorities: Vec<WorthQueryGraphReadPredicateFieldAuthority>,
    ordering_posture: Option<WorthQueryGraphReadOrderingPosture>,
    ordering_field_authorities: Vec<WorthQueryGraphReadOrderingFieldAuthority>,
    traversal_operator: Option<WorthQueryGraphReadTraversalOperator>,
    lifecycle_class: Option<WorthQueryGraphReadLifecycleClass>,
    result_pressure: Option<WorthQueryGraphReadResultPressure>,
    operation_capability_requirement: Option<WorthQueryGraphReadOperationCapabilityRequirement>,
    invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
    complexity_contract: WorthQueryGraphReadAccessComplexityContract,
    memory_estimate_basis: WorthQueryGraphReadAccessMemoryEstimateBasis,
    maximum_cardinality: Option<usize>,
}

impl WorthQueryGraphReadAccessRequirementRow {
    pub fn new(
        kind: WorthQueryGraphReadAccessRequirementKind,
        rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
        invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
        complexity_contract: WorthQueryGraphReadAccessComplexityContract,
        memory_estimate_basis: WorthQueryGraphReadAccessMemoryEstimateBasis,
    ) -> Self {
        Self {
            kind,
            rebuild_basis,
            relation_name: None,
            relation_authority: None,
            relation_direction: None,
            relation_depth: None,
            fanout_posture: None,
            predicate_family: None,
            predicate_field_authorities: Vec::new(),
            ordering_posture: None,
            ordering_field_authorities: Vec::new(),
            traversal_operator: None,
            lifecycle_class: None,
            result_pressure: None,
            operation_capability_requirement: None,
            invalidation_basis,
            complexity_contract,
            memory_estimate_basis,
            maximum_cardinality: None,
        }
    }

    pub fn kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.kind
    }

    pub fn rebuild_basis(&self) -> &WorthQueryGraphReadAccessRebuildBasis {
        &self.rebuild_basis
    }

    pub fn relation_name(&self) -> Option<&str> {
        self.relation_name.as_deref()
    }

    pub fn relation_authority(&self) -> Option<&WorthQueryGraphReadRelationAuthority> {
        self.relation_authority.as_ref()
    }

    pub fn relation_direction(&self) -> Option<&WorthQueryAdmittedGraphReadRelationDirection> {
        self.relation_direction.as_ref()
    }

    pub fn relation_depth(&self) -> Option<usize> {
        self.relation_depth
    }

    pub fn fanout_posture(&self) -> Option<&WorthQueryGraphReadFanoutPosture> {
        self.fanout_posture.as_ref()
    }

    pub fn predicate_family(&self) -> Option<&WorthQueryGraphReadPredicateFamily> {
        self.predicate_family.as_ref()
    }

    pub fn predicate_field_authorities(&self) -> &[WorthQueryGraphReadPredicateFieldAuthority] {
        &self.predicate_field_authorities
    }

    pub fn ordering_posture(&self) -> Option<&WorthQueryGraphReadOrderingPosture> {
        self.ordering_posture.as_ref()
    }

    pub fn ordering_field_authorities(&self) -> &[WorthQueryGraphReadOrderingFieldAuthority] {
        &self.ordering_field_authorities
    }

    pub fn traversal_operator(&self) -> Option<&WorthQueryGraphReadTraversalOperator> {
        self.traversal_operator.as_ref()
    }

    pub fn lifecycle_class(&self) -> Option<&WorthQueryGraphReadLifecycleClass> {
        self.lifecycle_class.as_ref()
    }

    pub fn result_pressure(&self) -> Option<&WorthQueryGraphReadResultPressure> {
        self.result_pressure.as_ref()
    }

    pub fn operation_capability_requirement(
        &self,
    ) -> Option<&WorthQueryGraphReadOperationCapabilityRequirement> {
        self.operation_capability_requirement.as_ref()
    }

    pub fn invalidation_basis(&self) -> &WorthQueryGraphReadAccessInvalidationBasis {
        &self.invalidation_basis
    }

    pub fn complexity_contract(&self) -> &WorthQueryGraphReadAccessComplexityContract {
        &self.complexity_contract
    }

    pub fn memory_estimate_basis(&self) -> &WorthQueryGraphReadAccessMemoryEstimateBasis {
        &self.memory_estimate_basis
    }

    pub fn maximum_cardinality(&self) -> Option<usize> {
        self.maximum_cardinality
    }

    pub fn semantic_slot_key(&self) -> String {
        let mut text = String::new();
        self.write_semantic_slot_key(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub fn with_relation(
        mut self,
        relation_name: impl Into<String>,
        relation_authority: WorthQueryGraphReadRelationAuthority,
        relation_direction: WorthQueryAdmittedGraphReadRelationDirection,
        relation_depth: usize,
    ) -> Self {
        self.relation_name = Some(relation_name.into());
        self.relation_authority = Some(relation_authority);
        self.relation_direction = Some(relation_direction);
        self.relation_depth = Some(relation_depth);
        self
    }

    pub fn with_fanout_posture(mut self, value: WorthQueryGraphReadFanoutPosture) -> Self {
        self.fanout_posture = Some(value);
        self
    }

    pub fn with_predicate_family(mut self, value: WorthQueryGraphReadPredicateFamily) -> Self {
        self.predicate_family = Some(value);
        self
    }

    pub fn with_predicate_field_authorities(
        mut self,
        mut values: Vec<WorthQueryGraphReadPredicateFieldAuthority>,
    ) -> Self {
        values.sort_by_key(|value| value.digest_part());
        values.dedup();
        self.predicate_field_authorities = values;
        self
    }

    pub(crate) fn with_predicate_field_authorities_sorted(
        mut self,
        values: Vec<WorthQueryGraphReadPredicateFieldAuthority>,
    ) -> Self {
        self.predicate_field_authorities = values;
        self
    }

    pub fn with_ordering_posture(mut self, value: WorthQueryGraphReadOrderingPosture) -> Self {
        self.ordering_posture = Some(value);
        self
    }

    pub fn with_ordering_field_authorities(
        mut self,
        mut values: Vec<WorthQueryGraphReadOrderingFieldAuthority>,
    ) -> Self {
        values.sort_by_key(|value| value.digest_part());
        values.dedup();
        self.ordering_field_authorities = values;
        self
    }

    pub(crate) fn with_ordering_field_authorities_sorted(
        mut self,
        values: Vec<WorthQueryGraphReadOrderingFieldAuthority>,
    ) -> Self {
        self.ordering_field_authorities = values;
        self
    }

    pub fn with_traversal_operator(mut self, value: WorthQueryGraphReadTraversalOperator) -> Self {
        self.traversal_operator = Some(value);
        self
    }

    pub fn with_lifecycle_class(mut self, value: WorthQueryGraphReadLifecycleClass) -> Self {
        self.lifecycle_class = Some(value);
        self
    }

    pub fn with_result_pressure(mut self, value: WorthQueryGraphReadResultPressure) -> Self {
        self.result_pressure = Some(value);
        self
    }

    pub fn with_operation_capability_requirement(
        mut self,
        value: WorthQueryGraphReadOperationCapabilityRequirement,
    ) -> Self {
        self.operation_capability_requirement = Some(value);
        self
    }

    pub fn with_maximum_cardinality(mut self, value: usize) -> Self {
        self.maximum_cardinality = Some(value);
        self
    }
}

trait Named {
    fn as_str(&self) -> &'static str;
}

macro_rules! named_impl {
    ($($type:ty),+ $(,)?) => {$(
        impl Named for $type {
            fn as_str(&self) -> &'static str {
                self.as_str()
            }
        }
    )+};
}

named_impl!(
    WorthQueryAdmittedGraphReadRelationDirection,
    WorthQueryGraphReadFanoutPosture,
    WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadTraversalOperator,
    WorthQueryGraphReadLifecycleClass,
    WorthQueryGraphReadResultPressure,
);

fn option_name<T: Named>(value: Option<&T>) -> &'static str {
    value.map_or("none", Named::as_str)
}
