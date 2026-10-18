//! Fresh witness projection funds its actual new aspect heaps before cloning.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
};
use worth_execution::ExecutionArrayBuilder;

impl PreparedNativeOutputFacts<'_> {
    pub(in crate::domain_computation::primary_graph) fn retain_payload(
        self,
        control: StorageControl<'_, '_>,
    ) -> Result<FactPayload, StoreDenial> {
        control.check_live()?;
        let heaps = control.policy().reserve_memory(self.retained_payload_bytes);
        control.check_live()?;
        let heaps = heaps?;
        let mut facts = ExecutionArrayBuilder::allocate(self.count, control.policy())?;
        for role in &self.witness.roles {
            control.check_live()?;
            let entity_id = role.entity.expect("prepared witness role is filled");
            facts.push(Fact::Entity {
                entity_id,
                kind: role.kind,
            })?;
            crate::domain_computation::primary_graph::fact_verification_observation::native_payload_fact_move();
            for aspect in &self.witness.aspects[role.first_aspect..role.end_aspect] {
                control.check_live()?;
                facts.push(Fact::SourceAspectRevision {
                    entity_id,
                    aspect: aspect.aspect.clone(),
                    native_revision: aspect.revision.expect("prepared aspect revision is filled"),
                })?;
                crate::domain_computation::primary_graph::fact_verification_observation::native_payload_fact_move();
            }
        }
        control.check_live()?;
        Ok(FactPayload::copied_native(facts.seal()?, heaps, None))
    }
}
