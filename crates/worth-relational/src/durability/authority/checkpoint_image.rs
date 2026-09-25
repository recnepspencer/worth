use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_to_image, partition_to_image_with_contracts,
};
use crate::durability::data::{
    CheckpointCoverage, DurabilityError, DurableBranchRootImage, DurableCheckpoint,
    RecoveryFailureClass,
};
use crate::lineage::data::{LineageCheckpointArtifact, LineageCheckpointDigestBasis};

use super::checkpoint_capture::{CapturedCheckpointBasis, CapturedRecordIdentity};

impl CapturedCheckpointBasis {
    pub(super) fn build_checkpoint_image(self) -> Result<DurableCheckpoint, DurabilityError> {
        let CapturedCheckpointBasis {
            latest_commit,
            branch_cells,
            branch_roots,
            record_identity,
            envelopes,
            partitions,
            aspect_contracts,
            lineage_nodes,
            index_definitions,
            derived_index_checkpoint,
            symbol_table,
            runtime_name,
        } = self;
        let published_lineage_commit_count = envelopes
            .iter()
            .filter(|envelope| envelope.has_lineage_authority())
            .count();
        let canonical_published_event_ids = envelopes
            .iter()
            .flat_map(|envelope| {
                envelope
                    .lineage_digest_basis()
                    .canonical_event_ids()
                    .iter()
                    .copied()
            })
            .collect();
        let published_lineage_event_count = envelopes
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
            .sum();
        let published_lineage_decision_count = envelopes
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
            .sum();
        let record_identity = checkpoint_record_identity(record_identity)?;
        let (branch_roots, branch_root_schema_images) = branch_root_images(branch_roots)?;
        Ok(DurableCheckpoint {
            coverage: CheckpointCoverage {
                up_to_commit: latest_commit.clone(),
                up_to_version: latest_commit.map(|commit| commit.version_id),
            },
            branch_cells,
            branch_roots,
            branch_root_schema_images,
            record_identity,
            record_generation_high_water: Vec::new(),
            reusable_record_slots: Vec::new(),
            record_slot_frontiers: Vec::new(),
            envelopes,
            partition_images: partitions
                .into_iter()
                .map(|partition| partition_to_image(partition, &aspect_contracts))
                .collect::<Result<Vec<_>, _>>()?,
            aspect_contracts: checkpoint_aspect_contracts(&aspect_contracts)?,
            lineage: LineageCheckpointArtifact::new(
                LineageCheckpointDigestBasis::new(
                    published_lineage_commit_count,
                    canonical_published_event_ids,
                    published_lineage_event_count,
                    published_lineage_decision_count,
                ),
                lineage_nodes,
            ),
            index_definitions,
            derived_index_artifacts: crate::indexes::data::DerivedIndexArtifacts::default(),
            derived_index_checkpoint: Some(derived_index_checkpoint),
            derived_index_checkpoint_format:
                crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
            symbol_table,
            runtime_name,
        })
    }
}

fn branch_root_images(
    branch_roots: Vec<crate::branch::RelationalBranchRootCheckpoint>,
) -> Result<
    (
        Vec<DurableBranchRootImage>,
        Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    ),
    DurabilityError,
> {
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
            let partition_images = root
                .partitions()
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
    Ok((root_images, schema_images.into_values().collect()))
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

fn checkpoint_aspect_contracts(
    plans: &crate::schema::data::AspectContractPlanCatalog,
) -> Result<Vec<worth_foundational::facade::PortableAspectContract>, DurabilityError> {
    let mut contracts = std::collections::BTreeMap::new();
    for binding in plans
        .entity_plans
        .values()
        .chain(plans.relation_plans.values())
        .flat_map(|plan| plan.executable_bindings.iter())
    {
        let candidate =
            worth_foundational::facade::PortableAspectContract::from_contract(&binding.contract);
        match contracts.get(candidate.key()) {
            Some(existing) if existing == &candidate => continue,
            Some(_) => {
                return Err(DurabilityError::new(
                    RecoveryFailureClass::SchemaMismatch,
                    format!(
                        "checkpoint has conflicting active contracts for aspect `{}`",
                        candidate.key().as_str()
                    ),
                ));
            }
            None => {
                contracts.insert(candidate.key().clone(), candidate);
            }
        }
    }
    Ok(contracts.into_values().collect())
}
