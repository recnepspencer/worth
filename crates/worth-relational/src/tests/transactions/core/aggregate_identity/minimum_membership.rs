//! A publication rule judges generated membership edges in the final candidate.
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::*;
use std::sync::{Arc, OnceLock};

pub(super) struct MinimumMembership(pub(super) Arc<OnceLock<EntityId>>);

impl CustomInvariantRule for MinimumMembership {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.aggregate.minimum-membership"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Aggregate membership minimum".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(3)],
                    read_relation_kinds: vec![KindId(5)],
                    affected_entity_kinds: vec![KindId(1), KindId(3)],
                    affected_relation_kinds: vec![KindId(2), KindId(5)],
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: InvariantExecutionPoint::SnapshotPublication,
                groups: InvariantGroupSet::of(InvariantGroup::PublicationCoherence),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockPublication,
            },
        }
    }

    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let Some(member) = self.0.get() else {
            return Ok(CustomInvariantVerdict::Pass);
        };
        let graph = context.relations();
        let memberships = graph
            .outgoing_relations_for_entity(*member)
            .map_err(|error| CustomInvariantExecutionError::new(error.detail()))?;
        let [membership] = memberships.as_slice() else {
            return Err(CustomInvariantExecutionError::new(
                "expected one stored membership",
            ));
        };
        let group = graph
            .relation(*membership)
            .map_err(|error| CustomInvariantExecutionError::new(format!("{error:?}")))?
            .target;
        let siblings = graph
            .incoming_relations_for_entity(group)
            .map_err(|error| CustomInvariantExecutionError::new(error.detail()))?;
        if siblings.len() < 2 {
            return Ok(CustomInvariantVerdict::Violation(
                CustomInvariantRefusal::Empty,
            ));
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
