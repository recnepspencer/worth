use super::validation_engine_fixtures::*;
use std::sync::Arc;

pub(super) struct AlwaysViolatesCustomRule;
pub(super) struct GraphCompositionViolatesCustomRule;
pub(super) struct StructuralSurfaceRule;
pub(super) struct PanicDuringPrepareRule;
pub(super) struct PanicDuringEvaluateRule;

impl CustomInvariantRule for AlwaysViolatesCustomRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom.violation"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Custom Violation"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Violation)
    }
}

impl CustomInvariantRule for GraphCompositionViolatesCustomRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom.graph-composition"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Graph Composition Custom Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::GraphComposition,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Violation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StructuralScope {
    visible_entities: usize,
    planned_relations: usize,
    touched_partitions: usize,
}

impl CustomInvariantRule for StructuralSurfaceRule {
    type Scope = StructuralScope;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom.structural-surface"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Structural Surface Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let source_entities = planner.touched().visible_entity_ids();
        let traversal = planner.traversal().walk_outgoing_from(source_entities, 1)?;
        assert!(traversal.frontier_exhausted());
        Ok(StructuralScope {
            visible_entities: source_entities.len(),
            planned_relations: planner.touched().planned_relation_creates().len(),
            touched_partitions: planner.touched().touched_partitions().len(),
        })
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let counts = context.counts();
        if counts.visible_entity_count() == scope.visible_entities
            && counts.planned_relation_create_count() == scope.planned_relations
            && counts.touched_partition_count() == scope.touched_partitions
        {
            Ok(CustomInvariantVerdict::Pass)
        } else {
            Ok(CustomInvariantVerdict::Violation)
        }
    }
}

impl CustomInvariantRule for PanicDuringPrepareRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom.panic-prepare"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Panic During Prepare"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        panic!("prepare panic");
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

impl CustomInvariantRule for PanicDuringEvaluateRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom.panic-evaluate"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Panic During Evaluate"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract {
                    read_entity_kinds: vec![crate::identity::data::KindId(1)],
                    read_relation_kinds: vec![crate::identity::data::KindId(2)],
                    affected_entity_kinds: vec![crate::identity::data::KindId(1)],
                    affected_relation_kinds: vec![crate::identity::data::KindId(2)],
                },
                execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: crate::validation::data::InvariantCostClass::Touched,
                failure_effect: crate::validation::data::InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        panic!("evaluate panic");
    }
}
