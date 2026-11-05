//! Reopen one actual selected image through its original protected provider.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;

pub(crate) fn recover(
    image: &CompleteRelationalNativeCheckpoint,
    source: Arc<dyn RelationalDurableColdSource>,
    mut receiver: RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> RelationalRuntime {
    let cut = RelationalDurableReadProtection::capture(&source, request).unwrap();
    let mut canonical = ExecutionRequestArrayBuilder::allocate(
        *request,
        (cut.bounds().canonical.0 - image.source_artifact().covered().0) as usize,
    )
    .unwrap();
    for position in image.source_artifact().covered().0 + 1..=cut.bounds().canonical.0 {
        canonical
            .push(
                cut.read_commit_owned_at(
                    &source,
                    request,
                    worth_relational::facade::publication::PatchStreamPosition(position),
                    1 << 30,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut transitions = ExecutionRequestArrayBuilder::allocate(
        *request,
        (cut.bounds().transitions.0 - image.source_artifact().covered_transitions().0) as usize,
    )
    .unwrap();
    for sequence in image.source_artifact().covered_transitions().0 + 1..=cut.bounds().transitions.0
    {
        transitions
            .push(
                cut.read_transition_owned_at(
                    &source,
                    request,
                    worth_relational::facade::branch::RelationalBranchTransitionSequence(sequence),
                    1 << 30,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let initial = cut
        .read_initial_inputs_owned(&source, request, 1 << 30)
        .unwrap();
    let input = RelationalDurableRecoverySource::from_home_original_bytes(
        Some(home_image_checkpoint(image, image.source_artifact().clone(), request).unwrap()),
        initial,
        canonical.seal().unwrap(),
        transitions.seal().unwrap(),
        cut.bounds(),
        source,
        request,
    )
    .unwrap();
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = receiver
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    drop(authority);
    receiver
}

pub(crate) fn member(
    image: &CompleteRelationalNativeCheckpoint,
    partition: PartitionId,
    history: bool,
) -> RelationalDurableMemberLocator {
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: Manifest,
    }
    #[derive(serde::Deserialize)]
    struct Manifest {
        mirror: Vec<Region>,
    }
    #[derive(serde::Deserialize)]
    struct Region {
        partition_id: PartitionId,
        head: RelationalDurableMemberLocator,
        history: RelationalDurableMemberLocator,
    }
    let input: Input = rmp_serde::from_slice(
        image
            .checkpoint()
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK18\n")
            .unwrap(),
    )
    .unwrap();
    let region = input
        .manifest
        .mirror
        .into_iter()
        .find(|row| row.partition_id == partition)
        .unwrap();
    if history {
        region.history
    } else {
        region.head
    }
}
