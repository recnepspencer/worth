//! Counted read-only roster of the components one capture's directory retains.
use std::sync::Arc;

/// The component obligations one capture's manifest retains: its retained
/// root records, retained Empty components, every recorded component
/// descriptor those roots carry and the installed schema registries its
/// schema images readmit. Descriptive only; it admits nothing.
#[derive(Debug, Clone)]
pub struct RelationalCheckpointComponentRoster {
    pub retained_roots: usize,
    pub retained_empty_components: usize,
    pub recorded_components: Vec<crate::branch::RelationalBranchBasisDescriptor>,
    pub schema_registries: Vec<crate::schema::data::RelationalSchemaRegistry>,
}

impl super::RelationalNativeCheckpoint {
    /// Decode this capture's directory (deferred or Embedded) through `source`
    /// (its external members' read port; `None` for an Embedded capture) and
    /// count its retained component roster. `None` means the bytes carry no
    /// directory. Cost: one full checkpoint decode, charged to `execution`.
    pub fn inspect_component_roster(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    ) -> Result<
        Option<RelationalCheckpointComponentRoster>,
        crate::durability::data::RelationalRecoveryStop,
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
                        crate::durability::data::RelationalRecoveryStop::Durability(
                            crate::durability::data::DurabilityError::new(
                                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
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
