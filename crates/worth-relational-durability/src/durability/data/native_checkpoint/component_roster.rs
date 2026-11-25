//! Counted read-only roster of the components one capture's directory retains.
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

/// The component obligations one capture's manifest retains: its retained
/// root records, retained Empty components, every recorded component
/// descriptor those roots carry and the installed schema registries its
/// schema images readmit. Descriptive only; it admits nothing.
#[derive(Debug, Clone)]
pub struct RelationalCheckpointComponentRoster {
    pub retained_roots: usize,
    pub retained_empty_components: usize,
    pub recorded_components: Vec<worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor>,
    pub schema_registries: Vec<contracts::schema::data::RelationalSchemaRegistry>,
}

impl super::RelationalNativeCheckpoint {
    /// Decode this capture's external-member directory through `source` and
    /// count its retained component roster. `None` means the bytes carry no
    /// directory. Cost: one full checkpoint decode, charged to `execution`.
    pub fn inspect_component_roster(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
    ) -> Result<
        Option<RelationalCheckpointComponentRoster>,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        let decoded =
            crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
                execution,
                self.bytes(),
                source,
            )?;
        let checkpoint = decoded.checkpoint;
        let directory = checkpoint
            .directory
            .as_ref()
            .or(checkpoint.canonical_directory.as_ref());
        directory
            .map(|directory| {
                let manifest = &directory.manifest;
                let schema_registries = manifest
                    .schemas
                    .iter()
                    .map(|image| image.readmit_registry().map(|(registry, _)| registry))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|detail| {
                        contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(
                            contracts::durability::data::recovery_errors::DurabilityError::new(
                                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                                detail,
                            ),
                        )
                    })?;
                Ok(RelationalCheckpointComponentRoster {
                    retained_roots: manifest.roots.len(),
                    retained_empty_components: manifest.retained_empty_components.len(),
                    recorded_components: manifest
                        .roots
                        .iter()
                        .flat_map(|root| root.recorded_components.iter().cloned())
                        .collect(),
                    schema_registries,
                })
            })
            .transpose()
    }
}
