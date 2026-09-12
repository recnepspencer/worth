use std::sync::{Arc, Mutex};

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, CustomInvariantVerdict,
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroupSet,
};

pub(super) const RULE_ID: &str = "supply-chain.graph-selected-state";

pub(super) struct GraphSelectedStateProbeRegistration {
    pub(super) registration: CustomInvariantRegistration,
    pub(super) expectation: GraphSelectedStateExpectation,
}

#[derive(Clone)]
pub(super) struct GraphSelectedStateExpectation(Arc<Mutex<Option<EntityId>>>);

impl GraphSelectedStateExpectation {
    pub(super) fn forbid(&self, entity: EntityId) {
        *self
            .0
            .lock()
            .expect("graph selected-state expectation lock") = Some(entity);
    }
}

pub(super) fn registration() -> GraphSelectedStateProbeRegistration {
    let forbidden_entity = Arc::new(Mutex::new(None));
    let registration = CustomInvariantRegistration::new(GraphSelectedStateProbe {
        forbidden_entity: Arc::clone(&forbidden_entity),
    })
    .expect("graph selected-state probe registers");
    GraphSelectedStateProbeRegistration {
        registration,
        expectation: GraphSelectedStateExpectation(forbidden_entity),
    }
}

#[derive(Clone)]
struct GraphSelectedStateProbe {
    forbidden_entity: Arc<Mutex<Option<EntityId>>>,
}

impl GraphSelectedStateProbe {
    fn forbidden_entity(&self) -> Option<EntityId> {
        *self
            .forbidden_entity
            .lock()
            .expect("graph selected-state expectation lock")
    }
}

impl CustomInvariantRule for GraphSelectedStateProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(RULE_ID),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Supply Chain graph selected-state probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1_000_000).unwrap(),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![super::world::supply_chain::entity_kind_id(
                        super::world::supply_chain::EntityKind::Vessel,
                    )],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![super::world::supply_chain::entity_kind_id(
                        super::world::supply_chain::EntityKind::Vessel,
                    )],
                    affected_relation_kinds: vec![],
                }
                .canonicalize(),
                execution_point: InvariantExecutionPoint::GraphComposition,
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
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
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let forbidden_is_visible = self.forbidden_entity().is_some_and(|entity| {
            context
                .committed_aspect_states()
                .entity_aspect_state(entity)
                .is_some()
        });
        Ok(if forbidden_is_visible {
            CustomInvariantVerdict::Violation
        } else {
            CustomInvariantVerdict::Pass
        })
    }
}
