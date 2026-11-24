//! One request-funded immutable schema graph follows exact source readmission.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::region_directory::corrupt;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage;
use contracts::schema::data::{RelationalSchemaRegistry, RelationalSchemaRegistryFieldsPort as _};
use contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::AspectContract;

#[derive(Debug)]
pub struct ReadmittedSourceSchemaCatalog {
    pub(crate) rows: ExecutionOrderedMap<[u8; 32], Arc<RelationalBranchRootSchemaAuthority>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ReadmittedSourceSchemaCatalog {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        directory_copies: usize,
    ) -> Result<Arc<Self>, RelationalRecoveryStop> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| {
                RelationalRecoveryStop::Request(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?
            .0
            .pad_to_align();
        let directory = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                crate::durability::data::RelationalCheckpointDirectory,
            >())
            .map_err(|_| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "directory binding layout overflow",
                )
            })?
            .0
            .pad_to_align()
            .size();
        let bytes = directory
            .checked_mul(directory_copies)
            .and_then(|copies| copies.checked_add(layout.size()))
            .ok_or_else(|| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "directory binding bill overflow",
                )
            })?;
        // These attempt-local directory bindings retain this catalog, so its
        // original grant follows every actual fixed-size binding allocation.
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        Ok(Arc::new(Self {
            rows: Default::default(),
            _memory: memory,
        }))
    }
    /// Admit authenticated schema DATA before the receiving compiler runs.
    /// No missing executable format is resolved or manufactured here.
    pub(crate) fn insert_unbound_image(
        catalog: &mut Arc<Self>,
        image: &RecordedBranchSchemaImage,
        configured: &RelationalSchemaRegistry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRecoveryStop> {
        let Some((registry, contracts, expected)) =
            Self::read_unseen_image(catalog, image, configured, request)?
        else {
            return Ok(());
        };
        Self::retain_image(catalog, image, registry, contracts, &expected, request)
    }

    /// Immediate executable admission for callers with installed receiver codecs.
    pub(crate) fn insert_image(
        catalog: &mut Arc<Self>,
        image: &RecordedBranchSchemaImage,
        configured: &RelationalSchemaRegistry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRecoveryStop> {
        let Some((mut registry, contracts, expected)) =
            Self::read_unseen_image(catalog, image, configured, request)?
        else {
            return Ok(());
        };
        (*registry.owner_content_formats_mut()) = (*registry.owner_content_formats())
            .bind_recorded(configured.owner_content_formats())?;
        Self::retain_image(catalog, image, registry, contracts, &expected, request)
    }

    fn read_unseen_image(
        catalog: &Self,
        image: &RecordedBranchSchemaImage,
        configured: &RelationalSchemaRegistry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<(
            RelationalSchemaRegistry,
            Vec<AspectContract>,
            SchemaAuthoritySnapshot,
        )>,
        RelationalRecoveryStop,
    > {
        contracts::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        if let Some(prior) = catalog.rows.get(&image.carrier_digest()) {
            if prior.recorded_image(request)? != *image {
                return Err(corrupt("same source carrier has differing full schema DATA").into());
            }
            return Ok(None);
        }
        let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
        let expected = registry
            .snapshot_for_compilation(
                contracts::schema::compilation_control::SchemaCompilationControl::Request(*request),
            )
            .map_err(RelationalRecoveryStop::Request)?;
        // Reuse genuine registered recipe/domain Arcs when the exact meaning
        // matches. This does not bind absent receiving codec implementations.
        let registry = if configured
            .snapshot_for_compilation(
                contracts::schema::compilation_control::SchemaCompilationControl::Request(*request),
            )
            .map_err(RelationalRecoveryStop::Request)?
            == expected
        {
            configured.clone()
        } else {
            registry
        };
        Ok(Some((registry, contracts, expected)))
    }

    fn retain_image(
        catalog: &mut Arc<Self>,
        image: &RecordedBranchSchemaImage,
        registry: RelationalSchemaRegistry,
        contracts: Vec<AspectContract>,
        expected: &SchemaAuthoritySnapshot,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRecoveryStop> {
        let schema = RelationalBranchRootSchemaAuthority::readmit_exact(
            catalog.rows.len() as u64 + 1,
            registry,
            contracts,
            expected,
            image.descriptor_semantics_version(),
            contracts::schema::compilation_control::SchemaCompilationControl::Request(*request),
        )
        .map_err(RelationalRecoveryStop::Request)?
        .ok_or_else(|| corrupt("original schema carrier DATA admission failed"))?;
        schema
            .retain_readmitted_image(image.clone())
            .map_err(corrupt)?;
        Arc::get_mut(catalog)
            .ok_or_else(|| corrupt("source schema graph was shared before admission completed"))?
            .rows
            .insert_cloned_native_for_request(&image.carrier_digest(), &schema, *request)
            .map_err(contracts::durability::data::recovery_stop::map_stop)?;
        Ok(())
    }
}
