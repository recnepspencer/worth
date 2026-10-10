//! Materialize only native locators not already represented by accepted outputs.
use super::{Denial, NativePriorCheckpointOutput, Structure};
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    output_lineage::invalidation::InvalidationEditAdmission,
};

impl NativePriorCheckpointOutput<'_> {
    pub(in crate::domain_computation::primary_graph) fn materialize_identity(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<WorthQueryAcceptedOutputCheckpointIdentity>, Denial> {
        let Some(locator) = self.identity.as_ref() else {
            return Ok(None);
        };
        let recorded = locator.recorded;
        let partition = self
            .partition
            .ok_or(Denial::Structural(Structure::MissingSourcePartition))?;
        let role_views = recorded.correspondence.native_witness_roles();
        let role_count = role_views.len();
        let role_bytes = role_views
            .clone()
            .try_fold(0usize, |total, (role, _, entity_name, _)| {
                total
                    .checked_add(role.len())?
                    .checked_add(entity_name.len())?
                    .checked_add(64)
            })
            .ok_or(Denial::arithmetic("native role text layout"))?;
        let retained_bytes = std::mem::size_of::<
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
        >()
        .checked_add(locator.producer.len())
        .and_then(|bytes| bytes.checked_add(role_bytes))
        .and_then(|bytes| {
            bytes.checked_add(
                role_count.checked_mul(std::mem::size_of::<
                    crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole,
                >() + 2 * std::mem::size_of::<String>())?,
            )
        })
        .ok_or(Denial::arithmetic("native locator layout"))?;
        admission
            .admit_read_scratch(
                u64::try_from(retained_bytes)
                    .map_err(|_| Denial::arithmetic("checkpoint size or work conversion"))?,
            )
            .map_err(|stop| Denial::admission("native locator scratch", stop))?;
        admission
            .charge_external_work(
                u64::try_from(retained_bytes)
                    .map_err(|_| Denial::arithmetic("checkpoint size or work conversion"))?,
            )
            .map_err(|stop| Denial::admission("native locator copy work", stop))?;
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
                producer,
                posture: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                source: locator.source,
                scope: self.scope,
                source_partition: partition,
                producer_dependency: recorded.producer_dependency_identity,
                idempotency_key: recorded.idempotency_key_identity,
                resources: recorded.resources(),
                roles,
                producer_facts: None,
                producer_fact_wire_version: 0,
            },
        ))
    }
}
