//! Mandatory candidate exclusivity, including descendants of a reparented node.
use super::{
    traversal::{walk, Read},
    *,
};
use crate::validation::data::*;
use crate::validation::StructuralPresenceDenial;
use std::sync::Arc;

impl RelationalModelOwnership {
    pub fn invariant_registration(
        self,
    ) -> Result<CustomInvariantRegistration, CustomInvariantRegistrationError> {
        let meaning = Arc::new(self);
        let mut registration =
            CustomInvariantRegistration::new(StructuralExclusivity(Arc::clone(&meaning)))?;
        registration.model_ownership = Some(meaning);
        Ok(registration)
    }
}

struct StructuralExclusivity(Arc<RelationalModelOwnership>);
impl CustomInvariantRule for StructuralExclusivity {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        let entities = self.0.entity_kinds();
        let relations = self.0.relation_kinds();
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("worth.model-ownership.exclusivity"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("One document owns each model entity"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: entities,
                    affected_entity_kinds: Vec::new(),
                    read_relation_kinds: relations.clone(),
                    affected_relation_kinds: relations,
                    include_relation_endpoint_entity_touches: false,
                }
                .canonicalize(),
            },
        }
    }

    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(if budget.checkpoint(0) {
            Ok(())
        } else {
            Err(CustomInvariantPreparationError::new(
                "ownership preparation stopped",
            ))
        })
    }
    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate(context, &()))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        match self.0.check_candidate(context) {
            Ok(()) => Ok(CustomInvariantVerdict::Pass),
            Err(RelationalModelOwnershipDenial::Conflict(conflict)) => {
                if !context.refusal_budget().claim_result(256) {
                    return Err(CustomInvariantExecutionError::new(
                        "ownership refusal retention stopped",
                    ));
                }
                let bytes = conflict
                    .encode()
                    .map_err(|cause| CustomInvariantExecutionError::new(cause.to_string()))?;
                Ok(CustomInvariantVerdict::Violation(
                    CustomInvariantRefusal::DeclaredValue {
                        value_identity: RelationalModelOwnershipConflict::VALUE_IDENTITY.to_owned(),
                        bytes,
                    },
                ))
            }
            Err(cause) => Err(CustomInvariantExecutionError::new(format!(
                "ownership observation: {cause:?}"
            ))),
        }
    }
}

impl RelationalModelOwnership {
    fn check_candidate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
    ) -> Result<(), RelationalModelOwnershipDenial> {
        let relations = context.relations();
        let mut read = CandidateRead(&relations);
        let mut affected = BTreeSet::new();
        for entity in changed_targets(context, &mut read)? {
            for domain in &self.domains {
                for descendant in walk(&mut read, entity, true, &domain.relations)? {
                    read.retain()?;
                    affected.insert(descendant);
                }
            }
        }
        for entity in affected {
            self.owner(&mut read, entity)?;
        }
        Ok(())
    }
}

/// Scalar edits cannot change ownership. Only a new or reparented ownership
/// edge can add an ancestor; removing an edge alone cannot create a second root.
fn changed_targets(
    context: &CustomInvariantExecutionContext<'_>,
    read: &mut CandidateRead<'_, '_>,
) -> Result<BTreeSet<EntityId>, RelationalModelOwnershipDenial> {
    let touched = context.touched();
    let mut targets = BTreeSet::new();
    for &id in touched.visible_relation_ids() {
        read.0
            .read_budget()
            .checkpoint(touched.planned_relation_deletes().len() as u64)
            .map_err(RelationalModelOwnershipDenial::Traversal)?;
        if touched.planned_relation_deletes().contains(&id) {
            continue;
        }
        let Some(after) = read.0.live_relation_record(id).map_err(presence_denial)? else {
            continue;
        };
        let before = context
            .committed_relations()
            .live_relation_record(id)
            .map_err(presence_denial)?;
        if before != Some(after) {
            read.retain()?;
            targets.insert(after.target);
        }
    }
    Ok(targets)
}

struct CandidateRead<'a, 'runtime>(&'a StructuralRelationView<'runtime>);
impl Read for CandidateRead<'_, '_> {
    fn charge_work(&mut self, units: u64) -> Result<(), RelationalModelOwnershipDenial> {
        self.0
            .read_budget()
            .checkpoint(units)
            .map_err(RelationalModelOwnershipDenial::Traversal)
    }

    fn kind(&mut self, entity: EntityId) -> Result<Option<KindId>, RelationalModelOwnershipDenial> {
        self.0
            .live_entity_kind(entity)
            .map_err(|cause| match cause {
                StructuralPresenceDenial::Read(cause) => {
                    RelationalModelOwnershipDenial::Read(cause)
                }
                StructuralPresenceDenial::Request(cause) => {
                    RelationalModelOwnershipDenial::Traversal(cause)
                }
                StructuralPresenceDenial::Native(cause) => {
                    RelationalModelOwnershipDenial::Native(cause)
                }
            })
    }
    fn adjacent(
        &mut self,
        entity: EntityId,
        outgoing: bool,
        relations: &[KindId],
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial> {
        let edges = self
            .0
            .live_relations_for_entity(entity, outgoing)
            .map_err(presence_denial)?;
        let mut neighbors = Vec::new();
        for edge in edges {
            self.0
                .read_budget()
                .checkpoint(relations.len() as u64)
                .map_err(RelationalModelOwnershipDenial::Traversal)?;
            if relations.contains(&edge.kind_id) {
                self.retain()?;
                neighbors.push(if outgoing { edge.target } else { edge.source });
            }
        }
        Ok(neighbors)
    }
    fn retain(&mut self) -> Result<(), RelationalModelOwnershipDenial> {
        self.0
            .read_budget()
            .claim_retained(512)
            .map_err(RelationalModelOwnershipDenial::Traversal)
    }
}

fn presence_denial(cause: StructuralPresenceDenial) -> RelationalModelOwnershipDenial {
    match cause {
        StructuralPresenceDenial::Read(cause) => RelationalModelOwnershipDenial::Read(cause),
        StructuralPresenceDenial::Request(cause) => {
            RelationalModelOwnershipDenial::Traversal(cause)
        }
        StructuralPresenceDenial::Native(cause) => RelationalModelOwnershipDenial::Native(cause),
    }
}
