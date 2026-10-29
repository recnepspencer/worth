pub(crate) use {std::sync::Arc, std::sync::Mutex};

pub(crate) use {worth_relational::facade::identity::EntityId};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRefusal, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet};

pub(crate) const RULE_ID: &str = "supply-chain.graph-selected-state";

pub(crate) struct GraphSelectedStateProbeRegistration {
    pub(crate) registration: CustomInvariantRegistration,
    pub(crate) expectation: GraphSelectedStateExpectation,
}

#[derive(Clone)]
pub(crate) struct GraphSelectedStateExpectation(pub(crate) Arc<Mutex<Option<EntityId>>>);

impl GraphSelectedStateExpectation {
    pub(crate) fn forbid(&self, entity: EntityId) {
        *self
            .0
            .lock()
            .expect("graph selected-state expectation lock") = Some(entity);
    }
}

pub(crate) fn registration() -> GraphSelectedStateProbeRegistration {
    let forbidden_entity = std::sync::Arc::new(std::sync::Mutex::new(None));
    let registration = worth_relational::facade::runtime::CustomInvariantRegistration::new(GraphSelectedStateProbe {
        forbidden_entity: std::sync::Arc::clone(&forbidden_entity),
    })
    .expect("graph selected-state probe registers");
    GraphSelectedStateProbeRegistration {
        registration,
        expectation: GraphSelectedStateExpectation(forbidden_entity),
    }
}

#[derive(Clone)]
pub(crate) struct GraphSelectedStateProbe {
    pub(crate) forbidden_entity: Arc<Mutex<Option<EntityId>>>,
}

impl GraphSelectedStateProbe {
    pub(crate) fn forbidden_entity(&self) -> Option<EntityId> {
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
                semantic_version: worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: std::sync::Arc::from("Supply Chain graph selected-state probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1_000_000).unwrap()),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(
                        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
                    )],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(
                        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
                    )],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: true,
                }
                .canonicalize(),
                execution_point: worth_relational::facade::runtime::InvariantExecutionPoint::GraphComposition,
                groups: worth_relational::facade::runtime::InvariantGroupSet::all(),
                cost_class: worth_relational::facade::runtime::InvariantCostClass::Touched,
                failure_effect: worth_relational::facade::runtime::InvariantFailureEffect::BlockCommit,
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
                .is_ok()
        });
        Ok(if forbidden_is_visible {
            worth_relational::facade::runtime::CustomInvariantVerdict::Violation(worth_relational::facade::runtime::CustomInvariantRefusal::Empty)
        } else {
            worth_relational::facade::runtime::CustomInvariantVerdict::Pass
        })
    }
}