use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::data::{
    CheckpointCoverage, DurabilityError, DurableBranchRootImage, DurableCheckpoint,
    RecoveryFailureClass,
};
use crate::lineage::data::LineageCheckpointArtifact;

use super::checkpoint_capture::{CapturedCheckpointBasis, CapturedRecordIdentity};

impl CapturedCheckpointBasis {
    pub(super) fn build_checkpoint_image(self) -> Result<DurableCheckpoint, DurabilityError> {
        let CapturedCheckpointBasis {
            selected_directory: _,
            symbol_pages_resident: _,
            latest_commit,
            branch_cells,
            branch_lifecycle,
            retired_branch_names,
            covered_transitions,
            branch_roots,
            record_identity,
            envelopes,
            journal_index: _,
            envelope_cells: _,
            lineage_basis,
            commit_descriptors,
            partitions,
            aspect_contracts,
            mirror_schema_authority,
            lineage_nodes,
            index_definitions,
            carried_index_image: _,
            derived_index_checkpoint,
            symbol_table,
            symbol_identity_frontier,
            runtime_name,
        } = self;
        // The mirror retains deleted records and MVCC cells from earlier
        // revisions. Their exact contracts belong to the selected main root;
        // the current runtime's active plans cannot reinterpret those cells.
        let (mirror_plans, mirror_contracts) = match &mirror_schema_authority {
            Some(schema) => (
                schema.aspect_plans(),
                CheckpointAspectContractCatalog::from_contracts(
                    schema.retained_aspect_contracts(),
                )?,
            ),
            None => (
                &aspect_contracts,
                CheckpointAspectContractCatalog::from_plans(&aspect_contracts)?,
            ),
        };
        let record_identity = checkpoint_record_identity(record_identity)?;
        let CapturedBranchRootImages {
            roots: branch_roots,
            schemas: branch_root_schema_images,
            kinds: root_kind_images,
        } = branch_root_images(branch_roots)?;
        let mirror_partitions = partitions.materialize_owned_partitions();
        let mirror_kind_images =
            crate::storage::residency::CheckpointRegionKindImages::capture(&mirror_partitions)
                .map_err(|cause| {
                    DurabilityError::new(
                        RecoveryFailureClass::CorruptCheckpoint,
                        format!("mirror kind capture: {cause:?}"),
                    )
                })?;
        let mirror_schema_digest = mirror_schema_authority
            .as_ref()
            .map(crate::durability::data::DurableBranchRootSchemaImage::capture)
            .transpose()
            .map_err(|error| {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string())
            })?
            .map(|image| image.carrier_digest());
        Ok(DurableCheckpoint {
            directory: None,
            root_kind_images, mirror_kind_images,
            commit_descriptors,
            mirror_schema_digest,
            coverage: CheckpointCoverage {
                up_to_commit: latest_commit.clone(),
                up_to_version: latest_commit.map(|commit| commit.version_id),
                up_to_branch_transition: covered_transitions,
            },
            branch_cells,
            branch_lifecycle,
            retired_branch_names,
            branch_roots,
            branch_root_schema_images,
            record_identity,
            record_generation_high_water: Vec::new(),
            reusable_record_slots: Vec::new(),
            record_slot_frontiers: Vec::new(),
            envelopes,
            partition_images: mirror_partitions.into_iter()
                .map(|partition| partition_to_image_with_contracts(partition, mirror_plans, &mirror_contracts))
                .collect::<Result<Vec<_>, _>>()?,
            aspect_contracts: mirror_contracts.portable_contracts(),
            lineage: LineageCheckpointArtifact::new(lineage_basis, lineage_nodes),
            index_definitions,
            derived_index_artifacts: crate::indexes::data::DerivedIndexArtifacts::default(),
            derived_index_checkpoint: Some(derived_index_checkpoint),
            derived_index_checkpoint_format:
                crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
            symbol_table,
            symbol_identity_frontier,
            runtime_name,
        })
    }
}

struct CapturedBranchRootImages {
    roots: Vec<DurableBranchRootImage>,
    schemas: Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    kinds: std::collections::BTreeMap<
        crate::history::data::CommitId,
        crate::storage::residency::CheckpointRegionKindImages,
    >,
}

fn branch_root_images(
    branch_roots: Vec<crate::branch::RelationalBranchRootCheckpoint>,
) -> Result<CapturedBranchRootImages, DurabilityError> {
    let mut root_kind_images = std::collections::BTreeMap::new();
    let mut schema_images = std::collections::BTreeMap::new();
    let root_images = branch_roots
        .into_iter()
        .map(|root| {
            let schema_image = crate::durability::data::DurableBranchRootSchemaImage::capture(
                root.schema_authority(),
            )
            .map_err(|error| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    format!("branch-root schema image encoding failed: {error}"),
                )
            })?;
            let schema_carrier_digest = schema_image.carrier_digest();
            schema_images
                .entry(schema_carrier_digest)
                .or_insert(schema_image);
            let retained_contracts = CheckpointAspectContractCatalog::from_contracts(
                root.schema_authority().retained_aspect_contracts(),
            )?;
            let partitions = root.partitions();
            root_kind_images.insert(
                root.commit_id(),
                crate::storage::residency::CheckpointRegionKindImages::capture(&partitions)
                    .map_err(|cause| {
                        DurabilityError::new(
                            RecoveryFailureClass::CorruptCheckpoint,
                            format!("root kind capture: {cause:?}"),
                        )
                    })?,
            );
            let partition_images = partitions
                .iter()
                .cloned()
                .map(|partition| {
                    partition_to_image_with_contracts(
                        partition,
                        root.schema_authority().aspect_plans(),
                        &retained_contracts,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let partition_image_digest =
                crate::durability::data::branch_root_partition_image_digest(&partition_images)
                    .map_err(|error| {
                        DurabilityError::new(
                            RecoveryFailureClass::CorruptCheckpoint,
                            format!("branch-root image encoding failed: {error}"),
                        )
                    })?;
            let format_version = DurableBranchRootImage::CURRENT_FORMAT_VERSION;
            let root_image_digest = crate::durability::data::branch_root_image_digest(
                format_version,
                root.commit_id(),
                partition_image_digest,
                schema_carrier_digest,
            );
            Ok(DurableBranchRootImage {
                format_version,
                commit_id: root.commit_id(),
                partition_images,
                partition_image_digest,
                schema_carrier_digest,
                root_image_digest,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CapturedBranchRootImages {
        roots: root_images,
        schemas: schema_images.into_values().collect(),
        kinds: root_kind_images,
    })
}

fn checkpoint_record_identity(
    captured: CapturedRecordIdentity,
) -> Result<crate::durability::data::DurableRecordIdentityState, DurabilityError> {
    let generation_high_water = captured
        .generation_high_water
        .into_iter()
        .map(|(class, partition_id, slot, generation)| {
            crate::durability::data::DurableRecordGenerationHighWater {
                class: durable_record_class(class),
                partition_id,
                slot,
                generation,
            }
        })
        .collect();
    let reusable_slots = captured
        .reusable_slots
        .into_iter()
        .map(|(class, partition_id, slot)| {
            let slot = u64::try_from(slot).map_err(|_| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "record slot cannot be encoded in a durable checkpoint",
                )
            })?;
            Ok(crate::durability::data::DurableReusableRecordSlot {
                class: durable_record_class(class),
                partition_id,
                slot,
            })
        })
        .collect::<Result<Vec<_>, DurabilityError>>()?;
    let append_frontiers = captured
        .append_frontiers
        .into_iter()
        .map(|(class, partition_id, next_slot)| {
            let next_slot = u64::try_from(next_slot).map_err(|_| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "record slot frontier cannot be encoded in a durable checkpoint",
                )
            })?;
            Ok(crate::durability::data::DurableRecordSlotFrontier {
                class: durable_record_class(class),
                partition_id,
                next_slot,
            })
        })
        .collect::<Result<Vec<_>, DurabilityError>>()?;
    let pending_reservations = captured
        .pending_reservations
        .into_iter()
        .map(|(class, partition_id, slot, generation, origin)| {
            crate::durability::data::DurablePendingRecordReservation {
                class: durable_record_class(class),
                partition_id,
                slot,
                generation,
                origin: match origin {
                    crate::history::data::RecordAllocationOrigin::AppendFrontier => {
                        crate::durability::data::DurableRecordReservationOrigin::AppendFrontier
                    }
                    crate::history::data::RecordAllocationOrigin::Reclaimed {
                        prior_generation,
                    } => crate::durability::data::DurableRecordReservationOrigin::Reclaimed {
                        prior_generation,
                    },
                },
            }
        })
        .collect();
    Ok(
        crate::durability::data::DurableRecordIdentityState::current(
            generation_high_water,
            reusable_slots,
            append_frontiers,
            pending_reservations,
        ),
    )
}

fn durable_record_class(
    class: crate::history::data::RecordAllocationClass,
) -> crate::durability::data::DurableRecordGenerationClass {
    match class {
        crate::history::data::RecordAllocationClass::Entity => {
            crate::durability::data::DurableRecordGenerationClass::Entity
        }
        crate::history::data::RecordAllocationClass::Relation => {
            crate::durability::data::DurableRecordGenerationClass::Relation
        }
    }
}
