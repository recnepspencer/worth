//! Materialize only native locators not already represented by accepted outputs.
use super::Denial;
use super::NativePriorCheckpointOutput;
use super::Structure;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;

impl NativePriorCheckpointOutput<'_> {
    pub(in crate::domain_computation::primary_graph) fn materialize_identity(
        &self,
    ) -> Result<Option<WorthQueryAcceptedOutputCheckpointIdentity>, Denial> {
        let Some(locator) = self.identity.as_ref() else {
            return Ok(None);
        };
        if let Some(recovered) = locator.recovered {
            // Preserve original posture/facts and the original immutable heaps'
            // payer. Reading this locator grants no new Ready or performed claim.

            return Ok(Some(recovered.clone()));
        }
        let recorded = locator.recorded;
        let partition = self
            .partition
            .ok_or(Denial::Structural(Structure::MissingSourcePartition))?;
        let role_views = recorded.correspondence.native_witness_roles();
        let role_count = role_views.len();

        let mut producer = String::new();
        producer
            .try_reserve_exact(locator.producer.len())
            .map_err(|error| Denial::allocation("producer locator text", error))?;
        producer.push_str(locator.producer);
        let mut roles = Vec::new();
        roles
            .try_reserve_exact(role_count)
            .map_err(|error| Denial::allocation("native output roles", error))?;
        for (role, posture, entity_name, entity) in role_views {
            let mut copied_role = String::new();
            copied_role
                .try_reserve_exact(role.len())
                .map_err(|error| Denial::allocation("native output role text", error))?;
            copied_role.push_str(role);
            let mut copied_entity_name = String::new();
            copied_entity_name
                .try_reserve_exact(entity_name.len())
                .map_err(|error| Denial::allocation("native output entity name", error))?;
            copied_entity_name.push_str(entity_name);
            roles.push(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole {
                    role: copied_role,
                    posture,
                    entity_name: copied_entity_name,
                    entity,
                },
            );
        }
        Ok(Some(
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
                producer: producer.into(),
                posture: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                source: locator.source,
                scope: self.scope,
                source_partition: partition,
                producer_dependency: recorded.producer_dependency_identity,
                idempotency_key: recorded.idempotency_key_identity,

                roles: roles.into(),
                producer_facts: None,
                producer_fact_wire_version: 0,
                metadata: None,
            },
        ))
    }
}
