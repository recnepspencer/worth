use std::sync::Arc;

use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

use super::super::super::{
    WorthQueryDomainInstallationDenial, WorthQueryDomainInstallationDenialKind,
    WorthQueryDomainInvariantDefinition, WorthQueryDomainInvariantPredicate,
};

#[derive(Clone, Debug)]
struct InstalledRequiresOutgoingRelationsRule {
    rule_id: String,
    display_name: Arc<str>,
    semantic_version: CustomInvariantSemanticVersion,
    relevant_entity_kinds: Vec<KindId>,
    required_relation_kinds: Vec<KindId>,
    traversal_depth: u16,
    maximum_work_units: std::num::NonZeroU64,
}

#[derive(Clone, Debug)]
struct RequiresOutgoingRelationsScope {
    visible_relevant_entities: Vec<EntityId>,
    planned_relevant_entity_count: usize,
    planned_relevant_relation_count: usize,
    traversal_exhausted: bool,
}

pub(super) fn compile_invariant_definition(
    provenance: &crate::runtime::WorthQueryInstalledDomainSubstrateProvenance,
    definition: &WorthQueryDomainInvariantDefinition,
) -> Result<CustomInvariantRegistration, WorthQueryDomainInstallationDenial> {
    let semantic_version = CustomInvariantSemanticVersion::new(
        u16::try_from(definition.semantic_version().major()).map_err(|_| {
            WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
                format!(
                    "{} major version exceeds invariant runtime range",
                    definition.name().as_str()
                ),
            )
        })?,
        u16::try_from(definition.semantic_version().minor()).map_err(|_| {
            WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
                format!(
                    "{} minor version exceeds invariant runtime range",
                    definition.name().as_str()
                ),
            )
        })?,
    );
    let rule = match definition.predicate() {
        WorthQueryDomainInvariantPredicate::RequiresOutgoingRelations {
            relevant_entity_kinds,
            required_relation_kinds,
            traversal_depth,
        } => InstalledRequiresOutgoingRelationsRule {
            rule_id: format!(
                "{}.package-{}.{}.{}",
                provenance.domain_owner(),
                provenance.package_version(),
                provenance.package_identity(),
                definition.name().as_str()
            ),
            display_name: Arc::from(format!(
                "{} package {} {}",
                provenance.domain_owner(),
                provenance.package_version(),
                definition.name().as_str()
            )),
            semantic_version,
            relevant_entity_kinds: relevant_entity_kinds.clone(),
            required_relation_kinds: required_relation_kinds.clone(),
            traversal_depth: *traversal_depth,
            maximum_work_units: definition.maximum_work_units(),
        },
    };
    CustomInvariantRegistration::new(rule).map_err(|error| {
        WorthQueryDomainInstallationDenial::new(
            WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
            format!("{}: {error:?}", definition.name().as_str()),
        )
    })
}

impl CustomInvariantRule for InstalledRequiresOutgoingRelationsRule {
    type Scope = RequiresOutgoingRelationsScope;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.rule_id.clone()),
                semantic_version: self.semantic_version,
            },
            display_name: self.display_name.clone(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: self.maximum_work_units,
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: self.relevant_entity_kinds.clone(),
                    read_relation_kinds: self.required_relation_kinds.clone(),
                    affected_entity_kinds: self.relevant_entity_kinds.clone(),
                    affected_relation_kinds: self.required_relation_kinds.clone(),
                }
                .canonicalize(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let touched = planner.touched();
        let visible_relevant_entities = touched
            .visible_entity_ids()
            .iter()
            .copied()
            .filter(|entity_id| {
                planner
                    .relations()
                    .entity_kind(*entity_id)
                    .is_some_and(|kind| self.relevant_entity_kinds.contains(&kind))
            })
            .collect::<Vec<_>>();
        let traversal = planner
            .traversal()
            .walk_outgoing_from(&visible_relevant_entities, self.traversal_depth as usize)?;
        Ok(RequiresOutgoingRelationsScope {
            visible_relevant_entities,
            planned_relevant_entity_count: touched
                .planned_entity_creates()
                .iter()
                .filter(|create| self.relevant_entity_kinds.contains(&create.kind_id()))
                .count(),
            planned_relevant_relation_count: touched
                .planned_relation_creates()
                .iter()
                .filter(|create| self.required_relation_kinds.contains(&create.kind_id()))
                .count(),
            traversal_exhausted: traversal.frontier_exhausted(),
        })
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if !scope.traversal_exhausted {
            return Ok(CustomInvariantVerdict::Violation);
        }
        if scope.visible_relevant_entities.is_empty()
            && scope.planned_relevant_entity_count == 0
            && scope.planned_relevant_relation_count == 0
        {
            return Ok(CustomInvariantVerdict::Pass);
        }
        for entity in &scope.visible_relevant_entities {
            if !self.visible_entity_satisfies(context, *entity)? {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

impl InstalledRequiresOutgoingRelationsRule {
    fn visible_entity_satisfies(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        entity: EntityId,
    ) -> Result<bool, CustomInvariantExecutionError> {
        let actual = context
            .relations()
            .outgoing_relations_for_entity(entity)?
            .into_iter()
            .filter_map(|relation_id| context.relations().relation(relation_id))
            .map(|relation| relation.kind_id)
            .collect::<Vec<_>>();
        Ok(self
            .required_relation_kinds
            .iter()
            .all(|required| actual.contains(required)))
    }
}
