use sha2::{Digest, Sha256};

use super::{
    resources, WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes,
    BODY_PREFIX_BYTES, CHECKSUM_BYTES, FORMAT_VERSION, HEADER_BYTES, MAGIC,
    MINIMUM_V5_ACCEPTED_OUTPUT_BYTES,
};

impl WorthQueryApplicationCheckpoint {
    pub(super) fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
    ) -> (Self, WorthQueryApplicationCheckpointSectionBytes) {
        let native_bytes = native.bytes();
        let accepted_bytes = accepted_outputs.iter().fold(0_usize, |total, accepted| {
            let role_bytes = accepted.roles.iter().fold(0_usize, |role_total, role| {
                role_total.saturating_add(8 + role.role.len() + 1 + 8 + role.entity_name.len() + 16)
            });
            total.saturating_add(
                MINIMUM_V5_ACCEPTED_OUTPUT_BYTES - 1
                    + accepted.producer.len()
                    + role_bytes
                    + accepted.producer_facts.as_ref().map_or(0, Vec::len),
            )
        });
        let mut body = Vec::with_capacity(BODY_PREFIX_BYTES + native_bytes.len() + accepted_bytes);
        body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        body.extend_from_slice(&publication.bootstrap_commit_id().0.to_be_bytes());
        body.extend_from_slice(&(native_bytes.len() as u64).to_be_bytes());
        body.extend_from_slice(&(accepted_outputs.len() as u64).to_be_bytes());
        body.extend_from_slice(native_bytes);
        let accepted_start = body.len();
        for accepted in accepted_outputs {
            body.extend_from_slice(&(accepted.producer.len() as u64).to_be_bytes());
            body.extend_from_slice(accepted.producer.as_bytes());
            body.extend_from_slice(&accepted.source);
            encode_scope(&mut body, accepted.scope);
            body.extend_from_slice(&accepted.source_partition);
            body.push(u8::from(accepted.producer_dependency.is_some()));
            body.extend_from_slice(&accepted.producer_dependency.unwrap_or_default());
            body.extend_from_slice(&accepted.idempotency_key);
            resources::encode_profile(&mut body, accepted.resources);
            body.extend_from_slice(&(accepted.roles.len() as u64).to_be_bytes());
            for role in &accepted.roles {
                body.extend_from_slice(&(role.role.len() as u64).to_be_bytes());
                body.extend_from_slice(role.role.as_bytes());
                body.push(match role.posture {
                    super::super::WorthQueryApplicationOutputPosture::Preserve => 0,
                    super::super::WorthQueryApplicationOutputPosture::Create => 1,
                    super::super::WorthQueryApplicationOutputPosture::Retire => 2,
                });
                body.extend_from_slice(&(role.entity_name.len() as u64).to_be_bytes());
                body.extend_from_slice(role.entity_name.as_bytes());
                encode_entity(&mut body, role.entity);
            }
            let fact_bytes = accepted.producer_facts.as_deref().unwrap_or_default();
            body.extend_from_slice(&(fact_bytes.len() as u64).to_be_bytes());
            body.extend_from_slice(fact_bytes);
        }
        let checksum = Sha256::digest(&body);
        let mut bytes = Vec::with_capacity(MAGIC.len() + CHECKSUM_BYTES + body.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&checksum);
        bytes.extend_from_slice(&body);
        let sections = WorthQueryApplicationCheckpointSectionBytes::new(
            HEADER_BYTES,
            native_bytes.len(),
            body.len() - accepted_start,
            accepted_outputs.len(),
        );
        debug_assert_eq!(sections.total_bytes(), bytes.len());
        (
            Self {
                bytes: bytes.into_boxed_slice(),
            },
            sections,
        )
    }
}

fn encode_scope(
    output: &mut Vec<u8>,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
) {
    output.extend_from_slice(&scope.partition_id().to_be_bytes());
    output.extend_from_slice(&scope.local_slot().to_be_bytes());
    output.extend_from_slice(&scope.generation().to_be_bytes());
}

fn encode_entity(output: &mut Vec<u8>, entity: worth_relational::facade::identity::EntityId) {
    output.extend_from_slice(&entity.partition_value().to_be_bytes());
    output.extend_from_slice(&entity.local_slot_value().to_be_bytes());
    output.extend_from_slice(&entity.generation_value().to_be_bytes());
}
