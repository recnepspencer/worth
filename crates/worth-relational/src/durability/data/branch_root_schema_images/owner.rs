//! Immutable schema-image DATA shares its actual allocation and admission.
use super::*;
use serde::{Deserialize, Deserializer};

impl DurableBranchRootSchemaImage {
    pub(crate) fn capture_for_request(
        authority: &crate::branch::RelationalBranchRootSchemaAuthority,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::RelationalRecoveryStop> {
        use crate::durability::data::{
            DurabilityError, RecoveryFailureClass, RelationalRecoveryStop,
        };
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let bytes = super::capture_memory::quote(authority, execution)?;
        let memory = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        Self::capture_owned(authority, memory).map_err(|error| {
            DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string()).into()
        })
    }

    pub(crate) fn capture(
        authority: &crate::branch::RelationalBranchRootSchemaAuthority,
    ) -> Result<Self, rmp_serde::encode::Error> {
        Self::capture_owned(authority, None)
    }

    fn capture_owned(
        authority: &crate::branch::RelationalBranchRootSchemaAuthority,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<Self, rmp_serde::encode::Error> {
        let registry = authority.registry();
        let entity_registrations = registry
            .entity_kinds
            .values()
            .map(DurableEntitySchemaRegistration::capture)
            .collect();
        let relation_registrations = registry
            .relation_kinds
            .values()
            .map(DurableRelationSchemaRegistration::capture)
            .collect();
        let mut image = SchemaImageData {
            format_version: ROOT_SCHEMA_IMAGE_VERSION,
            carrier_digest: [0; 32],
            schema_root: authority.authority_digest(),
            descriptor_semantics_version: authority.descriptor_semantics_version(),
            entity_registrations,
            relation_registrations,
            derived_relations: registry.derived_parts(),
            projected_member_domains: registry.projected_domain_parts(),
            _memory: memory,
            retained_aspect_contracts: authority
                .retained_aspect_contracts()
                .iter()
                .map(PortableAspectContract::from_contract)
                .collect(),
        };
        image.carrier_digest = image.recompute_digest()?;
        Ok(Self(std::sync::Arc::new(image)))
    }
}

impl PartialEq for DurableBranchRootSchemaImage {
    fn eq(&self, other: &Self) -> bool {
        let (left, right) = (&self.0, &other.0);
        left.format_version == right.format_version
            && left.carrier_digest == right.carrier_digest
            && left.schema_root == right.schema_root
            && left.descriptor_semantics_version == right.descriptor_semantics_version
            && left.entity_registrations == right.entity_registrations
            && left.relation_registrations == right.relation_registrations
            && left.retained_aspect_contracts == right.retained_aspect_contracts
            && left.derived_relations == right.derived_relations
            && left.projected_member_domains == right.projected_member_domains
    }
}
impl Eq for DurableBranchRootSchemaImage {}

impl<'de> Deserialize<'de> for DurableBranchRootSchemaImage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        SchemaImageData::deserialize(deserializer).map(|data| Self(std::sync::Arc::new(data)))
    }
}
