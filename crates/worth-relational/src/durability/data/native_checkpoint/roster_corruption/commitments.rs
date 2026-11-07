//! Re-record content descriptors through the actual immutable-region owner.
use crate::{branch::*, durability::data::*};
use worth_foundational::facade::FoundationalBranchTarget;

pub(super) fn rebind(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &mut DurableCheckpoint,
) -> Result<(), RelationalRecoveryStop> {
    let mut symbols = crate::symbols::data::StringInterner::default();
    symbols.restore_snapshot(checkpoint.symbol_table.clone());
    for root in &checkpoint.branch_roots {
        checkpoint
            .root_kind_images
            .get_mut(&root.commit_id)
            .ok_or_else(|| invalid("missing kind images"))?
            .omit_deleted_current_records(&root.partition_images);
        let image = checkpoint
            .branch_root_schema_images
            .iter()
            .find(|image| image.carrier_digest() == root.schema_carrier_digest)
            .ok_or_else(|| invalid("corruption recorder lost the original schema"))?;
        let (registry, contracts) = image.readmit_registry().map_err(invalid)?;
        let expected = registry.authority_snapshot();
        let authority = RelationalBranchRootSchemaAuthority::readmit_exact(
            1,
            registry,
            contracts,
            &expected,
            image.descriptor_semantics_version(),
        )
        .ok_or_else(|| invalid("schema readmission failed"))?;
        let catalog=crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::from_contracts(authority.retained_aspect_contracts())?;
        let mut partitions = std::collections::BTreeMap::new();
        for image in &root.partition_images {
            let partition = crate::durability::checkpoints::images::partition_from_image(
                image.clone(),
                authority.aspect_plans(),
                &catalog,
            )?;
            partitions.insert(partition.partition_id, partition);
        }
        checkpoint
            .root_kind_images
            .get(&root.commit_id)
            .ok_or_else(|| invalid("missing original kind authority"))?
            .restore(lane, execution, &mut partitions)
            .map_err(|cause| {
                crate::durability::checkpoints::kind_reconstruction::stop(
                    "kind reconstruction",
                    cause,
                )
            })?;
        let truth = RelationalBranchRoot::reconstruct_storage_root_for_test(
            lane, execution, partitions, &symbols,
        )
        .map_err(|cause| match cause {
            RelationalBranchRootCaptureDenial::Execution(cause) => {
                RelationalRecoveryStop::Request(cause)
            }
            other => invalid(format!("{other:?}")).into(),
        })?;
        let original = checkpoint
            .commit_descriptors
            .get(&root.commit_id)
            .ok_or_else(|| invalid("missing original descriptor"))?;
        checkpoint.commit_descriptors.insert(
            root.commit_id,
            RelationalBranchRootDescriptor::new(truth, *original.schema_root()),
        );
    }
    for cell in &mut checkpoint.branch_cells {
        rebind_observation(&mut cell.observation, &checkpoint.commit_descriptors);
        if let Some(source) = &mut cell.fork_provenance {
            rebind_observation(source, &checkpoint.commit_descriptors);
        }
    }
    Ok(())
}
fn rebind_observation(
    observation: &mut RelationalBranchReferenceObservation,
    descriptors: &std::collections::BTreeMap<
        crate::history::data::CommitId,
        RelationalBranchRootDescriptor,
    >,
) {
    let FoundationalBranchTarget::Basis(target) = observation.target() else {
        return;
    };
    let Some(roots) = descriptors.get(&crate::history::data::CommitId(target.selected_commit_id()))
    else {
        return;
    };
    let target = RelationalBranchTarget::new(
        target.runtime_instance_id(),
        target.selected_commit_id(),
        target.version_id(),
        target.parent_commit_ids().to_vec(),
        roots.clone(),
    );
    *observation = RelationalBranchReferenceObservation::new(
        observation.branch_id().clone(),
        FoundationalBranchTarget::Basis(target),
        observation.generation(),
    );
}
fn invalid(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
