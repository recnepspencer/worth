//! Receiver protocol only. The public court proves actual correspondence issuance.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::positioned_snapshot::{PositionedRelationalSnapshotFieldsPort as _, PositionedRelationalSnapshotFixturePort as _, PositionedRelationalSnapshotParts, assemble_positioned_relational_snapshot};
use worth_relational_branch::internal_port::validation::candidate_source::image::publication::ManagedIndexBranchImageOwnerPort as _;
use super::*;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::transfer::PreparedCompanionSlotSwap;
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::primitives::VersionId;
use contracts::validation::candidate_source::program::schema_fixture::managed_cache_schema;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn source_image(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    selected: &PositionedRelationalSnapshot,
    availability: ManagedIndexSourceAvailability,
    payload: &Arc<dyn Any + Send + Sync>,
) -> Arc<ManagedIndexBranchImage> {
    let mut image = ManagedIndexBranchImage::default();
    image
        .prepare_source(
            execution,
            program,
            scope,
            availability,
            (*selected.owner_commit_id()).unwrap(),
            true,
            Some(payload),
        )
        .unwrap();
    ManagedIndexBranchImage::retain_header(execution, image).unwrap()
}

#[test]
fn managed_source_correspondence_receiver_binds_exact_selected_image_and_unavailable_payload() {
    let (_, program) = managed_cache_schema();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let scope = EntityId::new(PartitionId::main(), 0, 1);
    let other_scope = EntityId::new(PartitionId::main(), 1, 1);
    let original_payload: Arc<dyn Any + Send + Sync> = Arc::new(7u64);
    let selected = assemble_positioned_relational_snapshot(PositionedRelationalSnapshotParts {
        runtime_instance_id: 1,
        branch_id: BranchId("main".into()),
        root_id: 1,
        version_id: VersionId(1),
        commit_id: Some(CommitId(1)),
        position: Some(PatchStreamPosition(1)),
    });
    let initial = CompanionRootImage::new(
        *selected.owner_root_id(),
        *selected.owner_commit_id(),
        *selected.owner_position(),
        source_image(
            &execution,
            &program,
            scope,
            &selected,
            ManagedIndexSourceAvailability::Unavailable,
            &original_payload,
        ),
    );
    let generation = 1;
    let cell = CompanionBranchCellCore::new(
        *selected.owner_runtime_instance_id(),
        (*selected.owner_branch_id()).clone(),
        generation,
        Arc::clone(&initial),
    );
    cell.retain_source_affinity(&execution).unwrap();
    let image = ManagedSourceRootImage::capture_current(&cell).unwrap();
    let payload = image
        .corresponding_payload(&selected, generation, &program, scope)
        .unwrap()
        .unwrap();
    assert!(
        Arc::ptr_eq(&payload, &original_payload),
        "receiver shares the exact immutable payload allocation"
    );
    assert!(
        matches!(
            image.observe_if_ready(selected.clone(), &program, scope, None, generation),
            Err(Denial::RootUnavailable)
        ),
        "logical backing is never a committed physical read"
    );
    let (_, foreign_program) = managed_cache_schema();
    assert!(matches!(
        image.corresponding_payload(&selected, generation, &foreign_program, scope),
        Err(Denial::ForeignSource)
    ));
    assert!(image
        .corresponding_payload(&selected, generation, &program, other_scope)
        .unwrap()
        .is_none());
    assert!(matches!(
        image.corresponding_payload(&selected, generation + 1, &program, scope),
        Err(Denial::Publication(
            CompanionPreflightStop::RegistrationChanged
        ))
    ));
    for axis in 0..5 {
        let mut foreign = selected.clone();
        match axis {
            0 => (*foreign.owner_runtime_instance_id_mut()) += 1,
            1 => foreign.append_foreign_branch_suffix_for_test("-foreign"),
            2 => (*foreign.owner_root_id_mut()) += 1,
            3 => (*foreign.owner_commit_id_mut()) = None,
            4 => (*foreign.owner_position_mut()) = None,
            _ => unreachable!(),
        }
        assert!(
            matches!(
                image.corresponding_payload(&foreign, generation, &program, scope),
                Err(Denial::ForeignSource)
            ),
            "receiver rejects changed selected axis {axis}"
        );
    }
    let resumed_selected =
        assemble_positioned_relational_snapshot(PositionedRelationalSnapshotParts {
            runtime_instance_id: selected.runtime_instance_id(),
            branch_id: selected.branch_id().clone(),
            root_id: 2,
            version_id: VersionId(2),
            commit_id: Some(CommitId(2)),
            position: Some(PatchStreamPosition(2)),
        });
    let mut swap = PreparedCompanionSlotSwap::prepare(
        cell.reserve_for_publication().unwrap(),
        initial,
        *resumed_selected.owner_root_id(),
        (*resumed_selected.owner_commit_id()).unwrap(),
        source_image(
            &execution,
            &program,
            scope,
            &resumed_selected,
            ManagedIndexSourceAvailability::Available,
            &original_payload,
        ),
        cell.generation(),
    )
    .unwrap();
    swap.check(generation).unwrap();
    swap.enter()
        .unwrap()
        .install_at((*resumed_selected.owner_position()).unwrap());
    drop(swap);
    let resumed_image = ManagedSourceRootImage::capture_current(&cell).unwrap();
    assert!(
        matches!(
            resumed_image.corresponding_payload(&resumed_selected, generation, &program, scope),
            Err(Denial::ForeignSource)
        ),
        "an available association cannot masquerade as a suspension payload"
    );
    assert!(
        matches!(
            image.corresponding_payload(&resumed_selected, generation, &program, scope),
            Err(Denial::ForeignSource)
        ),
        "an old image cannot supply the successor payload"
    );
    assert!(
        matches!(
            image.evict_selected(&execution, &selected, &program, scope, generation),
            Err(Denial::Publication(
                CompanionPreflightStop::ManagedSourceNotPrepared
            ))
        ),
        "public cache eviction cannot edit an unavailable association"
    );
}
