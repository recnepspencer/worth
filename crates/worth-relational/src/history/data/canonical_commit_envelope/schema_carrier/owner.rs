//! Immutable schema-image DATA shares its actual allocation and admission.
use super::*;
use serde::{Deserialize, Deserializer};

impl RecordedBranchSchemaImage {
    pub(crate) fn capture_for_request(
        source: SchemaImageSource<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::recovery_stop::RelationalRecoveryStop> {
        use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
        use crate::durability::data::recovery_stop::RelationalRecoveryStop;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let bytes = super::capture_memory::quote(source, execution)?;
        let memory = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        Self::capture_owned(source, memory).map_err(|error| {
            DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string()).into()
        })
    }

    pub(crate) fn capture(source: SchemaImageSource<'_>) -> Result<Self, rmp_serde::encode::Error> {
        Self::capture_owned(source, None)
    }

    fn capture_owned(
        source: SchemaImageSource<'_>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<Self, rmp_serde::encode::Error> {
        let registry = source.registry;
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
            schema_root: source.schema_root,
            descriptor_semantics_version: source.descriptor_semantics_version,
            entity_registrations,
            relation_registrations,
            derived_relations: registry.derived_parts(),
            projected_member_domains: registry.projected_domain_parts(),
            content_formats: registry.content_formats.snapshot(),
            persistent_name_schemes: registry.persistent_name_scheme_snapshots(),
            kind_name_schemes: registry.kind_name_scheme_snapshots(),
            _memory: memory,
            retained_aspect_contracts: source
                .retained_aspect_contracts
                .iter()
                .map(PortableAspectContract::from_contract)
                .collect(),
        };
        image.carrier_digest = image.recompute_digest()?;
        Ok(Self(std::sync::Arc::new(image)))
    }
}

impl PartialEq for RecordedBranchSchemaImage {
    fn eq(&self, other: &Self) -> bool {
        if std::sync::Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
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
            && left.content_formats == right.content_formats
            && left.persistent_name_schemes == right.persistent_name_schemes
            && left.kind_name_schemes == right.kind_name_schemes
    }
}
impl Eq for RecordedBranchSchemaImage {}

impl<'de> Deserialize<'de> for RecordedBranchSchemaImage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        SchemaImageData::deserialize(deserializer).map(|data| Self(std::sync::Arc::new(data)))
    }
}
